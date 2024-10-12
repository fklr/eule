use crate::ContextData;
use crate::{CleanupTaskCount, StartTime};
use serenity::client::Context;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::Message;
use std::time::{Duration, SystemTime};

#[command]
#[description("Check Eule's status")]
pub async fn status(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data().read().await;
    let start_time = data
        .get::<StartTime>()
        .expect("Expected StartTime in TypeMap");
    let uptime = SystemTime::now()
        .duration_since(*start_time)
        .unwrap_or(Duration::from_secs(0));

    let cleanup_tasks = data
        .get::<CleanupTaskCount>()
        .expect("Expected CleanupTaskCount in TypeMap");
    let task_count = cleanup_tasks.load(std::sync::atomic::Ordering::Relaxed);

    let status_message = format!(
        "Eule says Hi!\nUptime: {} days, {} hours, {} minutes\nScheduled Cleanup Tasks: {}",
        uptime.as_secs() / 86400,
        (uptime.as_secs() % 86400) / 3600,
        (uptime.as_secs() % 3600) / 60,
        task_count
    );

    msg.reply(&ctx.http, status_message).await?;
    Ok(())
}
