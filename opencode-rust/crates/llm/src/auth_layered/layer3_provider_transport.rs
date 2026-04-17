use async_trait::async_trait;
use opencode_core::OpenCodeError;
use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::layer2_auth_mechanism::AuthMechanism;

pub mod sealed {
    pub trait Sealed {}
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransportHeaders {
    pub custom_headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
}

pub trait ProviderTransport: Send + Sync + sealed::Sealed {
    fn apply_auth(
        &self,
        request: RequestBuilder,
        token: &str,
        mechanism: &AuthMechanism,
    ) -> RequestBuilder;
    fn endpoint_path(&self) -> &str;
    fn required_headers(&self) -> Vec<(&str, &str)>;
}

pub struct OpenAICompatibleTransport;
pub struct AnthropicTransport;
pub struct ResponsesAPITransport;

impl sealed::Sealed for OpenAICompatibleTransport {}
impl ProviderTransport for OpenAICompatibleTransport {
    fn apply_auth(
        &self,
        request: RequestBuilder,
        token: &str,
        mechanism: &AuthMechanism,
    ) -> RequestBuilder {
        match mechanism {
            AuthMechanism::BearerToken
            | AuthMechanism::OAuthBrowser
            | AuthMechanism::DeviceCode => {
                request.header("Authorization", format!("Bearer {}", token))
            }
            AuthMechanism::BasicAuth => {
                let encoded =
                    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, token);
                request.header("Authorization", format!("Basic {}", encoded))
            }
            AuthMechanism::ApiKey => request.header("Authorization", format!("Bearer {}", token)),
            _ => request.header("Authorization", format!("Bearer {}", token)),
        }
    }

    fn endpoint_path(&self) -> &str {
        "/v1/chat/completions"
    }

    fn required_headers(&self) -> Vec<(&str, &str)> {
        vec![("Content-Type", "application/json")]
    }
}

impl sealed::Sealed for AnthropicTransport {}
impl ProviderTransport for AnthropicTransport {
    fn apply_auth(
        &self,
        request: RequestBuilder,
        token: &str,
        mechanism: &AuthMechanism,
    ) -> RequestBuilder {
        match mechanism {
            AuthMechanism::ApiKey => request.header("x-api-key", token),
            AuthMechanism::BearerToken => {
                request.header("Authorization", format!("Bearer {}", token))
            }
            _ => request.header("x-api-key", token),
        }
        .header("anthropic-version", "2023-06-01")
    }

    fn endpoint_path(&self) -> &str {
        "/v1/messages"
    }

    fn required_headers(&self) -> Vec<(&str, &str)> {
        vec![
            ("Content-Type", "application/json"),
            ("anthropic-version", "2023-06-01"),
        ]
    }
}

impl sealed::Sealed for ResponsesAPITransport {}
impl ProviderTransport for ResponsesAPITransport {
    fn apply_auth(
        &self,
        request: RequestBuilder,
        token: &str,
        mechanism: &AuthMechanism,
    ) -> RequestBuilder {
        match mechanism {
            AuthMechanism::BearerToken | AuthMechanism::OAuthBrowser => {
                request.header("Authorization", format!("Bearer {}", token))
            }
            _ => request.header("Authorization", format!("Bearer {}", token)),
        }
    }

    fn endpoint_path(&self) -> &str {
        "/v1/responses"
    }

    fn required_headers(&self) -> Vec<(&str, &str)> {
        vec![("Content-Type", "application/json")]
    }
}

pub struct AwsSigV4Transport {
    pub region: String,
    pub service: String,
    endpoint: String,
}

impl AwsSigV4Transport {
    pub fn new(region: String, service: String) -> Self {
        Self {
            endpoint: "/2023-05-31/inference-profiles".to_string(),
            region,
            service,
        }
    }
}

impl sealed::Sealed for AwsSigV4Transport {}
impl ProviderTransport for AwsSigV4Transport {
    fn apply_auth(
        &self,
        request: RequestBuilder,
        token: &str,
        _mechanism: &AuthMechanism,
    ) -> RequestBuilder {
        request.header("Authorization", format!("Bearer {}", token))
    }

    fn endpoint_path(&self) -> &str {
        &self.endpoint
    }

    fn required_headers(&self) -> Vec<(&str, &str)> {
        vec![("Content-Type", "application/json")]
    }
}

pub struct TransportLayer {
    pub transport: Box<dyn ProviderTransport>,
    pub base_url: String,
    pub headers: TransportHeaders,
}

impl TransportLayer {
    pub fn new(transport: Box<dyn ProviderTransport>, base_url: String) -> Self {
        Self {
            transport,
            base_url,
            headers: TransportHeaders::default(),
        }
    }

    pub fn full_url(&self, path_override: Option<&str>) -> String {
        let path = path_override.unwrap_or_else(|| self.transport.endpoint_path());
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }

    pub fn build_request(&self, client: &reqwest::Client) -> RequestBuilder {
        let mut req = client.get(self.full_url(None));
        for (key, value) in self.transport.required_headers() {
            req = req.header(key, value);
        }
        for (key, value) in &self.headers.custom_headers {
            req = req.header(key, value);
        }
        req
    }
}

#[async_trait]
pub trait AuthenticatedTransport: Send + Sync + sealed::Sealed {
    async fn authenticated_request(
        &self,
        client: &reqwest::Client,
        token: &str,
        mechanism: &AuthMechanism,
    ) -> Result<RequestBuilder, OpenCodeError>;
}

impl sealed::Sealed for TransportLayer {}
#[async_trait]
impl AuthenticatedTransport for TransportLayer {
    async fn authenticated_request(
        &self,
        client: &reqwest::Client,
        token: &str,
        mechanism: &AuthMechanism,
    ) -> Result<RequestBuilder, OpenCodeError> {
        let mut req = self.build_request(client);
        req = self.transport.apply_auth(req, token, mechanism);
        Ok(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_compatible_transport_endpoint() {
        let transport = OpenAICompatibleTransport;
        assert_eq!(transport.endpoint_path(), "/v1/chat/completions");
    }

    #[test]
    fn test_anthropic_transport_endpoint() {
        let transport = AnthropicTransport;
        assert_eq!(transport.endpoint_path(), "/v1/messages");
    }

    #[test]
    fn test_responses_api_transport_endpoint() {
        let transport = ResponsesAPITransport;
        assert_eq!(transport.endpoint_path(), "/v1/responses");
    }

    #[test]
    fn test_transport_layer_full_url() {
        let layer = TransportLayer::new(
            Box::new(OpenAICompatibleTransport),
            "https://api.openai.com".to_string(),
        );
        assert_eq!(
            layer.full_url(None),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn test_transport_layer_full_url_trailing_slash() {
        let layer = TransportLayer::new(
            Box::new(OpenAICompatibleTransport),
            "https://api.openai.com/".to_string(),
        );
        assert_eq!(
            layer.full_url(None),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn test_transport_layer_full_url_with_path_override() {
        let layer = TransportLayer::new(
            Box::new(OpenAICompatibleTransport),
            "https://api.openai.com".to_string(),
        );
        assert_eq!(
            layer.full_url(Some("/v1/completions")),
            "https://api.openai.com/v1/completions"
        );
    }

    #[test]
    fn test_openai_compatible_transport_required_headers() {
        let transport = OpenAICompatibleTransport;
        let headers = transport.required_headers();
        assert!(headers
            .iter()
            .any(|(k, v)| *k == "Content-Type" && *v == "application/json"));
    }

    #[test]
    fn test_anthropic_transport_required_headers() {
        let transport = AnthropicTransport;
        let headers = transport.required_headers();
        assert!(headers
            .iter()
            .any(|(k, v)| *k == "Content-Type" && *v == "application/json"));
        assert!(headers
            .iter()
            .any(|(k, v)| *k == "anthropic-version" && *v == "2023-06-01"));
    }

    #[test]
    fn test_responses_api_transport_required_headers() {
        let transport = ResponsesAPITransport;
        let headers = transport.required_headers();
        assert!(headers
            .iter()
            .any(|(k, v)| *k == "Content-Type" && *v == "application/json"));
    }

    #[test]
    fn test_aws_sigv4_transport_new() {
        let transport = AwsSigV4Transport::new("us-east-1".to_string(), "bedrock".to_string());
        assert_eq!(transport.region, "us-east-1");
        assert_eq!(transport.service, "bedrock");
    }

    #[test]
    fn test_aws_sigv4_transport_endpoint() {
        let transport = AwsSigV4Transport::new("us-east-1".to_string(), "bedrock".to_string());
        assert_eq!(transport.endpoint_path(), "/2023-05-31/inference-profiles");
    }

    #[test]
    fn test_aws_sigv4_transport_required_headers() {
        let transport = AwsSigV4Transport::new("us-east-1".to_string(), "bedrock".to_string());
        let headers = transport.required_headers();
        assert!(headers
            .iter()
            .any(|(k, v)| *k == "Content-Type" && *v == "application/json"));
    }

    #[test]
    fn test_transport_headers_default() {
        let headers = TransportHeaders::default();
        assert!(headers.custom_headers.is_empty());
        assert!(headers.query_params.is_empty());
    }

    #[test]
    fn test_transport_headers_with_values() {
        let mut headers = TransportHeaders::default();
        headers
            .custom_headers
            .insert("X-Custom".to_string(), "value".to_string());
        headers
            .query_params
            .insert("api-version".to_string(), "2023-01-01".to_string());
        assert_eq!(
            headers.custom_headers.get("X-Custom"),
            Some(&"value".to_string())
        );
        assert_eq!(
            headers.query_params.get("api-version"),
            Some(&"2023-01-01".to_string())
        );
    }

    #[test]
    fn test_transport_layer_build_request() {
        let layer = TransportLayer::new(
            Box::new(OpenAICompatibleTransport),
            "https://api.openai.com".to_string(),
        );
        let client = reqwest::Client::new();
        let _request = layer.build_request(&client);
    }

    #[test]
    fn test_transport_layer_with_custom_headers() {
        let mut layer = TransportLayer::new(
            Box::new(OpenAICompatibleTransport),
            "https://api.openai.com".to_string(),
        );
        layer
            .headers
            .custom_headers
            .insert("X-Custom-Header".to_string(), "custom-value".to_string());
        let client = reqwest::Client::new();
        let _request = layer.build_request(&client);
    }

    #[test]
    fn test_openai_compatible_transport_apply_auth_bearer_token() {
        let transport = OpenAICompatibleTransport;
        let client = reqwest::Client::new();
        let request = client.post("https://api.openai.com/v1/chat/completions");
        let _auth_request =
            transport.apply_auth(request, "test-token", &AuthMechanism::BearerToken);
    }

    #[test]
    fn test_openai_compatible_transport_apply_auth_basic_auth() {
        let transport = OpenAICompatibleTransport;
        let client = reqwest::Client::new();
        let request = client.post("https://api.openai.com/v1/chat/completions");
        let _auth_request =
            transport.apply_auth(request, "username:password", &AuthMechanism::BasicAuth);
    }

    #[test]
    fn test_openai_compatible_transport_apply_auth_api_key() {
        let transport = OpenAICompatibleTransport;
        let client = reqwest::Client::new();
        let request = client.post("https://api.openai.com/v1/chat/completions");
        let _auth_request = transport.apply_auth(request, "api-key", &AuthMechanism::ApiKey);
    }

    #[test]
    fn test_anthropic_transport_apply_auth_api_key() {
        let transport = AnthropicTransport;
        let client = reqwest::Client::new();
        let request = client.post("https://api.anthropic.com/v1/messages");
        let _auth_request = transport.apply_auth(request, "sk-ant-api-key", &AuthMechanism::ApiKey);
    }

    #[test]
    fn test_anthropic_transport_apply_auth_bearer_token() {
        let transport = AnthropicTransport;
        let client = reqwest::Client::new();
        let request = client.post("https://api.anthropic.com/v1/messages");
        let _auth_request =
            transport.apply_auth(request, "bearer-token", &AuthMechanism::BearerToken);
    }

    #[test]
    fn test_responses_api_transport_apply_auth_bearer() {
        let transport = ResponsesAPITransport;
        let client = reqwest::Client::new();
        let request = client.post("https://api.openai.com/v1/responses");
        let _auth_request = transport.apply_auth(request, "token", &AuthMechanism::BearerToken);
    }

    #[test]
    fn test_responses_api_transport_apply_auth_oauth_browser() {
        let transport = ResponsesAPITransport;
        let client = reqwest::Client::new();
        let request = client.post("https://api.openai.com/v1/responses");
        let _auth_request =
            transport.apply_auth(request, "oauth-token", &AuthMechanism::OAuthBrowser);
    }

    #[test]
    fn test_aws_sigv4_transport_apply_auth() {
        let transport = AwsSigV4Transport::new("us-east-1".to_string(), "bedrock".to_string());
        let client = reqwest::Client::new();
        let request = client.post("https://bedrock.us-east-1.amazonaws.com/...");
        let _auth_request =
            transport.apply_auth(request, "sigv4-token", &AuthMechanism::BearerToken);
    }
}
