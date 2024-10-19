use crate::{
    commands::{autoclean, clean, status, workers},
    error::EuleError,
    store::KvStore,
    tasks::AutocleanManager,
    Data,
};
use poise::serenity_prelude::{ActivityData, ClientBuilder, GatewayIntents, Http};
use rpassword::read_password;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use tokio::time::{Duration, Instant};

/// A trait for reading a token from user input.
pub trait TokenInput {
    fn read_token(&self) -> Result<String, EuleError>;
}

/// Implementation of TokenInput for reading Discord tokens.
pub struct DiscordToken;

impl TokenInput for DiscordToken {
    fn read_token(&self) -> Result<String, EuleError> {
        read_password().map_err(EuleError::Io)
    }
}

/// The main struct representing the Eule bot.
///
/// This struct contains the core components of the bot, including the key-value store,
/// autoclean manager, and start time.
pub struct Bot {
    kv_store: Arc<KvStore>,
    autoclean_manager: AutocleanManager,
    pub start_time: Instant,
    is_connected: AtomicBool,
    connection_attempts: AtomicUsize,
}

impl Bot {
    /// Creates a new `Bot` instance.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the new `Bot` instance if successful,
    /// or an error if initialization fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The KvStore initialization fails
    /// - The AutocleanManager fails to load tasks
    pub async fn new() -> Result<Self, EuleError> {
        let kv_store = Arc::new(KvStore::new("eule_data")?);
        tracing::info!("KvStore initialized at 'eule_data'");
        let autoclean_manager = AutocleanManager::new(Arc::clone(&kv_store));
        tracing::info!("AutocleanManager initialized with KvStore");
        autoclean_manager.load_tasks().await?;
        tracing::info!("Tasks loaded into AutocleanManager");

        Ok(Self {
            kv_store,
            autoclean_manager,
            start_time: Instant::now(),
            is_connected: AtomicBool::new(false),
            connection_attempts: AtomicUsize::new(0),
        })
    }

    /// Retrieves the stored Discord API token or prompts the user to enter a new one.
    ///
    /// # Arguments
    ///
    /// * `kv_store` - An `Arc<KvStore>` representing the key-value store used for token storage.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the valid Discord API token as a `String` if successful,
    /// or an `Err` containing an `EuleError` if token retrieval or storage fails.
    pub async fn get_or_set_token(kv_store: Arc<KvStore>) -> Result<String, EuleError> {
        if let Some(token) = kv_store.get("discord_token").await? {
            match Self::validate_token(&token).await {
                Ok(_) => return Ok(token),
                Err(_) => {
                    println!("Stored token is invalid. Removing it from storage.");
                    kv_store.delete("discord_token").await?;
                }
            }
        }

        println!("Please enter your Discord API token (input will be hidden):");
        let token = DiscordToken.read_token()?;
        match Self::validate_token(&token).await {
            Ok(_) => {
                kv_store.set("discord_token", &token).await?;
                println!("Token validated and saved successfully.");
                Ok(token)
            }
            Err(e) => {
                println!("Invalid token: {}. Please try again.", e);
                Err(EuleError::AuthenticationFailed(
                    "Invalid token provided".to_string(),
                ))
            }
        }
    }

    /// Validates a Discord API token by attempting to fetch the current application info.
    ///
    /// # Arguments
    ///
    /// * `token` - A string slice containing the Discord API token to validate.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the token is valid, or an `Err` containing an `EuleError::AuthenticationFailed`
    /// if the token is invalid or the authentication process fails.
    pub async fn validate_token(token: &str) -> Result<(), EuleError> {
        let http = Http::new(token);
        http.get_current_application_info()
            .await
            .map_err(|e| EuleError::AuthenticationFailed(e.to_string()))?;
        Ok(())
    }

    /// Delete the Discord API token from the key-value store.
    ///
    /// # Returns
    ///
    /// A Result containing Ok(()) if the token was deleted successfully.
    pub async fn delete_token(&self) -> Result<(), EuleError> {
        self.kv_store.delete("discord_token").await?;
        Ok(())
    }

    /// Attempts to connect the bot to Discord.
    ///
    /// This method performs the following steps:
    /// 1. Retrieves the bot token from the key-value store.
    /// 2. Creates an HTTP client with the token.
    /// 3. Verifies the token by fetching the current application info.
    /// 4. Sets up the bot's activity status.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the connection is successful, or an `EuleError` if any step fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The bot token cannot be retrieved from the key-value store.
    /// - The HTTP client cannot be created.
    /// - The token verification fails (i.e., cannot fetch application info).
    /// - Setting up the activity status fails.
    pub async fn connect(&self) -> Result<(), EuleError> {
        self.connection_attempts.fetch_add(1, Ordering::SeqCst);

        // Retrieve the token from the key-value store
        let token = self.kv_store.get("discord_token").await?.ok_or_else(|| {
            EuleError::AuthenticationFailed(
                "Discord token not found in key-value store".to_string(),
            )
        })?;

        // Create an HTTP client with the token
        let http = Http::new(&token);

        // Verify the token by fetching the current application info
        http.get_current_application_info().await.map_err(|e| {
            EuleError::AuthenticationFailed(format!("Failed to verify token: {}", e))
        })?;

        // Set up the bot's activity
        let activity = ActivityData::listening("Cigarette Wife");

        // Create a client builder with the verified token and intents
        let _client_builder =
            ClientBuilder::new(token, GatewayIntents::non_privileged()).activity(activity);

        // Not starting the client here, just verifying that it can be created
        // The actual client start will happen in the `run` method

        self.is_connected.store(true, Ordering::SeqCst);
        tracing::info!("Bot successfully connected and verified with Discord API");

        Ok(())
    }

    /// Run the bot with the Discord API token stored in the key-value store.
    ///
    /// This method sets up the framework, registers the commands, and starts the bot
    /// with the specified token.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the bot runs successfully, or an `Err` if an error occurs.
    pub async fn run(&self) -> Result<(), EuleError> {
        let token = Self::get_or_set_token(Arc::clone(&self.kv_store)).await?;

        let options = poise::FrameworkOptions {
            commands: vec![autoclean(), clean(), status(), workers()],
            ..Default::default()
        };

        let mut autoclean_manager = self.autoclean_manager.clone();
        let kv_store = Arc::clone(&self.kv_store);

        let framework = poise::Framework::builder()
            .options(options)
            .setup(move |ctx, _ready, framework| {
                Box::pin(async move {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                    autoclean_manager.start(ctx.http.clone()).await;
                    tracing::info!("AutocleanManager started");
                    let bot = Arc::new(Bot {
                        kv_store: Arc::clone(&kv_store),
                        autoclean_manager: autoclean_manager.clone(),
                        start_time: Instant::now(),
                        is_connected: AtomicBool::new(true),
                        connection_attempts: AtomicUsize::new(1),
                    });

                    // Create and return the Data instance
                    Ok(Data::new(autoclean_manager, Arc::clone(&kv_store), bot))
                })
            })
            .build();

        let intents = GatewayIntents::non_privileged();

        let activity = ActivityData::listening("Cigarette Wife");

        ClientBuilder::new(token, intents)
            .framework(framework)
            .activity(activity)
            .await
            .map_err(EuleError::from)?
            .start()
            .await
            .map_err(EuleError::DiscordApi)?;
        Ok(())
    }

    /// Returns the uptime of the bot.
    ///
    /// # Returns
    ///
    /// The duration since the bot was started.
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Checks if the bot is currently connected.
    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::SeqCst)
    }

    /// Gets the number of connection attempts made.
    pub fn connection_attempts(&self) -> usize {
        self.connection_attempts.load(Ordering::SeqCst)
    }
}
