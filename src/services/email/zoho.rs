use reqwest::Method;
use serde_json::Value;

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::ApiClient,
    input,
    services::shared::{account_id, enc, zoho_mail_base, CtxProfile},
};

pub(crate) async fn messages(
    client: &ApiClient,
    ctx: &Context,
    action: EmailMessagesAction,
) -> Result<Value, AppError> {
    match action {
        EmailMessagesAction::List(args) => {
            let account = account_id(ctx.profile(), "email", "messages.list")?;
            let url = format!(
                "{}/api/accounts/{}/messages/view?limit={}",
                zoho_mail_base(ctx.profile()),
                enc(account),
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
            let account = account_id(ctx.profile(), "email", "messages.get")?;
            let url = format!(
                "{}/api/accounts/{}/messages/{}",
                zoho_mail_base(ctx.profile()),
                enc(account),
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

fn send_body(args: EmailSend) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("email", "messages.send", args.json.as_deref())?;
    input::set_string(&mut body, "toAddress", &args.to);
    input::set_string(&mut body, "subject", &args.subject);
    input::set_string(&mut body, "content", &args.body);
    Ok(body)
}
