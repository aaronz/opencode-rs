use opencode_core::OpenCodeError;
use opencode_storage::{StorageService, models::AccountModel};
use crate::password::{hash_password, verify_password};
use crate::jwt::{create_token, validate_token};
use chrono::Utc;
use uuid::Uuid;

pub struct AuthManager {
    storage: StorageService,
    jwt_secret: String,
}

impl AuthManager {
    pub fn new(storage: StorageService, jwt_secret: String) -> Self {
        Self { storage, jwt_secret }
    }

    pub async fn register(&self, username: &str, email: Option<&str>, password: &str) -> Result<AccountModel, OpenCodeError> {
        let password_hash = hash_password(password)?;
        let account = AccountModel {
            id: Uuid::new_v4().to_string(),
            username: username.to_string(),
            email: email.map(|s| s.to_string()),
            password_hash,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
            is_active: true,
            data: None,
        };

        self.storage.save_account(&account).await?;
        Ok(account)
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<String, OpenCodeError> {
        // This is a bit inefficient, should have load_account_by_username
        // For now let's assume we can find it in list or add the method to storage
        let accounts = self.storage.list_accounts(100, 0).await?;
        let account = accounts.into_iter().find(|a| a.username == username)
            .ok_or_else(|| OpenCodeError::Storage("User not found".to_string()))?;

        if verify_password(password, &account.password_hash)? {
            create_token(&account.id, &self.jwt_secret)
        } else {
            Err(OpenCodeError::Storage("Invalid password".to_string()))
        }
    }

    pub fn verify_token(&self, token: &str) -> Result<String, OpenCodeError> {
        let claims = validate_token(token, &self.jwt_secret)?;
        Ok(claims.sub)
    }
}
