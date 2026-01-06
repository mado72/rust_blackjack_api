//! Blackjack Multi-Player REST API
//!
//! This crate provides a production-ready REST API for a multi-player blackjack game
//! with JWT authentication, rate limiting, structured logging, and health checks.
//!
//! # Architecture
//!
//! The API is built on [Axum](https://github.com/tokio-rs/axum) and follows a layered architecture:
//!
//! - **Handlers** - Process HTTP requests and return responses
//! - **Middleware** - Authentication, rate limiting, deprecation headers
//! - **Service Layer** - Game logic and state management (via `blackjack-service`)
//! - **Core Domain** - Game rules and data structures (via `blackjack-core`)
//!
//! # Features
//!
//! - **JWT Authentication** - Secure player authentication per game session
//! - **Rate Limiting** - Per-player request limits using sliding window algorithm
//! - **CORS Support** - Configurable cross-origin resource sharing
//! - **Structured Logging** - Tracing-based observability with configurable levels
//! - **External Configuration** - File-based config with environment variable overrides
//! - **API Versioning** - Deprecation headers for graceful version transitions
//! - **Standardized Errors** - Consistent JSON error responses with details
//!
//! # Quick Start
//!
//! ```no_run
//! use blackjack_api::{AppState, config::AppConfig};
//! use blackjack_service::{GameService, ServiceConfig};
//! use blackjack_api::rate_limiter::RateLimiter;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Load configuration
//!     let config = Arc::new(AppConfig::from_file().expect("Failed to load config"));
//!     
//!     // Create services
//!     let game_service = Arc::new(GameService::new(ServiceConfig::from_env()));
//!     let rate_limiter = RateLimiter::new(config.rate_limit.requests_per_minute);
//!     
//!     // Create shared state
//!     let state = AppState {
//!         game_service,
//!         config,
//!         rate_limiter,
//!     };
//!     
//!     // Build and run server (see main.rs for full example)
//! }
//! ```
//!
//! # Configuration
//!
//! Configuration is loaded from `config.toml` and can be overridden with environment variables:
//!
//! ```toml
//! [server]
//! host = "127.0.0.1"
//! port = 8080
//!
//! [jwt]
//! secret = "your-secret-key"
//! expiration_hours = 24
//!
//! [rate_limit]
//! requests_per_minute = 10
//! ```
//!
//! Environment variables use the `BLACKJACK_` prefix:
//!
//! ```bash
//! export BLACKJACK_SERVER_PORT=8080
//! export BLACKJACK_JWT_SECRET=my-secret-key
//! ```
//!
//! # API Endpoints
//!
//! ## Public Endpoints
//!
//! - `POST /api/v1/auth/login` - Authenticate a player for a game
//!
//! ## Protected Endpoints (require JWT)
//!
//! - `GET /api/v1/games/:id` - Get game state
//! - `POST /api/v1/games/:id/draw` - Draw a card
//! - `PUT /api/v1/games/:id/ace` - Change Ace value
//! - `POST /api/v1/games/:id/finish` - Finish the game
//! - `GET /api/v1/games/:id/results` - Get game results
//!
//! # Modules
//!
//! - [`auth`] - JWT claims and authentication types
//! - [`config`] - Application configuration structures
//! - [`error`] - Standardized API error responses
//! - [`handlers`] - HTTP request handlers
//! - [`middleware`] - Authentication, rate limiting, and deprecation middleware
//! - [`rate_limiter`] - Request rate limiting implementation

pub mod auth;
pub mod config;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod rate_limiter;

use config::AppConfig;
use rate_limiter::RateLimiter;
use blackjack_service::GameService;
use std::sync::Arc;

/// Shared application state
///
/// This structure is cloned for each request handler and middleware,
/// providing access to shared services and configuration. All fields
/// use `Arc` or are `Clone` to enable efficient sharing across threads.
///
/// # Thread Safety
///
/// All components are designed for concurrent access:
/// - `GameService` uses `Arc<Mutex<HashMap>>` internally
/// - `RateLimiter` uses `Arc<Mutex<HashMap>>` internally  
/// - `AppConfig` is immutable after initialization
///
/// # Example
///
/// ```
/// use blackjack_api::{AppState, config::AppConfig};
/// use blackjack_service::{GameService, ServiceConfig};
/// use blackjack_api::rate_limiter::RateLimiter;
/// use std::sync::Arc;
///
/// let config = Arc::new(AppConfig::from_file().unwrap());
/// let game_service = Arc::new(GameService::new(ServiceConfig::default()));
/// let rate_limiter = RateLimiter::new(10);
///
/// let state = AppState {
///     game_service,
///     config: config.clone(),
///     rate_limiter,
/// };
///
/// // State can be cloned efficiently for each request
/// let state_clone = state.clone();
/// ```
#[derive(Clone)]
pub struct AppState {
    /// Game service for managing blackjack games
    ///
    /// Provides methods for creating games, drawing cards, changing Ace values,
    /// and retrieving game state. Thread-safe for concurrent access.
    pub game_service: Arc<GameService>,
    
    /// Application configuration
    ///
    /// Contains all runtime configuration including server settings, JWT secrets,
    /// CORS origins, rate limits, and API versioning policies.
    pub config: Arc<AppConfig>,
    
    /// Rate limiter for request throttling
    ///
    /// Enforces per-player request limits using a sliding window algorithm.
    /// Tracks requests by `{game_id}:{email}` key.
    pub rate_limiter: RateLimiter,
}
