//!
//! This module contains the core functionality for the autoclean feature.
//!
use crate::{
    error::EuleError,
    store::KvStore,
    tasks::{cleanup_task::CleanupTask, worker_pool::WorkerPool},
    utils::{rate_limiter::RateLimiter, serializable_instant::SerializableInstant},
};
use miette::Result;
use poise::serenity_prelude::{ChannelId, GuildId, Http};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    sync::{Mutex, RwLock},
    time::Duration,
};

/// Manages cleanup tasks for Discord channels.
///
/// This struct is responsible for scheduling and executing cleanup tasks
/// across multiple Discord guilds and channels.
#[derive(Clone)]
pub struct AutocleanManager {
    /// Stores cleanup tasks, organized by guild and channel.
    tasks: Arc<RwLock<HashMap<GuildId, HashMap<ChannelId, CleanupTask>>>>,
    /// Optional worker pool for executing cleanup tasks.
    pub worker_pool: Option<Arc<WorkerPool>>,
    /// Key-value store for persisting tasks.
    kv_store: Arc<KvStore>,
    /// Mutex for ensuring thread-safe task saving.
    save_lock: Arc<Mutex<()>>,
}

/// Obfuscates an ID for logging purposes.
///
/// # Parameters
/// - `id`: The ID to obfuscate.
///
/// # Returns
/// An obfuscated version of the ID.
fn obfuscate_id(id: u64) -> String {
    format!("{:x}", id)
}

impl Default for AutocleanManager {
    fn default() -> Self {
        Self {
            tasks: Default::default(),
            worker_pool: None,
            kv_store: Arc::new(
                KvStore::new("eule_data/blobs/db").expect("Failed to create KvStore"),
            ),
            save_lock: Arc::new(Mutex::new(())),
        }
    }
}

impl AutocleanManager {
    pub fn new(kv_store: Arc<KvStore>) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            worker_pool: None,
            kv_store,
            save_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Adds a new cleanup task for a specific channel in a guild.
    ///
    /// # Parameters
    /// - `guild_id`: The ID of the guild where the task should be added.
    /// - `channel_id`: The ID of the channel to be cleaned.
    /// - `interval`: The time interval between cleanups.
    ///
    /// This method is safe to call from multiple threads as it uses a RwLock.
    pub async fn add_task(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
        interval: Duration,
    ) -> Result<()> {
        {
            let mut tasks = self.tasks.write().await;
            tasks
                .entry(guild_id)
                .or_insert_with(HashMap::new)
                .insert(channel_id, CleanupTask::new(interval).await);
        }
        self.save_tasks().await?;
        let obfuscated_guild = obfuscate_id(guild_id.get());
        let obfuscated_channel = obfuscate_id(channel_id.get());
        tracing::info!(
            "Added cleanup task for guild {} channel {} with interval {:?}",
            obfuscated_guild,
            obfuscated_channel,
            interval
        );
        Ok(())
    }

    /// Removes a cleanup task for a specific channel in a guild.
    ///
    /// # Parameters
    /// - `guild_id`: The ID of the guild where the task should be removed.
    /// - `channel_id`: The ID of the channel whose task should be removed.
    ///
    /// # Returns
    /// `true` if a task was removed, `false` if no task was found.
    pub async fn remove_task(&self, guild_id: GuildId, channel_id: ChannelId) -> Result<bool> {
        let mut tasks = self.tasks.write().await;
        let removed = if let Some(guild_tasks) = tasks.get_mut(&guild_id) {
            guild_tasks.remove(&channel_id).is_some()
        } else {
            false
        };
        if removed {
            self.save_tasks().await?;
            let obfuscated_guild = obfuscate_id(guild_id.get());
            let obfuscated_channel = obfuscate_id(channel_id.get());
            tracing::info!(
                "Removed cleanup task for guild {} channel {}",
                obfuscated_guild,
                obfuscated_channel
            );
        }
        Ok(removed)
    }

    /// Lists all cleanup tasks for a specific guild.
    ///
    /// # Parameters
    /// - `guild_id`: The ID of the guild whose tasks should be listed.
    ///
    /// # Returns
    /// A vector of tuples, each containing a channel ID and its cleanup interval.
    pub async fn list_tasks(&self, guild_id: GuildId) -> Vec<(ChannelId, Duration)> {
        let tasks = self.tasks.read().await;
        tasks
            .get(&guild_id)
            .map(|guild_tasks| {
                guild_tasks
                    .iter()
                    .map(|(channel_id, task)| (*channel_id, task.interval))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns the number of cleanup tasks for a specific guild.
    ///
    /// # Parameters
    /// - `guild_id`: The ID of the guild whose task count should be returned.
    /// # Returns
    /// The number of cleanup tasks for the specified guild.
    ///
    /// This method is safe to call from multiple threads as it uses a RwLock.
    pub async fn task_count(&self, guild_id: GuildId) -> usize {
        let tasks = self.tasks.read().await;
        tasks
            .get(&guild_id)
            .map(|guild_tasks| guild_tasks.len())
            .unwrap_or(0)
    }

    /// Saves the current task map to persistent storage.
    /// This method is called automatically by add_task and remove_task.
    ///
    /// # Returns
    /// A Result indicating success or failure of the save operation.
    ///
    pub async fn save_tasks(&self) -> Result<()> {
        let _lock = self.save_lock.lock().await;
        let tasks = self.tasks.read().await;
        let serialized = serde_json::to_string(&*tasks).map_err(EuleError::Serialization)?;
        self.kv_store.set("cleanup_tasks", &serialized).await?;
        Ok(())
    }

    /// Starts the AutocleanManager, initializing the worker pool.
    ///
    /// # Parameters
    /// - `http`: An Arc-wrapped Http client for making Discord API calls.
    ///
    /// This method spawns a new tokio task for scheduling cleanup operations.
    ///
    pub async fn start(&mut self, http: Arc<Http>) {
        let tasks = Arc::clone(&self.tasks);
        let worker_pool = Arc::new(WorkerPool::new(4, http.clone(), tasks.clone()));
        self.worker_pool = Some(Arc::clone(&worker_pool));

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let tasks_read = tasks.read().await;
                for (guild_id, guild_tasks) in tasks_read.iter() {
                    for (channel_id, task) in guild_tasks.iter() {
                        if task.is_due().await {
                            tracing::info!(
                                "Queueing cleanup task for guild {} channel {}",
                                guild_id,
                                channel_id
                            );
                            worker_pool.queue_task(*guild_id, *channel_id).await;
                        }
                    }
                }
            }
        });
    }

    /// Shuts down the AutocleanManager, stopping the worker pool.
    ///
    /// This method is safe to call from multiple threads, but should only be called once.
    ///
    pub async fn shutdown(&mut self) {
        if let Some(worker_pool) = self.worker_pool.take() {
            match Arc::try_unwrap(worker_pool) {
                Ok(pool) => pool.shutdown().await,
                Err(arc_pool) => {
                    tracing::warn!(
                        "Could not shutdown worker pool: Arc has {} strong references",
                        Arc::strong_count(&arc_pool) - 1
                    );
                }
            }
        }
    }

    /// Loads tasks from persistent storage into the current task map.
    ///
    /// # Returns
    /// A Result indicating success or failure of the load operation.
    ///
    pub async fn load_tasks(&self) -> Result<()> {
        let serialized = self.kv_store.get("cleanup_tasks").await?;
        if let Some(serialized) = serialized {
            let loaded_tasks: HashMap<GuildId, HashMap<ChannelId, CleanupTask>> =
                serde_json::from_str(&serialized).map_err(EuleError::Serialization)?;
            let mut tasks = self.tasks.write().await;
            *tasks = loaded_tasks;
            tracing::info!("Loaded {} guild tasks from persistent storage", tasks.len());
        } else {
            tracing::info!("No tasks found in persistent storage");
        }
        Ok(())
    }

    pub fn new_worker_pool(
        num_workers: usize,
        http: Arc<Http>,
        tasks: Arc<RwLock<HashMap<GuildId, HashMap<ChannelId, CleanupTask>>>>,
    ) -> Arc<WorkerPool> {
        Arc::new(WorkerPool::new(num_workers, http, tasks))
    }

    pub async fn worker_count(&self) -> usize {
        self.worker_pool
            .as_ref()
            .map(|pool| pool.worker_count())
            .unwrap_or(0)
    }
}

/// Performs the actual cleanup of messages in a channel.
///
/// # Parameters
/// - `http`: The Http client for making Discord API calls.
/// - `guild_id`: The ID of the guild where the cleanup is occurring.
/// - `channel_id`: The ID of the channel to be cleaned.
/// - `tasks`: The shared task map for updating task status.
///
/// # Returns
/// A Result indicating success or failure of the cleanup operation.
///
/// # Concurrency
/// This function is designed to be called concurrently by multiple workers.
///
pub async fn cleanup_channel(
    http: &Http,
    guild_id: GuildId,
    channel_id: ChannelId,
    tasks: &Arc<RwLock<HashMap<GuildId, HashMap<ChannelId, CleanupTask>>>>,
) -> Result<()> {
    let obfuscated_guild = obfuscate_id(guild_id.get());
    let obfuscated_channel = obfuscate_id(channel_id.get());
    tracing::info!(
        "Starting cleanup for channel {} in guild {}",
        obfuscated_channel,
        obfuscated_guild
    );

    let messages = match channel_id
        .messages(http, poise::serenity_prelude::GetMessages::new().limit(100))
        .await
    {
        Ok(msgs) => msgs,
        Err(e) => {
            tracing::error!(
                "Failed to fetch messages for channel {} in guild {}: {:?}",
                obfuscated_channel,
                obfuscated_guild,
                e
            );
            return Err(EuleError::from(e).into());
        }
    };

    tracing::info!(
        "Found {} messages to delete in channel {} of guild {}",
        messages.len(),
        obfuscated_channel,
        obfuscated_guild
    );

    let rate_limiter = RateLimiter::new(5, Duration::from_secs(10));
    let mut deleted_count = 0;

    for chunk in messages.chunks(100) {
        if rate_limiter.check().await.is_err() {
            tracing::warn!("Rate limit reached, waiting before next deletion attempt");
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        match channel_id.delete_messages(http, chunk).await {
            Ok(_) => {
                deleted_count += chunk.len();
                tracing::info!(
                    "Deleted {} messages in channel {} of guild {}",
                    chunk.len(),
                    obfuscated_channel,
                    obfuscated_guild
                );
            }
            Err(e) => {
                tracing::error!(
                    "Error deleting messages in channel {} of guild {}: {:?}",
                    obfuscated_channel,
                    obfuscated_guild,
                    e
                );
                return Err(EuleError::from(e).into());
            }
        }
    }

    // Update last_cleanup time
    let mut tasks_write = tasks.write().await;
    if let Some(guild_tasks) = tasks_write.get_mut(&guild_id) {
        if let Some(task) = guild_tasks.get_mut(&channel_id) {
            task.last_cleanup = SerializableInstant::now();
        }
    }

    tracing::info!(
        "Cleanup completed. Deleted {} out of {} messages in channel {} of guild {}",
        deleted_count,
        messages.len(),
        obfuscated_channel,
        obfuscated_guild
    );
    Ok(())
}
