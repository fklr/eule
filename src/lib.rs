//! Eule - Einfache Uneinigkeit Leichte Replika 🦉
//!
//! Core functionality for Eule, designed for automated server tasks.
//!
//! This module provides shared components and functionality used by Eule,
//! including command handling, database operations, and error management.
//!
//! This lib.rs file allows for code organization
//! and potential reuse in tests or benchmarks.

use poise::serenity_prelude as serenity;
use std::{sync::atomic::AtomicUsize, sync::Arc, time::SystemTime};

pub mod commands;
pub mod error;
pub mod store;
pub mod tasks;
pub mod utils;

pub use crate::error::EuleError;

/// Represents the shared data accessible by all command handlers.
pub struct Data {
    /// Manager for autoclean tasks.
    pub autoclean_manager: tasks::AutocleanManager,
    /// Key-value store for persistent data.
    pub kv_store: Arc<store::KvStore>,
    /// Reference to the main Bot instance.
    pub bot: Arc<Bot>,
}

/// Type alias for the context used in command handlers.
pub type Context<'a> = poise::Context<'a, Data, EuleError>;

/// Represents Eule's start time.
pub struct StartTime;

impl serenity::prelude::TypeMapKey for StartTime {
    type Value = SystemTime;
}

/// Represents the count of autoclean tasks.
pub struct AutocleanTaskCount;

impl serenity::prelude::TypeMapKey for AutocleanTaskCount {
    type Value = AtomicUsize;
}

mod bot;
pub use bot::Bot;

// Re-export only the necessary items for the main executable
pub use commands::autoclean::{add, autoclean, list, remove};
pub use commands::clean::clean;
pub use commands::status::status;
pub use commands::workers::workers;
