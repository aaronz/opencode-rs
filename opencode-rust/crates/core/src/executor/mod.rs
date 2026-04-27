use crate::session::ToolInvocationRecord;
use crate::tool::{ToolCall, ToolRegistry, ToolResult};
use chrono::Utc;
use uuid::Uuid;

pub struct AgentExecutor {
    registry: ToolRegistry,
    max_iterations: usize,
}

impl AgentExecutor {
    pub fn new(registry: ToolRegistry) -> Self {
        Self {
            registry,
            max_iterations: 10,
        }
    }

    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    pub fn execute_tool_call(&self, call: ToolCall) -> ToolResult {
        let started_at = Utc::now();
        let tool_name = call.name.clone();

        if self.registry.is_disabled(&call.name) {
            return ToolResult {
                id: Uuid::new_v4(),
                tool_name,
                success: false,
                result: None,
                error: Some(format!("Tool '{}' is disabled", call.name)),
                started_at,
                completed_at: Utc::now(),
            };
        }

        if self.registry.requires_approval(&call.name) {
            return ToolResult {
                id: Uuid::new_v4(),
                tool_name,
                success: false,
                result: None,
                error: Some(format!(
                    "Tool '{}' requires approval before execution. Use the permission approval flow.",
                    call.name
                )),
                started_at,
                completed_at: Utc::now(),
            };
        }

        let executor = match self.registry.get_executor(&call.name) {
            Some(exec) => exec,
            None => {
                return ToolResult {
                    id: Uuid::new_v4(),
                    tool_name,
                    success: false,
                    result: None,
                    error: Some(format!("Tool '{}' not found", call.name)),
                    started_at,
                    completed_at: Utc::now(),
                };
            }
        };

        match executor(call.arguments) {
            Ok(result) => ToolResult::success(tool_name, result),
            Err(error) => ToolResult::failure(tool_name, error),
        }
    }

    pub fn execute_tool_calls(&self, calls: Vec<ToolCall>) -> Vec<ToolResult> {
        calls
            .into_iter()
            .map(|call| self.execute_tool_call(call))
            .collect()
    }

    pub fn get_available_tools(&self) -> Vec<&crate::tool::ToolDefinition> {
        self.registry.get_all()
    }

    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    pub fn create_invocation_record(&self, call: &ToolCall) -> ToolInvocationRecord {
        let span = tracing::info_span!("tool_invocation", tool = %call.name);
        let _enter = span.enter();

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(call.arguments.to_string().as_bytes());
        let args_hash = format!("{:x}", hasher.finalize());

        tracing::info!(tool = %call.name, args_hash = %args_hash, "Tool invocation started");

        ToolInvocationRecord {
            id: Uuid::new_v4(),
            tool_name: call.name.clone(),
            arguments: call.arguments.clone(),
            args_hash,
            result: None,
            started_at: Utc::now(),
            completed_at: None,
            latency_ms: None,
        }
    }

    pub fn update_invocation_record(
        &self,
        mut record: ToolInvocationRecord,
        result: &ToolResult,
    ) -> ToolInvocationRecord {
        record.result = Some(if result.success {
            result.result.clone().unwrap_or_default()
        } else {
            result.error.clone().unwrap_or_default()
        });
        record.completed_at = Some(Utc::now());
        let latency = record
            .completed_at
            .map(|t| (t - record.started_at).num_milliseconds().max(0) as u64);
        record.latency_ms = latency;

        tracing::info!(
            tool = %record.tool_name,
            latency_ms = record.latency_ms,
            success = result.success,
            "Tool invocation completed"
        );

        record
    }
}

pub fn build_default_executor() -> AgentExecutor {
    AgentExecutor::new(crate::tool::build_default_registry())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::ToolDefinition;
    use std::sync::Arc;

    #[test]
    fn test_execute_tool_success() {
        let mut registry = ToolRegistry::new();
        registry.register(
            ToolDefinition {
                name: "echo".to_string(),
                description: "Echo input".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            Arc::new(|_| Ok("hello".to_string())),
        );

        let executor = AgentExecutor::new(registry);

        let call = ToolCall {
            id: "1".to_string(),
            name: "echo".to_string(),
            arguments: serde_json::json!({}),
        };

        let result = executor.execute_tool_call(call);
        assert!(result.success);
        assert_eq!(result.result, Some("hello".to_string()));
    }

    #[test]
    fn test_execute_tool_not_found() {
        let registry = ToolRegistry::new();
        let executor = AgentExecutor::new(registry);

        let call = ToolCall {
            id: "1".to_string(),
            name: "nonexistent".to_string(),
            arguments: serde_json::json!({}),
        };

        let result = executor.execute_tool_call(call);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("not found"));
    }

    #[test]
    fn test_execute_tool_disabled() {
        let mut registry = ToolRegistry::new();
        registry.register(
            ToolDefinition {
                name: "echo".to_string(),
                description: "Echo input".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            Arc::new(|_| Ok("hello".to_string())),
        );
        registry.set_disabled(std::collections::HashSet::from(["echo".to_string()]));

        let executor = AgentExecutor::new(registry);
        let call = ToolCall {
            id: "1".to_string(),
            name: "echo".to_string(),
            arguments: serde_json::json!({}),
        };

        let result = executor.execute_tool_call(call);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("disabled"));
    }

    #[test]
    fn test_execute_multiple_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(
            ToolDefinition {
                name: "echo".to_string(),
                description: "Echo input".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            Arc::new(|_| Ok("hello".to_string())),
        );

        let executor = AgentExecutor::new(registry);

        let calls = vec![
            ToolCall {
                id: "1".to_string(),
                name: "echo".to_string(),
                arguments: serde_json::json!({}),
            },
            ToolCall {
                id: "2".to_string(),
                name: "echo".to_string(),
                arguments: serde_json::json!({}),
            },
        ];

        let results = executor.execute_tool_calls(calls);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.success));
    }

    #[test]
    fn test_execute_tool_requires_approval() {
        let mut registry = ToolRegistry::new();
        registry.register(
            ToolDefinition {
                name: "ask_tool".to_string(),
                description: "A tool that requires approval".to_string(),
                parameters: vec![],
                requires_approval: true,
            },
            Arc::new(|_| Ok("should not execute".to_string())),
        );

        let executor = AgentExecutor::new(registry);

        let call = ToolCall {
            id: "1".to_string(),
            name: "ask_tool".to_string(),
            arguments: serde_json::json!({}),
        };

        let result = executor.execute_tool_call(call);
        assert!(!result.success);
        let error = result.error.expect("should have error message");
        assert!(error.contains("requires approval"));
        assert!(error.contains("ask_tool"));
    }

    #[test]
    fn test_execute_tool_allows_without_approval() {
        let mut registry = ToolRegistry::new();
        registry.register(
            ToolDefinition {
                name: "auto_tool".to_string(),
                description: "A tool that auto-approves".to_string(),
                parameters: vec![],
                requires_approval: false,
            },
            Arc::new(|_| Ok("executed successfully".to_string())),
        );

        let executor = AgentExecutor::new(registry);

        let call = ToolCall {
            id: "1".to_string(),
            name: "auto_tool".to_string(),
            arguments: serde_json::json!({}),
        };

        let result = executor.execute_tool_call(call);
        assert!(result.success);
        assert_eq!(result.result, Some("executed successfully".to_string()));
    }

    #[test]
    fn test_agent_executor_with_max_iterations() {
        let registry = ToolRegistry::new();
        let executor = AgentExecutor::new(registry).with_max_iterations(20);
        assert_eq!(executor.max_iterations, 20);
    }

    #[test]
    fn test_get_available_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(
            ToolDefinition {
                name: "tool1".to_string(),
                description: "First tool".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            Arc::new(|_| Ok("result1".to_string())),
        );
        registry.register(
            ToolDefinition {
                name: "tool2".to_string(),
                description: "Second tool".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            Arc::new(|_| Ok("result2".to_string())),
        );

        let executor = AgentExecutor::new(registry);
        let tools = executor.get_available_tools();

        assert_eq!(tools.len(), 2);
        assert!(tools.iter().any(|t| t.name == "tool1"));
        assert!(tools.iter().any(|t| t.name == "tool2"));
    }

    #[test]
    fn test_executor_registry() {
        let registry = ToolRegistry::new();
        let executor = AgentExecutor::new(registry);

        assert!(executor.registry().get_all().is_empty());
    }

    #[test]
    fn test_create_invocation_record() {
        let registry = ToolRegistry::new();
        let executor = AgentExecutor::new(registry);

        let call = ToolCall {
            id: "1".to_string(),
            name: "test_tool".to_string(),
            arguments: serde_json::json!({"arg1": "value1"}),
        };

        let record = executor.create_invocation_record(&call);

        assert_eq!(record.tool_name, "test_tool");
        assert_eq!(record.arguments, serde_json::json!({"arg1": "value1"}));
        assert!(record.result.is_none());
        assert!(record.completed_at.is_none());
        assert!(record.latency_ms.is_none());
    }

    #[test]
    fn test_update_invocation_record_success() {
        let registry = ToolRegistry::new();
        let executor = AgentExecutor::new(registry);

        let call = ToolCall {
            id: "1".to_string(),
            name: "test_tool".to_string(),
            arguments: serde_json::json!({}),
        };

        let record = executor.create_invocation_record(&call);

        let result = ToolResult {
            id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            success: true,
            result: Some("success result".to_string()),
            error: None,
            started_at: Utc::now(),
            completed_at: Utc::now(),
        };

        let updated = executor.update_invocation_record(record, &result);

        assert_eq!(updated.result, Some("success result".to_string()));
        assert!(updated.completed_at.is_some());
        assert!(updated.latency_ms.is_some());
    }

    #[test]
    fn test_update_invocation_record_failure() {
        let registry = ToolRegistry::new();
        let executor = AgentExecutor::new(registry);

        let call = ToolCall {
            id: "1".to_string(),
            name: "test_tool".to_string(),
            arguments: serde_json::json!({}),
        };

        let record = executor.create_invocation_record(&call);

        let result = ToolResult {
            id: Uuid::new_v4(),
            tool_name: "test_tool".to_string(),
            success: false,
            result: None,
            error: Some("error message".to_string()),
            started_at: Utc::now(),
            completed_at: Utc::now(),
        };

        let updated = executor.update_invocation_record(record, &result);

        assert_eq!(updated.result, Some("error message".to_string()));
        assert!(updated.completed_at.is_some());
    }

    #[test]
    fn test_build_default_executor() {
        let executor = build_default_executor();
        let tools = executor.get_available_tools();
        assert!(!tools.is_empty());
    }
}