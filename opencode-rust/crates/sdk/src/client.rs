//! OpenCode SDK Client.
//!
//! Main client for interacting with the OpenCode REST API.

use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::auth::ApiKeyAuth;
use crate::error::{SdkError, SdkResult};
use crate::session::{
    AddMessageRequest, AddMessageResponse, CreateSessionRequest, CreateSessionResponse,
    ForkSessionRequest, ForkSessionResponse, SdkSession, SessionInfo,
};
use crate::tools::{ToolCall, ToolDefinition, ToolExecutionResponse, ToolResult};

/// Client configuration.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Base URL of the OpenCode API server.
    /// Defaults to `http://localhost:8080/api`.
    pub base_url: String,

    /// API authentication.
    pub auth: ApiKeyAuth,

    /// Request timeout duration.
    pub timeout: Duration,

    /// Whether to skip TLS verification (for development only).
    pub skip_tls_verification: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: std::env::var("OPENCODE_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080/api".to_string()),
            auth: ApiKeyAuth::default(),
            timeout: Duration::from_secs(30),
            skip_tls_verification: false,
        }
    }
}

/// Builder for creating an OpenCodeClient.
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    config: ClientConfig,
}

impl ClientBuilder {
    /// Creates a new client builder with default configuration.
    pub fn new() -> Self {
        Self {
            config: ClientConfig::default(),
        }
    }

    /// Sets the base URL of the OpenCode API server.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = url.into();
        self
    }

    /// Sets the API key for authentication.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.config.auth = ApiKeyAuth::new(key);
        self
    }

    /// Sets the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Sets whether to skip TLS verification (development only).
    pub fn skip_tls_verification(mut self, skip: bool) -> Self {
        self.config.skip_tls_verification = skip;
        self
    }

    /// Builds the OpenCodeClient.
    pub fn build(self) -> SdkResult<OpenCodeClient> {
        // Validate configuration
        if self.config.auth.key().is_empty() {
            return Err(SdkError::missing_config("api_key"));
        }

        // Build HTTP client
        let client = Client::builder()
            .timeout(self.config.timeout)
            .danger_accept_invalid_certs(self.config.skip_tls_verification)
            .build()
            .map_err(|e| SdkError::network_error(format!("Failed to create HTTP client: {}", e)))?;

        Ok(OpenCodeClient {
            config: Arc::new(self.config),
            http_client: Arc::new(client),
            local_session: Arc::new(RwLock::new(None)),
        })
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// OpenCode SDK Client for programmatic access to OpenCode.
///
/// ## Example
///
/// ```rust,no_run
/// use opencode_sdk::OpenCodeClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = OpenCodeClient::builder()
///         .api_key("sk-your-api-key")
///         .build()?;
///
///     let session = client.create_session(Some("Hello!")).await?;
///     println!("Created session: {}", session.session_id);
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct OpenCodeClient {
    /// Client configuration.
    config: Arc<ClientConfig>,

    /// HTTP client for making requests.
    http_client: Arc<Client>,

    /// Local session storage (for offline mode).
    local_session: Arc<RwLock<Option<SdkSession>>>,
}

impl OpenCodeClient {
    /// Creates a new client builder.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Returns the client configuration.
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    // ==================== Session API ====================

    /// Creates a new session.
    ///
    /// ## Arguments
    ///
    /// * `initial_prompt` - Optional initial prompt to start the session with.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use opencode_sdk::OpenCodeClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OpenCodeClient::builder()
    ///     .api_key("sk-test")
    ///     .build()?;
    ///
    /// let session = client.create_session(Some("Hello!")).await?;
    /// println!("Created session: {}", session.session_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_session(
        &self,
        initial_prompt: Option<&str>,
    ) -> SdkResult<CreateSessionResponse> {
        let request = CreateSessionRequest {
            initial_prompt: initial_prompt.map(|s| s.to_string()),
        };

        let url = format!("{}/sessions", self.config.base_url);
        let response = self
            .http_client
            .post(&url)
            .header("Authorization", self.config.auth.authorization_header())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| SdkError::network_error(format!("Request failed: {}", e)))?;

        if response.status().is_success() {
            response
                .json::<CreateSessionResponse>()
                .await
                .map_err(|e| SdkError::internal_error(format!("Failed to parse response: {}", e)))
        } else {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(SdkError::from_http_status(status, &message))
        }
    }

    /// Gets a session by ID.
    pub async fn get_session(&self, session_id: &str) -> SdkResult<SdkSession> {
        let url = format!("{}/sessions/{}", self.config.base_url, session_id);
        let response = self
            .http_client
            .get(&url)
            .header("Authorization", self.config.auth.authorization_header())
            .send()
            .await
            .map_err(|e| SdkError::network_error(format!("Request failed: {}", e)))?;

        if response.status().is_success() {
            response
                .json::<SdkSession>()
                .await
                .map_err(|e| SdkError::internal_error(format!("Failed to parse response: {}", e)))
        } else {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(SdkError::from_http_status(status, &message))
        }
    }

    /// Lists all sessions.
    pub async fn list_sessions(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> SdkResult<Vec<SessionInfo>> {
        let mut url = format!("{}/sessions", self.config.base_url);
        let mut params = Vec::new();

        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if let Some(o) = offset {
            params.push(format!("offset={}", o));
        }

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        #[derive(Deserialize)]
        struct ListResponse {
            items: Vec<SessionInfo>,
        }

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", self.config.auth.authorization_header())
            .send()
            .await
            .map_err(|e| SdkError::network_error(format!("Request failed: {}", e)))?;

        if response.status().is_success() {
            let list_resp: ListResponse = response.json().await.map_err(|e| {
                SdkError::internal_error(format!("Failed to parse response: {}", e))
            })?;
            Ok(list_resp.items)
        } else {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(SdkError::from_http_status(status, &message))
        }
    }

    /// Forks a session at the given message index.
    pub async fn fork_session(
        &self,
        session_id: &str,
        fork_at_message_index: usize,
    ) -> SdkResult<ForkSessionResponse> {
        let url = format!("{}/sessions/{}/fork", self.config.base_url, session_id);
        let request = ForkSessionRequest {
            fork_at_message_index,
        };

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", self.config.auth.authorization_header())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| SdkError::network_error(format!("Request failed: {}", e)))?;

        if response.status().is_success() {
            response
                .json::<ForkSessionResponse>()
                .await
                .map_err(|e| SdkError::internal_error(format!("Failed to parse response: {}", e)))
        } else {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(SdkError::from_http_status(status, &message))
        }
    }

    /// Adds a message to a session.
    pub async fn add_message(
        &self,
        session_id: &str,
        role: Option<&str>,
        content: &str,
    ) -> SdkResult<AddMessageResponse> {
        let url = format!("{}/sessions/{}/messages", self.config.base_url, session_id);
        let request = AddMessageRequest {
            role: role
                .map(|s| s.to_string())
                .unwrap_or_else(|| "user".to_string()),
            content: content.to_string(),
        };

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", self.config.auth.authorization_header())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| SdkError::network_error(format!("Request failed: {}", e)))?;

        if response.status().is_success() {
            response
                .json::<AddMessageResponse>()
                .await
                .map_err(|e| SdkError::internal_error(format!("Failed to parse response: {}", e)))
        } else {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(SdkError::from_http_status(status, &message))
        }
    }

    /// Aborts a session.
    pub async fn abort_session(&self, session_id: &str) -> SdkResult<()> {
        let url = format!("{}/sessions/{}/abort", self.config.base_url, session_id);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", self.config.auth.authorization_header())
            .send()
            .await
            .map_err(|e| SdkError::network_error(format!("Request failed: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(SdkError::from_http_status(status, &message))
        }
    }

    /// Deletes a session.
    pub async fn delete_session(&self, session_id: &str) -> SdkResult<()> {
        let url = format!("{}/sessions/{}", self.config.base_url, session_id);

        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", self.config.auth.authorization_header())
            .send()
            .await
            .map_err(|e| SdkError::network_error(format!("Request failed: {}", e)))?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(SdkError::from_http_status(status, &message))
        }
    }

    // ==================== Tools API ====================

    /// Lists all available tools.
    pub async fn list_tools(&self) -> SdkResult<Vec<ToolDefinition>> {
        #[derive(Deserialize)]
        struct ListToolsResponse {
            items: Vec<ToolDefinition>,
        }

        let url = format!("{}/tools", self.config.base_url);
        let response = self
            .http_client
            .get(&url)
            .header("Authorization", self.config.auth.authorization_header())
            .send()
            .await
            .map_err(|e| SdkError::network_error(format!("Request failed: {}", e)))?;

        if response.status().is_success() {
            let list_resp: ListToolsResponse = response.json().await.map_err(|e| {
                SdkError::internal_error(format!("Failed to parse response: {}", e))
            })?;
            Ok(list_resp.items)
        } else {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(SdkError::from_http_status(status, &message))
        }
    }

    /// Executes a tool.
    pub async fn execute_tool(&self, tool_call: ToolCall) -> SdkResult<ToolResult> {
        let url = format!("{}/tools/execute", self.config.base_url);

        #[derive(Serialize)]
        struct ExecuteRequest {
            name: String,
            arguments: serde_json::Value,
        }

        let request = ExecuteRequest {
            name: tool_call.name.clone(),
            arguments: tool_call.arguments,
        };

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", self.config.auth.authorization_header())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| SdkError::network_error(format!("Request failed: {}", e)))?;

        if response.status().is_success() {
            let exec_resp: ToolExecutionResponse = response.json().await.map_err(|e| {
                SdkError::internal_error(format!("Failed to parse response: {}", e))
            })?;

            Ok(ToolResult {
                id: exec_resp.id,
                tool_name: exec_resp.tool_name,
                success: exec_resp.success,
                result: exec_resp.result,
                error: exec_resp.error,
                started_at: chrono::Utc::now(),
                completed_at: chrono::Utc::now(),
            })
        } else {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(SdkError::from_http_status(status, &message))
        }
    }

    // ==================== Local Session (Offline Mode) ====================

    /// Creates a local session (offline mode, no server required).
    pub async fn create_local_session(&self, initial_prompt: Option<&str>) -> SdkResult<()> {
        let mut session = SdkSession::new(uuid::Uuid::new_v4());

        if let Some(prompt) = initial_prompt {
            session.add_user_message(prompt);
        }

        let mut local = self.local_session.write().await;
        *local = Some(session);

        Ok(())
    }

    /// Gets the current local session.
    pub async fn get_local_session(&self) -> SdkResult<Option<SdkSession>> {
        let local = self.local_session.read().await;
        Ok(local.clone())
    }

    /// Adds a message to the local session.
    pub async fn add_local_message(&self, role: &str, content: &str) -> SdkResult<()> {
        let mut local = self.local_session.write().await;
        if let Some(ref mut session) = *local {
            match role.to_lowercase().as_str() {
                "user" => session.add_user_message(content),
                "assistant" => session.add_assistant_message(content),
                _ => {
                    session.messages.push(crate::session::SessionMessage {
                        role: role.to_string(),
                        content: content.to_string(),
                    });
                }
            }
            session.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(SdkError::session_not_found("no local session"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.base_url, "http://localhost:8080/api");
        assert!(!config.auth.key().is_empty() || std::env::var("OPENCODE_API_KEY").is_err());
    }

    #[test]
    fn test_client_builder() {
        let client = ClientBuilder::new()
            .base_url("http://localhost:9000/api")
            .api_key("sk-test-key")
            .timeout(Duration::from_secs(60))
            .build();

        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.config.base_url, "http://localhost:9000/api");
    }

    #[test]
    fn test_client_builder_requires_api_key() {
        // Clear the environment variable if set
        std::env::remove_var("OPENCODE_API_KEY");

        let client = ClientBuilder::new()
            .base_url("http://localhost:9000/api")
            .build();

        assert!(client.is_err());
    }
}
