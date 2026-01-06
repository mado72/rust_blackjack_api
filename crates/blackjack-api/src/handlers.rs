//! HTTP request handlers for the Blackjack API
//!
//! This module contains all HTTP endpoint handlers that process incoming requests
//! and return responses. Handlers are responsible for:
//!
//! - Extracting and validating request data
//! - Calling service layer methods
//! - Converting service responses to HTTP responses
//! - Handling errors and returning appropriate status codes
//!
//! # Handler Design
//!
//! All handlers follow Axum's handler pattern:
//! - Accept extractors as parameters (State, Json, Path, etc.)
//! - Return `Result<T, ApiError>` where T implements `IntoResponse`
//! - Use `#[tracing::instrument]` for automatic logging
//!
//! # Example
//!
//! ```ignore
//! use axum::{Json, extract::State};
//! use blackjack_api::error::ApiError;
//!
//! async fn example_handler(
//!     State(state): State<AppState>,
//!     Json(payload): Json<RequestType>,
//! ) -> Result<Json<ResponseType>, ApiError> {
//!     // Handler implementation
//!     todo!()
//! }
//! ```

use crate::auth::Claims;
use crate::error::ApiError;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

/// Request payload for player authentication
///
/// Used by the `POST /api/v1/auth/login` endpoint to authenticate
/// a player for a specific game session.
///
/// # Validation
///
/// - `email` must not be empty (validated by service layer)
/// - `game_id` must be a valid UUID v4 string
/// - Player must exist in the specified game
///
/// # Example
///
/// ```json
/// {
///   "email": "player@example.com",
///   "game_id": "550e8400-e29b-41d4-a716-446655440000"
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// Player's email address
    ///
    /// Must match an email in the game's player list
    pub email: String,
    
    /// Game UUID as a string
    ///
    /// Must be a valid UUID v4 format and correspond to an existing game
    pub game_id: String,
}

/// Response payload for successful authentication
///
/// Contains the JWT token and its expiration information.
/// The client should store the token and include it in the
/// `Authorization: Bearer <token>` header for all protected endpoints.
///
/// # Example
///
/// ```json
/// {
///   "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
///   "expires_in": 86400
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    /// JWT token for authentication
    ///
    /// This token should be included in the Authorization header:
    /// `Authorization: Bearer <token>`
    pub token: String,
    
    /// Token expiration time in seconds
    ///
    /// Calculated as `expiration_hours * 3600`
    /// Default: 86400 (24 hours)
    pub expires_in: u64,
}

/// Authenticates a player for a game session
///
/// This handler validates that a player exists in a game and issues a JWT token
/// that grants access to protected endpoints. The token binds the player's email
/// to a specific game ID.
///
/// # Endpoint
///
/// `POST /api/v1/auth/login`
///
/// # Authentication
///
/// This is a public endpoint - no JWT required.
///
/// # Request Body
///
/// ```json
/// {
///   "email": "player@example.com",
///   "game_id": "550e8400-e29b-41d4-a716-446655440000"
/// }
/// ```
///
/// # Response
///
/// **Success (200 OK)**:
/// ```json
/// {
///   "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJlbWFpbCI6InBsYXllckBleGFtcGxlLmNvbSIsImdhbWVfaWQiOiI1NTBlODQwMC1lMjliLTQxZDQtYTcxNi00NDY2NTU0NDAwMDAiLCJleHAiOjE3MDQ3NTg0MDB9.signature",
///   "expires_in": 86400
/// }
/// ```
///
/// # Errors
///
/// - **400 Bad Request** - Invalid game_id format
///   ```json
///   {
///     "message": "Invalid game ID format",
///     "code": "INVALID_GAME_ID",
///     "status": 400
///   }
///   ```
///
/// - **403 Forbidden** - Player not found in game
///   ```json
///   {
///     "message": "Player not found in this game",
///     "code": "PLAYER_NOT_IN_GAME",
///     "status": 403
///   }
///   ```
///
/// - **404 Not Found** - Game does not exist
///   ```json
///   {
///     "message": "Game not found",
///     "code": "GAME_NOT_FOUND",
///     "status": 404
///   }
///   ```
///
/// - **500 Internal Server Error** - Token generation failed
///   ```json
///   {
///     "message": "Failed to generate authentication token",
///     "code": "TOKEN_GENERATION_FAILED",
///     "status": 500
///   }
///   ```
///
/// # Security
///
/// - Tokens are signed with HMAC-SHA256 using the configured JWT secret
/// - Token includes expiration timestamp (validated automatically)
/// - Each token is bound to a specific game and player
/// - Failed authentication attempts are logged with warning level
///
/// # Logging
///
/// - Info: Successful authentication with email and game_id
/// - Warn: Authentication attempt for non-existent player
/// - Error: JWT token generation failures
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:8080/api/v1/auth/login \
///   -H "Content-Type: application/json" \
///   -d '{
///     "email": "player1@example.com",
///     "game_id": "550e8400-e29b-41d4-a716-446655440000"
///   }'
/// ```
#[tracing::instrument(skip(state))]
pub async fn login(
    State(state): State<crate::AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // Parse game_id from string to UUID
    let game_id = uuid::Uuid::parse_str(&payload.game_id).map_err(|_| {
        ApiError::new(StatusCode::BAD_REQUEST, "INVALID_GAME_ID", "Invalid game ID format")
    })?;

    // Validate that the player is in the game
    let game_state = state.game_service.get_game_state(game_id)?;

    if !game_state.players.contains_key(&payload.email) {
        tracing::warn!(
            email = payload.email,
            game_id = payload.game_id,
            "Authentication attempt for non-existent player"
        );
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "PLAYER_NOT_IN_GAME",
            "Player not found in this game",
        ));
    }

    // Calculate expiration time
    let expiration = chrono::Utc::now()
        + chrono::Duration::hours(state.config.jwt.expiration_hours as i64);

    let claims = Claims {
        email: payload.email.clone(),
        game_id: payload.game_id.clone(),
        exp: expiration.timestamp() as usize,
    };

    // Generate JWT token
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt.secret.as_bytes()),
    )
    .map_err(|err| {
        tracing::error!(error = ?err, "Failed to generate JWT token");
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "TOKEN_GENERATION_FAILED",
            "Failed to generate authentication token",
        )
    })?;

    tracing::info!(
        email = payload.email,
        game_id = payload.game_id,
        "Player authenticated successfully"
    );

    Ok(Json(LoginResponse {
        token,
        expires_in: state.config.jwt.expiration_hours * 3600, // Convert to seconds
    }))
}
