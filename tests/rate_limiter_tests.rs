use eule::utils::rate_limiter::RateLimiter;
use poise::serenity_prelude::futures::future::join_all;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_rate_limiter_basic() {
    let limiter = RateLimiter::new(2, Duration::from_secs(1));

    assert!(limiter.check().await.is_ok());
    assert!(limiter.check().await.is_ok());
    assert!(limiter.check().await.is_err());
}

#[tokio::test]
async fn test_rate_limiter_recovery() {
    let limiter = RateLimiter::new(1, Duration::from_secs(1));

    assert!(limiter.check().await.is_ok());
    assert!(limiter.check().await.is_err());

    sleep(Duration::from_secs(1)).await;

    assert!(limiter.check().await.is_ok());
}

#[tokio::test]
async fn test_rate_limiter_concurrent() {
    let limiter = Arc::new(RateLimiter::new(100, Duration::from_secs(1)));

    let tasks: Vec<_> = (0..200)
        .map(|_| {
            let l = Arc::clone(&limiter);
            tokio::spawn(async move { l.check().await.is_ok() })
        })
        .collect();

    let results = join_all(tasks).await;
    let successful = results
        .into_iter()
        .filter(|r| r.as_ref().map(|&b| b).unwrap_or(false))
        .count();

    assert_eq!(successful, 100);
}
