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
                let mut url = format!(
                    "{}/rest/api/3/search/jql?maxResults={}",
                    site_url(ctx.profile(), "jira", "issues.list")?,
                    args.limit
                );
                if let Some(jql) = args.jql {
                    url.push_str("&jql=");
                    url.push_str(&urlencoding::encode(&jql));
                }
                client
                    .request("jira", "issues.list", ctx.profile(), Method::GET, url, None)
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
            ListGetAction::List(args) => {
                let url = format!(
                    "{}/rest/api/3/project/search?maxResults={}",
                    site_url(ctx.profile(), "jira", "projects.list")?,
                    args.limit
                );
                client
                    .request(
                        "jira",
                        "projects.list",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await
            }
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
