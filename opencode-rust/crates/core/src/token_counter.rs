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
        text.chars().count().div_ceil(4)
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

    #[test]
    fn tiktoken_calibration_single_tokens() {
        let counter = TokenCounter::for_model("gpt-4o");
        if counter.is_fallback() {
            return;
        }
        assert_eq!(counter.count_tokens("a"), 1);
        assert_eq!(counter.count_tokens("ab"), 1);
        assert_eq!(counter.count_tokens("abc"), 1);
        assert_eq!(counter.count_tokens("abcd"), 1);
        assert_eq!(counter.count_tokens("abcde"), 2);
    }

    #[test]
    fn tiktoken_calibration_common_tokens() {
        let counter = TokenCounter::for_model("gpt-4o");
        if counter.is_fallback() {
            return;
        }
        let tokens1 = counter.count_tokens(" the ");
        assert!(
            (1..=3).contains(&tokens1),
            "expected 1-3 tokens for ' the ', got {}",
            tokens1
        );
        let tokens2 = counter.count_tokens("ing");
        assert!(
            (1..=2).contains(&tokens2),
            "expected 1-2 tokens for 'ing', got {}",
            tokens2
        );
    }

    #[test]
    fn tiktoken_calibration_english_text() {
        let counter = TokenCounter::for_model("gpt-4o");
        if counter.is_fallback() {
            return;
        }
        let text = "The quick brown fox jumps over the lazy dog.";
        let tokens = counter.count_tokens(text);
        assert!(
            (9..=12).contains(&tokens),
            "expected 9-12 tokens, got {}",
            tokens
        );
    }

    #[test]
    fn tiktoken_calibration_code_snippet() {
        let counter = TokenCounter::for_model("gpt-4o");
        if counter.is_fallback() {
            return;
        }
        let code = "fn main() {\n    println!(\"Hello, world!\");\n}";
        let tokens = counter.count_tokens(code);
        assert!(
            (8..=30).contains(&tokens),
            "expected 8-30 tokens, got {}",
            tokens
        );
    }

    #[test]
    fn tiktoken_calibration_chinese_text() {
        let counter = TokenCounter::for_model("gpt-4o");
        if counter.is_fallback() {
            return;
        }
        let text = "你好世界";
        let tokens = counter.count_tokens(text);
        assert!(
            (4..=8).contains(&tokens),
            "expected 4-8 tokens for Chinese, got {}",
            tokens
        );
    }

    #[test]
    fn tiktoken_calibration_special_characters() {
        let counter = TokenCounter::for_model("gpt-4o");
        if counter.is_fallback() {
            return;
        }
        assert_eq!(counter.count_tokens(""), 0);
        let spaces = counter.count_tokens("   ");
        assert!(
            (1..=3).contains(&spaces),
            "expected 1-3 tokens for spaces, got {}",
            spaces
        );
        let newlines = counter.count_tokens("\n\n");
        assert!(
            (1..=4).contains(&newlines),
            "expected 1-4 tokens for newlines, got {}",
            newlines
        );
        let emoji = counter.count_tokens("🎉");
        assert!(
            (1..=4).contains(&emoji),
            "expected 1-4 tokens for emoji, got {}",
            emoji
        );
    }

    #[test]
    fn tiktoken_calibration_longer_text() {
        let counter = TokenCounter::for_model("gpt-4o");
        if counter.is_fallback() {
            return;
        }
        let text = "This is a longer piece of text that should contain multiple tokens when encoded with cl100k_base encoding. It includes multiple words and spaces and punctuation marks.";
        let tokens = counter.count_tokens(text);
        let fallback = TokenCounter::estimate_tokens_fallback(text);
        let ratio = tokens as f64 / fallback as f64;
        assert!(
            ratio > 0.5 && ratio < 2.0,
            "tiktoken count {} seems unreasonable compared to fallback {} (ratio: {:.2})",
            tokens,
            fallback,
            ratio
        );
    }

    #[test]
    fn tiktoken_encoding_consistency() {
        let counter1 = TokenCounter::for_model("gpt-4o");
        let counter2 = TokenCounter::for_model("gpt-4o");
        if counter1.is_fallback() || counter2.is_fallback() {
            return;
        }
        let text = "Consistency test string for token counting";
        assert_eq!(counter1.count_tokens(text), counter2.count_tokens(text));
    }

    #[test]
    fn fallback_vs_tiktoken_accuracy() {
        let counter = TokenCounter::for_model("gpt-4o");
        if counter.is_fallback() {
            return;
        }
        let test_cases = vec![
            ("hello", 1..3),
            ("hello world", 2..4),
            ("The quick brown fox", 4..6),
            ("function add(a, b) { return a + b; }", 8..20),
        ];
        for (text, expected_range) in test_cases {
            let count = counter.count_tokens(text);
            let fallback = TokenCounter::estimate_tokens_fallback(text);
            assert!(
                expected_range.contains(&count),
                "tiktoken count {} for '{}' outside expected range {:?}, fallback={}",
                count,
                text,
                expected_range,
                fallback
            );
        }
    }

    #[test]
    fn tiktoken_special_tokens_handling() {
        let counter = TokenCounter::for_model("gpt-4o");
        if counter.is_fallback() {
            return;
        }
        let tokens = counter.count_tokens("Hello <|end|>");
        assert!(
            tokens >= 2,
            "special token should be counted as separate token"
        );
    }

    #[test]
    fn tiktoken_repeating_characters() {
        let counter = TokenCounter::for_model("gpt-4o");
        if counter.is_fallback() {
            return;
        }
        let single = counter.count_tokens("a");
        let repeated = counter.count_tokens("aaaaaaaaaa");
        assert!(
            repeated >= single,
            "repeated chars should have at least as many tokens as single char"
        );
    }
}
