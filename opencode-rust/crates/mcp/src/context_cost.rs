use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use opencode_core::TokenCounter;

const CONTEXT_WARNING_THRESHOLD_PERCENT: f64 = 0.80;
const CONTEXT_LIMIT_PERCENT: f64 = 0.95;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostLevel {
    Normal,
    Warning,
    Critical,
    LimitExceeded,
}

#[derive(Debug, Clone, Copy)]
pub struct CostLimits {
    pub max_tokens: usize,
    pub warning_threshold: f64,
    pub limit_threshold: f64,
}

impl Default for CostLimits {
    fn default() -> Self {
        Self {
            max_tokens: 128_000,
            warning_threshold: CONTEXT_WARNING_THRESHOLD_PERCENT,
            limit_threshold: CONTEXT_LIMIT_PERCENT,
        }
    }
}

impl CostLimits {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            ..Default::default()
        }
    }

    pub fn warning_at(&self) -> usize {
        (self.max_tokens as f64 * self.warning_threshold) as usize
    }

    pub fn limit_at(&self) -> usize {
        (self.max_tokens as f64 * self.limit_threshold) as usize
    }
}

#[derive(Debug, Clone)]
pub struct CostRecord {
    pub tool_name: String,
    pub server_name: String,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_tokens: usize,
    pub cost_level: CostLevel,
    pub timestamp: Instant,
}

#[derive(Debug)]
pub struct ContextCostStats {
    pub total_input_tokens: usize,
    pub total_output_tokens: usize,
    pub total_tokens: usize,
    pub tool_call_count: usize,
    pub current_level: CostLevel,
    pub remaining_tokens: usize,
    pub limit: usize,
}

pub struct ContextCostTracker {
    token_counter: TokenCounter,
    limits: CostLimits,
    total_input_tokens: usize,
    total_output_tokens: usize,
    tool_call_count: usize,
    records: Vec<CostRecord>,
    last_warning_at: Option<Instant>,
    warning_cooldown: Duration,
}

impl Default for ContextCostTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextCostTracker {
    pub fn new() -> Self {
        Self {
            token_counter: TokenCounter::new(),
            limits: CostLimits::default(),
            total_input_tokens: 0,
            total_output_tokens: 0,
            tool_call_count: 0,
            records: Vec::new(),
            last_warning_at: None,
            warning_cooldown: Duration::from_secs(30),
        }
    }

    pub fn with_limits(mut self, limits: CostLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.token_counter = TokenCounter::for_model(model);
        self
    }

    pub fn with_warning_cooldown(mut self, cooldown: Duration) -> Self {
        self.warning_cooldown = cooldown;
        self
    }

    pub fn limits(&self) -> CostLimits {
        self.limits
    }

    pub fn update_limits(&mut self, limits: CostLimits) {
        self.limits = limits;
    }

    fn get_total_tokens(&self) -> usize {
        self.total_input_tokens
            .saturating_add(self.total_output_tokens)
    }

    pub fn record_tool_call(
        &mut self,
        tool_name: &str,
        server_name: &str,
        input_text: &str,
        output_text: &str,
    ) -> CostRecord {
        let input_tokens = self.token_counter.count_tokens(input_text);
        let output_tokens = self.token_counter.count_tokens(output_text);
        let total_tokens = input_tokens + output_tokens;

        self.total_input_tokens += input_tokens;
        self.total_output_tokens += output_tokens;
        self.tool_call_count += 1;

        let cost_level = self.calculate_cost_level(self.get_total_tokens());

        let record = CostRecord {
            tool_name: tool_name.to_string(),
            server_name: server_name.to_string(),
            input_tokens,
            output_tokens,
            total_tokens,
            cost_level,
            timestamp: Instant::now(),
        };

        self.records.push(record.clone());
        record
    }

    pub fn record_tool_call_with_tokens(
        &mut self,
        tool_name: &str,
        server_name: &str,
        input_tokens: usize,
        output_tokens: usize,
    ) -> CostRecord {
        let total_tokens = input_tokens + output_tokens;

        self.total_input_tokens += input_tokens;
        self.total_output_tokens += output_tokens;
        self.tool_call_count += 1;

        let cost_level = self.calculate_cost_level(self.get_total_tokens());

        let record = CostRecord {
            tool_name: tool_name.to_string(),
            server_name: server_name.to_string(),
            input_tokens,
            output_tokens,
            total_tokens,
            cost_level,
            timestamp: Instant::now(),
        };

        self.records.push(record.clone());
        record
    }

    pub fn calculate_cost_level(&self, total_tokens: usize) -> CostLevel {
        let warning_at = self.limits.warning_at();
        let limit_at = self.limits.limit_at();

        if total_tokens >= limit_at {
            CostLevel::LimitExceeded
        } else if total_tokens >= warning_at {
            CostLevel::Critical
        } else if total_tokens > warning_at / 2 {
            CostLevel::Warning
        } else {
            CostLevel::Normal
        }
    }

    pub fn should_warn(&self) -> bool {
        if let Some(last_warning) = self.last_warning_at {
            if last_warning.elapsed() < self.warning_cooldown {
                return false;
            }
        }

        let current_level = self.calculate_cost_level(self.get_total_tokens());
        matches!(
            current_level,
            CostLevel::Warning | CostLevel::Critical | CostLevel::LimitExceeded
        )
    }

    pub fn record_warning_shown(&mut self) {
        self.last_warning_at = Some(Instant::now());
    }

    pub fn get_warning_message(&self) -> Option<String> {
        if !self.should_warn() {
            return None;
        }

        let current_total = self.get_total_tokens();
        let current_level = self.calculate_cost_level(current_total);

        Some(match current_level {
            CostLevel::Normal => return None,
            CostLevel::Warning => format!(
                "MCP context usage is at {:.0}% ({}/{} tokens). Consider using /compact to reduce context size.",
                current_total as f64 / self.limits.max_tokens as f64 * 100.0,
                current_total,
                self.limits.max_tokens
            ),
            CostLevel::Critical => format!(
                "MCP context usage is high at {:.0}% ({}/{} tokens). Consider using /compact soon.",
                current_total as f64 / self.limits.max_tokens as f64 * 100.0,
                current_total,
                self.limits.max_tokens
            ),
            CostLevel::LimitExceeded => format!(
                "MCP context limit exceeded ({}/{} tokens). Use /compact to free up context.",
                current_total,
                self.limits.max_tokens
            ),
        })
    }

    pub fn stats(&self) -> ContextCostStats {
        let total = self.get_total_tokens();
        ContextCostStats {
            total_input_tokens: self.total_input_tokens,
            total_output_tokens: self.total_output_tokens,
            total_tokens: total,
            tool_call_count: self.tool_call_count,
            current_level: self.calculate_cost_level(total),
            remaining_tokens: self.limits.max_tokens.saturating_sub(total),
            limit: self.limits.max_tokens,
        }
    }

    pub fn total_tokens(&self) -> usize {
        self.get_total_tokens()
    }

    pub fn remaining_tokens(&self) -> usize {
        self.limits
            .max_tokens
            .saturating_sub(self.get_total_tokens())
    }

    pub fn would_exceed_limit(&self, additional_tokens: usize) -> bool {
        self.get_total_tokens() + additional_tokens >= self.limits.limit_at()
    }

    pub fn records(&self) -> &[CostRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.total_input_tokens = 0;
        self.total_output_tokens = 0;
        self.tool_call_count = 0;
        self.records.clear();
        self.last_warning_at = None;
    }

    pub fn estimate_tokens(&self, text: &str) -> usize {
        self.token_counter.count_tokens(text)
    }

    pub fn tool_call_count(&self) -> usize {
        self.tool_call_count
    }
}

#[derive(Clone)]
pub struct SharedContextCostTracker {
    inner: Arc<Mutex<ContextCostTracker>>,
}

impl Default for SharedContextCostTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedContextCostTracker {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ContextCostTracker::new())),
        }
    }

    pub fn with_limits(self, limits: CostLimits) -> Self {
        self.inner.lock().unwrap().update_limits(limits);
        self
    }

    pub fn with_model(self, model: &str) -> Self {
        let mut tracker = self.inner.lock().unwrap();
        let limits = tracker.limits;
        let tool_call_count = tracker.tool_call_count;
        let records = tracker.records.clone();
        let last_warning_at = tracker.last_warning_at;
        let warning_cooldown = tracker.warning_cooldown;

        let mut new_tracker = ContextCostTracker::new()
            .with_model(model)
            .with_limits(limits)
            .with_warning_cooldown(warning_cooldown);
        new_tracker.tool_call_count = tool_call_count;
        new_tracker.records = records;
        new_tracker.last_warning_at = last_warning_at;

        drop(tracker);
        *self.inner.lock().unwrap() = new_tracker;
        self
    }

    pub fn record_tool_call(
        &self,
        tool_name: &str,
        server_name: &str,
        input_text: &str,
        output_text: &str,
    ) -> CostRecord {
        self.inner
            .lock()
            .unwrap()
            .record_tool_call(tool_name, server_name, input_text, output_text)
    }

    pub fn record_tool_call_with_tokens(
        &self,
        tool_name: &str,
        server_name: &str,
        input_tokens: usize,
        output_tokens: usize,
    ) -> CostRecord {
        self.inner.lock().unwrap().record_tool_call_with_tokens(
            tool_name,
            server_name,
            input_tokens,
            output_tokens,
        )
    }

    pub fn should_warn(&self) -> bool {
        self.inner.lock().unwrap().should_warn()
    }

    pub fn get_warning_message(&self) -> Option<String> {
        self.inner.lock().unwrap().get_warning_message()
    }

    pub fn record_warning_shown(&self) {
        self.inner.lock().unwrap().record_warning_shown();
    }

    pub fn stats(&self) -> ContextCostStats {
        self.inner.lock().unwrap().stats()
    }

    pub fn total_tokens(&self) -> usize {
        self.inner.lock().unwrap().total_tokens()
    }

    pub fn remaining_tokens(&self) -> usize {
        self.inner.lock().unwrap().remaining_tokens()
    }

    pub fn would_exceed_limit(&self, additional_tokens: usize) -> bool {
        self.inner
            .lock()
            .unwrap()
            .would_exceed_limit(additional_tokens)
    }

    pub fn clear(&self) {
        self.inner.lock().unwrap().clear();
    }

    pub fn estimate_tokens(&self, text: &str) -> usize {
        self.inner.lock().unwrap().estimate_tokens(text)
    }
}

impl std::fmt::Debug for SharedContextCostTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tracker = self.inner.lock().unwrap();
        f.debug_struct("SharedContextCostTracker")
            .field("total_tokens", &tracker.get_total_tokens())
            .field("tool_call_count", &tracker.tool_call_count)
            .field("limits", &tracker.limits)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_tracker_records_tool_calls() {
        let mut tracker = ContextCostTracker::new();

        let record = tracker.record_tool_call(
            "search_docs",
            "docs_server",
            "query string",
            "search result content here",
        );

        assert_eq!(record.tool_name, "search_docs");
        assert_eq!(record.server_name, "docs_server");
        assert!(record.input_tokens > 0);
        assert!(record.output_tokens > 0);
        assert_eq!(tracker.tool_call_count(), 1);
    }

    #[test]
    fn test_cost_level_calculation() {
        let mut tracker = ContextCostTracker::new();
        tracker.update_limits(CostLimits::new(1000));

        assert_eq!(tracker.calculate_cost_level(0), CostLevel::Normal);
        assert_eq!(tracker.calculate_cost_level(400), CostLevel::Normal);
        assert_eq!(tracker.calculate_cost_level(500), CostLevel::Warning);
        assert_eq!(tracker.calculate_cost_level(800), CostLevel::Critical);
        assert_eq!(tracker.calculate_cost_level(950), CostLevel::LimitExceeded);
    }

    #[test]
    fn test_cost_limits_defaults() {
        let limits = CostLimits::default();

        assert_eq!(limits.max_tokens, 128_000);
        assert!(limits.warning_at() < limits.limit_at());
        assert!(limits.limit_at() <= limits.max_tokens);
    }

    #[test]
    fn test_cost_tracker_accumulates_costs() {
        let mut tracker = ContextCostTracker::new();

        tracker.record_tool_call("tool1", "server1", "input1", "output1");
        tracker.record_tool_call("tool2", "server2", "input2", "output2");

        assert_eq!(tracker.tool_call_count(), 2);
        assert!(tracker.total_tokens() > 0);
    }

    #[test]
    fn test_would_exceed_limit() {
        let mut tracker = ContextCostTracker::new();
        tracker.update_limits(CostLimits::new(100));

        assert!(!tracker.would_exceed_limit(50));
        assert!(tracker.would_exceed_limit(100));
    }

    #[test]
    fn test_warning_message_generation() {
        let mut tracker = ContextCostTracker::new();
        tracker.update_limits(CostLimits::new(100));

        tracker.record_tool_call("tool", "server", "input", "output");

        let msg = tracker.get_warning_message();
        assert!(msg.is_none());

        for _ in 0..15 {
            tracker.record_tool_call(
                "tool",
                "server",
                "some input text here for token counting",
                "some output text here for token counting",
            );
        }

        let msg = tracker.get_warning_message();
        assert!(msg.is_some());
    }

    #[test]
    fn test_shared_tracker() {
        let tracker = SharedContextCostTracker::new();

        tracker.record_tool_call("search", "docs", "query", "result");

        assert_eq!(tracker.stats().tool_call_count, 1);
        assert!(tracker.total_tokens() > 0);
    }

    #[test]
    fn test_estimate_tokens() {
        let tracker = ContextCostTracker::new();

        let tokens = tracker.estimate_tokens("hello world");
        assert!(tokens >= 2);
    }

    #[test]
    fn test_clear_resets_state() {
        let mut tracker = ContextCostTracker::new();

        tracker.record_tool_call("tool", "server", "input", "output");
        assert_eq!(tracker.tool_call_count(), 1);

        tracker.clear();
        assert_eq!(tracker.tool_call_count(), 0);
        assert_eq!(tracker.total_tokens(), 0);
    }
}
