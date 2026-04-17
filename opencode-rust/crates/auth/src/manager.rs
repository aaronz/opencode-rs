use crate::jwt::{create_token, validate_token};
use crate::password::{hash_password, verify_password};
use chrono::Utc;
use opencode_core::OpenCodeError;
use opencode_storage::{models::AccountModel, StorageService};
use uuid::Uuid;

pub struct AuthManager {
    storage: StorageService,
    jwt_secret: String,
}

impl AuthManager {
    pub fn new(storage: StorageService, jwt_secret: String) -> Self {
        Self {
            storage,
            jwt_secret,
        }
    }

    pub async fn register(
        &self,
        username: &str,
        email: Option<&str>,
        password: &str,
    ) -> Result<AccountModel, OpenCodeError> {
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
        let accounts = self.storage.list_accounts(100, 0).await?;
        let account = accounts
            .into_iter()
            .find(|a| a.username == username)
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

#[cfg(test)]
mod tests {
    use super::*;
    use opencode_storage::database::StoragePool;
    use opencode_storage::memory_repository::InMemorySessionRepository;
    use opencode_storage::migration::MigrationManager;
    use std::sync::Arc;

    struct TestEnv {
        _temp_dir: tempfile::TempDir,
        auth_manager: AuthManager,
    }

    async fn create_test_env() -> TestEnv {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();

        let manager = MigrationManager::new(pool.clone(), 2);
        manager.migrate().await.unwrap();

        let session_repo = Arc::new(InMemorySessionRepository::new());
        let project_repo =
            Arc::new(opencode_storage::memory_repository::InMemoryProjectRepository::new());

        let storage = StorageService::new(session_repo, project_repo, pool);

        let auth_manager =
            AuthManager::new(storage, "test-secret-key-12345678901234567890".to_string());

        TestEnv {
            _temp_dir: temp_dir,
            auth_manager,
        }
    }

    #[tokio::test]
    async fn test_auth_manager_register() {
        let env = create_test_env().await;
        let result = env
            .auth_manager
            .register("testuser", Some("test@example.com"), "password123")
            .await;
        assert!(result.is_ok());

        let account = result.unwrap();
        assert_eq!(account.username, "testuser");
        assert_eq!(account.email, Some("test@example.com".to_string()));
    }

    #[tokio::test]
    async fn test_auth_manager_register_without_email() {
        let env = create_test_env().await;
        let result = env
            .auth_manager
            .register("testuser", None, "password123")
            .await;
        assert!(result.is_ok());

        let account = result.unwrap();
        assert_eq!(account.username, "testuser");
        assert_eq!(account.email, None);
    }

    #[tokio::test]
    async fn test_auth_manager_login() {
        let env = create_test_env().await;

        env.auth_manager
            .register("testuser", None, "password123")
            .await
            .unwrap();

        let result = env.auth_manager.login("testuser", "password123").await;
        assert!(result.is_ok());

        let token = result.unwrap();
        assert!(!token.is_empty());

        let user_id = env.auth_manager.verify_token(&token);
        assert!(user_id.is_ok());
    }

    #[tokio::test]
    async fn test_auth_manager_login_wrong_password() {
        let env = create_test_env().await;

        env.auth_manager
            .register("testuser", None, "password123")
            .await
            .unwrap();

        let result = env.auth_manager.login("testuser", "wrongpassword").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auth_manager_login_user_not_found() {
        let env = create_test_env().await;

        let result = env.auth_manager.login("nonexistent", "password123").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auth_manager_verify_token_invalid() {
        let env = create_test_env().await;

        let result = env.auth_manager.verify_token("invalid-token");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auth_manager_verify_token_wrong_secret() {
        let env = create_test_env().await;

        let token = create_token("user-123", "different-secret").unwrap();

        let result = env.auth_manager.verify_token(&token);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auth_manager_login_and_verify_token() {
        let env = create_test_env().await;

        env.auth_manager
            .register("testuser", None, "password123")
            .await
            .unwrap();

        let token = env
            .auth_manager
            .login("testuser", "password123")
            .await
            .unwrap();
        let user_id = env.auth_manager.verify_token(&token).unwrap();

        let accounts = env
            .auth_manager
            .storage
            .list_accounts(100, 0)
            .await
            .unwrap();
        let account = accounts.iter().find(|a| a.username == "testuser").unwrap();
        assert_eq!(user_id, account.id);
    }
}
