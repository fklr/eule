//! Command for cleaning up messages in a channel.
//!
//! This module contains the `clean` command, which allows users to delete
//! a specified number of messages from the current channel.

use crate::{Data, EuleError};
use poise::serenity_prelude as serenity;

/// Cleans up a specified number of messages in the current channel.
///
/// This command deletes a number of recent messages from the channel where it's invoked.
/// If no number is specified, it defaults to 10 messages. The maximum number of
/// messages that can be deleted in a single command is 100.
///
/// # Arguments
///
/// * `ctx` - The command context, containing information about the invocation and bot data.
/// * `number` - An optional number of messages to clean. If not provided, defaults to 10.
///
/// # Returns
///
/// A Result containing Ok(()) if the messages were successfully cleaned,
/// or an EuleError if there was an issue.
///
/// # Permissions
///
/// Requires the `MANAGE_MESSAGES` permission.
///
/// # Examples
///
/// To clean the default number of messages (10):
/// ```
/// /clean
/// ```
///
/// To clean a specific number of messages (e.g., 25):
/// ```
/// /clean 25
/// ```
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_MESSAGES"
)]
pub async fn clean(
    ctx: poise::Context<'_, Data, EuleError>,
    #[description = "Number of messages to clean"] number: Option<u64>,
) -> Result<(), EuleError> {
    // Ensure the number of messages to clean is between 1 and 100
    let number = number.unwrap_or(10).min(100) as u8;

    // Fetch the messages to be deleted
    let messages = ctx
        .channel_id()
        .messages(
            &ctx.http(),
            serenity::builder::GetMessages::new().limit(number),
        )
        .await?;

    // Delete the messages
    ctx.channel_id()
        .delete_messages(&ctx.http(), &messages)
        .await?;

    // Confirm the number of messages cleaned
    ctx.say(format!("Cleaned {} messages! 🚮", messages.len()))
        .await?;

    Ok(())
}
