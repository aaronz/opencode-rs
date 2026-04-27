use crate::Session;

use super::types::{ExportFormat, ExportOptions, ShareLink};

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

    pub fn export_session(&self, session: &Session, options: &ExportOptions) -> String {
        match options.format {
            ExportFormat::Json => self.export_json(session, options),
            ExportFormat::Markdown => self.export_markdown(session),
            ExportFormat::PatchBundle => self.export_patch_bundle(session),
        }
    }

    fn export_json(&self, session: &Session, options: &ExportOptions) -> String {
        let export_data = if options.sanitize_sensitive {
            self.sanitize_session(session)
        } else {
            session.clone()
        };

        if options.include_metadata {
            serde_json::to_string_pretty(&export_data).unwrap_or_default()
        } else {
            serde_json::to_string_pretty(&export_data.messages).unwrap_or_default()
        }
    }

    fn export_markdown(&self, session: &Session) -> String {
        let mut md = String::new();

        md.push_str(&format!("# Session {}\n\n", session.id));
        md.push_str(&format!(
            "**Created:** {}\n",
            session.created_at.format("%Y-%m-%d %H:%M:%S")
        ));
        md.push_str(&format!(
            "**Updated:** {}\n\n",
            session.updated_at.format("%Y-%m-%d %H:%M:%S")
        ));

        if let Some(parent) = &session.parent_session_id {
            md.push_str(&format!("**Forked from:** {}\n\n", parent));
        }

        md.push_str("---\n\n");

        for msg in &session.messages {
            let role = match msg.role {
                crate::message::Role::User => "**User**",
                crate::message::Role::Assistant => "**Assistant**",
                crate::message::Role::System => "**System**",
            };
            md.push_str(&format!("### {}\n\n", role));
            md.push_str(&msg.content);
            md.push_str("\n\n");
        }

        md
    }

    fn export_patch_bundle(&self, session: &Session) -> String {
        let mut bundle = String::new();

        bundle.push_str("# Patch Bundle\n\n");

        let mut patch_count = 0;
        for msg in &session.messages {
            if msg.role == crate::message::Role::Assistant
                && (msg.content.contains("```diff") || msg.content.contains("```patch"))
            {
                bundle.push_str(&msg.content);
                bundle.push_str("\n---\n\n");
                patch_count += 1;
            }
        }

        if patch_count == 0 {
            bundle.push_str("*No patches found in session*\n");
        }

        bundle
    }

    fn sanitize_session(&self, session: &Session) -> Session {
        let mut sanitized = session.clone();

        let sensitive_patterns = ["sk-", "api_key", "api-key", "secret", "password", "token"];

        for msg in &mut sanitized.messages {
            for pattern in &sensitive_patterns {
                if msg.content.to_lowercase().contains(pattern) {
                    msg.content = msg
                        .content
                        .split(pattern)
                        .enumerate()
                        .map(|(i, part)| {
                            if i > 0 && part.len() > 4 {
                                format!("{}**[REDACTED]**", &part[..4])
                            } else {
                                part.to_string()
                            }
                        })
                        .collect();
                }
            }
        }

        sanitized
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
    use crate::message::Message;

    fn create_test_session() -> Session {
        let mut session = Session::new();
        session.messages.push(Message::user("Hello"));
        session.messages.push(Message::assistant("Hi there!"));
        session
    }

    fn create_session_with_sensitive_data() -> Session {
        let mut session = Session::new();
        session
            .messages
            .push(Message::user("My API key is sk-1234567890abcdef"));
        session.messages.push(Message::assistant(
            "I used api_key=secret123 to authenticate",
        ));
        session
    }

    fn create_session_with_patches() -> Session {
        let mut session = Session::new();
        session.messages.push(Message::user("Please fix the bug"));
        session.messages.push(Message::assistant("Here is the fix: ```diff\n--- a/foo.rs\n+++ b/foo.rs\n@@ -1 +1 @@\n-old code\n+new code\n```"));
        session
    }

    fn create_session_with_patch_and_regular_content() -> Session {
        let mut session = Session::new();
        session
            .messages
            .push(Message::assistant("Regular response without patches"));
        session.messages.push(Message::assistant("Here is a change: ```patch\n--- original\n+++ modified\n@@ -1 +1 @@\n-content\n+new content\n```"));
        session
            .messages
            .push(Message::assistant("Another response without patches"));
        session
    }

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

    #[test]
    fn test_export_session_json_format() {
        let sm = ShareManager::new();
        let session = create_test_session();
        let options = ExportOptions {
            include_metadata: true,
            sanitize_sensitive: false,
            format: ExportFormat::Json,
        };

        let result = sm.export_session(&session, &options);

        assert!(result.contains("\"id\""));
        assert!(result.contains("\"messages\""));
        assert!(result.contains("Hello"));
        assert!(result.contains("Hi there!"));
    }

    #[test]
    fn test_export_session_markdown_format() {
        let sm = ShareManager::new();
        let session = create_test_session();
        let options = ExportOptions {
            include_metadata: true,
            sanitize_sensitive: false,
            format: ExportFormat::Markdown,
        };

        let result = sm.export_session(&session, &options);

        assert!(result.contains("# Session"));
        assert!(result.contains("**Created:**"));
        assert!(result.contains("**Updated:**"));
        assert!(result.contains("**User**"));
        assert!(result.contains("**Assistant**"));
        assert!(result.contains("Hello"));
        assert!(result.contains("Hi there!"));
    }

    #[test]
    fn test_export_session_patch_bundle_format() {
        let sm = ShareManager::new();
        let session = create_session_with_patches();
        let options = ExportOptions {
            include_metadata: true,
            sanitize_sensitive: false,
            format: ExportFormat::PatchBundle,
        };

        let result = sm.export_session(&session, &options);

        assert!(result.contains("# Patch Bundle"));
        assert!(result.contains("```diff"));
    }

    #[test]
    fn test_export_session_patch_bundle_no_patches() {
        let sm = ShareManager::new();
        let session = create_test_session();
        let options = ExportOptions {
            include_metadata: true,
            sanitize_sensitive: false,
            format: ExportFormat::PatchBundle,
        };

        let result = sm.export_session(&session, &options);

        assert!(result.contains("# Patch Bundle"));
        assert!(result.contains("*No patches found in session*"));
    }

    #[test]
    fn test_export_json_with_metadata() {
        let sm = ShareManager::new();
        let session = create_test_session();
        let options = ExportOptions {
            include_metadata: true,
            sanitize_sensitive: false,
            format: ExportFormat::Json,
        };

        let result = sm.export_json(&session, &options);

        assert!(result.contains("\"id\""));
        assert!(result.contains("\"messages\""));
        assert!(result.contains("\"created_at\""));
    }

    #[test]
    fn test_export_json_without_metadata() {
        let sm = ShareManager::new();
        let session = create_test_session();
        let options = ExportOptions {
            include_metadata: false,
            sanitize_sensitive: false,
            format: ExportFormat::Json,
        };

        let result = sm.export_json(&session, &options);

        let parsed: Vec<Message> = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_export_json_with_sanitization() {
        let sm = ShareManager::new();
        let session = create_session_with_sensitive_data();
        let options = ExportOptions {
            include_metadata: true,
            sanitize_sensitive: true,
            format: ExportFormat::Json,
        };

        let result = sm.export_json(&session, &options);

        assert!(result.contains("**[REDACTED]**"));
        assert!(!result.contains("sk-1234567890abcdef"));
    }

    #[test]
    fn test_export_json_without_sanitization() {
        let sm = ShareManager::new();
        let session = create_session_with_sensitive_data();
        let options = ExportOptions {
            include_metadata: true,
            sanitize_sensitive: false,
            format: ExportFormat::Json,
        };

        let result = sm.export_json(&session, &options);

        assert!(result.contains("sk-1234567890abcdef"));
    }

    #[test]
    fn test_export_markdown_with_forked_session() {
        let sm = ShareManager::new();
        let mut session = create_test_session();
        session.parent_session_id = Some("parent-session-id".to_string());

        let result = sm.export_markdown(&session);

        assert!(result.contains("**Forked from:**"));
        assert!(result.contains("parent-session-id"));
    }

    #[test]
    fn test_export_markdown_empty_session() {
        let sm = ShareManager::new();
        let session = Session::new();

        let result = sm.export_markdown(&session);

        assert!(result.contains("# Session"));
        assert!(result.contains("**Created:**"));
    }

    #[test]
    fn test_export_patch_bundle_multiple_patches() {
        let sm = ShareManager::new();
        let session = create_session_with_patch_and_regular_content();

        let result = sm.export_patch_bundle(&session);

        assert!(result.contains("# Patch Bundle"));
        assert!(result.contains("```patch"));
        assert!(result.contains("--- original"));
    }

    #[test]
    fn test_sanitize_session_api_key() {
        let sm = ShareManager::new();
        let session = create_session_with_sensitive_data();

        let sanitized = sm.sanitize_session(&session);

        let user_content = &sanitized.messages[0].content;
        assert!(user_content.contains("**[REDACTED]**"));
        assert!(!user_content.contains("sk-1234567890abcdef"));
    }

    #[test]
    fn test_sanitize_session_api_key_pattern() {
        let sm = ShareManager::new();
        let mut session = Session::new();
        session
            .messages
            .push(Message::user("api_key=my-secret-value"));
        session.messages.push(Message::user("my-api-key=something"));
        session.messages.push(Message::user("token=abc123"));

        let sanitized = sm.sanitize_session(&session);

        for msg in &sanitized.messages {
            if msg.content.contains("**[REDACTED]**") {
                assert!(!msg.content.contains("my-secret-value"));
                assert!(!msg.content.contains("something"));
                assert!(!msg.content.contains("abc123"));
            }
        }
    }

    #[test]
    fn test_sanitize_session_no_sensitive_data() {
        let sm = ShareManager::new();
        let session = create_test_session();

        let sanitized = sm.sanitize_session(&session);

        assert_eq!(sanitized.messages.len(), session.messages.len());
    }

    #[test]
    fn test_export_options_default() {
        let options = ExportOptions::default();

        assert!(options.include_metadata);
        assert!(options.sanitize_sensitive);
        assert_eq!(options.format, ExportFormat::Json);
    }

    #[test]
    fn test_share_manager_default() {
        let sm = ShareManager::default();
        assert!(sm.list().is_empty());
    }
}
