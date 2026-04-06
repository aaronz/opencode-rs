use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub enterprise_id: Option<String>,
    pub role: AccountRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountRole {
    User,
    Admin,
    Owner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub account_id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub enterprise_id: String,
    pub members: Vec<TeamMember>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub user_id: String,
    pub role: TeamRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TeamRole {
    Member,
    Lead,
    Admin,
}

pub struct AccountManager;

impl AccountManager {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AccountManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let account = Account {
            id: "acc-123".to_string(),
            name: "Test Account".to_string(),
            email: "test@example.com".to_string(),
            created_at: chrono::Utc::now(),
            enterprise_id: None,
            role: AccountRole::User,
        };

        assert_eq!(account.name, "Test Account");
    }
}
