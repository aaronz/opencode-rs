use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    pub content: String,
    pub variables: Vec<String>,
}

pub struct PromptManager {
    templates: HashMap<String, PromptTemplate>,
}

impl PromptManager {
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        templates.insert(
            "system".to_string(),
            PromptTemplate {
                name: "system".to_string(),
                content: "You are OpenCode, an AI coding assistant. Be helpful and concise."
                    .to_string(),
                variables: Vec::new(),
            },
        );

        templates.insert(
            "user".to_string(),
            PromptTemplate {
                name: "user".to_string(),
                content: "{{message}}".to_string(),
                variables: vec!["message".to_string()],
            },
        );

        templates.insert(
            "assistant".to_string(),
            PromptTemplate {
                name: "assistant".to_string(),
                content: "{{response}}".to_string(),
                variables: vec!["response".to_string()],
            },
        );

        Self { templates }
    }

    pub fn get(&self, name: &str) -> Option<&PromptTemplate> {
        self.templates.get(name)
    }

    pub fn add_template(&mut self, template: PromptTemplate) {
        self.templates.insert(template.name.clone(), template);
    }

    pub fn render(&self, name: &str, variables: &HashMap<String, String>) -> Option<String> {
        let template = self.templates.get(name)?;
        let mut result = template.content.clone();

        for (key, value) in variables {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }

        Some(result)
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new()
    }
}
