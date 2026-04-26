use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};
use tracing_subscriber::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub provider: String,
    pub model: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub cost_usd: f64,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionTrace {
    pub session_id: String,
    pub start_time: String,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub provider_usage: HashMap<String, TokenUsage>,
    pub tool_calls: u64,
    pub errors: u64,
}

#[derive(Clone)]
pub struct ObservabilityTracker {
    session_id: String,
    total_prompt_tokens: Arc<AtomicU64>,
    total_completion_tokens: Arc<AtomicU64>,
    total_cost_cents: Arc<AtomicU64>,
    provider_calls: Arc<AtomicU64>,
    tool_calls: Arc<AtomicU64>,
    errors: Arc<AtomicU64>,
    start_time: Instant,
}

impl ObservabilityTracker {
    pub fn new(session_id: &str) -> Self {
        if !cfg!(test) {
            info!(session_id, "observability tracker initialized");
        }
        Self {
            session_id: session_id.to_string(),
            total_prompt_tokens: Arc::new(AtomicU64::new(0)),
            total_completion_tokens: Arc::new(AtomicU64::new(0)),
            total_cost_cents: Arc::new(AtomicU64::new(0)),
            provider_calls: Arc::new(AtomicU64::new(0)),
            tool_calls: Arc::new(AtomicU64::new(0)),
            errors: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
        }
    }

    pub fn record_token_usage(
        &self,
        provider: &str,
        model: &str,
        prompt: u64,
        completion: u64,
        cost_usd: f64,
        latency_ms: u64,
    ) {
        self.total_prompt_tokens
            .fetch_add(prompt, Ordering::Relaxed);
        self.total_completion_tokens
            .fetch_add(completion, Ordering::Relaxed);
        self.total_cost_cents
            .fetch_add((cost_usd * 1_000_000.0).round() as u64, Ordering::Relaxed);
        self.provider_calls.fetch_add(1, Ordering::Relaxed);

        if !cfg!(test) {
            info!(
                session_id = self.session_id,
                provider,
                model,
                prompt_tokens = prompt,
                completion_tokens = completion,
                total_tokens = prompt + completion,
                cost_usd = cost_usd,
                latency_ms = latency_ms,
                "token usage recorded"
            );
        }
    }

    pub fn record_tool_call(&self, tool_name: &str, success: bool, latency_ms: u64) {
        self.tool_calls.fetch_add(1, Ordering::Relaxed);

        if !cfg!(test) {
            if success {
                info!(
                    session_id = self.session_id,
                    tool = tool_name,
                    latency_ms = latency_ms,
                    "tool call completed"
                );
            } else {
                warn!(
                    session_id = self.session_id,
                    tool = tool_name,
                    latency_ms = latency_ms,
                    "tool call failed"
                );
                self.errors.fetch_add(1, Ordering::Relaxed);
            }
        } else if !success {
            self.errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_error(&self, error_type: &str, message: &str) {
        self.errors.fetch_add(1, Ordering::Relaxed);
        if !cfg!(test) {
            warn!(
                session_id = self.session_id,
                error_type, message, "error recorded"
            );
        }
    }

    pub fn get_summary(&self) -> SessionTrace {
        SessionTrace {
            session_id: self.session_id.clone(),
            start_time: format!("{:?}", self.start_time.elapsed()),
            total_tokens: self.total_prompt_tokens.load(Ordering::Relaxed)
                + self.total_completion_tokens.load(Ordering::Relaxed),
            total_cost_usd: self.total_cost_cents.load(Ordering::Relaxed) as f64 / 1_000_000.0,
            provider_usage: HashMap::new(),
            tool_calls: self.tool_calls.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
        }
    }

    pub fn total_tokens(&self) -> u64 {
        self.total_prompt_tokens.load(Ordering::Relaxed)
            + self.total_completion_tokens.load(Ordering::Relaxed)
    }

    pub fn total_cost_usd(&self) -> f64 {
        self.total_cost_cents.load(Ordering::Relaxed) as f64 / 1_000_000.0
    }
}

pub fn setup_tracing(log_level: &str) {
    let level = match log_level.to_lowercase().as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .with_writer(std::io::stderr);

    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(level.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observability_tracker_records_tokens() {
        let tracker = ObservabilityTracker::new("test-session");
        tracker.record_token_usage("openai", "gpt-4", 100, 200, 0.005, 1500);

        assert_eq!(tracker.total_tokens(), 300);
        assert!((tracker.total_cost_usd() - 0.005).abs() < 0.001);
    }

    #[test]
    fn test_observability_tracker_accumulates() {
        let tracker = ObservabilityTracker::new("test-session");
        tracker.record_token_usage("openai", "gpt-4", 100, 200, 0.005, 1500);
        tracker.record_token_usage("anthropic", "claude-3", 150, 250, 0.008, 2000);

        assert_eq!(tracker.total_tokens(), 700);
        assert!((tracker.total_cost_usd() - 0.013).abs() < 0.001);
    }

    #[test]
    fn test_observability_tracker_records_errors() {
        let tracker = ObservabilityTracker::new("test-session");
        tracker.record_error("api_error", "rate limited");
        tracker.record_tool_call("write", false, 100);

        let summary = tracker.get_summary();
        assert_eq!(summary.errors, 2);
    }

    #[test]
    fn test_observability_tracker_records_tool_calls() {
        let tracker = ObservabilityTracker::new("test-session");
        tracker.record_tool_call("read", true, 50);
        tracker.record_tool_call("write", true, 100);
        tracker.record_tool_call("bash", false, 5000);

        let summary = tracker.get_summary();
        assert_eq!(summary.tool_calls, 3);
        assert_eq!(summary.errors, 1);
    }

    #[test]
    fn test_session_trace_serialization() {
        let trace = SessionTrace {
            session_id: "sess-123".to_string(),
            start_time: "1.5s".to_string(),
            total_tokens: 5000,
            total_cost_usd: 0.025,
            provider_usage: HashMap::new(),
            tool_calls: 10,
            errors: 1,
        };

        let json = serde_json::to_string(&trace).unwrap();
        assert!(json.contains("sess-123"));
        assert!(json.contains("5000"));
    }
}
