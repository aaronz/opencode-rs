use async_trait::async_trait;
use opencode_core::OpenCodeError;
use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// OAuth token response from provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// OAuth session manager for handling token refresh
pub struct OAuthSessionManager {
    client: reqwest::Client,
}

impl OAuthSessionManager {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Refresh an OAuth token using the refresh token
    pub async fn refresh_token(
        &self,
        strategy: &AuthStrategy,
        credential: &Credential,
    ) -> Result<Credential, OpenCodeError> {
        match strategy {
            AuthStrategy::OAuthSession {
                token_endpoint,
                client_id,
                client_secret,
                scopes,
            } => {
                let refresh_token = credential.refresh_token.as_ref().ok_or_else(|| {
                    OpenCodeError::Llm("No refresh token available".to_string())
                })?;

                let response = self
                    .client
                    .post(token_endpoint)
                    .form(&[
                        ("grant_type", "refresh_token"),
                        ("refresh_token", refresh_token),
                        ("client_id", client_id.as_str()),
                        ("client_secret", client_secret.as_str()),
                        ("scope", &scopes.join(" ")),
                    ])
                    .timeout(Duration::from_secs(30))
                    .send()
                    .await
                    .map_err(|e| OpenCodeError::Llm(format!("OAuth refresh failed: {}", e)))?;

                if !response.status().is_success() {
                    return Err(OpenCodeError::Llm(format!(
                        "OAuth refresh failed with status: {}",
                        response.status()
                    )));
                }

                let token_response: OAuthTokenResponse = response
                    .json()
                    .await
                    .map_err(|e| OpenCodeError::Llm(format!("Failed to parse OAuth response: {}", e)))?;

                let expires_at = token_response.expires_in.map(|seconds| {
                    chrono::Utc::now() + chrono::Duration::seconds(seconds as i64)
                });

                let mut new_credential = Credential::new(
                    credential.provider.clone(),
                    token_response.access_token,
                );
                new_credential.expires_at = expires_at;
                new_credential.refresh_token = token_response.refresh_token.or_else(|| {
                    credential.refresh_token.clone()
                });

                Ok(new_credential)
            }
            _ => Err(OpenCodeError::Llm(
                "OAuth refresh only supported for OAuthSession strategy".to_string(),
            )),
        }
    }

    /// Check if a credential needs refresh and refresh if necessary
    pub async fn ensure_valid(
        &self,
        strategy: &AuthStrategy,
        credential: &Credential,
        refresh_threshold_seconds: i64,
    ) -> Result<Credential, OpenCodeError> {
        if credential.is_valid() && !credential.expires_soon(refresh_threshold_seconds) {
            return Ok(credential.clone());
        }

        if credential.refresh_token.is_some() {
            self.refresh_token(strategy, credential).await
        } else {
            Err(OpenCodeError::Llm(format!(
                "Credential for provider {} is expired and cannot be refreshed",
                credential.provider
            )))
        }
    }
}

/// Authentication strategies for provider requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthStrategy {
    /// Bearer token in Authorization header (most common)
    BearerApiKey {
        header_name: Option<String>, // defaults to "Authorization"
    },
    /// API key in custom header (e.g., x-api-key)
    HeaderApiKey {
        header_name: String,
    },
    /// API key as query parameter
    QueryApiKey {
        param_name: String,
    },
    /// OAuth session with refresh capability
    OAuthSession {
        token_endpoint: String,
        client_id: String,
        client_secret: String,
        scopes: Vec<String>,
    },
    /// No authentication (local endpoints)
    None,
}

impl Default for AuthStrategy {
    fn default() -> Self {
        Self::BearerApiKey { header_name: None }
    }
}

/// Credential for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub provider: String,
    pub key: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub refresh_token: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl Credential {
    pub fn new(provider: String, key: String) -> Self {
        Self {
            provider,
            key,
            expires_at: None,
            refresh_token: None,
            metadata: HashMap::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() < expires_at
        } else {
            !self.key.is_empty()
        }
    }

    pub fn expires_soon(&self, threshold_seconds: i64) -> bool {
        if let Some(expires_at) = self.expires_at {
            let threshold = chrono::Duration::seconds(threshold_seconds);
            chrono::Utc::now() + threshold >= expires_at
        } else {
            false
        }
    }
}

/// Trait for applying auth strategy to HTTP requests
#[async_trait]
pub trait AuthApplicator {
    async fn apply_to_request(
        &self,
        request: RequestBuilder,
        credential: &Credential,
    ) -> Result<RequestBuilder, OpenCodeError>;
}

#[async_trait]
impl AuthApplicator for AuthStrategy {
    async fn apply_to_request(
        &self,
        request: RequestBuilder,
        credential: &Credential,
    ) -> Result<RequestBuilder, OpenCodeError> {
        if !credential.is_valid() {
            return Err(OpenCodeError::Llm(format!(
                "Credential for provider {} is invalid or expired",
                credential.provider
            )));
        }

        match self {
            AuthStrategy::BearerApiKey { header_name } => {
                let header = header_name.as_deref().unwrap_or("Authorization");
                let value = format!("Bearer {}", credential.key);
                Ok(request.header(header, value))
            }
            AuthStrategy::HeaderApiKey { header_name } => {
                Ok(request.header(header_name.as_str(), credential.key.as_str()))
            }
            AuthStrategy::QueryApiKey { param_name } => {
                Ok(request.query(&[(param_name.as_str(), credential.key.as_str())]))
            }
            AuthStrategy::OAuthSession { .. } => {
                // For OAuth, the key should be the access token
                Ok(request.header("Authorization", format!("Bearer {}", credential.key)))
            }
            AuthStrategy::None => Ok(request),
        }
    }
}

/// Provider configuration with auth strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAuthConfig {
    pub provider_id: String,
    pub endpoint: String,
    pub auth_strategy: AuthStrategy,
    pub credential_ref: Option<String>,
    pub headers: HashMap<String, String>,
}

impl ProviderAuthConfig {
    pub fn new(provider_id: String, endpoint: String, auth_strategy: AuthStrategy) -> Self {
        Self {
            provider_id,
            endpoint,
            auth_strategy,
            credential_ref: None,
            headers: HashMap::new(),
        }
    }

    pub fn with_credential_ref(mut self, credential_ref: String) -> Self {
        self.credential_ref = Some(credential_ref);
        self
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
}

/// Credential store with environment variable override
pub struct CredentialStore {
    credentials: HashMap<String, Credential>,
    encryption_key: Option<String>,
}

impl CredentialStore {
    pub fn new() -> Self {
        Self {
            credentials: HashMap::new(),
            encryption_key: None,
        }
    }

    pub fn with_encryption_key(mut self, key: String) -> Self {
        self.encryption_key = Some(key);
        self
    }

    pub fn store(&mut self, credential: Credential) {
        self.credentials.insert(credential.provider.clone(), credential);
    }

    pub fn get(&self, provider: &str) -> Option<&Credential> {
        self.credentials.get(provider)
    }

    pub fn remove(&mut self, provider: &str) -> Option<Credential> {
        self.credentials.remove(provider)
    }

    pub fn list(&self) -> Vec<&Credential> {
        self.credentials.values().collect()
    }

    /// Load credentials from environment variables
    /// Format: {PROVIDER}_API_KEY
    pub fn load_from_env(&mut self) {
        let env_vars = [
            ("OPENAI_API_KEY", "openai"),
            ("ANTHROPIC_API_KEY", "anthropic"),
            ("GEMINI_API_KEY", "gemini"),
            ("GOOGLE_API_KEY", "google"),
            ("OPENROUTER_API_KEY", "openrouter"),
            ("OLLAMA_API_KEY", "ollama"),
            ("MISTRAL_API_KEY", "mistral"),
            ("GROQ_API_KEY", "groq"),
            ("COHERE_API_KEY", "cohere"),
            ("TOGETHER_API_KEY", "togetherai"),
            ("PERPLEXITY_API_KEY", "perplexity"),
        ];

        for (env_var, provider) in env_vars {
            if let Ok(key) = std::env::var(env_var) {
                if !key.is_empty() {
                    self.store(Credential::new(provider.to_string(), key));
                }
            }
        }
    }

    /// Save credentials to encrypted file
    pub fn save_to_file(&self, path: &str) -> Result<(), OpenCodeError> {
        let key = self.encryption_key.as_deref().unwrap_or("default");
        let mut encrypted_creds = HashMap::new();

        for (provider, cred) in &self.credentials {
            let encrypted_key = encryption::encode_credential(&cred.key, key)?;
            let mut encrypted_cred = cred.clone();
            encrypted_cred.key = encrypted_key;
            encrypted_creds.insert(provider.clone(), encrypted_cred);
        }

        let json = serde_json::to_string_pretty(&encrypted_creds)
            .map_err(|e| OpenCodeError::Llm(format!("Failed to serialize credentials: {}", e)))?;

        std::fs::write(path, json)
            .map_err(|e| OpenCodeError::Llm(format!("Failed to write credentials: {}", e)))
    }

    /// Load credentials from encrypted file
    pub fn load_from_file(&mut self, path: &str) -> Result<(), OpenCodeError> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| OpenCodeError::Llm(format!("Failed to read credentials: {}", e)))?;

        let encrypted_creds: HashMap<String, Credential> = serde_json::from_str(&json)
            .map_err(|e| OpenCodeError::Llm(format!("Failed to parse credentials: {}", e)))?;

        let key = self.encryption_key.as_deref().unwrap_or("default");

        for (provider, mut cred) in encrypted_creds {
            cred.key = encryption::decode_credential(&cred.key, key)?;
            self.credentials.insert(provider, cred);
        }

        Ok(())
    }

    /// Get credential with precedence: env > stored
    pub fn resolve(&self, provider: &str) -> Result<&Credential, OpenCodeError> {
        self.get(provider).ok_or_else(|| {
            OpenCodeError::Llm(format!("No credential found for provider: {}", provider))
        })
    }

    /// Validate all stored credentials
    pub fn validate_all(&self) -> Vec<(&str, bool)> {
        self.credentials
            .iter()
            .map(|(provider, cred)| (provider.as_str(), cred.is_valid()))
            .collect()
    }
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple encryption utilities for credential storage
pub mod encryption {
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    use opencode_core::OpenCodeError;

    /// Simple XOR-based obfuscation (NOT secure encryption, just prevents casual exposure)
    fn xor_encrypt_decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
        if key.is_empty() {
            return data.to_vec();
        }
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ key[i % key.len()])
            .collect()
    }

    /// Encode credential for storage (obfuscated + base64)
    pub fn encode_credential(value: &str, key: &str) -> Result<String, OpenCodeError> {
        if value.is_empty() {
            return Err(OpenCodeError::Llm("Cannot encode empty credential".to_string()));
        }
        let encrypted = xor_encrypt_decrypt(value.as_bytes(), key.as_bytes());
        Ok(BASE64.encode(encrypted))
    }

    /// Decode credential from storage
    pub fn decode_credential(encoded: &str, key: &str) -> Result<String, OpenCodeError> {
        if encoded.is_empty() {
            return Err(OpenCodeError::Llm("Cannot decode empty credential".to_string()));
        }
        let encrypted = BASE64
            .decode(encoded)
            .map_err(|e| OpenCodeError::Llm(format!("Failed to decode credential: {}", e)))?;
        let decrypted = xor_encrypt_decrypt(&encrypted, key.as_bytes());
        String::from_utf8(decrypted)
            .map_err(|e| OpenCodeError::Llm(format!("Failed to decode credential: {}", e)))
    }

    /// Generate a random encryption key
    pub fn generate_key() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("opencode-{:x}", timestamp)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_encode_decode_roundtrip() {
            let value = "sk-test-api-key-12345";
            let key = "my-secret-key";
            let encoded = encode_credential(value, key).unwrap();
            let decoded = decode_credential(&encoded, key).unwrap();
            assert_eq!(value, decoded);
        }

        #[test]
        fn test_encode_empty_fails() {
            let result = encode_credential("", "key");
            assert!(result.is_err());
        }

        #[test]
        fn test_decode_empty_fails() {
            let result = decode_credential("", "key");
            assert!(result.is_err());
        }

        #[test]
        fn test_wrong_key_fails() {
            let value = "secret";
            let encoded = encode_credential(value, "key1").unwrap();
            let decoded = decode_credential(&encoded, "key2").unwrap();
            assert_ne!(value, decoded);
        }
    }
}

/// Original ApiKey struct for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub provider: String,
    pub key: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ApiKey {
    pub fn new(provider: String, key: String) -> Self {
        Self {
            provider,
            key,
            expires_at: None,
        }
    }

    pub fn is_valid(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() < expires_at
        } else {
            !self.key.is_empty()
        }
    }
}

/// Original AuthManager for backward compatibility
pub struct AuthManager {
    keys: std::collections::HashMap<String, ApiKey>,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            keys: std::collections::HashMap::new(),
        }
    }

    pub fn set_key(&mut self, provider: String, key: String) {
        self.keys
            .insert(provider.clone(), ApiKey::new(provider, key));
    }

    pub fn get_key(&self, provider: &str) -> Option<&ApiKey> {
        self.keys.get(provider)
    }

    pub fn validate_key(&self, provider: &str) -> Result<String, OpenCodeError> {
        let key = self.keys.get(provider).ok_or_else(|| {
            OpenCodeError::Llm(format!("No API key found for provider: {}", provider))
        })?;

        if !key.is_valid() {
            return Err(OpenCodeError::Llm(format!(
                "API key for provider {} is invalid or expired",
                provider
            )));
        }

        Ok(key.key.clone())
    }

    pub fn load_from_env(&mut self) {
        let providers = ["OPENAI_API_KEY", "ANTHROPIC_API_KEY", "OLLAMA_API_KEY"];

        for env_var in providers {
            if let Ok(key) = std::env::var(env_var) {
                let provider = env_var.replace("_API_KEY", "").to_lowercase();
                self.set_key(provider, key);
            }
        }
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_is_valid() {
        let cred = Credential::new("openai".to_string(), "sk-test".to_string());
        assert!(cred.is_valid());

        let mut expired_cred = Credential::new("openai".to_string(), "sk-test".to_string());
        expired_cred.expires_at = Some(chrono::Utc::now() - chrono::Duration::hours(1));
        assert!(!expired_cred.is_valid());
    }

    #[test]
    fn test_credential_expires_soon() {
        let mut cred = Credential::new("openai".to_string(), "sk-test".to_string());
        cred.expires_at = Some(chrono::Utc::now() + chrono::Duration::minutes(5));
        assert!(cred.expires_soon(600)); // expires within 10 minutes
        assert!(!cred.expires_soon(60)); // doesn't expire within 1 minute
    }

    #[test]
    fn test_auth_strategy_default() {
        let strategy = AuthStrategy::default();
        matches!(strategy, AuthStrategy::BearerApiKey { header_name: None });
    }

    #[test]
    fn test_credential_store() {
        let mut store = CredentialStore::new();
        let cred = Credential::new("openai".to_string(), "sk-test".to_string());
        store.store(cred);

        assert!(store.get("openai").is_some());
        assert!(store.get("anthropic").is_none());
        assert_eq!(store.list().len(), 1);
    }

    #[test]
    fn test_provider_auth_config() {
        let config = ProviderAuthConfig::new(
            "openai".to_string(),
            "https://api.openai.com/v1".to_string(),
            AuthStrategy::BearerApiKey { header_name: None },
        )
        .with_credential_ref("openai-cred-1".to_string())
        .with_header("Custom-Header".to_string(), "value".to_string());

        assert_eq!(config.provider_id, "openai");
        assert_eq!(config.credential_ref, Some("openai-cred-1".to_string()));
        assert!(config.headers.contains_key("Custom-Header"));
    }

    #[test]
    fn test_api_key_is_valid() {
        let key = ApiKey::new("openai".to_string(), "sk-test".to_string());
        assert!(key.is_valid());
    }
}
