//! Unicrypt — Cryptographic Mechanism Manager.
//!
//! The initial implementation supports password hashing and verification with
//! Argon2. Password hashing is one-way; it is not reversible encryption.

use argon2::{
    password_hash::{
        rand_core::OsRng, ErrorKind, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};

#[derive(Debug, PartialEq, Eq)]
pub enum UnicryptError {
    HashingFailed,
    InvalidEncodedHash,
    VerificationFailed,
}

impl std::fmt::Display for UnicryptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HashingFailed => write!(f, "password hashing failed"),
            Self::InvalidEncodedHash => write!(f, "encoded password hash is invalid"),
            Self::VerificationFailed => write!(f, "password verification failed"),
        }
    }
}

impl std::error::Error for UnicryptError {}

/// Hash a password using Argon2 and a cryptographically secure random salt.
pub fn hash_password(password: &str) -> Result<String, UnicryptError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| UnicryptError::HashingFailed)
}

/// Verify a password against an encoded hash.
pub fn verify_password(
    password: &str,
    encoded_hash: &str,
) -> Result<bool, UnicryptError> {
    let parsed_hash =
        PasswordHash::new(encoded_hash).map_err(|_| UnicryptError::InvalidEncodedHash)?;

    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == Some(ErrorKind::Password) => Ok(false),
        Err(_) => Err(UnicryptError::VerificationFailed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifies_the_original_password() {
        let encoded_hash = hash_password("correct horse battery staple").unwrap();
        assert!(verify_password("correct horse battery staple", &encoded_hash).unwrap());
    }

    #[test]
    fn rejects_a_different_password() {
        let encoded_hash = hash_password("first password").unwrap();
        assert!(!verify_password("different password", &encoded_hash).unwrap());
    }

    #[test]
    fn uses_a_new_random_salt_for_each_hash() {
        let first = hash_password("same password").unwrap();
        let second = hash_password("same password").unwrap();
        assert_ne!(first, second);
    }

    #[test]
    fn rejects_a_malformed_hash() {
        assert_eq!(
            verify_password("password", "not-an-encoded-hash"),
            Err(UnicryptError::InvalidEncodedHash)
        );
    }
}
