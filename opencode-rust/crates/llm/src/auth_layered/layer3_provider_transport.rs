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
}
