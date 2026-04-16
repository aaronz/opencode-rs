use crate::routes::error::{internal_error, json_error};
use crate::ServerState;
use actix_web::{http::StatusCode, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use opencode_core::config::ShareMode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortShareConfig {
    pub base_url: String,
    pub token_length: usize,
    pub default_expiry_hours: u64,
    pub max_expiry_hours: u64,
}

impl Default for ShortShareConfig {
    fn default() -> Self {
        Self {
            base_url: "https://share.opencode.ai".to_string(),
            token_length: 8,
            default_expiry_hours: 24 * 7,
            max_expiry_hours: 24 * 30,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShortShareLink {
    pub short_code: String,
    pub access_token: String,
    pub session_id: String,
    pub share_mode: ShareMode,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub view_count: u64,
    pub max_views: Option<u64>,
    pub allowed_operations: Vec<ShareOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShareOperation {
    Read,
    Write,
    Delete,
    Fork,
}

pub struct ShareServer {
    config: ShortShareConfig,
    links: Arc<RwLock<HashMap<String, ShortShareLink>>>,
}

impl ShareServer {
    pub fn new(config: ShortShareConfig) -> Self {
        Self {
            config,
            links: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(ShortShareConfig::default())
    }

    fn generate_short_code(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let charset: Vec<char> = "abcdefghijkmnpqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789"
            .chars()
            .collect();
        (0..self.config.token_length)
            .map(|_| charset[rng.gen_range(0..charset.len())])
            .collect()
    }

    fn generate_access_token(&self) -> String {
        Uuid::new_v4().to_string().replace("-", "")
    }

    pub async fn create_short_link(
        &self,
        session_id: String,
        expiry_hours: Option<u64>,
        max_views: Option<u64>,
    ) -> ShortShareLink {
        self.create_short_link_with_mode(session_id, ShareMode::Manual, expiry_hours, max_views)
            .await
    }

    pub async fn create_short_link_with_mode(
        &self,
        session_id: String,
        share_mode: ShareMode,
        expiry_hours: Option<u64>,
        max_views: Option<u64>,
    ) -> ShortShareLink {
        let short_code = self.generate_short_code();
        let access_token = self.generate_access_token();

        let now = Utc::now();
        let expiry_hours = expiry_hours.unwrap_or(self.config.default_expiry_hours);
        let expires_at = if expiry_hours > 0 {
            Some(now + chrono::Duration::hours(expiry_hours as i64))
        } else {
            None
        };

        let allowed_operations = Self::get_allowed_operations_for_mode(&share_mode);

        let link = ShortShareLink {
            short_code,
            access_token,
            session_id,
            share_mode,
            created_at: now,
            expires_at,
            view_count: 0,
            max_views,
            allowed_operations,
        };

        let mut links = self.links.write().await;
        links.insert(link.short_code.clone(), link.clone());

        link
    }

    fn get_allowed_operations_for_mode(mode: &ShareMode) -> Vec<ShareOperation> {
        match mode {
            ShareMode::Disabled => vec![],
            ShareMode::ReadOnly => vec![ShareOperation::Read],
            ShareMode::Manual | ShareMode::Auto => vec![ShareOperation::Read],
            ShareMode::Collaborative => vec![
                ShareOperation::Read,
                ShareOperation::Write,
                ShareOperation::Fork,
            ],
            ShareMode::Controlled => vec![ShareOperation::Read],
        }
    }

    pub async fn check_permission(&self, short_code: &str, operation: ShareOperation) -> bool {
        let links = self.links.read().await;
        if let Some(link) = links.get(short_code) {
            if link.share_mode == ShareMode::Disabled {
                return false;
            }
            return link.allowed_operations.contains(&operation);
        }
        false
    }

    pub async fn update_share_mode(&self, short_code: &str, new_mode: ShareMode) -> bool {
        let mut links = self.links.write().await;
        if let Some(link) = links.get_mut(short_code) {
            link.share_mode = new_mode.clone();
            link.allowed_operations = Self::get_allowed_operations_for_mode(&new_mode);
            return true;
        }
        false
    }

    pub async fn add_allowed_operation(&self, short_code: &str, operation: ShareOperation) -> bool {
        let mut links = self.links.write().await;
        if let Some(link) = links.get_mut(short_code) {
            if !link.allowed_operations.contains(&operation) {
                link.allowed_operations.push(operation);
            }
            return true;
        }
        false
    }

    pub async fn get_short_link(&self, short_code: &str) -> Option<ShortShareLink> {
        let links = self.links.read().await;
        links.get(short_code).cloned()
    }

    pub async fn validate_access(&self, short_code: &str, access_token: Option<&str>) -> bool {
        let links = self.links.read().await;
        if let Some(link) = links.get(short_code) {
            if let Some(token) = access_token {
                return link.access_token == token;
            }
        }
        false
    }

    pub async fn record_view(&self, short_code: &str) -> bool {
        let mut links = self.links.write().await;
        if let Some(link) = links.get_mut(short_code) {
            if let Some(max) = link.max_views {
                if link.view_count >= max {
                    return false;
                }
            }
            link.view_count += 1;
            return true;
        }
        false
    }

    pub async fn is_expired(&self, short_code: &str) -> bool {
        let links = self.links.read().await;
        if let Some(link) = links.get(short_code) {
            if let Some(expires_at) = link.expires_at {
                return Utc::now() > expires_at;
            }
        }
        false
    }

    pub async fn delete_short_link(&self, short_code: &str) -> bool {
        let mut links = self.links.write().await;
        links.remove(short_code).is_some()
    }

    pub async fn cleanup_expired(&self) -> usize {
        let mut links = self.links.write().await;
        let now = Utc::now();
        let before = links.len();
        links.retain(|_, link| link.expires_at.map(|e| e > now).unwrap_or(true));
        before - links.len()
    }

    pub fn full_url(&self, short_code: &str) -> String {
        format!("{}/s/{}", self.config.base_url, short_code)
    }

    pub fn full_url_with_token(&self, short_code: &str, access_token: &str) -> String {
        format!(
            "{}/s/{}/?token={}",
            self.config.base_url, short_code, access_token
        )
    }
}

impl Default for ShareServer {
    fn default() -> Self {
        Self::with_default_config()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateShortLinkRequest {
    pub session_id: String,
    pub expiry_hours: Option<u64>,
    pub max_views: Option<u64>,
    pub include_access_token: Option<bool>,
    #[serde(default)]
    pub share_mode: Option<ShareMode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShortLinkResponse {
    pub short_code: String,
    pub short_url: String,
    pub access_token: Option<String>,
    pub session_id: String,
    pub share_mode: ShareMode,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_views: Option<u64>,
    pub allowed_operations: Vec<ShareOperation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShortLinkInfo {
    pub short_code: String,
    pub session_id: String,
    pub share_mode: ShareMode,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub view_count: u64,
    pub max_views: Option<u64>,
    pub allowed_operations: Vec<ShareOperation>,
}

pub async fn create_short_link(
    state: web::Data<ServerState>,
    req: web::Json<CreateShortLinkRequest>,
) -> impl Responder {
    let share_server = match state.share_server.read() {
        Ok(guard) => guard,
        Err(_) => {
            return internal_error("Share server lock poisoned");
        }
    };

    match state.storage.load_session(&req.session_id).await {
        Ok(Some(_session)) => {
            let share_mode = req.share_mode.clone().unwrap_or(ShareMode::Manual);
            let link = share_server
                .create_short_link_with_mode(
                    req.session_id.clone(),
                    share_mode.clone(),
                    req.expiry_hours,
                    req.max_views,
                )
                .await;

            let short_url = share_server.full_url(&link.short_code);
            let access_token = if req.include_access_token.unwrap_or(false) {
                Some(link.access_token.clone())
            } else {
                None
            };

            HttpResponse::Created().json(ShortLinkResponse {
                short_code: link.short_code,
                short_url,
                access_token,
                session_id: link.session_id,
                share_mode,
                expires_at: link.expires_at,
                max_views: link.max_views,
                allowed_operations: link.allowed_operations,
            })
        }
        Ok(None) => json_error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            format!("Session not found: {}", req.session_id),
        ),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        ),
    }
}

pub async fn get_short_link_info(
    state: web::Data<ServerState>,
    path: web::Path<String>,
) -> impl Responder {
    let short_code = path.into_inner();
    let share_server = match state.share_server.read() {
        Ok(guard) => guard,
        Err(_) => {
            return internal_error("Share server lock poisoned");
        }
    };

    match share_server.get_short_link(&short_code).await {
        Some(link) => {
            if share_server.is_expired(&short_code).await {
                return json_error(
                    StatusCode::GONE,
                    "link_expired",
                    "This share link has expired",
                );
            }

            HttpResponse::Ok().json(ShortLinkInfo {
                short_code: link.short_code,
                session_id: link.session_id,
                share_mode: link.share_mode,
                created_at: link.created_at,
                expires_at: link.expires_at,
                view_count: link.view_count,
                max_views: link.max_views,
                allowed_operations: link.allowed_operations,
            })
        }
        None => json_error(
            StatusCode::NOT_FOUND,
            "link_not_found",
            format!("Share link not found: {}", short_code),
        ),
    }
}

pub async fn access_shared_session(
    state: web::Data<ServerState>,
    path: web::Path<String>,
    query: web::Query<AccessQueryParams>,
) -> impl Responder {
    let short_code = path.into_inner();
    let share_server = match state.share_server.read() {
        Ok(guard) => guard,
        Err(_) => {
            return internal_error("Share server lock poisoned");
        }
    };

    let link = match share_server.get_short_link(&short_code).await {
        Some(l) => l,
        None => {
            return json_error(
                StatusCode::NOT_FOUND,
                "link_not_found",
                format!("Share link not found: {}", short_code),
            );
        }
    };

    if share_server.is_expired(&short_code).await {
        return json_error(
            StatusCode::GONE,
            "link_expired",
            "This share link has expired",
        );
    }

    if !share_server
        .validate_access(&short_code, query.token.as_deref())
        .await
    {
        return json_error(
            StatusCode::FORBIDDEN,
            "access_denied",
            "Invalid or missing access token",
        );
    }

    if !share_server.record_view(&short_code).await {
        return json_error(
            StatusCode::FORBIDDEN,
            "view_limit_reached",
            "This share link has reached its maximum number of views",
        );
    }

    match state.storage.load_session(&link.session_id).await {
        Ok(Some(session)) => {
            let sanitized = session.sanitize_for_export();
            let read_only = !share_server
                .check_permission(&short_code, ShareOperation::Write)
                .await;
            HttpResponse::Ok().json(serde_json::json!({
                "id": sanitized.id,
                "created_at": sanitized.created_at,
                "updated_at": sanitized.updated_at,
                "messages": sanitized.messages,
                "share_mode": link.share_mode,
                "allowed_operations": link.allowed_operations,
                "read_only": read_only,
                "view_count": link.view_count + 1,
            }))
        }
        Ok(None) => json_error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            "The shared session no longer exists",
        ),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct AccessQueryParams {
    pub token: Option<String>,
}

pub async fn delete_short_link(
    state: web::Data<ServerState>,
    path: web::Path<String>,
    query: web::Query<DeleteQueryParams>,
) -> impl Responder {
    let short_code = path.into_inner();
    let share_server = match state.share_server.read() {
        Ok(guard) => guard,
        Err(_) => {
            return internal_error("Share server lock poisoned");
        }
    };

    if !share_server
        .validate_access(&short_code, Some(&query.token))
        .await
    {
        return json_error(
            StatusCode::FORBIDDEN,
            "access_denied",
            "Invalid or missing access token",
        );
    }

    if share_server.delete_short_link(&short_code).await {
        HttpResponse::NoContent().finish()
    } else {
        json_error(
            StatusCode::NOT_FOUND,
            "link_not_found",
            format!("Share link not found: {}", short_code),
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct DeleteQueryParams {
    pub token: String,
}

pub async fn refresh_short_link(
    state: web::Data<ServerState>,
    path: web::Path<String>,
    req: web::Json<RefreshRequest>,
) -> impl Responder {
    let short_code = path.into_inner();
    let share_server = match state.share_server.read() {
        Ok(guard) => guard,
        Err(_) => {
            return internal_error("Share server lock poisoned");
        }
    };

    if !share_server
        .validate_access(&short_code, Some(&req.access_token))
        .await
    {
        return json_error(
            StatusCode::FORBIDDEN,
            "access_denied",
            "Invalid access token",
        );
    }

    let links = share_server.links.read().await;
    if let Some(link) = links.get(&short_code) {
        let now = Utc::now();
        let new_expiry = req
            .expiry_hours
            .map(|h| now + chrono::Duration::hours(h as i64))
            .or(link.expires_at);

        drop(links);

        let mut links = share_server.links.write().await;
        if let Some(existing) = links.get_mut(&short_code) {
            existing.expires_at = new_expiry;
            let updated = existing.clone();
            drop(links);

            return HttpResponse::Ok().json(serde_json::json!({
                "short_code": updated.short_code,
                "expires_at": updated.expires_at,
                "status": "refreshed",
            }));
        }
    }

    json_error(
        StatusCode::NOT_FOUND,
        "link_not_found",
        format!("Share link not found: {}", short_code),
    )
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub access_token: String,
    pub expiry_hours: Option<u64>,
}

pub async fn list_short_links(state: web::Data<ServerState>) -> impl Responder {
    let share_server = match state.share_server.read() {
        Ok(guard) => guard,
        Err(_) => {
            return internal_error("Share server lock poisoned");
        }
    };
    let links = share_server.links.read().await;

    let items: Vec<ShortLinkInfo> = links
        .values()
        .map(|link| ShortLinkInfo {
            short_code: link.short_code.clone(),
            session_id: link.session_id.clone(),
            share_mode: link.share_mode.clone(),
            created_at: link.created_at,
            expires_at: link.expires_at,
            view_count: link.view_count,
            max_views: link.max_views,
            allowed_operations: link.allowed_operations.clone(),
        })
        .collect();

    HttpResponse::Ok().json(serde_json::json!({
        "items": items,
        "count": items.len(),
    }))
}

pub async fn cleanup_expired_links(state: web::Data<ServerState>) -> impl Responder {
    let share_server = match state.share_server.read() {
        Ok(guard) => guard,
        Err(_) => {
            return internal_error("Share server lock poisoned");
        }
    };
    let cleaned = share_server.cleanup_expired().await;

    HttpResponse::Ok().json(serde_json::json!({
        "cleaned_count": cleaned,
        "status": "completed",
    }))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::post().to(create_short_link));
    cfg.route("", web::get().to(list_short_links));
    cfg.route("/cleanup", web::post().to(cleanup_expired_links));
    cfg.route("/{short_code}", web::get().to(get_short_link_info));
    cfg.route("/{short_code}/access", web::get().to(access_shared_session));
    cfg.route("/{short_code}", web::delete().to(delete_short_link));
    cfg.route("/{short_code}/refresh", web::post().to(refresh_short_link));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_share_config_default() {
        let config = ShortShareConfig::default();
        assert_eq!(config.base_url, "https://share.opencode.ai");
        assert_eq!(config.token_length, 8);
        assert_eq!(config.default_expiry_hours, 24 * 7);
        assert_eq!(config.max_expiry_hours, 24 * 30);
    }

    #[test]
    fn test_short_share_config_custom() {
        let config = ShortShareConfig {
            base_url: "https://custom.share.com".to_string(),
            token_length: 16,
            default_expiry_hours: 48,
            max_expiry_hours: 24 * 7,
        };
        assert_eq!(config.base_url, "https://custom.share.com");
        assert_eq!(config.token_length, 16);
    }

    #[test]
    fn test_share_server_default_config() {
        let server = ShareServer::default();
        let config = server.config;
        assert_eq!(config.base_url, "https://share.opencode.ai");
    }

    #[test]
    fn test_share_server_with_default_config() {
        let server = ShareServer::with_default_config();
        assert_eq!(server.config.token_length, 8);
    }

    #[test]
    fn test_share_server_with_custom_config() {
        let config = ShortShareConfig {
            base_url: "https://test.share.com".to_string(),
            token_length: 12,
            default_expiry_hours: 72,
            max_expiry_hours: 24 * 14,
        };
        let server = ShareServer::new(config);
        assert_eq!(server.config.token_length, 12);
    }

    #[tokio::test]
    async fn test_create_short_link() {
        let server = ShareServer::with_default_config();
        let link = server
            .create_short_link("session-123".to_string(), None, None)
            .await;

        assert_eq!(link.session_id, "session-123");
        assert!(!link.short_code.is_empty());
        assert!(!link.access_token.is_empty());
        assert!(link.expires_at.is_some());
    }

    #[tokio::test]
    async fn test_create_short_link_with_expiry() {
        let server = ShareServer::with_default_config();
        let link = server
            .create_short_link("session-456".to_string(), Some(1), None)
            .await;

        assert!(link.expires_at.is_some());
        if let Some(expires) = link.expires_at {
            let now = chrono::Utc::now();
            assert!(expires > now);
        }
    }

    #[tokio::test]
    async fn test_create_short_link_with_max_views() {
        let server = ShareServer::with_default_config();
        let link = server
            .create_short_link("session-789".to_string(), None, Some(5))
            .await;

        assert_eq!(link.max_views, Some(5));
        assert_eq!(link.view_count, 0);
    }

    #[tokio::test]
    async fn test_get_short_link() {
        let server = ShareServer::with_default_config();
        let created = server
            .create_short_link("session-abc".to_string(), None, None)
            .await;

        let retrieved = server.get_short_link(&created.short_code).await;
        assert!(retrieved.is_some());
        let link = retrieved.unwrap();
        assert_eq!(link.session_id, "session-abc");
    }

    #[tokio::test]
    async fn test_get_short_link_not_found() {
        let server = ShareServer::with_default_config();
        let retrieved = server.get_short_link("nonexistent").await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_validate_access_with_correct_token() {
        let server = ShareServer::with_default_config();
        let link = server
            .create_short_link("session-xyz".to_string(), None, None)
            .await;

        let is_valid = server
            .validate_access(&link.short_code, Some(&link.access_token))
            .await;
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_validate_access_with_wrong_token() {
        let server = ShareServer::with_default_config();
        let link = server
            .create_short_link("session-xyz".to_string(), None, None)
            .await;

        let is_valid = server
            .validate_access(&link.short_code, Some("wrong-token"))
            .await;
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_validate_access_no_token() {
        let server = ShareServer::with_default_config();
        let link = server
            .create_short_link("session-xyz".to_string(), None, None)
            .await;

        let is_valid = server.validate_access(&link.short_code, None).await;
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_record_view() {
        let server = ShareServer::with_default_config();
        let link = server
            .create_short_link("session-view".to_string(), None, None)
            .await;

        let result = server.record_view(&link.short_code).await;
        assert!(result);

        let updated = server.get_short_link(&link.short_code).await.unwrap();
        assert_eq!(updated.view_count, 1);
    }

    #[tokio::test]
    async fn test_record_view_respects_max_views() {
        let server = ShareServer::with_default_config();
        let link = server
            .create_short_link("session-max".to_string(), None, Some(2))
            .await;

        assert!(server.record_view(&link.short_code).await);
        assert!(server.record_view(&link.short_code).await);
        assert!(!server.record_view(&link.short_code).await);
    }

    #[tokio::test]
    async fn test_delete_short_link() {
        let server = ShareServer::with_default_config();
        let link = server
            .create_short_link("session-del".to_string(), None, None)
            .await;

        let deleted = server.delete_short_link(&link.short_code).await;
        assert!(deleted);

        let retrieved = server.get_short_link(&link.short_code).await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_delete_short_link_not_found() {
        let server = ShareServer::with_default_config();
        let deleted = server.delete_short_link("nonexistent").await;
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_is_expired_false() {
        let server = ShareServer::with_default_config();
        let link = server
            .create_short_link("session-valid".to_string(), Some(24), None)
            .await;

        let expired = server.is_expired(&link.short_code).await;
        assert!(!expired);
    }

    #[tokio::test]
    async fn test_is_expired_nonexistent() {
        let server = ShareServer::with_default_config();
        let expired = server.is_expired("nonexistent").await;
        assert!(!expired);
    }

    #[tokio::test]
    async fn test_check_permission_read() {
        let server = ShareServer::with_default_config();
        let link = server
            .create_short_link("session-perm".to_string(), None, None)
            .await;

        let has_read = server
            .check_permission(&link.short_code, ShareOperation::Read)
            .await;
        assert!(has_read);
    }

    #[tokio::test]
    async fn test_check_permission_disabled_mode() {
        let server = ShareServer::with_default_config();
        let link = server
            .create_short_link_with_mode(
                "session-disabled".to_string(),
                ShareMode::Disabled,
                None,
                None,
            )
            .await;

        let has_read = server
            .check_permission(&link.short_code, ShareOperation::Read)
            .await;
        assert!(!has_read);
    }

    #[tokio::test]
    async fn test_update_share_mode() {
        let server = ShareServer::with_default_config();
        let link = server
            .create_short_link("session-mode".to_string(), None, None)
            .await;

        let updated = server
            .update_share_mode(&link.short_code, ShareMode::ReadOnly)
            .await;
        assert!(updated);

        let retrieved = server.get_short_link(&link.short_code).await.unwrap();
        assert_eq!(retrieved.share_mode, ShareMode::ReadOnly);
    }

    #[tokio::test]
    async fn test_update_share_mode_not_found() {
        let server = ShareServer::with_default_config();
        let updated = server
            .update_share_mode("nonexistent", ShareMode::ReadOnly)
            .await;
        assert!(!updated);
    }

    #[tokio::test]
    async fn test_add_allowed_operation() {
        let server = ShareServer::with_default_config();
        let link = server
            .create_short_link("session-add".to_string(), None, None)
            .await;

        let added = server
            .add_allowed_operation(&link.short_code, ShareOperation::Write)
            .await;
        assert!(added);

        let has_write = server
            .check_permission(&link.short_code, ShareOperation::Write)
            .await;
        assert!(has_write);
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let server = ShareServer::with_default_config();

        let link1 = server
            .create_short_link("session-keep".to_string(), Some(24 * 30), None)
            .await;
        let link2 = server
            .create_short_link("session-expired".to_string(), Some(0), None)
            .await;

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let cleaned = server.cleanup_expired().await;
        assert_eq!(cleaned, 1);

        assert!(server.get_short_link(&link1.short_code).await.is_some());
    }

    #[test]
    fn test_full_url() {
        let server = ShareServer::with_default_config();
        let url = server.full_url("abc123");
        assert_eq!(url, "https://share.opencode.ai/s/abc123");
    }

    #[test]
    fn test_full_url_with_token() {
        let server = ShareServer::with_default_config();
        let url = server.full_url_with_token("abc123", "token-xyz");
        assert_eq!(url, "https://share.opencode.ai/s/abc123/?token=token-xyz");
    }

    #[test]
    fn test_share_operation_serialization() {
        let ops = vec![
            ShareOperation::Read,
            ShareOperation::Write,
            ShareOperation::Delete,
            ShareOperation::Fork,
        ];

        for op in ops {
            let json = serde_json::to_string(&op).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_share_operation_deserialization() {
        let read: ShareOperation = serde_json::from_str("\"Read\"").unwrap();
        assert_eq!(read, ShareOperation::Read);

        let write: ShareOperation = serde_json::from_str("\"Write\"").unwrap();
        assert_eq!(write, ShareOperation::Write);

        let delete: ShareOperation = serde_json::from_str("\"Delete\"").unwrap();
        assert_eq!(delete, ShareOperation::Delete);

        let fork: ShareOperation = serde_json::from_str("\"Fork\"").unwrap();
        assert_eq!(fork, ShareOperation::Fork);
    }

    #[test]
    fn test_short_link_info_serialization() {
        let info = ShortLinkInfo {
            short_code: "abc123".to_string(),
            session_id: "session-xyz".to_string(),
            share_mode: ShareMode::Manual,
            created_at: chrono::Utc::now(),
            expires_at: None,
            view_count: 5,
            max_views: Some(10),
            allowed_operations: vec![ShareOperation::Read, ShareOperation::Write],
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("abc123"));
        assert!(json.contains("session-xyz"));
    }

    #[test]
    fn test_short_link_response_serialization() {
        let response = ShortLinkResponse {
            short_code: "code123".to_string(),
            short_url: "https://share.opencode.ai/s/code123".to_string(),
            access_token: Some("token-abc".to_string()),
            session_id: "session-456".to_string(),
            share_mode: ShareMode::Collaborative,
            expires_at: None,
            max_views: Some(20),
            allowed_operations: vec![
                ShareOperation::Read,
                ShareOperation::Write,
                ShareOperation::Fork,
            ],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("code123"));
        assert!(json.contains("token-abc"));
    }

    #[test]
    fn test_create_short_link_request_deserialization() {
        let json = r#"{
            "session_id": "test-session",
            "expiry_hours": 48,
            "max_views": 100,
            "include_access_token": true,
            "share_mode": "collaborative"
        }"#;

        let req: CreateShortLinkRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.session_id, "test-session");
        assert_eq!(req.expiry_hours, Some(48));
        assert_eq!(req.max_views, Some(100));
        assert_eq!(req.include_access_token, Some(true));
        assert_eq!(req.share_mode, Some(ShareMode::Collaborative));
    }

    #[test]
    fn test_create_short_link_request_minimal() {
        let json = r#"{"session_id": "minimal-session"}"#;

        let req: CreateShortLinkRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.session_id, "minimal-session");
        assert!(req.expiry_hours.is_none());
        assert!(req.max_views.is_none());
        assert!(req.include_access_token.is_none());
        assert!(req.share_mode.is_none());
    }

    #[test]
    fn test_refresh_request_deserialization() {
        let json = r#"{"access_token": "token-xyz", "expiry_hours": 72}"#;
        let req: RefreshRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.access_token, "token-xyz");
        assert_eq!(req.expiry_hours, Some(72));
    }

    #[test]
    fn test_delete_query_params_deserialization() {
        let json = r#"{"token": "delete-token"}"#;
        let req: DeleteQueryParams = serde_json::from_str(json).unwrap();
        assert_eq!(req.token, "delete-token");
    }

    #[test]
    fn test_access_query_params_empty() {
        let json = r#"{}"#;
        let req: AccessQueryParams = serde_json::from_str(json).unwrap();
        assert!(req.token.is_none());
    }

    #[test]
    fn test_access_query_params_with_token() {
        let json = r#"{"token": "access-token-123"}"#;
        let req: AccessQueryParams = serde_json::from_str(json).unwrap();
        assert_eq!(req.token, Some("access-token-123".to_string()));
    }

    #[test]
    fn test_share_mode_all_variants() {
        let modes = vec![
            ShareMode::Disabled,
            ShareMode::ReadOnly,
            ShareMode::Manual,
            ShareMode::Auto,
            ShareMode::Collaborative,
            ShareMode::Controlled,
        ];

        for mode in modes {
            let json = serde_json::to_string(&mode).unwrap();
            let parsed: ShareMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, parsed);
        }
    }

    #[test]
    fn test_get_allowed_operations_for_mode_disabled() {
        let ops = ShareServer::get_allowed_operations_for_mode(&ShareMode::Disabled);
        assert!(ops.is_empty());
    }

    #[test]
    fn test_get_allowed_operations_for_mode_read_only() {
        let ops = ShareServer::get_allowed_operations_for_mode(&ShareMode::ReadOnly);
        assert_eq!(ops, vec![ShareOperation::Read]);
    }

    #[test]
    fn test_get_allowed_operations_for_mode_manual() {
        let ops = ShareServer::get_allowed_operations_for_mode(&ShareMode::Manual);
        assert_eq!(ops, vec![ShareOperation::Read]);
    }

    #[test]
    fn test_get_allowed_operations_for_mode_collaborative() {
        let ops = ShareServer::get_allowed_operations_for_mode(&ShareMode::Collaborative);
        assert_eq!(
            ops,
            vec![
                ShareOperation::Read,
                ShareOperation::Write,
                ShareOperation::Fork
            ]
        );
    }

    #[test]
    fn test_get_allowed_operations_for_mode_controlled() {
        let ops = ShareServer::get_allowed_operations_for_mode(&ShareMode::Controlled);
        assert_eq!(ops, vec![ShareOperation::Read]);
    }
}
