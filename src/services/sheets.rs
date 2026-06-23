use reqwest::Method;
use serde_json::{json, Value};

use crate::{
    cli::*,
    config::Context,
    error::AppError,
    http::ApiClient,
    services::shared::{enc, google_base, sheets_base, CtxProfile},
};

pub(crate) async fn dispatch(
    client: &ApiClient,
    ctx: &Context,
    command: SheetsCommand,
) -> Result<Value, AppError> {
    match command.resource {
        SheetsResource::Spreadsheets(cmd) => spreadsheets(client, ctx, cmd.action).await,
        SheetsResource::Values(cmd) => values(client, ctx, cmd.action).await,
    }
}

async fn spreadsheets(
    client: &ApiClient,
    ctx: &Context,
    action: SpreadsheetsAction,
) -> Result<Value, AppError> {
    match action {
        SpreadsheetsAction::List(args) => {
            let page_token_param = args.page_token
                .as_deref()
                .map(|t| format!("&pageToken={}", enc(t)))
                .unwrap_or_default();
            let url = format!(
                "{}/drive/v3/files?q=mimeType%3D%27application%2Fvnd.google-apps.spreadsheet%27&fields=files(id%2Cname)%2CnextPageToken&pageSize=25{}",
                google_base(ctx.profile()),
                page_token_param
            );
            client
                .request("sheets", "spreadsheets.list", ctx.profile(), Method::GET, url, None)
                .await
        }
        SpreadsheetsAction::Get(args) => {
            let url = format!(
                "{}/v4/spreadsheets/{}?fields=sheets.properties",
                sheets_base(),
                enc(&args.spreadsheet_id)
            );
            client
                .request("sheets", "spreadsheets.get", ctx.profile(), Method::GET, url, None)
                .await
        }
    }
}

async fn values(
    client: &ApiClient,
    ctx: &Context,
    action: ValuesAction,
) -> Result<Value, AppError> {
    match action {
        ValuesAction::Get(args) => {
            let url = format!(
                "{}/v4/spreadsheets/{}/values/{}?valueRenderOption=UNFORMATTED_VALUE",
                sheets_base(),
                enc(&args.spreadsheet_id),
                enc(&args.range)
            );
            client
                .request("sheets", "values.get", ctx.profile(), Method::GET, url, None)
                .await
        }
        ValuesAction::Update(args) => {
            let parsed_values: Value =
                serde_json::from_str(&args.values).map_err(|err| {
                    AppError::invalid_input(
                        "sheets",
                        "values.update",
                        format!("--values must be a JSON array of arrays: {err}"),
                    )
                })?;
            if !parsed_values.is_array() {
                return Err(AppError::invalid_input(
                    "sheets",
                    "values.update",
                    "--values must be a JSON array of arrays",
                ));
            }
            let body = json!({
                "range": args.range,
                "majorDimension": "ROWS",
                "values": parsed_values,
            });
            let url = format!(
                "{}/v4/spreadsheets/{}/values/{}?valueInputOption=USER_ENTERED",
                sheets_base(),
                enc(&args.spreadsheet_id),
                enc(&args.range)
            );
            client
                .request("sheets", "values.update", ctx.profile(), Method::PUT, url, Some(body))
                .await
        }
        ValuesAction::Clear(args) => {
            let url = format!(
                "{}/v4/spreadsheets/{}/values/{}:clear",
                sheets_base(),
                enc(&args.spreadsheet_id),
                enc(&args.range)
            );
            client
                .request(
                    "sheets",
                    "values.clear",
                    ctx.profile(),
                    Method::POST,
                    url,
                    Some(json!({})),
                )
                .await
        }
    }
}
