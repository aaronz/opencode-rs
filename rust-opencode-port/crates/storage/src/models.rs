use chrono::{DateTime, Utc};
use opencode_core::{OpenCodeError, Session};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionModel {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectModel {
    pub id: String,
    pub path: String,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountModel {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub data: Option<String>,
}

impl From<Session> for SessionModel {
    fn from(session: Session) -> Self {
        Self {
            id: session.id.to_string(),
            created_at: session.created_at,
            updated_at: session.updated_at,
            data: serde_json::to_string(&session).unwrap_or_default(),
        }
    }
}

impl TryFrom<SessionModel> for Session {
    type Error = OpenCodeError;

    fn try_from(model: SessionModel) -> Result<Self, Self::Error> {
        serde_json::from_str(&model.data).map_err(|e| OpenCodeError::Storage(e.to_string()))
    }
}
