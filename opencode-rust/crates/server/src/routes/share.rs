use crate::routes::error::json_error;
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
        self.create_short_link_with_mode(session_id, ShareMode::Manual, expiry_hours, max_views).await
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
            ShareMode::Collaborative => vec![ShareOperation::Read, ShareOperation::Write, ShareOperation::Fork],
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
    let share_server = state.share_server.read().unwrap();

    match state.storage.load_session(&req.session_id).await {
        Ok(Some(_session)) => {
            let share_mode = req.share_mode.clone().unwrap_or(ShareMode::Manual);
            let link = share_server
                .create_short_link_with_mode(req.session_id.clone(), share_mode.clone(), req.expiry_hours, req.max_views)
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
    let share_server = state.share_server.read().unwrap();

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
    let share_server = state.share_server.read().unwrap();

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
            let read_only = !share_server.check_permission(&short_code, ShareOperation::Write).await;
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
    let share_server = state.share_server.read().unwrap();

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
    let share_server = state.share_server.read().unwrap();

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
    let share_server = state.share_server.read().unwrap();
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
    let share_server = state.share_server.read().unwrap();
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
