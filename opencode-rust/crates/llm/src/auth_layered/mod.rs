pub mod layer1_credential_source;
pub mod layer2_auth_mechanism;
pub mod layer3_provider_transport;
pub mod layer4_runtime_access_control;
pub mod tests;

pub use layer1_credential_source::{CredentialSource, CredentialResolver, CompositeCredentialResolver};
pub use layer2_auth_mechanism::AuthMechanism;
pub use layer3_provider_transport::{
    ProviderTransport, TransportLayer, OpenAICompatibleTransport, AnthropicTransport, AwsSigV4Transport,
};
pub use layer4_runtime_access_control::{RuntimeAccessControl, AccessControlResult};

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
