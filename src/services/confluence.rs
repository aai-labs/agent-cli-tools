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
    command: ConfluenceCommand,
) -> Result<Value, AppError> {
    match command.resource {
        ConfluenceResource::Spaces(command) => match command.action {
            ListGetAction::List(args) => {
                let url = format!(
                    "{}/wiki/api/v2/spaces?limit={}",
                    site_url(ctx.profile(), "confluence", "spaces.list")?,
                    page_size(args.limit, 250)
                );
                collect_cursor_results(client, ctx, "spaces.list", url, args.limit).await
            }
            ListGetAction::Get(args) => get_space(client, ctx, &args.id).await,
        },
        ConfluenceResource::Pages(command) => match command.action {
            ConfluencePagesAction::List(args) => {
                let url = format!(
                    "{}/wiki/api/v2/pages?limit={}",
                    site_url(ctx.profile(), "confluence", "pages.list")?,
                    page_size(args.limit, 250)
                );
                collect_cursor_results(client, ctx, "pages.list", url, args.limit).await
            }
            ConfluencePagesAction::Get(args) => {
                let url = format!(
                    "{}/wiki/api/v2/pages/{}?body-format=storage",
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
                let body = page_create_body(client, ctx, args).await?;
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
            ConfluencePagesAction::Move(args) => {
                let url = format!(
                    "{}/wiki/rest/api/content/{}/move/{}/{}",
                    site_url(ctx.profile(), "confluence", "pages.move")?,
                    enc(&args.id),
                    args.position.as_str(),
                    enc(&args.target_id)
                );
                client
                    .request(
                        "confluence",
                        "pages.move",
                        ctx.profile(),
                        Method::PUT,
                        url,
                        None,
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
        ConfluenceResource::Search(args) => {
            let (cql, limit) = search_cql(args)?;
            let url = format!(
                "{}/wiki/rest/api/content/search?cql={}&limit={}",
                site_url(ctx.profile(), "confluence", "search")?,
                enc(&cql),
                page_size(limit, 100)
            );
            collect_cursor_results(client, ctx, "search", url, limit).await
        }
    }
}

async fn collect_cursor_results(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    first_url: String,
    limit: u32,
) -> Result<Value, AppError> {
    if limit == 0 {
        return Ok(json!({ "results": [], "limit": 0, "size": 0 }));
    }

    let site = site_url(ctx.profile(), "confluence", operation)?;
    let mut url = first_url;
    let mut first_page = None;
    let mut results = Vec::new();

    loop {
        let page = client
            .request(
                "confluence",
                operation,
                ctx.profile(),
                Method::GET,
                url,
                None,
            )
            .await?;
        if first_page.is_none() {
            first_page = Some(page.clone());
        }

        let page_results = page
            .get("results")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if page_results.is_empty() {
            break;
        }
        for result in page_results {
            if results.len() >= limit as usize {
                break;
            }
            results.push(result);
        }

        if results.len() >= limit as usize {
            break;
        }
        let Some(next) = page
            .pointer("/_links/next")
            .and_then(Value::as_str)
            .filter(|next| !next.is_empty())
        else {
            break;
        };
        url = confluence_link_url(&site, next);
    }

    let mut response = first_page.unwrap_or_else(|| json!({}));
    let object = input::ensure_object(&mut response);
    let size = results.len();
    object.insert("results".to_string(), Value::Array(results));
    object.insert("limit".to_string(), json!(limit));
    object.insert("size".to_string(), json!(size));
    Ok(response)
}

fn confluence_link_url(site: &str, link: &str) -> String {
    if link.starts_with("http://") || link.starts_with("https://") {
        link.to_string()
    } else if link.starts_with("/wiki/") {
        format!("{site}{link}")
    } else if link.starts_with('/') {
        format!("{site}/wiki{link}")
    } else {
        format!("{site}/wiki/{link}")
    }
}

fn page_size(limit: u32, max: u32) -> u32 {
    limit.clamp(1, max)
}

fn search_cql(args: ConfluenceSearch) -> Result<(String, u32), AppError> {
    if let Some(cql) = args.cql {
        if !cql.trim().is_empty() {
            return Ok((cql, args.limit));
        }
    }
    if let Some(query) = args.query {
        if !query.trim().is_empty() {
            return Ok((format!("text ~ {}", cql_string_literal(&query)), args.limit));
        }
    }
    Err(AppError::invalid_input(
        "confluence",
        "search",
        "--cql or --query is required",
    ))
}

fn cql_string_literal(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

async fn get_space(client: &ApiClient, ctx: &Context, id_or_key: &str) -> Result<Value, AppError> {
    if is_numeric_id(id_or_key) {
        let url = format!(
            "{}/wiki/api/v2/spaces/{}",
            site_url(ctx.profile(), "confluence", "spaces.get")?,
            enc(id_or_key)
        );
        return client
            .request(
                "confluence",
                "spaces.get",
                ctx.profile(),
                Method::GET,
                url,
                None,
            )
            .await;
    }
    space_by_key(client, ctx, "spaces.get", id_or_key).await
}

async fn page_create_body(
    client: &ApiClient,
    ctx: &Context,
    args: ConfluencePageCreate,
) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("confluence", "pages.create", args.json.as_deref())?;
    let space_ref = args.space_key.as_deref().or(args.space_id.as_deref());
    if let Some(space_ref) = space_ref {
        let space_id = resolve_space_id(client, ctx, "pages.create", space_ref).await?;
        input::ensure_object(&mut body).insert("spaceId".to_string(), Value::String(space_id));
    }
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

async fn resolve_space_id(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    id_or_key: &str,
) -> Result<String, AppError> {
    if is_numeric_id(id_or_key) {
        return Ok(id_or_key.to_string());
    }
    let space = space_by_key(client, ctx, operation, id_or_key).await?;
    space
        .get("id")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| AppError::internal("confluence", operation, "space response missing id"))
}

async fn space_by_key(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    key: &str,
) -> Result<Value, AppError> {
    let url = format!(
        "{}/wiki/api/v2/spaces?keys={}&limit=1",
        site_url(ctx.profile(), "confluence", operation)?,
        enc(key)
    );
    let response = client
        .request(
            "confluence",
            operation,
            ctx.profile(),
            Method::GET,
            url,
            None,
        )
        .await?;
    response
        .get("results")
        .and_then(Value::as_array)
        .and_then(|spaces| {
            spaces
                .iter()
                .find(|space| space.get("key").and_then(Value::as_str) == Some(key))
                .or_else(|| spaces.first())
        })
        .cloned()
        .ok_or_else(|| {
            AppError::not_found(
                "confluence",
                operation,
                format!("space key {key} was not found"),
            )
        })
}

fn is_numeric_id(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit())
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
