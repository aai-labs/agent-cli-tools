use reqwest::Method;
use serde_json::Value;

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
    command: ConfluenceCommand,
) -> Result<Value, AppError> {
    match command.resource {
        ConfluenceResource::Spaces(command) => match command.action {
            ListGetAction::List(args) => {
                let url = format!(
                    "{}/wiki/api/v2/spaces?limit={}",
                    site_url(ctx.profile(), "confluence", "spaces.list")?,
                    args.limit
                );
                client
                    .request(
                        "confluence",
                        "spaces.list",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await
            }
            ListGetAction::Get(args) => {
                let url = format!(
                    "{}/wiki/api/v2/spaces/{}",
                    site_url(ctx.profile(), "confluence", "spaces.get")?,
                    enc(&args.id)
                );
                client
                    .request(
                        "confluence",
                        "spaces.get",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await
            }
        },
        ConfluenceResource::Pages(command) => match command.action {
            ConfluencePagesAction::List(args) => {
                let url = format!(
                    "{}/wiki/api/v2/pages?limit={}",
                    site_url(ctx.profile(), "confluence", "pages.list")?,
                    args.limit
                );
                client
                    .request(
                        "confluence",
                        "pages.list",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await
            }
            ConfluencePagesAction::Get(args) => {
                let url = format!(
                    "{}/wiki/api/v2/pages/{}",
                    site_url(ctx.profile(), "confluence", "pages.get")?,
                    enc(&args.id)
                );
                client
                    .request(
                        "confluence",
                        "pages.get",
                        ctx.profile(),
                        Method::GET,
                        url,
                        None,
                    )
                    .await
            }
            ConfluencePagesAction::Create(args) => {
                let body = page_create_body(args)?;
                let url = format!(
                    "{}/wiki/api/v2/pages",
                    site_url(ctx.profile(), "confluence", "pages.create")?
                );
                client
                    .request(
                        "confluence",
                        "pages.create",
                        ctx.profile(),
                        Method::POST,
                        url,
                        Some(body),
                    )
                    .await
            }
            ConfluencePagesAction::Update(args) => {
                let id = args.id.clone();
                let body = page_update_body(args)?;
                let url = format!(
                    "{}/wiki/api/v2/pages/{}",
                    site_url(ctx.profile(), "confluence", "pages.update")?,
                    enc(&id)
                );
                client
                    .request(
                        "confluence",
                        "pages.update",
                        ctx.profile(),
                        Method::PUT,
                        url,
                        Some(body),
                    )
                    .await
            }
            ConfluencePagesAction::Delete(args) => {
                let url = format!(
                    "{}/wiki/api/v2/pages/{}",
                    site_url(ctx.profile(), "confluence", "pages.delete")?,
                    enc(&args.id)
                );
                client
                    .request(
                        "confluence",
                        "pages.delete",
                        ctx.profile(),
                        Method::DELETE,
                        url,
                        None,
                    )
                    .await
            }
        },
    }
}

fn page_create_body(args: ConfluencePageCreate) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("confluence", "pages.create", args.json.as_deref())?;
    input::set_string(&mut body, "spaceId", &args.space_id);
    input::set_string(&mut body, "title", &args.title);
    input::set_string(&mut body, "parentId", &args.parent_id);
    if let Some(body_text) = args.body {
        input::ensure_object(&mut body).insert(
            "body".to_string(),
            serde_json::json!({ "representation": "storage", "value": body_text }),
        );
    }
    Ok(body)
}

fn page_update_body(args: ConfluencePageUpdate) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("confluence", "pages.update", args.json.as_deref())?;
    input::set_string(&mut body, "title", &args.title);
    input::set_u64(&mut body, "version", args.version);
    if let Some(body_text) = args.body {
        input::ensure_object(&mut body).insert(
            "body".to_string(),
            serde_json::json!({ "representation": "storage", "value": body_text }),
        );
    }
    Ok(body)
}
