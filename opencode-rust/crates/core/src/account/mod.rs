mod types;
pub(crate) use types::{Account, AccountManager};

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
