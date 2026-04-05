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
        ToolInvocationRecord {
            id: Uuid::new_v4(),
            tool_name: call.name.clone(),
            arguments: call.arguments.clone(),
            result: None,
            started_at: Utc::now(),
            completed_at: None,
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
}
