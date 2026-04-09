use async_trait::async_trait;
use opencode_core::{Message, OpenCodeError};
use serde::{Deserialize, Serialize};

pub type StreamingCallback = Box<dyn FnMut(String) + Send>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
}

impl Model {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub model: String,
    pub api_key: String,
    pub temperature: f32,
}

impl ProviderConfig {
    pub fn sanitize_for_logging(&self) -> Self {
        let mut sanitized = self.clone();
        if !sanitized.api_key.is_empty() {
            sanitized.api_key = "***REDACTED***".to_string();
        }
        sanitized
    }
}

impl std::fmt::Debug for ProviderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sanitized = self.sanitize_for_logging();
        f.debug_struct("ProviderConfig")
            .field("model", &sanitized.model)
            .field("api_key", &sanitized.api_key)
            .field("temperature", &sanitized.temperature)
            .finish()
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4o".to_string(),
            api_key: String::new(),
            temperature: 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl From<&Message> for ChatMessage {
    fn from(msg: &Message) -> Self {
        Self {
            role: match msg.role {
                opencode_core::Role::System => "system".to_string(),
                opencode_core::Role::User => "user".to_string(),
                opencode_core::Role::Assistant => "assistant".to_string(),
            },
            content: msg.content.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
}

#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub content: String,
    pub done: bool,
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError>;
    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        let content = self.complete(prompt, None).await?;
        callback(content);
        Ok(())
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, OpenCodeError> {
        let prompt = messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");
        let content = self.complete(&prompt, None).await?;
        Ok(ChatResponse {
            content,
            model: String::new(),
        })
    }

    fn get_models(&self) -> Vec<Model> {
        vec![]
    }

    fn provider_name(&self) -> &str {
        "unknown"
    }
}

pub trait SimpleProvider: Send + Sync {
    fn get_models(&self) -> Vec<Model>;
    fn provider_name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_new() {
        let model = Model::new("gpt-4", "GPT-4");
        assert_eq!(model.id, "gpt-4");
        assert_eq!(model.name, "GPT-4");
    }

    #[test]
    fn test_chat_message_from_message() {
        let msg = opencode_core::Message {
            role: opencode_core::Role::User,
            content: "Hello".to_string(),
            timestamp: chrono::Utc::now(),
            parts: None,
        };
        let chat_msg = ChatMessage::from(&msg);
        assert_eq!(chat_msg.role, "user");
        assert_eq!(chat_msg.content, "Hello");
    }

    #[test]
    fn test_provider_config_default() {
        let config = ProviderConfig::default();
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.temperature, 0.7);
        assert!(config.api_key.is_empty());
    }

    #[test]
    fn test_simple_provider_trait() {
        struct TestProvider;
        impl SimpleProvider for TestProvider {
            fn get_models(&self) -> Vec<Model> {
                vec![Model::new("test-model", "Test Model")]
            }
            fn provider_name(&self) -> &str {
                "test"
            }
        }

        let provider = TestProvider;
        assert_eq!(provider.provider_name(), "test");
        assert_eq!(provider.get_models().len(), 1);
    }
}
