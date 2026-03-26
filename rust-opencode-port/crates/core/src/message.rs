use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

impl Message {
    pub fn new(role: Role, content: String) -> Self {
        Self {
            role,
            content,
            timestamp: Utc::now(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new(Role::User, content.into())
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(Role::Assistant, content.into())
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self::new(Role::System, content.into())
    }
}
