use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use opencode_core::OpenCodeError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn create_token(user_id: &str, secret: &str) -> Result<String, OpenCodeError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .unwrap_or_else(|| panic!("invalid timestamp calculation"))
        .timestamp();

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration as usize,
        iat: Utc::now().timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| OpenCodeError::Storage(e.to_string()))
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, OpenCodeError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
    .map_err(|e| OpenCodeError::Storage(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_validate_token() {
        let user_id = "user-123";
        let secret = "test-secret-key-12345678901234567890";

        let token = create_token(user_id, secret).unwrap();
        assert!(!token.is_empty());

        let claims = validate_token(&token, secret).unwrap();
        assert_eq!(claims.sub, user_id);
    }

    #[test]
    fn test_validate_token_invalid_secret() {
        let user_id = "user-123";
        let secret = "correct-secret-key-123456789012345";
        let wrong_secret = "wrong-secret-key-123456789012345";

        let token = create_token(user_id, secret).unwrap();
        let result = validate_token(&token, wrong_secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_token_malformed_token() {
        let secret = "test-secret-key-12345678901234567890";
        let result = validate_token("not.a.valid.token", secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_token_empty() {
        let secret = "test-secret-key-12345678901234567890";
        let result = validate_token("", secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_struct() {
        let claims = Claims {
            sub: "test-user".to_string(),
            exp: 1234567890,
            iat: 1234500000,
        };
        assert_eq!(claims.sub, "test-user");
        assert_eq!(claims.exp, 1234567890);
        assert_eq!(claims.iat, 1234500000);
    }
}
