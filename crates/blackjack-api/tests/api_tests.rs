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
use blackjack_service::{GameService, ServiceConfig};
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
    let game_service = Arc::new(GameService::new(ServiceConfig::default()));
    let rate_limiter = blackjack_api::rate_limiter::RateLimiter::new(
        config.rate_limit.requests_per_minute
    );
    
    let _state = AppState {
        game_service,
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
        "Test message"
    ).with_details(details);
    
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
    use blackjack_service::GameError;
    use blackjack_api::error::ApiError;
    
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
}
