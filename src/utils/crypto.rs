//! Cryptographic utilities for secure data handling in Eule.
//!
//! This module provides a set of cryptographic functions for key derivation,
//! encryption, and decryption.
//! It uses Argon2id for key derivation and AES-256-GCM for encryption.

use crate::error::EuleError;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as Argon2OsRng, PasswordHasher, SaltString},
    Argon2,
};
use zeroize::Zeroizing;

/// The size of the encryption key in bytes.
const KEY_SIZE: usize = 32;

/// Cryptographic utility struct for Eule.
pub struct Crypto;

impl Crypto {
    /// Derives a key from a password using Argon2id.
    ///
    /// # Arguments
    ///
    /// * `password` - The password to derive the key from.
    /// * `salt` - A unique salt for this derivation.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a 32-byte key if successful, or an `EuleError` if key derivation fails.
    pub fn derive_key(password: &str, salt: &[u8]) -> Result<Zeroizing<[u8; KEY_SIZE]>, EuleError> {
        let salt = SaltString::encode_b64(salt)
            .map_err(|e| EuleError::KeyDerivationError(e.to_string()))?;

        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| EuleError::KeyDerivationError(e.to_string()))?;

        let mut key = Zeroizing::new([0u8; KEY_SIZE]);
        key.copy_from_slice(password_hash.hash.unwrap().as_bytes());
        Ok(key)
    }

    /// Encrypts a string using AES-256-GCM.
    ///
    /// # Arguments
    ///
    /// * `data` - The string to encrypt.
    /// * `key` - The 32-byte key to use for encryption.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the encrypted data if successful,
    /// or an `EuleError` if encryption fails.
    pub fn encrypt(data: &str, key: &[u8; KEY_SIZE]) -> Result<Vec<u8>, EuleError> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, data.as_bytes())
            .map_err(|e| EuleError::EncryptionError(e.to_string()))?;
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Decrypts a string that was encrypted using AES-256-GCM.
    ///
    /// # Arguments
    ///
    /// * `encrypted_data` - The encrypted data.
    /// * `key` - The 32-byte key used for encryption.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the decrypted string if successful,
    /// or an `EuleError` if decryption fails.
    pub fn decrypt(
        encrypted_data: &[u8],
        key: &[u8; KEY_SIZE],
    ) -> Result<Zeroizing<String>, EuleError> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let plaintext = cipher
            .decrypt(nonce, &encrypted_data[12..])
            .map_err(|e| EuleError::DecryptionError(e.to_string()))?;
        String::from_utf8(plaintext)
            .map(Zeroizing::new)
            .map_err(|e| EuleError::DecryptionError(e.to_string()))
    }

    /// Generates a random salt for key derivation.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a 16-byte random salt if successful,
    /// or an `EuleError` if salt generation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use eule::utils::crypto::Crypto;
    ///
    /// let salt = Crypto::generate_salt().unwrap();
    /// assert_eq!(salt.len(), 16);
    /// ```
    pub fn generate_salt() -> Result<[u8; 16], EuleError> {
        let salt = SaltString::generate(&mut Argon2OsRng);
        let mut salt_bytes = [0u8; 16];
        salt_bytes.copy_from_slice(&salt.as_str().as_bytes()[..16]);
        Ok(salt_bytes)
    }
}
