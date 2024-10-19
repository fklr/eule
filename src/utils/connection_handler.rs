//! Connection Handler for Eule
//!
//! This module provides a utility class for managing the bot's connection to Discord,
//! including automatic reconnection with exponential backoff and runtime reconnection attempts.

use crate::bot::Bot;
use crate::error::EuleError;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::{sleep, Duration, Instant};

/// Represents the different states of the connection.
#[derive(Debug)]
enum ConnectionState {
    Connected,
    Disconnected,
    Reconnecting,
}

/// Handles the bot's connection to Discord, including automatic reconnection.
pub struct ConnectionHandler {
    bot: Arc<Bot>,
    pub max_retry_interval: Duration,
    state: ConnectionState,
    last_connection_attempt: Instant,
    command_rx: Receiver<ConnectionCommand>,
    command_tx: Sender<ConnectionCommand>,
}

/// Commands that can be sent to the ConnectionHandler.
#[derive(Debug)]
pub enum ConnectionCommand {
    Reconnect,
    Shutdown,
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
                                println!("Bot connected successfully.");
                                self.state = ConnectionState::Connected;
                                backoff = Duration::from_secs(1);
                            }
                            Err(e) => {
                                eprintln!(
                                    "Connection attempt failed: {}. Retrying in {:?}...",
                                    e, backoff
                                );
                                self.state = ConnectionState::Reconnecting;
                                self.last_connection_attempt = Instant::now();
                                backoff = std::cmp::min(backoff * 2, self.max_retry_interval);
                            }
                        }
                    }
                }
                ConnectionState::Connected => {
                    if let Err(e) = self.bot.run().await {
                        eprintln!("Bot disconnected: {}. Attempting to reconnect...", e);
                        self.state = ConnectionState::Reconnecting;
                        self.last_connection_attempt = Instant::now();
                    }
                }
            }

            // Check for incoming commands
            match self.command_rx.try_recv() {
                Ok(ConnectionCommand::Reconnect) => {
                    println!("Received reconnection command.");
                    self.state = ConnectionState::Reconnecting;
                    self.last_connection_attempt = Instant::now() - backoff; // Attempt immediate reconnection
                }
                Ok(ConnectionCommand::Shutdown) => {
                    println!("Received shutdown command. Exiting.");
                    break Ok(());
                }
                Err(_) => {} // No command received, continue as normal
            }

            // Small delay to prevent tight looping
            sleep(Duration::from_millis(100)).await;
        }
    }
}
