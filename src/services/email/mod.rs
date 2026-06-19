pub(crate) mod google;
pub(super) mod mime;
pub(crate) mod smtp_imap;
pub(crate) mod zoho;

use serde_json::Value;

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::ApiClient,
    services::{
        generic_request,
        shared::{google_base, provider, zoho_mail_base, CtxProfile},
    },
};

pub(crate) async fn dispatch(
    client: &ApiClient,
    ctx: &Context,
    command: EmailCommand,
) -> Result<Value, AppError> {
    match command.resource {
        EmailResource::Request(args) => {
            if matches!(
                ctx.profile().transport.as_deref(),
                Some("smtp_imap" | "imap_smtp")
            ) || matches!(ctx.profile().auth_type.as_deref(), Some("app_password"))
            {
                return Err(AppError::invalid_input(
                    "email",
                    "request",
                    "generic requests require a REST email profile; SMTP/IMAP is not supported",
                ));
            }
            let base = match provider(ctx.profile(), "email", "request")? {
                "google" => google_base(ctx.profile()),
                "zoho" => zoho_mail_base(ctx.profile()),
                other => {
                    return Err(AppError::invalid_input(
                        "email",
                        "request",
                        format!("unsupported REST email provider {other}"),
                    ))
                }
            };
            generic_request::dispatch(client, ctx, "email", base, args).await
        }
        EmailResource::Messages(command) => {
            if matches!(
                ctx.profile().transport.as_deref(),
                Some("smtp_imap" | "imap_smtp")
            ) || matches!(ctx.profile().auth_type.as_deref(), Some("app_password"))
            {
                return smtp_imap::messages(ctx, command.action).await;
            }

            match provider(ctx.profile(), "email", "messages")? {
                "google" => google::messages(client, ctx, command.action).await,
                "zoho" => zoho::messages(client, ctx, command.action).await,
                other => Err(AppError::invalid_input(
                    "email",
                    "messages",
                    format!("unsupported email provider {other}"),
                )),
            }
        }
    }
}
