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
use std::{
    sync::atomic::{AtomicBool, AtomicUsize},
    sync::Arc,
    time::SystemTime,
};

pub mod commands;
pub mod error;
pub mod store;
pub mod tasks;
pub mod utils;

pub use crate::error::EuleError;

/// Represents the shared data accessible by all command handlers and bot components.
///
/// This structure encapsulates the core components and state of Eule,
/// providing a centralized way to access and manage shared resources.
pub struct Data {
    /// Manager for autoclean tasks, handling scheduled message deletions.
    pub autoclean_manager: tasks::AutocleanManager,

    /// Key-value store for persistent data storage.
    pub kv_store: Arc<store::KvStore>,

    /// Reference to the main Bot instance.
    pub bot: Arc<Bot>,

    /// Atomic flag indicating whether the bot is currently connected to Discord.
    pub is_connected: AtomicBool,

    /// Atomic counter tracking the number of connection attempts made.
    pub connection_attempts: AtomicUsize,
}

impl Data {
    /// Creates a new instance of `Data` with the provided components.
    ///
    /// This constructor initializes a new `Data` structure, setting up the shared
    /// state for the bot. It sets the initial connection status to false and
    /// the connection attempts to zero.
    ///
    /// # Arguments
    ///
    /// * `autoclean_manager` - The manager handling autoclean tasks.
    /// * `kv_store` - Arc-wrapped KvStore for persistent storage.
    /// * `bot` - Arc-wrapped Bot instance.
    ///
    /// # Returns
    ///
    /// A new `Data` instance with initialized components and default state values.
    pub fn new(
        autoclean_manager: tasks::AutocleanManager,
        kv_store: Arc<store::KvStore>,
        bot: Arc<Bot>,
    ) -> Self {
        Self {
            autoclean_manager,
            kv_store,
            bot,
            is_connected: AtomicBool::new(false),
            connection_attempts: AtomicUsize::new(0),
        }
    }
}

/// Type alias for the context used in command handlers.
///
/// This type combines the Poise context with the `Data` structure
/// and `EuleError` type, providing a convenient way to access bot state
/// and handle errors in command implementations.
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
