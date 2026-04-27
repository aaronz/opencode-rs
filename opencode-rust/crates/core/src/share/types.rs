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
