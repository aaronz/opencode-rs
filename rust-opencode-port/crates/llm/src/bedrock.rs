use crate::provider::{Provider, StreamingCallback, Model, ProviderConfig};
use opencode_core::OpenCodeError;

pub struct BedrockProvider {
    _config: ProviderConfig,
}

impl BedrockProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { _config: config }
    }
}

#[async_trait::async_trait]
impl Provider for BedrockProvider {
    async fn complete(&self, _prompt: &str, _context: Option<&str>) -> Result<String, OpenCodeError> {
        Err(OpenCodeError::Llm("Bedrock provider not fully implemented".to_string()))
    }

    async fn complete_streaming(&self, prompt: &str, mut callback: StreamingCallback) -> Result<(), OpenCodeError> {
        let content = self.complete(prompt, None).await?;
        callback(content);
        Ok(())
    }

    fn get_models(&self) -> Vec<Model> {
        vec![
            Model::new("anthropic.claude-3-opus", "Claude 3 Opus"),
            Model::new("anthropic.claude-3-sonnet", "Claude 3 Sonnet"),
            Model::new("anthropic.claude-3-haiku", "Claude 3 Haiku"),
            Model::new("meta.llama2-70b", "Llama 2 70B"),
            Model::new("meta.llama3-70b", "Llama 3 70B"),
            Model::new("amazon.titan-text", "Titan Text"),
        ]
    }

    fn provider_name(&self) -> &str {
        "bedrock"
    }
}
