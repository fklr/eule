use crate::{Context, EuleError};

#[poise::command(slash_command, prefix_command)]
pub async fn status(ctx: Context<'_>) -> Result<(), EuleError> {
    let uptime = ctx.data().cleanup_manager.uptime().await;
    let task_count = ctx.data().cleanup_manager.task_count().await;

    let days = uptime.as_secs() / 86400;
    let hours = (uptime.as_secs() % 86400) / 3600;
    let minutes = (uptime.as_secs() % 3600) / 60;
    let seconds = uptime.as_secs() % 60;

    ctx.say(format!(
        "You look kind of familiar... have we met before?\nUptime: {} days, {} hours, {} minutes, {} seconds\nScheduled Cleanup Tasks: {}",
        days, hours, minutes, seconds, task_count
    ))
    .await?;

    Ok(())
}
