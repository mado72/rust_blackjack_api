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
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use blackjack_core::GameResult;
use blackjack_service::{DrawCardResponse, GameStateResponse, PlayerStateResponse};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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

// ============================================================================
// Health Check Endpoints
// ============================================================================

/// Health check response
///
/// Provides basic server health information including uptime and version.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Server health status
    pub status: String,
    
    /// Server uptime in seconds since startup
    pub uptime_seconds: u64,
    
    /// API version
    pub version: String,
}

/// Readiness check response
///
/// Provides detailed readiness information for all system components.
#[derive(Debug, Serialize)]
pub struct ReadyResponse {
    /// Overall readiness status
    pub ready: bool,
    
    /// Individual component health checks
    pub checks: HashMap<String, String>,
}

/// Basic health check endpoint
///
/// Returns the current health status of the server. This endpoint is useful
/// for load balancers and monitoring systems to verify the server is running.
///
/// # Endpoint
///
/// `GET /health`
///
/// # Authentication
///
/// No authentication required (public endpoint).
///
/// # Response
///
/// **Success (200 OK)**:
/// ```json
/// {
///   "status": "healthy",
///   "uptime_seconds": 3600,
///   "version": "1.0.0"
/// }
/// ```
///
/// # Example
///
/// ```bash
/// curl http://localhost:8080/health
/// ```
#[tracing::instrument]
pub async fn health_check() -> Json<HealthResponse> {
    // Calculate uptime from process start
    // In production, this would use a global start time variable
    static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    let start = START_TIME.get_or_init(std::time::Instant::now);
    let uptime_seconds = start.elapsed().as_secs();

    Json(HealthResponse {
        status: "healthy".to_string(),
        uptime_seconds,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Readiness check endpoint
///
/// Returns detailed readiness information for all system components.
/// This endpoint can be used by orchestration systems (like Kubernetes)
/// to determine if the service is ready to accept traffic.
///
/// # Endpoint
///
/// `GET /health/ready`
///
/// # Authentication
///
/// No authentication required (public endpoint).
///
/// # Response
///
/// **Success (200 OK)**:
/// ```json
/// {
///   "ready": true,
///   "checks": {
///     "memory": "ok",
///     "config": "loaded",
///     "future_sqlite": "pending",
///     "future_metrics": "pending"
///   }
/// }
/// ```
///
/// # Future Enhancements
///
/// In future versions, this endpoint will include:
/// - Database connection check (SQLite)
/// - Metrics system availability
/// - External service dependencies
///
/// # Example
///
/// ```bash
/// curl http://localhost:8080/health/ready
/// ```
#[tracing::instrument]
pub async fn ready_check() -> Json<ReadyResponse> {
    let mut checks = HashMap::new();
    checks.insert("memory".to_string(), "ok".to_string());
    checks.insert("config".to_string(), "loaded".to_string());
    checks.insert("future_sqlite".to_string(), "pending".to_string());
    checks.insert("future_metrics".to_string(), "pending".to_string());

    Json(ReadyResponse {
        ready: true,
        checks,
    })
}

// ============================================================================
// Game Management Endpoints
// ============================================================================

/// Request to create a new game
///
/// # Validation
///
/// - Must contain 1-10 unique email addresses
/// - Email addresses must not be empty
#[derive(Debug, Deserialize)]
pub struct CreateGameRequest {
    /// List of player email addresses
    ///
    /// Must contain between 1 and 10 unique, non-empty emails
    pub emails: Vec<String>,
}

/// Response for game creation
#[derive(Debug, Serialize)]
pub struct CreateGameResponse {
    /// Unique identifier for the created game
    pub game_id: Uuid,
    
    /// Success message
    pub message: String,
    
    /// Number of players in the game
    pub player_count: usize,
}

/// Creates a new game with specified players
///
/// This endpoint initializes a new blackjack game with 1-10 players.
/// Each game has its own 52-card deck and independent state.
///
/// # Endpoint
///
/// `POST /api/v1/games`
///
/// # Authentication
///
/// No authentication required (public endpoint).
///
/// # Request Body
///
/// ```json
/// {
///   "emails": [
///     "player1@example.com",
///     "player2@example.com",
///     "player3@example.com"
///   ]
/// }
/// ```
///
/// # Response
///
/// **Success (200 OK)**:
/// ```json
/// {
///   "game_id": "550e8400-e29b-41d4-a716-446655440000",
///   "message": "Game created successfully",
///   "player_count": 3
/// }
/// ```
///
/// # Errors
///
/// - **400 Bad Request** - Invalid player count
///   ```json
///   {
///     "message": "Invalid number of players",
///     "code": "INVALID_PLAYER_COUNT",
///     "status": 400,
///     "details": {
///       "min": "1",
///       "max": "10",
///       "provided": "15"
///     }
///   }
///   ```
///
/// - **400 Bad Request** - Empty email address
///   ```json
///   {
///     "message": "Email cannot be empty",
///     "code": "INVALID_EMAIL",
///     "status": 400
///   }
///   ```
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:8080/api/v1/games \
///   -H "Content-Type: application/json" \
///   -d '{
///     "emails": ["player1@example.com", "player2@example.com"]
///   }'
/// ```
#[tracing::instrument(skip(state))]
pub async fn create_game(
    State(state): State<crate::AppState>,
    Json(payload): Json<CreateGameRequest>,
) -> Result<Json<CreateGameResponse>, ApiError> {
    // Validate player count
    let player_count = payload.emails.len();
    let min = state.game_service.config().min_players as usize;
    let max = state.game_service.config().max_players as usize;

    if player_count < min || player_count > max {
        let mut details = HashMap::new();
        details.insert("min".to_string(), min.to_string());
        details.insert("max".to_string(), max.to_string());
        details.insert("provided".to_string(), player_count.to_string());

        return Err(ApiError::new(StatusCode::BAD_REQUEST, "INVALID_PLAYER_COUNT", "Invalid number of players").with_details(details));
    }

    // Create game via service
    let game_id = state.game_service.create_game(payload.emails)?;

    tracing::info!(
        game_id = %game_id,
        player_count = player_count,
        "Game created successfully"
    );

    Ok(Json(CreateGameResponse {
        game_id,
        message: "Game created successfully".to_string(),
        player_count,
    }))
}

/// Retrieves the current state of a game
///
/// Returns complete game state including all players, their cards,
/// points, and the number of cards remaining in the deck.
///
/// # Endpoint
///
/// `GET /api/v1/games/:game_id`
///
/// # Authentication
///
/// **Required** - Must include valid JWT token in Authorization header.
///
/// # Path Parameters
///
/// - `game_id` - UUID of the game
///
/// # Response
///
/// **Success (200 OK)**:
/// ```json
/// {
///   "players": {
///     "player1@example.com": {
///       "points": 18,
///       "cards_history": [
///         {
///           "id": "card-uuid-1",
///           "name": "King",
///           "value": 10,
///           "suit": "Hearts"
///         },
///         {
///           "id": "card-uuid-2",
///           "name": "8",
///           "value": 8,
///           "suit": "Diamonds"
///         }
///       ],
///       "busted": false
///     }
///   },
///   "cards_in_deck": 48,
///   "finished": false
/// }
/// ```
///
/// # Errors
///
/// - **401 Unauthorized** - Missing or invalid JWT token
/// - **404 Not Found** - Game does not exist
///
/// # Example
///
/// ```bash
/// curl http://localhost:8080/api/v1/games/550e8400-e29b-41d4-a716-446655440000 \
///   -H "Authorization: Bearer YOUR_JWT_TOKEN"
/// ```
#[tracing::instrument(skip(state))]
pub async fn get_game_state(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Path(game_id): Path<Uuid>,
) -> Result<Json<GameStateResponse>, ApiError> {
    // Verify the game_id matches the token
    let token_game_id = Uuid::parse_str(&claims.game_id).map_err(|_| {
        ApiError::new(StatusCode::BAD_REQUEST, "INVALID_GAME_ID", "Invalid game ID in token")
    })?;

    if game_id != token_game_id {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "GAME_MISMATCH",
            "Token is for a different game",
        ));
    }

    let state_response = state.game_service.get_game_state(game_id)?;

    Ok(Json(state_response))
}

/// Request to draw a card
///
/// Players use this endpoint to draw cards from the deck during their turn.
#[derive(Debug, Deserialize)]
pub struct DrawCardRequest {
    // No body needed - email comes from JWT token
}

/// Draws a card for the authenticated player
///
/// Removes a random card from the deck and adds it to the player's hand.
/// Automatically calculates the new point total and checks for bust.
///
/// # Endpoint
///
/// `POST /api/v1/games/:game_id/draw`
///
/// # Authentication
///
/// **Required** - Player email extracted from JWT token.
///
/// # Path Parameters
///
/// - `game_id` - UUID of the game
///
/// # Response
///
/// **Success (200 OK)**:
/// ```json
/// {
///   "card": {
///     "id": "card-uuid",
///     "name": "Ace",
///     "value": 11,
///     "suit": "Spades"
///   },
///   "current_points": 21,
///   "busted": false,
///   "cards_remaining": 47,
///   "cards_history": [
///     {
///       "id": "card-uuid-1",
///       "name": "King",
///       "value": 10,
///       "suit": "Hearts"
///     },
///     {
///       "id": "card-uuid-2",
///       "name": "Ace",
///       "value": 11,
///       "suit": "Spades"
///     }
///   ]
/// }
/// ```
///
/// # Errors
///
/// - **401 Unauthorized** - Missing or invalid JWT token
/// - **403 Forbidden** - Game already finished
///   ```json
///   {
///     "message": "Cannot draw cards from a finished game",
///     "code": "GAME_FINISHED",
///     "status": 403
///   }
///   ```
/// - **404 Not Found** - Game or player does not exist
/// - **410 Gone** - Deck is empty
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:8080/api/v1/games/550e8400-e29b-41d4-a716-446655440000/draw \
///   -H "Authorization: Bearer YOUR_JWT_TOKEN"
/// ```
#[tracing::instrument(skip(state), fields(player_email = %claims.email))]
pub async fn draw_card(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Path(game_id): Path<Uuid>,
) -> Result<Json<DrawCardResponse>, ApiError> {
    // Verify the game_id matches the token
    let token_game_id = Uuid::parse_str(&claims.game_id).map_err(|_| {
        ApiError::new(StatusCode::BAD_REQUEST, "INVALID_GAME_ID", "Invalid game ID in token")
    })?;

    if game_id != token_game_id {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "GAME_MISMATCH",
            "Token is for a different game",
        ));
    }

    let response = state.game_service.draw_card(game_id, &claims.email)?;

    Ok(Json(response))
}

/// Request to change an Ace value
///
/// Allows players to change an Ace card between 1 and 11 points.
#[derive(Debug, Deserialize)]
pub struct SetAceValueRequest {
    /// UUID of the Ace card to modify
    pub card_id: Uuid,
    
    /// Whether to count the Ace as 11 (true) or 1 (false)
    pub as_eleven: bool,
}

/// Changes the value of an Ace card
///
/// Players can change an Ace between 1 and 11 points at any time
/// before the game is finished. The same Ace can be changed multiple times.
///
/// # Endpoint
///
/// `PUT /api/v1/games/:game_id/ace`
///
/// # Authentication
///
/// **Required** - Player email extracted from JWT token.
///
/// # Path Parameters
///
/// - `game_id` - UUID of the game
///
/// # Request Body
///
/// ```json
/// {
///   "card_id": "card-uuid",
///   "as_eleven": true
/// }
/// ```
///
/// # Response
///
/// **Success (200 OK)**:
/// ```json
/// {
///   "points": 21,
///   "busted": false
/// }
/// ```
///
/// # Errors
///
/// - **401 Unauthorized** - Missing or invalid JWT token
/// - **403 Forbidden** - Game already finished
/// - **404 Not Found** - Game, player, or card does not exist
///
/// # Example
///
/// ```bash
/// curl -X PUT http://localhost:8080/api/v1/games/550e8400-e29b-41d4-a716-446655440000/ace \
///   -H "Authorization: Bearer YOUR_JWT_TOKEN" \
///   -H "Content-Type: application/json" \
///   -d '{
///     "card_id": "card-uuid",
///     "as_eleven": false
///   }'
/// ```
#[tracing::instrument(skip(state), fields(player_email = %claims.email))]
pub async fn set_ace_value(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Path(game_id): Path<Uuid>,
    Json(payload): Json<SetAceValueRequest>,
) -> Result<Json<PlayerStateResponse>, ApiError> {
    // Verify the game_id matches the token
    let token_game_id = Uuid::parse_str(&claims.game_id).map_err(|_| {
        ApiError::new(StatusCode::BAD_REQUEST, "INVALID_GAME_ID", "Invalid game ID in token")
    })?;

    if game_id != token_game_id {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "GAME_MISMATCH",
            "Token is for a different game",
        ));
    }

    let response = state
        .game_service
        .set_ace_value(game_id, &claims.email, payload.card_id, payload.as_eleven)?;

    Ok(Json(response))
}

/// Finishes a game and calculates results
///
/// Marks the game as finished and determines the winner based on
/// the highest score without busting. No further cards can be drawn
/// or Ace values changed after this.
///
/// # Endpoint
///
/// `POST /api/v1/games/:game_id/finish`
///
/// # Authentication
///
/// **Required** - Must include valid JWT token.
///
/// # Path Parameters
///
/// - `game_id` - UUID of the game
///
/// # Response
///
/// **Success (200 OK)**:
/// ```json
/// {
///   "winner": "player1@example.com",
///   "tied_players": [],
///   "highest_score": 21,
///   "all_players": {
///     "player1@example.com": {
///       "points": 21,
///       "cards_count": 2,
///       "busted": false
///     },
///     "player2@example.com": {
///       "points": 19,
///       "cards_count": 3,
///       "busted": false
///     }
///   }
/// }
/// ```
///
/// # Errors
///
/// - **401 Unauthorized** - Missing or invalid JWT token
/// - **404 Not Found** - Game does not exist
/// - **409 Conflict** - Game already finished
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:8080/api/v1/games/550e8400-e29b-41d4-a716-446655440000/finish \
///   -H "Authorization: Bearer YOUR_JWT_TOKEN"
/// ```
#[tracing::instrument(skip(state))]
pub async fn finish_game(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Path(game_id): Path<Uuid>,
) -> Result<Json<GameResult>, ApiError> {
    // Verify the game_id matches the token
    let token_game_id = Uuid::parse_str(&claims.game_id).map_err(|_| {
        ApiError::new(StatusCode::BAD_REQUEST, "INVALID_GAME_ID", "Invalid game ID in token")
    })?;

    if game_id != token_game_id {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "GAME_MISMATCH",
            "Token is for a different game",
        ));
    }

    let result = state.game_service.finish_game(game_id)?;

    Ok(Json(result))
}

/// Retrieves the results of a finished game
///
/// Returns the winner, final scores, and complete player information.
/// Can only be called after the game has been finished.
///
/// # Endpoint
///
/// `GET /api/v1/games/:game_id/results`
///
/// # Authentication
///
/// **Required** - Must include valid JWT token.
///
/// # Path Parameters
///
/// - `game_id` - UUID of the game
///
/// # Response
///
/// **Success (200 OK)**:
/// ```json
/// {
///   "winner": "player1@example.com",
///   "tied_players": [],
///   "highest_score": 21,
///   "all_players": {
///     "player1@example.com": {
///       "points": 21,
///       "cards_count": 2,
///       "busted": false
///     }
///   }
/// }
/// ```
///
/// # Errors
///
/// - **401 Unauthorized** - Missing or invalid JWT token
/// - **404 Not Found** - Game does not exist
/// - **409 Conflict** - Game not yet finished
///   ```json
///   {
///     "message": "Game is not finished yet",
///     "code": "GAME_NOT_FINISHED",
///     "status": 409
///   }
///   ```
///
/// # Example
///
/// ```bash
/// curl http://localhost:8080/api/v1/games/550e8400-e29b-41d4-a716-446655440000/results \
///   -H "Authorization: Bearer YOUR_JWT_TOKEN"
/// ```
#[tracing::instrument(skip(state))]
pub async fn get_game_results(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Path(game_id): Path<Uuid>,
) -> Result<Json<GameResult>, ApiError> {
    // Verify the game_id matches the token
    let token_game_id = Uuid::parse_str(&claims.game_id).map_err(|_| {
        ApiError::new(StatusCode::BAD_REQUEST, "INVALID_GAME_ID", "Invalid game ID in token")
    })?;

    if game_id != token_game_id {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "GAME_MISMATCH",
            "Token is for a different game",
        ));
    }

    // Get game state to check if finished
    let game_state = state.game_service.get_game_state(game_id)?;
    
    if !game_state.finished {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "GAME_NOT_FINISHED",
            "Game is not finished yet",
        ));
    }

    // Game is finished, calculate results
    let result = state.game_service.get_game_results(game_id)?;

    Ok(Json(result))
}
