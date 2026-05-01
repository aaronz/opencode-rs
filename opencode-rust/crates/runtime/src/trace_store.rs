use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::errors::RuntimeFacadeError;
use crate::types::TraceId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageSummary {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeFacadeTrace {
    pub id: TraceId,
    pub session_id: Uuid,
    pub turn_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub token_usage: Option<TokenUsageSummary>,
    pub tool_call_count: usize,
    pub success: bool,
    pub error: Option<String>,
    #[serde(default)]
    pub events: Vec<TraceEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum TraceEvent {
    LlmRequestStarted {
        messages_count: usize,
    },
    LlmTokenStreamed {
        delta: String,
    },
    LlmResponseCompleted {
        content: String,
        token_usage: Option<TokenUsageSummary>,
    },
    LlmError {
        error: String,
    },
    ToolCallStarted {
        tool_name: String,
        arguments: Value,
    },
    ToolCallCompleted {
        tool_name: String,
        result: String,
        duration_ms: u64,
    },
    ToolCallFailed {
        tool_name: String,
        error: String,
        duration_ms: u64,
    },
    ContextBuilt {
        input_tokens: u64,
    },
    UserInput {
        content: String,
    },
    MessageAdded {
        role: String,
        content: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeFacadeTraceSummary {
    pub id: TraceId,
    pub session_id: Uuid,
    pub turn_id: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub tool_call_count: usize,
    pub success: bool,
    pub error: Option<String>,
}

impl From<&RuntimeFacadeTrace> for RuntimeFacadeTraceSummary {
    fn from(trace: &RuntimeFacadeTrace) -> Self {
        Self {
            id: trace.id,
            session_id: trace.session_id,
            turn_id: trace.turn_id,
            started_at: trace.started_at,
            ended_at: trace.ended_at,
            tool_call_count: trace.tool_call_count,
            success: trace.success,
            error: trace.error.clone(),
        }
    }
}

#[derive(Default, Clone)]
pub struct RuntimeFacadeTraceStore {
    traces: Arc<RwLock<HashMap<TraceId, RuntimeFacadeTrace>>>,
    by_session: Arc<RwLock<HashMap<Uuid, Vec<TraceId>>>>,
}

impl RuntimeFacadeTraceStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn begin_trace(
        &self,
        session_id: Uuid,
        turn_id: Option<Uuid>,
        task_id: Option<Uuid>,
    ) -> Result<TraceId, RuntimeFacadeError> {
        let trace_id = TraceId::new();
        let trace = RuntimeFacadeTrace {
            id: trace_id,
            session_id,
            turn_id,
            task_id,
            started_at: Utc::now(),
            ended_at: None,
            provider: None,
            model: None,
            token_usage: None,
            tool_call_count: 0,
            success: false,
            error: None,
            events: Vec::new(),
        };

        {
            let mut traces = self.traces.write().await;
            traces.insert(trace_id, trace);
        }
        {
            let mut by_session = self.by_session.write().await;
            by_session.entry(session_id).or_default().push(trace_id);
        }

        Ok(trace_id)
    }

    pub async fn end_trace(
        &self,
        trace_id: TraceId,
        success: bool,
        error: Option<String>,
    ) -> Result<(), RuntimeFacadeError> {
        let mut traces = self.traces.write().await;
        if let Some(trace) = traces.get_mut(&trace_id) {
            trace.ended_at = Some(Utc::now());
            trace.success = success;
            trace.error = error;
        }
        Ok(())
    }

    pub async fn record_tool_call(
        &self,
        trace_id: TraceId,
        _tool_name: &str,
    ) -> Result<(), RuntimeFacadeError> {
        let mut traces = self.traces.write().await;
        if let Some(trace) = traces.get_mut(&trace_id) {
            trace.tool_call_count += 1;
        }
        Ok(())
    }

    pub async fn record_event(
        &self,
        trace_id: TraceId,
        event: TraceEvent,
    ) -> Result<(), RuntimeFacadeError> {
        let mut traces = self.traces.write().await;
        if let Some(trace) = traces.get_mut(&trace_id) {
            trace.events.push(event);
        }
        Ok(())
    }

    pub async fn list_session_traces(
        &self,
        session_id: &Uuid,
    ) -> Result<Vec<RuntimeFacadeTraceSummary>, RuntimeFacadeError> {
        let by_session = self.by_session.read().await;
        let trace_ids = by_session.get(session_id).cloned().unwrap_or_default();
        drop(by_session);

        let traces = self.traces.read().await;
        let summaries: Vec<RuntimeFacadeTraceSummary> = trace_ids
            .iter()
            .filter_map(|id| traces.get(id))
            .map(RuntimeFacadeTraceSummary::from)
            .collect();

        Ok(summaries)
    }

    pub async fn get_trace(
        &self,
        trace_id: &TraceId,
    ) -> Result<Option<RuntimeFacadeTrace>, RuntimeFacadeError> {
        let traces = self.traces.read().await;
        Ok(traces.get(trace_id).cloned())
    }

    pub async fn export_jsonl(&self, trace_id: &TraceId) -> Result<String, RuntimeFacadeError> {
        let traces = self.traces.read().await;
        if let Some(trace) = traces.get(trace_id) {
            let mut jsonl = String::new();
            for event in &trace.events {
                let line = serde_json::to_string(event)
                    .map_err(|e| RuntimeFacadeError::InvalidConfiguration(e.to_string()))?;
                jsonl.push_str(&line);
                jsonl.push('\n');
            }
            Ok(jsonl)
        } else {
            Err(RuntimeFacadeError::InvalidConfiguration(
                "Trace not found".to_string(),
            ))
        }
    }

    pub async fn import_jsonl(
        &self,
        session_id: Uuid,
        jsonl: &str,
    ) -> Result<TraceId, RuntimeFacadeError> {
        let trace_id = TraceId::new();
        let mut events = Vec::new();

        for line in jsonl.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let event: TraceEvent = serde_json::from_str(line).map_err(|e| {
                RuntimeFacadeError::InvalidConfiguration(format!("Invalid JSONL: {}", e))
            })?;
            events.push(event);
        }

        let trace = RuntimeFacadeTrace {
            id: trace_id,
            session_id,
            turn_id: None,
            task_id: None,
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            provider: None,
            model: None,
            token_usage: None,
            tool_call_count: events
                .iter()
                .filter(|e| matches!(e, TraceEvent::ToolCallCompleted { .. }))
                .count(),
            success: true,
            error: None,
            events,
        };

        {
            let mut traces = self.traces.write().await;
            traces.insert(trace_id, trace);
        }
        {
            let mut by_session = self.by_session.write().await;
            by_session.entry(session_id).or_default().push(trace_id);
        }

        Ok(trace_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trace_with_events() {
        let store = RuntimeFacadeTraceStore::new();
        let session_id = Uuid::new_v4();
        let trace_id = store.begin_trace(session_id, None, None).await.unwrap();

        store
            .record_event(
                trace_id,
                TraceEvent::UserInput {
                    content: "Hello".to_string(),
                },
            )
            .await
            .unwrap();

        store
            .record_event(
                trace_id,
                TraceEvent::LlmRequestStarted { messages_count: 1 },
            )
            .await
            .unwrap();

        let trace = store.get_trace(&trace_id).await.unwrap().unwrap();
        assert_eq!(trace.events.len(), 2);
    }

    #[tokio::test]
    async fn test_export_import_jsonl() {
        let store = RuntimeFacadeTraceStore::new();
        let session_id = Uuid::new_v4();
        let trace_id = store.begin_trace(session_id, None, None).await.unwrap();

        store
            .record_event(
                trace_id,
                TraceEvent::UserInput {
                    content: "Test input".to_string(),
                },
            )
            .await
            .unwrap();

        let jsonl = store.export_jsonl(&trace_id).await.unwrap();
        assert!(jsonl.contains("UserInput"));

        let store2 = RuntimeFacadeTraceStore::new();
        let imported_id = store2.import_jsonl(session_id, &jsonl).await.unwrap();
        let imported = store2.get_trace(&imported_id).await.unwrap().unwrap();
        assert_eq!(imported.events.len(), 1);
    }
}
