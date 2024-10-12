use crate::utils::rate_limiter::RateLimiter;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{interval, Duration};

pub struct CleanupManager {
    tasks: Arc<RwLock<HashMap<ChannelId, CleanupTask>>>,
}

struct CleanupTask {
    interval: Duration,
    last_cleanup: std::time::Instant,
}

impl CleanupManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_task(&self, channel_id: ChannelId, interval: Duration) {
        let mut tasks = self.tasks.write().await;
        tasks.insert(
            channel_id,
            CleanupTask {
                interval,
                last_cleanup: std::time::Instant::now(),
            },
        );
    }

    pub async fn task_count(&self) -> usize {
        self.tasks.read().await.len()
    }

    pub async fn start(&self, ctx: Context) {
        let tasks = Arc::clone(&self.tasks);
        let ctx = Arc::new(ctx);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let tasks_read = tasks.read().await;
                for (channel_id, task) in tasks_read.iter() {
                    if task.last_cleanup.elapsed() >= task.interval {
                        let ctx = Arc::clone(&ctx);
                        let channel_id = *channel_id;
                        tokio::spawn(async move {
                            if let Err(e) = cleanup_channel(&ctx, channel_id).await {
                                eprintln!("Error cleaning up channel {}: {:?}", channel_id, e);
                            }
                        });
                    }
                }
            }
        });
    }
}

impl Default for CleanupManager {
    fn default() -> Self {
        Self::new()
    }
}

async fn cleanup_channel(
    ctx: &Context,
    channel_id: ChannelId,
) -> Result<(), Box<dyn std::error::Error>> {
    let rate_limiter = RateLimiter::new(5, 10); // 5 requests per 10 seconds

    let messages = channel_id
        .messages(&ctx.http, |retriever| retriever.limit(100))
        .await?;

    for chunk in messages.chunks(100) {
        if rate_limiter.check().await.is_err() {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        channel_id.delete_messages(&ctx.http, chunk).await?;
    }

    println!(
        "Cleaned up {} messages in channel {}",
        messages.len(),
        channel_id
    );
    Ok(())
}
