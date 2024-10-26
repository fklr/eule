//! Commands for managing autoclean tasks.
//!
//! This module contains commands for adding, removing, and listing autoclean tasks.
//! All commands in this module require the `MANAGE_MESSAGES` permission.

use crate::{Context, EuleError};
use miette::Result;
use poise::serenity_prelude::ChannelId;
use tokio::time::Duration;

/// Parent command for autoclean functionality.
///
/// This command serves as a container for subcommands related to autoclean tasks.
///
/// # Permissions
///
/// Requires the `MANAGE_MESSAGES` permission.
#[poise::command(
    slash_command,
    prefix_command,
    subcommands("add", "remove", "list", "workers"),
    required_permissions = "MANAGE_MESSAGES"
)]
pub async fn autoclean(_: Context<'_>) -> Result<(), EuleError> {
    Ok(())
}

/// Adds a new autoclean task for a specified channel.
///
/// # Arguments
///
/// * `ctx` - The command context.
/// * `channel` - The channel to autoclean.
/// * `interval` - The interval value for cleaning.
/// * `unit` - The time unit for the interval (minutes, hours, days).
///
/// # Returns
///
/// A Result containing Ok(()) if the task was added successfully, or an EuleError if there was an issue.
#[poise::command(slash_command, prefix_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Channel to autoclean"] channel: ChannelId,
    #[description = "Interval value"] interval: u64,
    #[description = "Time unit (minutes, hours, days)"] unit: String,
) -> Result<(), EuleError> {
    let guild_id = ctx.guild_id().ok_or(EuleError::NotInGuild)?;

    let duration = match unit.to_lowercase().as_str() {
        "minutes" | "minute" | "m" => Duration::from_secs(interval * 60),
        "hours" | "hour" | "h" => Duration::from_secs(interval * 3600),
        "days" | "day" | "d" => Duration::from_secs(interval * 86400),
        _ => return Err(EuleError::InvalidTimeUnit),
    };

    ctx.data()
        .autoclean_manager
        .add_task(guild_id, channel, duration)
        .await?;

    ctx.say(format!(
        "Added autoclean task for channel <#{0}> every {1} {2}! ⏰",
        channel, interval, unit
    ))
    .await?;

    Ok(())
}

/// Removes an autoclean task for a specified channel.
///
/// # Arguments
///
/// * `ctx` - The command context.
/// * `channel` - The channel to remove the autoclean task from.
///
/// # Returns
///
/// A Result containing Ok(()) if the task was removed successfully or if no task was found,
/// or an EuleError if there was an issue.
#[poise::command(slash_command, prefix_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Channel to remove autoclean task from"] channel: ChannelId,
) -> Result<(), EuleError> {
    let guild_id = ctx.guild_id().ok_or(EuleError::NotInGuild)?;

    if ctx
        .data()
        .autoclean_manager
        .remove_task(guild_id, channel)
        .await?
    {
        ctx.say(format!(
            "Removed autoclean task for channel <#{0}>! ✅",
            channel
        ))
        .await?;
    } else {
        ctx.say(format!(
            "No autoclean task found for channel <#{0}>! ❌",
            channel
        ))
        .await?;
    }

    Ok(())
}

/// Lists all autoclean tasks in the current server.
///
/// # Arguments
///
/// * `ctx` - The command context.
///
/// # Returns
///
/// A Result containing Ok(()) if the tasks were listed successfully,
/// or an EuleError if there was an issue.
#[poise::command(slash_command, prefix_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), EuleError> {
    let guild_id = ctx.guild_id().ok_or(EuleError::NotInGuild)?;

    let tasks = ctx.data().autoclean_manager.list_tasks(guild_id).await;

    if tasks.is_empty() {
        ctx.say("No cleaning tasks scheduled for this server.")
            .await?;
    } else {
        let task_list = tasks
            .iter()
            .map(|(channel_id, interval)| {
                format!(
                    "Channel: <#{0}>, Interval: {1} minutes",
                    channel_id,
                    interval.as_secs() / 60
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        ctx.say(format!(
            "Scheduled cleaning tasks for this server:\n{}",
            task_list
        ))
        .await?;
    }

    Ok(())
}

/// Displays the current number of active cleaning workers.
///
/// This command shows how many worker threads are currently processing cleanup tasks.
#[poise::command(slash_command, prefix_command)]
pub async fn workers(ctx: Context<'_>) -> Result<(), EuleError> {
    let worker_count = ctx.data().autoclean_manager.worker_count().await;

    let message = if worker_count == 1 {
        "I am currently the only EULR unit on duty, Commander! 🫡"
    } else {
        &format!(
            "There are currently {} EULR units on duty, Commander! 🫡",
            worker_count
        )
    };

    ctx.say(message).await?;

    Ok(())
}
