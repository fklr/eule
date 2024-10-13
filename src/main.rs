use clap::Command;
use eule::{Bot, EuleError, Result};
use jemallocator::Jemalloc;
use tracing_appender::rolling::daily;
use tracing_subscriber::{fmt, EnvFilter};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> Result<()> {
    let file_appender = daily("logs", "eule.log");

    let subscriber = fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(file_appender)
        .finish();

    tracing::subscriber::set_global_default(subscriber).map_err(|e| {
        eprintln!("Eule failed to set up tracing: {}", e);
        EuleError::TracingSetupFailed(e.to_string())
    })?;

    let matches = Command::new("Eule")
        .version("1.0")
        .author("@ovnanova")
        .about("Einfache Uneinigkeit Leichte Replika")
        .subcommand(Command::new("delete-token").about("Delete the stored Discord token"))
        .get_matches();

    match matches.subcommand() {
        Some(("delete-token", _)) => {
            let bot = Bot::new().await?;
            bot.delete_token().await?;
            println!("Discord token has been deleted.");
        }
        _ => {
            let bot = Bot::new().await?;
            bot.run().await?;
        }
    }

    Ok(())
}
