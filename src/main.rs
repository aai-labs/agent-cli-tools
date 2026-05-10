mod cli;
mod config;
mod error;
mod http;
mod input;
mod services;

use std::process::ExitCode;

use clap::Parser;
use cli::Cli;
use error::AppError;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(value) => {
            match serde_json::to_string_pretty(&value) {
                Ok(rendered) => println!("{rendered}"),
                Err(err) => {
                    let app_err = AppError::internal("output", "serialize", err.to_string());
                    eprintln!("{}", app_err.to_json_line());
                    return app_err.exit_code();
                }
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{}", err.to_json_line());
            err.exit_code()
        }
    }
}

async fn run() -> Result<serde_json::Value, AppError> {
    let cli = Cli::parse();
    let ctx = config::Context::load(cli.config.as_deref(), cli.profile.as_deref())?;
    services::dispatch(&ctx, cli.command).await
}
