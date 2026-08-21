use reqwest::Method;
use serde_json::{json, Value};

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::ApiClient,
    services::shared::{posthog_base, posthog_project_id, CtxProfile},
};

pub(crate) async fn dispatch(
    client: &ApiClient,
    ctx: &Context,
    command: PosthogCommand,
) -> Result<Value, AppError> {
    match command.resource {
        PosthogResource::Projects(command) => projects(client, ctx, command).await,
        PosthogResource::Events(command) => events(client, ctx, command).await,
        PosthogResource::Insights(command) => insights(client, ctx, command).await,
        PosthogResource::Persons(command) => persons(client, ctx, command).await,
        PosthogResource::Cohorts(command) => cohorts(client, ctx, command).await,
        PosthogResource::Dashboards(command) => dashboards(client, ctx, command).await,
        PosthogResource::Annotations(command) => annotations(client, ctx, command).await,
    }
}

async fn projects(
    client: &ApiClient,
    ctx: &Context,
    command: PosthogProjectsCommand,
) -> Result<Value, AppError> {
    match command.action {
        PosthogProjectsAction::List(args) => {
            let operation = "projects.list";
            let mut url = format!("{}/api/projects/", posthog_base(ctx.profile()));
            let mut query = Vec::new();
            if args.limit > 0 {
                query.push(format!("limit={}", args.limit));
            }
            if args.offset > 0 {
                query.push(format!("offset={}", args.offset));
            }
            if !query.is_empty() {
                url.push('?');
                url.push_str(&query.join("&"));
            }
            client
                .request("posthog", operation, ctx.profile(), Method::GET, url, None)
                .await
        }
        PosthogProjectsAction::Get(args) => {
            let operation = "projects.get";
            let project_id =
                posthog_project_id(ctx.profile(), args.project_id.as_deref(), operation)?;
            let url = format!(
                "{}/api/projects/{}/",
                posthog_base(ctx.profile()),
                project_id
            );
            client
                .request("posthog", operation, ctx.profile(), Method::GET, url, None)
                .await
        }
    }
}

async fn events(
    client: &ApiClient,
    ctx: &Context,
    command: PosthogEventsCommand,
) -> Result<Value, AppError> {
    match command.action {
        PosthogEventsAction::Query(args) => {
            let operation = "events.query";
            let project_id =
                posthog_project_id(ctx.profile(), args.project_id.as_deref(), operation)?;

            let body = if let Some(file_path) = &args.query_file {
                let content = std::fs::read_to_string(file_path).map_err(|err| {
                    AppError::invalid_input(
                        "posthog",
                        operation,
                        format!("failed to read query file '{file_path}': {err}"),
                    )
                })?;
                serde_json::from_str::<Value>(&content).map_err(|err| {
                    AppError::invalid_input(
                        "posthog",
                        operation,
                        format!("invalid JSON in query file '{file_path}': {err}"),
                    )
                })?
            } else if let Some(query_str) = &args.query {
                let trimmed = query_str.trim();
                if (trimmed.starts_with('{') && trimmed.ends_with('}'))
                    || (trimmed.starts_with('[') && trimmed.ends_with(']'))
                {
                    serde_json::from_str::<Value>(trimmed).map_err(|err| {
                        AppError::invalid_input(
                            "posthog",
                            operation,
                            format!("invalid JSON in query string: {err}"),
                        )
                    })?
                } else {
                    json!({
                        "query": {
                            "kind": "HogQLQuery",
                            "query": trimmed
                        }
                    })
                }
            } else {
                return Err(AppError::invalid_input(
                    "posthog",
                    operation,
                    "posthog.events.query requires either --query or --query-file",
                ));
            };

            let url = format!(
                "{}/api/projects/{}/query/",
                posthog_base(ctx.profile()),
                project_id
            );
            client
                .request(
                    "posthog",
                    operation,
                    ctx.profile(),
                    Method::POST,
                    url,
                    Some(body),
                )
                .await
        }
    }
}

async fn insights(
    client: &ApiClient,
    ctx: &Context,
    command: PosthogInsightsCommand,
) -> Result<Value, AppError> {
    match command.action {
        PosthogInsightsAction::List(args) => {
            let operation = "insights.list";
            let project_id =
                posthog_project_id(ctx.profile(), args.project_id.as_deref(), operation)?;
            let url = build_list_url(
                &format!(
                    "{}/api/projects/{}/insights/",
                    posthog_base(ctx.profile()),
                    project_id
                ),
                args.limit,
                args.offset,
            );
            client
                .request("posthog", operation, ctx.profile(), Method::GET, url, None)
                .await
        }
        PosthogInsightsAction::Get(args) => {
            let operation = "insights.get";
            let project_id =
                posthog_project_id(ctx.profile(), args.project_id.as_deref(), operation)?;
            let url = format!(
                "{}/api/projects/{}/insights/{}/",
                posthog_base(ctx.profile()),
                project_id,
                args.insight_id
            );
            client
                .request("posthog", operation, ctx.profile(), Method::GET, url, None)
                .await
        }
    }
}

async fn persons(
    client: &ApiClient,
    ctx: &Context,
    command: PosthogPersonsCommand,
) -> Result<Value, AppError> {
    match command.action {
        PosthogPersonsAction::List(args) => {
            let operation = "persons.list";
            let project_id =
                posthog_project_id(ctx.profile(), args.project_id.as_deref(), operation)?;
            let url = build_list_url(
                &format!(
                    "{}/api/projects/{}/persons/",
                    posthog_base(ctx.profile()),
                    project_id
                ),
                args.limit,
                args.offset,
            );
            client
                .request("posthog", operation, ctx.profile(), Method::GET, url, None)
                .await
        }
        PosthogPersonsAction::Get(args) => {
            let operation = "persons.get";
            let project_id =
                posthog_project_id(ctx.profile(), args.project_id.as_deref(), operation)?;
            let url = format!(
                "{}/api/projects/{}/persons/{}/",
                posthog_base(ctx.profile()),
                project_id,
                args.person_id
            );
            client
                .request("posthog", operation, ctx.profile(), Method::GET, url, None)
                .await
        }
    }
}

async fn cohorts(
    client: &ApiClient,
    ctx: &Context,
    command: PosthogCohortsCommand,
) -> Result<Value, AppError> {
    match command.action {
        PosthogCohortsAction::List(args) => {
            let operation = "cohorts.list";
            let project_id =
                posthog_project_id(ctx.profile(), args.project_id.as_deref(), operation)?;
            let url = build_list_url(
                &format!(
                    "{}/api/projects/{}/cohorts/",
                    posthog_base(ctx.profile()),
                    project_id
                ),
                args.limit,
                args.offset,
            );
            client
                .request("posthog", operation, ctx.profile(), Method::GET, url, None)
                .await
        }
        PosthogCohortsAction::Get(args) => {
            let operation = "cohorts.get";
            let project_id =
                posthog_project_id(ctx.profile(), args.project_id.as_deref(), operation)?;
            let url = format!(
                "{}/api/projects/{}/cohorts/{}/",
                posthog_base(ctx.profile()),
                project_id,
                args.cohort_id
            );
            client
                .request("posthog", operation, ctx.profile(), Method::GET, url, None)
                .await
        }
    }
}

async fn dashboards(
    client: &ApiClient,
    ctx: &Context,
    command: PosthogDashboardsCommand,
) -> Result<Value, AppError> {
    match command.action {
        PosthogDashboardsAction::List(args) => {
            let operation = "dashboards.list";
            let project_id =
                posthog_project_id(ctx.profile(), args.project_id.as_deref(), operation)?;
            let url = build_list_url(
                &format!(
                    "{}/api/projects/{}/dashboards/",
                    posthog_base(ctx.profile()),
                    project_id
                ),
                args.limit,
                args.offset,
            );
            client
                .request("posthog", operation, ctx.profile(), Method::GET, url, None)
                .await
        }
        PosthogDashboardsAction::Get(args) => {
            let operation = "dashboards.get";
            let project_id =
                posthog_project_id(ctx.profile(), args.project_id.as_deref(), operation)?;
            let url = format!(
                "{}/api/projects/{}/dashboards/{}/",
                posthog_base(ctx.profile()),
                project_id,
                args.dashboard_id
            );
            client
                .request("posthog", operation, ctx.profile(), Method::GET, url, None)
                .await
        }
    }
}

async fn annotations(
    client: &ApiClient,
    ctx: &Context,
    command: PosthogAnnotationsCommand,
) -> Result<Value, AppError> {
    match command.action {
        PosthogAnnotationsAction::List(args) => {
            let operation = "annotations.list";
            let project_id =
                posthog_project_id(ctx.profile(), args.project_id.as_deref(), operation)?;
            let url = build_list_url(
                &format!(
                    "{}/api/projects/{}/annotations/",
                    posthog_base(ctx.profile()),
                    project_id
                ),
                args.limit,
                args.offset,
            );
            client
                .request("posthog", operation, ctx.profile(), Method::GET, url, None)
                .await
        }
        PosthogAnnotationsAction::Get(args) => {
            let operation = "annotations.get";
            let project_id =
                posthog_project_id(ctx.profile(), args.project_id.as_deref(), operation)?;
            let url = format!(
                "{}/api/projects/{}/annotations/{}/",
                posthog_base(ctx.profile()),
                project_id,
                args.annotation_id
            );
            client
                .request("posthog", operation, ctx.profile(), Method::GET, url, None)
                .await
        }
    }
}

fn build_list_url(base: &str, limit: u32, offset: u32) -> String {
    let mut url = base.to_string();
    let mut params = Vec::new();
    if limit > 0 {
        params.push(format!("limit={limit}"));
    }
    if offset > 0 {
        params.push(format!("offset={offset}"));
    }
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_list_url() {
        assert_eq!(
            build_list_url("https://us.i.posthog.com/api/projects/1/insights/", 50, 0),
            "https://us.i.posthog.com/api/projects/1/insights/?limit=50"
        );
        assert_eq!(
            build_list_url("https://us.i.posthog.com/api/projects/1/insights/", 10, 20),
            "https://us.i.posthog.com/api/projects/1/insights/?limit=10&offset=20"
        );
    }
}
