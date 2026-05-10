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

pub(crate) async fn messages(
    client: &ApiClient,
    ctx: &Context,
    action: EmailMessagesAction,
) -> Result<Value, AppError> {
    match action {
        EmailMessagesAction::List(args) => {
            let user = ctx.profile().user_id.as_deref().unwrap_or("me");
            let url = format!(
                "{}/gmail/v1/users/{}/messages?maxResults={}",
                google_base(ctx.profile()),
                enc(user),
                args.limit
            );
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
            client
                .request(
                    "email",
                    "messages.get",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
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
