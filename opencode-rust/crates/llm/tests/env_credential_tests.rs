use opencode_llm::auth_layered::layer1_credential_source::{
    CompositeCredentialResolver, CredentialResolver, CredentialSource,
};

#[test]
fn test_credential_source_priority_ordering() {
    let sources = vec![
        CredentialSource::OAuthStore,
        CredentialSource::AwsCredentialChain,
        CredentialSource::SystemKeychain,
        CredentialSource::AuthFile,
        CredentialSource::EnvVar,
        CredentialSource::ConfigInline,
    ];

    let mut sorted = sources.clone();
    sorted.sort_by_key(|s| -(s.priority() as i8));

    assert_eq!(sorted[0], CredentialSource::OAuthStore);
    assert_eq!(sorted[sorted.len() - 1], CredentialSource::ConfigInline);
}

#[test]
fn test_credential_source_cloud_native() {
    assert!(CredentialSource::AwsCredentialChain.is_cloud_native());
    assert!(CredentialSource::GcpServiceAccount.is_cloud_native());
    assert!(CredentialSource::AzureIdentity.is_cloud_native());
    assert!(!CredentialSource::EnvVar.is_cloud_native());
    assert!(!CredentialSource::ConfigInline.is_cloud_native());
}

#[test]
fn test_composite_credential_resolver_new() {
    let resolver = CompositeCredentialResolver::new();
    assert!(resolver
        .resolve("test", &CredentialSource::EnvVar)
        .is_none());
}

#[test]
fn test_composite_resolver_with_inline_credentials() {
    let mut inline = std::collections::HashMap::new();
    inline.insert("test-provider".to_string(), "test-key".to_string());
    let resolver = CompositeCredentialResolver::new().with_inline(inline);

    let credential = resolver.resolve("test-provider", &CredentialSource::ConfigInline);
    assert!(credential.is_some());
    assert_eq!(credential.unwrap().value, "test-key");
}

#[test]
fn test_composite_resolver_resolve_with_fallback() {
    let mut inline = std::collections::HashMap::new();
    inline.insert("openai".to_string(), "inline-key".to_string());
    let resolver = CompositeCredentialResolver::new().with_inline(inline);

    let sources = vec![CredentialSource::EnvVar, CredentialSource::ConfigInline];

    let credential = resolver.resolve_with_fallback("openai", &sources);
    assert!(credential.is_some());
    assert_eq!(credential.unwrap().value, "inline-key");
}

#[test]
fn test_composite_resolver_fallback_order() {
    let resolver = CompositeCredentialResolver::new();
    let sources = vec![
        CredentialSource::OAuthStore,
        CredentialSource::EnvVar,
        CredentialSource::ConfigInline,
    ];

    let credential = resolver.resolve_with_fallback("nonexistent", &sources);
    assert!(credential.is_none());
}

#[test]
fn test_credential_source_priority_values() {
    assert_eq!(CredentialSource::OAuthStore.priority(), 5);
    assert_eq!(CredentialSource::AwsCredentialChain.priority(), 4);
    assert_eq!(CredentialSource::SystemKeychain.priority(), 3);
    assert_eq!(CredentialSource::AuthFile.priority(), 2);
    assert_eq!(CredentialSource::EnvVar.priority(), 1);
    assert_eq!(CredentialSource::ConfigInline.priority(), 0);
}

#[test]
fn test_resolved_credential_metadata() {
    use opencode_llm::auth_layered::layer1_credential_source::ResolvedCredential;
    use std::collections::HashMap;

    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "env".to_string());

    let cred = ResolvedCredential {
        provider: "openai".to_string(),
        value: "key".to_string(),
        source: CredentialSource::EnvVar,
        metadata,
    };

    assert_eq!(cred.provider, "openai");
    assert_eq!(cred.value, "key");
    assert_eq!(cred.source, CredentialSource::EnvVar);
    assert_eq!(cred.metadata.get("source").unwrap(), "env");
}
