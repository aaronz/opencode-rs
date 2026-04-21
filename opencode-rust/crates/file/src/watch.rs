use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub struct Debouncer {
    delay: Duration,
    pending: Arc<Mutex<HashMap<PathBuf, u64>>>,
}

impl Debouncer {
    pub fn new(delay: Duration) -> Self {
        Self {
            delay,
            pending: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn queue<F>(&self, path: PathBuf, callback: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let delay = self.delay;
        let pending = self.pending.clone();

        let seq = {
            let mut guard = pending.lock().await;
            let counter = guard.entry(path.clone()).or_insert(0);
            *counter += 1;
            *counter
        };

        let pending2 = pending.clone();
        let path2 = path.clone();

        tokio::spawn(async move {
            tokio::time::sleep(delay).await;

            let should_call = {
                let mut guard = pending2.lock().await;
                if let Some(counter) = guard.get(&path2) {
                    if *counter == seq {
                        guard.remove(&path2);
                        true
                    } else {
                        guard.remove(&path2);
                        false
                    }
                } else {
                    false
                }
            };

            if should_call {
                callback();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_debouncer_merges_rapid_events() {
        let debounce = Duration::from_millis(50);
        let debouncer = Debouncer::new(debounce);
        let count = Arc::new(AtomicUsize::new(0));
        let count2 = count.clone();

        debouncer
            .queue(PathBuf::from("a.txt"), move || {
                count2.fetch_add(1, Ordering::SeqCst);
            })
            .await;

        let count3 = count.clone();
        debouncer
            .queue(PathBuf::from("a.txt"), move || {
                count3.fetch_add(1, Ordering::SeqCst);
            })
            .await;

        tokio::time::sleep(Duration::from_millis(150)).await;
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}