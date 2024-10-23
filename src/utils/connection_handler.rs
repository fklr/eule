//! Connection Handler for Eule
//!
//! This module provides a utility struct for managing the bot's connection to Discord,
//! including automatic reconnection with exponential backoff and runtime reconnection attempts.

use crate::bot::Bot;
use crate::error::EuleError;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::{Duration, Instant};

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
pub struct ConnectionHandler {
    /// The bot instance being managed.
    bot: Arc<Bot>,
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

impl ConnectionHandler {
    /// Creates a new ConnectionHandler.
    ///
    /// # Arguments
    ///
    /// * `bot` - An Arc-wrapped instance of the Bot.
    /// * `max_retry_interval` - The maximum duration to wait between reconnection attempts.
    ///
    /// # Returns
    ///
    /// A new instance of ConnectionHandler.
    pub fn new(bot: Arc<Bot>, max_retry_interval: Duration) -> Self {
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

    /// Runs the bot, handling disconnections and reconnections.
    ///
    /// This method will continue running until a shutdown command is received
    /// or an unrecoverable error occurs.
    ///
    /// # Returns
    ///
    /// A Result indicating success or containing an EuleError if an unrecoverable error occurred.
    pub async fn run(&mut self) -> Result<(), EuleError> {
        let mut backoff = Duration::from_secs(1);

        loop {
            match self.state {
                ConnectionState::Disconnected | ConnectionState::Reconnecting => {
                    if self.last_connection_attempt.elapsed() >= backoff {
                        match self.bot.connect().await {
                            Ok(_) => {
                                tracing::info!("Bot connected successfully.");
                                self.state = ConnectionState::Connected;
                                backoff = Duration::from_secs(1);
                            }
                            Err(e) => {
                                tracing::error!("Connection attempt failed: {}. Retrying in {:?}...", e, backoff);
                                self.state = ConnectionState::Reconnecting;
                                self.last_connection_attempt = Instant::now();
                                backoff = std::cmp::min(backoff * 2, self.max_retry_interval);
                            }
                        }
                    }
                }
                ConnectionState::Connected => {
                    if let Err(e) = self.bot.run().await {
                        tracing::error!("Bot disconnected: {}. Attempting to reconnect...", e);
                        self.state = ConnectionState::Reconnecting;
                        self.last_connection_attempt = Instant::now();
                    }
                }
            }

            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                command = self.command_rx.recv() => {
                    match command {
                        Some(ConnectionCommand::Reconnect) => {
                            tracing::info!("Received reconnection command.");
                            self.state = ConnectionState::Reconnecting;
                            self.last_connection_attempt = Instant::now() - backoff;
                        }
                        Some(ConnectionCommand::Shutdown) => {
                            tracing::info!("Received shutdown command. Exiting.");
                            break Ok(());
                        }
                        None => break Ok(()),
                    }
                }
            }
        }
    }
}
