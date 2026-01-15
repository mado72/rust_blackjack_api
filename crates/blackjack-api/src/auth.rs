use serde::{Deserialize, Serialize};

/// JWT (JSON Web Token) claims structure
///
/// This structure represents the payload of a JWT token used for authenticating
/// users in the Blackjack API. Each token binds a user's email and ID and includes
/// an expiration timestamp.
///
/// # Security
///
/// - Tokens are signed using HMAC-SHA256 with a secret key (configured in `AppConfig`)
/// - The `exp` field enforces automatic token expiration
/// - Tokens are validated on every protected endpoint request
///
/// # Token Lifecycle
///
/// 1. User requests token via `POST /api/v1/auth/login` with email and password
/// 2. Server validates credentials
/// 3. Server generates JWT with these claims and signs it
/// 4. Client includes token in `Authorization: Bearer <token>` header
/// 5. Middleware validates token and extracts claims for each protected request
/// 6. Token automatically expires after `expiration_hours` (default: 24h)
///
/// # Example
///
/// ```
/// use blackjack_api::auth::Claims;
///
/// let claims = Claims {
///     user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
///     email: "user@example.com".to_string(),
///     exp: 1704672000, // Unix timestamp
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// User's unique ID
    ///
    /// This uniquely identifies the user across the system. It's used to:
    /// - Identify the user in game operations
    /// - Form the rate limiting key
    /// - Ensure users can only act on their own behalf
    pub user_id: String,

    /// User's email address
    ///
    /// The email associated with the user account for reference.
    pub email: String,

    /// Token expiration time as Unix timestamp (seconds since epoch)
    ///
    /// The JWT library automatically validates this field. Once the current time
    /// exceeds this timestamp, the token is considered invalid and authentication
    /// will fail with a 401 error.
    ///
    /// Example: 1704672000 represents January 8, 2024, 00:00:00 UTC
    pub exp: usize,
}
