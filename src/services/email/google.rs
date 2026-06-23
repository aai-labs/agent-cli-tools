use base64::{engine::general_purpose, Engine as _};
use reqwest::Method;
use serde_json::{json, Value};

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::ApiClient,
    input,
    services::shared::{enc, google_base, CtxProfile},
};

use super::mime::find_body_in_gmail_payload;

pub(crate) async fn messages(
    client: &ApiClient,
    ctx: &Context,
    action: EmailMessagesAction,
) -> Result<Value, AppError> {
    match action {
        EmailMessagesAction::List(args) => {
            let user = ctx.profile().user_id.as_deref().unwrap_or("me");
            let mut q_parts: Vec<String> = Vec::new();
            if let Some(after) = &args.received_after {
                q_parts.push(format!("after:{}", after.replace('-', "/")));
            }
            if let Some(before) = &args.received_before {
                q_parts.push(format!("before:{}", before.replace('-', "/")));
            }
            let mut url = format!(
                "{}/gmail/v1/users/{}/messages?maxResults={}",
                google_base(ctx.profile()),
                enc(user),
                args.limit
            );
            if !q_parts.is_empty() {
                url.push_str(&format!("&q={}", urlencoding::encode(&q_parts.join(" "))));
            }
            client
                .request(
                    "email",
                    "messages.list",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        EmailMessagesAction::Get(args) => {
            let user = ctx.profile().user_id.as_deref().unwrap_or("me");
            let url = format!(
                "{}/gmail/v1/users/{}/messages/{}",
                google_base(ctx.profile()),
                enc(user),
                enc(&args.id)
            );
            let raw = client
                .request(
                    "email",
                    "messages.get",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await?;
            Ok(extract_gmail_message(raw))
        }
        EmailMessagesAction::Send(args) => {
            let user = ctx.profile().user_id.as_deref().unwrap_or("me");
            let body = send_body(args)?;
            let url = format!(
                "{}/gmail/v1/users/{}/messages/send",
                google_base(ctx.profile()),
                enc(user)
            );
            client
                .request(
                    "email",
                    "messages.send",
                    ctx.profile(),
                    Method::POST,
                    url,
                    Some(body),
                )
                .await
        }
        EmailMessagesAction::Delete(args) => {
            let user = ctx.profile().user_id.as_deref().unwrap_or("me");
            let url = format!(
                "{}/gmail/v1/users/{}/messages/{}",
                google_base(ctx.profile()),
                enc(user),
                enc(&args.id)
            );
            client
                .request(
                    "email",
                    "messages.delete",
                    ctx.profile(),
                    Method::DELETE,
                    url,
                    None,
                )
                .await
        }
    }
}

fn extract_gmail_message(v: Value) -> Value {
    let id = v
        .get("id")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let thread_id = v
        .get("threadId")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let snippet = v
        .get("snippet")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();

    let empty = vec![];
    let headers = v
        .get("payload")
        .and_then(|p| p.get("headers"))
        .and_then(|h| h.as_array())
        .unwrap_or(&empty);

    let header = |name: &str| -> String {
        headers
            .iter()
            .find(|h| h.get("name").and_then(|n| n.as_str()) == Some(name))
            .and_then(|h| h.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };

    let subject = header("Subject");
    let from = header("From");
    let to = header("To");
    let date = header("Date");

    let payload = v.get("payload");
    let (body, body_type) = payload
        .and_then(|p| find_body_in_gmail_payload(p))
        .unwrap_or_else(|| (snippet.clone(), "snippet"));

    json!({
        "id": id,
        "thread_id": thread_id,
        "subject": subject,
        "from": from,
        "to": to,
        "date": date,
        "body": body,
        "body_type": body_type,
    })
}

fn send_body(args: EmailSend) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("email", "messages.send", args.json.as_deref())?;
    if body.as_object().is_some_and(|obj| obj.is_empty()) {
        let to = args.to.unwrap_or_default();
        let subject = args.subject.unwrap_or_default();
        let content = args.body.unwrap_or_default();
        let raw = format!("To: {to}\r\nSubject: {subject}\r\n\r\n{content}");
        body = json!({ "raw": general_purpose::URL_SAFE_NO_PAD.encode(raw) });
    }
    Ok(body)
}
