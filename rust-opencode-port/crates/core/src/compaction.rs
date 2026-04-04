use crate::message::{Message, Role};
use serde::{Deserialize, Serialize};
use tracing::warn;

const DEFAULT_KEEP_RECENT_TOOL_OUTPUTS: usize = 3;
const PRUNED_TOOL_OUTPUT_PLACEHOLDER: &str = "[content pruned to save tokens]";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompactionError {
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

/// Context hierarchy levels per PRD Section 7.6
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ContextLevel {
    /// L0: System prompt
    L0,
    /// L1: Project context (workspace, git info)
    L1,
    /// L2: Recent session history
    L2,
    /// L3: Tool definitions and results
    L3,
    /// L4: Current prompt context
    L4,
}

impl ContextLevel {
    pub fn priority(&self) -> u8 {
        match self {
            ContextLevel::L0 => 100, // System - highest priority
            ContextLevel::L4 => 90,  // Current prompt - very high
            ContextLevel::L1 => 80,  // Project context - high
            ContextLevel::L2 => 50,  // Recent session - medium
            ContextLevel::L3 => 20,  // Tool results - lower
        }
    }
}

/// Token budget distribution per PRD Section 7.6
/// - 70% main context
/// - 20% tool output
/// - 10% response space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    /// Total budget in tokens (default 128K)
    pub total: usize,
    #[serde(default = "TokenBudget::default_model_max_tokens")]
    pub model_max_tokens: usize,
    /// Percentage for main context (default 70%)
    pub main_context_percent: f64,
    /// Percentage for tool output (default 20%)
    pub tool_output_percent: f64,
    /// Percentage for response space (default 10%)
    pub response_space_percent: f64,
    /// Threshold for warning (85%)
    pub warning_threshold: f64,
    /// Threshold for auto compact (92%)
    pub compact_threshold: f64,
    /// Threshold for session continuation (95%)
    pub continuation_threshold: f64,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            total: 128_000,
            model_max_tokens: Self::default_model_max_tokens(),
            main_context_percent: 0.70,
            tool_output_percent: 0.20,
            response_space_percent: 0.10,
            warning_threshold: 0.85,
            compact_threshold: 0.92,
            continuation_threshold: 0.95,
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

/// Compaction level based on usage
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CompactionLevel {
    Normal,
    Warning,
    AutoCompact,
    ForceContinuation,
}

/// Context ranking score for messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRanking {
    /// Message index in conversation
    pub message_index: usize,
    /// Recency score (0-1, higher = more recent)
    pub recency: f64,
    /// Relevance score (0-1, based on embedding similarity)
    pub relevance: f64,
    /// Importance score (0-1, based on tool results, errors, confirmations)
    pub importance: f64,
    /// Overall score weighted: recency(0.4) + relevance(0.3) + importance(0.3)
    pub overall: f64,
}

impl ContextRanking {
    pub fn new(message_index: usize, recency: f64, relevance: f64, importance: f64) -> Self {
        let overall = recency * 0.4 + relevance * 0.3 + importance * 0.3;
        Self {
            message_index,
            recency,
            relevance,
            importance,
            overall,
        }
    }

    /// Create ranking with default values (used when no explicit ranking needed)
    pub fn default_for_index(index: usize, total: usize) -> Self {
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
            max_tokens: 100_000,
            preserve_system_messages: true,
            preserve_recent_messages: 10,
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

pub struct Compactor {
    config: CompactionConfig,
}

impl Compactor {
    pub fn new(config: CompactionConfig) -> Self {
        Self { config }
    }

    pub fn needs_compaction(&self, messages: &[Message]) -> bool {
        let total: usize = messages
            .iter()
            .map(|m| self.estimate_tokens(&m.content))
            .sum();
        total > self.config.max_tokens
    }

    pub fn compact(&self, messages: Vec<Message>) -> CompactionResult {
        let total_tokens: usize = messages
            .iter()
            .map(|m| self.estimate_tokens(&m.content))
            .sum();

        if total_tokens <= self.config.max_tokens {
            return CompactionResult {
                messages,
                was_compacted: false,
                pruned_count: 0,
                summary_inserted: false,
            };
        }

        let mut system_messages: Vec<Message> = Vec::new();
        let mut non_system: Vec<Message> = Vec::new();

        for msg in messages {
            if self.config.preserve_system_messages && msg.role == Role::System {
                system_messages.push(msg);
            } else {
                non_system.push(msg);
            }
        }

        let recent_count = self.config.preserve_recent_messages.min(non_system.len());
        let split_at = non_system.len().saturating_sub(recent_count);
        let pruned: Vec<Message> = non_system[..split_at].to_vec();
        let recent: Vec<Message> = non_system[split_at..].to_vec();
        let pruned_count = pruned.len();

        let summary = if !pruned.is_empty() {
            let topics: Vec<String> = pruned
                .iter()
                .filter(|m| m.role == Role::User)
                .take(5)
                .map(|m| {
                    let preview: String = m.content.chars().take(80).collect();
                    format!("- {}", preview)
                })
                .collect();

            let topic_text = if topics.is_empty() {
                "  (tool calls and assistant responses)".to_string()
            } else {
                topics.join("\n")
            };

            let summary_text = format!(
                "{} {} message(s) summarized to save context.\n\nTopics discussed:\n{}",
                self.config.summary_prefix, pruned_count, topic_text
            );

            Some(Message::assistant(summary_text))
        } else {
            None
        };

        let mut result = system_messages;
        if let Some(s) = summary {
            result.push(s);
        }
        result.extend(recent);

        CompactionResult {
            messages: result,
            was_compacted: true,
            pruned_count,
            summary_inserted: pruned_count > 0,
        }
    }

    pub fn compact_to_fit(&self, messages: Vec<Message>) -> CompactionResult {
        let mut result = self.compact(messages);
        let mut iterations = 0;

        while result.was_compacted && self.needs_compaction(&result.messages) && iterations < 10 {
            let prev_count = result.messages.len();
            let next = self.compact(result.messages);
            result.pruned_count += next.pruned_count;
            result.messages = next.messages;
            result.was_compacted = next.was_compacted;
            if result.messages.len() >= prev_count {
                break;
            }
            iterations += 1;
        }

        result
    }

    pub fn estimate_tokens(&self, text: &str) -> usize {
        (text.len() + 3) / 4
    }

    pub fn validate_reserved(reserved: u32) -> Result<(), CompactionError> {
        if reserved == 0 {
            return Err(CompactionError::InvalidReserved(reserved));
        }

        if reserved < 5000 {
            warn!(
                reserved,
                "compaction reserved tokens below recommended minimum (5000)"
            );
        }

        Ok(())
    }

    pub fn prune_old_tool_outputs(messages: &mut Vec<Message>, keep_recent: usize) {
        let keep_recent = if keep_recent == 0 {
            DEFAULT_KEEP_RECENT_TOOL_OUTPUTS
        } else {
            keep_recent
        };

        let tool_indices: Vec<usize> = messages
            .iter()
            .enumerate()
            .filter_map(|(idx, message)| is_tool_output_message(message).then_some(idx))
            .collect();

        if tool_indices.len() <= keep_recent {
            return;
        }

        let prune_until = tool_indices.len() - keep_recent;
        for idx in tool_indices.into_iter().take(prune_until) {
            if let Some(message) = messages.get_mut(idx) {
                message.content = PRUNED_TOOL_OUTPUT_PLACEHOLDER.to_string();
            }
        }
    }

    pub fn generate_summary_prompt(messages: &[Message]) -> String {
        let mut lines = vec![
            "Summarize the following conversation for context compression.".to_string(),
            "Keep decisions, constraints, unresolved issues, and next steps.".to_string(),
            "Conversation: ".to_string(),
        ];

        for message in messages {
            let role = match message.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
            };
            lines.push(format!("- {role}: {}", message.content));
        }

        lines.push("Return concise bullet points only.".to_string());
        lines.join("\n")
    }

    pub fn auto_compact_if_needed(
        session: &mut crate::session::Session,
        config: &crate::config::CompactionConfig,
        model_max_context: usize,
    ) -> Result<bool, CompactionError> {
        if model_max_context == 0 {
            return Err(CompactionError::InvalidModelContext(model_max_context));
        }

        if config.auto == Some(false) {
            return Ok(false);
        }

        let reserved = config.reserved.unwrap_or(10_000);
        Self::validate_reserved(reserved)?;
        if reserved as usize >= model_max_context {
            return Err(CompactionError::ReservedExceedsModelContext {
                reserved,
                model_max_context,
            });
        }

        let session_tokens: usize = session
            .messages
            .iter()
            .map(|m| (m.content.len() + 3) / 4)
            .sum();

        let trigger_threshold = model_max_context - reserved as usize;
        if session_tokens <= trigger_threshold {
            return Ok(false);
        }

        if config.prune.unwrap_or(true) {
            Self::prune_old_tool_outputs(&mut session.messages, DEFAULT_KEEP_RECENT_TOOL_OUTPUTS);
        }

        let compactor = Compactor::new(CompactionConfig {
            max_tokens: trigger_threshold,
            ..Default::default()
        });

        let messages = std::mem::take(&mut session.messages);
        let result = compactor.compact_to_fit(messages);
        session.messages = result.messages;
        session.updated_at = chrono::Utc::now();

        Ok(result.was_compacted)
    }
}

fn is_tool_output_message(message: &Message) -> bool {
    message.role == Role::Assistant
        && (message.content.contains("Tool")
            || message.content.contains("tool")
            || message.content.contains("stdout")
            || message.content.contains("stderr")
            || message.content.contains("```json")
            || message.content.contains("```"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CompactionConfig as RuntimeCompactionConfig;
    use crate::session::Session;

    #[test]
    fn test_compaction_config_default() {
        let config = CompactionConfig::default();
        assert!(config.max_tokens > 0);
        assert!(config.preserve_system_messages);
        assert!(config.preserve_recent_messages > 0);
    }

    #[test]
    fn test_token_budget_model_mapping() {
        assert_eq!(TokenBudget::from_model("gpt-4o").model_max_tokens, 128_000);
        assert_eq!(TokenBudget::from_model("gpt-4").model_max_tokens, 8_192);
        assert_eq!(
            TokenBudget::from_model("gpt-3.5-turbo").model_max_tokens,
            16_385
        );
        assert_eq!(
            TokenBudget::from_model("claude-3-5-sonnet").model_max_tokens,
            200_000
        );
        assert_eq!(TokenBudget::from_model("unknown").model_max_tokens, 128_000);
    }

    #[test]
    fn test_compactor_estimate_tokens() {
        let compactor = Compactor::new(CompactionConfig::default());
        let tokens = compactor.estimate_tokens("hello world");
        assert!(tokens > 0);
    }

    #[test]
    fn test_compactor_no_op_when_under_limit() {
        let compactor = Compactor::new(CompactionConfig {
            max_tokens: 100_000,
            ..Default::default()
        });
        let messages = vec![Message::user("hello")];
        let result = compactor.compact(messages.clone());
        assert!(!result.was_compacted);
        assert_eq!(result.messages.len(), messages.len());
    }

    #[test]
    fn test_compactor_keeps_system_messages() {
        let compactor = Compactor::new(CompactionConfig {
            max_tokens: 1,
            preserve_system_messages: true,
            preserve_recent_messages: 0,
            ..Default::default()
        });
        let messages = vec![Message::system("system"), Message::user("user")];
        let result = compactor.compact(messages);
        assert!(result.was_compacted);
        let roles: Vec<&Role> = result.messages.iter().map(|m| &m.role).collect();
        assert!(roles.contains(&&Role::System));
    }

    #[test]
    fn test_compactor_keeps_recent_messages() {
        let compactor = Compactor::new(CompactionConfig {
            max_tokens: 1,
            preserve_system_messages: false,
            preserve_recent_messages: 1,
            ..Default::default()
        });
        let messages = vec![Message::user("old message"), Message::user("new message")];
        let result = compactor.compact(messages);
        assert!(result.was_compacted);
        let last = result.messages.last().unwrap();
        assert_eq!(last.content, "new message");
    }

    #[test]
    fn test_compactor_inserts_summary() {
        let compactor = Compactor::new(CompactionConfig {
            max_tokens: 1,
            preserve_system_messages: false,
            preserve_recent_messages: 1,
            summary_prefix: "[Compacted]".to_string(),
            ..Default::default()
        });
        let messages = vec![
            Message::user("old question about foo"),
            Message::assistant("old answer"),
            Message::user("new question"),
        ];
        let result = compactor.compact(messages);
        assert!(result.summary_inserted);
        assert!(result
            .messages
            .iter()
            .any(|m| m.content.contains("[Compacted]")));
    }

    #[test]
    fn test_needs_compaction() {
        let compactor = Compactor::new(CompactionConfig {
            max_tokens: 1,
            ..Default::default()
        });
        let messages = vec![Message::user("hello world")];
        assert!(compactor.needs_compaction(&messages));
    }

    #[test]
    fn test_validate_reserved() {
        assert!(Compactor::validate_reserved(1).is_ok());
        assert_eq!(
            Compactor::validate_reserved(0).unwrap_err(),
            CompactionError::InvalidReserved(0)
        );
    }

    #[test]
    fn test_prune_old_tool_outputs_keeps_recent() {
        let mut messages = vec![
            Message::assistant("Tool call 1\nstdout:\nA"),
            Message::assistant("Tool call 2\nstdout:\nB"),
            Message::assistant("Tool call 3\nstdout:\nC"),
            Message::assistant("Tool call 4\nstdout:\nD"),
        ];

        Compactor::prune_old_tool_outputs(&mut messages, 2);
        assert_eq!(messages[0].content, PRUNED_TOOL_OUTPUT_PLACEHOLDER);
        assert_eq!(messages[1].content, PRUNED_TOOL_OUTPUT_PLACEHOLDER);
        assert!(messages[2].content.contains("Tool call 3"));
        assert!(messages[3].content.contains("Tool call 4"));
    }

    #[test]
    fn test_generate_summary_prompt_contains_roles_and_content() {
        let messages = vec![
            Message::system("system constraints"),
            Message::user("do the thing"),
            Message::assistant("done"),
        ];

        let prompt = Compactor::generate_summary_prompt(&messages);
        assert!(prompt.contains("- system: system constraints"));
        assert!(prompt.contains("- user: do the thing"));
        assert!(prompt.contains("- assistant: done"));
    }

    #[test]
    fn test_auto_compact_if_needed_triggers_near_limit() {
        let mut session = Session::new();
        for i in 0..15 {
            session.add_message(Message::assistant(format!(
                "Tool call {}\nstdout:\n{}",
                i,
                "x".repeat(40)
            )));
            session.add_message(Message::user(format!("request {}", i)));
        }

        let before: usize = session
            .messages
            .iter()
            .map(|m| (m.content.len() + 3) / 4)
            .sum();

        let changed = Compactor::auto_compact_if_needed(
            &mut session,
            &RuntimeCompactionConfig {
                auto: Some(true),
                prune: Some(true),
                reserved: Some(50),
            },
            200,
        )
        .unwrap();

        assert!(changed);
        let estimated: usize = session
            .messages
            .iter()
            .map(|m| (m.content.len() + 3) / 4)
            .sum();
        assert!(estimated < before);
    }
}
