/// Password hashing and verification using Argon2id
///
/// This module provides secure password hashing using the Argon2id algorithm,
/// which is the recommended password hashing algorithm by OWASP.
///
/// # Security Parameters
///
/// - Algorithm: Argon2id (hybrid mode combining Argon2i and Argon2d)
/// - Memory cost: 19456 KiB (19 MiB) - OWASP recommended
/// - Time cost: 2 iterations - OWASP recommended
/// - Parallelism: 1 thread
/// - Salt: Random 16 bytes (generated automatically)
///
/// # Examples
///
/// ```
/// use blackjack_core::password::{hash_password, verify_password};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let password = "MySecureP@ssw0rd";
/// let hash = hash_password(password)?;
///
/// assert!(verify_password(password, &hash)?);
/// assert!(!verify_password("WrongPassword", &hash)?);
/// # Ok(())
/// # }
/// ```

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString},
    Argon2, ParamsBuilder,
};
use std::fmt;

/// Errors that can occur during password hashing or verification
#[derive(Debug)]
pub enum HashError {
    /// Password is empty or invalid
    InvalidPassword,
    /// Failed to hash the password
    HashingFailed(String),
    /// Failed to verify the password
    VerificationFailed(String),
}

impl fmt::Display for HashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HashError::InvalidPassword => write!(f, "Password is empty or invalid"),
            HashError::HashingFailed(msg) => write!(f, "Password hashing failed: {}", msg),
            HashError::VerificationFailed(msg) => write!(f, "Password verification failed: {}", msg),
        }
    }
}

impl std::error::Error for HashError {}

/// Hashes a password using Argon2id with OWASP recommended parameters
///
/// # Arguments
///
/// * `password` - The plaintext password to hash
///
/// # Returns
///
/// * `Ok(String)` - The password hash in PHC string format
/// * `Err(HashError)` - If hashing fails
///
/// # Security
///
/// - Uses Argon2id algorithm (hybrid mode)
/// - Memory cost: 19456 KiB (19 MiB)
/// - Time cost: 2 iterations
/// - Random 16-byte salt generated per hash
///
/// # Examples
///
/// ```
/// use blackjack_core::password::hash_password;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let password = "MySecureP@ssw0rd";
/// let hash = hash_password(password)?;
/// println!("Hash: {}", hash);
/// # Ok(())
/// # }
/// ```
#[tracing::instrument(skip(password))]
pub fn hash_password(password: &str) -> Result<String, HashError> {
    if password.is_empty() {
        return Err(HashError::InvalidPassword);
    }

    // OWASP recommended parameters for Argon2id
    let params = ParamsBuilder::new()
        .m_cost(19456) // 19 MiB memory
        .t_cost(2)      // 2 iterations
        .p_cost(1)      // 1 thread
        .build()
        .map_err(|e| HashError::HashingFailed(e.to_string()))?;

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    );

    // Generate random salt
    let salt = SaltString::generate(&mut OsRng);

    // Hash password
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| HashError::HashingFailed(e.to_string()))?;

    tracing::debug!("Password hashed successfully");
    Ok(password_hash.to_string())
}

/// Verifies a password against a hash using constant-time comparison
///
/// # Arguments
///
/// * `password` - The plaintext password to verify
/// * `hash` - The password hash in PHC string format
///
/// # Returns
///
/// * `Ok(true)` - Password matches the hash
/// * `Ok(false)` - Password does not match the hash
/// * `Err(HashError)` - If verification fails due to invalid hash format
///
/// # Security
///
/// Uses constant-time comparison to prevent timing attacks.
///
/// # Examples
///
/// ```
/// use blackjack_core::password::{hash_password, verify_password};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let password = "MySecureP@ssw0rd";
/// let hash = hash_password(password)?;
///
/// // Correct password
/// assert!(verify_password(password, &hash)?);
///
/// // Wrong password
/// assert!(!verify_password("WrongPassword", &hash)?);
/// # Ok(())
/// # }
/// ```
#[tracing::instrument(skip(password, hash))]
pub fn verify_password(password: &str, hash: &str) -> Result<bool, HashError> {
    if password.is_empty() {
        return Err(HashError::InvalidPassword);
    }

    // Parse the hash
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| HashError::VerificationFailed(format!("Invalid hash format: {}", e)))?;

    // Verify password using constant-time comparison
    let argon2 = Argon2::default();
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => {
            tracing::debug!("Password verification successful");
            Ok(true)
        }
        Err(argon2::password_hash::Error::Password) => {
            tracing::debug!("Password verification failed: incorrect password");
            Ok(false)
        }
        Err(e) => {
            tracing::warn!("Password verification error: {}", e);
            Err(HashError::VerificationFailed(e.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_success() {
        let password = "MySecureP@ssw0rd";
        let result = hash_password(password);
        assert!(result.is_ok());
        
        let hash = result.unwrap();
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn test_hash_password_empty() {
        let result = hash_password("");
        assert!(matches!(result, Err(HashError::InvalidPassword)));
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "MySecureP@ssw0rd";
        let hash = hash_password(password).unwrap();
        let result = verify_password(password, &hash);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "MySecureP@ssw0rd";
        let hash = hash_password(password).unwrap();
        let result = verify_password("WrongPassword", &hash);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_verify_password_empty() {
        let hash = "$argon2id$v=19$m=19456,t=2,p=1$test$test";
        let result = verify_password("", hash);
        assert!(matches!(result, Err(HashError::InvalidPassword)));
    }

    #[test]
    fn test_verify_password_invalid_hash() {
        let result = verify_password("password", "invalid-hash");
        assert!(matches!(result, Err(HashError::VerificationFailed(_))));
    }

    #[test]
    fn test_hash_unique_salts() {
        let password = "MySecureP@ssw0rd";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();
        
        // Different hashes due to different salts
        assert_ne!(hash1, hash2);
        
        // Both should verify correctly
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn test_constant_time_verification() {
        // This test ensures the function doesn't panic on different inputs
        let password = "MySecureP@ssw0rd";
        let hash = hash_password(password).unwrap();
        
        // All these should return Ok(bool), not panic
        let _ = verify_password("short", &hash);
        let _ = verify_password("very_long_password_with_many_characters", &hash);
        let _ = verify_password(password, &hash);
    }
}
