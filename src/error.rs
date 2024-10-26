//! Error types for Eule.
//!
//! This module defines the `EuleError` enum, which represents
//! all possible errors that can occur within Eule.
//! It uses the `miette` crate for improved error handling and diagnostics.

use miette::{Diagnostic, Report as MietteReport};
use owo_colors::OwoColorize;
use poise::serenity_prelude::Error as SerenityError;
use serde_json::Error as SerdeError;
use sled::Error as SledError;
use std::{fmt, io::Error as IoError};
use tokio::{sync::mpsc::error::SendError, task::JoinError};

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
    Io(IoError),

    /// Represents errors during serialization or deserialization.
    #[diagnostic(code(eule::serialization))]
    Serialization(SerdeError),

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

    /// Represents errors during key derivation.
    #[diagnostic(code(eule::key_derivation))]
    KeyDerivationError(String),

    /// Represents errors during encryption.
    #[diagnostic(code(eule::encryption))]
    EncryptionError(String),

    /// Represents errors during decryption.
    #[diagnostic(code(eule::decryption))]
    DecryptionError(String),

    /// Represents errors related to connection handling.
    #[diagnostic(code(eule::connection))]
    Connection(ConnectionError),
}

/// Conversion from std::io::Error to EuleError
impl From<std::io::Error> for EuleError {
    fn from(err: std::io::Error) -> Self {
        EuleError::Io(err)
    }
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

/// Conversion from tokio's SendError to EuleError
impl<T> From<SendError<T>> for EuleError {
    fn from(err: SendError<T>) -> Self {
        EuleError::Connection(ConnectionError::CommandSendError(err.to_string()))
    }
}

/// Conversion from tokio's JoinError to EuleError
impl From<JoinError> for EuleError {
    fn from(err: JoinError) -> Self {
        EuleError::Connection(ConnectionError::TaskJoinError(err.to_string()))
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
            EuleError::KeyDerivationError(e) => {
                write!(f, "{}: {}", "Key derivation error".red().bold(), e)
            }
            EuleError::EncryptionError(e) => {
                write!(f, "{}: {}", "Encryption error".red().bold(), e)
            }
            EuleError::DecryptionError(e) => {
                write!(f, "{}: {}", "Decryption error".red().bold(), e)
            }
            EuleError::Connection(e) => write!(f, "{}: {}", "Connection error".red().bold(), e),
        }
    }
}

/// Represents errors specific to the connection handling process.
#[derive(Clone, Debug, Diagnostic)]
pub enum ConnectionError {
    /// Represents a failed connection attempt.
    #[diagnostic(code(eule::connection::failed_attempt))]
    FailedConnectionAttempt(String),

    /// Represents an error when the maximum retry limit is reached.
    #[diagnostic(code(eule::connection::max_retries_reached))]
    MaxRetriesReached,

    /// Represents an error when trying to send a command to the connection handler.
    #[diagnostic(code(eule::connection::command_send_error))]
    CommandSendError(String),

    /// Represents an error when trying to receive a command in the connection handler.
    #[diagnostic(code(eule::connection::command_receive_error))]
    CommandReceiveError(String),

    /// Represents an unexpected shutdown of the connection.
    #[diagnostic(code(eule::connection::unexpected_shutdown))]
    UnexpectedShutdown,

    /// Represents an error when trying to join a task.
    #[diagnostic(code(eule::connection::task_join_error))]
    TaskJoinError(String),

    /// Error from the handler itself.
    #[diagnostic(code(eule::connection::handler_error))]
    HandlerError(String),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::FailedConnectionAttempt(msg) => {
                write!(f, "Failed connection attempt: {}", msg)
            }
            ConnectionError::MaxRetriesReached => {
                write!(f, "Maximum number of retry attempts reached")
            }
            ConnectionError::CommandSendError(msg) => write!(f, "Failed to send command: {}", msg),
            ConnectionError::CommandReceiveError(msg) => {
                write!(f, "Failed to receive command: {}", msg)
            }
            ConnectionError::UnexpectedShutdown => {
                write!(f, "Connection handler unexpectedly shut down")
            }
            ConnectionError::TaskJoinError(msg) => {
                write!(f, "Failed to join task: {}", msg)
            }
            ConnectionError::HandlerError(msg) => {
                write!(f, "Handler error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConnectionError {}

// Add a conversion from ConnectionError to EuleError
impl From<ConnectionError> for EuleError {
    fn from(err: ConnectionError) -> Self {
        EuleError::Connection(err)
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
