//! Utility struct for rate limiting operations.
//!
//! The `RateLimiter` struct is used to limit the rate at which operations can be executed.
//! This is to ensure the bot does not exceed the rate limits set by Discord.

use tokio::{sync::Mutex, time::Duration, time::Instant};

/// A rate limiter to control the frequency of operations.
///
/// # Examples
///
/// Basic usage of RateLimiter:
///
/// ```
/// use eule::utils::rate_limiter::RateLimiter;
/// use tokio::time::Duration;
///
/// #[tokio::main]
/// async fn main() {
///     let limiter = RateLimiter::new(2, Duration::from_secs(1));
///
///     assert!(limiter.check().await.is_ok());
///     assert!(limiter.check().await.is_ok());
///     assert!(limiter.check().await.is_err());
///
///     // Wait for the rate limit to reset
///     tokio::time::sleep(Duration::from_secs(1)).await;
///
///     assert!(limiter.check().await.is_ok());
/// }
/// ```
pub struct RateLimiter {
    rate: u32,
    per: Duration,
    allowance: Mutex<f64>,
    last_check: Mutex<Instant>,
}

impl RateLimiter {
    /// Creates a new RateLimiter instance.
    ///
    /// # Arguments
    ///
    /// * `rate` - The number of requests allowed per time period.
    /// * `per` - The duration of the time period.
    ///
    /// # Returns
    ///
    /// A new RateLimiter instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use eule::utils::rate_limiter::RateLimiter;
    /// use tokio::time::Duration;
    ///
    /// let limiter = RateLimiter::new(5, Duration::from_secs(60));
    /// ```
    pub fn new(rate: u32, per: Duration) -> Self {
        RateLimiter {
            rate,
            per,
            allowance: Mutex::new(rate as f64),
            last_check: Mutex::new(Instant::now()),
        }
    }

    /// Checks if a request is allowed under the current rate limit.
    ///
    /// # Returns
    ///
    /// A Result containing:
    /// * `Ok(())` if the request is allowed.
    /// * `Err(&'static str)` if it exceeds the rate limit.
    ///
    /// # Concurrency
    ///
    /// This method is safe to call from multiple threads as it uses Mutex for synchronization.
    ///
    /// # Examples
    ///
    /// ```
    /// use eule::utils::rate_limiter::RateLimiter;
    /// use tokio::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let limiter = RateLimiter::new(2, Duration::from_secs(1));
    ///
    ///     assert!(limiter.check().await.is_ok());
    ///     assert!(limiter.check().await.is_ok());
    ///     assert!(limiter.check().await.is_err());
    /// }
    /// ```
    pub async fn check(&self) -> Result<(), &'static str> {
        let mut allowance = self.allowance.lock().await;
        let mut last_check = self.last_check.lock().await;

        let current = Instant::now();
        let time_passed = current.duration_since(*last_check).as_secs_f64();
        *last_check = current;

        *allowance += time_passed * (self.rate as f64 / self.per.as_secs_f64());
        if *allowance > self.rate as f64 {
            *allowance = self.rate as f64;
        }

        if *allowance < 1.0 {
            Err("Rate limit exceeded")
        } else {
            *allowance -= 1.0;
            Ok(())
        }
    }
}
