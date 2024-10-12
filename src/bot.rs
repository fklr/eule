use std::sync::Arc;

use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::commands::{clean::*, status::*};
use crate::store::{KvStore, KvStoreOperations};
use crate::tasks::CleanupManager;

#[group]
#[commands(clean, status)]
struct General;

pub struct Bot {
    kv_store: Arc<dyn KvStoreOperations>,
    cleanup_manager: CleanupManager,
}

impl Bot {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            kv_store: Arc::new(KvStore::new("eule_data")?),
            cleanup_manager: CleanupManager::new(),
        })
    }

    pub fn new_with_store(kv_store: Arc<dyn KvStoreOperations>) -> Self {
        Self {
            kv_store,
            cleanup_manager: CleanupManager::new(),
        }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let token = self
            .kv_store
            .get("discord_token")?
            .ok_or("Discord token not found in key-value store")?;

        let framework = StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .group(&GENERAL_GROUP);

        let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
        let mut client = Client::builder(token, intents)
            .event_handler(BotHandler {
                cleanup_manager: self.cleanup_manager,
            })
            .framework(framework)
            .await?;

        client.start().await?;
        Ok(())
    }
}

struct BotHandler {
    cleanup_manager: CleanupManager,
}

#[serenity::async_trait]
impl EventHandler for BotHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        self.cleanup_manager.start(ctx).await;
    }
}
