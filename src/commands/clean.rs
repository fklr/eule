use crate::utils::RateLimiter;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[description("Clean the last n messages")]
#[usage("clean <number_of_messages>")]
pub async fn clean(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let number: u64 = args.single().unwrap_or(10);
    let rate_limiter = RateLimiter::new(5, 10); // 5 requests per 10 seconds

    let messages = msg
        .channel_id
        .messages(&ctx.http, |retriever| {
            retriever.before(msg.id).limit(number)
        })
        .await?;

    for chunk in messages.chunks(100) {
        if rate_limiter.check().await.is_err() {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
        msg.channel_id.delete_messages(&ctx.http, chunk).await?;
    }

    msg.reply(&ctx.http, format!("Cleaned {} messages", messages.len()))
        .await?;
    Ok(())
}
