//! Blackjack API Server Entry Point
//!
//! This binary starts the HTTP server for the Blackjack multi-player game API.
//!
//! # Configuration
//!
//! The server loads configuration from multiple sources (in order of precedence):
//! 1. Environment variables prefixed with `BLACKJACK_`
//! 2. `config.toml` file
//! 3. `.env` file (if present)
//!
//! # Logging
//!
//! Structured logging is controlled by the `RUST_LOG` environment variable:
//! - `RUST_LOG=debug` - Detailed logs including auth attempts
//! - `RUST_LOG=info` - Standard operational logs (default)
//! - `RUST_LOG=warn` - Only warnings and errors
//!
//! # Running the Server
//!
//! ```bash
//! # Development with debug logs
//! RUST_LOG=debug cargo run -p blackjack-api
//!
//! # Production with custom port
//! BLACKJACK_SERVER_PORT=3000 cargo run -p blackjack-api --release
//! ```

use blackjack_api::config::AppConfig;
use blackjack_api::handlers::{
    accept_invitation, create_game, create_invitation, decline_invitation, draw_card,
    finish_game, get_game_results, get_game_state, get_pending_invitations, health_check,
    login, ready_check, register_user, set_ace_value, stand, close_enrollment, enroll_player,
    get_open_games,
};
use blackjack_api::middleware::{auth_middleware, rate_limit_middleware, version_deprecation_middleware};
use blackjack_api::rate_limiter::RateLimiter;
use blackjack_api::AppState;
use axum::routing::{get, post, put};
use axum::Router;
use blackjack_service::{GameService, ServiceConfig, UserService, InvitationService, InvitationConfig};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    // Load .env file if it exists (for local development)
    // This is optional and won't fail if the file doesn't exist
    dotenv::dotenv().ok();

    // Initialize structured logging with tracing
    // Reads RUST_LOG environment variable for filter configuration
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting Blackjack API server");

    // Load application configuration from config.toml with env var overrides
    // Panics if configuration is invalid (fail-fast on startup)
    let app_config = AppConfig::from_file().expect("Failed to load configuration");
    let app_config = Arc::new(app_config);

    tracing::info!(
        host = app_config.server.host,
        port = app_config.server.port,
        "Server configuration loaded"
    );

    // Create game service with configuration from environment variables
    // Service manages all active games with thread-safe concurrent access
    let service_config = ServiceConfig::from_env();
    let game_service = Arc::new(GameService::new(service_config));

    // Create user service for authentication
    let user_service = Arc::new(UserService::new());

    // Create invitation service with configuration
    let invitation_config = InvitationConfig::from_env();
    let invitation_service = Arc::new(InvitationService::new(invitation_config));

    // Create rate limiter with configured requests per minute
    // Uses sliding window algorithm to track requests per user
    let rate_limiter = RateLimiter::new(app_config.rate_limit.requests_per_minute);

    // Build shared application state
    // This state is cloned for each request and provides access to services
    let state = AppState {
        game_service,
        user_service,
        invitation_service,
        config: app_config.clone(),
        rate_limiter,
    };

    // Configure CORS (Cross-Origin Resource Sharing)
    // Uses allowed origins from configuration
    let cors = CorsLayer::new()
        .allow_origin(
            app_config
                .cors
                .allowed_origins
                .iter()
                .map(|origin| origin.parse().expect("Invalid CORS origin"))
                .collect::<Vec<_>>(),
        )
        .allow_methods(Any)
        .allow_headers(Any);

    tracing::info!(
        allowed_origins = ?app_config.cors.allowed_origins,
        "CORS configured"
    );

    // Build the application router with all routes and middleware
    let app = Router::new()
        // Health check endpoints (public, no authentication)
        .route("/health", get(health_check))
        .route("/health/ready", get(ready_check))
        // M7: User authentication endpoints
        .route("/api/v1/auth/register", post(register_user))
        .route("/api/v1/auth/login", post(login))
        // M7: Game enrollment endpoints
        .route("/api/v1/games", post(create_game))
        .route("/api/v1/games/open", get(get_open_games))
        .route("/api/v1/games/:game_id/enroll", post(enroll_player))
        .route("/api/v1/games/:game_id/close-enrollment", post(close_enrollment))
        // Protected game endpoints (require JWT authentication)
        .route("/api/v1/games/:game_id", get(get_game_state))
        .route("/api/v1/games/:game_id/draw", post(draw_card))
        .route("/api/v1/games/:game_id/ace", put(set_ace_value))
        .route("/api/v1/games/:game_id/stand", post(stand))
        .route("/api/v1/games/:game_id/finish", post(finish_game))
        .route("/api/v1/games/:game_id/results", get(get_game_results))
        // M7: Invitation endpoints
        .route("/api/v1/games/:game_id/invitations", post(create_invitation))
        .route("/api/v1/invitations/pending", get(get_pending_invitations))
        .route("/api/v1/invitations/:id/accept", post(accept_invitation))
        .route("/api/v1/invitations/:id/decline", post(decline_invitation))
        // Apply middleware layers in order (executed bottom-to-top)
        .layer(
            ServiceBuilder::new()
                // Rate limiting (applied first, checks all requests)
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    rate_limit_middleware,
                ))
                // Authentication (applied second, injects Claims for protected routes)
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                ))
                // API deprecation headers (applied last, adds headers to responses)
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    version_deprecation_middleware,
                ))
                // CORS support
                .layer(cors),
        )
        // Attach shared state to all handlers
        .with_state(state);

    // Bind TCP listener to configured host and port
    // Panics if binding fails (e.g., port already in use)
    let addr = format!("{}:{}", app_config.server.host, app_config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind server");

    tracing::info!(address = addr, "Server listening");

    // Start the HTTP server
    // This blocks until the server is shut down (e.g., via SIGTERM/SIGINT)
    axum::serve(listener, app)
        .await
        .expect("Server error");
}
