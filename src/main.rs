mod cli;
mod config;
mod config_commands;
mod error;
mod http;
mod input;
mod pagination;
mod secrets;
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
    let command_args = std::env::args().collect::<Vec<_>>();
    let cli = Cli::parse();
    match cli.command {
        cli::Command::Config(command) => config_commands::dispatch(cli.config.as_deref(), command),
        command => {
            let ctx = config::Context::load(
                cli.config.as_deref(),
                cli.profile.as_deref(),
                cli.secrets_file.as_deref(),
                cli.key_file.as_deref(),
            )?;
            let value = services::dispatch(&ctx, command).await?;
            Ok(pagination::annotate(value, &command_args))
        }
    }
}
