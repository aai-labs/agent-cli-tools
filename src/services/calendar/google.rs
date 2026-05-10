use reqwest::Method;
use serde_json::{json, Value};

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::ApiClient,
    input,
    services::shared::{calendar_id, enc, google_base, CtxProfile},
};

pub(crate) async fn events(
    client: &ApiClient,
    ctx: &Context,
    action: CalendarEventsAction,
) -> Result<Value, AppError> {
    match action {
        CalendarEventsAction::List(args) => {
            let calendar = calendar_id(ctx.profile(), args.calendar_id.as_deref());
            let url = format!(
                "{}/calendar/v3/calendars/{}/events?maxResults={}",
                google_base(ctx.profile()),
                enc(calendar),
                args.limit
            );
            client
                .request(
                    "calendar",
                    "events.list",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        CalendarEventsAction::Get(args) => {
            let calendar = calendar_id(ctx.profile(), None);
            let url = format!(
                "{}/calendar/v3/calendars/{}/events/{}",
                google_base(ctx.profile()),
                enc(calendar),
                enc(&args.id)
            );
            client
                .request(
                    "calendar",
                    "events.get",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        CalendarEventsAction::Create(args) => {
            let calendar = calendar_id(ctx.profile(), args.calendar_id.as_deref()).to_string();
            let body = event_body(args)?;
            let url = format!(
                "{}/calendar/v3/calendars/{}/events",
                google_base(ctx.profile()),
                enc(&calendar)
            );
            client
                .request(
                    "calendar",
                    "events.create",
                    ctx.profile(),
                    Method::POST,
                    url,
                    Some(body),
                )
                .await
        }
        CalendarEventsAction::Update(args) => {
            let calendar = calendar_id(ctx.profile(), args.calendar_id.as_deref()).to_string();
            let id = args.id.clone();
            let body = event_update_body(args)?;
            let url = format!(
                "{}/calendar/v3/calendars/{}/events/{}",
                google_base(ctx.profile()),
                enc(&calendar),
                enc(&id)
            );
            client
                .request(
                    "calendar",
                    "events.update",
                    ctx.profile(),
                    Method::PATCH,
                    url,
                    Some(body),
                )
                .await
        }
        CalendarEventsAction::Delete(args) => {
            let calendar = calendar_id(ctx.profile(), args.calendar_id.as_deref()).to_string();
            let url = format!(
                "{}/calendar/v3/calendars/{}/events/{}",
                google_base(ctx.profile()),
                enc(&calendar),
                enc(&args.id)
            );
            client
                .request(
                    "calendar",
                    "events.delete",
                    ctx.profile(),
                    Method::DELETE,
                    url,
                    None,
                )
                .await
        }
    }
}

fn event_body(args: CalendarEventCreate) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("calendar", "events.create", args.json.as_deref())?;
    input::set_string(&mut body, "summary", &args.summary);
    input::set_string(&mut body, "description", &args.description);
    set_time(&mut body, "start", args.start);
    set_time(&mut body, "end", args.end);
    Ok(body)
}

fn event_update_body(args: CalendarEventUpdate) -> Result<Value, AppError> {
    let mut body = input::read_json_arg("calendar", "events.update", args.json.as_deref())?;
    input::set_string(&mut body, "summary", &args.summary);
    input::set_string(&mut body, "description", &args.description);
    set_time(&mut body, "start", args.start);
    set_time(&mut body, "end", args.end);
    Ok(body)
}

fn set_time(body: &mut Value, key: &str, value: Option<String>) {
    if let Some(value) = value {
        input::ensure_object(body).insert(key.to_string(), json!({ "dateTime": value }));
    }
}
