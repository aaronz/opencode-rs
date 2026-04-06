use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthMechanism {
    ApiKey,
    BearerToken,
    BasicAuth,
    OAuthBrowser,
    DeviceCode,
    AwsCredentialChain,
    ServiceAccountJson,
    SsoGatewayToken,
}

impl AuthMechanism {
    pub fn requires_interactive_login(&self) -> bool {
        matches!(self, Self::OAuthBrowser | Self::DeviceCode)
    }

    pub fn supports_refresh(&self) -> bool {
        matches!(
            self,
            Self::OAuthBrowser | Self::DeviceCode | Self::AwsCredentialChain
        )
    }

    pub fn is_cloud_native(&self) -> bool {
        matches!(
            self,
            Self::AwsCredentialChain | Self::ServiceAccountJson | Self::SsoGatewayToken
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interactive_login_mechanisms() {
        assert!(AuthMechanism::OAuthBrowser.requires_interactive_login());
        assert!(AuthMechanism::DeviceCode.requires_interactive_login());
        assert!(!AuthMechanism::ApiKey.requires_interactive_login());
        assert!(!AuthMechanism::BearerToken.requires_interactive_login());
    }

    #[test]
    fn test_refresh_capable_mechanisms() {
        assert!(AuthMechanism::OAuthBrowser.supports_refresh());
        assert!(AuthMechanism::DeviceCode.supports_refresh());
        assert!(AuthMechanism::AwsCredentialChain.supports_refresh());
        assert!(!AuthMechanism::ApiKey.supports_refresh());
    }

    #[test]
    fn test_cloud_native_mechanisms() {
        assert!(AuthMechanism::AwsCredentialChain.is_cloud_native());
        assert!(AuthMechanism::ServiceAccountJson.is_cloud_native());
        assert!(AuthMechanism::SsoGatewayToken.is_cloud_native());
        assert!(!AuthMechanism::ApiKey.is_cloud_native());
    }
}
