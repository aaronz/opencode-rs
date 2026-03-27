use crate::message::{Message, Role};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    pub max_tokens: usize,
    pub preserve_system_messages: bool,
    pub preserve_recent_messages: usize,
    pub summary_prefix: String,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            max_tokens: 100_000,
            preserve_system_messages: true,
            preserve_recent_messages: 10,
            summary_prefix: "[Context compacted]".to_string(),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compaction_config_default() {
        let config = CompactionConfig::default();
        assert!(config.max_tokens > 0);
        assert!(config.preserve_system_messages);
        assert!(config.preserve_recent_messages > 0);
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
}
