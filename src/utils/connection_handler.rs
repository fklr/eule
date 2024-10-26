//! Connection Handler for Eule
//!
//! This module provides functionality for managing Discord bot connections,
//! including automatic reconnection with exponential backoff and runtime
//! reconnection attempts.

use crate::error::EuleError;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    time::{Duration, Instant},
};

/// Interface defining required bot functionality for connection handling.
#[async_trait]
pub trait BotInterface: Send + Sync {
    /// Attempts to establish a connection to Discord.
    async fn connect(&self) -> Result<(), EuleError>;

    /// Runs the main bot loop.
    async fn run(&self) -> Result<(), EuleError>;
}

/// Represents the different states of the connection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionState {
    /// The bot is connected to Discord.
    Connected,
    /// The bot is disconnected from Discord.
    Disconnected,
    /// The bot is attempting to reconnect to Discord.
    Reconnecting,
}

/// Commands that can be sent to the ConnectionHandler.
#[derive(Debug)]
pub enum ConnectionCommand {
    /// Command to trigger a reconnection attempt.
    Reconnect,
    /// Command to shut down the connection handler.
    Shutdown,
}

/// Handles the bot's connection to Discord, including automatic reconnection.
pub struct ConnectionHandler<B: BotInterface> {
    /// The bot instance being managed.
    bot: Arc<B>,
    /// The maximum duration to wait between reconnection attempts.
    pub max_retry_interval: Duration,
    /// The current state of the connection.
    state: ConnectionState,
    /// The timestamp of the last connection attempt.
    last_connection_attempt: Instant,
    /// Receiver for connection commands.
    command_rx: Receiver<ConnectionCommand>,
    /// Sender for connection commands.
    command_tx: Sender<ConnectionCommand>,
}

impl<B: BotInterface> ConnectionHandler<B> {
    /// Creates a new ConnectionHandler.
    ///
    /// # Arguments
    ///
    /// * `bot` - An Arc-wrapped instance implementing BotInterface
    /// * `max_retry_interval` - The maximum duration to wait between reconnection attempts
    ///
    /// # Returns
    ///
    /// A new instance of ConnectionHandler
    pub fn new(bot: Arc<B>, max_retry_interval: Duration) -> Self {
        let (tx, rx) = channel(100);
        Self {
            bot,
            max_retry_interval,
            state: ConnectionState::Disconnected,
            last_connection_attempt: Instant::now(),
            command_rx: rx,
            command_tx: tx,
        }
    }

    /// Returns a sender for sending commands to the ConnectionHandler.
    pub fn get_command_sender(&self) -> Sender<ConnectionCommand> {
        self.command_tx.clone()
    }

    /// Returns the current connection state.
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    /// Runs the connection handler, managing reconnection attempts and handling commands.
    ///
    /// This method will continue running until a shutdown command is received
    /// or an unrecoverable error occurs.
    ///
    /// # Returns
    ///
    /// A Result indicating success or containing an EuleError if an unrecoverable error occurred.
    pub async fn run(&mut self) -> Result<(), EuleError> {
        loop {
            tokio::select! {
                biased;  // Check commands first

                // Check for commands
                Some(cmd) = self.command_rx.recv() => {
                    match cmd {
                        ConnectionCommand::Reconnect => {
                            self.state = ConnectionState::Reconnecting;
                        }
                        ConnectionCommand::Shutdown => {
                            return Ok(());
                        }
                    }
                }

                // Handle connection state
                _ = tokio::time::sleep(Duration::from_millis(10)) => {
                    match self.state {
                        ConnectionState::Disconnected | ConnectionState::Reconnecting => {
                            match self.bot.connect().await {
                                Ok(_) => {
                                    self.state = ConnectionState::Connected;
                                }
                                Err(_) => {
                                    self.state = ConnectionState::Reconnecting;
                                }
                            }
                        }
                        ConnectionState::Connected => {
                            if (self.bot.run().await).is_err() {
                                self.state = ConnectionState::Reconnecting;
                            }
                        }
                    }
                }
            }
        }
    }
}
