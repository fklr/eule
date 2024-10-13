use crate::{Data, EuleError};
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, prefix_command)]
pub async fn clean(
    ctx: poise::Context<'_, Data, EuleError>,
    #[description = "Number of messages to clean"] number: Option<u64>,
) -> Result<(), EuleError> {
    let number = number.unwrap_or(10).min(100) as u8;

    let messages = ctx
        .channel_id()
        .messages(
            &ctx.http(),
            serenity::builder::GetMessages::new().limit(number),
        )
        .await?;

    ctx.channel_id()
        .delete_messages(&ctx.http(), &messages)
        .await?;

    ctx.say(format!("Cleaned {} messages", messages.len()))
        .await?;
    Ok(())
}
