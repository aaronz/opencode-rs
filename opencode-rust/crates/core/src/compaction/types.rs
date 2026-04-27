use crate::message::{Message, Role};
use serde::{Deserialize, Serialize};
use tracing::warn;

pub const COMPACTION_WARN_THRESHOLD: f32 = 0.85;
pub const COMPACTION_START_THRESHOLD: f32 = 0.92;
pub const COMPACTION_FORCE_THRESHOLD: f32 = 0.95;

#[allow(dead_code)]
const CONTEXT_PRIORITY_SYSTEM: u8 = 100;
#[allow(dead_code)]
const CONTEXT_PRIORITY_PROMPT: u8 = 90;
#[allow(dead_code)]
const CONTEXT_PRIORITY_PROJECT: u8 = 80;
#[allow(dead_code)]
const CONTEXT_PRIORITY_SESSION: u8 = 50;
#[allow(dead_code)]
const CONTEXT_PRIORITY_TOOL: u8 = 20;

const TOKEN_BUDGET_MAIN_CONTEXT_PERCENT: f64 = 0.70;
const TOKEN_BUDGET_TOOL_OUTPUT_PERCENT: f64 = 0.20;
const TOKEN_BUDGET_RESPONSE_SPACE_PERCENT: f64 = 0.10;

#[allow(dead_code)]
const CONTEXT_RANKING_RECENTCY_WEIGHT: f64 = 0.4;
#[allow(dead_code)]
const CONTEXT_RANKING_RELEVANCE_WEIGHT: f64 = 0.3;
#[allow(dead_code)]
const CONTEXT_RANKING_IMPORTANCE_WEIGHT: f64 = 0.3;

pub(crate) const DEFAULT_MAX_TOKENS: usize = 100_000;
pub(crate) const DEFAULT_PRESERVE_RECENT_MESSAGES: usize = 10;
pub(crate) const SUMMARY_TOPICS_EXTRACTION_LIMIT: usize = 5;
pub(crate) const SUMMARY_PREVIEW_MAX_CHARS: usize = 80;
pub(crate) const MAX_COMPACTION_ITERATIONS: usize = 10;
pub(crate) const MIN_RESERVED_TOKENS_WARNING: u32 = 5000;
pub(crate) const DEFAULT_RESERVED_TOKENS: u32 = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CompactionError {
    InvalidReserved(u32),
    InvalidModelContext(usize),
    ReservedExceedsModelContext {
        reserved: u32,
        model_max_context: usize,
    },
}

impl std::fmt::Display for CompactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompactionError::InvalidReserved(value) => {
                write!(f, "reserved must be greater than 0, got {value}")
            }
            CompactionError::InvalidModelContext(value) => {
                write!(f, "model_max_context must be greater than 0, got {value}")
            }
            CompactionError::ReservedExceedsModelContext {
                reserved,
                model_max_context,
            } => write!(
                f,
                "reserved ({reserved}) must be lower than model_max_context ({model_max_context})"
            ),
        }
    }
}

impl std::error::Error for CompactionError {}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub(crate) enum ContextLevel {
    L0,
    L1,
    L2,
    L3,
    L4,
}

#[allow(dead_code)]
impl ContextLevel {
    pub(crate) fn priority(&self) -> u8 {
        match self {
            ContextLevel::L0 => CONTEXT_PRIORITY_SYSTEM,
            ContextLevel::L4 => CONTEXT_PRIORITY_PROMPT,
            ContextLevel::L1 => CONTEXT_PRIORITY_PROJECT,
            ContextLevel::L2 => CONTEXT_PRIORITY_SESSION,
            ContextLevel::L3 => CONTEXT_PRIORITY_TOOL,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    pub total: usize,
    #[serde(default = "TokenBudget::default_model_max_tokens")]
    pub model_max_tokens: usize,
    pub main_context_percent: f64,
    pub tool_output_percent: f64,
    pub response_space_percent: f64,
    pub warning_threshold: f64,
    pub compact_threshold: f64,
    pub continuation_threshold: f64,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            total: 128_000,
            model_max_tokens: Self::default_model_max_tokens(),
            main_context_percent: TOKEN_BUDGET_MAIN_CONTEXT_PERCENT,
            tool_output_percent: TOKEN_BUDGET_TOOL_OUTPUT_PERCENT,
            response_space_percent: TOKEN_BUDGET_RESPONSE_SPACE_PERCENT,
            warning_threshold: COMPACTION_WARN_THRESHOLD as f64,
            compact_threshold: COMPACTION_START_THRESHOLD as f64,
            continuation_threshold: COMPACTION_FORCE_THRESHOLD as f64,
        }
    }
}

impl TokenBudget {
    const fn default_model_max_tokens() -> usize {
        128_000
    }

    pub fn from_model(model_name: &str) -> Self {
        let model = model_name.trim().to_ascii_lowercase();
        let model_max_tokens = match model.as_str() {
            "gpt-4o" | "gpt-4-turbo" => 128_000,
            "gpt-4" => 8_192,
            "gpt-3.5-turbo" => 16_385,
            "claude-3-5-sonnet" | "claude-3-opus" | "claude-3-haiku" | "claude-2.1" => 200_000,
            _ => 128_000,
        };

        Self {
            total: model_max_tokens,
            model_max_tokens,
            ..Self::default()
        }
    }

    pub fn with_thresholds(
        mut self,
        warning_threshold: f64,
        compact_threshold: f64,
        continuation_threshold: f64,
    ) -> Self {
        self.warning_threshold = warning_threshold;
        self.compact_threshold = compact_threshold;
        self.continuation_threshold = continuation_threshold;
        self
    }

    pub fn from_config(config: &crate::config::CompactionConfig, model_max_context: usize) -> Self {
        let mut budget = if model_max_context > 0 {
            Self {
                total: model_max_context,
                model_max_tokens: model_max_context,
                ..Self::default()
            }
        } else {
            Self::default()
        };

        if let Some(warning) = config.warning_threshold {
            budget.warning_threshold = warning.clamp(0.0, 1.0);
        }
        if let Some(compact) = config.compact_threshold {
            budget.compact_threshold = compact.clamp(0.0, 1.0);
        }
        if let Some(continuation) = config.continuation_threshold {
            budget.continuation_threshold = continuation.clamp(0.0, 1.0);
        }

        budget
    }

    pub fn main_context_tokens(&self) -> usize {
        (self.total as f64 * self.main_context_percent).ceil() as usize
    }

    pub fn tool_output_tokens(&self) -> usize {
        (self.total as f64 * self.tool_output_percent).ceil() as usize
    }

    pub fn response_space_tokens(&self) -> usize {
        (self.total as f64 * self.response_space_percent).ceil() as usize
    }

    pub fn warning_limit(&self) -> usize {
        (self.total as f64 * self.warning_threshold).ceil() as usize
    }

    pub fn compact_limit(&self) -> usize {
        (self.total as f64 * self.compact_threshold).ceil() as usize
    }

    pub fn continuation_limit(&self) -> usize {
        (self.total as f64 * self.continuation_threshold).ceil() as usize
    }

    pub fn usage_level(&self, used: usize) -> CompactionLevel {
        let ratio = used as f64 / self.total as f64;
        if ratio >= self.continuation_threshold {
            CompactionLevel::ForceContinuation
        } else if ratio >= self.compact_threshold {
            CompactionLevel::AutoCompact
        } else if ratio >= self.warning_threshold {
            CompactionLevel::Warning
        } else {
            CompactionLevel::Normal
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CompactionLevel {
    Normal,
    Warning,
    AutoCompact,
    ForceContinuation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub(crate) struct ContextRanking {
    pub message_index: usize,
    pub recency: f64,
    pub relevance: f64,
    pub importance: f64,
    pub overall: f64,
}

#[allow(dead_code)]
impl ContextRanking {
    pub(crate) fn new(message_index: usize, recency: f64, relevance: f64, importance: f64) -> Self {
        let overall = recency * CONTEXT_RANKING_RECENTCY_WEIGHT
            + relevance * CONTEXT_RANKING_RELEVANCE_WEIGHT
            + importance * CONTEXT_RANKING_IMPORTANCE_WEIGHT;
        Self {
            message_index,
            recency,
            relevance,
            importance,
            overall,
        }
    }

    pub(crate) fn default_for_index(index: usize, total: usize) -> Self {
        let recency = if total == 0 {
            1.0
        } else {
            1.0 - (index as f64 / total as f64)
        };
        Self::new(index, recency, 0.5, 0.5)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CompactionTrigger {
    None,
    Warning,
    AutoCompact,
    ForceContinuation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionStatus {
    pub used_tokens: usize,
    pub total_budget: usize,
    pub usage_ratio: f64,
    pub trigger: CompactionTrigger,
    pub needs_attention: bool,
}

impl CompactionStatus {
    pub fn check(budget: &TokenBudget, used: usize) -> Self {
        let usage_ratio = used as f64 / budget.total as f64;
        let (trigger, needs_attention) = match budget.usage_level(used) {
            CompactionLevel::Normal => (CompactionTrigger::None, false),
            CompactionLevel::Warning => (CompactionTrigger::Warning, true),
            CompactionLevel::AutoCompact => (CompactionTrigger::AutoCompact, true),
            CompactionLevel::ForceContinuation => (CompactionTrigger::ForceContinuation, true),
        };
        Self {
            used_tokens: used,
            total_budget: budget.total,
            usage_ratio,
            trigger,
            needs_attention,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    pub max_tokens: usize,
    pub preserve_system_messages: bool,
    pub preserve_recent_messages: usize,
    pub summary_prefix: String,
    pub token_budget: TokenBudget,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            max_tokens: DEFAULT_MAX_TOKENS,
            preserve_system_messages: true,
            preserve_recent_messages: DEFAULT_PRESERVE_RECENT_MESSAGES,
            summary_prefix: "[Context compacted]".to_string(),
            token_budget: TokenBudget::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompactionResult {
    pub messages: Vec<Message>,
    pub was_compacted: bool,
    pub pruned_count: usize,
    pub summary_inserted: bool,
}
