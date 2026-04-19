use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuthMethod {
    Browser,
    ApiKey,
    Local,
    DeviceFlow,
}

impl AuthMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthMethod::Browser => "browser",
            AuthMethod::ApiKey => "api_key",
            AuthMethod::Local => "local",
            AuthMethod::DeviceFlow => "device_flow",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AuthMethod::Browser => "Browser auth",
            AuthMethod::ApiKey => "API key",
            AuthMethod::Local => "Local",
            AuthMethod::DeviceFlow => "Device code",
        }
    }
}

pub trait ProviderAuth {
    fn supported_auth_methods(&self) -> Vec<AuthMethod>;
}

pub fn get_provider_auth_methods(provider_id: &str) -> Vec<AuthMethod> {
    match provider_id {
        "openai" => vec![AuthMethod::Browser, AuthMethod::ApiKey],
        "google" => vec![AuthMethod::Browser],
        "copilot" => vec![AuthMethod::Browser],
        "ollama" => vec![AuthMethod::Local],
        "lmstudio" => vec![AuthMethod::Local],
        "local" => vec![AuthMethod::Local],
        _ => vec![AuthMethod::ApiKey],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_method_as_str() {
        assert_eq!(AuthMethod::Browser.as_str(), "browser");
        assert_eq!(AuthMethod::ApiKey.as_str(), "api_key");
        assert_eq!(AuthMethod::Local.as_str(), "local");
        assert_eq!(AuthMethod::DeviceFlow.as_str(), "device_flow");
    }

    #[test]
    fn test_auth_method_display_name() {
        assert_eq!(AuthMethod::Browser.display_name(), "Browser auth");
        assert_eq!(AuthMethod::ApiKey.display_name(), "API key");
        assert_eq!(AuthMethod::Local.display_name(), "Local");
        assert_eq!(AuthMethod::DeviceFlow.display_name(), "Device code");
    }

    #[test]
    fn test_get_provider_auth_methods_openai() {
        let methods = get_provider_auth_methods("openai");
        assert_eq!(methods, vec![AuthMethod::Browser, AuthMethod::ApiKey]);
    }

    #[test]
    fn test_get_provider_auth_methods_google() {
        let methods = get_provider_auth_methods("google");
        assert_eq!(methods, vec![AuthMethod::Browser]);
    }

    #[test]
    fn test_get_provider_auth_methods_copilot() {
        let methods = get_provider_auth_methods("copilot");
        assert_eq!(methods, vec![AuthMethod::Browser]);
    }

    #[test]
    fn test_get_provider_auth_methods_ollama() {
        let methods = get_provider_auth_methods("ollama");
        assert_eq!(methods, vec![AuthMethod::Local]);
    }

    #[test]
    fn test_get_provider_auth_methods_lmstudio() {
        let methods = get_provider_auth_methods("lmstudio");
        assert_eq!(methods, vec![AuthMethod::Local]);
    }

    #[test]
    fn test_get_provider_auth_methods_anthropic() {
        let methods = get_provider_auth_methods("anthropic");
        assert_eq!(methods, vec![AuthMethod::ApiKey]);
    }

    #[test]
    fn test_get_provider_auth_methods_unknown_defaults_to_api_key() {
        let methods = get_provider_auth_methods("unknown");
        assert_eq!(methods, vec![AuthMethod::ApiKey]);
    }

    #[test]
    fn test_provider_auth_trait_impl() {
        struct TestProvider {
            methods: Vec<AuthMethod>,
        }

        impl ProviderAuth for TestProvider {
            fn supported_auth_methods(&self) -> Vec<AuthMethod> {
                self.methods.clone()
            }
        }

        let provider = TestProvider {
            methods: vec![AuthMethod::Browser, AuthMethod::ApiKey],
        };

        assert_eq!(
            provider.supported_auth_methods(),
            vec![AuthMethod::Browser, AuthMethod::ApiKey]
        );
    }
}
