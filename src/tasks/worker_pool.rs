use crate::tasks::{
    autoclean_manager::cleanup_channel,
    cleanup_task::CleanupTask
};
use poise::serenity_prelude::{ChannelId, GuildId, Http};
use std::{collections::HashMap, sync::Arc};
use tokio::{sync::{mpsc, RwLock}, task::JoinHandle};

pub struct WorkerCleanupTask {
    guild_id: GuildId,
    channel_id: ChannelId,
}

/// Manages a pool of workers for executing tasks.
pub struct WorkerPool {
    /// Channel for sending tasks to workers.
    sender: mpsc::Sender<WorkerCleanupTask>,
    /// Handles for worker threads.
    workers: Vec<JoinHandle<()>>,
    /// Number of worker threads in the pool.
    worker_count: usize,
}

impl WorkerPool {
    /// Creates a new WorkerPool with the specified number of workers.
    ///
    /// # Parameters
    /// - `num_workers`: The number of worker threads to spawn.
    /// - `http`: An Arc-wrapped Http client for making Discord API calls.
    /// - `tasks`: The shared task map for updating task status.
    ///
    /// # Returns
    /// A new WorkerPool instance.
    pub fn new(
        num_workers: usize,
        http: Arc<Http>,
        tasks: Arc<RwLock<HashMap<GuildId, HashMap<ChannelId, CleanupTask>>>>,
    ) -> Self {
        let (sender, receiver) = mpsc::channel::<WorkerCleanupTask>(100);
        let receiver = Arc::new(tokio::sync::Mutex::new(receiver));

        let mut workers = Vec::with_capacity(num_workers);

        for _ in 0..num_workers {
            let worker_receiver = Arc::clone(&receiver);
            let worker_http = Arc::clone(&http);
            let worker_tasks = Arc::clone(&tasks);

            let handle = tokio::spawn(async move {
                while let Some(task) = worker_receiver.lock().await.recv().await {
                    tracing::info!(
                        "Worker processing cleanup task for guild {} channel {}",
                        task.guild_id,
                        task.channel_id
                    );
                    if let Err(e) =
                        cleanup_channel(&worker_http, task.guild_id, task.channel_id, &worker_tasks)
                            .await
                    {
                        tracing::error!(
                            "Error cleaning up channel {} in guild {}: {:?}",
                            task.channel_id,
                            task.guild_id,
                            e
                        );
                    }
                }
            });

            workers.push(handle);
        }

        WorkerPool {
            sender,
            workers,
            worker_count: num_workers,
        }
    }

    pub fn worker_count(&self) -> usize {
        self.worker_count
    }

    /// Queues a task for execution.
    ///
    /// # Parameters
    /// - `guild_id`: The ID of the guild where the task should occur.
    /// - `channel_id`: The ID of the channel where the task should occur.
    ///
    /// This method is safe to call from multiple threads.
    pub async fn queue_task(&self, guild_id: GuildId, channel_id: ChannelId) {
        let task = WorkerCleanupTask {
            guild_id,
            channel_id,
        };
        if let Err(e) = self.sender.send(task).await {
            tracing::error!("Failed to queue cleanup task: {:?}", e);
        }
    }

    /// Shuts down the worker pool, stopping all worker threads.
    ///
    /// This method should only be called once, typically when shutting down the bot.
    pub async fn shutdown(self) {
        drop(self.sender);
        for worker in self.workers {
            worker.await.unwrap();
        }
    }
}
