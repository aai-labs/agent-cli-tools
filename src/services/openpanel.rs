use reqwest::Method;
use serde_json::{json, Value};

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::ApiClient,
    input,
    services::{
        generic_request,
        shared::{enc, openpanel_base, openpanel_project_id, CtxProfile},
    },
};

const SERVICE: &str = "openpanel";

pub(crate) async fn dispatch(
    client: &ApiClient,
    ctx: &Context,
    command: OpenpanelCommand,
) -> Result<Value, AppError> {
    match command.resource {
        OpenpanelResource::Projects(command) => projects(client, ctx, command).await,
        OpenpanelResource::Events(command) => events(client, ctx, command).await,
        OpenpanelResource::Insights(command) => insights(client, ctx, command).await,
        OpenpanelResource::Profiles(command) => profiles(client, ctx, command).await,
        OpenpanelResource::Request(args) => {
            generic_request::dispatch(client, ctx, SERVICE, openpanel_base(ctx.profile()), args)
                .await
        }
    }
}

async fn projects(
    client: &ApiClient,
    ctx: &Context,
    command: OpenpanelProjectsCommand,
) -> Result<Value, AppError> {
    match command.action {
        OpenpanelProjectsAction::List => {
            get(
                client,
                ctx,
                "projects.list",
                "/manage/projects",
                Query::new(),
            )
            .await
        }
        OpenpanelProjectsAction::Get(args) => {
            get(
                client,
                ctx,
                "projects.get",
                &format!("/manage/projects/{}", enc(&args.project_id)),
                Query::new(),
            )
            .await
        }
    }
}

async fn events(
    client: &ApiClient,
    ctx: &Context,
    command: OpenpanelEventsCommand,
) -> Result<Value, AppError> {
    match command.action {
        OpenpanelEventsAction::Export(args) => events_export(client, ctx, args).await,
    }
}

/// `/export/events` paginates with a 1-indexed `page` + `limit` (per-page size, capped at
/// 1000 by the provider), and reports `meta.totalCount` - not a cursor. We loop pages here
/// so `--limit` reads as "total rows wanted", same contract as every other list command.
async fn events_export(
    client: &ApiClient,
    ctx: &Context,
    args: OpenpanelEventsExport,
) -> Result<Value, AppError> {
    let operation = "events.export";
    if args.limit == 0 {
        return Ok(json!({
            "meta": { "count": 0, "totalCount": 0, "pages": 0, "current": 0 },
            "data": [],
            "has_more": false,
        }));
    }

    let mut base_query = Query::new();
    base_query.push("projectId", args.project_id.as_deref());
    base_query.push_list("event", &args.event);
    base_query.push("profileId", args.profile_id.as_deref());
    base_query.push("start", args.start.as_deref());
    base_query.push("end", args.end.as_deref());
    base_query.push("includes", args.includes.as_deref());
    base_query.push("filters", args.filters.as_deref());

    let page_size = args.limit.clamp(1, 1000);
    let mut page_num = 1u64;
    let mut last_page = json!({});
    let mut values = Vec::new();

    loop {
        let mut query = base_query.clone();
        query.set("page", page_num.to_string());
        query.set("limit", page_size.to_string());
        let mut url = format!("{}/export/events", openpanel_base(ctx.profile()));
        query.append_to(&mut url);
        let page = request(client, ctx, operation, Method::GET, url, None).await?;

        let page_values = page
            .get("data")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let returned = page_values.len();
        last_page = page;
        for value in page_values {
            if values.len() >= args.limit as usize {
                break;
            }
            values.push(value);
        }
        if values.len() >= args.limit as usize || returned < page_size as usize {
            break;
        }
        page_num += 1;
    }

    let total_count = last_page
        .pointer("/meta/totalCount")
        .and_then(Value::as_u64);
    let has_more = total_count
        .map(|total| (values.len() as u64) < total)
        .unwrap_or(false);
    let response = input::ensure_object(&mut last_page);
    response.insert("data".to_string(), Value::Array(values));
    response.insert("has_more".to_string(), json!(has_more));
    Ok(last_page)
}

async fn insights(
    client: &ApiClient,
    ctx: &Context,
    command: OpenpanelInsightsCommand,
) -> Result<Value, AppError> {
    match command.action {
        OpenpanelInsightsAction::Metrics(args) => insights_metrics(client, ctx, args).await,
        OpenpanelInsightsAction::Pages(args) => insights_pages(client, ctx, args).await,
        OpenpanelInsightsAction::Referrers(args) => insights_referrers(client, ctx, args).await,
        OpenpanelInsightsAction::Devices(args) => insights_devices(client, ctx, args).await,
        OpenpanelInsightsAction::Geo(args) => insights_geo(client, ctx, args).await,
    }
}

fn date_range_query(args: &OpenpanelDateRangeArgs) -> Query {
    let mut query = Query::new();
    query.push("startDate", args.start_date.as_deref());
    query.push("endDate", args.end_date.as_deref());
    query.push("range", args.range.as_deref());
    query.push("filters", args.filters.as_deref());
    query
}

async fn insights_metrics(
    client: &ApiClient,
    ctx: &Context,
    args: OpenpanelDateRangeArgs,
) -> Result<Value, AppError> {
    let operation = "insights.metrics";
    let project_id =
        openpanel_project_id(ctx.profile(), args.project_id.as_deref(), operation)?.to_string();
    let query = date_range_query(&args);
    get(
        client,
        ctx,
        operation,
        &format!("/insights/{}/metrics", enc(&project_id)),
        query,
    )
    .await
}

async fn insights_pages(
    client: &ApiClient,
    ctx: &Context,
    args: OpenpanelBreakdownListArgs,
) -> Result<Value, AppError> {
    let operation = "insights.pages";
    let project_id = openpanel_project_id(
        ctx.profile(),
        args.date_range.project_id.as_deref(),
        operation,
    )?
    .to_string();
    let query = breakdown_query(&args);
    get(
        client,
        ctx,
        operation,
        &format!("/insights/{}/pages", enc(&project_id)),
        query,
    )
    .await
}

fn breakdown_query(args: &OpenpanelBreakdownListArgs) -> Query {
    let mut query = date_range_query(&args.date_range);
    query.push(
        "cursor",
        args.cursor.map(|value| value.to_string()).as_deref(),
    );
    query.set("limit", args.limit.to_string());
    query
}

async fn insights_referrers(
    client: &ApiClient,
    ctx: &Context,
    args: OpenpanelReferrersArgs,
) -> Result<Value, AppError> {
    let operation = "insights.referrers";
    let project_id = openpanel_project_id(
        ctx.profile(),
        args.list.date_range.project_id.as_deref(),
        operation,
    )?
    .to_string();
    let query = breakdown_query(&args.list);
    get(
        client,
        ctx,
        operation,
        &format!(
            "/insights/{}/{}",
            enc(&project_id),
            referrer_column(args.breakdown)
        ),
        query,
    )
    .await
}

async fn insights_devices(
    client: &ApiClient,
    ctx: &Context,
    args: OpenpanelDevicesArgs,
) -> Result<Value, AppError> {
    let operation = "insights.devices";
    let project_id = openpanel_project_id(
        ctx.profile(),
        args.list.date_range.project_id.as_deref(),
        operation,
    )?
    .to_string();
    let query = breakdown_query(&args.list);
    get(
        client,
        ctx,
        operation,
        &format!(
            "/insights/{}/{}",
            enc(&project_id),
            device_column(args.breakdown)
        ),
        query,
    )
    .await
}

async fn insights_geo(
    client: &ApiClient,
    ctx: &Context,
    args: OpenpanelGeoArgs,
) -> Result<Value, AppError> {
    let operation = "insights.geo";
    let project_id = openpanel_project_id(
        ctx.profile(),
        args.list.date_range.project_id.as_deref(),
        operation,
    )?
    .to_string();
    let query = breakdown_query(&args.list);
    get(
        client,
        ctx,
        operation,
        &format!(
            "/insights/{}/{}",
            enc(&project_id),
            geo_column(args.breakdown)
        ),
        query,
    )
    .await
}

fn referrer_column(value: OpenpanelReferrerBreakdown) -> &'static str {
    match value {
        OpenpanelReferrerBreakdown::Referrer => "referrer",
        OpenpanelReferrerBreakdown::ReferrerName => "referrer_name",
        OpenpanelReferrerBreakdown::ReferrerType => "referrer_type",
        OpenpanelReferrerBreakdown::UtmSource => "utm_source",
        OpenpanelReferrerBreakdown::UtmMedium => "utm_medium",
        OpenpanelReferrerBreakdown::UtmCampaign => "utm_campaign",
        OpenpanelReferrerBreakdown::UtmTerm => "utm_term",
        OpenpanelReferrerBreakdown::UtmContent => "utm_content",
    }
}

fn device_column(value: OpenpanelDeviceBreakdown) -> &'static str {
    match value {
        OpenpanelDeviceBreakdown::Device => "device",
        OpenpanelDeviceBreakdown::Brand => "brand",
        OpenpanelDeviceBreakdown::Model => "model",
        OpenpanelDeviceBreakdown::Browser => "browser",
        OpenpanelDeviceBreakdown::BrowserVersion => "browser_version",
        OpenpanelDeviceBreakdown::Os => "os",
        OpenpanelDeviceBreakdown::OsVersion => "os_version",
    }
}

fn geo_column(value: OpenpanelGeoBreakdown) -> &'static str {
    match value {
        OpenpanelGeoBreakdown::Country => "country",
        OpenpanelGeoBreakdown::Region => "region",
        OpenpanelGeoBreakdown::City => "city",
    }
}

async fn profiles(
    client: &ApiClient,
    ctx: &Context,
    command: OpenpanelProfilesCommand,
) -> Result<Value, AppError> {
    match command.action {
        OpenpanelProfilesAction::List(args) => profiles_list(client, ctx, args).await,
        OpenpanelProfilesAction::Get(args) => profiles_get(client, ctx, args).await,
    }
}

async fn profiles_list(
    client: &ApiClient,
    ctx: &Context,
    args: OpenpanelProfilesList,
) -> Result<Value, AppError> {
    let operation = "profiles.list";
    let project_id =
        openpanel_project_id(ctx.profile(), args.project_id.as_deref(), operation)?.to_string();
    let mut query = Query::new();
    query.push("name", args.name.as_deref());
    query.push("email", args.email.as_deref());
    query.push("country", args.country.as_deref());
    query.push("city", args.city.as_deref());
    query.push("device", args.device.as_deref());
    query.push("browser", args.browser.as_deref());
    query.push(
        "inactiveDays",
        args.inactive_days.map(|value| value.to_string()).as_deref(),
    );
    query.push(
        "minSessions",
        args.min_sessions.map(|value| value.to_string()).as_deref(),
    );
    query.push("performedEvent", args.performed_event.as_deref());
    query.push("filters", args.filters.as_deref());
    query.set("sortOrder", sort_order(args.sort_order).to_string());
    query.set("limit", args.limit.to_string());
    get(
        client,
        ctx,
        operation,
        &format!("/insights/{}/profiles", enc(&project_id)),
        query,
    )
    .await
}

fn sort_order(value: OpenpanelSortOrder) -> &'static str {
    match value {
        OpenpanelSortOrder::Asc => "asc",
        OpenpanelSortOrder::Desc => "desc",
    }
}

async fn profiles_get(
    client: &ApiClient,
    ctx: &Context,
    args: OpenpanelProfileGet,
) -> Result<Value, AppError> {
    let operation = "profiles.get";
    let project_id =
        openpanel_project_id(ctx.profile(), args.project_id.as_deref(), operation)?.to_string();
    let mut query = Query::new();
    query.set("eventLimit", args.event_limit.to_string());
    get(
        client,
        ctx,
        operation,
        &format!(
            "/insights/{}/profiles/{}",
            enc(&project_id),
            enc(&args.profile_id)
        ),
        query,
    )
    .await
}

async fn request(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    method: Method,
    url: String,
    body: Option<Value>,
) -> Result<Value, AppError> {
    client
        .request(SERVICE, operation, ctx.profile(), method, url, body)
        .await
}

async fn get(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    path: &str,
    query: Query,
) -> Result<Value, AppError> {
    let mut url = format!("{}{}", openpanel_base(ctx.profile()), path);
    query.append_to(&mut url);
    request(client, ctx, operation, Method::GET, url, None).await
}

#[derive(Clone, Debug, Default)]
struct Query(Vec<(String, String)>);

impl Query {
    fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, key: &str, value: Option<&str>) {
        if let Some(value) = value.filter(|value| !value.is_empty()) {
            self.0.push((key.to_string(), value.to_string()));
        }
    }

    fn push_list(&mut self, key: &str, values: &[String]) {
        for value in values.iter().filter(|value| !value.is_empty()) {
            self.0.push((key.to_string(), value.clone()));
        }
    }

    fn set(&mut self, key: &str, value: String) {
        self.0.retain(|(existing, _)| existing != key);
        self.0.push((key.to_string(), value));
    }

    fn append_to(&self, url: &mut String) {
        for (index, (key, value)) in self.0.iter().enumerate() {
            url.push(if index == 0 { '?' } else { '&' });
            url.push_str(&enc(key));
            url.push('=');
            url.push_str(&enc(value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Profile;

    #[test]
    fn openpanel_base_defaults_to_api_host() {
        assert_eq!(
            openpanel_base(&Profile::default()),
            "https://api.openpanel.dev"
        );
    }

    #[test]
    fn openpanel_project_id_falls_back_to_profile() {
        let profile = Profile {
            project_id: Some("proj_from_profile".to_string()),
            ..Profile::default()
        };
        assert_eq!(
            openpanel_project_id(&profile, None, "insights.metrics").unwrap(),
            "proj_from_profile"
        );
        assert_eq!(
            openpanel_project_id(&profile, Some("proj_from_arg"), "insights.metrics").unwrap(),
            "proj_from_arg"
        );
    }

    #[test]
    fn openpanel_project_id_errors_when_missing() {
        let error =
            openpanel_project_id(&Profile::default(), None, "insights.metrics").unwrap_err();
        assert_eq!(error.code, "invalid_input");
        assert!(error.message.contains("--project-id"));
    }

    #[test]
    fn referrer_column_maps_to_snake_case() {
        assert_eq!(
            referrer_column(OpenpanelReferrerBreakdown::UtmSource),
            "utm_source"
        );
        assert_eq!(
            referrer_column(OpenpanelReferrerBreakdown::ReferrerName),
            "referrer_name"
        );
    }

    #[test]
    fn device_column_maps_to_snake_case() {
        assert_eq!(
            device_column(OpenpanelDeviceBreakdown::BrowserVersion),
            "browser_version"
        );
        assert_eq!(device_column(OpenpanelDeviceBreakdown::Os), "os");
    }

    #[test]
    fn query_set_replaces_existing_key() {
        let mut query = Query::new();
        query.set("limit", "10".to_string());
        query.set("limit", "20".to_string());
        let mut url = "https://example.test/items".to_string();
        query.append_to(&mut url);
        assert_eq!(url, "https://example.test/items?limit=20");
    }

    #[test]
    fn query_push_list_repeats_key() {
        let mut query = Query::new();
        query.push_list("event", &["signup".to_string(), "purchase".to_string()]);
        let mut url = "https://example.test/export/events".to_string();
        query.append_to(&mut url);
        assert_eq!(
            url,
            "https://example.test/export/events?event=signup&event=purchase"
        );
    }
}
