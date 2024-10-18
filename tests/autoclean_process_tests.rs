use eule::{store::KvStore, tasks::AutocleanManager};
use poise::serenity_prelude::{ChannelId, GuildId, MessageId};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    sync::Arc,
};
use tokio::{sync::RwLock, time::Duration};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn unique_test_db() -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    PathBuf::from(format!("testdb{}", id))
}
struct TestCleanup {
    path: PathBuf,
}
impl Drop for TestCleanup {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
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

type DeleteMessagesCalls = Arc<RwLock<Vec<(ChannelId, Vec<MessageId>)>>>;

struct TestHttpClient {
    delete_messages_calls: DeleteMessagesCalls,
    should_fail: bool,
}

impl TestHttpClient {
    fn new(should_fail: bool) -> Self {
        Self {
            delete_messages_calls: Arc::new(RwLock::new(Vec::new())),
            should_fail,
        }
    }

    async fn get_delete_messages_calls(&self) -> Vec<(ChannelId, Vec<MessageId>)> {
        self.delete_messages_calls.read().await.clone()
    }
}

#[async_trait::async_trait]
impl HttpClientTrait for TestHttpClient {
    async fn delete_messages<T: AsRef<MessageId> + Send + Sync + 'static>(
        &self,
        channel_id: ChannelId,
        message_ids: &[T],
    ) -> Result<(), poise::serenity_prelude::Error> {
        if self.should_fail {
            return Err(poise::serenity_prelude::Error::Other("API Error"));
        }
        let mut calls = self.delete_messages_calls.write().await;
        calls.push((
            channel_id,
            message_ids.iter().map(|m| *m.as_ref()).collect(),
        ));
        Ok(())
    }
}

#[tokio::test]
async fn test_autoclean_manager_creation() {
    let db_path = unique_test_db();
    let _cleanup = TestCleanup {
        path: db_path.clone(),
    };
    let kv_store = Arc::new(KvStore::new(db_path).unwrap());
    let cleanup_manager = AutocleanManager::new(kv_store);
    assert_eq!(cleanup_manager.task_count(GuildId::new(1)).await, 0);
}

async fn cleanup_channel(
    http: &impl HttpClientTrait,
    guild_id: GuildId,
    channel_id: ChannelId,
    tasks: &Arc<RwLock<HashMap<GuildId, HashMap<ChannelId, Duration>>>>,
) -> Result<(), poise::serenity_prelude::Error> {
    let messages = vec![MessageId::new(1), MessageId::new(2), MessageId::new(3)];
    http.delete_messages(channel_id, &messages).await?;

    let mut tasks_write = tasks.write().await;
    if let Some(guild_tasks) = tasks_write.get_mut(&guild_id) {
        if let Some(interval) = guild_tasks.get_mut(&channel_id) {
            *interval = Duration::from_secs(0);
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_cleanup_channel() {
    let guild_id = GuildId::new(1);
    let channel_id = ChannelId::new(1);
    let http_client = TestHttpClient::new(false);

    let tasks = Arc::new(RwLock::new(HashMap::new()));
    tasks.write().await.insert(guild_id, HashMap::new());
    tasks
        .write()
        .await
        .get_mut(&guild_id)
        .unwrap()
        .insert(channel_id, Duration::from_secs(3600));

    let result = cleanup_channel(&http_client, guild_id, channel_id, &tasks).await;
    assert!(result.is_ok());

    let tasks_read = tasks.read().await;
    let guild_tasks = tasks_read.get(&guild_id).unwrap();
    let channel_interval = guild_tasks.get(&channel_id).unwrap();
    assert_eq!(*channel_interval, Duration::from_secs(0));

    let delete_calls = http_client.get_delete_messages_calls().await;
    assert_eq!(delete_calls.len(), 1);
    assert_eq!(delete_calls[0].0, channel_id);
    assert_eq!(
        delete_calls[0].1,
        vec![MessageId::new(1), MessageId::new(2), MessageId::new(3)]
    );
}

#[tokio::test]
async fn test_cleanup_with_multiple_calls() {
    let guild_id = GuildId::new(1);
    let channel_id = ChannelId::new(1);
    let http_client = TestHttpClient::new(false);

    let tasks = Arc::new(RwLock::new(HashMap::new()));
    tasks.write().await.insert(guild_id, HashMap::new());
    tasks
        .write()
        .await
        .get_mut(&guild_id)
        .unwrap()
        .insert(channel_id, Duration::from_secs(3600));

    // Perform multiple cleanup calls
    for _ in 0..3 {
        let result = cleanup_channel(&http_client, guild_id, channel_id, &tasks).await;
        assert!(result.is_ok());
    }

    // Verify that the interval has been reset to 0 after multiple cleanups
    let tasks_read = tasks.read().await;
    let guild_tasks = tasks_read.get(&guild_id).unwrap();
    let channel_interval = guild_tasks.get(&channel_id).unwrap();
    assert_eq!(*channel_interval, Duration::from_secs(0));

    let delete_calls = http_client.get_delete_messages_calls().await;
    assert_eq!(delete_calls.len(), 3);
}

#[tokio::test]
async fn test_cleanup_error_handling() {
    let guild_id = GuildId::new(1);
    let channel_id = ChannelId::new(1);
    let http_client = TestHttpClient::new(true);

    let tasks = Arc::new(RwLock::new(HashMap::new()));
    tasks.write().await.insert(guild_id, HashMap::new());
    tasks
        .write()
        .await
        .get_mut(&guild_id)
        .unwrap()
        .insert(channel_id, Duration::from_secs(3600));

    let result = cleanup_channel(&http_client, guild_id, channel_id, &tasks).await;
    assert!(result.is_err());
}
