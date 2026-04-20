# Module: effect

## Overview

The `effect` module in `opencode-core` (`crates/core/src/effect.rs`) provides a lightweight `Effect<T>` monad for composing lazy async computations. It wraps a boxed future factory and supports `map`, `and_then`, `success`, and `failure` combinators.

**Crate**: `opencode-core`  
**Source**: `crates/core/src/effect.rs`  
**Status**: Fully implemented (101 lines)

---

## Crate Layout

```
crates/core/src/
└── effect.rs
```

**`crates/core/src/lib.rs`** exports:
```rust
pub mod effect;
pub use effect::{Effect, EffectError, EffectFuture, EffectResult, EffectRunner};
```

---

## Core Types

### Type Aliases

```rust
/// Result type for all Effect computations.
pub type EffectResult<T> = Result<T, EffectError>;

/// A pinned, boxed, Send-safe future.
pub type EffectFuture<T> = Pin<Box<dyn Future<Output = EffectResult<T>> + Send>>;

/// A boxed, once-callable closure producing an EffectFuture.
pub type EffectRunner<T> = Box<dyn FnOnce() -> EffectFuture<T> + Send>;
```

### `EffectError`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectError {
    Generic(String),
    Io(String),
    Network(String),
    Validation(String),
    NotFound(String),
    PermissionDenied(String),
    Timeout(String),
    Cancelled(String),
}

impl std::fmt::Display for EffectError { ... }
impl std::error::Error for EffectError {}

impl From<std::io::Error> for EffectError {
    fn from(err: std::io::Error) -> Self { EffectError::Io(err.to_string()) }
}

impl From<serde_json::Error> for EffectError {
    fn from(err: serde_json::Error) -> Self { EffectError::Generic(err.to_string()) }
}
```

### `Effect<T>`

```rust
pub struct Effect<T> {
    run: EffectRunner<T>,
}
```

The inner `EffectRunner<T>` is a `Box<dyn FnOnce() -> EffectFuture<T> + Send>` — a lazy factory: the async computation is not started until `.run()` is called.

---

## Key Implementations

```rust
impl<T: Send + 'static> Effect<T> {
    /// Construct from any async closure.
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = EffectResult<T>> + Send + 'static,
    {
        Self { run: Box::new(move || Box::pin(f())) }
    }

    /// Execute the effect and await the result.
    pub async fn run(self) -> EffectResult<T> {
        (self.run)().await
    }

    /// Transform the success value.
    pub fn map<U: Send + 'static>(self, f: impl FnOnce(T) -> U + Send + 'static) -> Effect<U> {
        Effect::new(move || async move {
            match self.run().await {
                Ok(val) => Ok(f(val)),
                Err(e) => Err(e),
            }
        })
    }

    /// Chain effects: run self, then run f(value) if Ok.
    pub fn and_then<U: Send + 'static>(
        self,
        f: impl FnOnce(T) -> Effect<U> + Send + 'static,
    ) -> Effect<U> {
        Effect::new(move || async move {
            match self.run().await {
                Ok(val) => f(val).run().await,
                Err(e) => Err(e),
            }
        })
    }

    /// Construct an immediately-successful effect.
    pub fn success(value: T) -> Self {
        Effect::new(move || async move { Ok(value) })
    }

    /// Construct an immediately-failing effect.
    pub fn failure(error: EffectError) -> Self {
        Effect::new(move || async move { Err(error) })
    }
}
```

---

## Usage Pattern

```rust
use opencode_core::effect::{Effect, EffectError};

// Basic effect
let effect = Effect::new(|| async {
    let content = tokio::fs::read_to_string("file.txt").await
        .map_err(EffectError::from)?;
    Ok(content)
});
let result = effect.run().await;

// Chaining
let chained = Effect::new(|| async { Ok(42u32) })
    .map(|n| n * 2)
    .and_then(|n| Effect::new(move || async move { Ok(format!("result: {n}")) }));
let result = chained.run().await;
assert_eq!(result.unwrap(), "result: 84");

// Immediate values
let ok = Effect::success("hello").run().await;
assert!(ok.is_ok());

let err = Effect::<()>::failure(EffectError::NotFound("file".into())).run().await;
assert!(err.is_err());

// From std::io::Error
let io_effect = Effect::new(|| async {
    tokio::fs::read_to_string("nonexistent.txt").await
        .map_err(EffectError::from)
});
match io_effect.run().await {
    Err(EffectError::Io(msg)) => eprintln!("IO error: {}", msg),
    _ => {}
}
```

---

## When to Use `Effect<T>` vs `async fn`

| Use `Effect<T>` | Use plain `async fn` |
|---|---|
| Deferred/lazy computation | Immediate execution |
| Composing pipelines with `map`/`and_then` | Simple sequential operations |
| Passing computation as a value | Direct function calls |
| Need `Send`-safe boxed futures | Inline async code |

In practice, most of the codebase uses plain `async fn`. `Effect<T>` is provided as a functional composition utility for cases where you need to build computation pipelines as data.

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn success_effect_returns_value() {
        let e = Effect::success(42u32);
        assert_eq!(e.run().await.unwrap(), 42);
    }

    #[tokio::test]
    async fn failure_effect_returns_error() {
        let e = Effect::<()>::failure(EffectError::Generic("oops".into()));
        assert!(e.run().await.is_err());
    }

    #[tokio::test]
    async fn map_transforms_ok_value() {
        let e = Effect::success(10u32).map(|n| n * 3);
        assert_eq!(e.run().await.unwrap(), 30);
    }

    #[tokio::test]
    async fn map_propagates_error() {
        let e = Effect::<u32>::failure(EffectError::Timeout("t".into()))
            .map(|n| n + 1);
        assert!(matches!(e.run().await, Err(EffectError::Timeout(_))));
    }

    #[tokio::test]
    async fn and_then_chains_effects() {
        let e = Effect::success(5u32)
            .and_then(|n| Effect::success(n * 2));
        assert_eq!(e.run().await.unwrap(), 10);
    }

    #[tokio::test]
    async fn and_then_short_circuits_on_error() {
        let e = Effect::<u32>::failure(EffectError::NotFound("x".into()))
            .and_then(|_| Effect::success(42u32));
        assert!(matches!(e.run().await, Err(EffectError::NotFound(_))));
    }

    #[tokio::test]
    async fn effect_from_async_closure() {
        let e = Effect::new(|| async { Ok(99u32) });
        assert_eq!(e.run().await.unwrap(), 99);
    }

    #[test]
    fn effect_error_display() {
        assert!(EffectError::Generic("msg".into()).to_string().contains("Generic error"));
        assert!(EffectError::Io("err".into()).to_string().contains("IO error"));
        assert!(EffectError::NotFound("x".into()).to_string().contains("Not found"));
    }

    #[test]
    fn io_error_converts_to_effect_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file");
        let effect_err = EffectError::from(io_err);
        assert!(matches!(effect_err, EffectError::Io(_)));
    }

    #[test]
    fn serde_error_converts_to_effect_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let effect_err = EffectError::from(json_err);
        assert!(matches!(effect_err, EffectError::Generic(_)));
    }
}
```
