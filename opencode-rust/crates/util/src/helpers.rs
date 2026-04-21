use crate::retry::RetryConfig;
use std::future::Future;
use std::sync::OnceLock;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TimeoutError;

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "operation timed out")
    }
}

impl std::error::Error for TimeoutError {}

pub struct Lazy<T> {
    cell: OnceLock<T>,
    init: fn() -> T,
}

impl<T: Send + Sync> Lazy<T> {
    pub const fn new(init: fn() -> T) -> Self {
        Self {
            cell: OnceLock::new(),
            init,
        }
    }

    pub fn get(&self) -> &T {
        self.cell.get_or_init(|| (self.init)())
    }
}

#[macro_export]
macro_rules! iife {
    (|| $expr:expr) => {
        $expr
    };
    (||$($tokens:tt)*) => {
        { $( $tokens )* }
    };
}

pub async fn with_timeout<T>(
    duration: Duration,
    future: impl Future<Output = T>,
) -> Result<T, TimeoutError> {
    tokio::time::timeout(duration, future)
        .await
        .map_err(|_| TimeoutError)
}

pub async fn wait_for<F, T>(condition: F, timeout: Duration) -> Result<T, TimeoutError>
where
    F: Fn() -> Option<T>,
{
    let start = std::time::Instant::now();
    let poll_interval = Duration::from_millis(100);

    loop {
        if let Some(value) = condition() {
            return Ok(value);
        }

        if start.elapsed() >= timeout {
            return Err(TimeoutError);
        }

        tokio::time::sleep(poll_interval).await;
    }
}

pub async fn retry_until<F, Fut, T, E>(config: RetryConfig, condition: F) -> Result<T, (E, u32)>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<Option<T>, E>>,
    E: std::fmt::Debug,
{
    let mut attempts = 0u32;

    loop {
        match condition().await {
            Ok(Some(value)) => return Ok(value),
            Ok(None) => {
                attempts += 1;
                if attempts >= config.max_attempts {
                    panic!("retry_until exhausted without error")
                }
                let delay = calculate_delay(&config, attempts);
                tokio::time::sleep(delay).await;
            }
            Err(e) => {
                attempts += 1;
                if attempts >= config.max_attempts {
                    return Err((e, attempts));
                }
                let delay = calculate_delay(&config, attempts);
                tokio::time::sleep(delay).await;
            }
        }
    }
}

fn calculate_delay(config: &RetryConfig, attempt: u32) -> Duration {
    let base = config.base_delay * 2u32.pow(attempt);
    std::cmp::min(base, config.max_delay)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_thread_safe() {
        use std::thread;

        static LAZY: Lazy<u32> = Lazy::new(|| 42);

        let handles: Vec<_> = (0..10).map(|_| thread::spawn(|| *LAZY.get())).collect();

        for handle in handles {
            assert_eq!(handle.join().unwrap(), 42);
        }
    }

    #[test]
    fn test_lazy_initializes_once() {
        static CALLED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        static LAZY: Lazy<u32> = Lazy::new(|| {
            CALLED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            42
        });

        assert_eq!(*LAZY.get(), 42);
        assert_eq!(*LAZY.get(), 42);
        assert_eq!(*LAZY.get(), 42);

        assert_eq!(CALLED.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn test_iife_macro() {
        let value = iife!(|| 1 + 2);
        assert_eq!(value, 3);

        let value2 = iife!(|| {
            let x = 1;
            let y = 2;
            x + y
        });
        assert_eq!(value2, 3);
    }

    #[tokio::test]
    async fn test_with_timeout_fires() {
        let result = with_timeout(Duration::from_millis(10), async {
            tokio::time::sleep(Duration::from_secs(1)).await;
        })
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_with_timeout_success() {
        let result = with_timeout(Duration::from_secs(1), async { 42 }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_wait_for_success() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c = counter.clone();
        let result = wait_for(
            move || {
                let c = c.clone();
                let val = c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if val >= 2 {
                    Some("ready".to_string())
                } else {
                    None
                }
            },
            Duration::from_secs(5),
        )
        .await;
        assert_eq!(result.unwrap(), "ready");
    }

    #[tokio::test]
    async fn test_wait_for_timeout() {
        let result = wait_for(|| None::<String>, Duration::from_millis(50)).await;
        assert!(result.is_err());
    }
}
