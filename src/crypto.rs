//! Cryptographic utilities.

use crate::BackupResult;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use sha2::{Digest, Sha256};

/// The number of bytes to use for an AES key.
pub const AES_KEY_SIZE: usize = 32;

/// The number of bytes to use for an AES nonce.
pub const AES_NONCE_SIZE: usize = 12;

/// Encrypts data with AES.
pub fn aes_encrypt(key: &[u8; AES_KEY_SIZE], plaintext: &[u8]) -> BackupResult<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let nonce_slice: [u8; AES_NONCE_SIZE] = rand::random();
    let nonce = Nonce::from(nonce_slice);
    let ciphertext = cipher.encrypt(&nonce, plaintext)?;

    let mut ciphertext_with_nonce = nonce_slice.to_vec();
    ciphertext_with_nonce.extend(ciphertext);

    Ok(ciphertext_with_nonce)
}

/// Decrypts data with AES.
pub fn aes_decrypt(
    key: &[u8; AES_KEY_SIZE],
    ciphertext_with_nonce: &[u8],
) -> BackupResult<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let (nonce_slice, ciphertext) = ciphertext_with_nonce.split_at(AES_NONCE_SIZE);
    let nonce_slice_sized: [u8; AES_NONCE_SIZE] =
        nonce_slice.try_into().map_err(|_| aes_gcm::Error)?;
    let nonce = Nonce::from(nonce_slice_sized);
    let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref())?;

    Ok(plaintext)
}

/// Converts a password of arbitrary length to an AES key by performing a SHA-256 hash.
pub fn password_to_key(password: &str) -> [u8; AES_KEY_SIZE] {
    let mut hasher = Sha256::new();
    hasher.update(password);
    let result = hasher.finalize();
    result.into()
}

/// Crypto tests.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes() {
        let aes_message = "Hello, AES!";
        let key = password_to_key("password123");
        let aes_encrypted = aes_encrypt(&key, aes_message.as_bytes()).unwrap();
        let aes_decrypted = aes_decrypt(&key, &aes_encrypted).unwrap();
        let aes_decrypted_message = std::str::from_utf8(&aes_decrypted).unwrap();
        assert_eq!(aes_decrypted_message, aes_message);
        assert_ne!(aes_encrypted, aes_message.as_bytes());
    }

    #[test]
    fn test_password_to_key() {
        let key1 = password_to_key("password123");
        let key2 = password_to_key("password123");
        let key3 = password_to_key("password124");
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}
