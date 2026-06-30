pub(crate) mod apollo;
pub(crate) mod bitbucket;
pub(crate) mod calendar;
pub(crate) mod confluence;
pub(crate) mod email;
pub(crate) mod generic_request;
pub(crate) mod github;
pub(crate) mod hubspot;
pub(crate) mod jira;
pub(crate) mod pipedrive;
pub(crate) mod shared;
pub(crate) mod sheets;

use serde_json::Value;

use crate::{cli::*, config::Context, error::AppError, http::ApiClient};

pub async fn dispatch(ctx: &Context, command: Command) -> Result<Value, AppError> {
    let client = ApiClient::new()?;
    match command {
        Command::Jira(command) => jira::dispatch(&client, ctx, command).await,
        Command::Confluence(command) => confluence::dispatch(&client, ctx, command).await,
        Command::Bitbucket(command) => bitbucket::dispatch(&client, ctx, command).await,
        Command::Github(command) => github::dispatch(&client, ctx, command).await,
        Command::Hubspot(command) => hubspot::dispatch(&client, ctx, command).await,
        Command::Email(command) => email::dispatch(&client, ctx, command).await,
        Command::Calendar(command) => calendar::dispatch(&client, ctx, command).await,
        Command::Pipedrive(command) => pipedrive::dispatch(&client, ctx, command).await,
        Command::Apollo(command) => apollo::dispatch(&client, ctx, command).await,
        Command::Sheets(command) => sheets::dispatch(&client, ctx, command).await,
        Command::Config(_) => unreachable!("config commands are dispatched before context loading"),
        Command::Skills(_) => unreachable!("skills commands are dispatched before context loading"),
        Command::Secrets(command) => crate::secrets::dispatch(ctx, command),
    }
}
