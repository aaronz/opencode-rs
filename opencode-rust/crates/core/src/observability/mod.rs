mod types;

pub use types::{ObservabilityTracker, SessionTrace, setup_tracing, TokenUsage};

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