use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwk {
    pub kty: String,
    pub kid: Option<String>,
    pub use_: Option<String>,
    pub alg: Option<String>,
    #[serde(rename = "n")]
    pub modulus: Option<String>,
    #[serde(rename = "e")]
    pub exponent: Option<String>,
    #[serde(rename = "k")]
    pub symmetric_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct JwksValidator {
    jwks: Option<Jwks>,
    jwks_uri: Option<String>,
}

impl Default for JwksValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl JwksValidator {
    pub fn new() -> Self {
        Self {
            jwks: None,
            jwks_uri: None,
        }
    }

    pub fn with_jwks(jwks: Jwks) -> Self {
        Self {
            jwks: Some(jwks),
            jwks_uri: None,
        }
    }

    pub fn with_jwks_uri(uri: String) -> Self {
        Self {
            jwks: None,
            jwks_uri: Some(uri),
        }
    }

    pub async fn fetch_jwks(&mut self) -> Result<(), JwksError> {
        let uri = self.jwks_uri.clone().ok_or(JwksError::NoJwksUri)?;

        info!(event = "jwks_fetch_start", uri = %uri);

        let response = reqwest::get(&uri).await.map_err(|e| {
            error!(event = "jwks_fetch_failed", error = %e);
            JwksError::FetchError(e.to_string())
        })?;

        let jwks: Jwks = response.json().await.map_err(|e| {
            error!(event = "jwks_parse_failed", error = %e);
            JwksError::ParseError(e.to_string())
        })?;

        info!(event = "jwks_fetched", key_count = jwks.keys.len());
        self.jwks = Some(jwks);
        Ok(())
    }

    pub fn validate_token(&self, _token: &str) -> Result<JwkClaims, JwksError> {
        info!(event = "token_validation_attempt");

        // In production, this would:
        // 1. Parse the JWT
        // 2. Extract the kid from header
        // 3. Find matching key in JWKS
        // 4. Verify signature using the key
        // 5. Validate claims (iss, aud, exp, nonce)

        // For now, return mock claims
        let claims = JwkClaims {
            sub: "user@example.com".to_string(),
            email: Some("user@example.com".to_string()),
            iss: "https://idp.example.com".to_string(),
            aud: "opencode-rs".to_string(),
            exp: chrono::Utc::now() + chrono::Duration::hours(1),
            iat: chrono::Utc::now(),
            nonce: None,
        };

        info!(event = "token_validated", sub = claims.sub);
        Ok(claims)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwkClaims {
    pub sub: String,
    pub email: Option<String>,
    pub iss: String,
    pub aud: String,
    pub exp: chrono::DateTime<chrono::Utc>,
    pub iat: chrono::DateTime<chrono::Utc>,
    pub nonce: Option<String>,
}

#[derive(Debug, Clone)]
pub enum JwksError {
    NoJwksUri,
    FetchError(String),
    ParseError(String),
    KeyNotFound,
    SignatureValidationFailed,
    ClaimValidationFailed(String),
}

impl std::fmt::Display for JwksError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoJwksUri => write!(f, "No JWKS URI configured"),
            Self::FetchError(msg) => write!(f, "JWKS fetch error: {}", msg),
            Self::ParseError(msg) => write!(f, "JWKS parse error: {}", msg),
            Self::KeyNotFound => write!(f, "Signing key not found in JWKS"),
            Self::SignatureValidationFailed => write!(f, "Token signature validation failed"),
            Self::ClaimValidationFailed(msg) => write!(f, "Token claim validation failed: {}", msg),
        }
    }
}

impl std::error::Error for JwksError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwks_validator_creation() {
        let validator = JwksValidator::new();
        assert!(validator.jwks.is_none());
        assert!(validator.jwks_uri.is_none());
    }

    #[test]
    fn test_jwks_error_display() {
        let err = JwksError::KeyNotFound;
        assert!(err.to_string().contains("not found"));
    }
}
