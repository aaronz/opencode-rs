use crate::message::{Message, Role};
use std::collections::HashMap;
use thiserror::Error;
use tiktoken_rs::{cl100k_base, p50k_base, r50k_base, CoreBPE};
use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EncodingKind {
    Cl100k,
    P50k,
    R50k,
}

impl EncodingKind {
    fn from_model(model: &str) -> Self {
        let model = model.to_ascii_lowercase();

        if model.contains("gpt-4") || model.contains("gpt-3.5") {
            Self::Cl100k
        } else if model.contains("code-davinci") {
            Self::P50k
        } else {
            Self::R50k
        }
    }

    fn from_name(name: &str) -> Result<Self, TokenCounterError> {
        match name {
            "cl100k_base" => Ok(Self::Cl100k),
            "p50k_base" => Ok(Self::P50k),
            "r50k_base" => Ok(Self::R50k),
            other => Err(TokenCounterError::UnknownEncoding(other.to_string())),
        }
    }

    fn as_name(&self) -> &'static str {
        match self {
            Self::Cl100k => "cl100k_base",
            Self::P50k => "p50k_base",
            Self::R50k => "r50k_base",
        }
    }
}

#[derive(Debug, Error)]
pub enum TokenCounterError {
    #[error("unknown token encoding: {0}")]
    UnknownEncoding(String),
    #[error("failed to load cl100k_base encoder: {0}")]
    Cl100kLoad(String),
    #[error("failed to load p50k_base encoder: {0}")]
    P50kLoad(String),
    #[error("failed to load r50k_base encoder: {0}")]
    R50kLoad(String),
}

pub struct TokenCounter {
    model: String,
    encoder: Option<CoreBPE>,
    total_tokens: usize,
    active_session_id: Option<String>,
    session_tokens: HashMap<String, usize>,
}

impl std::fmt::Debug for TokenCounter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenCounter")
            .field("model", &self.model)
            .field("encoder", &self.encoder.as_ref().map(|_| "<CoreBPE>"))
            .field("total_tokens", &self.total_tokens)
            .field("active_session_id", &self.active_session_id)
            .field("session_tokens", &self.session_tokens)
            .finish()
    }
}

impl Clone for TokenCounter {
    fn clone(&self) -> Self {
        let encoder = self.encoder.as_ref().and_then(|_| {
            // Re-load the encoder for the same model
            Self::load_encoder(EncodingKind::from_model(&self.model)).ok()
        });
        Self {
            model: self.model.clone(),
            encoder,
            total_tokens: self.total_tokens,
            active_session_id: self.active_session_id.clone(),
            session_tokens: self.session_tokens.clone(),
        }
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::for_model("gpt-4o")
    }
}

impl TokenCounter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn for_model(model: &str) -> Self {
        Self::for_model_and_encoding(model, None)
    }

    fn for_model_and_encoding(model: &str, encoding_name: Option<&str>) -> Self {
        let encoder_result = match encoding_name {
            Some(name) => EncodingKind::from_name(name).and_then(Self::load_encoder),
            None => Self::load_encoder(EncodingKind::from_model(model)),
        };

        let encoder = match encoder_result {
            Ok(encoder) => Some(encoder),
            Err(error) => {
                warn!(model = model, error = %error, "tiktoken unavailable; using char/4 fallback");
                None
            }
        };

        Self {
            model: model.to_string(),
            encoder,
            total_tokens: 0,
            active_session_id: None,
            session_tokens: HashMap::new(),
        }
    }

    fn load_encoder(kind: EncodingKind) -> Result<CoreBPE, TokenCounterError> {
        match kind {
            EncodingKind::Cl100k => {
                cl100k_base().map_err(|e| TokenCounterError::Cl100kLoad(e.to_string()))
            }
            EncodingKind::P50k => {
                p50k_base().map_err(|e| TokenCounterError::P50kLoad(e.to_string()))
            }
            EncodingKind::R50k => {
                r50k_base().map_err(|e| TokenCounterError::R50kLoad(e.to_string()))
            }
        }
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn encoding_name(&self) -> &'static str {
        EncodingKind::from_model(&self.model).as_name()
    }

    pub fn count_tokens(&self, text: &str) -> usize {
        if let Some(encoder) = &self.encoder {
            encoder.encode_with_special_tokens(text).len()
        } else {
            Self::estimate_tokens_fallback(text)
        }
    }

    pub fn count_messages(&self, messages: &[Message]) -> usize {
        messages
            .iter()
            .map(|message| {
                let role = match message.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                };
                self.count_tokens(role)
                    .saturating_add(self.count_tokens(&message.content))
            })
            .sum()
    }

    pub fn estimate_tokens_fallback(text: &str) -> usize {
        (text.chars().count() + 3) / 4
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

    pub fn is_fallback(&self) -> bool {
        self.encoder.is_none()
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
    fn token_counting_model_mapping_correct() {
        let gpt = TokenCounter::for_model("gpt-4o");
        let gpt35 = TokenCounter::for_model("gpt-3.5-turbo");
        let davinci = TokenCounter::for_model("code-davinci-002");
        let unknown = TokenCounter::for_model("my-custom-model");

        assert_eq!(gpt.encoding_name(), "cl100k_base");
        assert_eq!(gpt35.encoding_name(), "cl100k_base");
        assert_eq!(davinci.encoding_name(), "p50k_base");
        assert_eq!(unknown.encoding_name(), "r50k_base");
    }

    #[test]
    fn token_counting_returns_reasonable_values() {
        let counter = TokenCounter::for_model("gpt-4o");
        assert_eq!(counter.count_tokens(""), 0);
        assert!(counter.count_tokens("hello world") >= 2);
        assert!(counter.count_tokens("OpenAI builds useful tools.") >= 4);
    }

    #[test]
    fn fallback_behavior_when_encoder_loading_fails() {
        let counter = TokenCounter::for_model_and_encoding("gpt-4o", Some("invalid_encoding"));
        assert!(counter.is_fallback());
        assert_eq!(counter.count_tokens("abcd"), 1);
        assert_eq!(counter.count_tokens("abcde"), 2);
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
