use config::{Config, ConfigError, File};
use serde::Deserialize;

/// Server configuration settings
///
/// Controls the HTTP server binding address and port.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// The host address to bind to (e.g., "127.0.0.1" or "0.0.0.0")
    pub host: String,
    /// The port number to listen on (typically 8080 for development)
    pub port: u16,
}

/// CORS (Cross-Origin Resource Sharing) configuration
///
/// Defines which origins are allowed to make requests to the API.
#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    /// List of allowed origin URLs (e.g., ["http://localhost:3000"])
    pub allowed_origins: Vec<String>,
}

/// JWT (JSON Web Token) authentication configuration
///
/// Controls token generation and validation settings.
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    /// Secret key used for signing and verifying JWT tokens
    ///
    /// **Security Note**: This should be a strong, randomly generated secret
    /// in production and stored securely (e.g., via environment variables).
    pub secret: String,
    /// Token expiration time in hours (default: 24)
    pub expiration_hours: u64,
}

/// Rate limiting configuration
///
/// Controls the maximum number of requests allowed per player per minute.
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed per minute per player
    /// 
    /// Requests are tracked using a key format: `{game_id}:{email}`
    pub requests_per_minute: u32,
}

/// API versioning configuration
///
/// Controls deprecation policies for API versions.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiConfig {
    /// Number of months before an API version is sunset after deprecation
    /// 
    /// Used to calculate the X-API-Sunset-Date header value
    pub version_deprecation_months: u64,
}

/// Main application configuration
///
/// Aggregates all configuration sections and provides methods for loading
/// from files and environment variables.
///
/// # Configuration Sources
///
/// Configuration is loaded in the following order (later sources override earlier ones):
/// 1. `config.toml` file (searched in multiple locations)
/// 2. Environment variables prefixed with `BLACKJACK_`
///
/// # Environment Variable Examples
///
/// - `BLACKJACK_SERVER_PORT=8080`
/// - `BLACKJACK_JWT_SECRET=your-secret-key`
/// - `BLACKJACK_RATE_LIMIT_REQUESTS_PER_MINUTE=20`
///
/// # Example
///
/// ```no_run
/// use blackjack_api::config::AppConfig;
///
/// let config = AppConfig::from_file().expect("Failed to load configuration");
/// println!("Server will listen on {}:{}", config.server.host, config.server.port);
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Server configuration (host, port)
    pub server: ServerConfig,
    /// CORS configuration (allowed origins)
    pub cors: CorsConfig,
    /// JWT authentication configuration (secret, expiration)
    pub jwt: JwtConfig,
    /// Rate limiting configuration (requests per minute)
    pub rate_limit: RateLimitConfig,
    /// API versioning configuration (deprecation period)
    pub api: ApiConfig,
}

impl AppConfig {
    /// Loads configuration from file and environment variables
    ///
    /// # Configuration File Locations
    ///
    /// The method searches for `config.toml` in the following locations (in order):
    /// 1. `crates/blackjack-api/config.toml` (for workspace root execution)
    /// 2. `config.toml` (for execution from crate directory)
    /// 3. `../config.toml` (fallback for test execution)
    ///
    /// # Environment Variable Overrides
    ///
    /// After loading the file, environment variables with the prefix `BLACKJACK_`
    /// are applied, allowing runtime configuration changes without modifying files.
    ///
    /// Environment variables use underscore as separator and support nested paths:
    /// - `BLACKJACK_SERVER_HOST` → `server.host`
    /// - `BLACKJACK_JWT_SECRET` → `jwt.secret`
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if:
    /// - Configuration file cannot be found in any of the search paths
    /// - Configuration file has invalid TOML syntax
    /// - Required fields are missing
    /// - Field types don't match expected types
    ///
    /// # Example
    ///
    /// ```no_run
    /// use blackjack_api::config::AppConfig;
    ///
    /// // Load configuration
    /// let config = AppConfig::from_file().expect("Failed to load config");
    ///
    /// // Access configuration values
    /// assert_eq!(config.server.port, 8080);
    /// assert_eq!(config.jwt.expiration_hours, 24);
    /// ```
    pub fn from_file() -> Result<Self, ConfigError> {
        // Try multiple paths to support different execution contexts
        let config_paths = vec![
            "crates/blackjack-api/config.toml",
            "config.toml",
            "../config.toml",
        ];

        let mut builder = Config::builder();
        
        // Try to find and load the config file
        for path in config_paths {
            if std::path::Path::new(path).exists() {
                builder = builder.add_source(File::with_name(path).required(false));
                break;
            }
        }

        // Add environment variable overrides
        // Variables must be prefixed with BLACKJACK_ and use _ as separator
        // Example: BLACKJACK_SERVER_PORT=8080 maps to server.port
        builder = builder.add_source(
            config::Environment::with_prefix("BLACKJACK")
                .separator("_")
                .try_parsing(true),
        );

        let config = builder.build()?;
        config.try_deserialize()
    }
}
