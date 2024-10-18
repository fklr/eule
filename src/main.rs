//! # Eule - Einfache Uneinigkeit Leichte Replika 🦉
//!
//! This is the main entry point for Eule, a server management bot for Discord.
//!
//! ## Features (at the moment):
//! - Automatic message cleaning at scheduled intervals (configurable via Discord commands)
//! - On-demand message cleaning
//! - Robust error handling and logging
//!
//! ## Technical Details:
//! - Built using the Serenity library for Discord API interaction
//! - Uses the Poise framework for command handling
//! - Employs asynchronous Rust with the tokio runtime
//! - Utilizes jemalloc for optimized memory allocation
//!
//! ## Main Components:
//! 1. Logging setup using tracing and tracing-subscriber
//! 2. Command-line interface using clap
//! 3. Bot instantiation and execution
//! 4. Error handling with miette
//!
//! This executable is responsible for setting up the environment, parsing command-line arguments,
//! initializing the bot, and running it or performing maintenance operations like token deletion.

use clap::Command;
use eule::{
    error::{create_report, EuleError},
    Bot,
};
use jemallocator::Jemalloc;
use miette::Result;
use std::env::{set_var, var};
use tracing::subscriber::set_global_default;
use tracing_appender::rolling::daily;
use tracing_subscriber::{fmt, fmt::time::UtcTime, EnvFilter};

// Use jemalloc as the global allocator for improved performance
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging configuration
    setup_logging()?;

    // Parse command-line arguments and execute appropriate action
    execute_cli_command().await?;

    Ok(())
}

/// Sets up the logging configuration for the application.
///
/// This function configures the logging level, output format, and destination.
/// It uses environment variables or default values to determine the log levels.
fn setup_logging() -> Result<()> {
    // Read log levels from Cargo.toml if RUST_LOG is not set
    if var("RUST_LOG").is_err() {
        let default_level =
            option_env!("CARGO_PKG_METADATA_EULE_DEFAULT_LOG_LEVEL").unwrap_or("info");
        let eule_level = option_env!("CARGO_PKG_METADATA_EULE_EULE_LOG_LEVEL").unwrap_or("debug");
        set_var("RUST_LOG", format!("{},eule={}", default_level, eule_level));
    }

    // Set up daily log file rotation
    let file_appender = daily("logs", "eule.log");
    let timer = UtcTime::rfc_3339();

    // Configure the logging subscriber
    let subscriber = fmt()
        .with_env_filter(EnvFilter::from_env("RUST_LOG"))
        .with_writer(file_appender)
        .with_timer(timer)
        .with_target(false)
        .with_level(true)
        .with_ansi(false)
        .with_file(true)
        .with_line_number(true)
        .finish();

    // Set the configured subscriber as the global default
    set_global_default(subscriber).map_err(|e| {
        create_report(
            EuleError::TracingSetupFailed(e.to_string()),
            Some("Check your logging configuration"),
        )
    })?;

    Ok(())
}

/// Parses command-line arguments and executes the appropriate action.
///
/// This function sets up the CLI, parses the arguments, and either
/// runs the bot or performs maintenance operations like deleting the Discord token.
async fn execute_cli_command() -> Result<()> {
    let matches = Command::new("Eule")
        .version("1.0")
        .author("@ovnanova")
        .about("Einfache Uneinigkeit Leichte Replika 🦉")
        .subcommand(Command::new("delete-token").about("Delete the stored Discord token"))
        .get_matches();

    match matches.subcommand() {
        Some(("delete-token", _)) => delete_token().await,
        _ => run_bot().await,
    }
}

/// Deletes the stored Discord token.
///
/// This function creates a new Bot instance and calls its `delete_token` method.
/// It's used for maintenance purposes when the token needs to be removed or reset.
async fn delete_token() -> Result<()> {
    let bot = Bot::new().await.map_err(|e| {
        create_report(
            EuleError::Poise(format!("Failed to create bot instance: {}", e)),
            Some("Check your bot configuration"),
        )
    })?;

    bot.delete_token().await.map_err(|e| {
        create_report(
            EuleError::Poise(format!("Failed to delete token: {}", e)),
            Some("Check your file permissions"),
        )
    })?;

    println!("Discord token has been deleted.");
    Ok(())
}

/// Creates a new Bot instance and runs it.
///
/// This function is the main entry point for starting Eule.
/// It creates a new Bot instance and calls its `run` method to start the bot's operation.
async fn run_bot() -> Result<()> {
    let bot = Bot::new().await.map_err(|e| {
        create_report(
            EuleError::Poise(format!("Failed to create bot instance: {}", e)),
            Some("Check your bot configuration"),
        )
    })?;

    bot.run().await.map_err(|e| {
        create_report(
            EuleError::Poise(format!("Failed to run bot: {}", e)),
            Some("Check the logs for more details"),
        )
    })?;

    Ok(())
}
