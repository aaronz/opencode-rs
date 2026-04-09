use bcrypt::{hash, verify, DEFAULT_COST};
use opencode_core::OpenCodeError;

pub fn hash_password(password: &str) -> Result<String, OpenCodeError> {
    hash(password, DEFAULT_COST).map_err(|e| OpenCodeError::Storage(e.to_string()))
}

pub fn verify_password(password: &str, hashed: &str) -> Result<bool, OpenCodeError> {
    verify(password, hashed).map_err(|e| OpenCodeError::Storage(e.to_string()))
}
