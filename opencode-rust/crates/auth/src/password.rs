use bcrypt::{hash, verify, DEFAULT_COST};
use opencode_core::OpenCodeError;

pub fn hash_password(password: &str) -> Result<String, OpenCodeError> {
    hash(password, DEFAULT_COST).map_err(|e| OpenCodeError::Storage(e.to_string()))
}

pub fn verify_password(password: &str, hashed: &str) -> Result<bool, OpenCodeError> {
    verify(password, hashed).map_err(|e| OpenCodeError::Storage(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_success() {
        let password = "secure-password-123";
        let result = hash_password(password);
        assert!(result.is_ok());
        let hashed = result.unwrap();
        assert!(!hashed.is_empty());
        assert_ne!(hashed, password);
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "secure-password-123";
        let hashed = hash_password(password).unwrap();
        let result = verify_password(password, &hashed);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "secure-password-123";
        let wrong_password = "wrong-password";
        let hashed = hash_password(password).unwrap();
        let result = verify_password(wrong_password, &hashed);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_verify_password_invalid_hash() {
        let result = verify_password("password", "not-a-valid-hash");
        assert!(result.is_err());
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let password1 = "password-one";
        let password2 = "password-two";
        let hash1 = hash_password(password1).unwrap();
        let hash2 = hash_password(password2).unwrap();
        assert_ne!(hash1, hash2);
    }
}
