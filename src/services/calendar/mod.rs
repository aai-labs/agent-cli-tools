pub(crate) mod caldav;
pub(crate) mod google;
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
    command: CalendarCommand,
) -> Result<Value, AppError> {
    match command.resource {
        CalendarResource::Events(command) => {
            if matches!(ctx.profile().transport.as_deref(), Some("caldav"))
                || matches!(ctx.profile().auth_type.as_deref(), Some("caldav_password"))
            {
                return caldav::events(ctx, command.action).await;
            }

            match provider(ctx.profile(), "calendar", "events")? {
                "google" => google::events(client, ctx, command.action).await,
                "zoho" => zoho::events(client, ctx, command.action).await,
                other => Err(AppError::invalid_input(
                    "calendar",
                    "events",
                    format!("unsupported calendar provider {other}"),
                )),
            }
        }
    }
}
