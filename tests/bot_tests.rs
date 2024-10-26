mod test_utils;

use eule::{store::KvStore, Bot, EuleError};
use std::sync::Arc;
use test_utils::{unique_test_path, TestCleanup};
use tokio::time::Duration;

async fn setup_test_bot() -> Result<(Bot, TestCleanup), EuleError> {
    let test_path = unique_test_path();
    let cleanup = TestCleanup::new(test_path.clone())?;

    let kv_store = Arc::new(KvStore::new(&test_path)?);
    kv_store.set("discord_token", "test_token").await?;

    let bot = Bot::with_store(Arc::clone(&kv_store)).await?;

    Ok((bot, cleanup))
}

#[tokio::test]
async fn test_bot_creation() -> Result<(), EuleError> {
    let (bot, _cleanup) = setup_test_bot().await?;

    assert!(!bot.is_connected());
    assert_eq!(bot.connection_attempts(), 0);

    Ok(())
}

#[tokio::test]
async fn test_bot_uptime() -> Result<(), EuleError> {
    let (bot, _cleanup) = setup_test_bot().await?;

    tokio::time::sleep(Duration::from_millis(10)).await;
    let uptime = bot.uptime();

    assert!(uptime >= Duration::from_millis(10));
    assert!(uptime <= Duration::from_millis(100));

    Ok(())
}
