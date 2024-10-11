use eule::Bot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the bot
    let bot = Bot::new().await?;

    // Run the bot
    bot.run().await?;

    Ok(())
}
