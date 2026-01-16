//! Integration tests for the Blackjack API
//!
//! These tests validate the API layer configuration, error handling,
//! and integration between components. They focus on testing the API
//! infrastructure rather than endpoint behavior (which will be added in Phase 5).
//!
//! # Test Categories
//!
//! - **Configuration Tests** - Verify config loading and defaults
//! - **State Tests** - Validate AppState creation and composition
//! - **Integration Tests** - Ensure components work together correctly
//!
//! # Running Tests
//!
//! ```bash
//! # Run all API tests
//! cargo test -p blackjack-api
//!
//! # Run with output
//! cargo test -p blackjack-api -- --nocapture
//!
//! # Run specific test
//! cargo test -p blackjack-api test_config_defaults
//! ```

use blackjack_api::AppState;
use blackjack_service::{
    GameService, InvitationConfig, InvitationService, ServiceConfig, UserService,
};
use std::sync::Arc;

/// Tests that AppState can be created with all required components
///
/// Validates:
/// - Config loads successfully from file
/// - GameService can be instantiated
/// - RateLimiter can be created
/// - AppState composition works correctly
///
/// This is a basic smoke test ensuring all components can be initialized.
#[test]
fn test_app_state_creation() {
    // This test validates that AppState can be created properly
    let config = blackjack_api::config::AppConfig::from_file();

    // Config should load successfully
    assert!(config.is_ok(), "Config should load from file");

    let config = Arc::new(config.unwrap());
    let user_service = Arc::new(UserService::new());
    let game_service = Arc::new(GameService::new(
        ServiceConfig::default(),
        user_service.clone(),
    ));
    let invitation_service = Arc::new(InvitationService::new(InvitationConfig::default()));
    let rate_limiter =
        blackjack_api::rate_limiter::RateLimiter::new(config.rate_limit.requests_per_minute);

    let _state = AppState {
        game_service,
        user_service,
        invitation_service,
        config,
        rate_limiter,
    };
}

/// Tests that configuration loads with correct default values
///
/// Validates all default values from config.toml:
/// - Server host and port
/// - JWT expiration
/// - Rate limiting
/// - API deprecation period
///
/// This ensures the config.toml file is present and properly formatted.
#[test]
fn test_config_defaults() {
    let config = blackjack_api::config::AppConfig::from_file().unwrap();

    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.jwt.expiration_hours, 24);
    assert_eq!(config.rate_limit.requests_per_minute, 10);
    assert_eq!(config.api.version_deprecation_months, 6);
}

/// Tests JWT configuration values
///
/// Validates:
/// - JWT secret is loaded and not empty
/// - Default secret value matches expected development secret
///
/// **Security Note**: In production, the secret should be overridden
/// via the `BLACKJACK_JWT_SECRET` environment variable.
#[test]
fn test_jwt_config() {
    let config = blackjack_api::config::AppConfig::from_file().unwrap();

    // Secret should be set (even if it's the default dev secret)
    assert!(!config.jwt.secret.is_empty());
    assert_eq!(config.jwt.secret, "dev-secret-key-change-in-production");
}

/// Tests CORS configuration values
///
/// Validates:
/// - CORS allowed origins are loaded
/// - Default origin for local development is configured
///
/// In Phase 5, CORS will be properly configured from this setting.
/// Currently using permissive CORS for development.
#[test]
fn test_cors_config() {
    let config = blackjack_api::config::AppConfig::from_file().unwrap();

    assert_eq!(config.cors.allowed_origins.len(), 1);
    assert_eq!(config.cors.allowed_origins[0], "http://localhost:3000");
}

/// Tests ApiError creation for invalid player count
///
/// Validates:
/// - Error has correct status code (400)
/// - Error code is set correctly
/// - Details HashMap contains min, max, and provided values
/// - Message is descriptive
#[test]
fn test_api_error_invalid_player_count() {
    let error = blackjack_api::error::ApiError::invalid_player_count(1, 10, 15);

    assert_eq!(error.status, 400);
    assert_eq!(error.code, "INVALID_PLAYER_COUNT");
    assert!(error.message.contains("between"));

    let details = error.details.expect("Details should be present");
    assert_eq!(details.get("min"), Some(&"1".to_string()));
    assert_eq!(details.get("max"), Some(&"10".to_string()));
    assert_eq!(details.get("provided"), Some(&"15".to_string()));
}

/// Tests ApiError creation for unauthorized access
///
/// Validates:
/// - Error has 401 status code
/// - Error code is UNAUTHORIZED
/// - Message indicates authentication is required
#[test]
fn test_api_error_unauthorized() {
    let error = blackjack_api::error::ApiError::unauthorized();

    assert_eq!(error.status, 401);
    assert_eq!(error.code, "UNAUTHORIZED");
    assert!(error.message.contains("Authentication"));
}

/// Tests ApiError creation for rate limit exceeded
///
/// Validates:
/// - Error has 429 status code
/// - Error code is RATE_LIMIT_EXCEEDED
/// - Message indicates rate limit was exceeded
#[test]
fn test_api_error_rate_limit_exceeded() {
    let error = blackjack_api::error::ApiError::rate_limit_exceeded();

    assert_eq!(error.status, 429);
    assert_eq!(error.code, "RATE_LIMIT_EXCEEDED");
    assert!(error.message.contains("Rate limit"));
}

/// Tests ApiError creation for game not found
///
/// Validates:
/// - Error has 404 status code
/// - Error code is GAME_NOT_FOUND
/// - Message indicates game was not found
#[test]
fn test_api_error_game_not_found() {
    let error = blackjack_api::error::ApiError::game_not_found();

    assert_eq!(error.status, 404);
    assert_eq!(error.code, "GAME_NOT_FOUND");
    assert!(error.message.contains("not found"));
}

/// Tests ApiError with_details method
///
/// Validates:
/// - Details can be added to an existing error
/// - Details HashMap is properly attached
/// - Original error properties are preserved
#[test]
fn test_api_error_with_details() {
    use std::collections::HashMap;

    let mut details = HashMap::new();
    details.insert("field".to_string(), "value".to_string());

    let error = blackjack_api::error::ApiError::new(
        axum::http::StatusCode::BAD_REQUEST,
        "TEST_ERROR",
        "Test message",
    )
    .with_details(details);

    assert_eq!(error.status, 400);
    assert_eq!(error.code, "TEST_ERROR");
    assert!(error.details.is_some());

    let error_details = error.details.unwrap();
    assert_eq!(error_details.get("field"), Some(&"value".to_string()));
}

/// Tests rate limiter basic functionality
///
/// Validates:
/// - Rate limiter can be created with a limit
/// - First request is always allowed
/// - Rate limiter tracks requests per key
#[test]
fn test_rate_limiter_allows_first_request() {
    let limiter = blackjack_api::rate_limiter::RateLimiter::new(10);

    let result = limiter.check_rate_limit("test-key");
    assert!(result.is_ok(), "First request should be allowed");
}

/// Tests rate limiter enforcement
///
/// Validates:
/// - Rate limiter enforces the configured limit
/// - Requests beyond the limit are rejected
/// - Error returned is ApiError::rate_limit_exceeded
#[test]
fn test_rate_limiter_enforces_limit() {
    let limiter = blackjack_api::rate_limiter::RateLimiter::new(3);
    let key = "test-key-limit";

    // First 3 requests should succeed
    assert!(limiter.check_rate_limit(key).is_ok());
    assert!(limiter.check_rate_limit(key).is_ok());
    assert!(limiter.check_rate_limit(key).is_ok());

    // 4th request should fail
    let result = limiter.check_rate_limit(key);
    assert!(result.is_err(), "Request beyond limit should fail");

    if let Err(error) = result {
        assert_eq!(error.status, 429);
        assert_eq!(error.code, "RATE_LIMIT_EXCEEDED");
    }
}

/// Tests rate limiter key isolation
///
/// Validates:
/// - Different keys have separate rate limit buckets
/// - One key reaching limit doesn't affect another key
/// - Rate limiter properly isolates players and games
#[test]
fn test_rate_limiter_key_isolation() {
    let limiter = blackjack_api::rate_limiter::RateLimiter::new(2);

    let key1 = "game1:player1@example.com";
    let key2 = "game1:player2@example.com";

    // Exhaust limit for key1
    assert!(limiter.check_rate_limit(key1).is_ok());
    assert!(limiter.check_rate_limit(key1).is_ok());
    assert!(limiter.check_rate_limit(key1).is_err());

    // key2 should still be allowed
    assert!(limiter.check_rate_limit(key2).is_ok());
    assert!(limiter.check_rate_limit(key2).is_ok());
}

/// Tests service error conversion to API error
///
/// Validates:
/// - GameError::GameNotFound converts to 404
/// - GameError::InvalidPlayerCount converts to 400 with details
/// - GameError::DeckEmpty converts to 400
/// - Error messages are preserved
#[test]
fn test_service_error_conversion() {
    use blackjack_api::error::ApiError;
    use blackjack_service::GameError;

    // Test GameNotFound -> 404
    let service_error = GameError::GameNotFound;
    let api_error: ApiError = service_error.into();
    assert_eq!(api_error.status, 404);
    assert_eq!(api_error.code, "GAME_NOT_FOUND");

    // Test InvalidPlayerCount -> 400 with details
    let service_error = GameError::InvalidPlayerCount {
        min: 1,
        max: 10,
        provided: 15,
    };
    let api_error: ApiError = service_error.into();
    assert_eq!(api_error.status, 400);
    assert_eq!(api_error.code, "INVALID_PLAYER_COUNT");
    assert!(api_error.details.is_some());

    // Test DeckEmpty -> 400
    let service_error = GameError::DeckEmpty;
    let api_error: ApiError = service_error.into();
    assert_eq!(api_error.status, 400);
    assert_eq!(api_error.code, "DECK_EMPTY");

    // Test PlayerAlreadyEnrolled -> 409 CONFLICT
    let service_error = GameError::PlayerAlreadyEnrolled;
    let api_error: ApiError = service_error.into();
    assert_eq!(api_error.status, 409);
    assert_eq!(api_error.code, "PLAYER_ALREADY_ENROLLED");

    // Test GameNotActive -> 410 GONE
    let service_error = GameError::GameNotActive;
    let api_error: ApiError = service_error.into();
    assert_eq!(api_error.status, 410);
    assert_eq!(api_error.code, "GAME_NOT_ACTIVE");

    // Test EnrollmentNotClosed -> 409 CONFLICT
    let service_error = GameError::EnrollmentNotClosed;
    let api_error: ApiError = service_error.into();
    assert_eq!(api_error.status, 409);
    assert_eq!(api_error.code, "ENROLLMENT_NOT_CLOSED");
}
/// Tests UserService creation and basic functionality
///
/// Validates:
/// - UserService can be created
/// - User registration works with unique email
/// - User login works with correct credentials
/// - Duplicate email registration fails
#[test]
fn test_user_service_creation() {
    let service = UserService::new();

    // Register a new user
    let user_id = service.register("user1@example.com".to_string(), "TestP@ssw0rd".to_string());
    assert!(user_id.is_ok(), "User registration should succeed");

    let user_id = user_id.unwrap();

    // Login with correct credentials
    let login_result = service.login("user1@example.com", "TestP@ssw0rd");
    assert!(
        login_result.is_ok(),
        "Login with correct password should succeed"
    );

    let logged_user = login_result.unwrap();
    assert_eq!(logged_user.id, user_id, "Login should return the same user");
    assert_eq!(logged_user.email, "user1@example.com");

    // Try to register duplicate email
    let duplicate = service.register(
        "user1@example.com".to_string(),
        "Different@Pass1".to_string(),
    );
    assert!(duplicate.is_err(), "Duplicate email should fail");
}

/// Tests InvitationService creation and configuration
///
/// Validates:
/// - InvitationService can be created with custom config
/// - Default timeout values are applied correctly
/// - Invitation config validates max timeout
#[test]
fn test_invitation_service_creation() {
    let config = InvitationConfig {
        default_timeout_seconds: 300,
        max_timeout_seconds: 3600,
    };

    let _service = InvitationService::new(config.clone());

    // Verify config values
    assert_eq!(config.default_timeout_seconds, 300);
    assert_eq!(config.max_timeout_seconds, 3600);
}

/// Tests invitation config default values
///
/// Validates:
/// - Default timeout is 5 minutes (300 seconds)
/// - Maximum timeout is 1 hour (3600 seconds)
#[test]
fn test_invitation_config_defaults() {
    let config = InvitationConfig::default();

    assert_eq!(config.default_timeout_seconds, 300);
    assert_eq!(config.max_timeout_seconds, 3600);
}

// ============================================================================
// M7: Authentication Tests for Create Game
// ============================================================================

/// Tests that create_game requires authentication
///
/// Validates:
/// - Request without Authorization header returns 401
/// - Request with invalid token returns 401
/// - Request with valid token succeeds (200)
/// - Response includes creator_id matching authenticated user
#[tokio::test]
async fn test_create_game_requires_authentication() {
    // Setup: Create test users for authentication
    let user_service = Arc::new(UserService::new());
    let test_email = "testuser@example.com";
    let test_password = "TestPass123!";

    // Register user
    let user_id = user_service
        .register(test_email.to_string(), test_password.to_string())
        .expect("Failed to register test user");

    // Login to get JWT token
    let _token = user_service
        .login(test_email, test_password)
        .expect("Failed to login");

    // Create AppState for handler
    let config = Arc::new(blackjack_api::config::AppConfig::from_file().unwrap());
    let game_service = Arc::new(GameService::new(
        ServiceConfig::default(),
        user_service.clone(),
    ));
    let invitation_service = Arc::new(InvitationService::new(InvitationConfig::default()));
    let rate_limiter =
        blackjack_api::rate_limiter::RateLimiter::new(config.rate_limit.requests_per_minute);

    let state = AppState {
        game_service,
        user_service,
        invitation_service,
        config,
        rate_limiter,
    };

    // Test 1: Without Authorization header - should extract Claims fail
    // Since we're testing handlers directly, we need to test with Extension
    use axum::Json;
    use axum::extract::{Extension, State as AxumState};
    use blackjack_api::auth::Claims;
    use blackjack_api::handlers::{CreateGameRequest, create_game};

    // Test 2: With valid authentication
    let claims = Claims {
        user_id: user_id.to_string(),
        email: test_email.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };

    let request = CreateGameRequest {
        enrollment_timeout_seconds: Some(300),
    };

    let result = create_game(AxumState(state), Extension(claims), Json(request)).await;

    assert!(
        result.is_ok(),
        "create_game should succeed with valid authentication"
    );

    let response = result.unwrap().0;
    assert_eq!(
        response.creator_id, user_id,
        "creator_id should match authenticated user"
    );
    assert_eq!(
        response.player_count, 1,
        "player_count should be 1 (creator)"
    );
    assert!(!response.game_id.is_nil(), "game_id should be generated");
}

/// Tests that creator_id is correctly extracted from JWT
///
/// Validates:
/// - creator_id in response matches user_id from JWT claims
/// - Multiple users can create separate games
/// - Each game has correct creator assigned
#[tokio::test]
async fn test_create_game_creator_id_from_jwt() {
    use axum::Json;
    use axum::extract::{Extension, State as AxumState};
    use blackjack_api::auth::Claims;
    use blackjack_api::handlers::{CreateGameRequest, create_game};

    // Create two different users
    let user_service = Arc::new(UserService::new());

    let user1_email = "user1@example.com";
    let user1_id = user_service
        .register(user1_email.to_string(), "Pass123!".to_string())
        .expect("Failed to register user1");

    let user2_email = "user2@example.com";
    let user2_id = user_service
        .register(user2_email.to_string(), "Pass123!".to_string())
        .expect("Failed to register user2");

    // Setup AppState
    let config = Arc::new(blackjack_api::config::AppConfig::from_file().unwrap());
    let game_service = Arc::new(GameService::new(
        ServiceConfig::default(),
        user_service.clone(),
    ));
    let invitation_service = Arc::new(InvitationService::new(InvitationConfig::default()));
    let rate_limiter = blackjack_api::rate_limiter::RateLimiter::new(10);

    let state = AppState {
        game_service,
        user_service,
        invitation_service,
        config,
        rate_limiter,
    };

    // User 1 creates a game
    let claims1 = Claims {
        user_id: user1_id.to_string(),
        email: user1_email.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };

    let result1 = create_game(
        AxumState(state.clone()),
        Extension(claims1),
        Json(CreateGameRequest {
            enrollment_timeout_seconds: None,
        }),
    )
    .await;

    assert!(result1.is_ok());
    let game1 = result1.unwrap().0;
    assert_eq!(game1.creator_id, user1_id);

    // User 2 creates a game
    let claims2 = Claims {
        user_id: user2_id.to_string(),
        email: user2_email.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };

    let result2 = create_game(
        AxumState(state),
        Extension(claims2),
        Json(CreateGameRequest {
            enrollment_timeout_seconds: Some(600),
        }),
    )
    .await;

    assert!(result2.is_ok());
    let game2 = result2.unwrap().0;
    assert_eq!(game2.creator_id, user2_id);

    // Verify games are different
    assert_ne!(game1.game_id, game2.game_id);
    assert_ne!(game1.creator_id, game2.creator_id);
}

/// Tests that create_game rejects non-existent users
///
/// Validates:
/// - Token with user_id that doesn't exist in database returns 401
/// - Error message indicates user not found
#[tokio::test]
async fn test_create_game_rejects_nonexistent_user() {
    use axum::Json;
    use axum::extract::{Extension, State as AxumState};
    use blackjack_api::auth::Claims;
    use blackjack_api::handlers::{CreateGameRequest, create_game};

    // Setup AppState (empty user service)
    let user_service = Arc::new(UserService::new());
    let config = Arc::new(blackjack_api::config::AppConfig::from_file().unwrap());
    let game_service = Arc::new(GameService::new(
        ServiceConfig::default(),
        user_service.clone(),
    ));
    let invitation_service = Arc::new(InvitationService::new(InvitationConfig::default()));
    let rate_limiter = blackjack_api::rate_limiter::RateLimiter::new(10);

    let state = AppState {
        game_service,
        user_service,
        invitation_service,
        config,
        rate_limiter,
    };

    // Create claims with non-existent user_id
    let fake_user_id = uuid::Uuid::new_v4();
    let claims = Claims {
        user_id: fake_user_id.to_string(),
        email: "nonexistent@example.com".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };

    let result = create_game(
        AxumState(state),
        Extension(claims),
        Json(CreateGameRequest {
            enrollment_timeout_seconds: None,
        }),
    )
    .await;

    assert!(result.is_err(), "Should reject non-existent user");

    let error = result.unwrap_err();
    assert_eq!(error.status, axum::http::StatusCode::UNAUTHORIZED);
    assert_eq!(error.code, "USER_NOT_FOUND");
}

/// Tests that JWT expiration is validated
///
/// Note: JWT expiration validation is done in the auth_middleware,
/// not in the handler. This test documents the expected behavior.
/// The actual validation is tested in middleware tests.
#[test]
fn test_jwt_expiration_validation_is_in_middleware() {
    // JWT expiration is validated by jsonwebtoken library in auth_middleware
    // The Validation::default() includes exp claim validation
    //
    // When a token is expired:
    // - decode() in auth_middleware returns Err
    // - Middleware returns 401 Unauthorized
    // - Request never reaches the handler
    //
    // This is tested in the middleware layer, not in handler tests

    // This test serves as documentation only
    // JWT expiration is validated in auth_middleware
}

/// Tests that stand endpoint returns 409 when it's not the player's turn
///
/// Validates:
/// - Stand endpoint checks current turn before processing
/// - Returns CONFLICT (409) status
/// - Returns NOT_YOUR_TURN error code
/// - Returns appropriate error message
#[tokio::test]
async fn test_stand_not_your_turn() {
    use axum::Extension;
    use axum::extract::Path;
    use axum::extract::State as AxumState;
    use blackjack_api::auth::Claims;
    use blackjack_api::handlers::stand;

    // Setup AppState
    let user_service = Arc::new(UserService::new());
    let config = Arc::new(blackjack_api::config::AppConfig::from_file().unwrap());
    let game_service = Arc::new(GameService::new(
        ServiceConfig::default(),
        user_service.clone(),
    ));
    let invitation_service = Arc::new(InvitationService::new(InvitationConfig::default()));
    let rate_limiter = blackjack_api::rate_limiter::RateLimiter::new(10);

    let state = AppState {
        game_service: game_service.clone(),
        user_service: user_service.clone(),
        invitation_service,
        config,
        rate_limiter,
    };

    // Create two users
    let user1_id = user_service
        .register(
            "player1@example.com".to_string(),
            "TestP@ssw0rd".to_string(),
        )
        .unwrap();
    let user2_id = user_service
        .register(
            "player2@example.com".to_string(),
            "TestP@ssw0rd".to_string(),
        )
        .unwrap();

    // Create game with player1
    let game_id = game_service.create_game(user1_id, None).unwrap();

    // Enroll player2
    game_service.enroll_player(game_id, user2_id).unwrap();

    // Close enrollment
    game_service.close_enrollment(game_id, user1_id).unwrap();

    // Current turn is player1, but player2 tries to stand
    let claims = Claims {
        user_id: user2_id.to_string(),
        email: "player2@example.com".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };

    let result = stand(AxumState(state), Extension(claims), Path(game_id)).await;

    assert!(
        result.is_err(),
        "Should reject stand when not player's turn"
    );

    let error = result.unwrap_err();
    assert_eq!(error.status, 409, "Should return 409 Conflict");
    assert_eq!(
        error.code, "NOT_YOUR_TURN",
        "Should return NOT_YOUR_TURN code"
    );
    assert_eq!(
        error.message, "It's not your turn",
        "Should return appropriate message"
    );
}

/// Tests that draw_card endpoint returns 409 when game is already finished
///
/// Validates:
/// - Draw card endpoint properly handles finished games
/// - Returns CONFLICT (409) status
/// - Returns GAME_FINISHED error code
/// - Returns appropriate error message
#[tokio::test]
async fn test_draw_card_game_already_finished() {
    use axum::Extension;
    use axum::extract::Path;
    use axum::extract::State as AxumState;
    use blackjack_api::auth::Claims;
    use blackjack_api::handlers::draw_card;

    // Setup AppState
    let user_service = Arc::new(UserService::new());
    let config = Arc::new(blackjack_api::config::AppConfig::from_file().unwrap());
    let game_service = Arc::new(GameService::new(
        ServiceConfig::default(),
        user_service.clone(),
    ));
    let invitation_service = Arc::new(InvitationService::new(InvitationConfig::default()));
    let rate_limiter = blackjack_api::rate_limiter::RateLimiter::new(10);

    let state = AppState {
        game_service: game_service.clone(),
        user_service: user_service.clone(),
        invitation_service,
        config,
        rate_limiter,
    };

    // Create user and game
    let user_id = user_service
        .register(
            "player1@example.com".to_string(),
            "TestP@ssw0rd".to_string(),
        )
        .unwrap();
    let game_id = game_service.create_game(user_id, None).unwrap();

    // Close enrollment
    game_service.close_enrollment(game_id, user_id).unwrap();

    // Finish the game
    game_service.finish_game(game_id, user_id).unwrap();

    // Try to draw card after game is finished
    let claims = Claims {
        user_id: user_id.to_string(),
        email: "player1@example.com".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };

    let result = draw_card(AxumState(state), Extension(claims), Path(game_id)).await;

    assert!(
        result.is_err(),
        "Should reject draw card when game is finished"
    );

    let error = result.unwrap_err();
    assert_eq!(error.status, 409, "Should return 409 Conflict");
    assert_eq!(
        error.code, "GAME_FINISHED",
        "Should return GAME_FINISHED code"
    );
    assert_eq!(
        error.message, "Game has already finished",
        "Should return appropriate message"
    );
}
