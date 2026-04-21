# Specification: Module `util` (Iteration 41)

**Crate**: `opencode-util` (new crate at `opencode-rust/crates/util/`)
**Source**: `opencode-rust/crates/util/src/`
**Visibility**: `pub` — public utility crate
**Status**: 🔴 **NOT IMPLEMENTED** — Module does not exist
**Last Updated**: 2026-04-21
**Gap Score**: 100% (All features missing - P0/P1/P2 items identified)

---

## 1. Overview

The `util` module provides general-purpose utilities for the OpenCode RS application, including:
- **Structured Logging** — `tracing`-based logging with levels, file output, and rotation
- **Error Types** — `NamedError` struct with context and data fields; `WithContext` error wrapper
- **Retry Utilities** — Configurable retry with exponential backoff and jitter
- **Filesystem Helpers** — Async file operations, atomic writes, JSON read/write
- **Async Helpers** — `with_timeout`, `wait_for`, `retry_until` utilities
- **Lazy Evaluation** — Thread-safe `Lazy<T>` and `iife!` macro

**Design Principles**:
- Async-first: All filesystem and retry operations are async
- Structured errors: `NamedError` provides serialization-friendly error representation
- Composable: `WithContext` allows adding context to any error type
- Production-ready: Logging includes file rotation and JSON structured output

---

## 2. Module Structure

```
opencode-rust/crates/util/
├── Cargo.toml              # Crate definition with dependencies
├── src/
│   ├── lib.rs              # Re-exports all public types
│   ├── logging.rs          # Logger, LogLevel, Rotation
│   ├── error.rs            # NamedError, WithContext
│   ├── retry.rs            # RetryConfig, retry()
│   ├── fs.rs               # atomic_write, read_json, write_json
│   └── helpers.rs          # Lazy, iife!, with_timeout
└── tests/
    └── util_tests.rs       # Integration tests
```

**Exports** (from `src/lib.rs`):
```rust
pub use logging::{Logger, LogLevel, Rotation, log_file_path};
pub use error::{NamedError, WithContext, Context};
pub use retry::{RetryConfig, retry};
pub use fs::{atomic_write, read_json, write_json, ensure_dir};
pub use helpers::{Lazy, with_timeout, wait_for, retry_until};
pub use crate::iife;
```

**Dependencies** (from `Cargo.toml`):
```toml
[dependencies]
tokio = { version = "1.45", features = ["fs", "time", "sync"] }
tracing = "0.1"
tracing-appender = "0.2"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rand = "0.8"
anyhow = "1.0"
walkdir = "2"

[dev-dependencies]
tempfile = "3"
tokio-test = "0.4"
```

---

## 3. Logging Module (`logging.rs`)

### FR-422: LogLevel Enum

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

**Description**: Log level filter for the global logger.

**Variants**:
| Variant | Description |
|---------|-------------|
| `Debug` | Detailed debugging information |
| `Info` | General informational messages |
| `Warn` | Warning conditions |
| `Error` | Error conditions |

**Derived Traits**: `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`
**Serialized As**: lowercase string (`"debug"`, `"info"`, `"warn"`, `"error"`)

---

### FR-423: Rotation Struct

```rust
pub struct Rotation {
    max_size_bytes: usize,
    max_files: usize,
}

impl Rotation {
    pub fn new(max_size_mb: usize, max_files: usize) -> Self
}
```

**Description**: Log file rotation configuration based on size.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `max_size_bytes` | `usize` | Maximum size per log file before rotation |
| `max_files` | `usize` | Maximum number of rotated log files to retain |

**Example**:
```rust
let rotation = Rotation::new(10, 5); // 10MB max, keep 5 files
```

---

### FR-424: Logger Struct

```rust
pub struct Logger {
    level: LogLevel,
    file_path: Option<PathBuf>,
    console: bool,
}
```

**Description**: Global logger configuration with fluent builder API.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `level` | `LogLevel` | Minimum log level to output |
| `file_path` | `Option<PathBuf>` | Optional file path for log output |
| `console` | `bool` | Whether to output to console (stdout) |

---

### FR-425: Logger::init()

```rust
impl Logger {
    pub fn init(&self) -> Result<(), SetLoggerError>
}
```

**Description**: Initialize the global logger. Call once at startup.

**Returns**: `Result<(), SetLoggerError>` — error if logger already initialized

**Behavior**:
- Sets the global `tracing` subscriber
- Applies level filter from `self.level`
- If `file_path` is set, enables file output with rotation
- If `console` is false, disables stdout/stderr output

---

### FR-426: Logger::with_level()

```rust
impl Logger {
    pub fn with_level(level: LogLevel) -> Self
}
```

**Description**: Create a Logger with specified level filter.

**Returns**: `Logger` — builder pattern instance

**Example**:
```rust
Logger::with_level(LogLevel::Info).init().unwrap();
```

---

### FR-427: Logger::with_file()

```rust
impl Logger {
    pub fn with_file(&mut self, path: impl Into<PathBuf>) -> &mut Self
}
```

**Description**: Enable file output with the specified path.

**Parameters**:
- `path`: File path for log output (can be directory or full path)

**Behavior**: Sets `file_path`, enabling file-based logging with rotation

---

### FR-428: Logger::with_no_console()

```rust
impl Logger {
    pub fn with_no_console(&mut self) -> &mut Self
}
```

**Description**: Disable console output.

**Behavior**: Sets `console = false`, suppressing stdout/stderr logging

---

### FR-429: log_file_path()

```rust
pub fn log_file_path() -> PathBuf
```

**Description**: Returns the default OpenCode log file path.

**Returns**: `PathBuf` — `Global::path_opencode().join("logs").join("opencode.log")`

**Note**: Depends on `Global::path_opencode()` from `opencode-core`

---

### FR-430: tracing Integration

The logging module wraps `tracing` with OpenCode-specific formatting:

```rust
// Format: "[2024-01-15 10:30:45] [INFO] module: message"
// Structured fields are serialized as JSON: { "key": "value" }

tracing::info!(target: "opencode::agent", "Tool executed: {}", tool_name);
tracing::debug!(target: "opencode::llm", latency_ms = 42, "Request completed");
```

**Targets**:
| Target | Description |
|--------|-------------|
| `opencode::agent` | Agent execution events |
| `opencode::llm` | LLM provider requests/responses |
| `opencode::tool` | Tool invocation events |
| `opencode::session` | Session management events |
| `opencode::error` | Error conditions |

---

## 4. Error Types Module (`error.rs`)

### FR-431: NamedError Struct

```rust
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
```

**Description**: Unified error type with name, code, message, and optional data.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Error type name (e.g., `"IOError"`, `"HttpError"`) |
| `code` | `Option<String>` | Error code for programmatic handling |
| `message` | `String` | Human-readable error message |
| `data` | `Option<serde_json::Value>` | Additional structured context |

**Derived Traits**: `Debug`, `Clone`, `Error`, `Serialize`, `Deserialize`

**Builder Methods**:
| Method | Description |
|--------|-------------|
| `new(name, message)` | Create a new NamedError |
| `with_code(code)` | Add error code |
| `with_data(data)` | Add structured data |

---

### FR-432: From<std::io::Error> for NamedError

```rust
impl From<std::io::Error> for NamedError {
    fn from(e: std::io::Error) -> Self {
        NamedError::new("IOError", e.to_string())
            .with_code(format!("IO_{}", e.kind() as i32))
    }
}
```

**Description**: Convert `std::io::Error` to `NamedError`.

**Error Code Format**: `IO_{kind_as_i32}` (e.g., `IO_2` for `NotFound`)

---

### FR-433: From<reqwest::Error> for NamedError

```rust
impl From<reqwest::Error> for NamedError {
    fn from(e: reqwest::Error) -> Self {
        NamedError::new("HttpError", e.to_string())
            .with_code("HTTP")
    }
}
```

**Description**: Convert `reqwest::Error` to `NamedError`.

**Error Code**: `HTTP`

---

### FR-434: WithContext Struct

```rust
pub struct WithContext<E> {
    context: String,
    inner: E,
}

impl<E: std::error::Error> std::fmt::Display for WithContext<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} (caused by: {})",
            self.context,
            self.inner,
            self.inner.source().map(|s| s.to_string()).unwrap_or_default()
        )
    }
}

impl<E: std::error::Error> std::error::Error for WithContext<E> {}
```

**Description**: Wraps any error with additional context.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `context` | `String` | Additional context describing where/why the error occurred |
| `inner` | `E` | The underlying error |

**Display Format**: `{context}: {inner} (caused by: {source})`

---

### FR-435: Context Trait

```rust
pub trait Context<T, E: std::error::Error> {
    fn context<C: Into<String>>(self, ctx: C) -> Result<T, WithContext<E>>;
}

impl<T, E: std::error::Error> Context<T, E> for Result<T, E> {
    fn context<C: Into<String>>(self, ctx: C) -> Result<T, WithContext<E>> {
        self.map_err(|e| WithContext {
            context: ctx.into(),
            inner: e,
        })
    }
}
```

**Description**: Extension trait for adding context to `Result<T, E>`.

**Usage**:
```rust
let session = session_store.get(&id)
    .await
    .context("Failed to load session from storage")?;
```

---

### FR-436: NamedError Usage Examples

```rust
// Simple error
let err = NamedError::new("ToolNotFound", "Tool 'foo' not found");
assert_eq!(err.name, "ToolNotFound");
assert_eq!(err.message, "Tool 'foo' not found");

// With code and data
let err = NamedError::new("ValidationError", "Invalid input")
    .with_code("VALIDATION_001")
    .with_data(json!({"field": "email", "reason": "invalid format"}));
assert_eq!(err.code, Some("VALIDATION_001".to_string()));

// From io::Error
let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file.txt");
let named: NamedError = io_err.into();
assert_eq!(named.name, "IOError");
assert!(named.code.is_some());

// From reqwest::Error
let req_err = reqwest::Error::new(/* ... */);
let named: NamedError = req_err.into();
assert_eq!(named.name, "HttpError");
```

---

## 5. Retry Module (`retry.rs`)

### FR-437: RetryConfig Struct

```rust
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter: bool,  // Add random jitter to avoid thundering herd
}

impl RetryConfig {
    pub fn new(max_attempts: u32, base_delay: Duration) -> Self {
        Self {
            max_attempts,
            base_delay,
            max_delay: Duration::MAX,
            jitter: true,
        }
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

**Description**: Configuration for retry behavior with exponential backoff.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `max_attempts` | `u32` | Maximum number of retry attempts |
| `base_delay` | `Duration` | Initial delay between retries |
| `max_delay` | `Duration` | Maximum delay cap |
| `jitter` | `bool` | Whether to add random jitter |

**Builder Methods**:
| Method | Description |
|--------|-------------|
| `new(max_attempts, base_delay)` | Create config with max attempts and base delay |
| `with_max_delay(max)` | Set maximum delay cap |

---

### FR-438: retry() Function

```rust
pub async fn retry<R, E, F, Fut>(
    config: RetryConfig,
    operation: F,
) -> Result<R, (E, u32)>
where
    F: Fn(u32) -> Fut,  // attempt number → future
    Fut: Future<Output = Result<R, E>>,
    E: std::fmt::Debug,
```

**Description**: Retry an async operation with exponential backoff and optional jitter.

**Parameters**:
| Parameter | Type | Description |
|-----------|------|-------------|
| `config` | `RetryConfig` | Retry configuration |
| `operation` | `F: Fn(u32) -> Fut` | Async operation to retry, receives attempt number |

**Returns**: `Result<R, (E, u32)>` — `Ok(value)` on success, `Err((error, attempts_made))` on exhaustion

**Delay Calculation**:
```
delay = min(base_delay * 2^attempt, max_delay)
if jitter:
    delay += random(0..100ms)
```

---

### FR-439: RetryConfig Default Values

| Field | Default Value | Description |
|-------|---------------|-------------|
| `max_attempts` | `3` | Retry up to 3 times |
| `base_delay` | `100ms` | Start with 100ms delay |
| `max_delay` | `10s` | Cap delay at 10 seconds |
| `jitter` | `true` | Add random jitter |

---

### FR-440: retry() Usage Examples

```rust
use std::time::Duration;
use opencode_util::retry::{RetryConfig, retry};
use std::sync::atomic::{AtomicU32, Ordering::SeqCst};
use std::sync::Arc;

// Success on first attempt
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

// Retry with backoff
let count = Arc::new(AtomicU32::new(0));
let c = count.clone();
let result = retry(
    RetryConfig::new(3, Duration::from_millis(10)),
    |attempt| {
        let c = c.clone();
        async move {
            c.fetch_add(1, SeqCst);
            if attempt < 2 { Err(()) } else { Ok(42) }
        }
    }
).await.unwrap();
assert_eq!(result, 42);
assert_eq!(count.load(SeqCst), 3);

// Return last error after max attempts
let result = retry(
    RetryConfig::new(2, Duration::from_millis(10)),
    |_| async { Err::<i32, ()>(()) }
).await;
assert!(result.is_err());
let (_, attempts) = result.unwrap_err();
assert_eq!(attempts, 2);
```

---

## 6. Filesystem Module (`fs.rs`)

### FR-441: read_to_string()

```rust
pub async fn read_to_string(path: &Path) -> Result<String, std::io::Error>
```

**Description**: Read file contents to string asynchronously, creating parent dirs if needed.

**Parameters**:
- `path`: Path to file to read

**Returns**: `Result<String, std::io::Error>`

---

### FR-442: write()

```rust
pub async fn write(path: &Path, contents: &str) -> Result<(), std::io::Error>
```

**Description**: Write string to file asynchronously, creating parent dirs if needed.

**Parameters**:
- `path`: Path to file to write
- `contents`: String content to write

**Returns**: `Result<(), std::io::Error>`

---

### FR-443: atomic_write()

```rust
pub async fn atomic_write(path: &Path, contents: &str) -> Result<(), std::io::Error>
```

**Description**: Atomically write to file using temp file + rename pattern.

**Behavior**:
1. Write content to temporary file in same directory
2. Rename temp file to target path (atomic on most filesystems)
3. On failure, temp file is not created/left behind

**Parameters**:
- `path`: Target file path
- `contents`: String content to write

**Returns**: `Result<(), std::io::Error>`

---

### FR-444: ensure_dir()

```rust
pub async fn ensure_dir(path: &Path) -> Result<(), std::io::Error>
```

**Description**: Ensure directory exists, creating it and parent directories if needed.

**Parameters**:
- `path`: Directory path to ensure

**Returns**: `Result<(), std::io::Error>`

---

### FR-445: read_json()

```rust
pub async fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, NamedError>
```

**Description**: Read and deserialize JSON file asynchronously.

**Parameters**:
- `path`: Path to JSON file

**Returns**: `Result<T, NamedError>` — deserialized value or error

**Error**: `NamedError` with kind `"JsonError"` on parse failure

---

### FR-446: write_json()

```rust
pub async fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), NamedError>
```

**Description**: Serialize and write JSON file with pretty printing asynchronously.

**Parameters**:
- `path`: Path to JSON file
- `value`: Value to serialize and write

**Returns**: `Result<(), NamedError>`

**Behavior**: Uses `serde_json::to_string_pretty()` for formatted output

---

### FR-447: fs Module Usage Examples

```rust
use std::path::Path;
use opencode_util::fs::{atomic_write, read_json, write_json, ensure_dir};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Config {
    name: String,
    version: u32,
}

// Atomic write
let path = Path::new("/tmp/test.txt");
atomic_write(path, "hello world").await.unwrap();

// Read JSON
let config: Config = read_json(path).await.unwrap();

// Write JSON
let config = Config { name: "test".to_string(), version: 1 };
write_json(path, &config).await.unwrap();

// Ensure directory
let dir = Path::new("/tmp/nested/deep/dir");
ensure_dir(dir).await.unwrap();
```

---

## 7. Helpers Module (`helpers.rs`)

### FR-448: Lazy<T> Struct

```rust
use std::sync::OnceCell;

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
```

**Description**: Lazily evaluated once cell, thread-safe, only initializes once.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `cell` | `OnceCell<T>` | Thread-safe once cell storage |
| `init` | `fn() -> T` | Initialization function |

**Behavior**:
- `get()` returns reference to value, initializing on first call
- Initialization function called exactly once, even in concurrent access
- Thread-safe via `OnceCell`

---

### FR-449: iife! Macro

```rust
#[macro_export]
macro_rules! iife {
    (|| $expr:expr) => { $expr };
    (||$($tokens:tt)*) => { ($( $tokens )*) };
}
```

**Description**: Immediately Invoked Function Expression macro.

**Usage**:
```rust
// Simple expression
let value = iife!(|| compute_initial_value());

// Block expression
let value = iife!(|| {
    let x = 1;
    let y = 2;
    x + y
});
```

---

### FR-450: with_timeout()

```rust
pub struct TimeoutError;

pub async fn with_timeout<T>(
    duration: Duration,
    future: impl Future<Output = T>,
) -> Result<T, TimeoutError> {
    tokio::time::timeout(duration, future).await.map_err(|_| TimeoutError)
}
```

**Description**: Run a future with a timeout, returning `TimeoutError` on timeout.

**Parameters**:
- `duration`: Maximum duration to wait
- `future`: Future to execute

**Returns**: `Result<T, TimeoutError>` — value on success, `TimeoutError` on timeout

---

### FR-451: wait_for()

```rust
pub async fn wait_for<F, T>(
    condition: F,
    timeout: Duration,
) -> Result<T, TimeoutError>
where
    F: Fn() -> Option<T>,
```

**Description**: Poll a condition until it returns `Some(T)` or timeout.

**Parameters**:
- `condition`: Closure returning `Option<T>`, `None` means keep waiting
- `timeout`: Maximum duration to wait

**Returns**: `Result<T, TimeoutError>` — value when condition met, `TimeoutError` on timeout

**Behavior**: Polls every 100ms by default

---

### FR-452: retry_until()

```rust
pub async fn retry_until<F, T, E>(
    config: RetryConfig,
    condition: F,
) -> Result<T, (E, u32)>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<Option<T>, E>>,
    E: std::fmt::Debug,
```

**Description**: Retry until condition returns `Ok(Some(T))` or `Err(E)`.

**Parameters**:
- `config`: Retry configuration
- `condition`: Closure returning `Result<Option<T>, E>`, `Ok(None)` means retry

**Returns**: `Result<T, (E, u32)>` — value when condition met, last error on exhaustion

---

### FR-453: helpers Usage Examples

```rust
use std::time::Duration;
use opencode_util::helpers::{Lazy, with_timeout, wait_for, retry_until};
use opencode_util::RetryConfig;

// Lazy evaluation
static CONFIG: Lazy<String> = Lazy::new(|| {
    println!("Initializing CONFIG");
    "initialized".to_string()
});
assert_eq!(*CONFIG.get(), "initialized"); // Only prints once

// Timeout
let result = with_timeout(Duration::from_millis(10), async {
    tokio::time::sleep(Duration::from_secs(1)).await;
}).await;
assert!(result.is_err()); // TimeoutError

// Wait for condition
let result = wait_for(|| {
    if false { Some("ready") } else { None }
}, Duration::from_millis(50)).await;
assert!(result.is_err()); // TimeoutError

// Retry until
let count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
let c = count.clone();
let result = retry_until(
    RetryConfig::default(),
    || {
        let c = c.clone();
        async move {
            let prev = c.fetch_add(1, SeqCst);
            if prev < 2 { Ok(None) } else { Ok(Some("ready")) }
        }
    }
).await.unwrap();
assert_eq!(result, "ready");
```

---

## 8. Dependencies Management

### FR-454: Required Crate Dependencies

Add to workspace `Cargo.toml`:

```toml
[workspace.dependencies]
# Add to existing tracing entries
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-appender = "0.2"
```

Create `crates/util/Cargo.toml`:

```toml
[package]
name = "opencode-util"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.45", features = ["fs", "time", "sync"] }
tracing = "0.1"
tracing-appender = "0.2"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rand = "0.8"
anyhow = "1.0"
walkdir = "2"

[dev-dependencies]
tempfile = "3"
tokio-test = "0.4"
```

### FR-455: Workspace Integration

Add to `opencode-rust/Cargo.toml` members:
```toml
members = [
    # ... existing crates ...
    "crates/util",
]
```

Add internal dependency:
```toml
opencode-util = { path = "crates/util" }
```

---

## 9. Acceptance Criteria

### FR-456: Logging Acceptance Criteria

| ID | Criterion | Test |
|----|-----------|------|
| FR-456.1 | Logger can be initialized with level filter | `Logger::with_level(LogLevel::Info).init()` succeeds |
| FR-456.2 | Log files rotate when max size is reached | `Rotation::new(1, 3)` creates rotation config |
| FR-456.3 | `log_file_path()` returns valid path | Path ends with `logs/opencode.log` |
| FR-456.4 | `LogLevel` serializes to lowercase | `serde_json::to_string(&LogLevel::Debug)` → `"debug"` |
| FR-456.5 | Console can be disabled | `Logger::with_no_console()` suppresses stdout |

### FR-457: Error Types Acceptance Criteria

| ID | Criterion | Test |
|----|-----------|------|
| FR-457.1 | `NamedError` implements `Display`, `Error`, `Serialize`, `Deserialize` | All traits compile and work |
| FR-457.2 | `From<std::io::Error>` works for `NamedError` | IO error converts with `IO_*` code |
| FR-457.3 | `From<reqwest::Error>` works for `NamedError` | HTTP error converts with `HTTP` code |
| FR-457.4 | `WithContext` wraps errors with context | Display includes context string |
| FR-457.5 | `Context::context()` method adds context | `.context("msg")` wraps error |

### FR-458: Retry Acceptance Criteria

| ID | Criterion | Test |
|----|-----------|------|
| FR-458.1 | `retry()` correctly retries with exponential backoff + jitter | 3 retries on failure with increasing delays |
| FR-458.2 | `retry()` returns last error after `max_attempts` | `Err((e, 3))` after 3 failed attempts |
| FR-458.3 | `RetryConfig` default values are sensible | 3 attempts, 100ms base, 10s max |

### FR-459: Filesystem Acceptance Criteria

| ID | Criterion | Test |
|----|-----------|------|
| FR-459.1 | `atomic_write()` writes to temp then renames | File appears atomically |
| FR-459.2 | `read_json()` deserializes JSON file | Returns parsed value |
| FR-459.3 | `write_json()` serializes with pretty print | Output is formatted |
| FR-459.4 | `ensure_dir()` creates nested directories | Deep path created |

### FR-460: Helpers Acceptance Criteria

| ID | Criterion | Test |
|----|-----------|------|
| FR-460.1 | `with_timeout()` returns `TimeoutError` when duration elapses | Timeout triggers |
| FR-460.2 | `Lazy<T>` is thread-safe and only initializes once | Concurrent access safe |
| FR-460.3 | `iife!` macro evaluates immediately | `iife!(|| 1 + 2)` returns 3 |

---

## 10. Test Specification

### FR-461: Test Coverage Matrix

| Test ID | Test Name | Module | Status |
|---------|-----------|--------|--------|
| FR-461.1 | `test_named_error_display` | error | Not implemented |
| FR-461.2 | `test_named_error_with_code` | error | Not implemented |
| FR-461.3 | `test_named_error_with_data` | error | Not implemented |
| FR-461.4 | `test_named_error_serde` | error | Not implemented |
| FR-461.5 | `test_named_error_from_io_error` | error | Not implemented |
| FR-461.6 | `test_named_error_from_reqwest_error` | error | Not implemented |
| FR-461.7 | `test_with_context_display` | error | Not implemented |
| FR-461.8 | `test_context_trait_extension` | error | Not implemented |
| FR-461.9 | `test_retry_config_default` | retry | Not implemented |
| FR-461.10 | `test_retry_succeeds_on_first_attempt` | retry | Not implemented |
| FR-461.11 | `test_retry_retries_on_failure` | retry | Not implemented |
| FR-461.12 | `test_retry_returns_err_after_max_attempts` | retry | Not implemented |
| FR-461.13 | `test_retry_with_jitter` | retry | Not implemented |
| FR-461.14 | `test_atomic_write` | fs | Not implemented |
| FR-461.15 | `test_read_json` | fs | Not implemented |
| FR-461.16 | `test_write_json` | fs | Not implemented |
| FR-461.17 | `test_ensure_dir` | fs | Not implemented |
| FR-461.18 | `test_lazy_thread_safe` | helpers | Not implemented |
| FR-461.19 | `test_lazy_initializes_once` | helpers | Not implemented |
| FR-461.20 | `test_iife_macro` | helpers | Not implemented |
| FR-461.21 | `test_with_timeout_fires` | helpers | Not implemented |
| FR-461.22 | `test_with_timeout_success` | helpers | Not implemented |
| FR-461.23 | `test_wait_for_success` | helpers | Not implemented |
| FR-461.24 | `test_wait_for_timeout` | helpers | Not implemented |
| FR-461.25 | `test_log_level_serde` | logging | Not implemented |

**Total**: 25 tests, 0 implemented, 25 missing

---

## 11. Implementation Checklist

| Requirement | ID | Status | Notes |
|------------|----|--------|-------|
| Create `crates/util/` directory | FR-422 | 🔴 Not Started | P0 |
| `LogLevel` enum | FR-422 | 🔴 Not Started | P0 |
| `Logger` struct | FR-423-428 | 🔴 Not Started | P0 |
| `Rotation` struct | FR-423 | 🔴 Not Started | P1 |
| `log_file_path()` | FR-429 | 🔴 Not Started | P2 |
| `NamedError` struct | FR-431 | 🔴 Not Started | P0 |
| `From<std::io::Error>` | FR-432 | 🔴 Not Started | P0 |
| `From<reqwest::Error>` | FR-433 | 🔴 Not Started | P1 |
| `WithContext<E>` | FR-434 | 🔴 Not Started | P1 |
| `Context` trait | FR-435 | 🔴 Not Started | P1 |
| `RetryConfig` | FR-437 | 🔴 Not Started | P1 |
| `retry()` function | FR-438 | 🔴 Not Started | P1 |
| `atomic_write()` | FR-443 | 🔴 Not Started | P1 |
| `read_json()` / `write_json()` | FR-445-446 | 🔴 Not Started | P1 |
| `Lazy<T>` | FR-448 | 🔴 Not Started | P2 |
| `iife!` macro | FR-449 | 🔴 Not Started | P2 |
| `with_timeout()` | FR-450 | 🔴 Not Started | P2 |
| `wait_for()` / `retry_until()` | FR-451-452 | 🔴 Not Started | P2 |
| Dependencies | FR-454-455 | 🔴 Not Started | P0 |
| Tests | FR-461 | 🔴 Not Started | P1 |

---

## 12. Gap Summary

### P0 - Blocking Issues

| Gap Item | Severity | Module | Remediation |
|----------|----------|--------|-------------|
| `crates/util/` crate does not exist | P0 | Workspace | Create `crates/util/` with Cargo.toml and src/ structure |
| `NamedError` struct not implemented | P0 | error.rs | Implement NamedError with name, code, message, data fields |
| `Logger` struct not implemented | P0 | logging.rs | Implement Logger with fluent API for level, file, console |
| `LogLevel` enum not implemented | P0 | logging.rs | Implement LogLevel with Serialize/Deserialize |
| Missing `tracing-appender` dependency | P0 | logging.rs | Add tracing-appender = "0.2" to workspace |

### P1 - High Priority Issues

| Gap Item | Severity | Module | Remediation |
|----------|----------|--------|-------------|
| `From<std::io::Error>` not implemented | P1 | error.rs | Implement From<io::Error> for NamedError |
| `From<reqwest::Error>` not implemented | P1 | error.rs | Implement From<reqwest::Error> for NamedError |
| `WithContext<E>` not implemented | P1 | error.rs | Implement WithContext wrapper struct |
| `Context` trait not implemented | P1 | error.rs | Implement Context extension trait |
| `RetryConfig` with jitter not implemented | P1 | retry.rs | Implement RetryConfig with jitter field |
| `retry()` function not implemented | P1 | retry.rs | Implement retry() with exponential backoff + jitter |
| `atomic_write()` not implemented | P1 | fs.rs | Implement atomic_write() using temp file + rename |
| `read_json()` / `write_json()` not async | P1 | fs.rs | Implement async versions |
| Tests not implemented | P1 | all | Add 25 tests per test matrix |

### P2 - Low Priority / Technical Debt

| Gap Item | Severity | Module | Remediation |
|----------|----------|--------|-------------|
| `Lazy<T>` not implemented | P2 | helpers.rs | Implement Lazy using OnceCell |
| `iife!` macro not implemented | P2 | helpers.rs | Implement #[macro_export] iife! |
| `with_timeout()` not implemented | P2 | helpers.rs | Implement with_timeout using tokio::time::timeout |
| `wait_for()` not implemented | P2 | helpers.rs | Implement wait_for polling helper |
| `retry_until()` not implemented | P2 | helpers.rs | Implement retry_until condition-based retry |
| `Rotation` struct not implemented | P2 | logging.rs | Implement log file rotation |
| `log_file_path()` not implemented | P2 | logging.rs | Implement Global::path_opencode().join("logs") |

---

## 13. Technical Debt

| Debt Item | Location | Description | Remediation |
|-----------|----------|-------------|-------------|
| **分散的错误处理** | `crates/core/src/error.rs`, `crates/llm/src/error.rs` | `OpenCodeError` and `LlmError` are separate `thiserror` enums | Create `NamedError` as primary, consider deprecating legacy enums |
| **重复的日志初始化** | `crates/core/src/observability.rs`, `crates/cli/src/main.rs` | `setup_tracing()` exists but `Logger` struct not implemented | Consolidate into `Logger` struct with fluent API |
| **同步文件系统操作** | `crates/core/src/filesystem.rs` | `AppFileSystem` methods are sync, PRD requires async | Add async wrappers using `tokio::fs` in `crates/util/fs.rs` |
| **Retry 实现不一致** | `crates/llm/src/error.rs` | Current retry uses `backoff_multiplier`, PRD specifies jitter-based exponential backoff | Align with PRD API — new `retry()` in `crates/util/retry.rs` |

---

## 14. Relationship to Other Modules

| Related Module | Relationship |
|----------------|--------------|
| `opencode-core` | Uses `Global::path_opencode()` for log file path; exports `NamedError` via re-export |
| `opencode-llm` | Current `with_retry()` in `llm/src/error.rs` to be replaced by `crates/util/retry::retry()` |
| `opencode-cli` | Uses `setup_tracing()` from `observability.rs` — will migrate to `Logger` |
| `opencode-storage` | Uses `AppFileSystem` sync methods — may migrate to async `fs` helpers |

---

## 15. Conclusion

The `util` module is **not implemented** — 100% gap with all features missing.

**Blocking issues (P0)**:
- `crates/util/` crate does not exist
- No `NamedError` struct per PRD spec
- No `Logger` struct per PRD spec
- No `LogLevel` enum per PRD spec

**High priority (P1)**:
- `RetryConfig` with jitter support
- `retry()` function with exponential backoff
- `atomic_write()` for safe file writes
- Async `read_json()` / `write_json()`

**Technical debt (P2)**:
- `Lazy<T>` thread-safe once cell
- `iife!` macro
- `with_timeout()` helper
- `Rotation` struct for log file rotation

**Overall assessment**: The `util` module requires full implementation from scratch. The PRD specifies a clean API design that should be followed closely. The existing scattered utilities (`OpenCodeError`, `AppFileSystem`, `setup_tracing`) will need to be consolidated or replaced.

---

*Specification generated by Sisyphus gap analysis pipeline*
*FR numbers: FR-422 to FR-461 (aligned to iteration 41)*
*Next iteration: FR-462 onwards*