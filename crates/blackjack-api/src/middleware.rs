use crate::auth::Claims;
use crate::error::ApiError;
use axum::extract::{Request, State};
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{DecodingKey, Validation, decode};

/// JWT authentication middleware
///
/// Validates JWT tokens from the Authorization header and injects the decoded
/// claims into the request extensions for use by downstream handlers.
///
/// This middleware allows public routes (without Authorization header) to pass through,
/// but validates and injects claims when the header is present.
///
/// # Authentication Flow
///
/// 1. Checks if the `Authorization` header is present
/// 2. If absent, allows the request to proceed (public route)
/// 3. If present, verifies it starts with "Bearer " prefix
/// 4. Decodes and validates the JWT using the configured secret
/// 5. Checks token expiration automatically via `exp` claim
/// 6. Injects validated `Claims` into request extensions
/// 7. Passes request to next middleware/handler
///
/// # Headers Required (for protected routes)
///
/// ```text
/// Authorization: Bearer <jwt_token>
/// ```
///
/// # Errors
///
/// Returns `ApiError::unauthorized()` (401) if:
/// - Header is present but doesn't start with "Bearer "
/// - Token is malformed or invalid
/// - Token signature verification fails
/// - Token has expired
///
/// # Protected Routes
///
/// Handlers can check for authentication by attempting to extract `Claims`:
///
/// ```ignore
/// use axum::Extension;
/// use blackjack_api::auth::Claims;
///
/// async fn protected_handler(
///     Extension(claims): Extension<Claims>
/// ) -> &'static str {
///     // This will fail with 500 if Claims are missing
///     // Always use this pattern on routes that require auth
///     "OK"
/// }
/// ```
///
/// # Usage in Routes
///
/// ```ignore
/// use axum::{Router, routing::get, middleware};
/// use blackjack_api::middleware::auth_middleware;
///
/// async fn protected_handler() -> &'static str { "OK" }
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

    // If no Authorization header, allow the request (public route)
    let auth_header = match headers.get("Authorization").and_then(|h| h.to_str().ok()) {
        Some(header) => header,
        None => return Ok(next.run(request).await),
    };

    // If Authorization header exists, it must be valid
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
        user_id = token_data.claims.user_id,
        "Authentication successful"
    );

    request.extensions_mut().insert(token_data.claims);

    Ok(next.run(request).await)
}

/// Rate limiting middleware
///
/// Enforces request rate limits per player using a sliding window algorithm.
/// Only applies rate limiting to authenticated requests (those with Claims).
///
/// # Rate Limit Key
///
/// Requests are tracked using: `{game_id}:{email}`
///
/// This ensures:
/// - Each player has their own rate limit bucket
/// - Players in different games don't interfere with each other
/// - One player can't exhaust limits for others
/// - Public routes (without Claims) bypass rate limiting
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
/// # Usage in Routes
///
/// ```ignore
/// use axum::{Router, routing::post, middleware};
/// use blackjack_api::middleware::{auth_middleware, rate_limit_middleware};
///
/// async fn draw_card_handler() -> &'static str { "OK" }
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
    // Only apply rate limiting to authenticated requests
    if let Some(claims) = request.extensions().get::<Claims>() {
        // M7: Use user_id for rate limiting instead of game_id:email
        let key = claims.user_id.clone();
        state.rate_limiter.check_rate_limit(&key)?;
    }

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
/// ```ignore
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

    response
        .headers_mut()
        .insert("X-API-Deprecated", HeaderValue::from_static("false"));

    response.headers_mut().insert(
        "X-API-Sunset-Date",
        HeaderValue::from_str(&sunset_date.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("N/A")),
    );

    Ok(response)
}

/// Security headers middleware (Milestone 8)
///
/// Adds security-related HTTP headers to all responses to protect against
/// common web vulnerabilities.
///
/// # Headers Added
///
/// - **X-Content-Type-Options: nosniff**
///   - Prevents MIME type sniffing
///   - Forces browsers to respect declared Content-Type
///
/// - **X-Frame-Options: DENY**
///   - Prevents clickjacking attacks
///   - Blocks embedding in iframes
///
/// - **X-XSS-Protection: 1; mode=block**
///   - Enables browser XSS filter
///   - Blocks page load on XSS detection
///
/// - **Strict-Transport-Security: max-age=31536000; includeSubDomains**
///   - Forces HTTPS connections
///   - Applies to all subdomains
///   - 1 year max-age
///
/// - **Content-Security-Policy: default-src 'self'**
///   - Restricts resource loading to same origin
///   - Prevents inline scripts and styles by default
///   - Mitigates XSS attacks
///
/// # Example
///
/// ```ignore
/// use axum::middleware;
/// use blackjack_api::middleware::security_headers_middleware;
///
/// let app = Router::new()
///     .route("/", get(handler))
///     .layer(middleware::from_fn(security_headers_middleware));
/// ```
///
/// # Security Best Practices
///
/// This middleware implements OWASP recommendations:
/// - Clickjacking protection (X-Frame-Options)
/// - MIME sniffing protection (X-Content-Type-Options)
/// - XSS protection (X-XSS-Protection, CSP)
/// - HTTPS enforcement (Strict-Transport-Security)
///
/// # Production Deployment
///
/// For production environments:
/// 1. Ensure HTTPS is configured at load balancer/reverse proxy
/// 2. Consider adding Referrer-Policy header
/// 3. Consider adding Permissions-Policy header
/// 4. Review CSP policy for your specific needs
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let mut response = next.run(request).await;

    // Prevent MIME type sniffing
    response.headers_mut().insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );

    // Prevent clickjacking attacks
    response.headers_mut().insert(
        "X-Frame-Options",
        HeaderValue::from_static("DENY"),
    );

    // Enable XSS filter in browsers
    response.headers_mut().insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );

    // Force HTTPS connections (only in production)
    // Note: This should be commented out in development without HTTPS
    response.headers_mut().insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // Content Security Policy - restrict resource loading
    response.headers_mut().insert(
        "Content-Security-Policy",
        HeaderValue::from_static("default-src 'self'"),
    );

    tracing::trace!("Security headers added to response");

    Ok(response)
}
