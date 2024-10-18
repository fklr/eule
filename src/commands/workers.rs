//! A command to check the number of active workers in the pool.

use crate::{Context, EuleError};

/// Displays the current number of active workers.
///
/// This command is mostly for debugging purposes (and fun).
///
/// # Arguments
///
/// * `ctx` - The command context, containing information about the invocation and bot data.
///
/// # Returns
///
/// A Result containing Ok(()) if the worker count message was successfully sent,
/// or an EuleError if there was an issue.
///
/// # Errors
///
/// This function will return an error if:
/// * There's an issue retrieving the worker count.
/// * The message cannot be sent to the channel.
///
/// # Examples
///
/// This command can be invoked in a Discord server as follows:
///
/// ```text
/// /workers
/// ```
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
