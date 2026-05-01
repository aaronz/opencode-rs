use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::stream::{self};
use futures_util::Stream;
use opencode_llm::{ChatMessage, DynProvider, ProviderManager, ProviderSpec};
use tokio::sync::RwLock;

use crate::provider_gateway::{
    ModelCapabilities, ModelInfo, ProviderError, ProviderErrorKind, ProviderRef,
    ProviderRequest, ProviderStreamEvent, ProviderStatus,
};

pub struct LlmProviderGateway {
    provider_manager: Arc<RwLock<ProviderManager>>,
}

impl LlmProviderGateway {
    pub fn new() -> Self {
        Self {
            provider_manager: Arc::new(RwLock::new(ProviderManager::new())),
        }
    }

    async fn get_provider(
        &self,
        provider_ref: &str,
    ) -> std::result::Result<DynProvider, ProviderError> {
        let manager = self.provider_manager.read().await;
        let spec = match provider_ref {
            "openai" => ProviderSpec::OpenAI {
                api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
                model: "gpt-4".to_string(),
                base_url: None,
            },
            "anthropic" => ProviderSpec::Anthropic {
                api_key: std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
                model: "claude-3-5-sonnet-20241022".to_string(),
                base_url: None,
            },
            _ => {
                return Err(ProviderError::new(
                    ProviderErrorKind::Unknown,
                    format!("Unknown provider: {}", provider_ref),
                ))
            }
        };
        manager
            .create_provider(&spec)
            .map_err(|e| ProviderError::new(ProviderErrorKind::Unknown, e.to_string()))
    }

    fn map_error(err: opencode_core::OpenCodeError) -> ProviderError {
        use opencode_core::OpenCodeError::*;
        let kind = match err {
            Llm(_) => ProviderErrorKind::Unknown,
            _ => ProviderErrorKind::Unknown,
        };
        ProviderError::new(kind, err.to_string())
    }
}

impl Default for LlmProviderGateway {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::provider_gateway::ProviderGateway for LlmProviderGateway {
    async fn validate_provider(
        &self,
        provider: ProviderRef,
    ) -> std::result::Result<ProviderStatus, ProviderError> {
        match self.get_provider(&provider).await {
            Ok(_) => Ok(ProviderStatus::Available),
            Err(e) => match e.kind {
                ProviderErrorKind::AuthenticationFailed => Ok(ProviderStatus::AuthRequired),
                ProviderErrorKind::RateLimited => Ok(ProviderStatus::RateLimited),
                _ => Ok(ProviderStatus::Unavailable),
            },
        }
    }

    async fn list_models(
        &self,
        provider: ProviderRef,
    ) -> std::result::Result<Vec<ModelInfo>, ProviderError> {
        let provider = self.get_provider(&provider).await?;
        let models = provider.get_models();
        Ok(models
            .into_iter()
            .map(|m| ModelInfo {
                id: m.id.clone(),
                name: m.name.clone(),
                provider: provider.provider_name().to_string(),
                capabilities: ModelCapabilities::default(),
            })
            .collect())
    }

    async fn stream_chat(
        &self,
        request: ProviderRequest,
    ) -> std::result::Result<
        Pin<Box<dyn Stream<Item = std::result::Result<ProviderStreamEvent, ProviderError>> + Send>>,
        ProviderError,
    > {
        let provider = self.get_provider(&request.provider).await?;
        let messages: Vec<ChatMessage> = request
            .messages
            .iter()
            .map(|pm| ChatMessage {
                role: pm.role.clone(),
                content: pm.content.clone(),
            })
            .collect();

        let request_id = uuid::Uuid::new_v4().to_string();

        match provider.chat(&messages).await {
            Ok(response) => {
                let events: Vec<std::result::Result<ProviderStreamEvent, ProviderError>> = vec![
                    Ok(ProviderStreamEvent::Started { request_id }),
                    Ok(ProviderStreamEvent::Token { text: response.content }),
                    Ok(ProviderStreamEvent::Completed { usage: None }),
                ];
                let stream = stream::iter(events);
                Ok(Box::pin(stream) as Pin<Box<dyn Stream<Item = std::result::Result<ProviderStreamEvent, ProviderError>> + Send>>)
            }
            Err(e) => Err(Self::map_error(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider_gateway::ProviderGateway;

    #[tokio::test]
    async fn test_provider_gateway_validate_unknown_provider() {
        let gateway = LlmProviderGateway::new();
        let status = gateway
            .validate_provider("nonexistent_provider".to_string())
            .await;
        assert!(status.is_ok());
        match status.unwrap() {
            ProviderStatus::Unavailable => {}
            other => panic!("Expected Unavailable, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_provider_gateway_list_models_unknown() {
        let gateway = LlmProviderGateway::new();
        let result = gateway.list_models("unknown".to_string()).await;
        assert!(result.is_err());
    }
}
