use reqwest::Method;
use serde_json::{json, Value};

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::ApiClient,
    input,
    services::shared::{account_id, enc, zoho_mail_base, CtxProfile},
};

use super::mime::{extract_readable_body, parse_rfc822_headers, split_rfc822};

pub(crate) async fn messages(
    client: &ApiClient,
    ctx: &Context,
    action: EmailMessagesAction,
) -> Result<Value, AppError> {
    match action {
        EmailMessagesAction::List(args) => {
            let account = account_id(ctx.profile(), "email", "messages.list")?;
            let after_ms = args
                .received_after
                .as_deref()
                .map(|s| date_to_unix_ms(s, "messages.list"))
                .transpose()?;
            let before_ms = args
                .received_before
                .as_deref()
                .map(|s| date_to_unix_ms(s, "messages.list"))
                .transpose()?;
            let fetch_limit = if after_ms.is_some() || before_ms.is_some() {
                200u32
            } else {
                args.limit
            };
            let url = format!(
                "{}/api/accounts/{}/messages/view?limit={}",
                zoho_mail_base(ctx.profile()),
                enc(account),
                fetch_limit
            );
            let response = client
                .request(
                    "email",
                    "messages.list",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await?;
            if after_ms.is_none() && before_ms.is_none() {
                return Ok(response);
            }
            let filtered = filter_by_received_time(response, after_ms, before_ms, args.limit);
            Ok(filtered)
        }
        EmailMessagesAction::Get(args) => {
            let account = account_id(ctx.profile(), "email", "messages.get")?;
            let url = format!(
                "{}/api/accounts/{}/messages/{}/originalmessage",
                zoho_mail_base(ctx.profile()),
                enc(account),
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
            Ok(extract_zoho_message(raw))
        }
        EmailMessagesAction::Send(args) => {
            let account = account_id(ctx.profile(), "email", "messages.send")?;
            let body = send_body(args)?;
            let url = format!(
                "{}/api/accounts/{}/messages",
                zoho_mail_base(ctx.profile()),
                enc(account)
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
            let account = account_id(ctx.profile(), "email", "messages.delete")?;
            let url = format!(
                "{}/api/accounts/{}/messages/{}",
                zoho_mail_base(ctx.profile()),
                enc(account),
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

fn extract_zoho_message(v: Value) -> Value {
    let message_id = v
        .get("data")
        .and_then(|d| d.get("messageId"))
        .and_then(|id| id.as_i64())
        .map(|n| n.to_string())
        .unwrap_or_default();

    let rfc822 = v
        .get("data")
        .and_then(|d| d.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("");

    let (header_text, _) = split_rfc822(rfc822);
    let headers = parse_rfc822_headers(header_text);

    let subject = headers.get("subject").cloned().unwrap_or_default();
    let from = headers.get("from").cloned().unwrap_or_default();
    let to = headers.get("to").cloned().unwrap_or_default();
    let date = headers.get("date").cloned().unwrap_or_default();

    let (body, body_type) = extract_readable_body(rfc822);

    json!({
        "id": message_id,
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
    input::set_string(&mut body, "toAddress", &args.to);
    input::set_string(&mut body, "subject", &args.subject);
    input::set_string(&mut body, "content", &args.body);
    Ok(body)
}

fn date_to_unix_ms(s: &str, op: &'static str) -> Result<i64, AppError> {
    let date = s.split('T').next().unwrap_or(s);
    let parts: Vec<i64> = date
        .split('-')
        .map(|p| p.parse::<i64>())
        .collect::<Result<_, _>>()
        .map_err(|_| AppError::invalid_input("email", op, format!("invalid date: {s}")))?;
    if parts.len() != 3 {
        return Err(AppError::invalid_input(
            "email",
            op,
            format!("invalid date: {s}"),
        ));
    }
    let (y, m, d) = (parts[0], parts[1], parts[2]);
    let jdn = 367 * y - 7 * (y + (m + 9) / 12) / 4 + 275 * m / 9 + d + 1_721_013;
    Ok((jdn - 2_440_588) * 86_400_000)
}

fn filter_by_received_time(
    response: Value,
    after_ms: Option<i64>,
    before_ms: Option<i64>,
    limit: u32,
) -> Value {
    let data = match response.get("data").and_then(|d| d.as_array()) {
        Some(arr) => arr,
        None => return response,
    };
    let filtered: Vec<Value> = data
        .iter()
        .filter(|msg| {
            let ts = msg
                .get("receivedTime")
                .and_then(|v| v.as_str().or_else(|| v.as_str()))
                .and_then(|s| s.parse::<i64>().ok())
                .or_else(|| msg.get("receivedTime").and_then(|v| v.as_i64()));
            let ts = match ts {
                Some(t) => t,
                None => return true,
            };
            after_ms.map_or(true, |a| ts >= a) && before_ms.map_or(true, |b| ts < b)
        })
        .take(limit as usize)
        .cloned()
        .collect();
    let truncated = filtered.len() == limit as usize && data.len() > filtered.len();
    let mut out = response.clone();
    if let Some(obj) = out.as_object_mut() {
        obj.insert("data".to_string(), Value::Array(filtered));
        obj.insert("truncated".to_string(), Value::Bool(truncated));
    }
    out
}
