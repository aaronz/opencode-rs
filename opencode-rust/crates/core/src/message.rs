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

    #[test]
    fn test_message_with_parts() {
        let msg = Message::from_parts(Role::User, vec![crate::part::Part::text("hello")]);
        assert!(msg.parts.is_some());
        assert_eq!(msg.parts.as_ref().map(|p| p.len()), Some(1));
    }

    #[test]
    fn test_message_backward_compat() {
        let json = r#"{"role":"user","content":"hello","timestamp":"2024-01-01T00:00:00Z"}"#;
        let msg: Message = serde_json::from_str(json).unwrap();
        assert_eq!(msg.content, "hello");
        assert!(msg.parts.is_none());
    }
}
