use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::provider::Model;

const AI_GATEWAY_MODELS_PATH: &str = "/api/v1/ai/models";
const GITLAB_DUO_WORKFLOW_FLAG: &str = "gitlab_duo_workflow";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiGatewayModelsResponse {
    pub models: Vec<AiGatewayModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiGatewayModel {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub feature_flags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GitLabProvider {
    pub instance_url: String,
    pub token: String,
    pub client: Client,
}

impl GitLabProvider {
    pub fn new(instance_url: String, token: String) -> Self {
        Self {
            instance_url,
            token,
            client: Client::new(),
        }
    }

    pub fn with_client(instance_url: String, token: String, client: Client) -> Self {
        Self {
            instance_url,
            token,
            client,
        }
    }

    pub async fn discover_models(&self) -> Result<Vec<Model>, GitLabDiscoveryError> {
        if self.token.is_empty() {
            return Err(GitLabDiscoveryError::MissingToken);
        }

        let url = format!("{}{}", self.instance_url.trim_end_matches('/'), AI_GATEWAY_MODELS_PATH);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .map_err(|e| GitLabDiscoveryError::RequestFailed(e.to_string()))?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(GitLabDiscoveryError::InvalidToken);
        }

        if !response.status().is_success() {
            return Err(GitLabDiscoveryError::RequestFailed(format!(
                "HTTP {} from AI Gateway",
                response.status()
            )));
        }

        let gateway_response: AiGatewayModelsResponse = response
            .json()
            .await
            .map_err(|e| GitLabDiscoveryError::ParseFailed(e.to_string()))?;

        let models = gateway_response
            .models
            .into_iter()
            .map(|m| Model::new(&m.id, &m.name))
            .collect();

        Ok(models)
    }
}

pub fn should_enable_gitlab_duo(feature_flags: &[String]) -> bool {
    feature_flags
        .iter()
        .any(|f| f == GITLAB_DUO_WORKFLOW_FLAG)
}

#[derive(Debug, thiserror::Error)]
pub enum GitLabDiscoveryError {
    #[error("GitLab token is missing or empty")]
    MissingToken,
    #[error("GitLab token is invalid or expired")]
    InvalidToken,
    #[error("AI Gateway request failed: {0}")]
    RequestFailed(String),
    #[error("Failed to parse AI Gateway response: {0}")]
    ParseFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitlab_provider_creation_with_instance_url_and_token() {
        let provider = GitLabProvider::new(
            "https://gitlab.example.com".to_string(),
            "glpat-test-token".to_string(),
        );

        assert_eq!(provider.instance_url, "https://gitlab.example.com");
        assert_eq!(provider.token, "glpat-test-token");
    }

    #[test]
    fn test_gitlab_provider_stores_instance_url() {
        let instance_url = "https://my-gitlab.internal".to_string();
        let provider = GitLabProvider::new(instance_url.clone(), "token".to_string());
        assert_eq!(provider.instance_url, instance_url);
    }

    #[test]
    fn test_gitlab_provider_stores_token() {
        let token = "glpat-super-secret-token".to_string();
        let provider = GitLabProvider::new("https://gitlab.com".to_string(), token.clone());
        assert_eq!(provider.token, token);
    }

    #[test]
    fn test_should_enable_gitlab_duo_with_flag_present() {
        let flags = vec![GITLAB_DUO_WORKFLOW_FLAG.to_string()];
        assert!(should_enable_gitlab_duo(&flags));
    }

    #[test]
    fn test_should_enable_gitlab_duo_with_flag_among_others() {
        let flags = vec![
            "other_flag".to_string(),
            GITLAB_DUO_WORKFLOW_FLAG.to_string(),
            "yet_another_flag".to_string(),
        ];
        assert!(should_enable_gitlab_duo(&flags));
    }

    #[test]
    fn test_should_enable_gitlab_duo_without_flag() {
        let flags = vec!["some_other_flag".to_string(), "unrelated_flag".to_string()];
        assert!(!should_enable_gitlab_duo(&flags));
    }

    #[test]
    fn test_should_enable_gitlab_duo_with_empty_flags() {
        let flags: Vec<String> = vec![];
        assert!(!should_enable_gitlab_duo(&flags));
    }

    #[test]
    fn test_should_enable_gitlab_duo_checks_exact_flag_name() {
        let flags = vec!["gitlab_duo".to_string(), "duo_workflow".to_string()];
        assert!(!should_enable_gitlab_duo(&flags));
    }

    #[tokio::test]
    async fn test_discover_models_returns_error_for_empty_token() {
        let provider = GitLabProvider::new(
            "https://gitlab.example.com".to_string(),
            "".to_string(),
        );

        let result = provider.discover_models().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GitLabDiscoveryError::MissingToken => {}
            other => panic!("expected MissingToken error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_discover_models_returns_vec_model() {
        let raw_response = serde_json::json!({
            "models": [
                { "id": "claude-3-5-sonnet", "name": "Claude 3.5 Sonnet", "feature_flags": [] },
                { "id": "mistral-small", "name": "Mistral Small", "feature_flags": ["gitlab_duo_workflow"] }
            ]
        });

        let parsed: AiGatewayModelsResponse =
            serde_json::from_value(raw_response).expect("parse should succeed");

        let models: Vec<Model> = parsed
            .models
            .into_iter()
            .map(|m| Model::new(&m.id, &m.name))
            .collect();

        assert_eq!(models.len(), 2);
        assert_eq!(models[0].id, "claude-3-5-sonnet");
        assert_eq!(models[0].name, "Claude 3.5 Sonnet");
        assert_eq!(models[1].id, "mistral-small");
    }

    #[test]
    fn test_ai_gateway_model_deserializes_without_feature_flags() {
        let raw = serde_json::json!({ "id": "gpt-4", "name": "GPT-4" });
        let model: AiGatewayModel = serde_json::from_value(raw).unwrap();
        assert_eq!(model.id, "gpt-4");
        assert!(model.feature_flags.is_empty());
    }

    #[test]
    fn test_ai_gateway_model_deserializes_with_feature_flags() {
        let raw = serde_json::json!({
            "id": "duo-model",
            "name": "Duo Model",
            "feature_flags": ["gitlab_duo_workflow"]
        });
        let model: AiGatewayModel = serde_json::from_value(raw).unwrap();
        assert_eq!(model.feature_flags, vec!["gitlab_duo_workflow"]);
    }
}
