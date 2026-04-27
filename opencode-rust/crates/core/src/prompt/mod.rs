use std::collections::HashMap;

mod types;
pub use types::{PromptManager, PromptTemplate};

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_prompt_manager_new() {
        let pm = PromptManager::new();
        assert!(pm.get("system").is_some());
        assert!(pm.get("user").is_some());
    }

    #[test]
    fn test_prompt_manager_get() {
        let pm = PromptManager::new();
        let template = pm.get("system");
        assert!(template.is_some());
        assert!(template.unwrap().content.contains("OpenCode"));
    }

    #[test]
    fn test_prompt_manager_add_template() {
        let mut pm = PromptManager::new();
        pm.add_template(PromptTemplate {
            name: "custom".to_string(),
            content: "Hello {{name}}".to_string(),
            variables: vec!["name".to_string()],
        });

        assert!(pm.get("custom").is_some());
    }

    #[test]
    fn test_prompt_manager_render() {
        let pm = PromptManager::new();
        let mut vars = HashMap::new();
        vars.insert("message".to_string(), "Hello World".to_string());

        let result = pm.render("user", &vars);
        assert!(result.is_some());
        assert!(result.unwrap().contains("Hello World"));
    }

    #[test]
    fn test_prompt_manager_render_missing_template() {
        let pm = PromptManager::new();
        let vars = HashMap::new();

        let result = pm.render("nonexistent", &vars);
        assert!(result.is_none());
    }
}
