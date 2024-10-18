use eule::tasks::{AutocleanManager, CleanupTask};
use poise::serenity_prelude::{ChannelId, GuildId, Http, MessageId};
use std::{collections::HashMap, sync::Arc};
use tokio::{sync::RwLock, time::Duration};

#[async_trait::async_trait]
impl HttpClientTrait for Http {
    async fn delete_messages<T: AsRef<MessageId> + Send + Sync + 'static>(
        &self,
        _channel_id: ChannelId,
        _message_ids: &[T],
    ) -> Result<(), poise::serenity_prelude::Error> {
        Ok(())
    }
}

#[async_trait::async_trait]
pub trait HttpClientTrait {
    async fn delete_messages<T: AsRef<MessageId> + Send + Sync + 'static>(
        &self,
        channel_id: ChannelId,
        message_ids: &[T],
    ) -> Result<(), poise::serenity_prelude::Error>;
}

#[tokio::test]
async fn test_worker_pool_creation() {
    let http = Arc::new(Http::new("test_token"));
    let tasks = Arc::new(RwLock::new(HashMap::new()));

    let worker_pool = AutocleanManager::new_worker_pool(4, http, tasks);

    assert_eq!(worker_pool.worker_count(), 4);
}

#[tokio::test]
async fn test_worker_pool_task_execution() {
    let http = Arc::new(Http::new("fake_token"));
    let tasks = Arc::new(RwLock::new(HashMap::new()));
    let worker_pool = Arc::new(AutocleanManager::new_worker_pool(1, http, tasks.clone()));

    let guild_id = GuildId::new(1);
    let channel_id = ChannelId::new(1);

    tasks.write().await.insert(guild_id, HashMap::new());
    tasks.write().await.get_mut(&guild_id).unwrap().insert(
        channel_id,
        CleanupTask::new(Duration::from_secs(3600)).await,
    );

    worker_pool.queue_task(guild_id, channel_id).await;

    // Allow some time for the task to be processed
    tokio::time::sleep(Duration::from_millis(100)).await;

    let tasks_read = tasks.read().await;
    let guild_tasks = tasks_read.get(&guild_id).unwrap();
    let channel_task = guild_tasks.get(&channel_id).unwrap();

    // Check if the last_cleanup time has been updated
    assert!(channel_task.last_cleanup.elapsed() < Duration::from_secs(1));
}

#[tokio::test]
async fn test_worker_pool_concurrency() {
    let http = Arc::new(Http::new("fake_token"));
    let tasks = Arc::new(RwLock::new(HashMap::new()));
    let worker_pool = Arc::new(AutocleanManager::new_worker_pool(4, http, tasks.clone()));

    for i in 1..=10 {
        let guild_id = GuildId::new(1);
        let channel_id = ChannelId::new(i);
        tasks
            .write()
            .await
            .entry(guild_id)
            .or_insert_with(HashMap::new)
            .insert(
                channel_id,
                CleanupTask::new(Duration::from_secs(3600)).await,
            );
        worker_pool.queue_task(guild_id, channel_id).await;
    }

    // Wait for all tasks to complete
    tokio::time::sleep(Duration::from_secs(1)).await;

    let tasks_read = tasks.read().await;
    let guild_tasks = tasks_read.get(&GuildId::new(1)).unwrap();
    assert_eq!(guild_tasks.len(), 10);

    for (_, task) in guild_tasks.iter() {
        assert!(task.last_cleanup.elapsed() < Duration::from_secs(2));
    }
}

#[tokio::test]
async fn test_worker_pool_shutdown() {
    let http = Arc::new(Http::new("fake_token"));
    let tasks = Arc::new(RwLock::new(HashMap::new()));
    let worker_pool = AutocleanManager::new_worker_pool(4, http, tasks.clone());

    for i in 1..=10 {
        let guild_id = GuildId::new(1);
        let channel_id = ChannelId::new(i);
        tasks
            .write()
            .await
            .entry(guild_id)
            .or_insert_with(HashMap::new)
            .insert(
                channel_id,
                CleanupTask::new(Duration::from_secs(3600)).await,
            );
        worker_pool.queue_task(guild_id, channel_id).await;
    }

    // Allow some time for tasks to be processed
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Shutdown the worker pool
    drop(worker_pool);

    // Verify that all tasks have been processed
    let tasks_read = tasks.read().await;
    for (_, guild_tasks) in tasks_read.iter() {
        for (_, task) in guild_tasks.iter() {
            assert!(task.last_cleanup.elapsed() < Duration::from_secs(2));
        }
    }
}
