//! A command to check the bot's uptime and the number of scheduled autoclean tasks.

use crate::{Context, EuleError};

/// Displays the bot's current status, including uptime and scheduled cleaning tasks.
///
/// This command provides information about how long the bot has been running
/// and how many cleaning tasks are currently scheduled for the guild where
/// the command is invoked.
///
/// # Arguments
///
/// * `ctx` - The command context, containing information about the invocation and bot data.
///
/// # Returns
///
/// A Result containing Ok(()) if the status message was successfully sent,
/// or an EuleError if there was an issue.
///
/// # Errors
///
/// This function will return an error if:
/// * The command is not used within a guild (EuleError::NotInGuild).
/// * There's an issue retrieving the bot's uptime or task count.
/// * The message cannot be sent to the channel.
///
/// # Examples
///
/// This command can be invoked in a Discord server as follows:
///
/// ```text
/// /status
/// ```
///
/// Note: This command cannot be demonstrated in a doc test due to its reliance on Discord context.
#[poise::command(slash_command, prefix_command)]
pub async fn status(ctx: Context<'_>) -> Result<(), EuleError> {
    let guild_id = ctx.guild_id().ok_or(EuleError::NotInGuild)?;
    let uptime = ctx.data().bot.uptime();
    let task_count = ctx.data().autoclean_manager.task_count(guild_id).await;

    let days = uptime.as_secs() / 86400;
    let hours = (uptime.as_secs() % 86400) / 3600;
    let minutes = (uptime.as_secs() % 3600) / 60;
    let seconds = uptime.as_secs() % 60;

    ctx.say(format!(
        "You look kind of familiar... have we met before? 🤔\nUptime: {} days, {} hours, {} minutes, {} seconds\nScheduled Cleaning Tasks: {} 🧹",
        days, hours, minutes, seconds, task_count
    ))
    .await?;

    Ok(())
}
