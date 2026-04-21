//! Integration tests for opencode-util

use opencode_util::{
    error::NamedError,
    fs::{atomic_write, ensure_dir, read_json, write_json},
    helpers::{wait_for, with_timeout, Lazy},
    iife,
    logging::{LogLevel, Logger},
    retry::{retry, RetryConfig},
};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::test]
async fn test_util_log_level_serde() {
    assert_eq!(
        serde_json::to_string(&LogLevel::Debug).unwrap(),
        "\"debug\""
    );
    assert_eq!(serde_json::to_string(&LogLevel::Info).unwrap(), "\"info\"");
    assert_eq!(serde_json::to_string(&LogLevel::Warn).unwrap(), "\"warn\"");
    assert_eq!(
        serde_json::to_string(&LogLevel::Error).unwrap(),
        "\"error\""
    );
}

#[tokio::test]
async fn test_util_retry_config_default() {
    let config = RetryConfig::default();
    assert_eq!(config.max_attempts, 3);
    assert_eq!(config.base_delay, Duration::from_millis(100));
    assert!(config.jitter);
}

#[tokio::test]
async fn test_util_retry_success() {
    let count = Arc::new(AtomicU32::new(0));
    let c = count.clone();
    let result = retry(RetryConfig::default(), move |_| {
        let c = c.clone();
        async move {
            c.fetch_add(1, Ordering::SeqCst);
            Ok::<_, ()>(42)
        }
    })
    .await;
    assert_eq!(result.unwrap(), 42);
}

#[tokio::test]
async fn test_util_retry_retries_and_succeeds() {
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
async fn test_util_retry_exhausted() {
    let result = retry(RetryConfig::new(2, Duration::from_millis(10)), |_| async {
        Err::<i32, ()>(())
    })
    .await;
    assert!(result.is_err());
    let (_, attempts) = result.unwrap_err();
    assert_eq!(attempts, 2);
}

#[tokio::test]
async fn test_util_atomic_write() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test.txt");
    atomic_write(&path, "hello world").await.unwrap();
    let contents = tokio::fs::read_to_string(&path).await.unwrap();
    assert_eq!(contents, "hello world");
}

#[tokio::test]
async fn test_util_read_json() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("config.json");
    let data = serde_json::json!({"name": "test", "version": 1});
    tokio::fs::write(&path, data.to_string()).await.unwrap();
    let decoded: serde_json::Value = read_json(&path).await.unwrap();
    assert_eq!(decoded["name"], "test");
}

#[tokio::test]
async fn test_util_write_json() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("config.json");
    let data = serde_json::json!({"name": "test", "version": 1});
    write_json(&path, &data).await.unwrap();
    let contents = tokio::fs::read_to_string(&path).await.unwrap();
    assert!(contents.contains('\n'));
}

#[tokio::test]
async fn test_util_ensure_dir() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("nested").join("deep").join("dir");
    ensure_dir(&path).await.unwrap();
    assert!(tokio::fs::metadata(&path).await.is_ok());
}

#[test]
fn test_util_lazy_thread_safe() {
    static LAZY: Lazy<u32> = Lazy::new(|| 42);
    assert_eq!(*LAZY.get(), 42);
}

#[test]
fn test_util_lazy_initializes_once() {
    static CALLED: AtomicU32 = AtomicU32::new(0);
    static LAZY: Lazy<u32> = Lazy::new(|| {
        CALLED.fetch_add(1, Ordering::SeqCst);
        42
    });
    assert_eq!(*LAZY.get(), 42);
    assert_eq!(*LAZY.get(), 42);
    assert_eq!(CALLED.load(Ordering::SeqCst), 1);
}

#[test]
fn test_util_iife_macro() {
    let value = iife!(|| 1 + 2);
    assert_eq!(value, 3);
}

#[tokio::test]
async fn test_util_with_timeout_success() {
    let result = with_timeout(Duration::from_secs(1), async { 42 }).await;
    assert_eq!(result.unwrap(), 42);
}

#[tokio::test]
async fn test_util_with_timeout_fires() {
    let result = with_timeout(Duration::from_millis(10), async {
        tokio::time::sleep(Duration::from_secs(1)).await;
    })
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_util_wait_for_success() {
    let counter = Arc::new(AtomicU32::new(0));
    let c = counter.clone();
    let result = wait_for(
        move || {
            let c = c.clone();
            let val = c.fetch_add(1, Ordering::SeqCst);
            if val >= 1 {
                Some("ready".to_string())
            } else {
                None
            }
        },
        Duration::from_secs(1),
    )
    .await;
    assert_eq!(result.unwrap(), "ready");
}

#[tokio::test]
async fn test_util_wait_for_timeout() {
    let result = wait_for(|| None::<String>, Duration::from_millis(50)).await;
    assert!(result.is_err());
}

#[test]
fn test_util_named_error_new() {
    let err = NamedError::new("TestError", "test message");
    assert_eq!(err.name, "TestError");
    assert_eq!(err.message, "test message");
    assert_eq!(err.code, None);
    assert_eq!(err.data, None);
}

#[test]
fn test_util_named_error_with_code() {
    let err = NamedError::new("TestError", "test message").with_code("CODE_001");
    assert_eq!(err.code, Some("CODE_001".to_string()));
}

#[test]
fn test_util_named_error_with_data() {
    let data = serde_json::json!({"key": "value"});
    let err = NamedError::new("TestError", "test message").with_data(data.clone());
    assert_eq!(err.data, Some(data));
}

#[test]
fn test_util_named_error_display() {
    let err = NamedError::new("IOError", "file not found");
    assert_eq!(format!("{}", err), "IOError: file not found");
}

#[test]
fn test_util_named_error_kind() {
    let err = NamedError::new("ToolNotFound", "tool not found");
    assert_eq!(err.kind(), "ToolNotFound");
}

#[test]
fn test_util_named_error_serde() {
    let err = NamedError::new("TestError", "test message")
        .with_code("TEST_001")
        .with_data(serde_json::json!({"field": "value"}));
    let json = serde_json::to_string(&err).unwrap();
    let decoded: NamedError = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.name, "TestError");
    assert_eq!(decoded.code, Some("TEST_001".to_string()));
    assert!(decoded.data.is_some());
}
