pub(crate) mod caldav;
pub(crate) mod google;
pub(crate) mod zoho;

use serde_json::Value;

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::ApiClient,
    services::{
        generic_request,
        shared::{google_base, provider, zoho_calendar_base, CtxProfile},
    },
};

pub(crate) async fn dispatch(
    client: &ApiClient,
    ctx: &Context,
    command: CalendarCommand,
) -> Result<Value, AppError> {
    match command.resource {
        CalendarResource::Request(args) => {
            if matches!(ctx.profile().transport.as_deref(), Some("caldav"))
                || matches!(ctx.profile().auth_type.as_deref(), Some("caldav_password"))
            {
                return Err(AppError::invalid_input(
                    "calendar",
                    "request",
                    "generic requests require a REST calendar profile; CalDAV is not supported",
                ));
            }
            let base = match provider(ctx.profile(), "calendar", "request")? {
                "google" => google_base(ctx.profile()),
                "zoho" => zoho_calendar_base(ctx.profile()),
                other => {
                    return Err(AppError::invalid_input(
                        "calendar",
                        "request",
                        format!("unsupported REST calendar provider {other}"),
                    ))
                }
            };
            generic_request::dispatch(client, ctx, "calendar", base, args).await
        }
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
