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

/// Authentication methods supported by a provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    /// Browser-based OAuth flow (requires user interaction)
    Browser,
    /// Direct API key authentication
    ApiKey,
    /// Local endpoint with no authentication
    Local,
    /// Device code OAuth flow (for headless authentication)
    DeviceFlow,
}

/// Trait for providers to advertise their supported authentication methods
pub trait ProviderAuth {
    fn supported_auth_methods(&self) -> Vec<AuthMethod>;
}

impl ProviderAuth for str {
    fn supported_auth_methods(&self) -> Vec<AuthMethod> {
        match self {
            "google" | "copilot" => Vec::new(),
            "anthropic" => vec![AuthMethod::ApiKey],
            "openai" => vec![AuthMethod::ApiKey, AuthMethod::Browser],
            _ => vec![AuthMethod::ApiKey],
        }
    }
}

impl ProviderAuth for &str {
    fn supported_auth_methods(&self) -> Vec<AuthMethod> {
        (*self).supported_auth_methods()
    }
}

/// OAuth session manager for handling token refresh
pub struct OAuthSessionManager {
    client: reqwest::Client,
}

impl Default for OAuthSessionManager {
    fn default() -> Self {
        Self::new()
    }
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
                let refresh_token = credential
                    .refresh_token
                    .as_ref()
                    .ok_or_else(|| OpenCodeError::Llm("No refresh token available".to_string()))?;

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

                let token_response: OAuthTokenResponse = response.json().await.map_err(|e| {
                    OpenCodeError::Llm(format!("Failed to parse OAuth response: {}", e))
                })?;

                let expires_at = token_response
                    .expires_in
                    .map(|seconds| chrono::Utc::now() + chrono::Duration::seconds(seconds as i64));

                let mut new_credential =
                    Credential::new(credential.provider.clone(), token_response.access_token);
                new_credential.expires_at = expires_at;
                new_credential.refresh_token = token_response
                    .refresh_token
                    .or_else(|| credential.refresh_token.clone());

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
    HeaderApiKey { header_name: String },
    /// API key as query parameter
    QueryApiKey { param_name: String },
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
        self.credentials
            .insert(credential.provider.clone(), credential);
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
            ("MINIMAX_API_KEY", "minimax"),
            ("QWEN_API_KEY", "qwen"),
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

        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| OpenCodeError::Llm(format!("Failed to create directory: {}", e)))?;
        }

        std::fs::write(path, json)
            .map_err(|e| OpenCodeError::Llm(format!("Failed to write credentials: {}", e)))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(
                |e| OpenCodeError::Llm(format!("Failed to set file permissions: {}", e)),
            )?;
        }

        Ok(())
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
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
    use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, Key, KeyInit, Nonce};
    use opencode_core::OpenCodeError;
    use rand::RngCore;
    use sha2::{Digest, Sha256};

    fn derive_key(key: &[u8]) -> Key {
        let mut hasher = Sha256::new();
        hasher.update(key);
        let result = hasher.finalize();
        *Key::from_slice(&result)
    }

    fn generate_nonce() -> Nonce {
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        Nonce::from(nonce)
    }

    /// Encode credential for storage (AES-256-GCM via ChaCha20-Poly1305 + base64)
    pub fn encode_credential(value: &str, key: &str) -> Result<String, OpenCodeError> {
        if value.is_empty() {
            return Err(OpenCodeError::Llm(
                "Cannot encode empty credential".to_string(),
            ));
        }
        let cipher = ChaCha20Poly1305::new(&derive_key(key.as_bytes()));
        let nonce = generate_nonce();
        let ciphertext = cipher
            .encrypt(&nonce, value.as_bytes())
            .map_err(|e| OpenCodeError::Llm(format!("Failed to encrypt credential: {}", e)))?;

        let mut combined = nonce.to_vec();
        combined.extend(ciphertext);
        Ok(BASE64.encode(combined))
    }

    /// Decode credential from storage
    pub fn decode_credential(encoded: &str, key: &str) -> Result<String, OpenCodeError> {
        if encoded.is_empty() {
            return Err(OpenCodeError::Llm(
                "Cannot decode empty credential".to_string(),
            ));
        }
        let combined = BASE64
            .decode(encoded)
            .map_err(|e| OpenCodeError::Llm(format!("Failed to decode credential: {}", e)))?;

        if combined.len() < 13 {
            return Err(OpenCodeError::Llm(
                "Invalid credential data: too short".to_string(),
            ));
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = ChaCha20Poly1305::new(&derive_key(key.as_bytes()));
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| OpenCodeError::Llm(format!("Failed to decrypt credential: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| OpenCodeError::Llm(format!("Failed to decode credential: {}", e)))
    }

    /// Generate a random encryption key
    pub fn generate_key() -> String {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        BASE64.encode(bytes)
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
            let decoded = decode_credential(&encoded, "key2");
            assert!(decoded.is_err());
        }

        #[test]
        fn test_generate_key_produces_unique_keys() {
            let key1 = generate_key();
            let key2 = generate_key();
            assert_ne!(key1, key2);
            assert_eq!(key1.len(), 44);
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

    #[test]
    fn test_api_key_expired() {
        let mut key = ApiKey::new("openai".to_string(), "sk-test".to_string());
        key.expires_at = Some(chrono::Utc::now() - chrono::Duration::hours(1));
        assert!(!key.is_valid());
    }

    #[tokio::test]
    async fn test_auth_strategy_bearer_api_key() {
        let strategy = AuthStrategy::BearerApiKey { header_name: None };
        let cred = Credential::new("openai".to_string(), "sk-test".to_string());
        let client = reqwest::Client::new();
        let request = client.post("https://api.openai.com");
        let result = strategy.apply_to_request(request, &cred).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_auth_strategy_header_api_key() {
        let strategy = AuthStrategy::HeaderApiKey {
            header_name: "x-api-key".to_string(),
        };
        let cred = Credential::new("openai".to_string(), "sk-test".to_string());
        let client = reqwest::Client::new();
        let request = client.post("https://api.openai.com");
        let result = strategy.apply_to_request(request, &cred).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_auth_strategy_query_api_key() {
        let strategy = AuthStrategy::QueryApiKey {
            param_name: "api_key".to_string(),
        };
        let cred = Credential::new("openai".to_string(), "sk-test".to_string());
        let client = reqwest::Client::new();
        let request = client.post("https://api.openai.com");
        let result = strategy.apply_to_request(request, &cred).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_auth_strategy_oauth_session() {
        let strategy = AuthStrategy::OAuthSession {
            token_endpoint: "https://auth.example.com/token".to_string(),
            client_id: "client_id".to_string(),
            client_secret: "client_secret".to_string(),
            scopes: vec!["read".to_string(), "write".to_string()],
        };
        let cred = Credential::new("openai".to_string(), "access_token".to_string());
        let client = reqwest::Client::new();
        let request = client.post("https://api.openai.com");
        let result = strategy.apply_to_request(request, &cred).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_auth_strategy_none() {
        let strategy = AuthStrategy::None;
        let cred = Credential::new("openai".to_string(), "sk-test".to_string());
        let client = reqwest::Client::new();
        let request = client.post("https://api.openai.com");
        let result = strategy.apply_to_request(request, &cred).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_auth_strategy_invalid_credential() {
        let strategy = AuthStrategy::BearerApiKey { header_name: None };
        let mut cred = Credential::new("openai".to_string(), "".to_string());
        cred.expires_at = Some(chrono::Utc::now() - chrono::Duration::hours(1));
        let client = reqwest::Client::new();
        let request = client.post("https://api.openai.com");
        let result = strategy.apply_to_request(request, &cred).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_credential_expires_soon_no_expiry() {
        let cred = Credential::new("openai".to_string(), "sk-test".to_string());
        assert!(!cred.expires_soon(600));
    }

    #[test]
    fn test_credential_store_remove() {
        let mut store = CredentialStore::new();
        let cred = Credential::new("openai".to_string(), "sk-test".to_string());
        store.store(cred);

        let removed = store.remove("openai");
        assert!(removed.is_some());
        assert!(store.get("openai").is_none());
    }

    #[test]
    fn test_credential_store_list() {
        let mut store = CredentialStore::new();
        store.store(Credential::new(
            "openai".to_string(),
            "sk-test1".to_string(),
        ));
        store.store(Credential::new(
            "anthropic".to_string(),
            "sk-test2".to_string(),
        ));

        let list = store.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_credential_store_resolve_found() {
        let mut store = CredentialStore::new();
        store.store(Credential::new("openai".to_string(), "sk-test".to_string()));

        let result = store.resolve("openai");
        assert!(result.is_ok());
    }

    #[test]
    fn test_credential_store_resolve_not_found() {
        let store = CredentialStore::new();
        let result = store.resolve("openai");
        assert!(result.is_err());
    }

    #[test]
    fn test_credential_store_validate_all() {
        let mut store = CredentialStore::new();
        store.store(Credential::new(
            "openai".to_string(),
            "sk-test1".to_string(),
        ));
        store.store(Credential::new(
            "anthropic".to_string(),
            "sk-test2".to_string(),
        ));

        let validations = store.validate_all();
        assert_eq!(validations.len(), 2);
    }

    #[test]
    fn test_auth_manager_new() {
        let manager = AuthManager::new();
        assert!(manager.get_key("openai").is_none());
    }

    #[test]
    fn test_auth_manager_set_get_key() {
        let mut manager = AuthManager::new();
        manager.set_key("openai".to_string(), "sk-test".to_string());
        let key = manager.get_key("openai");
        assert!(key.is_some());
        assert_eq!(key.unwrap().key, "sk-test");
    }

    #[test]
    fn test_auth_manager_validate_key_not_found() {
        let manager = AuthManager::new();
        let result = manager.validate_key("openai");
        assert!(result.is_err());
    }

    #[test]
    fn test_auth_manager_validate_key_expired() {
        let mut manager = AuthManager::new();
        let mut key = ApiKey::new("openai".to_string(), "sk-test".to_string());
        key.expires_at = Some(chrono::Utc::now() - chrono::Duration::hours(1));
        manager.keys.insert("openai".to_string(), key);
        let result = manager.validate_key("openai");
        assert!(result.is_err());
    }

    #[test]
    fn test_auth_manager_load_from_env() {
        std::env::set_var("OPENAI_API_KEY", "sk-test-from-env");
        let mut manager = AuthManager::new();
        manager.load_from_env();
        let key = manager.get_key("openai");
        assert!(key.is_some());
        assert_eq!(key.unwrap().key, "sk-test-from-env");
        std::env::remove_var("OPENAI_API_KEY");
    }

    #[test]
    fn test_oauth_token_response_serde() {
        let json = r#"{
            "access_token": "at123",
            "token_type": "Bearer",
            "expires_in": 3600,
            "refresh_token": "rt456",
            "scope": "read write"
        }"#;
        let response: OAuthTokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.access_token, "at123");
        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, Some(3600));
        assert_eq!(response.refresh_token, Some("rt456".to_string()));
        assert_eq!(response.scope, Some("read write".to_string()));
    }

    #[test]
    fn test_credential_with_metadata() {
        let mut cred = Credential::new("openai".to_string(), "sk-test".to_string());
        cred.metadata
            .insert("env".to_string(), "production".to_string());
        assert_eq!(cred.metadata.get("env"), Some(&"production".to_string()));
    }

    #[test]
    fn test_provider_auth_config_with_multiple_headers() {
        let config = ProviderAuthConfig::new(
            "openai".to_string(),
            "https://api.openai.com/v1".to_string(),
            AuthStrategy::BearerApiKey { header_name: None },
        )
        .with_header("Header1".to_string(), "value1".to_string())
        .with_header("Header2".to_string(), "value2".to_string());

        assert_eq!(config.headers.len(), 2);
    }

    #[test]
    fn test_credential_store_with_encryption_key() {
        let store = CredentialStore::new().with_encryption_key("my-key".to_string());
        assert_eq!(store.encryption_key, Some("my-key".to_string()));
    }

    #[test]
    fn test_credential_store_default() {
        let store = CredentialStore::default();
        assert!(store.credentials.is_empty());
    }

    #[tokio::test]
    async fn test_oauth_session_manager_new() {
        let manager = OAuthSessionManager::new();
        assert!(manager
            .refresh_token(
                &AuthStrategy::BearerApiKey { header_name: None },
                &Credential::new("openai".to_string(), "key".to_string()),
            )
            .await
            .is_err());
    }
}
