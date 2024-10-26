mod test_utils;

use eule::store::KvStore;
use poise::serenity_prelude::futures::future::join_all;
use std::sync::Arc;
use test_utils::{unique_test_path, TestCleanup};

/// Helper function to create a test store with encryption initialized
async fn create_encrypted_store() -> (KvStore, TestCleanup) {
    let path = unique_test_path();
    let cleanup = TestCleanup::new(path.clone()).unwrap();
    let mut store = KvStore::new(path).unwrap();
    store.initialize_encryption("test_password").await.unwrap();
    (store, cleanup)
}

#[tokio::test]
async fn test_kv_store_set_and_get() {
    let path = unique_test_path();
    let _cleanup = TestCleanup::new(path.clone()).unwrap();
    let kv_store = Arc::new(KvStore::new(path).unwrap());

    let key = "test_key";
    let value = "test_value";
    kv_store.set(key, value).await.unwrap();
    let result = kv_store.get(key).await.unwrap();
    assert_eq!(result, Some(value.to_string()));
}

#[tokio::test]
async fn test_kv_store_delete() {
    let path = unique_test_path();
    let _cleanup = TestCleanup::new(path.clone()).unwrap();
    let kv_store = Arc::new(KvStore::new(path).unwrap());
    
    let key = "test_key";
    let value = "test_value";
    kv_store.set(key, value).await.unwrap();
    kv_store.delete(key).await.unwrap();

    let result = kv_store.get(key).await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_kv_store_concurrent_operations() {
    let path = unique_test_path();
    let _cleanup = TestCleanup::new(path.clone()).unwrap();
    let kv_store = Arc::new(KvStore::new(path).unwrap());

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

#[tokio::test]
async fn test_encryption_initialization() {
    let (store, _cleanup) = create_encrypted_store().await;
    
    // Check if encryption was initialized by verifying we can store and retrieve sensitive data
    let sensitive_key = "discord_token";
    let sensitive_value = "test_token";
    store.set(sensitive_key, sensitive_value).await.unwrap();
    let retrieved = store.get(sensitive_key).await.unwrap();
    assert_eq!(retrieved, Some(sensitive_value.to_string()));
}

#[tokio::test]
async fn test_sensitive_data_encryption() {
    let (store, _cleanup) = create_encrypted_store().await;
        
    // Store sensitive data and verify it can be retrieved
    let sensitive_key = "discord_token";
    let sensitive_value = "super_secret_token";
    store.set(sensitive_key, sensitive_value).await.unwrap();

    // Verify we can retrieve the data
    let retrieved = store.get(sensitive_key).await.unwrap().unwrap();
    assert_eq!(retrieved, sensitive_value);

    // Store and retrieve multiple sensitive values
    let test_keys = ["api_key", "auth_token", "encryption_key"];
    for key in test_keys.iter() {
        store.set(key, "secret_value").await.unwrap();
        let retrieved = store.get(key).await.unwrap().unwrap();
        assert_eq!(retrieved, "secret_value");
    }
}

#[tokio::test]
async fn test_non_sensitive_data_storage() {
    let (store, _cleanup) = create_encrypted_store().await;
        
    // Test storing non-sensitive data
    let regular_key = "regular_key";
    let regular_value = "normal_data";
    store.set(regular_key, regular_value).await.unwrap();

    // Verify normal retrieval works
    let retrieved = store.get(regular_key).await.unwrap().unwrap();
    assert_eq!(retrieved, regular_value);
}

#[tokio::test]
async fn test_sensitive_keys() {
    let path = unique_test_path();
    let _cleanup = TestCleanup::new(path.clone()).unwrap();
    let mut store = KvStore::new(path).unwrap();
    
    // Initialize encryption
    store.initialize_encryption("test_password").await.unwrap();

    // Test various keys that should be encrypted
    let sensitive_keys = [
        "discord_token",
        "api_key",
        "auth_token",
        "encryption_key"
    ];

    for key in sensitive_keys.iter() {
        store.set(key, "secret_value").await.unwrap();
        let value = store.get(key).await.unwrap().unwrap();
        assert_eq!(value, "secret_value");
    }

    // Test non-sensitive keys
    let non_sensitive_keys = [
        "regular_key",
        "non_sensitive_data",
        "public_data"
    ];

    for key in non_sensitive_keys.iter() {
        store.set(key, "normal_value").await.unwrap();
        let value = store.get(key).await.unwrap().unwrap();
        assert_eq!(value, "normal_value");
    }
}

#[tokio::test]
async fn test_unencrypted_sensitive_data() {
    let path = unique_test_path();
    let _cleanup = TestCleanup::new(path.clone()).unwrap();
    let store = KvStore::new(path).unwrap();
    
    // Without encryption initialized, sensitive data should still be stored safely
    let result = store.set("discord_token", "secret").await;
    assert!(result.is_ok());

    // Verify we can still retrieve the value
    let retrieved = store.get("discord_token").await.unwrap().unwrap();
    assert_eq!(retrieved, "secret");
}
