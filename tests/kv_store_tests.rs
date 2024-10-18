use eule::store::KvStore;
use poise::serenity_prelude::futures::future::join_all;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    sync::Arc,
};

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

#[tokio::test]
async fn test_kv_store_set_and_get() {
    let db_path = unique_test_db();
    let _cleanup = TestCleanup {
        path: db_path.clone(),
    };
    let kv_store = Arc::new(KvStore::new(db_path).unwrap());

    let key = "test_key";
    let value = "test_value";

    kv_store.set(key, value).await.unwrap();
    let result = kv_store.get(key).await.unwrap();
    assert_eq!(result, Some(value.to_string()));
}

#[tokio::test]
async fn test_kv_store_delete() {
    let db_path = unique_test_db();
    let _cleanup = TestCleanup {
        path: db_path.clone(),
    };
    let kv_store = Arc::new(KvStore::new(db_path).unwrap());
    let key = "test_key";
    let value = "test_value";

    kv_store.set(key, value).await.unwrap();
    kv_store.delete(key).await.unwrap();

    let result = kv_store.get(key).await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_kv_store_concurrent_operations() {
    let db_path = unique_test_db();
    let _cleanup = TestCleanup {
        path: db_path.clone(),
    };
    let kv_store = Arc::new(KvStore::new(db_path).unwrap());

    let tasks: Vec<_> = (0..100)
        .map(|i| {
            let s = Arc::clone(&kv_store);
            tokio::spawn(async move {
                let key = format!("key_{}", i);
                let value = format!("value_{}", i);
                s.set(&key, &value).await.unwrap();
                assert_eq!(s.get(&key).await.unwrap(), Some(value));
                s.delete(&key).await.unwrap();
                assert_eq!(s.get(&key).await.unwrap(), None);
            })
        })
        .collect();

    let results = join_all(tasks).await;
    let successful = results.into_iter().filter(|r| r.is_ok()).count();

    assert_eq!(successful, 100);
}
