use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub struct Debouncer {
    delay: Duration,
    pending: Arc<Mutex<HashMap<PathBuf, Arc<AtomicU64>>>>,
    counter: Arc<AtomicU64>,
}

impl Debouncer {
    pub fn new(delay: Duration) -> Self {
        Self {
            delay,
            pending: Arc::new(Mutex::new(HashMap::new())),
            counter: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn queue<F>(&self, path: PathBuf, callback: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let delay = self.delay;
        let pending = self.pending.clone();
        let counter = self.counter.clone();

        let seq = counter.fetch_add(1, Ordering::SeqCst);

        let seq_arc = {
            let mut guard = pending.lock().await;
            let entry = guard.entry(path.clone()).or_insert_with(|| Arc::new(AtomicU64::new(0)));
            entry.store(seq, Ordering::SeqCst);
            entry.clone()
        };

        let pending2 = pending.clone();
        let path2 = path.clone();

        tokio::spawn(async move {
            tokio::time::sleep(delay).await;

            let should_call = {
                let guard = pending2.lock().await;
                if let Some(current_seq) = guard.get(&path2) {
                    let current = current_seq.load(Ordering::SeqCst);
                    let mine = seq_arc.load(Ordering::SeqCst);
                    current == mine
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

        tokio::time::sleep(Duration::from_millis(200)).await;
        let final_count = count.load(Ordering::SeqCst);
        assert_eq!(final_count, 1, "Expected 1 callback but got {}", final_count);
    }
}