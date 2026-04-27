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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parts: Option<Vec<crate::part::Part>>,
}

impl Message {
    pub fn new(role: Role, content: String) -> Self {
        Self {
            role,
            content,
            timestamp: Utc::now(),
            parts: None,
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

    pub fn from_parts(role: Role, parts: Vec<crate::part::Part>) -> Self {
        let content = parts
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        Self {
            role,
            content,
            timestamp: Utc::now(),
            parts: Some(parts),
        }
    }
}
