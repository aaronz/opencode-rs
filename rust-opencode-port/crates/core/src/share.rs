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
