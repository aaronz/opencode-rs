use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareLink {
    pub id: String,
    pub session_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_public: bool,
}

pub struct ShareManager {
    links: Vec<ShareLink>,
}

impl ShareManager {
    pub fn new() -> Self {
        Self { links: Vec::new() }
    }

    pub fn create(&mut self, session_id: String, is_public: bool) -> ShareLink {
        let link = ShareLink {
            id: uuid::Uuid::new_v4().to_string(),
            session_id,
            created_at: chrono::Utc::now(),
            expires_at: None,
            is_public,
        };
        self.links.push(link.clone());
        link
    }

    pub fn get(&self, id: &str) -> Option<&ShareLink> {
        self.links.iter().find(|l| l.id == id)
    }

    pub fn list(&self) -> &[ShareLink] {
        &self.links
    }
}

impl Default for ShareManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_share_manager_new() {
        let sm = ShareManager::new();
        assert!(sm.list().is_empty());
    }

    #[test]
    fn test_share_manager_create() {
        let mut sm = ShareManager::new();
        let link = sm.create("session-123".to_string(), true);

        assert_eq!(link.session_id, "session-123");
        assert!(link.is_public);
        assert!(!link.id.is_empty());
    }

    #[test]
    fn test_share_manager_get() {
        let mut sm = ShareManager::new();
        let link = sm.create("session-123".to_string(), true);
        let id = link.id.clone();

        let found = sm.get(&id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().session_id, "session-123");
    }

    #[test]
    fn test_share_manager_list() {
        let mut sm = ShareManager::new();
        sm.create("session-1".to_string(), true);
        sm.create("session-2".to_string(), false);

        assert_eq!(sm.list().len(), 2);
    }

    #[test]
    fn test_share_manager_get_not_found() {
        let sm = ShareManager::new();
        assert!(sm.get("nonexistent").is_none());
    }
}
