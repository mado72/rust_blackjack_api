use crate::auth::Claims;
use crate::error::ApiError;
use axum::extract::{Request, State};
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{decode, DecodingKey, Validation};

/// JWT authentication middleware
///
/// Validates JWT tokens from the Authorization header and injects the decoded
/// claims into the request extensions for use by downstream handlers.
///
/// # Authentication Flow
///
/// 1. Extracts the `Authorization` header from the request
/// 2. Verifies it starts with "Bearer " prefix
/// 3. Decodes and validates the JWT using the configured secret
/// 4. Checks token expiration automatically via `exp` claim
/// 5. Injects validated `Claims` into request extensions
/// 6. Passes request to next middleware/handler
///
/// # Headers Required
///
/// ```text
/// Authorization: Bearer <jwt_token>
/// ```
///
/// # Errors
///
/// Returns `ApiError::unauthorized()` (401) if:
/// - Authorization header is missing
/// - Header doesn't start with "Bearer "
/// - Token is malformed or invalid
/// - Token signature verification fails
/// - Token has expired
///
/// # Usage in Routes
///
/// ```no_run
/// use axum::{Router, routing::get, middleware};
/// use blackjack_api::middleware::auth_middleware;
///
/// # async fn protected_handler() -> &'static str { "OK" }
/// let app = Router::new()
///     .route("/protected", get(protected_handler))
///     .layer(middleware::from_fn_with_state(state, auth_middleware));
/// ```
///
/// # Logging
///
/// - Logs authentication failures at DEBUG level
/// - Logs successful authentication at DEBUG level with email and game_id
pub async fn auth_middleware(
    State(state): State<crate::AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let headers = request.headers();
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(ApiError::unauthorized)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(ApiError::unauthorized());
    }

    let token = &auth_header[7..];

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt.secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|err| {
        tracing::debug!(error = ?err, "JWT validation failed");
        ApiError::unauthorized()
    })?;

    tracing::debug!(
        email = token_data.claims.email,
        game_id = token_data.claims.game_id,
        "Authentication successful"
    );

    request.extensions_mut().insert(token_data.claims);

    Ok(next.run(request).await)
}

/// Rate limiting middleware
///
/// Enforces request rate limits per player using a sliding window algorithm.
/// Must be used after `auth_middleware` as it requires the `Claims` extension.
///
/// # Rate Limit Key
///
/// Requests are tracked using: `{game_id}:{email}`
///
/// This ensures:
/// - Each player has their own rate limit bucket
/// - Players in different games don't interfere with each other
/// - One player can't exhaust limits for others
///
/// # Configuration
///
/// The limit is configured via `config.rate_limit.requests_per_minute`
/// (default: 10 requests per minute)
///
/// # Errors
///
/// Returns `ApiError::rate_limit_exceeded()` (429) if:
/// - Player has made too many requests in the last 60 seconds
///
/// Returns `ApiError::unauthorized()` (401) if:
/// - Claims extension is not present (auth_middleware not applied)
///
/// # Usage in Routes
///
/// ```no_run
/// use axum::{Router, routing::post, middleware};
/// use blackjack_api::middleware::{auth_middleware, rate_limit_middleware};
///
/// # async fn draw_card_handler() -> &'static str { "OK" }
/// let app = Router::new()
///     .route("/games/:id/draw", post(draw_card_handler))
///     .layer(middleware::from_fn_with_state(state.clone(), rate_limit_middleware))
///     .layer(middleware::from_fn_with_state(state, auth_middleware));
/// ```
///
/// # Logging
///
/// Logs rate limit violations at WARN level with the tracking key
pub async fn rate_limit_middleware(
    State(state): State<crate::AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(ApiError::unauthorized)?;

    let key = format!("{}:{}", claims.game_id, claims.email);
    state.rate_limiter.check_rate_limit(&key)?;

    Ok(next.run(request).await)
}

/// API version deprecation middleware
///
/// Adds deprecation headers to responses to inform clients about API lifecycle.
/// This middleware supports graceful API version transitions by providing
/// advance notice of deprecation and sunset dates.
///
/// # Headers Added
///
/// - `X-API-Deprecated: false` - Indicates if this API version is deprecated
/// - `X-API-Sunset-Date: YYYY-MM-DD` - Date when this version will be retired
///
/// The sunset date is calculated as:
/// ```text
/// current_date + (version_deprecation_months * 30 days)
/// ```
///
/// # Configuration
///
/// Controlled by `config.api.version_deprecation_months` (default: 6 months)
///
/// # Purpose
///
/// Following the [Sunset HTTP Header specification](https://datatracker.ietf.org/doc/html/rfc8594),
/// this middleware:
/// - Provides clients visibility into API lifecycle
/// - Allows gradual migration to new API versions
/// - Prevents sudden breaking changes
///
/// # Future Enhancement
///
/// When implementing v2 alongside v1, set `X-API-Deprecated: true` for v1 routes.
///
/// # Example Response Headers
///
/// ```text
/// HTTP/1.1 200 OK
/// X-API-Deprecated: false
/// X-API-Sunset-Date: 2026-07-06
/// Content-Type: application/json
/// ```
///
/// # Usage
///
/// ```no_run
/// use axum::{Router, middleware};
/// use blackjack_api::middleware::version_deprecation_middleware;
///
/// let app = Router::new()
///     .layer(middleware::from_fn_with_state(state, version_deprecation_middleware));
/// ```
pub async fn version_deprecation_middleware(
    State(state): State<crate::AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let mut response = next.run(request).await;

    // Calculate sunset date (current date + deprecation months)
    let sunset_date = chrono::Utc::now()
        + chrono::Duration::days((state.config.api.version_deprecation_months * 30) as i64);

    response.headers_mut().insert(
        "X-API-Deprecated",
        HeaderValue::from_static("false"),
    );

    response.headers_mut().insert(
        "X-API-Sunset-Date",
        HeaderValue::from_str(&sunset_date.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("N/A")),
    );

    Ok(response)
}
