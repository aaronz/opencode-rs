//! Retry module for OpenCode
//!
//! Provides configurable retry with exponential backoff and jitter.

use rand::Rng;
use std::time::Duration;

/// Configuration for retry behavior with exponential backoff.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub base_delay: Duration,
    /// Maximum delay cap
    pub max_delay: Duration,
    /// Whether to add random jitter
    pub jitter: bool,
}

impl RetryConfig {
    /// Create a new retry config.
    ///
    /// # Arguments
    /// * `max_attempts` - Maximum number of attempts
    /// * `base_delay` - Initial delay between retries
    pub fn new(max_attempts: u32, base_delay: Duration) -> Self {
        Self {
            max_attempts,
            base_delay,
            max_delay: Duration::MAX,
            jitter: true,
        }
    }

    /// Set the maximum delay cap.
    pub fn with_max_delay(mut self, max: Duration) -> Self {
        self.max_delay = max;
        self
    }

    /// Disable jitter.
    pub fn with_no_jitter(mut self) -> Self {
        self.jitter = false;
        self
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::new(3, Duration::from_millis(100)).with_max_delay(Duration::from_secs(10))
    }
}

/// Retry an async operation with exponential backoff and optional jitter.
///
/// # Arguments
/// * `config` - Retry configuration
/// * `operation` - Async operation to retry, receives attempt number (0-based)
///
/// # Returns
/// * `Ok(value)` on success
/// * `Err((error, attempts_made))` on exhaustion
pub async fn retry<R, E, F, Fut>(config: RetryConfig, operation: F) -> Result<R, (E, u32)>
where
    F: Fn(u32) -> Fut,
    Fut: std::future::Future<Output = Result<R, E>>,
    E: std::fmt::Debug,
{
    let mut attempts = 0u32;
    let mut last_error = None;

    while attempts < config.max_attempts {
        match operation(attempts).await {
            Ok(value) => return Ok(value),
            Err(e) => {
                last_error = Some(e);
                attempts += 1;

                if attempts < config.max_attempts {
                    let delay = calculate_delay(&config, attempts);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    Err((
        last_error.expect("At least one error should have occurred"),
        attempts,
    ))
}

/// Calculate the delay for a given attempt number.
fn calculate_delay(config: &RetryConfig, attempt: u32) -> Duration {
    let base = config.base_delay * 2u32.pow(attempt);
    let capped = std::cmp::min(base, config.max_delay);

    if config.jitter {
        let mut rng = rand::thread_rng();
        let jitter_ms = rng.gen_range(0..100);
        capped + Duration::from_millis(jitter_ms)
    } else {
        capped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.base_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(10));
        assert!(config.jitter);
    }

    #[tokio::test]
    async fn test_retry_succeeds_on_first_attempt() {
        let count = Arc::new(AtomicU32::new(0));
        let c = count.clone();
        let result = retry(RetryConfig::default(), |_| {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok::<_, ()>(42)
            }
        })
        .await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_retries_on_failure() {
        let count = Arc::new(AtomicU32::new(0));
        let c = count.clone();
        let result = retry(
            RetryConfig::new(3, Duration::from_millis(10)),
            move |attempt| {
                let c = c.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    if attempt < 2 {
                        Err(())
                    } else {
                        Ok(42)
                    }
                }
            },
        )
        .await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_returns_err_after_max_attempts() {
        let result = retry(RetryConfig::new(2, Duration::from_millis(10)), |_| async {
            Err::<i32, ()>(())
        })
        .await;
        assert!(result.is_err());
        let (_, attempts) = result.unwrap_err();
        assert_eq!(attempts, 2);
    }

    #[tokio::test]
    async fn test_retry_with_jitter() {
        let config = RetryConfig::new(3, Duration::from_millis(50)).with_no_jitter();
        let start = std::time::Instant::now();
        let _ = retry(config, |_| async { Err::<i32, ()>(()) }).await;
        let elapsed = start.elapsed();
        // Without jitter, delays should be: 50ms, 100ms = ~150ms minimum
        assert!(elapsed.as_millis() >= 100);
    }
}
