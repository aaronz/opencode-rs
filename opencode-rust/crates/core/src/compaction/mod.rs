pub mod types;

pub use types::{
    CompactionConfig, CompactionLevel, CompactionResult, CompactionStatus, CompactionTrigger,
    TokenBudget, COMPACTION_FORCE_THRESHOLD, COMPACTION_START_THRESHOLD, COMPACTION_WARN_THRESHOLD,
};

pub(crate) use types::{
    DEFAULT_MAX_TOKENS, DEFAULT_PRESERVE_RECENT_MESSAGES, DEFAULT_RESERVED_TOKENS,
    MAX_COMPACTION_ITERATIONS, MIN_RESERVED_TOKENS_WARNING, SUMMARY_PREVIEW_MAX_CHARS,
    SUMMARY_TOPICS_EXTRACTION_LIMIT,
};

use crate::config::CompactionConfig as RuntimeCompactionConfig;
use crate::message::{Message, Role};
use types::CompactionError;

const DEFAULT_KEEP_RECENT_TOOL_OUTPUTS: usize = 3;
const PRUNED_TOOL_OUTPUT_PLACEHOLDER: &str = "[content pruned to save tokens]";

pub struct Compactor {
    config: CompactionConfig,
}

impl Compactor {
    pub fn new(config: CompactionConfig) -> Self {
        Self { config }
    }

    pub(crate) fn needs_compaction(&self, messages: &[Message]) -> bool {
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
                .take(SUMMARY_TOPICS_EXTRACTION_LIMIT)
                .map(|m| {
                    let preview: String =
                        m.content.chars().take(SUMMARY_PREVIEW_MAX_CHARS).collect();
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

        while result.was_compacted
            && self.needs_compaction(&result.messages)
            && iterations < MAX_COMPACTION_ITERATIONS
        {
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

    pub(crate) fn estimate_tokens(&self, text: &str) -> usize {
        text.len().div_ceil(4)
    }

    pub(crate) fn validate_reserved(reserved: u32) -> Result<(), CompactionError> {
        if reserved == 0 {
            return Err(CompactionError::InvalidReserved(reserved));
        }

        if reserved < MIN_RESERVED_TOKENS_WARNING {
            tracing::warn!(
                reserved,
                "compaction reserved tokens below recommended minimum ({})",
                MIN_RESERVED_TOKENS_WARNING
            );
        }

        Ok(())
    }

    pub(crate) fn prune_old_tool_outputs(messages: &mut [Message], keep_recent: usize) {
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

    #[allow(dead_code)]
    pub(crate) fn generate_summary_prompt(messages: &[Message]) -> String {
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

    pub(crate) fn auto_compact_if_needed(
        session: &mut crate::session::Session,
        config: &RuntimeCompactionConfig,
        model_max_context: usize,
    ) -> Result<bool, CompactionError> {
        if model_max_context == 0 {
            return Err(CompactionError::InvalidModelContext(model_max_context));
        }

        if config.auto == Some(false) {
            return Ok(false);
        }

        let reserved = config.reserved.unwrap_or(DEFAULT_RESERVED_TOKENS);
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
            .map(|m| m.content.len().div_ceil(4))
            .sum();

        let trigger_threshold = model_max_context - reserved as usize;
        if session_tokens <= trigger_threshold {
            return Ok(false);
        }

        if config.prune.unwrap_or(true) {
            Self::prune_old_tool_outputs(&mut session.messages, DEFAULT_KEEP_RECENT_TOOL_OUTPUTS);
        }

        let token_budget = TokenBudget::from_config(config, model_max_context);
        let preserve_recent = config
            .preserve_recent_messages
            .unwrap_or(DEFAULT_PRESERVE_RECENT_MESSAGES);
        let preserve_system = config.preserve_system_messages.unwrap_or(true);
        let summary_prefix = config
            .summary_prefix
            .clone()
            .unwrap_or_else(|| "[Context compacted]".to_string());

        let compactor = Compactor::new(CompactionConfig {
            max_tokens: trigger_threshold,
            preserve_system_messages: preserve_system,
            preserve_recent_messages: preserve_recent,
            summary_prefix,
            token_budget,
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
    fn test_token_budget_with_custom_thresholds() {
        let budget = TokenBudget::default().with_thresholds(0.80, 0.90, 0.98);

        assert_eq!(budget.warning_threshold, 0.80);
        assert_eq!(budget.compact_threshold, 0.90);
        assert_eq!(budget.continuation_threshold, 0.98);

        assert_eq!(budget.usage_level(100_000), CompactionLevel::Normal);
        assert_eq!(budget.usage_level(115_000), CompactionLevel::Warning);
        assert_eq!(budget.usage_level(120_000), CompactionLevel::AutoCompact);
        assert_eq!(
            budget.usage_level(126_000),
            CompactionLevel::ForceContinuation
        );
    }

    #[test]
    fn test_token_budget_from_config() {
        let config = RuntimeCompactionConfig {
            warning_threshold: Some(0.75),
            compact_threshold: Some(0.85),
            continuation_threshold: Some(0.95),
            ..Default::default()
        };

        let budget = TokenBudget::from_config(&config, 100_000);

        assert_eq!(budget.total, 100_000);
        assert_eq!(budget.warning_threshold, 0.75);
        assert_eq!(budget.compact_threshold, 0.85);
        assert_eq!(budget.continuation_threshold, 0.95);
    }

    #[test]
    fn test_token_budget_from_config_with_defaults() {
        let config = RuntimeCompactionConfig::default();
        let budget = TokenBudget::from_config(&config, 0);

        assert_eq!(budget.warning_threshold, COMPACTION_WARN_THRESHOLD as f64);
        assert_eq!(budget.compact_threshold, COMPACTION_START_THRESHOLD as f64);
        assert_eq!(
            budget.continuation_threshold,
            COMPACTION_FORCE_THRESHOLD as f64
        );
    }

    #[test]
    fn test_token_budget_from_config_clamps_values() {
        let config = RuntimeCompactionConfig {
            warning_threshold: Some(1.5),
            compact_threshold: Some(-0.5),
            continuation_threshold: Some(2.0),
            ..Default::default()
        };

        let budget = TokenBudget::from_config(&config, 100_000);

        assert_eq!(budget.warning_threshold, 1.0);
        assert_eq!(budget.compact_threshold, 0.0);
        assert_eq!(budget.continuation_threshold, 1.0);
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
            .map(|m| m.content.len().div_ceil(4))
            .sum();

        let changed = Compactor::auto_compact_if_needed(
            &mut session,
            &RuntimeCompactionConfig {
                auto: Some(true),
                prune: Some(true),
                reserved: Some(50),
                ..Default::default()
            },
            200,
        )
        .unwrap();

        assert!(changed);
        let estimated: usize = session
            .messages
            .iter()
            .map(|m| m.content.len().div_ceil(4))
            .sum();
        assert!(estimated < before);
    }

    #[test]
    fn test_compaction_performance_small_messages() {
        let compactor = Compactor::new(CompactionConfig {
            max_tokens: 100,
            ..Default::default()
        });
        let messages: Vec<Message> = (0..100)
            .map(|i| Message::user(format!("Message {} with some content", i)))
            .collect();

        let start = std::time::Instant::now();
        let result = compactor.compact(messages);
        let elapsed = start.elapsed();

        assert!(result.was_compacted);
        assert!(
            elapsed.as_millis() < 100,
            "compaction took {}ms, expected < 100ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_compaction_performance_large_messages() {
        let compactor = Compactor::new(CompactionConfig {
            max_tokens: 1000,
            ..Default::default()
        });
        let messages: Vec<Message> = (0..500)
            .map(|i| {
                Message::user(format!(
                    "Message {} with content that is somewhat longer to simulate real messages {}",
                    i,
                    "x".repeat(100)
                ))
            })
            .collect();

        let start = std::time::Instant::now();
        let result = compactor.compact(messages);
        let elapsed = start.elapsed();

        assert!(result.was_compacted);
        assert!(
            elapsed.as_millis() < 500,
            "compaction took {}ms, expected < 500ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_compaction_iteration_performance() {
        let compactor = Compactor::new(CompactionConfig {
            max_tokens: 50,
            preserve_system_messages: false,
            preserve_recent_messages: 5,
            ..Default::default()
        });
        let messages: Vec<Message> = (0..100)
            .map(|i| {
                Message::assistant(format!(
                    "Tool result {} with output: {}",
                    i,
                    "y".repeat(200)
                ))
            })
            .collect();

        let start = std::time::Instant::now();
        let result = compactor.compact_to_fit(messages);
        let elapsed = start.elapsed();

        assert!(result.was_compacted);
        assert!(
            elapsed.as_millis() < 200,
            "compact_to_fit took {}ms, expected < 200ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_compaction_no_compaction_under_limit() {
        let compactor = Compactor::new(CompactionConfig {
            max_tokens: 100_000,
            ..Default::default()
        });
        let messages = vec![
            Message::system("You are a helpful assistant"),
            Message::user("Hello"),
            Message::assistant("Hi there!"),
        ];

        let start = std::time::Instant::now();
        let result = compactor.compact(messages);
        let elapsed = start.elapsed();

        assert!(!result.was_compacted);
        assert!(
            elapsed.as_millis() < 10,
            "compaction should be instant for small messages, took {}ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_token_budget_usage_level_performance() {
        let budget = TokenBudget::from_model("gpt-4o");
        let start = std::time::Instant::now();
        for used in (0..120_000).step_by(1000) {
            let _ = budget.usage_level(used);
        }
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_millis() < 50,
            "usage_level should be fast, took {}ms for 120 calls",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_prune_tool_outputs_performance() {
        let mut messages: Vec<Message> = (0..200)
            .map(|i| Message::assistant(format!("Tool call {}\nstdout:\n{}", i, "x".repeat(100))))
            .collect();

        let start = std::time::Instant::now();
        Compactor::prune_old_tool_outputs(&mut messages, 10);
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_millis() < 100,
            "prune took {}ms, expected < 100ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_compaction_memory_efficiency() {
        let compactor = Compactor::new(CompactionConfig {
            max_tokens: 100,
            preserve_system_messages: true,
            preserve_recent_messages: 5,
            ..Default::default()
        });
        let messages: Vec<Message> = (0..1000)
            .map(|i| Message::user(format!("Message {} with content {}", i, "z".repeat(50))))
            .collect();

        let memory_before = estimate_memory(&messages);
        let result = compactor.compact(messages);
        let memory_after = estimate_memory(&result.messages);

        assert!(
            memory_after < memory_before,
            "compacted messages should use less memory"
        );
        assert!(result.pruned_count > 0);
    }

    fn estimate_memory(messages: &[Message]) -> usize {
        messages
            .iter()
            .map(|m| std::mem::size_of::<Message>() + m.content.capacity())
            .sum()
    }
}
