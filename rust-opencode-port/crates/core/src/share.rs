use crate::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareLink {
    pub id: String,
    pub session_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub include_metadata: bool,
    pub sanitize_sensitive: bool,
    pub format: ExportFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Markdown,
    PatchBundle,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_metadata: true,
            sanitize_sensitive: true,
            format: ExportFormat::Json,
        }
    }
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
            if msg.role == crate::message::Role::Assistant {
                if msg.content.contains("```diff") || msg.content.contains("```patch") {
                    bundle.push_str(&msg.content);
                    bundle.push_str("\n---\n\n");
                    patch_count += 1;
                }
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
