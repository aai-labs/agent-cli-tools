use reqwest::Method;
use serde_json::{json, Value};

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::ApiClient,
    input,
    services::shared::{enc, site_url, CtxProfile},
};

pub(crate) async fn dispatch(
    client: &ApiClient,
    ctx: &Context,
    command: JiraCommand,
) -> Result<Value, AppError> {
    match command.resource {
        JiraResource::Issues(command) => match command.action {
            JiraIssuesAction::List(args) => {
                search_issues(
                    client,
                    ctx,
                    "issues.list",
                    args.jql.as_deref(),
                    args.fields.as_deref(),
                    args.limit,
                )
                .await
            }
            JiraIssuesAction::Search(args) => {
                search_issues(
                    client,
                    ctx,
                    "issues.search",
                    Some(&args.jql),
                    args.fields.as_deref(),
                    args.limit,
                )
                .await
            }
            JiraIssuesAction::Get(args) => {
                let url = format!(
                    "{}/rest/api/3/issue/{}",
                    site_url(ctx.profile(), "jira", "issues.get")?,
                    enc(&args.id)
                );
                client
                    .request("jira", "issues.get", ctx.profile(), Method::GET, url, None)
                    .await
            }
            JiraIssuesAction::Create(args) => {
                let body = issue_create_body(args)?;
                let url = format!(
                    "{}/rest/api/3/issue",
                    site_url(ctx.profile(), "jira", "issues.create")?
                );
                client
                    .request(
                        "jira",
                        "issues.create",
                        ctx.profile(),
                        Method::POST,
                        url,
                        Some(body),
                    )
                    .await
            }
            JiraIssuesAction::Update(args) => {
                let id = args.id.clone();
                let body = issue_update_body(args)?;
                let url = format!(
                    "{}/rest/api/3/issue/{}",
                    site_url(ctx.profile(), "jira", "issues.update")?,
                    enc(&id)
                );
                client
                    .request(
                        "jira",
                        "issues.update",
                        ctx.profile(),
                        Method::PUT,
                        url,
                        Some(body),
                    )
                    .await
            }
            JiraIssuesAction::Delete(args) => {
                let url = format!(
                    "{}/rest/api/3/issue/{}",
                    site_url(ctx.profile(), "jira", "issues.delete")?,
                    enc(&args.id)
                );
                client
                    .request(
                        "jira",
                        "issues.delete",
                        ctx.profile(),
                        Method::DELETE,
                        url,
                        None,
                    )
                    .await
            }
        },
        JiraResource::Projects(command) => match command.action {
            ListGetAction::List(args) => list_projects(client, ctx, args.limit).await,
            ListGetAction::Get(args) => {
                let url = format!(
                    "{}/rest/api/3/project/{}",
                    site_url(ctx.profile(), "jira", "projects.get")?,
                    enc(&args.id)
                );
                client
                    .request(
                        "jira",
                        "projects.get",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await
            }
        },
    }
}

async fn search_issues(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    jql: Option<&str>,
    fields: Option<&str>,
    limit: u32,
) -> Result<Value, AppError> {
    if limit == 0 {
        return Ok(json!({ "issues": [], "maxResults": 0, "total": 0, "isLast": true }));
    }

    let base = site_url(ctx.profile(), "jira", operation)?;
    let page_size = limit.clamp(1, 100);
    let mut next_page_token = None;
    let mut first_page = None;
    let mut issues = Vec::new();

    loop {
        let mut url = format!("{base}/rest/api/3/search/jql?maxResults={page_size}");
        if let Some(jql) = jql {
            url.push_str("&jql=");
            url.push_str(&urlencoding::encode(jql));
        }
        url.push_str("&fields=");
        url.push_str(&urlencoding::encode(fields.unwrap_or(
            "key,summary,status,issuetype,assignee,created,updated,description,project",
        )));
        if let Some(token) = next_page_token.as_deref() {
            url.push_str("&nextPageToken=");
            url.push_str(&urlencoding::encode(token));
        }

        let page = client
            .request("jira", operation, ctx.profile(), Method::GET, url, None)
            .await?;
        if first_page.is_none() {
            first_page = Some(page.clone());
        }

        let page_issues = page
            .get("issues")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for issue in page_issues {
            if issues.len() >= limit as usize {
                break;
            }
            issues.push(issue);
        }

        if issues.len() >= limit as usize {
            break;
        }
        next_page_token = page
            .get("nextPageToken")
            .and_then(Value::as_str)
            .map(ToString::to_string);
        if next_page_token.is_none() || page.get("isLast").and_then(Value::as_bool) == Some(true) {
            break;
        }
    }

    let mut response = first_page.unwrap_or_else(|| json!({}));
    let object = input::ensure_object(&mut response);
    object.insert("issues".to_string(), Value::Array(issues));
    object.insert("maxResults".to_string(), json!(limit));
    object.insert("isLast".to_string(), Value::Bool(next_page_token.is_none()));
    object.remove("nextPageToken");
    Ok(response)
}

async fn list_projects(client: &ApiClient, ctx: &Context, limit: u32) -> Result<Value, AppError> {
    if limit == 0 {
        return Ok(json!({ "values": [], "maxResults": 0, "total": 0, "isLast": true }));
    }

    let base = site_url(ctx.profile(), "jira", "projects.list")?;
    let page_size = limit.clamp(1, 100);
    let mut start_at = 0usize;
    let mut first_page = None;
    let mut values = Vec::new();

    loop {
        let url =
            format!("{base}/rest/api/3/project/search?maxResults={page_size}&startAt={start_at}");
        let page = client
            .request(
                "jira",
                "projects.list",
                ctx.profile(),
                Method::GET,
                url,
                None,
            )
            .await?;
        if first_page.is_none() {
            first_page = Some(page.clone());
        }

        let page_values = page
            .get("values")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if page_values.is_empty() {
            break;
        }
        for value in page_values {
            if values.len() >= limit as usize {
                break;
            }
            values.push(value);
        }

        if values.len() >= limit as usize
            || page.get("isLast").and_then(Value::as_bool) == Some(true)
        {
            break;
        }
        start_at += page
            .get("maxResults")
            .and_then(Value::as_u64)
            .unwrap_or(page_size as u64) as usize;
    }

    let mut response = first_page.unwrap_or_else(|| json!({}));
    let object = input::ensure_object(&mut response);
    object.insert("values".to_string(), Value::Array(values));
    object.insert("maxResults".to_string(), json!(limit));
    object.insert("startAt".to_string(), json!(0));
    Ok(response)
}

fn issue_create_body(args: JiraIssueCreate) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("jira", "issues.create", args.json.as_deref())?;
    let fields = fields_object(&mut body);
    if let Some(project) = args.project {
        fields.insert("project".to_string(), json!({ "key": project }));
    }
    if let Some(issue_type) = args.issue_type.or_else(|| Some("Task".to_string())) {
        fields.insert("issuetype".to_string(), json!({ "name": issue_type }));
    }
    if let Some(summary) = args.summary {
        fields.insert("summary".to_string(), Value::String(summary));
    }
    if let Some(description) = args.description {
        fields.insert("description".to_string(), input::minimal_adf(&description));
    }
    Ok(body)
}

fn issue_update_body(args: JiraIssueUpdate) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("jira", "issues.update", args.json.as_deref())?;
    let fields = fields_object(&mut body);
    if let Some(summary) = args.summary {
        fields.insert("summary".to_string(), Value::String(summary));
    }
    if let Some(description) = args.description {
        fields.insert("description".to_string(), input::minimal_adf(&description));
    }
    Ok(body)
}

fn fields_object(value: &mut Value) -> &mut serde_json::Map<String, Value> {
    let root = input::ensure_object(value);
    let fields = root
        .entry("fields")
        .or_insert_with(|| Value::Object(Default::default()));
    input::ensure_object(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn description_flag_becomes_adf() {
        let body = issue_create_body(JiraIssueCreate {
            json: None,
            project: Some("ENG".to_string()),
            issue_type: None,
            summary: Some("Test".to_string()),
            description: Some("Details".to_string()),
        })
        .unwrap();
        assert_eq!(body["fields"]["project"]["key"], "ENG");
        assert_eq!(body["fields"]["issuetype"]["name"], "Task");
        assert_eq!(body["fields"]["description"]["type"], "doc");
    }
}
