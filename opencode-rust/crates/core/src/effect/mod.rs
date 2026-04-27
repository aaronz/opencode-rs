use std::future::Future;
use std::pin::Pin;

mod types;

pub use types::{EffectError, EffectFuture, EffectResult, EffectRunner};

pub struct Effect<T> {
    run: EffectRunner<T>,
}

impl<T: Send + 'static> Effect<T> {
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = EffectResult<T>> + Send + 'static,
    {
        Self {
            run: Box::new(move || Box::pin(f())),
        }
    }

    pub async fn run(self) -> EffectResult<T> {
        (self.run)().await
    }

    pub fn map<U: Send + 'static>(self, f: impl FnOnce(T) -> U + Send + 'static) -> Effect<U> {
        Effect::new(move || async move {
            match self.run().await {
                Ok(val) => Ok(f(val)),
                Err(e) => Err(e),
            }
        })
    }

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

    pub fn success(value: T) -> Self {
        Effect::new(move || async move { Ok(value) })
    }

    pub fn failure(error: EffectError) -> Self {
        Effect::new(move || async move { Err(error) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_effect_success() {
        let effect = Effect::<i32>::success(42);
        let result = effect.run().await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_effect_failure() {
        let effect = Effect::<i32>::failure(EffectError::Generic("test error".to_string()));
        let result = effect.run().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Generic error: test error");
    }

    #[tokio::test]
    async fn test_effect_map_transforms_value() {
        let effect = Effect::success(21).map(|x| x * 2);
        let result = effect.run().await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_effect_map_passes_through_error() {
        let effect =
            Effect::<i32>::failure(EffectError::NotFound("item".to_string())).map(|x| x * 2);
        let result = effect.run().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not found"));
    }

    #[tokio::test]
    async fn test_effect_and_then_chains_correctly() {
        let effect = Effect::success(21).and_then(|x| Effect::success(x * 2));
        let result = effect.run().await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_effect_and_then_error_propagates() {
        let effect = Effect::<i32>::failure(EffectError::Validation("invalid".to_string()))
            .and_then(|x| Effect::success(x * 2));
        let result = effect.run().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Validation"));
    }

    #[tokio::test]
    async fn test_effect_pipeline_multiple_chains() {
        let effect = Effect::success(10)
            .and_then(|x| Effect::success(x + 5))
            .and_then(|x| Effect::success(x * 2))
            .map(|x| x - 5);
        let result = effect.run().await;
        assert_eq!(result.unwrap(), 25);
    }

    #[tokio::test]
    async fn test_effect_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let effect_err: EffectError = io_err.into();
        assert!(matches!(effect_err, EffectError::Io(_)));
    }

    #[tokio::test]
    async fn test_effect_error_display() {
        let errors = vec![
            (EffectError::Generic("gen".into()), "Generic error: gen"),
            (EffectError::Io("io".into()), "IO error: io"),
            (EffectError::Network("net".into()), "Network error: net"),
            (
                EffectError::Validation("val".into()),
                "Validation error: val",
            ),
            (EffectError::NotFound("nf".into()), "Not found: nf"),
            (
                EffectError::PermissionDenied("perm".into()),
                "Permission denied: perm",
            ),
            (EffectError::Timeout("timeout".into()), "Timeout: timeout"),
            (EffectError::Cancelled("cancel".into()), "Cancelled: cancel"),
        ];
        for (err, expected) in errors {
            assert_eq!(err.to_string(), expected);
        }
    }

    #[tokio::test]
    async fn test_effect_error_serde() {
        let err = EffectError::Validation("test".to_string());
        let json = serde_json::to_string(&err).unwrap();
        let decoded: EffectError = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, EffectError::Validation(v) if v == "test"));
    }

    #[tokio::test]
    async fn test_effect_new_with_async_fn() {
        let effect = Effect::new(|| async move { Ok::<i32, EffectError>(100) });
        let result = effect.run().await;
        assert_eq!(result.unwrap(), 100);
    }

    #[tokio::test]
    async fn test_effect_new_with_async_fn_error() {
        let effect =
            Effect::<i32>::new(
                || async move { Err(EffectError::Network("connection refused".into())) },
            );
        let result = effect.run().await;
        let err = result.unwrap_err();
        assert!(matches!(err, EffectError::Network(_)));
    }

    #[tokio::test]
    async fn test_effect_pipeline_error_in_middle() {
        let effect = Effect::success(10)
            .and_then(|_x| Effect::<i32>::failure(EffectError::Cancelled("stop".into())))
            .and_then(|x| Effect::success(x + 5))
            .map(|x| x * 2);
        let result = effect.run().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cancelled"));
    }

    #[tokio::test]
    async fn test_effect_file_read_pipeline() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.txt");
        tokio::fs::write(&path, "hello world").await.unwrap();

        let effect = Effect::new(move || {
            let path = path.clone();
            async move {
                let content = tokio::fs::read_to_string(&path).await?;
                Ok::<_, EffectError>(content)
            }
        })
        .map(|s| s.to_uppercase());

        let result = effect.run().await;
        assert_eq!(result.unwrap(), "HELLO WORLD");
    }

    #[tokio::test]
    async fn test_effect_resource_cleanup_on_success() {
        let cleanup_called = Arc::new(Mutex::new(false));
        let cleanup_clone = cleanup_called.clone();

        struct TestResource {
            cleanup: Arc<Mutex<bool>>,
        }
        impl Drop for TestResource {
            fn drop(&mut self) {
                *self.cleanup.lock().unwrap() = true;
            }
        }

        let effect = Effect::new(|| async move {
            let _resource = TestResource {
                cleanup: cleanup_clone,
            };
            Ok::<_, EffectError>(42)
        });

        let result = effect.run().await;
        assert_eq!(result.unwrap(), 42);
        assert!(*cleanup_called.lock().unwrap());
    }

    #[tokio::test]
    async fn test_effect_resource_cleanup_on_error() {
        let cleanup_called = Arc::new(Mutex::new(false));
        let cleanup_clone = cleanup_called.clone();

        struct TestResource {
            cleanup: Arc<Mutex<bool>>,
        }
        impl Drop for TestResource {
            fn drop(&mut self) {
                *self.cleanup.lock().unwrap() = true;
            }
        }

        let effect = Effect::<i32>::new(move || async move {
            let _resource = TestResource {
                cleanup: cleanup_clone,
            };
            Err(EffectError::Generic("fail".into()))
        });

        let result = effect.run().await;
        assert!(result.is_err());
        assert!(*cleanup_called.lock().unwrap());
    }

    #[tokio::test]
    async fn test_effect_multiple_transformations() {
        let effect = Effect::success(vec![1, 2, 3])
            .map(|v| v.iter().sum::<i32>())
            .map(|x| x * 2)
            .map(|x| format!("result: {}", x));

        let result = effect.run().await;
        assert_eq!(result.unwrap(), "result: 12");
    }

    #[tokio::test]
    async fn test_effect_hashmap_pipeline() {
        let effect = Effect::success(HashMap::from([("a".to_string(), 1), ("b".to_string(), 2)]))
            .map(|mut h| {
                h.insert("c".to_string(), 3);
                h
            })
            .map(|h| h.values().sum::<i32>());

        let result = effect.run().await;
        assert_eq!(result.unwrap(), 6);
    }
}