use serde::{Deserialize, Serialize};
use crate::message::{Message, Role};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    pub max_tokens: usize,
    pub preserve_system_messages: bool,
    pub preserve_recent_messages: usize,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            preserve_system_messages: true,
            preserve_recent_messages: 5,
        }
    }
}

pub struct Compactor {
    config: CompactionConfig,
}

impl Compactor {
    pub fn new(config: CompactionConfig) -> Self {
        Self { config }
    }

    pub fn compact(&self, messages: Vec<Message>) -> Vec<Message> {
        let mut total_tokens = 0;
        for msg in &messages {
            total_tokens += self.estimate_tokens(&msg.content);
        }

        if total_tokens <= self.config.max_tokens {
            return messages;
        }

        let mut preserved = Vec::new();
        let mut recent = Vec::new();
        let mut candidates = Vec::new();

        for (i, msg) in messages.into_iter().enumerate() {
            if self.config.preserve_system_messages && msg.role == Role::System {
                preserved.push(msg);
            } else {
                candidates.push(msg);
            }
        }

        let recent_count = std::cmp::min(candidates.len(), self.config.preserve_recent_messages);
        let split_at = candidates.len() - recent_count;
        recent = candidates.split_off(split_at);

        let mut result = preserved;
        result.extend(recent);
        result
    }

    pub fn estimate_tokens(&self, text: &str) -> usize {
        text.split_whitespace().count() * 4 / 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compaction_config_default() {
        let config = CompactionConfig::default();
        assert!(config.max_tokens > 0);
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
            max_tokens: 1000,
            ..Default::default()
        });
        let messages = vec![Message::user("hello")];
        let result = compactor.compact(messages.clone());
        assert_eq!(result.len(), messages.len());
    }

    #[test]
    fn test_compactor_keeps_system_messages() {
        let compactor = Compactor::new(CompactionConfig {
            max_tokens: 1,
            preserve_system_messages: true,
            preserve_recent_messages: 0,
        });
        let messages = vec![Message::system("system"), Message::user("user")];
        let result = compactor.compact(messages);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].role, Role::System);
    }

    #[test]
    fn test_compactor_keeps_recent_messages() {
        let compactor = Compactor::new(CompactionConfig {
            max_tokens: 1,
            preserve_system_messages: false,
            preserve_recent_messages: 1,
        });
        let messages = vec![Message::user("old"), Message::user("new")];
        let result = compactor.compact(messages);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "new");
    }
}
