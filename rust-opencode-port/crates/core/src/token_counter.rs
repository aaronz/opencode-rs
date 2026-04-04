use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct TokenCounter {
    total_tokens: usize,
    active_session_id: Option<String>,
    session_tokens: HashMap<String, usize>,
}

impl TokenCounter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_active_session(&mut self, session_id: impl Into<String>) {
        self.active_session_id = Some(session_id.into());
    }

    pub fn record_tokens(&mut self, _model: &str, input_tokens: usize, output_tokens: usize) {
        let total = input_tokens.saturating_add(output_tokens);
        self.total_tokens = self.total_tokens.saturating_add(total);

        if let Some(session_id) = &self.active_session_id {
            let entry = self.session_tokens.entry(session_id.clone()).or_insert(0);
            *entry = entry.saturating_add(total);
        }
    }

    pub fn get_total_tokens(&self) -> usize {
        self.total_tokens
    }

    pub fn get_session_tokens(&self, session_id: &str) -> usize {
        self.session_tokens.get(session_id).copied().unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
pub struct CostCalculator {
    pricing_per_1k: HashMap<String, (f64, f64)>,
}

impl Default for CostCalculator {
    fn default() -> Self {
        let mut pricing_per_1k = HashMap::new();
        pricing_per_1k.insert("gpt-4o".to_string(), (0.005, 0.015));
        pricing_per_1k.insert("gpt-4.1".to_string(), (0.002, 0.008));
        pricing_per_1k.insert("gpt-5.3-codex".to_string(), (0.003, 0.012));
        pricing_per_1k.insert("claude-3-5-sonnet".to_string(), (0.003, 0.015));

        Self { pricing_per_1k }
    }
}

impl CostCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_custom_pricing(mut self, pricing: HashMap<String, (f64, f64)>) -> Self {
        self.pricing_per_1k.extend(pricing);
        self
    }

    pub fn calculate_cost(&self, model: &str, input_tokens: usize, output_tokens: usize) -> f64 {
        let (input_price, output_price) = self
            .pricing_per_1k
            .get(model)
            .copied()
            .unwrap_or((0.0015, 0.006));

        (input_tokens as f64 / 1000.0) * input_price
            + (output_tokens as f64 / 1000.0) * output_price
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_total_and_session_tokens() {
        let mut counter = TokenCounter::new();
        counter.set_active_session("session-a");
        counter.record_tokens("gpt-4o", 100, 50);
        counter.record_tokens("gpt-4o", 20, 30);

        assert_eq!(counter.get_total_tokens(), 200);
        assert_eq!(counter.get_session_tokens("session-a"), 200);
    }

    #[test]
    fn calculates_cost_with_defaults_and_custom_pricing() {
        let default_calc = CostCalculator::new();
        let default_cost = default_calc.calculate_cost("unknown-model", 1000, 1000);
        assert!(default_cost > 0.0);

        let custom = CostCalculator::new()
            .with_custom_pricing(HashMap::from([("custom-model".to_string(), (0.01, 0.02))]));
        let custom_cost = custom.calculate_cost("custom-model", 1000, 500);
        assert!((custom_cost - 0.02).abs() < f64::EPSILON);
    }
}
