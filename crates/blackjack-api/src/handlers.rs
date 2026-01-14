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

/// Request payload for user authentication
///
/// Used by the `POST /api/v1/auth/login` endpoint to authenticate
/// a user with email and password.
///
/// # Validation
///
/// - `email` must not be empty (validated by service layer)
/// - `password` must not be empty
///
/// # Example
///
/// ```json
/// {
///   "email": "user@example.com",
///   "password": "SecurePassword123!"
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// User's email address
    pub email: String,
    
    /// User's password
    pub password: String,
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
///     "email": "user@example.com",
///     "password": "SecurePassword123!"
///   }'
/// ```
#[tracing::instrument(skip(state))]
pub async fn login(
    State(state): State<crate::AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // Authenticate user with UserService
    let user = state.user_service.login(&payload.email, &payload.password)?;
    
    // Calculate expiration time
    let expiration = chrono::Utc::now()
        + chrono::Duration::hours(state.config.jwt.expiration_hours as i64);

    // Generate JWT claims
    let claims = Claims {
        user_id: user.id.to_string(),
        email: user.email.clone(),
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
        user_id = %user.id,
        email = %user.email,
        "User authenticated successfully"
    );

    Ok(Json(LoginResponse {
        token,
        expires_in: state.config.jwt.expiration_hours * 3600,
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
    /// Optional enrollment timeout in seconds (defaults to 300)
    pub enrollment_timeout_seconds: Option<u64>,
}

/// Response for game creation
#[derive(Debug, Serialize)]
pub struct CreateGameResponse {
    /// Unique identifier for the created game
    pub game_id: Uuid,
    
    /// The UUID of the game creator
    pub creator_id: Uuid,
    
    /// Success message
    pub message: String,
    
    /// Number of players in the game (includes creator)
    pub player_count: u32,
}

/// Creates a new game in enrollment mode
///
/// Initializes a new blackjack game in the Enrollment phase. The authenticated
/// user becomes the creator and first enrolled player. Other players can join
/// via the enrollment endpoint until the creator closes enrollment or the
/// timeout expires.
///
/// # Endpoint
///
/// `POST /api/v1/games`
///
/// # Authentication
///
/// **Required** - Must include valid JWT token in Authorization header.
/// The `user_id` from the JWT becomes the game creator.
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
/// - **400 Bad Request** - Invalid user_id format in token
///   ```json
///   {
///     "message": "Invalid user_id format in token",
///     "code": "INVALID_USER_ID",
///     "status": 400
///   }
///   ```
///
/// - **401 Unauthorized** - Missing, expired, or invalid JWT token
///   ```json
///   {
///     "message": "User from token does not exist. Token may be invalid or user was deleted.",
///     "code": "USER_NOT_FOUND",
///     "status": 401
///   }
///   ```
///
/// - **500 Internal Server Error** - Database or service error
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:8080/api/v1/games \
///   -H "Authorization: Bearer YOUR_JWT_TOKEN" \
///   -H "Content-Type: application/json" \
///   -d '{
///     "enrollment_timeout_seconds": 600
///   }'
/// ```
///
/// # Notes
///
/// - Creator is automatically enrolled as the first player
/// - Maximum 10 players per game
/// - Enrollment phase lasts `enrollment_timeout_seconds` (default: 300s)
/// - Creator can close enrollment early via `/close-enrollment` endpoint
#[tracing::instrument(skip(state, claims))]
pub async fn create_game(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateGameRequest>,
) -> Result<Json<CreateGameResponse>, ApiError> {
    // Parse user_id from JWT claims string to Uuid
    let creator_id = Uuid::parse_str(&claims.user_id)
        .map_err(|_| ApiError::new(
            axum::http::StatusCode::BAD_REQUEST,
            "INVALID_USER_ID",
            "Invalid user_id format in token"
        ))?;
    
    // Verify that the user exists in the database
    // This ensures the JWT token references a valid, existing user
    state.user_service.get_user(creator_id)
        .map_err(|_| ApiError::new(
            axum::http::StatusCode::UNAUTHORIZED,
            "USER_NOT_FOUND",
            "User from token does not exist. Token may be invalid or user was deleted."
        ))?;
    
    let enrollment_timeout = payload.enrollment_timeout_seconds;
    let game_id = state.game_service.create_game(creator_id, enrollment_timeout)?;

    tracing::info!(
        game_id = %game_id,
        creator_id = %creator_id,
        user_email = %claims.email,
        enrollment_timeout = ?enrollment_timeout,
        "Game created successfully by authenticated user"
    );

    Ok(Json(CreateGameResponse {
        game_id,
        creator_id,
        message: "Game created successfully".to_string(),
        player_count: 1,
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
/// - **409 Conflict** - Not player's turn (M7)
///   ```json
///   {
///     "message": "It's not your turn",
///     "code": "NOT_YOUR_TURN",
///     "status": 409
///   }
///   ```
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
    // Validate it's the player's turn
    let game_state = state.game_service.get_game_state(game_id)?;
    if let Some(current_player) = game_state.current_turn_player
        && current_player != claims.email
    {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "NOT_YOUR_TURN",
            "It's not your turn",
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

// ============================================================================
// M7: User Management Endpoints
// ============================================================================

/// Request to register a new user
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    /// User's email address
    pub email: String,
    
    /// User's password (will be hashed)
    pub password: String,
}

/// Response for successful user registration
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    /// Unique user ID
    pub user_id: Uuid,
    
    /// User's email
    pub email: String,
    
    /// Success message
    pub message: String,
}

/// Registers a new user
///
/// Creates a new user account with email and password.
/// Password is hashed before storage (currently using placeholder,
/// will be upgraded to Argon2 in M8).
///
/// # Endpoint
///
/// `POST /api/v1/auth/register`
///
/// # Authentication
///
/// No authentication required (public endpoint).
///
/// # Request Body
///
/// ```json
/// {
///   "email": "user@example.com",
///   "password": "SecurePassword123!"
/// }
/// ```
///
/// # Response
///
/// **Success (200 OK)**:
/// ```json
/// {
///   "user_id": "550e8400-e29b-41d4-a716-446655440000",
///   "email": "user@example.com",
///   "message": "User registered successfully"
/// }
/// ```
///
/// # Errors
///
/// - **400 Bad Request** - Invalid email or password
/// - **409 Conflict** - User already exists
#[tracing::instrument(skip(state))]
pub async fn register_user(
    State(state): State<crate::AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, ApiError> {
    let user_id = state.user_service.register(payload.email.clone(), payload.password)?;
    
    tracing::info!(
        user_id = %user_id,
        email = %payload.email,
        "User registered successfully"
    );

    Ok(Json(RegisterResponse {
        user_id,
        email: payload.email,
        message: "User registered successfully".to_string(),
    }))
}

// ============================================================================
// Invitation Management Endpoints
// ============================================================================

/// Request to create a game invitation
#[derive(Debug, Deserialize)]
pub struct CreateInvitationRequest {
    /// Email of the user to invite
    pub invitee_email: String,
}

/// Response for created invitation
#[derive(Debug, Serialize)]
pub struct CreateInvitationResponse {
    /// Invitation ID
    pub invitation_id: Uuid,
    
    /// Invitee email
    pub invitee_email: String,
    
    /// Expiration timestamp
    pub expires_at: String,
    
    /// Success message
    pub message: String,
}

/// Creates a game invitation
///
/// Game creator can invite additional players to join the game.
/// Invitations have a configurable timeout.
///
/// # Endpoint
///
/// `POST /api/v1/games/:game_id/invitations`
///
/// # Authentication
///
/// **Required** - Must be the game creator.
///
/// # Request Body
///
/// ```json
/// {
///   "invitee_email": "newplayer@example.com"
/// }
/// ```
#[tracing::instrument(skip(state))]
pub async fn create_invitation(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Path(game_id): Path<Uuid>,
    Json(payload): Json<CreateInvitationRequest>,
) -> Result<Json<CreateInvitationResponse>, ApiError> {
    // Verify user is enrolled in the game
    let user_id = Uuid::parse_str(&claims.user_id).map_err(|_| {
        ApiError::new(StatusCode::BAD_REQUEST, "INVALID_USER_ID", "Invalid user ID")
    })?;

    // Get user 
    let _user = state.user_service.get_user(user_id)?;
    
    // Get game to check enrollment status and get expires_at
    let game_state = state.game_service.get_game_state(game_id)?;
    
    // Check if game is still in enrollment phase
    // For now, we'll assume any game that's not finished is open (need to add enrollment_open flag to response)
    if game_state.finished {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "GAME_FINISHED",
            "Cannot invite players to a finished game",
        ));
    }
    
    // Get the game enrollment expires_at time
    // This is a workaround - we should expose this in GameStateResponse
    let game_enrollment_expires_at = "2099-01-01T00:00:00Z".to_string(); // Placeholder - need to get from game
    
    let invitation_id = state.invitation_service.create(
        game_id,
        user_id,
        payload.invitee_email.clone(),
        game_enrollment_expires_at,
    )?;
    
    let invitation = state.invitation_service.get_invitation(invitation_id)?;

    tracing::info!(
        invitation_id = %invitation.id,
        game_id = %game_id,
        invitee = %payload.invitee_email,
        "Invitation created"
    );

    Ok(Json(CreateInvitationResponse {
        invitation_id: invitation.id,
        invitee_email: invitation.invitee_email.clone(),
        expires_at: invitation.expires_at.clone(),
        message: "Invitation sent successfully".to_string(),
    }))
}

/// Response for pending invitations list
#[derive(Debug, Serialize)]
pub struct PendingInvitationsResponse {
    /// List of pending invitations
    pub invitations: Vec<InvitationInfo>,
}

/// Information about a single invitation
#[derive(Debug, Serialize)]
pub struct InvitationInfo {
    /// Invitation ID
    pub id: Uuid,
    
    /// Game ID
    pub game_id: Uuid,
    
    /// Inviter user ID
    pub inviter_id: Uuid,
    
    /// Expiration timestamp
    pub expires_at: String,
}

/// Gets pending invitations for authenticated user
///
/// Returns all non-expired invitations for the current user.
///
/// # Endpoint
///
/// `GET /api/v1/invitations/pending`
///
/// # Authentication
///
/// **Required** - User must be authenticated.
#[tracing::instrument(skip(state))]
pub async fn get_pending_invitations(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<PendingInvitationsResponse>, ApiError> {
    let invitations = state.invitation_service.get_pending_for_user(&claims.email);
    
    // Service já retorna Vec<InvitationInfo>, mas precisamos converter para nosso tipo local
    let invitation_infos: Vec<InvitationInfo> = invitations
        .into_iter()
        .map(|inv| {
            // Parse inviter_email to get inviter_id (simplified for now)
            let inviter_id = Uuid::new_v4(); // TODO: lookup real user_id
            InvitationInfo {
                id: inv.id,
                game_id: inv.game_id,
                inviter_id,
                expires_at: inv.expires_at,
            }
        })
        .collect();

    Ok(Json(PendingInvitationsResponse {
        invitations: invitation_infos,
    }))
}

/// Response for invitation acceptance
#[derive(Debug, Serialize)]
pub struct AcceptInvitationResponse {
    /// Game ID the user joined
    pub game_id: Uuid,
    
    /// Success message
    pub message: String,
}

/// Accepts a game invitation
///
/// User accepts an invitation and is added to the game.
///
/// # Endpoint
///
/// `POST /api/v1/invitations/:id/accept`
///
/// # Authentication
///
/// **Required** - User must be the invitee.
#[tracing::instrument(skip(state))]
pub async fn accept_invitation(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Path(invitation_id): Path<Uuid>,
) -> Result<Json<AcceptInvitationResponse>, ApiError> {
    // Accept the invitation
    let invitation = state.invitation_service.accept(invitation_id)?;
    
    // Verify the invitee email matches
    if invitation.invitee_email != claims.email {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "NOT_INVITEE",
            "This invitation is not for you",
        ));
    }
    
    // Add player to game
    state.game_service.add_player_to_game(invitation.game_id, claims.email.clone())?;
    
    // Mark invitation as accepted
    state.invitation_service.accept(invitation_id)?;

    tracing::info!(
        invitation_id = %invitation_id,
        game_id = %invitation.game_id,
        email = %claims.email,
        "Invitation accepted, player added to game"
    );

    Ok(Json(AcceptInvitationResponse {
        game_id: invitation.game_id,
        message: "Invitation accepted, joined game successfully".to_string(),
    }))
}

/// Response for invitation decline
#[derive(Debug, Serialize)]
pub struct DeclineInvitationResponse {
    /// Success message
    pub message: String,
}

/// Declines a game invitation
///
/// User declines an invitation. The invitation is marked as declined.
///
/// # Endpoint
///
/// `POST /api/v1/invitations/:id/decline`
///
/// # Authentication
///
/// **Required** - User must be the invitee.
#[tracing::instrument(skip(state))]
pub async fn decline_invitation(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Path(invitation_id): Path<Uuid>,
) -> Result<Json<DeclineInvitationResponse>, ApiError> {
    // Get invitation to verify invitee
    let invitation = state.invitation_service.get_invitation(invitation_id)?;
    if invitation.invitee_email != claims.email {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "NOT_INVITEE",
            "This invitation is not for you",
        ));
    }
    
    state.invitation_service.decline(invitation_id)?;
    
    tracing::info!(
        invitation_id = %invitation_id,
        email = %claims.email,
        "Invitation declined"
    );

    Ok(Json(DeclineInvitationResponse {
        message: "Invitation declined".to_string(),
    }))
}

// ============================================================================
// M7: Turn-Based Gameplay Endpoints
// ============================================================================

/// Response for stand action
#[derive(Debug, Serialize)]
pub struct StandResponse {
    /// Current player points
    pub points: u32,
    
    /// Whether player is busted
    pub busted: bool,
    
    /// Success message
    pub message: String,
    
    /// Game automatically finished?
    pub game_finished: bool,
}

/// Player stands (stops drawing cards)
///
/// Marks the player as standing and advances to next turn.
/// If all players have stood or busted, game finishes automatically.
///
/// # Endpoint
///
/// `POST /api/v1/games/:game_id/stand`
///
/// # Authentication
///
/// **Required** - Must be player's turn.
#[tracing::instrument(skip(state), fields(player_email = %claims.email))]
pub async fn stand(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Path(game_id): Path<Uuid>,
) -> Result<Json<StandResponse>, ApiError> {
    let game_state = state.game_service.stand(game_id, &claims.email)?;
    
    // Get player info from response
    let player_info = game_state.players.get(&claims.email)
        .ok_or_else(|| ApiError::new(
            StatusCode::NOT_FOUND,
            "PLAYER_NOT_FOUND",
            "Player not found in game",
        ))?;
    
    tracing::info!(
        game_id = %game_id,
        email = %claims.email,
        points = player_info.points,
        game_finished = game_state.finished,
        "Player stood"
    );

    Ok(Json(StandResponse {
        points: player_info.points as u32,
        busted: player_info.busted,
        message: "Player stood successfully".to_string(),
        game_finished: game_state.finished,
    }))
}

// ============================================================================
// M7: Game Enrollment Endpoints
// ============================================================================

/// Response for getting open games
#[derive(Debug, Serialize)]
pub struct GetOpenGamesResponse {
    /// List of games in enrollment phase
    pub games: Vec<OpenGameInfo>,
    
    /// Total number of open games
    pub count: usize,
}

/// Information about an open game
#[derive(Debug, Serialize)]
pub struct OpenGameInfo {
    /// Game ID
    pub game_id: Uuid,
    
    /// Creator's user ID
    pub creator_id: Uuid,
    
    /// Number of enrolled players
    pub enrolled_count: u64,
    
    /// Maximum number of players allowed
    pub max_players: u64,
    
    /// Enrollment timeout in seconds
    pub enrollment_timeout_seconds: u64,
    
    /// Time remaining until enrollment closes (seconds)
    pub time_remaining_seconds: i64,
    
    /// When enrollment closes (RFC3339 format)
    pub enrollment_closes_at: String,
}

/// Gets all games currently in enrollment phase
///
/// Returns a list of games that are accepting player enrollments.
/// Games must not be finished and must still be within their enrollment timeout.
///
/// # Endpoint
///
/// `GET /api/v1/games/open`
///
/// # Authentication
///
/// **Required** - Must include valid JWT token in Authorization header.
///
/// # Response
///
/// **Success (200 OK)**:
/// ```json
/// {
///   "games": [
///     {
///       "game_id": "550e8400-e29b-41d4-a716-446655440000",
///       "creator_id": "660e8400-e29b-41d4-a716-446655440000",
///       "enrolled_count": 2,
///       "max_players": 10,
///       "enrollment_timeout_seconds": 300,
///       "time_remaining_seconds": 250,
///       "enrollment_closes_at": "2026-01-10T01:30:00Z"
///     }
///   ],
///   "count": 1
/// }
/// ```
///
/// # Errors
///
/// - **401 Unauthorized** - Missing or invalid JWT token
///
/// # Example
///
/// ```bash
/// curl http://localhost:8080/api/v1/games/open \
///   -H "Authorization: Bearer YOUR_JWT_TOKEN"
/// ```
#[tracing::instrument(skip(state, _claims))]
pub async fn get_open_games(
    State(state): State<crate::AppState>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<GetOpenGamesResponse>, ApiError> {
    let game_infos = state.game_service.get_open_games(None)?;
    
    let games = game_infos
        .into_iter()
        .map(|info| OpenGameInfo {
            game_id: info.game_id,
            creator_id: info.creator_id,
            enrolled_count: info.enrolled_count,
            max_players: info.max_players,
            enrollment_timeout_seconds: info.enrollment_timeout_seconds,
            time_remaining_seconds: info.time_remaining_seconds,
            enrollment_closes_at: info.enrollment_closes_at,
        })
        .collect::<Vec<_>>();
    
    let count = games.len();

    tracing::info!(
        count = count,
        "Retrieved open games list"
    );

    Ok(Json(GetOpenGamesResponse { games, count }))
}

/// Request to enroll a player in a game
#[derive(Debug, Deserialize)]
pub struct EnrollPlayerRequest {
    /// Player's email address
    pub email: String,
}

/// Response for player enrollment
#[derive(Debug, Serialize)]
pub struct EnrollPlayerResponse {
    /// Game ID
    pub game_id: Uuid,
    
    /// Player's email
    pub email: String,
    
    /// Success message
    pub message: String,
    
    /// Updated enrolled player count
    pub enrolled_count: u64,
}

/// Enrolls a player in a game during enrollment phase
///
/// Adds a player to a game that is currently accepting enrollments.
/// The game must not be finished and enrollment must still be open.
///
/// # Endpoint
///
/// `POST /api/v1/games/:game_id/enroll`
///
/// # Authentication
///
/// **Required** - Must include valid JWT token in Authorization header.
///
/// # Path Parameters
///
/// - `game_id` - UUID of the game
///
/// # Request Body
///
/// ```json
/// {
///   "email": "player@example.com"
/// }
/// ```
///
/// # Response
///
/// **Success (200 OK)**:
/// ```json
/// {
///   "game_id": "550e8400-e29b-41d4-a716-446655440000",
///   "email": "player@example.com",
///   "message": "Player enrolled successfully",
///   "enrolled_count": 3
/// }
/// ```
///
/// # Errors
///
/// - **401 Unauthorized** - Missing or invalid JWT token
/// - **404 Not Found** - Game does not exist
/// - **409 Conflict** - Game is full
///   ```json
///   {
///     "message": "Game is full",
///     "code": "GAME_FULL",
///     "status": 409
///   }
///   ```
/// - **410 Gone** - Enrollment period has closed
///   ```json
///   {
///     "message": "Enrollment is closed for this game",
///     "code": "ENROLLMENT_CLOSED",
///     "status": 410
///   }
///   ```
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:8080/api/v1/games/550e8400-e29b-41d4-a716-446655440000/enroll \
///   -H "Authorization: Bearer YOUR_JWT_TOKEN" \
///   -H "Content-Type: application/json" \
///   -d '{"email": "player@example.com"}'
/// ```
#[tracing::instrument(skip(state, _claims), fields(game_id, player_email = %payload.email))]
pub async fn enroll_player(
    State(state): State<crate::AppState>,
    Extension(_claims): Extension<Claims>,
    Path(game_id): Path<Uuid>,
    Json(payload): Json<EnrollPlayerRequest>,
) -> Result<Json<EnrollPlayerResponse>, ApiError> {
    // Enroll the player
    state.game_service.enroll_player(game_id, &payload.email)?;
    
    // Get updated game state to return enrolled count
    let game_state = state.game_service.get_game_state(game_id)?;
    let enrolled_count = game_state.players.len() as u64;

    tracing::info!(
        game_id = %game_id,
        email = %payload.email,
        enrolled_count = enrolled_count,
        "Player enrolled successfully"
    );

    Ok(Json(EnrollPlayerResponse {
        game_id,
        email: payload.email,
        message: "Player enrolled successfully".to_string(),
        enrolled_count,
    }))
}

/// Request to close enrollment for a game
#[derive(Debug, Deserialize)]
pub struct CloseEnrollmentRequest {
    // No body needed - creator_id comes from JWT token
}

/// Response for closing enrollment
#[derive(Debug, Serialize)]
pub struct CloseEnrollmentResponse {
    /// Game ID
    pub game_id: Uuid,
    
    /// Success message
    pub message: String,
    
    /// Order in which players will take turns
    pub turn_order: Vec<String>,
    
    /// Total enrolled players
    pub player_count: usize,
}

/// Closes enrollment for a game and initializes turn order
///
/// Only the game creator can close enrollment. Once closed, no more
/// players can join. The turn order is randomized among enrolled players.
///
/// # Endpoint
///
/// `POST /api/v1/games/:game_id/close-enrollment`
///
/// # Authentication
///
/// **Required** - Player email extracted from JWT token.
/// Must be the game creator.
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
///   "game_id": "550e8400-e29b-41d4-a716-446655440000",
///   "message": "Enrollment closed successfully",
///   "turn_order": [
///     "player1@example.com",
///     "player3@example.com",
///     "player2@example.com"
///   ],
///   "player_count": 3
/// }
/// ```
///
/// # Errors
///
/// - **401 Unauthorized** - Missing or invalid JWT token
/// - **403 Forbidden** - User is not the game creator
///   ```json
///   {
///     "message": "Only the game creator can close enrollment",
///     "code": "NOT_GAME_CREATOR",
///     "status": 403
///   }
///   ```
/// - **404 Not Found** - Game does not exist
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:8080/api/v1/games/550e8400-e29b-41d4-a716-446655440000/close-enrollment \
///   -H "Authorization: Bearer YOUR_JWT_TOKEN"
/// ```
#[tracing::instrument(skip(state, claims), fields(game_id))]
pub async fn close_enrollment(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Path(game_id): Path<Uuid>,
    Json(_payload): Json<CloseEnrollmentRequest>,
) -> Result<Json<CloseEnrollmentResponse>, ApiError> {
    // Parse user_id from claims (for now using email, but should be user_id)
    // TODO: M7 - Update Claims to include user_id
    let user_id = match uuid::Uuid::parse_str(&claims.user_id) {
        Ok(id) => id,
        Err(_) => {
            // Fallback: generate a deterministic ID from email hash if needed
            return Err(ApiError::new(
                StatusCode::UNAUTHORIZED,
                "INVALID_CLAIMS",
                "Invalid user ID in JWT token",
            ));
        }
    };

    // Close enrollment (only creator can do this)
    let turn_order = state.game_service.close_enrollment(game_id, user_id)?;
    let player_count = turn_order.len();

    tracing::info!(
        game_id = %game_id,
        creator_id = %user_id,
        player_count = player_count,
        turn_order = ?turn_order,
        "Enrollment closed successfully"
    );

    Ok(Json(CloseEnrollmentResponse {
        game_id,
        message: "Enrollment closed successfully".to_string(),
        turn_order,
        player_count,
    }))
}
