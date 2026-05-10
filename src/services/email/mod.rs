pub(crate) mod google;
pub(crate) mod smtp_imap;
pub(crate) mod zoho;

use serde_json::Value;

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::ApiClient,
    services::shared::{provider, CtxProfile},
};

pub(crate) async fn dispatch(
    client: &ApiClient,
    ctx: &Context,
    command: EmailCommand,
) -> Result<Value, AppError> {
    match command.resource {
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
