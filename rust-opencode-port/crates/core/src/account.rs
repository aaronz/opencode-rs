use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub username: String,
    pub email: String,
    pub provider: String,
    pub api_key: Option<String>,
    pub metadata: HashMap<String, String>,
}

pub struct AccountManager {
    accounts: HashMap<String, Account>,
    current: Option<String>,
}

impl AccountManager {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            current: None,
        }
    }

    pub fn add(&mut self, account: Account) {
        let id = account.id.clone();
        self.accounts.insert(id.clone(), account);
        if self.current.is_none() {
            self.current = Some(id);
        }
    }

    pub fn get(&self, id: &str) -> Option<&Account> {
        self.accounts.get(id)
    }

    pub fn current(&self) -> Option<&Account> {
        self.current.as_ref().and_then(|id| self.accounts.get(id))
    }

    pub fn set_current(&mut self, id: &str) {
        if self.accounts.contains_key(id) {
            self.current = Some(id.to_string());
        }
    }

    pub fn list(&self) -> Vec<&Account> {
        self.accounts.values().collect()
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
    fn test_account_manager_new() {
        let am = AccountManager::new();
        assert!(am.list().is_empty());
        assert!(am.current().is_none());
    }

    #[test]
    fn test_account_manager_add() {
        let mut am = AccountManager::new();
        am.add(Account {
            id: "acc1".to_string(),
            username: "user1".to_string(),
            email: "user@test.com".to_string(),
            provider: "github".to_string(),
            api_key: None,
            metadata: std::collections::HashMap::new(),
        });

        assert_eq!(am.list().len(), 1);
        assert!(am.current().is_some());
    }

    #[test]
    fn test_account_manager_get() {
        let mut am = AccountManager::new();
        am.add(Account {
            id: "acc1".to_string(),
            username: "user1".to_string(),
            email: "user@test.com".to_string(),
            provider: "github".to_string(),
            api_key: None,
            metadata: std::collections::HashMap::new(),
        });

        assert!(am.get("acc1").is_some());
        assert!(am.get("nonexistent").is_none());
    }

    #[test]
    fn test_account_manager_set_current() {
        let mut am = AccountManager::new();
        am.add(Account {
            id: "acc1".to_string(),
            username: "user1".to_string(),
            email: "user@test.com".to_string(),
            provider: "github".to_string(),
            api_key: None,
            metadata: std::collections::HashMap::new(),
        });

        am.set_current("acc1");
        assert_eq!(am.current().unwrap().username, "user1");
    }
}
