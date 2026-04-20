//! Tool management for OpenCode SDK.
//!
//! Provides types and operations for executing and listing tools.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Tool parameter definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    /// Parameter name.
    pub name: String,

    /// Parameter description.
    pub description: String,

    /// Whether the parameter is required.
    #[serde(default)]
    pub required: bool,

    /// JSON Schema for the parameter.
    #[serde(default)]
    pub schema: serde_json::Value,
}

/// Tool definition from the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name.
    pub name: String,

    /// Tool description.
    pub description: String,

    /// Tool parameters.
    #[serde(default)]
    pub parameters: Vec<ToolParameter>,
}

/// Tool execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Result ID.
    pub id: Uuid,

    /// Name of the executed tool.
    pub tool_name: String,

    /// Whether the execution was successful.
    pub success: bool,

    /// Result content (if successful).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,

    /// Error message (if failed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Execution start time.
    pub started_at: DateTime<Utc>,

    /// Execution completion time.
    pub completed_at: DateTime<Utc>,
}

impl ToolResult {
    /// Creates a successful tool result.
    pub fn success(tool_name: impl Into<String>, result: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tool_name: tool_name.into(),
            success: true,
            result: Some(result.into()),
            error: None,
            started_at: now,
            completed_at: now,
        }
    }

    /// Creates a failed tool result.
    pub fn failure(tool_name: impl Into<String>, error: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tool_name: tool_name.into(),
            success: false,
            result: None,
            error: Some(error.into()),
            started_at: now,
            completed_at: now,
        }
    }

    /// Returns true if the result is successful.
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns the result content or panics if not successful.
    pub fn unwrap_result(&self) -> &str {
        self.result.as_deref().unwrap_or("")
    }

    /// Returns the error message or panics if successful.
    pub fn unwrap_error(&self) -> &str {
        self.error.as_deref().unwrap_or("")
    }
}

/// Tool call request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool name to execute.
    pub name: String,

    /// Tool arguments as JSON.
    #[serde(default)]
    pub arguments: serde_json::Value,
}

impl ToolCall {
    /// Creates a new tool call with the given name and arguments.
    pub fn new(name: impl Into<String>, arguments: impl Into<serde_json::Value>) -> Self {
        Self {
            name: name.into(),
            arguments: arguments.into(),
        }
    }

    /// Creates a tool call with a single argument.
    pub fn with_arg(name: impl Into<String>, key: &str, value: impl Serialize) -> Self {
        let arguments = serde_json::json!({ key: value });
        Self::new(name, arguments)
    }
}

/// Tool execution response from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ToolExecutionResponse {
    /// Execution result ID.
    pub id: Uuid,

    /// Tool name.
    pub tool_name: String,

    /// Whether successful.
    pub success: bool,

    /// Result or error.
    #[serde(default)]
    pub result: Option<String>,

    /// Error message if failed.
    #[serde(default)]
    pub error: Option<String>,
}

/// Tool executor for local tool execution.
pub struct ToolExecutor {
    executor: std::sync::Arc<dyn Fn(serde_json::Value) -> Result<String, String> + Send + Sync>,
}

impl ToolExecutor {
    /// Creates a new tool executor with the given function.
    pub fn new<F>(executor: F) -> Self
    where
        F: Fn(serde_json::Value) -> Result<String, String> + Send + Sync + 'static,
    {
        Self {
            executor: std::sync::Arc::new(executor),
        }
    }

    /// Executes the tool with the given arguments.
    pub fn execute(&self, arguments: serde_json::Value) -> Result<String, String> {
        (self.executor)(arguments)
    }
}

impl Clone for ToolExecutor {
    fn clone(&self) -> Self {
        Self {
            executor: self.executor.clone(),
        }
    }
}

/// Local tool registry for registering and executing custom tools.
///
/// # Example
///
/// ```rust
/// use opencode_sdk::tools::{ToolRegistry, ToolDefinition, ToolParameter};
///
/// let mut registry = ToolRegistry::new();
///
/// registry.register_tool(
///     "echo",
///     "Echoes the input back",
///     vec![ToolParameter {
///         name: "message".to_string(),
///         description: "Message to echo".to_string(),
///         required: true,
///         schema: serde_json::json!({"type": "string"}),
///     }],
///     |args| {
///         let msg = args["message"].as_str().unwrap_or("");
///         Ok(format!("Echo: {}", msg))
///     }
/// );
///
/// let result = registry.execute_tool("echo", serde_json::json!({"message": "Hello"}));
/// assert!(result.is_success());
/// ```
#[derive(Default)]
pub struct ToolRegistry {
    definitions: std::collections::HashMap<String, ToolDefinition>,
    executors: std::collections::HashMap<String, ToolExecutor>,
}

impl ToolRegistry {
    /// Creates a new empty tool registry.
    pub fn new() -> Self {
        Self {
            definitions: std::collections::HashMap::new(),
            executors: std::collections::HashMap::new(),
        }
    }

    /// Registers a custom tool with the given name, description, parameters, and executor.
    pub fn register_tool<F>(
        &mut self,
        name: &str,
        description: &str,
        parameters: Vec<ToolParameter>,
        executor: F,
    ) where
        F: Fn(serde_json::Value) -> Result<String, String> + Send + Sync + 'static,
    {
        let def = ToolDefinition {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
        };
        let exec = ToolExecutor::new(executor);
        self.definitions.insert(name.to_string(), def);
        self.executors.insert(name.to_string(), exec);
    }

    /// Executes a registered tool with the given arguments.
    ///
    /// Returns a `ToolResult` indicating success or failure.
    pub fn execute_tool(&self, name: &str, arguments: serde_json::Value) -> ToolResult {
        let started_at = chrono::Utc::now();

        match (self.definitions.get(name), self.executors.get(name)) {
            (Some(def), Some(exec)) => match exec.execute(arguments) {
                Ok(result) => ToolResult {
                    id: uuid::Uuid::new_v4(),
                    tool_name: def.name.clone(),
                    success: true,
                    result: Some(result),
                    error: None,
                    started_at,
                    completed_at: chrono::Utc::now(),
                },
                Err(e) => ToolResult {
                    id: uuid::Uuid::new_v4(),
                    tool_name: def.name.clone(),
                    success: false,
                    result: None,
                    error: Some(e),
                    started_at,
                    completed_at: chrono::Utc::now(),
                },
            },
            _ => ToolResult {
                id: uuid::Uuid::new_v4(),
                tool_name: name.to_string(),
                success: false,
                result: None,
                error: Some(format!("Tool not found: {}", name)),
                started_at,
                completed_at: chrono::Utc::now(),
            },
        }
    }

    /// Lists all registered tool names.
    pub fn list_tools(&self) -> Vec<String> {
        self.definitions.keys().cloned().collect()
    }

    /// Gets a tool definition by name.
    pub fn get_tool(&self, name: &str) -> Option<&ToolDefinition> {
        self.definitions.get(name)
    }

    /// Returns true if a tool with the given name is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.definitions.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definition_serialization() {
        let def = ToolDefinition {
            name: "read".to_string(),
            description: "Read a file".to_string(),
            parameters: vec![ToolParameter {
                name: "path".to_string(),
                description: "File path".to_string(),
                required: true,
                schema: serde_json::json!({"type": "string"}),
            }],
        };

        let json = serde_json::to_string(&def).unwrap();
        assert!(json.contains("read"));
        assert!(json.contains("File path"));
    }

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success("read", "file contents");
        assert!(result.is_success());
        assert_eq!(result.unwrap_result(), "file contents");
        assert!(result.error.is_none());
    }

    #[test]
    fn test_tool_result_failure() {
        let result = ToolResult::failure("read", "File not found");
        assert!(!result.is_success());
        assert_eq!(result.unwrap_error(), "File not found");
        assert!(result.result.is_none());
    }

    #[test]
    fn test_tool_call_creation() {
        let call = ToolCall::new("read", serde_json::json!({"path": "/tmp/test.txt"}));
        assert_eq!(call.name, "read");
        assert_eq!(call.arguments["path"], "/tmp/test.txt");
    }

    #[test]
    fn test_tool_executor() {
        let executor = ToolExecutor::new(|args| {
            let path = args["path"].as_str().unwrap();
            Ok(format!("Read: {}", path))
        });

        let result = executor.execute(serde_json::json!({"path": "/tmp/test.txt"}));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Read: /tmp/test.txt");
    }
}
