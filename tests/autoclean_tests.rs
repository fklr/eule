use eule::{store::KvStore, tasks::AutocleanManager};
use poise::serenity_prelude::{ChannelId, GuildId};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    sync::Arc,
};
use tokio::{runtime::Runtime, time::Duration};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn unique_test_db() -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    PathBuf::from(format!("test_db_{}", id))
}

struct TestCleanup {
    path: PathBuf,
}

impl Drop for TestCleanup {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_cleanup_manager_creation() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let db_path = unique_test_db();
        let _cleanup = TestCleanup {
            path: db_path.clone(),
        };
        let kv_store = Arc::new(KvStore::new(db_path).unwrap());
        let cleanup_manager = AutocleanManager::new(Arc::clone(&kv_store));
        assert_eq!(cleanup_manager.task_count(GuildId::new(1)).await, 0);
    });
}

#[test]
fn test_add_task() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let db_path = unique_test_db();
        let _cleanup = TestCleanup {
            path: db_path.clone(),
        };
        let kv_store = Arc::new(KvStore::new(db_path).unwrap());
        let cleanup_manager = AutocleanManager::new(Arc::clone(&kv_store));
        let channel_id = ChannelId::new(12345);
        let interval = Duration::from_secs(3600);

        cleanup_manager
            .add_task(GuildId::new(1), channel_id, interval)
            .await
            .unwrap();
        assert_eq!(cleanup_manager.task_count(GuildId::new(1)).await, 1);

        // Add another task
        let channel_id2 = ChannelId::new(67890);
        cleanup_manager
            .add_task(GuildId::new(1), channel_id2, interval)
            .await
            .unwrap();
        assert_eq!(cleanup_manager.task_count(GuildId::new(1)).await, 2);
    });
}

#[test]
fn test_save_and_load_tasks() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let db_path = unique_test_db();
        let _cleanup = TestCleanup {
            path: db_path.clone(),
        };
        let kv_store = Arc::new(KvStore::new(db_path).unwrap());
        let cleanup_manager = AutocleanManager::new(Arc::clone(&kv_store));

        let channel_id = ChannelId::new(12345);
        let channel_id2 = ChannelId::new(67890);

        cleanup_manager.save_tasks().await.unwrap();

        cleanup_manager
            .add_task(GuildId::new(1), channel_id, Duration::from_secs(3600))
            .await
            .unwrap();
        cleanup_manager
            .add_task(GuildId::new(1), channel_id2, Duration::from_secs(7200))
            .await
            .unwrap();

        cleanup_manager.save_tasks().await.unwrap();

        let new_cleanup_manager = AutocleanManager::new(Arc::clone(&kv_store));
        new_cleanup_manager.load_tasks().await.unwrap();

        assert_eq!(new_cleanup_manager.task_count(GuildId::new(1)).await, 2);
    });
}
