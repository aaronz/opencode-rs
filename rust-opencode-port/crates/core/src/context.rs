use crate::compaction::{CompactionConfig, Compactor, TokenBudget};
use crate::message::{Message, Role};
use crate::tool::ToolRegistry;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;

const PRESERVE_LAST_MESSAGES: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBudget {
    pub total_tokens: usize,
    pub max_tokens: usize,
    pub remaining_tokens: usize,
}

impl ContextBudget {
    pub fn from_usage(max_tokens: usize, total_tokens: usize) -> Self {
        Self {
            total_tokens,
            max_tokens,
            remaining_tokens: max_tokens.saturating_sub(total_tokens),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub file_context: Vec<String>,
    pub tool_context: Vec<String>,
    pub session_context: Vec<String>,
    pub prompt_messages: Vec<Message>,
    pub budget: ContextBudget,
}

pub struct ContextBuilder {
    token_budget: TokenBudget,
    file_context: Vec<String>,
    tool_context: Vec<String>,
    session_context: Vec<String>,
    prompt_messages: Vec<Message>,
}

impl ContextBuilder {
    pub fn new(token_budget: TokenBudget) -> Self {
        Self {
            token_budget,
            file_context: Vec::new(),
            tool_context: Vec::new(),
            session_context: Vec::new(),
            prompt_messages: Vec::new(),
        }
    }

    pub fn collect_file_context(
        mut self,
        opened_files: &[PathBuf],
        referenced_messages: &[Message],
    ) -> Self {
        let mut collected = BTreeSet::new();

        for file in opened_files {
            collected.insert(file.display().to_string());
        }

        let file_pattern = Regex::new(r"([\w./-]+\.[\w]+)").expect("valid file path regex");
        for message in referenced_messages {
            for captures in file_pattern.captures_iter(&message.content) {
                if let Some(m) = captures.get(1) {
                    collected.insert(m.as_str().to_string());
                }
            }
        }

        self.file_context = collected.into_iter().collect();
        self
    }

    pub fn collect_tool_context(mut self, registry: &ToolRegistry) -> Self {
        self.tool_context = registry
            .get_all()
            .iter()
            .map(|tool| format!("{}: {}", tool.name, tool.description))
            .collect();
        self
    }

    pub fn collect_session_context(mut self, messages: &[Message]) -> Self {
        self.prompt_messages = messages.to_vec();
        self.session_context = messages
            .iter()
            .rev()
            .take(10)
            .rev()
            .map(|m| {
                let preview: String = m.content.chars().take(120).collect();
                format!("{:?}: {}", m.role, preview)
            })
            .collect();
        self
    }

    pub fn build(mut self) -> Context {
        let max_tokens = self.token_budget.main_context_tokens();
        let mut budget = ContextBudget::from_usage(max_tokens, self.total_tokens());

        trim_to_budget(&mut self.prompt_messages, &budget);

        let compactor = Compactor::new(CompactionConfig {
            max_tokens,
            preserve_system_messages: true,
            preserve_recent_messages: PRESERVE_LAST_MESSAGES,
            ..Default::default()
        });

        if compactor.needs_compaction(&self.prompt_messages) {
            self.prompt_messages = compactor.compact_to_fit(self.prompt_messages).messages;
        }

        budget = ContextBudget::from_usage(max_tokens, self.total_tokens());

        Context {
            file_context: self.file_context,
            tool_context: self.tool_context,
            session_context: self.session_context,
            prompt_messages: self.prompt_messages,
            budget,
        }
    }

    fn total_tokens(&self) -> usize {
        let file_tokens: usize = self.file_context.iter().map(|s| estimate_tokens(s)).sum();
        let tool_tokens: usize = self.tool_context.iter().map(|s| estimate_tokens(s)).sum();
        let session_tokens: usize = self
            .session_context
            .iter()
            .map(|s| estimate_tokens(s))
            .sum();
        let prompt_tokens: usize = self
            .prompt_messages
            .iter()
            .map(|m| estimate_tokens(&m.content))
            .sum();
        file_tokens + tool_tokens + session_tokens + prompt_tokens
    }
}

pub fn estimate_tokens(text: &str) -> usize {
    (text.chars().count() + 3) / 4
}

pub fn trim_to_budget(messages: &mut Vec<Message>, budget: &ContextBudget) {
    let total_tokens = |msgs: &[Message]| {
        msgs.iter()
            .map(|m| estimate_tokens(&m.content))
            .sum::<usize>()
    };

    if total_tokens(messages) <= budget.max_tokens {
        return;
    }

    while total_tokens(messages) > budget.max_tokens {
        let len = messages.len();
        if len <= PRESERVE_LAST_MESSAGES {
            break;
        }

        let preserve_from = len.saturating_sub(PRESERVE_LAST_MESSAGES);

        let removable_idx = messages.iter().enumerate().find_map(|(idx, msg)| {
            let is_protected_system = msg.role == Role::System;
            let is_protected_recent = idx >= preserve_from;
            if is_protected_system || is_protected_recent {
                None
            } else {
                Some(idx)
            }
        });

        match removable_idx {
            Some(idx) => {
                messages.remove(idx);
            }
            None => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::build_default_registry;

    #[test]
    fn test_estimate_tokens_char_approximation() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcde"), 2);
    }

    #[test]
    fn test_trim_to_budget_keeps_system_and_last_three() {
        let mut messages = vec![
            Message::system("system prompt"),
            Message::user("first"),
            Message::assistant("second"),
            Message::user("third"),
            Message::assistant("fourth"),
            Message::user("fifth"),
        ];

        let budget = ContextBudget::from_usage(3, 999);
        trim_to_budget(&mut messages, &budget);

        assert_eq!(messages[0].role, Role::System);
        let tail: Vec<String> = messages
            .iter()
            .rev()
            .take(3)
            .map(|m| m.content.clone())
            .collect();
        assert!(tail.contains(&"third".to_string()));
        assert!(tail.contains(&"fourth".to_string()));
        assert!(tail.contains(&"fifth".to_string()));
    }

    #[test]
    fn test_context_builder_builds_sections() {
        let registry = build_default_registry();
        let messages = vec![
            Message::user("Please inspect src/main.rs"),
            Message::assistant("Sure"),
        ];

        let context = ContextBuilder::new(TokenBudget::default())
            .collect_file_context(&[PathBuf::from("Cargo.toml")], &messages)
            .collect_tool_context(&registry)
            .collect_session_context(&messages)
            .build();

        assert!(context
            .file_context
            .iter()
            .any(|p| p.contains("Cargo.toml")));
        assert!(!context.tool_context.is_empty());
        assert_eq!(context.prompt_messages.len(), 2);
        assert!(context.budget.max_tokens > 0);
    }
}
