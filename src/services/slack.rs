use reqwest::{Method, StatusCode};
use serde_json::{json, Value};

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::ApiClient,
    input,
    services::{
        generic_request,
        shared::{slack_base, write_download, CtxProfile},
    },
};

pub(crate) async fn dispatch(
    client: &ApiClient,
    ctx: &Context,
    command: SlackCommand,
) -> Result<Value, AppError> {
    match command.resource {
        SlackResource::Channels(command) => channels(client, ctx, command).await,
        SlackResource::Files(command) => files(client, ctx, command).await,
        SlackResource::Bookmarks(command) => bookmarks(client, ctx, command).await,
        SlackResource::Links(command) => links(client, ctx, command).await,
        SlackResource::Canvas(command) => canvas(client, ctx, command).await,
        SlackResource::Request(args) => {
            generic_request::dispatch(client, ctx, "slack", slack_base(ctx.profile()), args).await
        }
    }
}

async fn channels(
    client: &ApiClient,
    ctx: &Context,
    command: SlackChannelsCommand,
) -> Result<Value, AppError> {
    match command.action {
        SlackChannelsAction::List(args) => channels_list(client, ctx, args).await,
        SlackChannelsAction::Get(args) => channels_get(client, ctx, args).await,
    }
}

async fn files(
    client: &ApiClient,
    ctx: &Context,
    command: SlackFilesCommand,
) -> Result<Value, AppError> {
    match command.action {
        SlackFilesAction::List(args) => files_list(client, ctx, args).await,
    }
}

async fn bookmarks(
    client: &ApiClient,
    ctx: &Context,
    command: SlackBookmarksCommand,
) -> Result<Value, AppError> {
    match command.action {
        SlackBookmarksAction::List(args) => bookmarks_list(client, ctx, args).await,
    }
}

async fn links(
    client: &ApiClient,
    ctx: &Context,
    command: SlackLinksCommand,
) -> Result<Value, AppError> {
    match command.action {
        SlackLinksAction::List(args) => links_list(client, ctx, args).await,
    }
}

async fn canvas(
    client: &ApiClient,
    ctx: &Context,
    command: SlackCanvasCommand,
) -> Result<Value, AppError> {
    match command.action {
        SlackCanvasAction::Download(args) => canvas_download(client, ctx, args).await,
    }
}

/// Every Slack HTTP call goes through this so an `ok:false` body (Slack's usual
/// HTTP-200 error shape) can never be missed, even mid-pagination-loop.
async fn call(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    method: Method,
    url: String,
) -> Result<Value, AppError> {
    let value = client
        .request("slack", operation, ctx.profile(), method, url, None)
        .await?;
    check_ok(&value, "slack", operation)?;
    Ok(value)
}

fn check_ok(value: &Value, service: &'static str, operation: &'static str) -> Result<(), AppError> {
    if value.get("ok").and_then(Value::as_bool) != Some(false) {
        return Ok(());
    }
    let error_code = value
        .get("error")
        .and_then(Value::as_str)
        .unwrap_or("unknown_error");
    let status = slack_error_status(error_code);
    Err(AppError::api(
        service,
        operation,
        status,
        format!("slack returned error '{error_code}'"),
        Some(value.clone()),
    ))
}

fn slack_error_status(error_code: &str) -> StatusCode {
    match error_code {
        "invalid_auth" | "not_authed" | "token_revoked" | "token_expired" | "account_inactive" => {
            StatusCode::UNAUTHORIZED
        }
        "missing_scope" => StatusCode::FORBIDDEN,
        "channel_not_found" | "not_visible" | "file_not_found" | "thread_not_found"
        | "page_not_found" => StatusCode::NOT_FOUND,
        "ratelimited" => StatusCode::TOO_MANY_REQUESTS,
        _ => StatusCode::BAD_REQUEST,
    }
}

async fn channels_list(
    client: &ApiClient,
    ctx: &Context,
    args: SlackChannelList,
) -> Result<Value, AppError> {
    let operation = "channels.list";
    let mut query = Query::new();
    query.push_value("types", &args.types);
    paginate_cursor(
        client,
        ctx,
        operation,
        "/conversations.list",
        &mut query,
        args.limit,
        &["channels"],
    )
    .await
}

async fn channels_get(
    client: &ApiClient,
    ctx: &Context,
    args: SlackChannelIdArg,
) -> Result<Value, AppError> {
    let operation = "channels.get";
    let mut url = format!("{}/conversations.info", slack_base(ctx.profile()));
    let mut query = Query::new();
    query.push_value("channel", &args.channel_id);
    query.append_to(&mut url);
    let mut response = call(client, ctx, operation, Method::GET, url).await?;
    if let Some(channel) = response.get_mut("channel") {
        let canvas_id = extract_canvas_file_id(channel);
        input::ensure_object(channel).insert(
            "canvas_id".to_string(),
            canvas_id.map(Value::String).unwrap_or(Value::Null),
        );
    }
    Ok(response)
}

/// Channel canvases surface at `properties.tabs[].type == "canvas"` -> `data.file_id`,
/// not at a top-level `properties.canvas` field (confirmed against a live workspace).
fn extract_canvas_file_id(channel: &Value) -> Option<String> {
    channel
        .pointer("/properties/tabs")?
        .as_array()?
        .iter()
        .find(|tab| tab.get("type").and_then(Value::as_str) == Some("canvas"))
        .and_then(|tab| tab.pointer("/data/file_id"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// files.list uses 1-indexed page/count pagination, not a row offset, so this is a
/// bespoke loop rather than a reuse of an offset-style paginator.
async fn files_list(
    client: &ApiClient,
    ctx: &Context,
    args: SlackChannelAssociatedList,
) -> Result<Value, AppError> {
    let operation = "files.list";
    if args.limit == 0 {
        return Ok(empty_aggregate(&["files"]));
    }

    let page_size = args.limit.clamp(1, 200);
    let mut page_num = 1u64;
    let mut total_pages;
    let mut first_page = None;
    let mut values = Vec::new();

    loop {
        let mut url = format!("{}/files.list", slack_base(ctx.profile()));
        let mut query = Query::new();
        query.push_value("channel", &args.channel_id);
        query.set("count", page_size.to_string());
        query.set("page", page_num.to_string());
        query.append_to(&mut url);
        let page = call(client, ctx, operation, Method::GET, url).await?;
        if first_page.is_none() {
            first_page = Some(page.clone());
        }
        total_pages = page
            .pointer("/paging/pages")
            .and_then(Value::as_u64)
            .unwrap_or(1);
        for value in values_at(&page, &["files"]) {
            if values.len() >= args.limit as usize {
                break;
            }
            values.push(value);
        }
        if values.len() >= args.limit as usize || page_num >= total_pages {
            break;
        }
        page_num += 1;
    }

    let has_more = page_num < total_pages;
    let mut response = aggregate_response(first_page, &["files"], values);
    input::ensure_object(&mut response).insert("has_more".to_string(), json!(has_more));
    Ok(response)
}

/// bookmarks.list is never paginated (Slack caps channels at 100 bookmarks), but the
/// CLI verb "list" would otherwise trip pagination::looks_like_collection_command and
/// produce a misleading "unknown, try a larger limit" hint without this explicit flag.
async fn bookmarks_list(
    client: &ApiClient,
    ctx: &Context,
    args: SlackChannelIdArg,
) -> Result<Value, AppError> {
    let operation = "bookmarks.list";
    let mut url = format!("{}/bookmarks.list", slack_base(ctx.profile()));
    let mut query = Query::new();
    query.push_value("channel_id", &args.channel_id);
    query.append_to(&mut url);
    let mut response = call(client, ctx, operation, Method::GET, url).await?;
    input::ensure_object(&mut response).insert("has_more".to_string(), json!(false));
    Ok(response)
}

async fn links_list(
    client: &ApiClient,
    ctx: &Context,
    args: SlackChannelAssociatedList,
) -> Result<Value, AppError> {
    let operation = "links.list";
    if args.limit == 0 {
        return Ok(json!({ "links": [], "has_more": false }));
    }

    let page_size = args.limit.clamp(1, 200);
    let mut cursor: Option<String> = None;
    let mut values = Vec::new();

    loop {
        let mut url = format!("{}/conversations.history", slack_base(ctx.profile()));
        let mut query = Query::new();
        query.push_value("channel", &args.channel_id);
        query.set("limit", page_size.to_string());
        if let Some(cursor) = cursor.as_deref() {
            query.set("cursor", cursor.to_string());
        }
        query.append_to(&mut url);
        let page = call(client, ctx, operation, Method::GET, url).await?;
        cursor = page
            .pointer("/response_metadata/next_cursor")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);

        for message in values_at(&page, &["messages"]) {
            for link in extract_links(&message) {
                if values.len() >= args.limit as usize {
                    break;
                }
                values.push(link);
            }
            if values.len() >= args.limit as usize {
                break;
            }
        }
        if values.len() >= args.limit as usize || cursor.is_none() {
            break;
        }
    }

    Ok(json!({ "links": values, "has_more": cursor.is_some() }))
}

/// Walks `message.blocks[].elements[].elements[]` for `type == "link"` and reads `.url`.
/// Deliberately does NOT scan `message.text` — Slack truncates display text and
/// HTML-escapes it there (confirmed live: `&` renders as `&amp;`), which silently
/// mangles or drops URLs.
fn extract_links(message: &Value) -> Vec<Value> {
    let mut links = Vec::new();
    let Some(blocks) = message.get("blocks").and_then(Value::as_array) else {
        return links;
    };
    for block in blocks {
        let Some(groups) = block.get("elements").and_then(Value::as_array) else {
            continue;
        };
        for group in groups {
            let Some(elements) = group.get("elements").and_then(Value::as_array) else {
                continue;
            };
            for element in elements {
                if element.get("type").and_then(Value::as_str) != Some("link") {
                    continue;
                }
                let url = element
                    .get("url")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if url.is_empty() {
                    continue;
                }
                let text = element.get("text").and_then(Value::as_str).unwrap_or(url);
                links.push(json!({
                    "url": url,
                    "text": text,
                    "message_ts": message.get("ts").cloned().unwrap_or(Value::Null),
                }));
            }
        }
    }
    links
}

/// Resolves the channel's canvas, downloads its content via `url_private_download`,
/// and writes it to `--output`. Same download mechanism for canvases and ordinary
/// files (confirmed live) - only the file_id lookup differs. Canvas content comes
/// back as an HTML fragment, not markdown; ordinary files come back as their native
/// bytes - the download itself doesn't special-case either.
async fn canvas_download(
    client: &ApiClient,
    ctx: &Context,
    args: SlackCanvasDownload,
) -> Result<Value, AppError> {
    let operation = "canvas.download";
    let mut info_url = format!("{}/conversations.info", slack_base(ctx.profile()));
    let mut info_query = Query::new();
    info_query.push_value("channel", &args.channel_id);
    info_query.append_to(&mut info_url);
    let info = call(client, ctx, operation, Method::GET, info_url).await?;
    let channel = info.get("channel").ok_or_else(|| {
        AppError::internal(
            "slack",
            operation,
            "conversations.info response missing channel",
        )
    })?;
    let file_id = extract_canvas_file_id(channel).ok_or_else(|| {
        AppError::not_found(
            "slack",
            operation,
            format!("channel {} has no canvas", args.channel_id),
        )
    })?;

    let mut file_url = format!("{}/files.info", slack_base(ctx.profile()));
    let mut file_query = Query::new();
    file_query.push_value("file", &file_id);
    file_query.append_to(&mut file_url);
    let file_info = call(client, ctx, operation, Method::GET, file_url).await?;
    let download_url = file_info
        .pointer("/file/url_private_download")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            AppError::internal("slack", operation, "file info missing url_private_download")
        })?
        .to_string();
    let title = file_info
        .pointer("/file/title")
        .and_then(Value::as_str)
        .map(str::to_string);

    let bytes = client
        .download("slack", operation, ctx.profile(), download_url)
        .await?;
    let mut result = write_download("slack", operation, &args.output, &bytes)?;
    let object = input::ensure_object(&mut result);
    object.insert("canvas_id".to_string(), json!(file_id));
    if let Some(title) = title {
        object.insert("title".to_string(), json!(title));
    }
    Ok(result)
}

/// Cursor pagination via `response_metadata.next_cursor` (exhausted as an empty
/// string, not a missing field - confirmed live).
async fn paginate_cursor(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    path: &str,
    query: &mut Query,
    limit: u32,
    array_path: &[&str],
) -> Result<Value, AppError> {
    if limit == 0 {
        return Ok(empty_aggregate(array_path));
    }

    let mut cursor: Option<String> = None;
    let mut first_page = None;
    let mut values = Vec::new();
    let page_size = limit.clamp(1, 200);

    loop {
        query.set("limit", page_size.to_string());
        if let Some(cursor) = cursor.as_deref() {
            query.set("cursor", cursor.to_string());
        }
        let mut url = format!("{}{}", slack_base(ctx.profile()), path);
        query.append_to(&mut url);
        let page = call(client, ctx, operation, Method::GET, url).await?;
        if first_page.is_none() {
            first_page = Some(page.clone());
        }
        cursor = page
            .pointer("/response_metadata/next_cursor")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
        for value in values_at(&page, array_path) {
            if values.len() >= limit as usize {
                break;
            }
            values.push(value);
        }
        if values.len() >= limit as usize {
            break;
        }
        if cursor.is_none() {
            break;
        }
    }

    set_optional_at(
        &mut first_page,
        &["response_metadata", "next_cursor"],
        cursor.map(Value::String),
    );
    Ok(aggregate_response(first_page, array_path, values))
}

fn values_at(value: &Value, path: &[&str]) -> Vec<Value> {
    let mut current = value;
    for segment in path {
        let Some(next) = current.get(*segment) else {
            return Vec::new();
        };
        current = next;
    }
    current.as_array().cloned().unwrap_or_default()
}

fn empty_aggregate(array_path: &[&str]) -> Value {
    aggregate_response(None, array_path, Vec::new())
}

fn aggregate_response(first_page: Option<Value>, array_path: &[&str], values: Vec<Value>) -> Value {
    let mut response = first_page.unwrap_or_else(|| json!({}));
    set_array_at(&mut response, array_path, values);
    response
}

fn set_optional_at(response: &mut Option<Value>, path: &[&str], value: Option<Value>) {
    let Some(response) = response.as_mut() else {
        return;
    };
    let mut current = response;
    for segment in &path[..path.len() - 1] {
        let object = input::ensure_object(current);
        current = object
            .entry((*segment).to_string())
            .or_insert_with(|| Value::Object(Default::default()));
    }
    let object = input::ensure_object(current);
    if let Some(value) = value {
        object.insert(path[path.len() - 1].to_string(), value);
    } else {
        object.remove(path[path.len() - 1]);
    }
}

fn set_array_at(response: &mut Value, path: &[&str], values: Vec<Value>) {
    if path.is_empty() {
        *response = Value::Array(values);
        return;
    }
    let mut current = response;
    for segment in &path[..path.len() - 1] {
        let object = input::ensure_object(current);
        current = object
            .entry((*segment).to_string())
            .or_insert_with(|| Value::Object(Default::default()));
    }
    input::ensure_object(current).insert(path[path.len() - 1].to_string(), Value::Array(values));
}

#[derive(Debug, Default)]
struct Query(Vec<(String, String)>);

impl Query {
    fn new() -> Self {
        Self::default()
    }

    fn push_value(&mut self, key: &str, value: &str) {
        self.0.push((key.to_string(), value.to_string()));
    }

    fn set(&mut self, key: &str, value: String) {
        if let Some((_, existing)) = self.0.iter_mut().find(|(existing, _)| existing == key) {
            *existing = value;
        } else {
            self.0.push((key.to_string(), value));
        }
    }

    fn append_to(&self, url: &mut String) {
        let mut separator = if url.contains('?') { '&' } else { '?' };
        for (key, value) in &self.0 {
            url.push(separator);
            separator = '&';
            url.push_str(&urlencoding::encode(key));
            url.push('=');
            url.push_str(&urlencoding::encode(value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_links_reads_url_from_blocks_not_truncated_text() {
        let message = json!({
            "ts": "1785483401.603779",
            "text": "this is sample link to AF board: <https://aai-labs.atlassian.net/jira/software/projects/AF/boards/1052?filter=&amp;groupBy=none|aai-labs.atlassian.net/jira/…/1052?groupBy=none&…>",
            "blocks": [
                {
                    "type": "rich_text",
                    "elements": [
                        {
                            "type": "rich_text_section",
                            "elements": [
                                { "type": "text", "text": "this is sample link to AF board: " },
                                {
                                    "type": "link",
                                    "url": "https://aai-labs.atlassian.net/jira/software/projects/AF/boards/1052?filter=&groupBy=none",
                                    "text": "aai-labs.atlassian.net/jira/…/1052?groupBy=none&…",
                                    "truncated": true
                                }
                            ]
                        }
                    ]
                }
            ]
        });

        let links = extract_links(&message);
        assert_eq!(links.len(), 1);
        assert_eq!(
            links[0]["url"],
            "https://aai-labs.atlassian.net/jira/software/projects/AF/boards/1052?filter=&groupBy=none"
        );
        assert_eq!(links[0]["message_ts"], "1785483401.603779");
    }

    #[test]
    fn extract_links_returns_empty_for_message_without_blocks() {
        let message = json!({ "ts": "1", "text": "no links here" });
        assert!(extract_links(&message).is_empty());
    }

    #[test]
    fn extract_canvas_file_id_finds_canvas_tab() {
        let channel = json!({
            "properties": {
                "tabs": [
                    { "type": "files", "id": "files" },
                    {
                        "type": "canvas",
                        "data": { "file_id": "F0BM3V2MP9Q", "shared_ts": "1785483700.717999" }
                    },
                    { "type": "folder", "label": "Bookmarks" }
                ]
            }
        });
        assert_eq!(
            extract_canvas_file_id(&channel).as_deref(),
            Some("F0BM3V2MP9Q")
        );
    }

    #[test]
    fn extract_canvas_file_id_none_when_no_canvas_tab() {
        let channel = json!({ "properties": { "tabs": [{ "type": "files", "id": "files" }] } });
        assert_eq!(extract_canvas_file_id(&channel), None);

        let channel_no_properties = json!({});
        assert_eq!(extract_canvas_file_id(&channel_no_properties), None);
    }

    #[test]
    fn slack_error_status_maps_known_error_codes() {
        assert_eq!(slack_error_status("invalid_auth"), StatusCode::UNAUTHORIZED);
        assert_eq!(slack_error_status("missing_scope"), StatusCode::FORBIDDEN);
        assert_eq!(
            slack_error_status("channel_not_found"),
            StatusCode::NOT_FOUND
        );
        assert_eq!(slack_error_status("not_visible"), StatusCode::NOT_FOUND);
        assert_eq!(
            slack_error_status("ratelimited"),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            slack_error_status("some_new_error"),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn check_ok_passes_through_successful_response() {
        let value = json!({ "ok": true, "channels": [] });
        assert!(check_ok(&value, "slack", "test").is_ok());
        let value_without_ok_field = json!({ "channels": [] });
        assert!(check_ok(&value_without_ok_field, "slack", "test").is_ok());
    }

    #[test]
    fn check_ok_raises_not_found_for_channel_not_found() {
        let value = json!({ "ok": false, "error": "channel_not_found" });
        let error = check_ok(&value, "slack", "test").unwrap_err();
        assert_eq!(error.code, "not_found");
    }
}
