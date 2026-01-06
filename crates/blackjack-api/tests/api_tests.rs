use blackjack_api::AppState;
use blackjack_service::{GameService, ServiceConfig};
use std::sync::Arc;

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

#[test]
fn test_config_defaults() {
    let config = blackjack_api::config::AppConfig::from_file().unwrap();
    
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.jwt.expiration_hours, 24);
    assert_eq!(config.rate_limit.requests_per_minute, 10);
    assert_eq!(config.api.version_deprecation_months, 6);
}

#[test]
fn test_jwt_config() {
    let config = blackjack_api::config::AppConfig::from_file().unwrap();
    
    // Secret should be set (even if it's the default dev secret)
    assert!(!config.jwt.secret.is_empty());
    assert_eq!(config.jwt.secret, "dev-secret-key-change-in-production");
}

#[test]
fn test_cors_config() {
    let config = blackjack_api::config::AppConfig::from_file().unwrap();
    
    assert_eq!(config.cors.allowed_origins.len(), 1);
    assert_eq!(config.cors.allowed_origins[0], "http://localhost:3000");
}
