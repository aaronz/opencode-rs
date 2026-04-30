use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{error, info, warn};

use super::types::{HookAction, HookDefinition, HookExecution, HookFailurePolicy, HookTrigger};
use super::{HookId, HookPoint};
use crate::events::DomainEvent;

#[derive(Debug, Clone)]
pub enum HookEvent {
    SessionStart { session_id: String },
    SessionEnd { session_id: String },
    ContextBuild { context_id: String },
    LlmRequest { request_id: String },
    LlmResponse { request_id: String },
    ToolExecution { tool_name: String },
    FilePatch { path: String },
    CommandCompleted { command: String },
    Validation { command: String },
    TaskFailed { task_id: String },
    SessionPersist { session_id: String },
}

pub struct HookEngine {
    hooks: Arc<RwLock<Vec<HookDefinition>>>,
    executions: Arc<RwLock<Vec<HookExecution>>>,
}

impl Default for HookEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl HookEngine {
    pub fn new() -> Self {
        Self {
            hooks: Arc::new(RwLock::new(Vec::new())),
            executions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn register(&self, hook: HookDefinition) {
        let mut hooks = self.hooks.write().await;
        hooks.push(hook);
    }

    pub async fn unregister(&self, hook_id: &HookId) -> bool {
        let mut hooks = self.hooks.write().await;
        let initial_len = hooks.len();
        hooks.retain(|h| &h.id != hook_id);
        hooks.len() < initial_len
    }

    pub async fn get_hooks_for_point(&self, point: HookPoint) -> Vec<HookDefinition> {
        let hooks = self.hooks.read().await;
        hooks
            .iter()
            .filter(|h| {
                if !h.enabled {
                    return false;
                }
                match &h.trigger {
                    HookTrigger::HookPoint(hp) => *hp == point,
                    HookTrigger::Event(_) | HookTrigger::Command(_) => false,
                }
            })
            .cloned()
            .collect()
    }

    pub async fn get_hooks_for_event(&self, event_name: &str) -> Vec<HookDefinition> {
        let hooks = self.hooks.read().await;
        hooks
            .iter()
            .filter(|h| {
                if !h.enabled {
                    return false;
                }
                match &h.trigger {
                    HookTrigger::Event(e) => e == event_name,
                    _ => false,
                }
            })
            .cloned()
            .collect()
    }

    pub async fn trigger(&self, point: HookPoint, event: HookEvent) -> Vec<HookResult> {
        let hooks = self.get_hooks_for_point(point).await;
        let mut results = Vec::new();

        for hook in hooks {
            let result = self.execute_hook(hook, event.clone()).await;
            results.push(result);
        }

        results
    }

    async fn execute_hook(&self, hook: HookDefinition, _event: HookEvent) -> HookResult {
        let mut execution = HookExecution::new(&hook);
        let timeout = hook.timeout_ms.unwrap_or(30_000);

        let result = tokio::time::timeout(Duration::from_millis(timeout), async {
            self.run_hook_action(&hook.action).await
        })
        .await;

        match result {
            Ok(Ok(output)) => {
                execution.output = Some(output);
                execution.success = true;
                execution.complete();
            }
            Ok(Err(e)) => {
                execution.error = Some(e);
                execution.success = false;
                execution.complete();
                self.handle_failure(&hook, &execution).await;
            }
            Err(_) => {
                execution.error = Some("Hook timed out".to_string());
                execution.success = false;
                execution.complete();
                self.handle_failure(&hook, &execution).await;
            }
        }

        let result = HookResult {
            hook_id: hook.id,
            success: execution.success,
            output: execution.output.clone(),
            error: execution.error.clone(),
        };

        let mut executions = self.executions.write().await;
        executions.push(execution);

        result
    }

    async fn run_hook_action(&self, action: &HookAction) -> Result<String, String> {
        match action {
            HookAction::Log { message, level } => {
                match level.as_str() {
                    "error" => error!("{}", message),
                    "warn" => warn!("{}", message),
                    _ => info!("{}", message),
                }
                Ok(message.clone())
            }
            HookAction::Notify { message } => {
                info!("Hook notification: {}", message);
                Ok(message.clone())
            }
            HookAction::SetEnv { key, value } => {
                std::env::set_var(key, value);
                Ok(format!("Set {}={}", key, value))
            }
            HookAction::RunCommand { command } => {
                let output = tokio::process::Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .output()
                    .await
                    .map_err(|e| e.to_string())?;
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                if output.status.success() {
                    Ok(stdout)
                } else {
                    Err(format!("Command failed: {}\n{}", stdout, stderr))
                }
            }
            HookAction::RunScript {
                script,
                interpreter,
            } => {
                let interpreter = interpreter.as_deref().unwrap_or("sh");
                let output = tokio::process::Command::new(interpreter)
                    .arg("-c")
                    .arg(script)
                    .output()
                    .await
                    .map_err(|e| e.to_string())?;
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                if output.status.success() {
                    Ok(stdout)
                } else {
                    Err(format!("Script failed: {}", stdout))
                }
            }
            HookAction::Block { reason } => Err(format!("Hook blocked: {}", reason)),
        }
    }

    async fn handle_failure(&self, hook: &HookDefinition, execution: &HookExecution) {
        match hook.failure_policy {
            HookFailurePolicy::Log => {
                if let Some(e) = &execution.error {
                    error!("Hook {} failed: {}", hook.name, e);
                }
            }
            HookFailurePolicy::Warn => {
                if let Some(e) = &execution.error {
                    warn!("Hook {} failed: {}", hook.name, e);
                }
            }
            HookFailurePolicy::Error => {
                if let Some(e) = &execution.error {
                    error!("Hook {} failed: {}", hook.name, e);
                }
            }
            HookFailurePolicy::Block => {
                error!("Hook {} blocking execution", hook.name);
            }
        }
    }

    pub async fn from_domain_event(event: &DomainEvent) -> Option<HookEvent> {
        match event {
            DomainEvent::SessionStarted(id) => Some(HookEvent::SessionStart {
                session_id: id.clone(),
            }),
            DomainEvent::SessionEnded(id) => Some(HookEvent::SessionEnd {
                session_id: id.clone(),
            }),
            DomainEvent::ToolCallStarted { .. } => None,
            DomainEvent::ToolCallEnded { .. } => None,
            DomainEvent::LlmRequestStarted { .. } => None,
            DomainEvent::LlmResponseCompleted { .. } => None,
            _ => None,
        }
    }

    pub async fn get_execution_history(&self) -> Vec<HookExecution> {
        let executions = self.executions.read().await;
        executions.clone()
    }

    pub async fn clear_history(&self) {
        let mut executions = self.executions.write().await;
        executions.clear();
    }
}

#[derive(Debug, Clone)]
pub struct HookResult {
    pub hook_id: HookId,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

use std::time::Duration;
