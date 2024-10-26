use crate::{error::EuleError, utils::crypto::Crypto};
use miette::Result;
use sled::Db;
use std::path::Path;
use zeroize::Zeroizing;

/// A secure key-value store providing transparent encryption for sensitive data.
///
/// The `KvStore` struct provides a persistent storage solution with automatic encryption
/// capabilities for sensitive information. It is built on top of the `sled` embedded
/// database and provides transparent encryption/decryption for designated sensitive keys.
///
/// # Security Features
///
/// * Automatic encryption of sensitive data using AES-256-GCM
/// * Secure key derivation using Argon2id with unique salt
/// * Protected memory handling for cryptographic keys using `Zeroizing`
/// * Transparent encryption/decryption operations
/// * Support for both sensitive and non-sensitive data
///
/// # Example
///
/// ```no_run
/// use eule::store::KvStore;
///
/// #[tokio::main]
/// async fn main() -> miette::Result<()> {
///     // Create a new store
///     let mut store = KvStore::new("my_database")?;
///
///     // Initialize encryption
///     store.initialize_encryption("strong_password").await?;
///
///     // Store sensitive data (automatically encrypted)
///     store.set("discord_token", "secret_token").await?;
///
///     // Store non-sensitive data (not encrypted)
///     store.set("public_data", "hello world").await?;
///
///     Ok(())
/// }
/// ```
///
/// # Security Considerations
///
/// * Sensitive keys (like "discord_token") are automatically encrypted
/// * The master key is securely wiped from memory when dropped
/// * Database operations are atomic and thread-safe
/// * No sensitive data is exposed in error messages or logs
pub struct KvStore {
    /// The underlying sled database instance
    db: Db,
    /// The master encryption key, protected by `Zeroizing`
    master_key: Option<Zeroizing<[u8; 32]>>,
}

impl KvStore {
    /// Creates a new `KvStore` instance at the specified path.
    ///
    /// The store is initially created without encryption enabled. To enable encryption,
    /// call `initialize_encryption` after creation.
    ///
    /// # Parameters
    ///
    /// * `path`: The filesystem path where the database should be stored
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the new `KvStore` instance or an error if the
    /// database cannot be opened.
    ///
    /// # Errors
    ///
    /// Will return `EuleError::Database` if:
    /// * The path cannot be accessed
    /// * The database files are corrupted
    /// * Insufficient permissions to create/access the database
    ///
    /// # Example
    ///
    /// ```no_run
    /// use eule::store::KvStore;
    ///
    /// let store = KvStore::new("my_database");
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, EuleError> {
        let db = sled::open(path).map_err(EuleError::from)?;
        Ok(Self {
            db,
            master_key: None,
        })
    }

    /// Initializes encryption for the store using the provided password.
    ///
    /// This method sets up encryption by:
    /// 1. Generating or retrieving the salt
    /// 2. Deriving the master key using Argon2id
    /// 3. Storing the encryption configuration securely
    ///
    /// Once encryption is initialized, sensitive keys will be automatically encrypted.
    /// Previously stored sensitive data will not be automatically encrypted.
    ///
    /// # Parameters
    ///
    /// * `password`: The password used to derive the master encryption key
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure of the encryption initialization.
    ///
    /// # Errors
    ///
    /// Will return an error if:
    /// * The salt cannot be generated or stored
    /// * The key derivation process fails
    /// * The database operations fail
    ///
    /// # Security Considerations
    ///
    /// * Use a strong password with high entropy
    /// * The password is never stored, only used for key derivation
    /// * The salt is stored in the database but the master key is kept only in memory
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use eule::store::KvStore;
    /// # #[tokio::main]
    /// # async fn main() -> miette::Result<()> {
    /// let mut store = KvStore::new("my_database")?;
    /// store.initialize_encryption("strong_password").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn initialize_encryption(&mut self, password: &str) -> Result<()> {
        let salt = match self.db.get("crypto_salt").map_err(EuleError::from)? {
            Some(existing_salt) => {
                let mut salt = [0u8; 16];
                salt.copy_from_slice(&existing_salt);
                salt
            }
            None => {
                let new_salt = Crypto::generate_salt()?;
                self.db
                    .insert("crypto_salt", new_salt.as_slice())
                    .map_err(EuleError::from)?;
                new_salt
            }
        };

        self.master_key = Some(Crypto::derive_key(password, &salt)?);
        Ok(())
    }

    /// Retrieves a value from the store by its key.
    ///
    /// If the key is marked as sensitive and encryption is enabled, the value will be
    /// automatically decrypted before being returned.
    ///
    /// # Parameters
    ///
    /// * `key`: The key to look up in the store
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// * `Some(String)` - The value if found
    /// * `None` - If the key doesn't exist
    ///
    /// # Errors
    ///
    /// Will return an error if:
    /// * The database operation fails
    /// * Decryption fails for sensitive data
    /// * The stored data is not valid UTF-8
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use eule::store::KvStore;
    /// # #[tokio::main]
    /// # async fn main() -> miette::Result<()> {
    /// # let store = KvStore::new("my_database")?;
    /// if let Some(value) = store.get("my_key").await? {
    ///     println!("Found value: {}", value);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        match self.db.get(key).map_err(EuleError::from)? {
            Some(ivec) => {
                if Self::is_sensitive_key(key) && self.master_key.is_some() {
                    let decrypted = Crypto::decrypt(&ivec, self.master_key.as_ref().unwrap())?;
                    Ok(Some(decrypted.to_string()))
                } else {
                    Ok(Some(String::from_utf8_lossy(&ivec).into_owned()))
                }
            }
            None => Ok(None),
        }
    }

    /// Sets a value in the store.
    ///
    /// If the key is marked as sensitive and encryption is enabled, the value will be
    /// automatically encrypted before storage.
    ///
    /// # Parameters
    ///
    /// * `key`: The key under which to store the value
    /// * `value`: The value to store
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure of the operation.
    ///
    /// # Errors
    ///
    /// Will return an error if:
    /// * The database operation fails
    /// * Encryption fails for sensitive data
    /// * The database flush operation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use eule::store::KvStore;
    /// # #[tokio::main]
    /// # async fn main() -> miette::Result<()> {
    /// # let store = KvStore::new("my_database")?;
    /// store.set("my_key", "my_value").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        let data = if Self::is_sensitive_key(key) && self.master_key.is_some() {
            Crypto::encrypt(value, self.master_key.as_ref().unwrap())?
        } else {
            value.as_bytes().to_vec()
        };

        self.db
            .insert(key, data)
            .and_then(|_| self.db.flush())
            .map_err(EuleError::from)?;

        Ok(())
    }

    /// Deletes a value from the store.
    ///
    /// # Parameters
    ///
    /// * `key`: The key of the value to delete
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure of the deletion.
    ///
    /// # Errors
    ///
    /// Will return an error if:
    /// * The database operation fails
    /// * The database flush operation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use eule::store::KvStore;
    /// # #[tokio::main]
    /// # async fn main() -> miette::Result<()> {
    /// # let store = KvStore::new("my_database")?;
    /// store.delete("my_key").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, key: &str) -> Result<()> {
        self.db
            .remove(key)
            .and_then(|_| self.db.flush())
            .map_err(EuleError::from)?;
        Ok(())
    }

    /// Determines if a key should be treated as sensitive and encrypted.
    ///
    /// # Parameters
    ///
    /// * `key`: The key to check
    ///
    /// # Returns
    ///
    /// Returns `true` if the key is considered sensitive and should be encrypted,
    /// `false` otherwise.
    ///
    /// # Security Note
    ///
    /// The following keys are always treated as sensitive:
    /// * `discord_token`
    /// * `encryption_key`
    /// * `api_key`
    /// * `auth_token`
    fn is_sensitive_key(key: &str) -> bool {
        matches!(
            key,
            "discord_token" | "encryption_key" | "api_key" | "auth_token"
        )
    }
}

impl Drop for KvStore {
    fn drop(&mut self) {
        if let Some(key) = self.master_key.take() {
            drop(key); // Zeroizing handles secure erasure
        }
    }
}
