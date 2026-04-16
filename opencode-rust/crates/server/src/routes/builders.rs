use crate::routes::provider::{
    ProviderConfigChangedEvent, ProviderResponse, ProviderStatusResponse,
};
use crate::routes::status::{PluginStatus, ProviderStatus, StatusResponse};
use opencode_llm::AuthStrategy;

#[derive(Debug, Clone)]
pub struct StatusResponseBuilder {
    version: Option<String>,
    rustc_version: Option<String>,
    build_timestamp: Option<String>,
    status: Option<String>,
    uptime_seconds: Option<u64>,
    active_sessions: Option<usize>,
    total_sessions: Option<usize>,
    providers: Vec<ProviderStatus>,
    plugins: Vec<PluginStatus>,
}

impl StatusResponseBuilder {
    pub fn new() -> Self {
        Self {
            version: None,
            rustc_version: None,
            build_timestamp: None,
            status: None,
            uptime_seconds: None,
            active_sessions: None,
            total_sessions: None,
            providers: Vec::new(),
            plugins: Vec::new(),
        }
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn rustc_version(mut self, version: impl Into<String>) -> Self {
        self.rustc_version = Some(version.into());
        self
    }

    pub fn build_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.build_timestamp = Some(timestamp.into());
        self
    }

    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn uptime_seconds(mut self, seconds: u64) -> Self {
        self.uptime_seconds = Some(seconds);
        self
    }

    pub fn active_sessions(mut self, count: usize) -> Self {
        self.active_sessions = Some(count);
        self
    }

    pub fn total_sessions(mut self, count: usize) -> Self {
        self.total_sessions = Some(count);
        self
    }

    pub fn add_provider(mut self, provider: ProviderStatus) -> Self {
        self.providers.push(provider);
        self
    }

    pub fn providers(mut self, providers: Vec<ProviderStatus>) -> Self {
        self.providers = providers;
        self
    }

    pub fn add_plugin(mut self, plugin: PluginStatus) -> Self {
        self.plugins.push(plugin);
        self
    }

    pub fn plugins(mut self, plugins: Vec<PluginStatus>) -> Self {
        self.plugins = plugins;
        self
    }

    pub fn build(self) -> StatusResponse {
        StatusResponse {
            version: self.version.unwrap_or_else(|| "unknown".to_string()),
            rustc_version: self.rustc_version.unwrap_or_else(|| "unknown".to_string()),
            build_timestamp: self
                .build_timestamp
                .unwrap_or_else(|| "unknown".to_string()),
            status: self.status.unwrap_or_else(|| "unknown".to_string()),
            uptime_seconds: self.uptime_seconds.unwrap_or(0),
            active_sessions: self.active_sessions.unwrap_or(0),
            total_sessions: self.total_sessions.unwrap_or(0),
            providers: self.providers,
            plugins: self.plugins,
        }
    }
}

impl Default for StatusResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ProviderStatusBuilder {
    name: Option<String>,
    status: Option<String>,
    model: Option<String>,
}

impl ProviderStatusBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            status: None,
            model: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn build(self) -> ProviderStatus {
        ProviderStatus {
            name: self.name.unwrap_or_else(|| "unknown".to_string()),
            status: self.status.unwrap_or_else(|| "unknown".to_string()),
            model: self.model.unwrap_or_else(|| "unknown".to_string()),
        }
    }
}

impl Default for ProviderStatusBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PluginStatusBuilder {
    name: Option<String>,
    version: Option<String>,
    status: Option<String>,
}

impl PluginStatusBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            status: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn build(self) -> PluginStatus {
        PluginStatus {
            name: self.name.unwrap_or_else(|| "unknown".to_string()),
            version: self.version.unwrap_or_else(|| "1.0.0".to_string()),
            status: self.status.unwrap_or_else(|| "unknown".to_string()),
        }
    }
}

impl Default for PluginStatusBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ProviderResponseBuilder {
    provider_id: Option<String>,
    endpoint: Option<String>,
    auth_strategy: Option<AuthStrategy>,
}

impl ProviderResponseBuilder {
    pub fn new() -> Self {
        Self {
            provider_id: None,
            endpoint: None,
            auth_strategy: None,
        }
    }

    pub fn provider_id(mut self, id: impl Into<String>) -> Self {
        self.provider_id = Some(id.into());
        self
    }

    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    pub fn auth_strategy(mut self, strategy: AuthStrategy) -> Self {
        self.auth_strategy = Some(strategy);
        self
    }

    pub fn build(self) -> ProviderResponse {
        ProviderResponse {
            provider_id: self.provider_id.unwrap_or_else(|| "unknown".to_string()),
            endpoint: self.endpoint.unwrap_or_else(|| String::new()),
            auth_strategy: self.auth_strategy.unwrap_or(AuthStrategy::None),
        }
    }
}

impl Default for ProviderResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ProviderStatusResponseBuilder {
    provider_id: Option<String>,
    enabled: Option<bool>,
    exists: Option<bool>,
}

impl ProviderStatusResponseBuilder {
    pub fn new() -> Self {
        Self {
            provider_id: None,
            enabled: None,
            exists: None,
        }
    }

    pub fn provider_id(mut self, id: impl Into<String>) -> Self {
        self.provider_id = Some(id.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    pub fn exists(mut self, exists: bool) -> Self {
        self.exists = Some(exists);
        self
    }

    pub fn build(self) -> ProviderStatusResponse {
        ProviderStatusResponse {
            provider_id: self.provider_id.unwrap_or_else(|| "unknown".to_string()),
            enabled: self.enabled.unwrap_or(false),
            exists: self.exists.unwrap_or(false),
        }
    }
}

impl Default for ProviderStatusResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ProviderConfigChangedEventBuilder {
    event: Option<String>,
    provider_id: Option<String>,
    enabled: Option<bool>,
}

impl ProviderConfigChangedEventBuilder {
    pub fn new() -> Self {
        Self {
            event: None,
            provider_id: None,
            enabled: None,
        }
    }

    pub fn event(mut self, event: impl Into<String>) -> Self {
        self.event = Some(event.into());
        self
    }

    pub fn provider_id(mut self, id: impl Into<String>) -> Self {
        self.provider_id = Some(id.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    pub fn build(self) -> ProviderConfigChangedEvent {
        ProviderConfigChangedEvent {
            event: self
                .event
                .unwrap_or_else(|| "provider_config_changed".to_string()),
            provider_id: self.provider_id.unwrap_or_else(|| "unknown".to_string()),
            enabled: self.enabled.unwrap_or(false),
        }
    }
}

impl Default for ProviderConfigChangedEventBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_response_builder_default_values() {
        let response = StatusResponseBuilder::new().build();
        assert_eq!(response.version, "unknown");
        assert_eq!(response.rustc_version, "unknown");
        assert_eq!(response.build_timestamp, "unknown");
        assert_eq!(response.status, "unknown");
        assert_eq!(response.uptime_seconds, 0);
        assert_eq!(response.active_sessions, 0);
        assert_eq!(response.total_sessions, 0);
        assert!(response.providers.is_empty());
        assert!(response.plugins.is_empty());
    }

    #[test]
    fn status_response_builder_with_values() {
        let response = StatusResponseBuilder::new()
            .version("1.0.0")
            .rustc_version("rustc 1.75.0")
            .build_timestamp("2024-01-15T12:00:00Z")
            .status("running")
            .uptime_seconds(3600)
            .active_sessions(5)
            .total_sessions(100)
            .build();

        assert_eq!(response.version, "1.0.0");
        assert_eq!(response.rustc_version, "rustc 1.75.0");
        assert_eq!(response.build_timestamp, "2024-01-15T12:00:00Z");
        assert_eq!(response.status, "running");
        assert_eq!(response.uptime_seconds, 3600);
        assert_eq!(response.active_sessions, 5);
        assert_eq!(response.total_sessions, 100);
    }

    #[test]
    fn status_response_builder_with_providers_and_plugins() {
        let provider = ProviderStatusBuilder::new()
            .name("openai")
            .status("ready")
            .model("gpt-4")
            .build();

        let plugin = PluginStatusBuilder::new()
            .name("test-plugin")
            .version("1.2.3")
            .status("loaded")
            .build();

        let response = StatusResponseBuilder::new()
            .version("1.0.0")
            .providers(vec![provider])
            .plugins(vec![plugin])
            .build();

        assert_eq!(response.providers.len(), 1);
        assert_eq!(response.providers[0].name, "openai");
        assert_eq!(response.plugins.len(), 1);
        assert_eq!(response.plugins[0].version, "1.2.3");
    }

    #[test]
    fn status_response_builder_chaining() {
        let response = StatusResponseBuilder::new()
            .version("1.0.0")
            .add_provider(ProviderStatusBuilder::new().name("p1").build())
            .add_provider(ProviderStatusBuilder::new().name("p2").build())
            .add_plugin(PluginStatusBuilder::new().name("plugin1").build())
            .build();

        assert_eq!(response.providers.len(), 2);
        assert_eq!(response.plugins.len(), 1);
    }

    #[test]
    fn provider_status_builder_default_values() {
        let status = ProviderStatusBuilder::new().build();
        assert_eq!(status.name, "unknown");
        assert_eq!(status.status, "unknown");
        assert_eq!(status.model, "unknown");
    }

    #[test]
    fn provider_status_builder_with_values() {
        let status = ProviderStatusBuilder::new()
            .name("anthropic")
            .status("ready")
            .model("claude-3")
            .build();

        assert_eq!(status.name, "anthropic");
        assert_eq!(status.status, "ready");
        assert_eq!(status.model, "claude-3");
    }

    #[test]
    fn plugin_status_builder_default_values() {
        let status = PluginStatusBuilder::new().build();
        assert_eq!(status.name, "unknown");
        assert_eq!(status.version, "1.0.0");
        assert_eq!(status.status, "unknown");
    }

    #[test]
    fn plugin_status_builder_with_values() {
        let status = PluginStatusBuilder::new()
            .name("my-plugin")
            .version("2.0.0")
            .status("loaded")
            .build();

        assert_eq!(status.name, "my-plugin");
        assert_eq!(status.version, "2.0.0");
        assert_eq!(status.status, "loaded");
    }

    #[test]
    fn provider_response_builder_default_values() {
        let response = ProviderResponseBuilder::new().build();
        assert_eq!(response.provider_id, "unknown");
        assert_eq!(response.endpoint, "");
        matches!(response.auth_strategy, AuthStrategy::None);
    }

    #[test]
    fn provider_response_builder_with_values() {
        let response = ProviderResponseBuilder::new()
            .provider_id("openai")
            .endpoint("https://api.openai.com")
            .auth_strategy(AuthStrategy::BearerApiKey { header_name: None })
            .build();

        assert_eq!(response.provider_id, "openai");
        assert_eq!(response.endpoint, "https://api.openai.com");
        matches!(response.auth_strategy, AuthStrategy::BearerApiKey { .. });
    }

    #[test]
    fn provider_status_response_builder_default_values() {
        let response = ProviderStatusResponseBuilder::new().build();
        assert_eq!(response.provider_id, "unknown");
        assert!(!response.enabled);
        assert!(!response.exists);
    }

    #[test]
    fn provider_status_response_builder_with_values() {
        let response = ProviderStatusResponseBuilder::new()
            .provider_id("anthropic")
            .enabled(true)
            .exists(true)
            .build();

        assert_eq!(response.provider_id, "anthropic");
        assert!(response.enabled);
        assert!(response.exists);
    }

    #[test]
    fn provider_config_changed_event_builder_default_values() {
        let event = ProviderConfigChangedEventBuilder::new().build();
        assert_eq!(event.event, "provider_config_changed");
        assert_eq!(event.provider_id, "unknown");
        assert!(!event.enabled);
    }

    #[test]
    fn provider_config_changed_event_builder_with_values() {
        let event = ProviderConfigChangedEventBuilder::new()
            .event("custom_event")
            .provider_id("openai")
            .enabled(true)
            .build();

        assert_eq!(event.event, "custom_event");
        assert_eq!(event.provider_id, "openai");
        assert!(event.enabled);
    }

    #[test]
    fn status_response_serializes_correctly() {
        let response = StatusResponseBuilder::new()
            .version("1.0.0")
            .rustc_version("rustc 1.75.0")
            .build_timestamp("2024-01-15T12:00:00Z")
            .status("running")
            .uptime_seconds(3600)
            .active_sessions(5)
            .total_sessions(100)
            .build();

        let json = serde_json::to_string(&response).expect("should serialize");
        assert!(json.contains("\"version\":\"1.0.0\""));
        assert!(json.contains("\"rustc_version\":\"rustc 1.75.0\""));
        assert!(json.contains("\"status\":\"running\""));
        assert!(json.contains("\"uptime_seconds\":3600"));
    }

    #[test]
    fn provider_status_serializes_correctly() {
        let status = ProviderStatusBuilder::new()
            .name("openai")
            .status("ready")
            .model("gpt-4")
            .build();

        let json = serde_json::to_string(&status).expect("should serialize");
        assert!(json.contains("\"name\":\"openai\""));
        assert!(json.contains("\"status\":\"ready\""));
        assert!(json.contains("\"model\":\"gpt-4\""));
    }

    #[test]
    fn plugin_status_serializes_correctly() {
        let status = PluginStatusBuilder::new()
            .name("test-plugin")
            .version("1.0.0")
            .status("loaded")
            .build();

        let json = serde_json::to_string(&status).expect("should serialize");
        assert!(json.contains("\"name\":\"test-plugin\""));
        assert!(json.contains("\"version\":\"1.0.0\""));
        assert!(json.contains("\"status\":\"loaded\""));
    }

    #[test]
    fn provider_response_serializes_correctly() {
        let response = ProviderResponseBuilder::new()
            .provider_id("openai")
            .endpoint("https://api.openai.com")
            .auth_strategy(AuthStrategy::BearerApiKey { header_name: None })
            .build();

        let json = serde_json::to_string(&response).expect("should serialize");
        assert!(json.contains("\"provider_id\":\"openai\""));
        assert!(json.contains("\"endpoint\":\"https://api.openai.com\""));
    }

    #[test]
    fn provider_status_response_serializes_correctly() {
        let response = ProviderStatusResponseBuilder::new()
            .provider_id("anthropic")
            .enabled(true)
            .exists(true)
            .build();

        let json = serde_json::to_string(&response).expect("should serialize");
        assert!(json.contains("\"provider_id\":\"anthropic\""));
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"exists\":true"));
    }

    #[test]
    fn provider_config_changed_event_serializes_correctly() {
        let event = ProviderConfigChangedEventBuilder::new()
            .provider_id("openai")
            .enabled(false)
            .build();

        let json = serde_json::to_string(&event).expect("should serialize");
        assert!(json.contains("\"event\":\"provider_config_changed\""));
        assert!(json.contains("\"provider_id\":\"openai\""));
        assert!(json.contains("\"enabled\":false"));
    }

    #[test]
    fn builder_pattern_fluent_interface() {
        let result = StatusResponseBuilder::new()
            .version("1.0.0")
            .status("running")
            .uptime_seconds(100)
            .active_sessions(2)
            .total_sessions(50)
            .rustc_version("rustc 1.80")
            .build_timestamp("2024-06-01T00:00:00Z")
            .providers(vec![])
            .plugins(vec![])
            .build();

        assert_eq!(result.version, "1.0.0");
        assert_eq!(result.status, "running");
        assert_eq!(result.uptime_seconds, 100);
    }

    #[test]
    fn builder_creates_valid_request_response_pairs() {
        let provider_status = ProviderStatusBuilder::new()
            .name("test-provider")
            .status("ready")
            .model("test-model")
            .build();

        let plugin_status = PluginStatusBuilder::new()
            .name("test-plugin")
            .version("1.0.0")
            .status("loaded")
            .build();

        let status_response = StatusResponseBuilder::new()
            .version("2.0.0")
            .rustc_version("rustc 2.0.0")
            .build_timestamp("2024-12-01T00:00:00Z")
            .status("operational")
            .uptime_seconds(7200)
            .active_sessions(10)
            .total_sessions(200)
            .providers(vec![provider_status])
            .plugins(vec![plugin_status])
            .build();

        assert_eq!(status_response.version, "2.0.0");
        assert_eq!(status_response.providers.len(), 1);
        assert_eq!(status_response.plugins.len(), 1);
        assert_eq!(status_response.active_sessions, 10);
    }

    #[test]
    fn empty_builder_produces_sensible_defaults() {
        let response = StatusResponseBuilder::new().build();
        assert!(!serde_json::to_string(&response).unwrap().is_empty());
    }
}
