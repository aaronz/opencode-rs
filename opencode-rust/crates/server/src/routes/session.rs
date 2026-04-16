use crate::routes::error::{bad_request, json_error, permission_denied_error};
use crate::routes::validation::{validate_pagination, validate_session_id, RequestValidator};
use crate::ServerState;
use actix_web::{http::StatusCode, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use opencode_core::{
    config::ShareMode, permission::Permission, CheckpointManager, Message, PermissionManager, Role,
    Session, SummaryGenerator,
};
use opencode_permission::{AuditDecision, AuditEntry};
use serde::Deserialize;
use tokio::process::Command;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub initial_prompt: Option<String>,
}

#[derive(Deserialize)]
pub struct AddMessageRequest {
    pub role: Option<String>,
    pub content: String,
}

#[derive(Deserialize)]
pub struct CommandRequest {
    pub command: String,
    pub args: Option<Vec<String>>,
    pub workdir: Option<String>,
}

#[derive(Deserialize)]
pub struct RevertRequest {
    pub sequence_number: usize,
}

#[derive(Deserialize)]
pub struct ForkSessionRequest {
    pub fork_at_message_index: usize,
}

#[derive(Deserialize)]
pub struct ShareRequest {
    pub mode: Option<ShareMode>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct PermissionReplyRequest {
    pub decision: String,
}

fn parse_role(role: Option<String>) -> Role {
    match role
        .as_deref()
        .unwrap_or("user")
        .to_ascii_lowercase()
        .as_str()
    {
        "assistant" => Role::Assistant,
        "system" => Role::System,
        _ => Role::User,
    }
}

fn to_session_uuid(id: &str) -> Result<Uuid, HttpResponse> {
    Uuid::parse_str(id).map_err(|_| {
        json_error(
            StatusCode::BAD_REQUEST,
            "invalid_session_id",
            format!("Invalid session id: {id}"),
        )
    })
}

pub async fn list_sessions(
    state: web::Data<ServerState>,
    params: web::Query<PaginationParams>,
) -> impl Responder {
    let limit = params.limit.unwrap_or(20).min(200);
    let offset = params.offset.unwrap_or(0);

    match state.storage.list_sessions(limit, offset).await {
        Ok(sessions) => HttpResponse::Ok().json(serde_json::json!({
            "items": sessions,
            "limit": limit,
            "offset": offset,
            "count": sessions.len(),
        })),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        ),
    }
}

pub async fn create_session(
    state: web::Data<ServerState>,
    req: web::Json<CreateSessionRequest>,
) -> impl Responder {
    let mut validator = RequestValidator::new();
    validator.validate_optional_string("initial_prompt", req.initial_prompt.as_deref(), 50000);
    if let Err(errors) = validator.validate() {
        return errors.to_response();
    }

    let mut session = Session::new();
    if let Some(prompt) = &req.initial_prompt {
        session.add_message(Message::user(prompt.clone()));
    }

    if let Err(e) = state.storage.save_session(&session).await {
        return json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        );
    }

    let _ = CheckpointManager::new().create(&session, "Session created");

    HttpResponse::Created().json(serde_json::json!({
        "session_id": session.id.to_string(),
        "created_at": session.created_at.to_rfc3339(),
        "status": "created",
        "message_count": session.messages.len(),
    }))
}

pub async fn get_session(state: web::Data<ServerState>, id: web::Path<String>) -> impl Responder {
    if let Err(errors) = validate_session_id(&id) {
        return errors.to_response();
    }
    match state.storage.load_session(&id).await {
        Ok(Some(session)) => HttpResponse::Ok().json(session),
        Ok(None) => json_error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            format!("Session not found: {}", id.as_str()),
        ),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        ),
    }
}

pub async fn delete_session(
    state: web::Data<ServerState>,
    id: web::Path<String>,
) -> impl Responder {
    if let Err(errors) = validate_session_id(&id) {
        return errors.to_response();
    }
    match state.storage.delete_session(&id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        ),
    }
}

pub async fn fork_session(
    state: web::Data<ServerState>,
    id: web::Path<String>,
    req: web::Json<ForkSessionRequest>,
) -> impl Responder {
    if let Err(errors) = validate_session_id(&id) {
        return errors.to_response();
    }

    let mut validator = RequestValidator::new();
    validator.validate_required_number(
        "fork_at_message_index",
        Some(req.fork_at_message_index),
        0,
        100000,
    );
    if let Err(errors) = validator.validate() {
        return errors.to_response();
    }

    match state.storage.load_session(&id).await {
        Ok(Some(session)) => match session.fork_at_message(req.fork_at_message_index) {
            Ok(forked) => match state.storage.save_session(&forked).await {
                Ok(_) => HttpResponse::Created().json(serde_json::json!({
                    "id": forked.id.to_string(),
                    "parent_session_id": forked.parent_session_id,
                    "message_count": forked.messages.len(),
                })),
                Err(e) => json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "storage_error",
                    e.to_string(),
                ),
            },
            Err(e) => json_error(StatusCode::BAD_REQUEST, "fork_error", e.to_string()),
        },

        Ok(None) => json_error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            format!("Session not found: {}", id.as_str()),
        ),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        ),
    }
}

pub async fn prompt_session(
    state: web::Data<ServerState>,
    id: web::Path<String>,
    req: web::Json<AddMessageRequest>,
) -> impl Responder {
    add_message_to_session(state, id, req).await
}

pub async fn add_message_to_session(
    state: web::Data<ServerState>,
    id: web::Path<String>,
    req: web::Json<AddMessageRequest>,
) -> impl Responder {
    if let Err(errors) = validate_session_id(&id) {
        return errors.to_response();
    }

    let mut validator = RequestValidator::new();
    if let Some(ref role) = req.role {
        validator.validate_enum("role", role, &["user", "assistant", "system"]);
    }
    validator.validate_required_string("content", Some(&req.content));
    if let Err(errors) = validator.validate() {
        return errors.to_response();
    }

    match state.storage.load_session(&id).await {
        Ok(Some(mut session)) => {
            let role = parse_role(req.role.clone());
            let message = Message::new(role, req.content.clone());
            session.add_message(message);
            let _ = CheckpointManager::new().create(&session, "Message added");

            match state.storage.save_session(&session).await {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                    "session_id": session.id.to_string(),
                    "message_count": session.messages.len(),
                })),
                Err(e) => json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "storage_error",
                    e.to_string(),
                ),
            }
        }
        Ok(None) => json_error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            format!("Session not found: {}", id.as_str()),
        ),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        ),
    }
}

pub async fn run_command_in_session(
    state: web::Data<ServerState>,
    id: web::Path<String>,
    req: web::Json<CommandRequest>,
) -> impl Responder {
    let session_id = id.into_inner();
    if let Err(errors) = validate_session_id(&session_id) {
        return errors.to_response();
    }

    let mut validator = RequestValidator::new();
    validator.validate_required_string("command", Some(&req.command));
    validator.validate_optional_string("workdir", req.workdir.as_deref(), 1000);
    if let Some(ref args) = req.args {
        for (i, arg) in args.iter().enumerate() {
            validator.validate_optional_string(&format!("args[{}]", i), Some(arg), 10000);
        }
    }
    if let Err(errors) = validator.validate() {
        return errors.to_response();
    }

    let Some(mut session) = (match state.storage.load_session(&session_id).await {
        Ok(s) => s,
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                e.to_string(),
            )
        }
    }) else {
        return json_error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            format!("Session not found: {session_id}"),
        );
    };

    if req.command.trim().is_empty() {
        return json_error(
            StatusCode::BAD_REQUEST,
            "invalid_command",
            "command cannot be empty",
        );
    }

    let mut command = Command::new(&req.command);
    if let Some(args) = &req.args {
        command.args(args);
    }
    if let Some(workdir) = &req.workdir {
        command.current_dir(workdir);
    }

    match command.output().await {
        Ok(output) => {
            let status = output.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            session.add_message(Message::assistant(format!(
                "Command `{}` finished with status {}\nstdout:\n{}\nstderr:\n{}",
                req.command, status, stdout, stderr
            )));

            if let Err(e) = state.storage.save_session(&session).await {
                return json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "storage_error",
                    e.to_string(),
                );
            }

            HttpResponse::Ok().json(serde_json::json!({
                "session_id": session.id.to_string(),
                "command": req.command,
                "status": status,
                "stdout": stdout,
                "stderr": stderr,
            }))
        }
        Err(e) => json_error(
            StatusCode::BAD_REQUEST,
            "command_execution_failed",
            e.to_string(),
        ),
    }
}

pub async fn list_messages(
    state: web::Data<ServerState>,
    id: web::Path<String>,
    params: web::Query<PaginationParams>,
) -> impl Responder {
    if let Err(errors) = validate_session_id(&id) {
        return errors.to_response();
    }

    let (limit, _offset) = validate_pagination(params.limit, params.offset);
    let offset = params.offset.unwrap_or(0);

    let session_id = id.into_inner();
    match state.storage.load_session(&session_id).await {
        Ok(Some(session)) => {
            let total = session.messages.len();
            let messages = session
                .messages
                .into_iter()
                .skip(offset)
                .take(limit)
                .collect::<Vec<_>>();

            HttpResponse::Ok().json(serde_json::json!({
                "items": messages,
                "limit": limit,
                "offset": offset,
                "total": total,
            }))
        }
        Ok(None) => json_error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            format!("Session not found: {session_id}"),
        ),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        ),
    }
}

pub async fn get_message(
    state: web::Data<ServerState>,
    path: web::Path<(String, usize)>,
) -> impl Responder {
    let (id, index) = path.into_inner();

    if let Err(errors) = validate_session_id(&id) {
        return errors.to_response();
    }

    match state.storage.load_session(&id).await {
        Ok(Some(session)) => {
            if let Some(message) = session.messages.get(index) {
                HttpResponse::Ok().json(message)
            } else {
                json_error(
                    StatusCode::NOT_FOUND,
                    "message_not_found",
                    format!("Message index {index} not found in session {id}"),
                )
            }
        }
        Ok(None) => json_error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            format!("Session not found: {id}"),
        ),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        ),
    }
}

pub async fn get_session_diff(
    state: web::Data<ServerState>,
    id: web::Path<String>,
) -> impl Responder {
    let session_id = id.into_inner();
    if let Err(errors) = validate_session_id(&session_id) {
        return errors.to_response();
    }
    match state.storage.load_session(&session_id).await {
        Ok(Some(session)) => {
            if session.messages.len() < 2 {
                return HttpResponse::Ok().json(serde_json::json!({
                    "session_id": session_id,
                    "has_diff": false,
                    "reason": "not enough messages",
                }));
            }

            let previous = &session.messages[session.messages.len() - 2].content;
            let current = &session.messages[session.messages.len() - 1].content;
            let prev_lines = previous.lines().collect::<Vec<_>>();
            let curr_lines = current.lines().collect::<Vec<_>>();
            let added = curr_lines
                .iter()
                .filter(|line| !prev_lines.contains(line))
                .count();
            let removed = prev_lines
                .iter()
                .filter(|line| !curr_lines.contains(line))
                .count();

            HttpResponse::Ok().json(serde_json::json!({
                "session_id": session_id,
                "has_diff": true,
                "added_lines": added,
                "removed_lines": removed,
                "from_index": session.messages.len() - 2,
                "to_index": session.messages.len() - 1,
            }))
        }
        Ok(None) => json_error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            format!("Session not found: {session_id}"),
        ),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        ),
    }
}

pub async fn list_session_snapshots(
    state: web::Data<ServerState>,
    id: web::Path<String>,
) -> impl Responder {
    let session_id = id.into_inner();
    if let Err(errors) = validate_session_id(&session_id) {
        return errors.to_response();
    }
    let uuid = match to_session_uuid(&session_id) {
        Ok(uuid) => uuid,
        Err(resp) => return resp,
    };

    match state.storage.load_session(&session_id).await {
        Ok(Some(_)) => match CheckpointManager::new().list(&uuid) {
            Ok(snapshots) => HttpResponse::Ok().json(serde_json::json!({
                "session_id": session_id,
                "items": snapshots,
                "count": snapshots.len(),
            })),
            Err(e) => json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "checkpoint_error",
                e.to_string(),
            ),
        },
        Ok(None) => json_error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            format!("Session not found: {session_id}"),
        ),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        ),
    }
}

pub async fn revert_session_to_checkpoint(
    state: web::Data<ServerState>,
    id: web::Path<String>,
    req: web::Json<RevertRequest>,
) -> impl Responder {
    let session_id = id.into_inner();
    if let Err(errors) = validate_session_id(&session_id) {
        return errors.to_response();
    }

    let mut validator = RequestValidator::new();
    validator.validate_required_number("sequence_number", Some(req.sequence_number), 0, 100000);
    if let Err(errors) = validator.validate() {
        return errors.to_response();
    }

    let uuid = match to_session_uuid(&session_id) {
        Ok(uuid) => uuid,
        Err(resp) => return resp,
    };

    match CheckpointManager::new().load(&uuid, req.sequence_number) {
        Ok(restored) => {
            if let Err(e) = state.storage.save_session(&restored).await {
                return json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "storage_error",
                    e.to_string(),
                );
            }
            HttpResponse::Ok().json(serde_json::json!({
                "session_id": session_id,
                "reverted_to": req.sequence_number,
                "message_count": restored.messages.len(),
            }))
        }
        Err(e) => json_error(StatusCode::BAD_REQUEST, "revert_failed", e.to_string()),
    }
}

pub async fn share_session(
    state: web::Data<ServerState>,
    id: web::Path<String>,
    req: Option<web::Json<ShareRequest>>,
) -> impl Responder {
    let session_id = id.into_inner();
    if let Err(errors) = validate_session_id(&session_id) {
        return errors.to_response();
    }

    match state.storage.load_session(&session_id).await {
        Ok(Some(mut session)) => {
            if let Some(mode) = req.as_ref().and_then(|r| r.mode.clone()) {
                session.set_share_mode(mode);
            }
            session.set_share_expiry(req.as_ref().and_then(|r| r.expires_at));

            match session.generate_share_link() {
                Ok(share_url) => {
                    if let Err(e) = state.storage.save_session(&session).await {
                        return json_error(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "storage_error",
                            e.to_string(),
                        );
                    }

                    HttpResponse::Ok().json(serde_json::json!({
                        "session_id": session_id,
                        "shared_id": session.get_share_id(),
                        "share_url": share_url,
                        "share_mode": session.share_mode,
                        "expires_at": session.share_expires_at,
                    }))
                }
                Err(e) => json_error(StatusCode::BAD_REQUEST, "share_error", e.to_string()),
            }
        }
        Ok(None) => json_error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            format!("Session not found: {session_id}"),
        ),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        ),
    }
}

pub async fn get_shared_session(
    state: web::Data<ServerState>,
    shared_id: web::Path<String>,
) -> impl Responder {
    let mut validator = RequestValidator::new();
    validator.validate_required_string("shared_id", Some(&shared_id));
    if let Err(errors) = validator.validate() {
        return errors.to_response();
    }
    let all = match state.storage.list_sessions(500, 0).await {
        Ok(s) => s,
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                e.to_string(),
            )
        }
    };

    for item in all {
        let sid = item.id.to_string();
        if let Ok(Some(session)) = state.storage.load_session(&sid).await {
            if session.get_share_id() == Some(shared_id.as_str()) {
                return HttpResponse::Ok().json(serde_json::json!({
                    "id": session.id,
                    "created_at": session.created_at,
                    "updated_at": session.updated_at,
                    "messages": session.messages,
                    "read_only": true,
                }));
            }
        }
    }

    json_error(
        StatusCode::NOT_FOUND,
        "shared_session_not_found",
        format!("Shared session not found: {}", shared_id.as_str()),
    )
}

pub async fn remove_share_session(
    state: web::Data<ServerState>,
    id: web::Path<String>,
) -> impl Responder {
    let session_id = id.into_inner();
    if let Err(errors) = validate_session_id(&session_id) {
        return errors.to_response();
    }
    match state.storage.load_session(&session_id).await {
        Ok(Some(mut session)) => {
            session.set_share_mode(ShareMode::Disabled);
            if let Err(e) = state.storage.save_session(&session).await {
                return json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "storage_error",
                    e.to_string(),
                );
            }
            HttpResponse::NoContent().finish()
        }
        Ok(None) => json_error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            format!("Session not found: {session_id}"),
        ),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        ),
    }
}

pub async fn abort_session(state: web::Data<ServerState>, id: web::Path<String>) -> impl Responder {
    let session_id = id.into_inner();
    if let Err(errors) = validate_session_id(&session_id) {
        return errors.to_response();
    }
    match state.storage.load_session(&session_id).await {
        Ok(Some(mut session)) => {
            session.state = opencode_core::session_state::SessionState::Aborted;
            match state.storage.save_session(&session).await {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                    "session_id": session_id,
                    "status": "aborted",
                    "message_count": session.messages.len(),
                })),
                Err(e) => json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "storage_error",
                    e.to_string(),
                ),
            }
        }
        Ok(None) => json_error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            format!("Session not found: {}", session_id),
        ),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        ),
    }
}

fn req_id_to_permission(req_id: &str) -> Permission {
    let lower = req_id.to_lowercase();
    if lower.contains("file_read") || lower.contains("read") {
        Permission::FileRead
    } else if lower.contains("file_write") || lower.contains("write") {
        Permission::FileWrite
    } else if lower.contains("file_delete") || lower.contains("delete") {
        Permission::FileDelete
    } else if lower.contains("bash") || lower.contains("execute") {
        Permission::BashExecute
    } else if lower.contains("network") || lower.contains("external") {
        Permission::NetworkAccess
    } else {
        Permission::FileRead
    }
}

fn derive_tool_name_from_req_id(req_id: &str) -> String {
    let lower = req_id.to_lowercase();
    if lower.contains("file_read") || lower.contains("read") {
        "read".to_string()
    } else if lower.contains("file_write") || lower.contains("write") {
        "write".to_string()
    } else if lower.contains("file_delete") || lower.contains("delete") {
        "delete".to_string()
    } else if lower.contains("bash") || lower.contains("execute") {
        "bash".to_string()
    } else if lower.contains("network") || lower.contains("external") {
        "network".to_string()
    } else if lower.contains("grep") {
        "grep".to_string()
    } else if lower.contains("glob") {
        "glob".to_string()
    } else if lower.contains("edit") {
        "edit".to_string()
    } else {
        req_id.to_string()
    }
}

pub async fn permission_reply(
    state: web::Data<ServerState>,
    path: web::Path<(String, String)>,
    body: web::Json<PermissionReplyRequest>,
) -> impl Responder {
    let (session_id, req_id) = path.into_inner();

    let mut validator = RequestValidator::new();
    validator.validate_required_string("session_id", Some(&session_id));
    validator.validate_required_string("req_id", Some(&req_id));
    validator.validate_required_string("decision", Some(&body.decision));
    validator.validate_enum(
        "decision",
        &body.decision.to_lowercase(),
        &["allow", "deny"],
    );
    if let Err(errors) = validator.validate() {
        return errors.to_response();
    }

    let decision = body.decision.to_lowercase();
    if decision != "allow" && decision != "deny" {
        return bad_request("decision must be 'allow' or 'deny'");
    }

    let permission = req_id_to_permission(&req_id);

    if let Ok(mut pm) = state.permission_manager.write() {
        match decision.as_str() {
            "allow" => {
                pm.grant(permission.clone());
                tracing::info!(
                    "Permission granted: session={}, req={}, permission={:?}",
                    session_id,
                    req_id,
                    permission
                );
            }
            "deny" => {
                pm.revoke(&permission);
                tracing::info!(
                    "Permission denied: session={}, req={}, permission={:?}",
                    session_id,
                    req_id,
                    permission
                );
            }
            _ => {}
        }
    }

    if let Ok(mut aq) = state.approval_queue.write() {
        if let Ok(approval_id) = Uuid::parse_str(&req_id) {
            match decision.as_str() {
                "allow" => {
                    if let Some(approved) = aq.approve(approval_id) {
                        tracing::info!(
                            "ApprovalQueue updated: approved tool={} for session={}",
                            approved.tool_name,
                            session_id
                        );
                    }
                }
                "deny" => {
                    if aq.reject(approval_id) {
                        tracing::info!(
                            "ApprovalQueue updated: rejected req_id={} for session={}",
                            req_id,
                            session_id
                        );
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(ref audit_log) = state.audit_log {
        let tool_name = derive_tool_name_from_req_id(&req_id);
        let audit_decision = match decision.as_str() {
            "allow" => AuditDecision::Allow,
            "deny" => AuditDecision::Deny,
            _ => AuditDecision::Ask,
        };
        if let Err(e) = audit_log.record_decision(AuditEntry {
            timestamp: Utc::now(),
            tool_name,
            decision: audit_decision,
            session_id: session_id.clone(),
            user_response: Some(decision.clone()),
        }) {
            tracing::error!("Failed to record audit log: {}", e);
        }
    }

    match decision.as_str() {
        "allow" => HttpResponse::Ok().json(serde_json::json!({
            "status": "ok",
            "session_id": session_id,
            "request_id": req_id,
            "decision": decision
        })),
        "deny" => permission_denied_error(format!(
            "Permission denied for session={}, request={}",
            session_id, req_id
        )),
        _ => HttpResponse::Ok().json(serde_json::json!({
            "status": "ok",
            "session_id": session_id,
            "request_id": req_id,
            "decision": decision
        })),
    }
}

pub async fn summarize_session(
    state: web::Data<ServerState>,
    id: web::Path<String>,
) -> impl Responder {
    let session_id = id.into_inner();
    if let Err(errors) = validate_session_id(&session_id) {
        return errors.to_response();
    }
    match state.storage.load_session(&session_id).await {
        Ok(Some(mut session)) => match SummaryGenerator::summarize_session(&session.messages) {
            Ok(summary) => {
                let created_at = Utc::now();
                session.store_summary_metadata(summary.clone(), created_at);

                if let Err(e) = state.storage.save_session(&session).await {
                    return json_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "storage_error",
                        e.to_string(),
                    );
                }

                HttpResponse::Ok().json(serde_json::json!({
                    "summary": summary,
                    "created_at": created_at,
                }))
            }
            Err(e) => json_error(StatusCode::BAD_REQUEST, "summary_error", e.to_string()),
        },
        Ok(None) => json_error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            format!("Session not found: {session_id}"),
        ),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        ),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(list_sessions));
    cfg.route("", web::post().to(create_session));
    cfg.route("/{id}/fork", web::post().to(fork_session));
    cfg.route("/{id}/prompt", web::post().to(prompt_session));
    cfg.route("/{id}/command", web::post().to(run_command_in_session));
    cfg.route("/{id}/abort", web::post().to(abort_session));
    cfg.route(
        "/{id}/permissions/{req_id}/reply",
        web::post().to(permission_reply),
    );
    cfg.route("/{id}/messages", web::get().to(list_messages));
    cfg.route("/{id}/messages", web::post().to(add_message_to_session));
    cfg.route("/{id}/messages/{msg_index}", web::get().to(get_message));
    cfg.route("/{id}/diff", web::get().to(get_session_diff));
    cfg.route("/{id}/snapshots", web::get().to(list_session_snapshots));
    cfg.route("/{id}/revert", web::post().to(revert_session_to_checkpoint));
    cfg.route("/{id}/share", web::post().to(share_session));
    cfg.route("/{id}/share", web::delete().to(remove_share_session));
    cfg.route("/{id}/summarize", web::post().to(summarize_session));
    cfg.route("/{id}", web::get().to(get_session));
    cfg.route("/{id}", web::delete().to(delete_session));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_role_defaults_to_user() {
        assert_eq!(parse_role(None), Role::User);
        assert_eq!(parse_role(Some("unknown".to_string())), Role::User);
    }

    #[test]
    fn parse_role_handles_known_roles() {
        assert_eq!(parse_role(Some("assistant".to_string())), Role::Assistant);
        assert_eq!(parse_role(Some("system".to_string())), Role::System);
    }

    #[test]
    fn to_session_uuid_rejects_bad_id() {
        let result = to_session_uuid("not-a-uuid");
        assert!(result.is_err());
    }
}
