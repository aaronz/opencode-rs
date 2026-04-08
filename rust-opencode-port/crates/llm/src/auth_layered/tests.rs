use std::collections::HashMap;

use crate::auth_layered::layer1_credential_source::{CredentialResolver, ResolvedCredential};
use crate::auth_layered::layer3_provider_transport::ProviderTransport;
use crate::auth_layered::{
    AccessControlResult, AnthropicTransport, AuthMechanism, AwsSigV4Transport,
    CompositeCredentialResolver, CredentialSource, OpenAICompatibleTransport, RuntimeAccessControl,
    TransportLayer,
};

#[test]
fn test_layer1_credential_source_env_var() {
    let mut env_vars = HashMap::new();
    env_vars.insert("openai".to_string(), "sk-env-key".to_string());

    let resolver = CompositeCredentialResolver::new().with_inline(env_vars);
    let cred = resolver.resolve("openai", &CredentialSource::ConfigInline);

    assert!(cred.is_some());
    assert_eq!(cred.unwrap().value, "sk-env-key");
}

#[test]
fn test_layer1_credential_source_fallback_order() {
    let mut creds = HashMap::new();
    creds.insert("openai".to_string(), "sk-fallback-key".to_string());

    let resolver = CompositeCredentialResolver::new().with_inline(creds);
    let cred = resolver.resolve_with_fallback(
        "openai",
        &[CredentialSource::OAuthStore, CredentialSource::ConfigInline],
    );

    assert!(cred.is_some());
}

#[test]
fn test_layer2_auth_mechanism_types() {
    assert!(!AuthMechanism::ApiKey.requires_interactive_login());

    assert!(AuthMechanism::OAuthBrowser.requires_interactive_login());
    assert!(AuthMechanism::DeviceCode.requires_interactive_login());
    assert!(!AuthMechanism::BearerToken.requires_interactive_login());

    assert!(AuthMechanism::AwsCredentialChain.supports_refresh());
    assert!(AuthMechanism::OAuthBrowser.supports_refresh());
    assert!(!AuthMechanism::ApiKey.supports_refresh());

    assert!(AuthMechanism::AwsCredentialChain.is_cloud_native());
    assert!(AuthMechanism::ServiceAccountJson.is_cloud_native());
    assert!(!AuthMechanism::ApiKey.is_cloud_native());
}

#[test]
fn test_layer3_openai_transport() {
    let transport = OpenAICompatibleTransport;
    assert_eq!(transport.endpoint_path(), "/v1/chat/completions");

    let headers = transport.required_headers();
    assert!(headers.iter().any(|(k, _)| *k == "Content-Type"));
}

#[test]
fn test_layer3_anthropic_transport() {
    let transport = AnthropicTransport;
    assert_eq!(transport.endpoint_path(), "/v1/messages");

    let headers = transport.required_headers();
    assert!(headers.iter().any(|(k, _)| *k == "Content-Type"));
    assert!(headers.iter().any(|(k, _)| *k == "anthropic-version"));
}

#[test]
fn test_layer3_aws_sigv4_transport() {
    let transport = AwsSigV4Transport::new("us-east-1".to_string(), "bedrock".to_string());
    assert_eq!(transport.endpoint_path(), "/2023-05-31/inference-profiles");
}

#[test]
fn test_layer3_transport_layer_full_url() {
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
fn test_layer4_access_control_default_allow() {
    let acl = RuntimeAccessControl::new();
    assert!(matches!(
        acl.check_provider_access("openai"),
        AccessControlResult::Allowed
    ));
}

#[test]
fn test_layer4_access_control_denylist() {
    let mut denylist = std::collections::HashSet::new();
    denylist.insert("disabled".to_string());

    let acl = RuntimeAccessControl::new().with_denylist(denylist);
    assert!(matches!(
        acl.check_provider_access("disabled"),
        AccessControlResult::Denied(_)
    ));
    assert!(matches!(
        acl.check_provider_access("openai"),
        AccessControlResult::Allowed
    ));
}

#[test]
fn test_layer4_access_control_allowlist() {
    let mut allowlist = std::collections::HashSet::new();
    allowlist.insert("openai".to_string());

    let acl = RuntimeAccessControl::new().with_allowlist(allowlist);
    assert!(matches!(
        acl.check_provider_access("openai"),
        AccessControlResult::Allowed
    ));
    assert!(matches!(
        acl.check_provider_access("anthropic"),
        AccessControlResult::Denied(_)
    ));
}

#[test]
fn test_layer4_denylist_precedence() {
    let mut allowlist = std::collections::HashSet::new();
    allowlist.insert("openai".to_string());
    let mut denylist = std::collections::HashSet::new();
    denylist.insert("openai".to_string());

    let acl = RuntimeAccessControl::new()
        .with_allowlist(allowlist)
        .with_denylist(denylist);

    assert!(matches!(
        acl.check_provider_access("openai"),
        AccessControlResult::Denied(_)
    ));
}

#[test]
fn test_layer4_server_basic_auth() {
    let acl = RuntimeAccessControl::new().with_server_basic_auth(true);
    assert!(acl.is_server_auth_required());

    let acl2 = RuntimeAccessControl::new().with_server_basic_auth(false);
    assert!(!acl2.is_server_auth_required());
}

#[test]
fn test_integration_all_layers() {
    let mut creds = HashMap::new();
    creds.insert("openai".to_string(), "sk-integration".to_string());
    let resolver = CompositeCredentialResolver::new().with_inline(creds);

    let mut denylist = std::collections::HashSet::new();
    denylist.insert("banned".to_string());
    let acl = RuntimeAccessControl::new().with_denylist(denylist);

    let result = acl.check_provider_access("openai");
    assert!(matches!(result, AccessControlResult::Allowed));

    let result = acl.check_provider_access("banned");
    assert!(matches!(result, AccessControlResult::Denied(_)));

    let cred = resolver.resolve("openai", &CredentialSource::ConfigInline);
    assert!(cred.is_some());
    assert_eq!(cred.unwrap().value, "sk-integration");
}
