use eule::utils::crypto::Crypto;

#[test]
fn test_key_derivation() {
    let password = "test_password";
    let salt = Crypto::generate_salt().unwrap();
    let key = Crypto::derive_key(password, &salt).unwrap();
    assert_eq!(key.len(), 32);
}

#[test]
fn test_encryption_decryption() {
    let password = "test_password";
    let salt = Crypto::generate_salt().unwrap();
    let key = Crypto::derive_key(password, &salt).unwrap();
    let data = "Hello, World!";
    let encrypted = Crypto::encrypt(data, &key).unwrap();
    let decrypted = Crypto::decrypt(&encrypted, &key).unwrap();
    assert_eq!(data, &*decrypted);
}

#[test]
fn test_encryption_tampering() {
    let password = "test_password";
    let salt = Crypto::generate_salt().unwrap();
    let key = Crypto::derive_key(password, &salt).unwrap();
    let data = "Hello, World!";
    let mut encrypted = Crypto::encrypt(data, &key).unwrap();
    // Tamper with the encrypted data
    encrypted[20] ^= 1;
    assert!(Crypto::decrypt(&encrypted, &key).is_err());
}

#[test]
fn test_different_keys() {
    let password1 = "password1";
    let password2 = "password2";
    let salt = Crypto::generate_salt().unwrap();
    let key1 = Crypto::derive_key(password1, &salt).unwrap();
    let key2 = Crypto::derive_key(password2, &salt).unwrap();
    let data = "Secret message";
    let encrypted = Crypto::encrypt(data, &key1).unwrap();
    assert!(Crypto::decrypt(&encrypted, &key2).is_err());
}
