use crate::error::ApiError;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Rate limiter implementation using a sliding window algorithm
///
/// This rate limiter tracks requests per unique key (typically `{game_id}:{email}`)
/// and enforces a maximum number of requests per minute. It uses a sliding window
/// approach where old requests automatically expire after 60 seconds.
///
/// # Thread Safety
///
/// The rate limiter is thread-safe and can be safely shared across multiple
/// async tasks. It uses `Arc<Mutex<HashMap>>` internally for concurrent access.
///
/// # Algorithm
///
/// 1. Each request is timestamped when it arrives
/// 2. Before checking limits, requests older than 60 seconds are removed
/// 3. If the count of recent requests exceeds the limit, the request is rejected
/// 4. Otherwise, the current request is added to the tracking queue
///
/// # Example
///
/// ```
/// use blackjack_api::rate_limiter::RateLimiter;
///
/// let limiter = RateLimiter::new(10); // 10 requests per minute
///
/// // First request succeeds
/// assert!(limiter.check_rate_limit("game123:player@example.com").is_ok());
///
/// // After 11 requests in a minute, would fail
/// // (not shown here as it would require 11 iterations)
/// ```
#[derive(Clone)]
pub struct RateLimiter {
    /// Stores request timestamps for each unique key
    ///
    /// Key format: `{game_id}:{email}`
    /// Value: Queue of request timestamps (oldest first)
    requests: Arc<Mutex<HashMap<String, VecDeque<Instant>>>>,

    /// Maximum number of requests allowed per minute per key
    requests_per_minute: u32,
}

impl RateLimiter {
    /// Creates a new rate limiter with the specified limit
    ///
    /// # Arguments
    ///
    /// * `requests_per_minute` - Maximum number of requests allowed per minute per key
    ///
    /// # Example
    ///
    /// ```
    /// use blackjack_api::rate_limiter::RateLimiter;
    ///
    /// // Allow 10 requests per minute per player
    /// let limiter = RateLimiter::new(10);
    /// ```
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            requests_per_minute,
        }
    }

    /// Checks if a request for the given key is within the rate limit
    ///
    /// This method performs the following operations:
    /// 1. Acquires a lock on the request tracking HashMap
    /// 2. Removes expired timestamps (older than 60 seconds)
    /// 3. Checks if the count exceeds the limit
    /// 4. If within limit, adds the current timestamp and returns Ok
    /// 5. If limit exceeded, logs a warning and returns an error
    ///
    /// # Arguments
    ///
    /// * `key` - Unique identifier for the rate limit bucket, typically `{game_id}:{email}`
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Request is allowed
    /// * `Err(ApiError)` - Rate limit exceeded (HTTP 429)
    ///
    /// # Example
    ///
    /// ```
    /// use blackjack_api::rate_limiter::RateLimiter;
    ///
    /// let limiter = RateLimiter::new(10);
    /// let key = "game-uuid:player@example.com";
    ///
    /// match limiter.check_rate_limit(key) {
    ///     Ok(()) => println!("Request allowed"),
    ///     Err(e) => println!("Rate limit exceeded: {}", e.message),
    /// }
    /// ```
    ///
    /// # Logging
    ///
    /// When the rate limit is exceeded, a warning is logged with the key:
    /// ```text
    /// WARN: Rate limit exceeded, key=game123:player@example.com
    /// ```
    pub fn check_rate_limit(&self, key: &str) -> Result<(), ApiError> {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();
        let one_minute_ago = now - std::time::Duration::from_secs(60);

        let player_requests = requests.entry(key.to_string()).or_default();

        // Remove requests older than 1 minute (sliding window)
        while let Some(&first) = player_requests.front() {
            if first < one_minute_ago {
                player_requests.pop_front();
            } else {
                break;
            }
        }

        // Check if limit exceeded
        if player_requests.len() >= self.requests_per_minute as usize {
            tracing::warn!(key = key, "Rate limit exceeded");
            return Err(ApiError::rate_limit_exceeded());
        }

        // Add current request
        player_requests.push_back(now);

        Ok(())
    }
}
