//! Error types for Eule.
//!
//! This module defines the `EuleError` enum, which represents
//! all possible errors that can occur within Eule.
//! It uses the `miette` crate for improved error handling and diagnostics.

use miette::{Diagnostic, Report as MietteReport};
use owo_colors::OwoColorize;
use poise::serenity_prelude::Error as SerenityError;
use sled::Error as SledError;
use std::fmt;

/// Represents all possible errors in Eule.
#[derive(Debug, Diagnostic)]
pub enum EuleError {
    /// Represents errors from the Discord API.
    #[diagnostic(code(eule::discord_api))]
    DiscordApi(SerenityError),

    /// Represents errors from the database operations.
    #[diagnostic(code(eule::database))]
    Database(SledError),

    /// Represents the error when an invalid Discord token is provided.
    #[diagnostic(code(eule::authentication_failed))]
    AuthenticationFailed(String),

    /// Represents I/O errors.
    #[diagnostic(code(eule::io))]
    Io(std::io::Error),

    /// Represents errors during serialization or deserialization.
    #[diagnostic(code(eule::serialization))]
    Serialization(serde_json::Error),

    /// Represents errors when acquiring locks.
    #[diagnostic(code(eule::lock))]
    LockError(String),

    /// Represents the error when a guild-only command is used outside a guild.
    #[diagnostic(code(eule::not_in_guild))]
    NotInGuild,

    /// Represents the error when an invalid time unit is provided.
    #[diagnostic(code(eule::invalid_time_unit))]
    InvalidTimeUnit,

    /// Represents errors during the setup of the tracing subscriber.
    #[diagnostic(code(eule::tracing_setup))]
    TracingSetupFailed(String),

    /// Represents errors from the Poise framework.
    #[diagnostic(code(eule::poise_framework))]
    Poise(String),

    /// Represents errors from the Miette error reporting library.
    #[diagnostic(code(eule::miette))]
    Miette(MietteReport),
}

/// Conversion from MietteReport to EuleError
impl From<MietteReport> for EuleError {
    fn from(err: MietteReport) -> Self {
        EuleError::Miette(err)
    }
}

/// Conversion from SerenityError to EuleError
impl From<SerenityError> for EuleError {
    fn from(err: SerenityError) -> Self {
        EuleError::DiscordApi(err)
    }
}

/// Conversion from SledError to EuleError
impl From<SledError> for EuleError {
    fn from(err: SledError) -> Self {
        EuleError::Database(err)
    }
}

/// Implementation of the standard error trait for EuleError
impl std::error::Error for EuleError {}

/// Implement the `std::fmt::Display` trait for the `EuleError` enum.
impl fmt::Display for EuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EuleError::DiscordApi(e) => write!(f, "{}: {}", "Discord API error".red().bold(), e),
            EuleError::Database(e) => write!(f, "{}: {}", "Database error".red().bold(), e),
            EuleError::AuthenticationFailed(e) => {
                write!(f, "{}: {}", "Authentication failed".red().bold(), e)
            }
            EuleError::Io(e) => write!(f, "{}: {}", "IO error".red().bold(), e),
            EuleError::Serialization(e) => {
                write!(f, "{}: {}", "Serialization error".red().bold(), e)
            }
            EuleError::LockError(e) => write!(f, "{}: {}", "Lock error".red().bold(), e),
            EuleError::NotInGuild => write!(f, "{}", "Not in a guild".yellow().bold()),
            EuleError::InvalidTimeUnit => write!(f, "{}", "Invalid time unit".yellow().bold()),
            EuleError::TracingSetupFailed(e) => {
                write!(f, "{}: {}", "Tracing setup failed".red().bold(), e)
            }
            EuleError::Poise(e) => write!(f, "{}: {}", "Poise framework error".red().bold(), e),
            EuleError::Miette(e) => write!(f, "{}: {}", "Miette error".red().bold(), e),
        }
    }
}

/// Helper function to create a colorful diagnostic report
///
/// This function takes an `EuleError` and an optional help message, and creates
/// a `MietteReport` with colorful formatting.
///
/// # Arguments
///
/// * `error` - The `EuleError` to be wrapped in the report
/// * `help` - An optional help message to be included in the report
///
/// # Returns
///
/// Returns a `MietteReport` containing the error and optional help message
///
/// # Examples
///
/// ```
/// use eule::error::{EuleError, create_report};
///
/// let error = EuleError::NotInGuild;
/// let report = create_report(error, Some("This command can only be used in a guild."));
/// ```
pub fn create_report(error: EuleError, help: Option<&str>) -> MietteReport {
    let mut report = MietteReport::new(error);
    if let Some(help_text) = help {
        report = report.context(help_text.on_bright_blue().to_string());
    }
    report
}
