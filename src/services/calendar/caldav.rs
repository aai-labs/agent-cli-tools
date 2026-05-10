use reqwest::Method;
use serde_json::{json, Value};

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    input,
    services::shared::{enc, CtxProfile},
};

pub(crate) async fn events(ctx: &Context, action: CalendarEventsAction) -> Result<Value, AppError> {
    match action {
        CalendarEventsAction::List(_) => report(ctx).await,
        CalendarEventsAction::Get(args) => get(ctx, &args.id).await,
        CalendarEventsAction::Create(args) => create(ctx, args).await,
        CalendarEventsAction::Update(args) => update(ctx, args).await,
        CalendarEventsAction::Delete(args) => delete(ctx, &args.id).await,
    }
}

async fn report(ctx: &Context) -> Result<Value, AppError> {
    let body = r#"<?xml version="1.0" encoding="utf-8" ?>
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop><d:getetag /><c:calendar-data /></d:prop>
  <c:filter><c:comp-filter name="VCALENDAR"><c:comp-filter name="VEVENT"/></c:comp-filter></c:filter>
</c:calendar-query>"#;
    request(
        ctx,
        "events.list",
        Method::from_bytes(b"REPORT").unwrap(),
        None,
        Some(body.to_string()),
    )
    .await
}

async fn get(ctx: &Context, id: &str) -> Result<Value, AppError> {
    request(ctx, "events.get", Method::GET, Some(id), None).await
}

async fn create(ctx: &Context, args: CalendarEventCreate) -> Result<Value, AppError> {
    let id = format!("aai-cli-{}.ics", timestamp());
    let body = ics(
        &id,
        args.summary.as_deref().unwrap_or("aai-cli event"),
        args.description.as_deref().unwrap_or(""),
        args.start.as_deref().unwrap_or("20300101T100000Z"),
        args.end.as_deref().unwrap_or("20300101T103000Z"),
    );
    let response = request(ctx, "events.create", Method::PUT, Some(&id), Some(body)).await?;
    Ok(json!({
        "transport": "caldav",
        "id": id,
        "response": response,
    }))
}

async fn update(ctx: &Context, args: CalendarEventUpdate) -> Result<Value, AppError> {
    let mut value = input::read_json_arg("calendar", "events.update", args.json.as_deref())?;
    input::set_string(&mut value, "summary", &args.summary);
    input::set_string(&mut value, "description", &args.description);
    let summary = value["summary"].as_str().unwrap_or("aai-cli event");
    let description = value["description"].as_str().unwrap_or("");
    let start = args.start.as_deref().unwrap_or("20300101T100000Z");
    let end = args.end.as_deref().unwrap_or("20300101T103000Z");
    let body = ics(&args.id, summary, description, start, end);
    request(
        ctx,
        "events.update",
        Method::PUT,
        Some(&args.id),
        Some(body),
    )
    .await
}

async fn delete(ctx: &Context, id: &str) -> Result<Value, AppError> {
    request(ctx, "events.delete", Method::DELETE, Some(id), None).await
}

async fn request(
    ctx: &Context,
    operation: &'static str,
    method: Method,
    id: Option<&str>,
    body: Option<String>,
) -> Result<Value, AppError> {
    let collection_url = ctx
        .profile()
        .caldav_url
        .as_deref()
        .ok_or_else(|| {
            AppError::service_config("calendar", operation, "profile.caldav_url is required")
        })?
        .trim();
    let url = if let Some(id) = id {
        format!("{}/{}", collection_url.trim_end_matches('/'), enc(id))
    } else if collection_url.ends_with('/') {
        collection_url.to_string()
    } else {
        format!("{collection_url}/")
    };
    let username = ctx
        .profile()
        .username
        .as_deref()
        .or(ctx.profile().email.as_deref())
        .ok_or_else(|| {
            AppError::auth(
                "calendar",
                operation,
                "profile is missing username or email",
            )
        })?;
    let password = ctx
        .profile()
        .password
        .as_deref()
        .or(ctx.profile().token.as_deref())
        .or(ctx.profile().api_token.as_deref())
        .map(|password| normalize_app_password(ctx, password))
        .ok_or_else(|| AppError::auth("calendar", operation, "profile is missing password"))?;

    let client = reqwest::Client::builder()
        .http1_only()
        .build()
        .map_err(|err| AppError::internal("calendar", operation, err.to_string()))?;
    let mut request = client
        .request(method, url)
        .basic_auth(username, Some(password));
    if operation == "events.list" {
        request = request
            .header("Depth", "1")
            .header("Content-Type", "application/xml");
    }
    if let Some(body) = body {
        request = request.header("Content-Type", "text/calendar").body(body);
    }
    let response = request
        .send()
        .await
        .map_err(|err| AppError::internal("calendar", operation, err.to_string()))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|err| AppError::internal("calendar", operation, err.to_string()))?;
    if status.is_success() {
        Ok(json!({
            "transport": "caldav",
            "status": status.as_u16(),
            "raw": text,
        }))
    } else {
        Err(AppError::api(
            "calendar",
            operation,
            status,
            format!("CalDAV returned HTTP {}", status.as_u16()),
            Some(json!({ "raw": text })),
        ))
    }
}

fn ics(id: &str, summary: &str, description: &str, start: &str, end: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//aai-cli//EN\r\nBEGIN:VEVENT\r\nUID:{id}\r\nDTSTAMP:{now}\r\nDTSTART:{start}\r\nDTEND:{end}\r\nSUMMARY:{summary}\r\nDESCRIPTION:{description}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        now = "20300101T000000Z",
        summary = escape_ics(summary),
        description = escape_ics(description)
    )
}

fn escape_ics(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
}

fn normalize_app_password(ctx: &Context, value: &str) -> String {
    if matches!(
        ctx.profile().auth_type.as_deref(),
        Some("app_password" | "app-password" | "caldav_password" | "caldav-password")
    ) {
        value.split_whitespace().collect()
    } else {
        value.to_string()
    }
}

fn timestamp() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_millis()
}
