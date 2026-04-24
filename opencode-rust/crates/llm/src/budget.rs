use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub use crate::provider::Usage;
use crate::provider_abstraction::ReasoningBudget;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BudgetLimit {
    None,
    PerRequest(f64),
    PerConversation(f64),
    Combined {
        per_request: f64,
        per_conversation: f64,
    },
}

impl BudgetLimit {
    pub fn is_exceeded(&self, request_cost: f64, conversation_cost: f64) -> bool {
        match self {
            BudgetLimit::None => false,
            BudgetLimit::PerRequest(limit) => request_cost > *limit,
            BudgetLimit::PerConversation(limit) => conversation_cost > *limit,
            BudgetLimit::Combined {
                per_request,
                per_conversation,
            } => request_cost > *per_request || conversation_cost > *per_conversation,
        }
    }

    pub fn check_and_update(
        &self,
        request_cost: f64,
        conversation_cost: f64,
    ) -> Result<(), BudgetExceededError> {
        if self.is_exceeded(request_cost, conversation_cost) {
            Err(BudgetExceededError {
                limit_type: *self,
                request_cost,
                conversation_cost,
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct BudgetExceededError {
    pub limit_type: BudgetLimit,
    pub request_cost: f64,
    pub conversation_cost: f64,
}

impl std::fmt::Display for BudgetExceededError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.limit_type {
            BudgetLimit::None => write!(f, "Budget exceeded but no limit set"),
            BudgetLimit::PerRequest(limit) => {
                write!(f, "Request cost ${:.6} exceeds limit ${:.6}", self.request_cost, limit)
            }
            BudgetLimit::PerConversation(limit) => write!(
                f,
                "Conversation cost ${:.6} exceeds limit ${:.6}",
                self.conversation_cost, limit
            ),
            BudgetLimit::Combined {
                per_request,
                per_conversation,
            } => write!(
                f,
                "Budget exceeded: request=${:.6} (limit=${:.6}), conversation=${:.6} (limit=${:.6})",
                self.request_cost, per_request, self.conversation_cost, per_conversation
            ),
        }
    }
}

impl std::error::Error for BudgetExceededError {}

#[derive(Debug, Clone)]
pub struct RequestBudgetState {
    pub request_num: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationBudgetState {
    pub total_requests: u64,
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub variant_costs: Vec<VariantCost>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantCost {
    pub variant_id: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub cost_usd: f64,
}

pub struct BudgetTracker {
    conversation_budget: Arc<AtomicU64>,
    request_budget: Arc<AtomicU64>,
    total_prompt_tokens: Arc<AtomicU64>,
    total_completion_tokens: Arc<AtomicU64>,
    total_requests: Arc<AtomicU64>,
    variant_costs: Arc<std::sync::Mutex<Vec<VariantCost>>>,
    reasoning_budget: Option<ReasoningBudget>,
    cost_per_1k_tokens: f64,
}

impl BudgetTracker {
    pub fn new() -> Self {
        Self::with_reasoning_budget(None, 0.0)
    }

    pub fn with_reasoning_budget(
        reasoning_budget: Option<ReasoningBudget>,
        cost_per_1k_tokens: f64,
    ) -> Self {
        Self {
            conversation_budget: Arc::new(AtomicU64::new(0)),
            request_budget: Arc::new(AtomicU64::new(0)),
            total_prompt_tokens: Arc::new(AtomicU64::new(0)),
            total_completion_tokens: Arc::new(AtomicU64::new(0)),
            total_requests: Arc::new(AtomicU64::new(0)),
            variant_costs: Arc::new(std::sync::Mutex::new(Vec::new())),
            reasoning_budget,
            cost_per_1k_tokens,
        }
    }

    pub fn with_conversation_limit(self, limit_cents: u64) -> Self {
        self.conversation_budget
            .store(limit_cents, Ordering::Relaxed);
        self
    }

    pub fn with_request_limit(self, limit_cents: u64) -> Self {
        self.request_budget.store(limit_cents, Ordering::Relaxed);
        self
    }

    pub fn with_cost_per_1k_tokens(mut self, cost: f64) -> Self {
        self.cost_per_1k_tokens = cost;
        self
    }

    pub fn reasoning_budget(&self) -> Option<ReasoningBudget> {
        self.reasoning_budget
    }

    pub fn set_reasoning_budget(&mut self, budget: ReasoningBudget) {
        self.reasoning_budget = Some(budget);
    }

    pub fn record_usage(&self, usage: &Usage) -> RequestBudgetState {
        let cost = (usage.total_tokens as f64 / 1000.0) * self.cost_per_1k_tokens;
        let cost_microcents = (cost * 1_000_000.0).round() as u64;

        let request_num = self.total_requests.fetch_add(1, Ordering::Relaxed) + 1;
        let prompt = self
            .total_prompt_tokens
            .fetch_add(usage.prompt_tokens, Ordering::Relaxed);
        let completion = self
            .total_completion_tokens
            .fetch_add(usage.completion_tokens, Ordering::Relaxed);

        RequestBudgetState {
            request_num,
            prompt_tokens: prompt + usage.prompt_tokens,
            completion_tokens: completion + usage.completion_tokens,
            total_tokens: (prompt + completion + usage.total_tokens),
            cost_usd: cost_microcents as f64 / 1_000_000.0,
        }
    }

    pub fn record_variant_usage(&self, variant_id: &str, usage: &Usage) -> VariantCost {
        let cost = (usage.total_tokens as f64 / 1000.0) * self.cost_per_1k_tokens;
        let cost_microcents = (cost * 1_000_000.0).round() as u64;

        let variant_cost = VariantCost {
            variant_id: variant_id.to_string(),
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
            cost_usd: cost_microcents as f64 / 1_000_000.0,
        };

        if let Ok(mut variants) = self.variant_costs.lock() {
            variants.push(variant_cost.clone());
        }

        variant_cost
    }

    pub fn check_request_budget(&self, usage: &Usage) -> Result<(), BudgetExceededError> {
        let cost = (usage.total_tokens as f64 / 1000.0) * self.cost_per_1k_tokens;
        let cost_microcents = (cost * 1_000_000.0).round() as u64;

        let limit = self.request_budget.load(Ordering::Relaxed);
        if limit > 0 && cost_microcents > limit {
            return Err(BudgetExceededError {
                limit_type: BudgetLimit::PerRequest(limit as f64 / 1_000_000.0),
                request_cost: cost,
                conversation_cost: 0.0,
            });
        }
        Ok(())
    }

    pub fn check_conversation_budget(
        &self,
        additional_cost: f64,
    ) -> Result<(), BudgetExceededError> {
        let limit = self.conversation_budget.load(Ordering::Relaxed);
        if limit > 0 {
            let current_cost = self.total_cost_usd();
            if current_cost + additional_cost > limit as f64 / 1_000_000.0 {
                return Err(BudgetExceededError {
                    limit_type: BudgetLimit::PerConversation(limit as f64 / 1_000_000.0),
                    request_cost: additional_cost,
                    conversation_cost: current_cost,
                });
            }
        }
        Ok(())
    }

    pub fn get_request_state(&self) -> RequestBudgetState {
        let request_num = self.total_requests.load(Ordering::Relaxed);
        let prompt = self.total_prompt_tokens.load(Ordering::Relaxed);
        let completion = self.total_completion_tokens.load(Ordering::Relaxed);

        RequestBudgetState {
            request_num,
            prompt_tokens: prompt,
            completion_tokens: completion,
            total_tokens: prompt + completion,
            cost_usd: self.total_cost_usd(),
        }
    }

    pub fn get_conversation_state(&self) -> ConversationBudgetState {
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let total_prompt_tokens = self.total_prompt_tokens.load(Ordering::Relaxed);
        let total_completion_tokens = self.total_completion_tokens.load(Ordering::Relaxed);

        ConversationBudgetState {
            total_requests,
            total_prompt_tokens,
            total_completion_tokens,
            total_tokens: total_prompt_tokens + total_completion_tokens,
            total_cost_usd: self.total_cost_usd(),
            variant_costs: self
                .variant_costs
                .lock()
                .map(|v| v.clone())
                .unwrap_or_default(),
        }
    }

    pub fn total_cost_usd(&self) -> f64 {
        let total_tokens = self.total_prompt_tokens.load(Ordering::Relaxed)
            + self.total_completion_tokens.load(Ordering::Relaxed);
        (total_tokens as f64 / 1000.0) * self.cost_per_1k_tokens
    }

    pub fn remaining_request_budget(&self) -> Option<f64> {
        let limit = self.request_budget.load(Ordering::Relaxed);
        if limit == 0 {
            None
        } else {
            let current = (self.total_cost_usd() * 1_000_000.0).round() as u64;
            Some((limit.saturating_sub(current)) as f64 / 1_000_000.0)
        }
    }

    pub fn remaining_conversation_budget(&self) -> Option<f64> {
        let limit = self.conversation_budget.load(Ordering::Relaxed);
        if limit == 0 {
            None
        } else {
            let current = (self.total_cost_usd() * 1_000_000.0).round() as u64;
            Some((limit.saturating_sub(current)) as f64 / 1_000_000.0)
        }
    }

    pub fn reset_request_budget(&self) {
        self.request_budget.store(0, Ordering::Relaxed);
    }

    pub fn reset_conversation_budget(&self) {
        self.conversation_budget.store(0, Ordering::Relaxed);
        self.total_prompt_tokens.store(0, Ordering::Relaxed);
        self.total_completion_tokens.store(0, Ordering::Relaxed);
        self.total_requests.store(0, Ordering::Relaxed);
        if let Ok(mut variants) = self.variant_costs.lock() {
            variants.clear();
        }
    }
}

impl Default for BudgetTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for BudgetTracker {
    fn clone(&self) -> Self {
        Self {
            conversation_budget: self.conversation_budget.clone(),
            request_budget: self.request_budget.clone(),
            total_prompt_tokens: self.total_prompt_tokens.clone(),
            total_completion_tokens: self.total_completion_tokens.clone(),
            total_requests: self.total_requests.clone(),
            variant_costs: self.variant_costs.clone(),
            reasoning_budget: self.reasoning_budget,
            cost_per_1k_tokens: self.cost_per_1k_tokens,
        }
    }
}

pub struct StreamingBudgetTracker<'a> {
    tracker: &'a BudgetTracker,
    accumulated_chars: usize,
    accumulated_tokens: u64,
    chars_per_token: f64,
}

impl<'a> StreamingBudgetTracker<'a> {
    pub fn new(tracker: &'a BudgetTracker) -> Self {
        Self {
            tracker,
            accumulated_chars: 0,
            accumulated_tokens: 0,
            chars_per_token: 4.0,
        }
    }

    pub fn with_chars_per_token(mut self, chars_per_token: f64) -> Self {
        self.chars_per_token = chars_per_token;
        self
    }

    pub fn estimate_tokens(&self, text: &str) -> u64 {
        (text.len() as f64 / self.chars_per_token).ceil() as u64
    }

    pub fn record_chunk(&mut self, text: &str) -> u64 {
        let tokens = self.estimate_tokens(text);
        self.accumulated_chars += text.len();
        self.accumulated_tokens += tokens;
        tokens
    }

    pub fn check_request_budget(&self, additional_tokens: u64) -> Result<(), BudgetExceededError> {
        let total_tokens = self.accumulated_tokens + additional_tokens;
        let cost = (total_tokens as f64 / 1000.0) * self.tracker.cost_per_1k_tokens;
        let cost_microcents = (cost * 1_000_000.0).round() as u64;

        let limit = self.tracker.request_budget.load(Ordering::Relaxed);
        if limit > 0 && cost_microcents > limit {
            return Err(BudgetExceededError {
                limit_type: BudgetLimit::PerRequest(limit as f64 / 1_000_000.0),
                request_cost: cost,
                conversation_cost: self.tracker.total_cost_usd(),
            });
        }
        Ok(())
    }

    pub fn check_conversation_budget(
        &self,
        additional_tokens: u64,
    ) -> Result<(), BudgetExceededError> {
        let additional_cost = (additional_tokens as f64 / 1000.0) * self.tracker.cost_per_1k_tokens;
        self.tracker.check_conversation_budget(additional_cost)
    }

    pub fn check_budget(&self, additional_tokens: u64) -> Result<(), BudgetExceededError> {
        self.check_request_budget(additional_tokens)?;
        self.check_conversation_budget(additional_tokens)
    }

    pub fn accumulated(&self) -> u64 {
        self.accumulated_tokens
    }

    pub fn estimate_total_cost(&self) -> f64 {
        (self.accumulated_tokens as f64 / 1000.0) * self.tracker.cost_per_1k_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_new() {
        let usage = Usage::new(100, 200);
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 200);
        assert_eq!(usage.total_tokens, 300);
    }

    #[test]
    fn test_usage_calculate_cost() {
        let usage = Usage::new(1000, 1000);
        let cost = usage.calculate_cost(0.001);
        assert!((cost - 0.002).abs() < 0.0001);
    }

    #[test]
    fn test_budget_tracker_record_usage() {
        let tracker = BudgetTracker::new();
        let usage = Usage::new(100, 50);

        let state = tracker.record_usage(&usage);

        assert_eq!(state.request_num, 1);
        assert_eq!(state.prompt_tokens, 100);
        assert_eq!(state.completion_tokens, 50);
        assert_eq!(state.total_tokens, 150);
    }

    #[test]
    fn test_budget_tracker_multiple_requests() {
        let tracker = BudgetTracker::new();
        tracker.record_usage(&Usage::new(100, 50));
        tracker.record_usage(&Usage::new(200, 100));

        let state = tracker.get_conversation_state();
        assert_eq!(state.total_requests, 2);
        assert_eq!(state.total_prompt_tokens, 300);
        assert_eq!(state.total_completion_tokens, 150);
    }

    #[test]
    fn test_budget_tracker_variant_costs() {
        let tracker = BudgetTracker::new();
        tracker.record_variant_usage("variant1", &Usage::new(100, 50));
        tracker.record_variant_usage("variant2", &Usage::new(200, 100));

        let state = tracker.get_conversation_state();
        assert_eq!(state.variant_costs.len(), 2);
        assert_eq!(state.variant_costs[0].variant_id, "variant1");
        assert_eq!(state.variant_costs[1].variant_id, "variant2");
    }

    #[test]
    fn test_budget_limit_per_request_exceeded() {
        let limit = BudgetLimit::PerRequest(0.001);
        assert!(limit.is_exceeded(0.0015, 0.0));
        assert!(!limit.is_exceeded(0.0005, 0.0));
    }

    #[test]
    fn test_budget_limit_per_conversation_exceeded() {
        let limit = BudgetLimit::PerConversation(0.01);
        assert!(limit.is_exceeded(0.0, 0.015));
        assert!(!limit.is_exceeded(0.0, 0.005));
    }

    #[test]
    fn test_budget_limit_combined_exceeded() {
        let limit = BudgetLimit::Combined {
            per_request: 0.005,
            per_conversation: 0.01,
        };
        assert!(limit.is_exceeded(0.006, 0.0));
        assert!(limit.is_exceeded(0.0, 0.015));
        assert!(!limit.is_exceeded(0.004, 0.009));
    }

    #[test]
    fn test_budget_tracker_request_limit() {
        let tracker = BudgetTracker::with_reasoning_budget(None, 0.001);
        let tracker = BudgetTracker::with_request_limit(tracker, 500);

        let small_usage = Usage::new(100, 100);
        let result = tracker.check_request_budget(&small_usage);
        assert!(result.is_ok());

        let large_usage = Usage::new(1000, 1000);
        let result = tracker.check_request_budget(&large_usage);
        assert!(result.is_err());
    }

    #[test]
    fn test_budget_tracker_remaining_budget() {
        let tracker = BudgetTracker::with_reasoning_budget(None, 0.001);
        let tracker = BudgetTracker::with_request_limit(tracker, 1000);

        let usage = Usage::new(100, 100);
        tracker.record_usage(&usage);

        let remaining = tracker.remaining_request_budget();
        assert!(remaining.is_some());
    }

    #[test]
    fn test_budget_tracker_reset() {
        let tracker = BudgetTracker::new();
        tracker.record_usage(&Usage::new(100, 50));

        tracker.reset_conversation_budget();

        let state = tracker.get_conversation_state();
        assert_eq!(state.total_requests, 0);
        assert_eq!(state.total_prompt_tokens, 0);
    }

    #[test]
    fn test_conversation_budget_state_serialization() {
        let state = ConversationBudgetState {
            total_requests: 5,
            total_prompt_tokens: 1000,
            total_completion_tokens: 500,
            total_tokens: 1500,
            total_cost_usd: 0.015,
            variant_costs: vec![VariantCost {
                variant_id: "v1".to_string(),
                prompt_tokens: 500,
                completion_tokens: 250,
                total_tokens: 750,
                cost_usd: 0.0075,
            }],
        };

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: ConversationBudgetState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_requests, 5);
        assert_eq!(deserialized.variant_costs.len(), 1);
        assert_eq!(deserialized.variant_costs[0].variant_id, "v1");
    }

    #[test]
    fn test_streaming_budget_tracker_estimate_tokens() {
        let tracker = BudgetTracker::new();
        let stream_tracker = StreamingBudgetTracker::new(&tracker);

        assert_eq!(stream_tracker.estimate_tokens("hello"), 2);
        assert_eq!(stream_tracker.estimate_tokens("hello world"), 3);
    }

    #[test]
    fn test_streaming_budget_tracker_record_chunk() {
        let tracker = BudgetTracker::new();
        let mut stream_tracker = StreamingBudgetTracker::new(&tracker);

        stream_tracker.record_chunk("hello");
        assert_eq!(stream_tracker.accumulated(), 2);

        stream_tracker.record_chunk(" world");
        assert_eq!(stream_tracker.accumulated(), 4);
    }

    #[test]
    fn test_streaming_budget_tracker_custom_chars_per_token() {
        let tracker = BudgetTracker::new();
        let stream_tracker = StreamingBudgetTracker::new(&tracker).with_chars_per_token(5.0);

        assert_eq!(stream_tracker.estimate_tokens("hello"), 1);
        assert_eq!(stream_tracker.estimate_tokens("hello world"), 3);
    }

    #[test]
    fn test_streaming_budget_tracker_check_request_budget() {
        let tracker = BudgetTracker::with_reasoning_budget(None, 0.001);
        let tracker = BudgetTracker::with_request_limit(tracker, 500);
        let stream_tracker = StreamingBudgetTracker::new(&tracker);

        let result = stream_tracker.check_request_budget(100);
        assert!(result.is_ok());

        let result = stream_tracker.check_request_budget(1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_streaming_budget_tracker_estimate_cost() {
        let tracker = BudgetTracker::with_reasoning_budget(None, 0.001);
        let mut stream_tracker = StreamingBudgetTracker::new(&tracker);

        stream_tracker.record_chunk("hello world");

        let cost = stream_tracker.estimate_total_cost();
        assert!((cost - 0.000003).abs() < 0.000001);
    }
}
