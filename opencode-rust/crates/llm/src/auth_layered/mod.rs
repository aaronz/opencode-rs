pub mod copilot_oauth;
pub mod google_oauth;
pub mod layer1_credential_source;
pub mod layer2_auth_mechanism;
pub mod layer3_provider_transport;
pub mod layer4_runtime_access_control;
pub mod tests;

pub use copilot_oauth::{
    is_oauth_only_provider, CopilotLocalCallbackServer, CopilotOAuthCallback, CopilotOAuthRequest,
    CopilotOAuthService, CopilotOAuthSession, CopilotOAuthStore,
};
pub use google_oauth::{
    GoogleLocalCallbackServer, GoogleOAuthCallback, GoogleOAuthRequest, GoogleOAuthService,
    GoogleOAuthSession, GoogleOAuthStore,
};
pub use layer1_credential_source::{
    CompositeCredentialResolver, CredentialResolver, CredentialSource,
};
pub use layer2_auth_mechanism::AuthMechanism;
pub use layer3_provider_transport::{
    AnthropicTransport, AwsSigV4Transport, OpenAICompatibleTransport, ProviderTransport,
    TransportLayer,
};
pub use layer4_runtime_access_control::{AccessControlResult, RuntimeAccessControl};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAuthSpec {
    pub provider_id: String,
    pub mechanism: AuthMechanism,
    pub source: CredentialSource,
    pub supports_interactive_login: bool,
    pub supports_env_override: bool,
    pub supports_header_injection: bool,
}
