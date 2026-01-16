use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use std::collections::HashMap;

/// Standardized API error response
///
/// This structure provides a consistent error format across all API endpoints,
/// making it easier for clients to parse and handle errors uniformly.
///
/// # JSON Response Format
///
/// ```json
/// {
///   "message": "Human-readable error description",
///   "code": "ERROR_CODE_CONSTANT",
///   "status": 400,
///   "details": {
///     "field1": "value1",
///     "field2": "value2"
///   }
/// }
/// ```
///
/// # Example
///
/// ```
/// use blackjack_api::error::ApiError;
/// use axum::http::StatusCode;
///
/// let error = ApiError::new(
///     StatusCode::BAD_REQUEST,
///     "INVALID_INPUT",
///     "Email cannot be empty"
/// );
/// ```
#[derive(Debug, Serialize)]
pub struct ApiError {
    /// Human-readable error message describing what went wrong
    pub message: String,
    /// Machine-readable error code (e.g., "UNAUTHORIZED", "RATE_LIMIT_EXCEEDED")
    ///
    /// These codes are stable and can be used by clients for error handling logic
    pub code: String,
    /// HTTP status code as a number (e.g., 400, 401, 404, 429, 500)
    pub status: u16,
    /// Optional additional context about the error
    ///
    /// Useful for validation errors or providing debugging information.
    /// This field is omitted from JSON if None.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, String>>,
}

impl ApiError {
    /// Creates a new API error with the specified status, code, and message
    ///
    /// # Arguments
    ///
    /// * `status` - HTTP status code (e.g., StatusCode::BAD_REQUEST)
    /// * `code` - Machine-readable error code
    /// * `message` - Human-readable error description
    ///
    /// # Example
    ///
    /// ```
    /// use blackjack_api::error::ApiError;
    /// use axum::http::StatusCode;
    ///
    /// let error = ApiError::new(
    ///     StatusCode::NOT_FOUND,
    ///     "GAME_NOT_FOUND",
    ///     "The requested game does not exist"
    /// );
    /// ```
    pub fn new(status: StatusCode, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: code.into(),
            status: status.as_u16(),
            details: None,
        }
    }

    /// Adds additional details to the error
    ///
    /// This is useful for providing structured context, such as validation errors
    /// with field-specific messages or constraints that were violated.
    ///
    /// # Arguments
    ///
    /// * `details` - HashMap of key-value pairs with additional error context
    ///
    /// # Example
    ///
    /// ```
    /// use blackjack_api::error::ApiError;
    /// use axum::http::StatusCode;
    /// use std::collections::HashMap;
    ///
    /// let mut details = HashMap::new();
    /// details.insert("min".to_string(), "1".to_string());
    /// details.insert("max".to_string(), "10".to_string());
    ///
    /// let error = ApiError::new(
    ///     StatusCode::BAD_REQUEST,
    ///     "INVALID_RANGE",
    ///     "Value out of range"
    /// ).with_details(details);
    /// ```
    pub fn with_details(mut self, details: HashMap<String, String>) -> Self {
        self.details = Some(details);
        self
    }

    /// Creates a 401 Unauthorized error
    ///
    /// Used when authentication is required but not provided or invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use blackjack_api::error::ApiError;
    ///
    /// let error = ApiError::unauthorized();
    /// assert_eq!(error.status, 401);
    /// assert_eq!(error.code, "UNAUTHORIZED");
    /// ```
    pub fn unauthorized() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "Authentication required",
        )
    }

    /// Creates a 429 Too Many Requests error
    ///
    /// Used when a client exceeds the configured rate limit.
    ///
    /// # Example
    ///
    /// ```
    /// use blackjack_api::error::ApiError;
    ///
    /// let error = ApiError::rate_limit_exceeded();
    /// assert_eq!(error.status, 429);
    /// assert_eq!(error.code, "RATE_LIMIT_EXCEEDED");
    /// ```
    pub fn rate_limit_exceeded() -> Self {
        Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMIT_EXCEEDED",
            "Rate limit exceeded. Please try again later.",
        )
    }

    /// Creates a 404 Not Found error for missing games
    ///
    /// # Example
    ///
    /// ```
    /// use blackjack_api::error::ApiError;
    ///
    /// let error = ApiError::game_not_found();
    /// assert_eq!(error.status, 404);
    /// assert_eq!(error.code, "GAME_NOT_FOUND");
    /// ```
    pub fn game_not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, "GAME_NOT_FOUND", "Game not found")
    }

    /// Creates a 400 Bad Request error for invalid player count
    ///
    /// Includes details about the valid range and the value that was provided.
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum allowed number of players
    /// * `max` - Maximum allowed number of players
    /// * `provided` - Number of players that was provided
    ///
    /// # Example
    ///
    /// ```
    /// use blackjack_api::error::ApiError;
    ///
    /// let error = ApiError::invalid_player_count(1, 10, 15);
    /// assert_eq!(error.status, 400);
    /// assert_eq!(error.code, "INVALID_PLAYER_COUNT");
    /// assert!(error.details.is_some());
    /// ```
    pub fn invalid_player_count(min: u8, max: u8, provided: usize) -> Self {
        let mut details = HashMap::new();
        details.insert("min".to_string(), min.to_string());
        details.insert("max".to_string(), max.to_string());
        details.insert("provided".to_string(), provided.to_string());

        Self::new(
            StatusCode::BAD_REQUEST,
            "INVALID_PLAYER_COUNT",
            format!("Player count must be between {} and {}", min, max),
        )
        .with_details(details)
    }
}

/// Converts ApiError into an Axum HTTP response
///
/// This implementation allows ApiError to be returned directly from Axum handlers.
/// The error is automatically serialized to JSON and sent with the appropriate
/// HTTP status code.
///
/// # Example
///
/// ```no_run
/// use axum::Json;
/// use blackjack_api::error::ApiError;
///
/// async fn handler() -> Result<Json<String>, ApiError> {
///     Err(ApiError::unauthorized())
/// }
/// ```
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

/// Converts service-layer GameError into API-layer ApiError
///
/// This implementation provides a bridge between the business logic layer
/// (blackjack-service) and the API layer, translating internal errors into
/// appropriate HTTP responses.
///
/// # Error Mapping
///
/// - `GameNotFound` → 404 GAME_NOT_FOUND
/// - `PlayerNotInGame` → 403 PLAYER_NOT_IN_GAME
/// - `PlayerAlreadyBusted` → 400 PLAYER_BUSTED
/// - `InvalidPlayerCount` → 400 INVALID_PLAYER_COUNT (with details)
/// - `InvalidEmail` → 400 INVALID_EMAIL
/// - `DeckEmpty` → 400 DECK_EMPTY
/// - `GameAlreadyFinished` → 400 GAME_FINISHED
/// - `CoreError` → 500 INTERNAL_ERROR
///
/// # Example
///
/// ```no_run
/// use blackjack_api::error::ApiError;
/// use blackjack_service::GameError;
///
/// fn handle_service_error(err: GameError) -> ApiError {
///     err.into()
/// }
/// ```
impl From<blackjack_service::GameError> for ApiError {
    fn from(err: blackjack_service::GameError) -> Self {
        use blackjack_service::GameError;

        match err {
            GameError::GameNotFound => Self::game_not_found(),
            GameError::PlayerNotInGame => {
                Self::new(StatusCode::FORBIDDEN, "PLAYER_NOT_IN_GAME", err.to_string())
            }
            GameError::PlayerAlreadyBusted => {
                Self::new(StatusCode::BAD_REQUEST, "PLAYER_BUSTED", err.to_string())
            }
            GameError::InvalidPlayerCount { min, max, provided } => {
                Self::invalid_player_count(min, max, provided)
            }
            GameError::GameFull => Self::new(
                StatusCode::BAD_REQUEST,
                "GAME_FULL",
                "Game is at maximum capacity (10 players)",
            ),
            GameError::EnrollmentClosed => Self::new(
                StatusCode::GONE,
                "ENROLLMENT_CLOSED",
                "Enrollment for this game is closed",
            ),
            GameError::InvalidEmail(msg) => {
                Self::new(StatusCode::BAD_REQUEST, "INVALID_EMAIL", msg)
            }
            GameError::DeckEmpty => Self::new(
                StatusCode::BAD_REQUEST,
                "DECK_EMPTY",
                "No more cards in deck",
            ),
            GameError::GameAlreadyFinished => Self::new(
                StatusCode::CONFLICT,
                "GAME_FINISHED",
                "Game has already finished",
            ),
            GameError::UserNotFound => {
                Self::new(StatusCode::NOT_FOUND, "USER_NOT_FOUND", "User not found")
            }
            GameError::UserAlreadyExists => {
                Self::new(StatusCode::CONFLICT, "USER_EXISTS", "User already exists")
            }
            GameError::InvalidCredentials => Self::unauthorized(),
            GameError::InvitationNotFound => Self::new(
                StatusCode::NOT_FOUND,
                "INVITATION_NOT_FOUND",
                "Invitation not found",
            ),
            GameError::InvitationExpired => Self::new(
                StatusCode::GONE,
                "INVITATION_EXPIRED",
                "Invitation has expired",
            ),
            GameError::InvalidTimeout { max } => Self::new(
                StatusCode::BAD_REQUEST,
                "INVALID_TIMEOUT",
                format!("Timeout exceeds maximum of {} seconds", max),
            ),
            GameError::NotPlayerTurn => {
                Self::new(StatusCode::FORBIDDEN, "NOT_YOUR_TURN", "It's not your turn")
            }
            GameError::PlayerNotActive => Self::new(
                StatusCode::FORBIDDEN,
                "PLAYER_NOT_ACTIVE",
                "Player is not active",
            ),
            GameError::NotGameCreator => Self::new(
                StatusCode::FORBIDDEN,
                "NOT_GAME_CREATOR",
                "Only the game creator can perform this action",
            ),
            GameError::EnrollmentNotClosed => Self::new(
                StatusCode::CONFLICT,
                "ENROLLMENT_NOT_CLOSED",
                "Cannot play until enrollment is closed",
            ),
            GameError::PlayerAlreadyEnrolled => Self::new(
                StatusCode::CONFLICT,
                "PLAYER_ALREADY_ENROLLED",
                "Player is already enrolled in this game",
            ),
            GameError::GameNotActive => Self::new(
                StatusCode::GONE,
                "GAME_NOT_ACTIVE",
                "Game is not active or has been deleted",
            ),
            GameError::WeakPassword(msg) => Self::new(
                StatusCode::BAD_REQUEST,
                "WEAK_PASSWORD",
                msg,
            ),
            GameError::AccountInactive => Self::new(
                StatusCode::FORBIDDEN,
                "ACCOUNT_INACTIVE",
                "Account is inactive or suspended",
            ),
            GameError::InsufficientPermissions => Self::new(
                StatusCode::FORBIDDEN,
                "INSUFFICIENT_PERMISSIONS",
                "You don't have permission to perform this action",
            ),
            GameError::AccountLocked => Self::new(
                StatusCode::FORBIDDEN,
                "ACCOUNT_LOCKED",
                "Account is locked due to too many failed login attempts",
            ),
            GameError::ValidationError(msg) => Self::new(
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                msg,
            ),
            GameError::PasswordHashError(_) => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "PASSWORD_HASH_ERROR",
                "Failed to hash password",
            ),
            GameError::CoreError(core_err) => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                core_err.to_string(),
            ),
        }
    }
}
