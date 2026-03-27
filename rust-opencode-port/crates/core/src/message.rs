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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_user() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_message_assistant() {
        let msg = Message::assistant("Response");
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.content, "Response");
    }

    #[test]
    fn test_message_system() {
        let msg = Message::system("System prompt");
        assert_eq!(msg.role, Role::System);
        assert_eq!(msg.content, "System prompt");
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::user("test");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("test"));
    }
}
