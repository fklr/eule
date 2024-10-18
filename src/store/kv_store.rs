//! Key-Value Store Module for Eule
//!
//! This module provides a thread-safe, persistent key-value store implementation
//! for Eule. It is built on top of the `sled` embedded database
//! and offers a simple interface for storing, retrieving, and deleting key-value pairs.
//!
//! The `KvStore` struct is the main component of this module. It encapsulates
//! the database operations and ensures thread-safety through the use of Tokio's
//! asynchronous read-write lock (`RwLock`).
//!
//! Key features:
//! - Persistent storage using the `sled` embedded database
//! - Asynchronous, thread-safe operations
//! - Simple API for get, set, and delete operations
//! - Error handling using the `miette` crate
//!
//! This module is crucial for Eule's functionality, allowing the bot to persist
//! data across restarts and maintain state information for various features,
//! such as tasks and bot configurations.

use crate::EuleError;
use miette::Result;
use sled::Db;
use std::path::Path;
use tokio::sync::RwLock;

/// This struct provides a simple interface for storing, retrieving, and deleting
/// key-value pairs in a persistent storage. It uses Tokio's RwLock to ensure
/// thread-safety for concurrent operations in an asynchronous context.
///
/// # Examples
///
/// ```
/// # use eule::store::KvStore;
/// # use miette::Result;
/// # use tempfile::tempdir;
/// #
/// # #[tokio::main]
/// # async fn main() -> Result<()> {
/// #     let dir = tempdir()?;
/// #     let store = KvStore::new(dir.path())?;
/// #
/// // Set a value
/// store.set("key1", "value1").await?;
///
/// // Get a value
/// let value = store.get("key1").await?;
/// assert_eq!(value, Some("value1".to_string()));
///
/// // Delete a value
/// store.delete("key1").await?;
///
/// // Value should now be None
/// let value = store.get("key1").await?;
/// assert_eq!(value, None);
/// #     Ok(())
/// # }
/// ```
pub struct KvStore {
    db: RwLock<Db>,
}

impl KvStore {
    /// Creates a new `KvStore` instance.
    ///
    /// # Parameters
    /// - `path`: A path-like object specifying the location of the database files.
    ///
    /// # Returns
    /// A `Result` containing the new `KvStore` instance if successful, or an error if
    /// the database couldn't be opened.
    ///
    /// # Errors
    /// This function will return an error if the sled database cannot be opened at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eule::store::KvStore;
    /// # use miette::Result;
    /// # use tempfile::tempdir;
    /// #
    /// # fn main() -> Result<()> {
    /// #     let dir = tempdir()?;
    /// let store = KvStore::new(dir.path())?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = sled::open(path).map_err(EuleError::from)?;
        Ok(Self {
            db: RwLock::new(db),
        })
    }

    /// Retrieves a value from the store by its key.
    ///
    /// # Parameters
    /// - `key`: The key to look up.
    ///
    /// # Returns
    /// A `Result` containing an `Option` with the value as a `String` if found, or `None` if not found.
    ///
    /// # Errors
    /// This function will return an error if the database operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eule::store::KvStore;
    /// # use miette::Result;
    /// # use tempfile::tempdir;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// #     let dir = tempdir()?;
    /// #     let store = KvStore::new(dir.path())?;
    /// #     store.set("key1", "value1").await?;
    /// let value = store.get("key1").await?;
    /// assert_eq!(value, Some("value1".to_string()));
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let db = self.db.read().await;
        Ok(db
            .get(key)
            .map_err(EuleError::from)?
            .map(|ivec| String::from_utf8_lossy(&ivec).into_owned()))
    }

    /// Sets a value in the store.
    ///
    /// # Parameters
    /// - `key`: The key under which to store the value.
    /// - `value`: The value to store.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    ///
    /// # Errors
    /// This function will return an error if the database operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eule::store::KvStore;
    /// # use miette::Result;
    /// # use tempfile::tempdir;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// #     let dir = tempdir()?;
    /// #     let store = KvStore::new(dir.path())?;
    /// store.set("key2", "value2").await?;
    /// let value = store.get("key2").await?;
    /// assert_eq!(value, Some("value2".to_string()));
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        let db = self.db.write().await;
        db.insert(key, value.as_bytes()).map_err(EuleError::from)?;
        db.flush().map_err(EuleError::from)?;
        Ok(())
    }

    /// Deletes a value from the store.
    ///
    /// # Parameters
    /// - `key`: The key of the value to delete.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    ///
    /// # Errors
    /// This function will return an error if the database operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eule::store::KvStore;
    /// # use miette::Result;
    /// # use tempfile::tempdir;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// #     let dir = tempdir()?;
    /// #     let store = KvStore::new(dir.path())?;
    /// #     store.set("key3", "value3").await?;
    /// store.delete("key3").await?;
    /// let value = store.get("key3").await?;
    /// assert_eq!(value, None);
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, key: &str) -> Result<()> {
        let db = self.db.write().await;
        db.remove(key).map_err(EuleError::from)?;
        db.flush().map_err(EuleError::from)?;
        Ok(())
    }
}
