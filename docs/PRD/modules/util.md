# PRD: util Module

## Module Overview

- **Module Name**: `util`
- **Source Path**: `packages/opencode/src/util/`
- **Type**: Utility
- **Rust Crate**: `crates/util/` (split into submodules: `logging`, `error`, `helpers`)
- **Purpose**: General utilities — structured logging, error types, retry helpers, and filesystem helpers used throughout the application.

---

## Functionality

### Core Features

1. **Structured Logging** — `tracing`-based logging with levels (DEBUG, INFO, WARN, ERROR), file output with rotation, console output control
2. **Error Types** — `NamedError` equivalent with code, context, and data fields; `From` implementations for ergonomic error propagation
3. **Filesystem Helpers** — Async file read/write, directory operations, path utilities
4. **Retry Utilities** — Configurable retry with exponential backoff and jitter
5. **Lazy Evaluation** — Once-cell and lazy static patterns
6. **Async Helpers** — `wait_for`, `retry_until`, `timeout` wrappers

---

## Module Structure

```
crates/util/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Re-exports
│   ├── logging.rs       # Logging setup and helpers
│   ├── error.rs         # Error types (NamedError, ErrorContext)
│   ├── retry.rs         # Retry utilities
│   ├── fs.rs            # Filesystem helpers
│   └── helpers.rs       # iife, lazy, misc helpers
└── tests/
    └── util_tests.rs
```

---

## 1. Logging (`logging.rs`)

### Log Levels

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}
```

### Logger Setup

```rust
pub struct Logger {
    level: LogLevel,
    file_path: Option<PathBuf>,
    console: bool,
}

impl Logger {
    /// Initialize the global logger. Call once at startup.
    pub fn init(&self) -> Result<(), SetLoggerError>

    /// Set log level filter (e.g., LogLevel::Info)
    pub fn with_level(level: LogLevel) -> Self

    /// Enable file output with rotation
    pub fn with_file(&mut self, path: impl Into<PathBuf>) -> &mut Self

    /// Disable console output
    pub fn with_no_console(&mut self) -> &mut Self
}
```

### `tracing` Integration

The `util` logging module wraps `tracing` with OpenCode-specific formatting:

```rust
// Uses tracing crate under the hood
// Format: "[2024-01-15 10:30:45] [INFO] module: message"
// Structured fields are serialized as JSON: { "key": "value" }

tracing::info!(target: "opencode::agent", "Tool executed: {}", tool_name);
tracing::debug!(target: "opencode::llm", latency_ms = 42, "Request completed");
```

### Log File Rotation

```rust
/// Log file rotation based on size
pub struct Rotation {
    max_size_bytes: usize,
    max_files: usize,
}

impl Rotation {
    pub fn new(max_size_mb: usize, max_files: usize) -> Self
}

/// OpenCode log file location
pub fn log_file_path() -> PathBuf {
    Global::path_opencode().join("logs").join("opencode.log")
}
```

---

## 2. Error Types (`error.rs`)

### `NamedError` Equivalent

```rust
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub struct NamedError {
    pub name: String,
    pub code: Option<String>,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl NamedError {
    pub fn new(name: impl Into<String>, message: impl Into<String>) -> Self
    pub fn with_code(mut self, code: impl Into<String>) -> Self
    pub fn with_data(mut self, data: serde_json::Value) -> Self
    pub fn kind(&self) -> &str { &self.name }
}

impl std::fmt::Display for NamedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.message)
    }
}

impl From<std::io::Error> for NamedError {
    fn from(e: std::io::Error) -> Self {
        NamedError::new("IOError", e.to_string())
            .with_code(format!("IO_{}", e.kind() as i32))
    }
}

impl From<reqwest::Error> for NamedError {
    fn from(e: reqwest::Error) -> Self {
        NamedError::new("HttpError", e.to_string())
            .with_code("HTTP")
    }
}
```

### Error Context (`WithContext`)

```rust
/// Wrap any error with additional context
pub struct WithContext<E> {
    context: String,
    inner: E,
}

impl<E: std::error::Error> std::fmt::Display for WithContext<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} (caused by: {})", self.context, self.inner, self.inner.source().map(|s| s.to_string()).unwrap_or_default())
    }
}

impl<E: std::error::Error> std::error::Error for WithContext<E> {}

pub trait Context<T, E: std::error::Error> {
    fn context<C: Into<String>>(self, ctx: C) -> Result<T, WithContext<E>>;
}

impl<T, E: std::error::Error> Context<T, E> for Result<T, E> {
    fn context<C: Into<String>>(self, ctx: C) -> Result<T, WithContext<E>> {
        self.map_err(|e| WithContext { context: ctx.into(), inner: e })
    }
}
```

### Usage

```rust
// Simple usage
let result = tokio::fs::read_to_string("config.toml")
    .map_err(|e| NamedError::from(e).with_code("READ_CONFIG")?)?;

// With context
let session = session_store.get(&id)
    .await
    .context("Failed to load session from storage")?;
```

---

## 3. Retry Utilities (`retry.rs`)

### Retry Config

```rust
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter: bool,    // Add random jitter to avoid thundering herd
}

impl RetryConfig {
    pub fn new(max_attempts: u32, base_delay: Duration) -> Self {
        Self { max_attempts, base_delay, max_delay: Duration::MAX, jitter: true }
    }

    pub fn with_max_delay(mut self, max: Duration) -> Self {
        self.max_delay = max;
        self
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::new(3, Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(10))
    }
}
```

### Retry Function

```rust
/// Retry an async operation with exponential backoff
pub async fn retry<R, E, F, Fut>(
    config: RetryConfig,
    operation: F,
) -> Result<R, (E, u32)> // (last_error, attempts_made)
where
    F: Fn(u32) -> Fut,  // attempt number → future
    Fut: Future<Output = Result<R, E>>,
    E: std::fmt::Debug,
{
    let mut attempts = 0;
    loop {
        match operation(attempts).await {
            Ok(v) => return Ok(v),
            Err(e) => {
                attempts += 1;
                if attempts >= config.max_attempts {
                    return Err((e, attempts));
                }
                let delay = next_delay(&config, attempts);
                tokio::time::sleep(delay).await;
            }
        }
    }
}

fn next_delay(config: &RetryConfig, attempt: u32) -> Duration {
    let exp = 2u32.saturating_pow(attempt.min(10));
    let delay = config.base_delay * exp;
    let delay = delay.min(config.max_delay);
    if config.jitter {
        let jitter = (rand::random::<u64>() % 100) as u64;
        delay + Duration::from_millis(jitter)
    } else {
        delay
    }
}
```

---

## 4. Filesystem Helpers (`fs.rs`)

```rust
/// Read file to string, creating parent dirs if needed
pub async fn read_to_string(path: &Path) -> Result<String, std::io::Error>

/// Write string to file, creating parent dirs if needed
pub async fn write(path: &Path, contents: &str) -> Result<(), std::io::Error>

/// Atomic write: write to temp file, then rename (atomic on most filesystems)
pub async fn atomic_write(path: &Path, contents: &str) -> Result<(), std::io::Error>

/// Ensure directory exists
pub async fn ensure_dir(path: &Path) -> Result<(), std::io::Error>

/// Read JSON with error context
pub async fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, NamedError>

/// Write JSON with pretty printing
pub async fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), NamedError>
```

---

## 5. Helpers (`helpers.rs`)

```rust
/// Lazily evaluated once cell
pub struct Lazy<T> {
    cell: OnceCell<T>,
    init: fn() -> T,
}

impl<T: Send + Sync> Lazy<T> {
    pub const fn new(init: fn() -> T) -> Self {
        Self { cell: OnceCell::new(), init }
    }
    pub fn get(&self) -> &T {
        self.cell.get_or_init(|| (self.init)())
    }
}

/// IIFE (Immediately Invoked Function Expression)
#[macro_export]
macro_rules! iife {
    (|| $expr:expr) => { $expr };
    (||$($tokens:tt)*) => { ($( $tokens )*) };
}

/// Run a future with a timeout
pub async fn with_timeout<T>(
    duration: Duration,
    future: impl Future<Output = T>,
) -> Result<T, TimeoutError> {
    tokio::time::timeout(duration, future).await.map_err(|_| TimeoutError)
}
```

---

## Dependencies

```toml
[package]
name = "opencode-util"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.45", features = ["fs", "time", "sync", "rt"] }
tracing = "0.1"
tracing-appender = "0.2"
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rand = "0.8"
anyhow = "1.0"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
walkdir = "2"

[dev-dependencies]
tracing-test = "0.2"
tempfile = "3"
```

---

## Acceptance Criteria

- [ ] Logger can be initialized with level filter, file output, and console toggle
- [ ] Log files rotate when max size is reached
- [ ] `NamedError` implements `Display`, `Error`, `Serialize`, `Deserialize`
- [ ] `From<std::io::Error>` and `From<reqwest::Error>` work for `NamedError`
- [ ] `retry()` correctly retries with exponential backoff + jitter
- [ ] `retry()` returns last error after `max_attempts`
- [ ] `atomic_write()` writes to temp then renames
- [ ] `read_json()` and `write_json()` work for any `Serialize`/`DeserializeOwned` type
- [ ] `with_timeout()` returns `TimeoutError` when duration elapses
- [ ] `Lazy<T>` is thread-safe and only initializes once

---

## Test Design

```rust
#[test]
fn test_named_error_display() {
    let e = NamedError::new("ToolNotFound", "Tool 'foo' not found")
        .with_code("TOOL_404")
        .with_data(json!({"tool": "foo"}));
    assert_eq!(e.name, "ToolNotFound");
    assert_eq!(e.code, Some("TOOL_404".to_string()));
}

#[test]
fn test_named_error_from_io_error() {
    let e: NamedError = std::io::Error::new(std::io::ErrorKind::NotFound, "file.txt").into();
    assert_eq!(e.name, "IOError");
}

#[tokio::test]
async fn test_retry_succeeds_on_first_attempt() {
    let count = Arc::new(AtomicU32::new(0));
    let c = count.clone();
    let result = retry(RetryConfig::default(), |_| {
        let c = c.clone();
        async move {
            c.fetch_add(1, SeqCst);
            Ok::<_, ()>(42)
        }
    }).await.unwrap();
    assert_eq!(result, 42);
    assert_eq!(count.load(SeqCst), 1);
}

#[tokio::test]
async fn test_retry_retries_on_failure() {
    let count = Arc::new(AtomicU32::new(0));
    let c = count.clone();
    let result = retry(RetryConfig::new(3, Duration::from_millis(10)), |attempt| {
        let c = c.clone();
        async move {
            c.fetch_add(1, SeqCst);
            if attempt < 2 { Err(()) } else { Ok(42) }
        }
    }).await.unwrap();
    assert_eq!(result, 42);
    assert_eq!(count.load(SeqCst), 3);
}

#[tokio::test]
async fn test_retry_returns_err_after_max_attempts() {
    let result = retry(RetryConfig::new(2, Duration::from_millis(10)), |_| {
        async move { Err::<i32, ()>(()) }
    }).await;
    assert!(result.is_err());
    let (_, attempts) = result.unwrap_err();
    assert_eq!(attempts, 2);
}

#[tokio::test]
async fn test_timeout_fires() {
    let result = with_timeout(Duration::from_millis(10), async {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_atomic_write() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("atomic.txt");
    atomic_write(&path, "hello").await.unwrap();
    assert_eq!(tokio::fs::read_to_string(&path).await.unwrap(), "hello");
}
```

---

## Source Reference

*Source: `packages/opencode/src/util/index.ts`*
*No existing Rust equivalent — implement in `crates/util/`*
