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
        SheetsResource::Sheets(cmd) => tabs(client, ctx, cmd.action).await,
        SheetsResource::Values(cmd) => values(client, ctx, cmd.action).await,
    }
}

async fn spreadsheets(
    client: &ApiClient,
    ctx: &Context,
    action: SpreadsheetsAction,
) -> Result<Value, AppError> {
    match action {
        SpreadsheetsAction::Create(args) => {
            let operation = "spreadsheets.create";
            let mut body = json!({"properties": {"title": args.title}});
            // Omit `sheets` entirely when unspecified — an empty array is not the same as
            // absent, and only absence makes Google seed the default tab.
            if let Some(raw) = args.sheets.as_deref() {
                let titles = parse_tab_titles(raw, operation)?;
                body["sheets"] = titles
                    .iter()
                    .map(|title| json!({"properties": {"title": title}}))
                    .collect::<Vec<Value>>()
                    .into();
            }
            // Ask for the url back explicitly: it is what an agent hands to a human, and
            // the default response omits it.
            let url = format!(
                "{}/v4/spreadsheets?fields=spreadsheetId%2CspreadsheetUrl%2Cproperties.title%2Csheets.properties",
                sheets_base()
            );
            client
                .request(
                    "sheets",
                    operation,
                    ctx.profile(),
                    Method::POST,
                    url,
                    Some(body),
                )
                .await
        }
        SpreadsheetsAction::List(args) => {
            let page_token_param = args
                .page_token
                .as_deref()
                .map(|t| format!("&pageToken={}", enc(t)))
                .unwrap_or_default();
            let url = format!(
                "{}/drive/v3/files?q=mimeType%3D%27application%2Fvnd.google-apps.spreadsheet%27&fields=files(id%2Cname)%2CnextPageToken&pageSize=25{}",
                google_base(ctx.profile()),
                page_token_param
            );
            client
                .request(
                    "sheets",
                    "spreadsheets.list",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        SpreadsheetsAction::Get(args) => {
            let url = format!(
                "{}/v4/spreadsheets/{}?fields=sheets.properties",
                sheets_base(),
                enc(&args.spreadsheet_id)
            );
            client
                .request(
                    "sheets",
                    "spreadsheets.get",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
    }
}

/// Split a comma-separated tab list. Google rejects duplicate and empty tab titles, so
/// catch those here with a clearer message than a 400 would give.
fn parse_tab_titles(raw: &str, operation: &'static str) -> Result<Vec<String>, AppError> {
    let titles: Vec<String> = raw
        .split(',')
        .map(|title| title.trim().to_string())
        .filter(|title| !title.is_empty())
        .collect();
    if titles.is_empty() {
        return Err(AppError::invalid_input(
            "sheets",
            operation,
            "--sheets listed no usable names",
        ));
    }
    for (i, title) in titles.iter().enumerate() {
        if titles[..i].contains(title) {
            return Err(AppError::invalid_input(
                "sheets",
                operation,
                format!("duplicate sheet name {title:?}; tab names must be unique"),
            ));
        }
    }
    Ok(titles)
}

async fn tabs(
    client: &ApiClient,
    ctx: &Context,
    action: SheetsTabAction,
) -> Result<Value, AppError> {
    match action {
        SheetsTabAction::Add(args) => {
            let operation = "sheets.add";
            batch_update(
                client,
                ctx,
                operation,
                &args.spreadsheet_id,
                json!({"addSheet": {"properties": {"title": args.title}}}),
            )
            .await
        }
        SheetsTabAction::Delete(args) => {
            let operation = "sheets.delete";
            let existing = fetch_tabs(client, ctx, operation, &args.spreadsheet_id).await?;
            // A spreadsheet must keep at least one tab. Google enforces this too, but its
            // error names neither the spreadsheet nor the rule.
            if existing.len() == 1 {
                return Err(AppError::invalid_input(
                    "sheets",
                    operation,
                    format!(
                        "cannot delete {:?}: it is the only tab, and a spreadsheet must keep at least one",
                        args.title
                    ),
                ));
            }
            let sheet_id = resolve_tab_id(&existing, &args.title, operation)?;
            batch_update(
                client,
                ctx,
                operation,
                &args.spreadsheet_id,
                json!({"deleteSheet": {"sheetId": sheet_id}}),
            )
            .await
        }
        SheetsTabAction::Rename(args) => {
            let operation = "sheets.rename";
            let existing = fetch_tabs(client, ctx, operation, &args.spreadsheet_id).await?;
            let sheet_id = resolve_tab_id(&existing, &args.title, operation)?;
            if existing
                .iter()
                .any(|(id, title)| *id != sheet_id && title == &args.new_title)
            {
                return Err(AppError::invalid_input(
                    "sheets",
                    operation,
                    format!(
                        "another tab is already named {:?}; tab names must be unique",
                        args.new_title
                    ),
                ));
            }
            batch_update(
                client,
                ctx,
                operation,
                &args.spreadsheet_id,
                json!({
                    "updateSheetProperties": {
                        "properties": {"sheetId": sheet_id, "title": args.new_title},
                        "fields": "title",
                    }
                }),
            )
            .await
        }
    }
}

/// Send a single batchUpdate request. Every tab mutation is one request, so the plural
/// wrapper is kept out of the callers.
async fn batch_update(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    spreadsheet_id: &str,
    request: Value,
) -> Result<Value, AppError> {
    let url = format!(
        "{}/v4/spreadsheets/{}:batchUpdate",
        sheets_base(),
        enc(spreadsheet_id)
    );
    client
        .request(
            "sheets",
            operation,
            ctx.profile(),
            Method::POST,
            url,
            Some(json!({"requests": [request]})),
        )
        .await
}

/// Fetch every tab as `(sheetId, title)`.
///
/// deleteSheet and updateSheetProperties address a tab by its numeric sheetId, but an
/// agent only ever knows the title it can see, so the lookup happens here rather than
/// being pushed onto the caller.
async fn fetch_tabs(
    client: &ApiClient,
    ctx: &Context,
    operation: &'static str,
    spreadsheet_id: &str,
) -> Result<Vec<(i64, String)>, AppError> {
    let url = format!(
        "{}/v4/spreadsheets/{}?fields=sheets.properties(sheetId%2Ctitle)",
        sheets_base(),
        enc(spreadsheet_id)
    );
    let response = client
        .request("sheets", operation, ctx.profile(), Method::GET, url, None)
        .await?;
    Ok(response
        .get("sheets")
        .and_then(Value::as_array)
        .map(|sheets| {
            sheets
                .iter()
                .filter_map(|sheet| {
                    let properties = sheet.get("properties")?;
                    // sheetId is absent on the very first tab, where it is 0.
                    let id = properties
                        .get("sheetId")
                        .and_then(Value::as_i64)
                        .unwrap_or(0);
                    Some((id, properties.get("title")?.as_str()?.to_string()))
                })
                .collect()
        })
        .unwrap_or_default())
}

fn resolve_tab_id(
    tabs: &[(i64, String)],
    title: &str,
    operation: &'static str,
) -> Result<i64, AppError> {
    tabs.iter()
        .find(|(_, existing)| existing == title)
        .map(|(id, _)| *id)
        .ok_or_else(|| {
            let available: Vec<&str> = tabs.iter().map(|(_, t)| t.as_str()).collect();
            AppError::not_found(
                "sheets",
                operation,
                format!(
                    "no tab named {title:?}; spreadsheet has: {}",
                    available.join(", ")
                ),
            )
        })
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
                .request(
                    "sheets",
                    "values.get",
                    ctx.profile(),
                    Method::GET,
                    url,
                    None,
                )
                .await
        }
        ValuesAction::Update(args) => {
            let parsed_values: Value = serde_json::from_str(&args.values).map_err(|err| {
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
                .request(
                    "sheets",
                    "values.update",
                    ctx.profile(),
                    Method::PUT,
                    url,
                    Some(body),
                )
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

#[cfg(test)]
mod tests {
    use super::*;

    const OP: &str = "sheets.delete";

    #[test]
    fn tab_titles_are_split_and_trimmed() {
        assert_eq!(
            parse_tab_titles(" Q1 , Q2 ,Q3", OP).expect("titles"),
            ["Q1", "Q2", "Q3"]
        );
    }

    #[test]
    fn tab_titles_reject_duplicates_and_emptiness() {
        assert!(parse_tab_titles("", OP).is_err());
        assert!(parse_tab_titles(" , ", OP).is_err());
        assert!(parse_tab_titles("Q1,Q1", OP).is_err());
    }

    #[test]
    fn a_tab_is_resolved_to_its_numeric_id() {
        let tabs = vec![(0, "Sheet1".to_string()), (183, "Q2".to_string())];
        assert_eq!(resolve_tab_id(&tabs, "Q2", OP).expect("id"), 183);
        // The first tab's id is genuinely 0, which must not be confused with "not found".
        assert_eq!(resolve_tab_id(&tabs, "Sheet1", OP).expect("id"), 0);
    }

    #[test]
    fn an_unknown_tab_names_the_ones_that_exist() {
        let tabs = vec![(0, "Sheet1".to_string()), (183, "Q2".to_string())];
        let err = resolve_tab_id(&tabs, "Nope", OP).expect_err("must fail");
        assert!(err.to_string().contains("Sheet1, Q2"), "{err}");
    }
}
