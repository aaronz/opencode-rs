use crate::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    pub max_messages: usize,
    pub max_tokens: usize,
    pub keep_system_messages: bool,
    pub keep_recent_messages: usize,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            max_messages: 100,
            max_tokens: 4096,
            keep_system_messages: true,
            keep_recent_messages: 10,
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

    pub fn compact(&self, session: &mut Session) {
        if session.messages.len() <= self.config.max_messages {
            return;
        }

        let mut compacted = Vec::new();
        let mut token_count = 0;

        if self.config.keep_system_messages {
            for msg in &session.messages {
                if msg.role == crate::message::Role::System {
                    compacted.push(msg.clone());
                    token_count += self.estimate_tokens(&msg.content);
                }
            }
        }

        let recent_start = session
            .messages
            .len()
            .saturating_sub(self.config.keep_recent_messages);
        for msg in &session.messages[recent_start..] {
            if token_count + self.estimate_tokens(&msg.content) <= self.config.max_tokens {
                compacted.push(msg.clone());
                token_count += self.estimate_tokens(&msg.content);
            }
        }

        session.messages = compacted;
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        text.len() / 4
    }
}
