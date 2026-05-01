use std::sync::Arc;

use opencode_core::OpenCodeError;
use opencode_tools::{Tool, ToolContext, ToolRegistry, ToolResult};
use tokio::sync::RwLock;

pub struct RuntimeFacadeToolRouter {
    registry: Arc<RwLock<ToolRegistry>>,
}

impl RuntimeFacadeToolRouter {
    pub fn new(registry: ToolRegistry) -> Self {
        Self {
            registry: Arc::new(RwLock::new(registry)),
        }
    }

    pub async fn execute(
        &self,
        name: &str,
        args: serde_json::Value,
        ctx: Option<ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        self.registry.read().await.execute(name, args, ctx).await
    }

    pub async fn execute_with_validation(
        &self,
        name: &str,
        args: serde_json::Value,
        ctx: Option<ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        // Deny-First: tool MUST exist in registry
        let tool = self.registry.read().await.get(name).await;
        let Some(tool) = tool else {
            return Ok(ToolResult::err(format!(
                "Tool '{}' not found or not available. Access denied.",
                name
            )));
        };

        // Validate JSON schema if provided
        if let Some(schema) = tool.input_schema() {
            if let Some(errors) = validate_json_schema(&args, &schema) {
                return Ok(ToolResult::err(format!(
                    "Invalid arguments for tool '{}': {}",
                    name,
                    errors.join("; ")
                )));
            }
        }
        self.execute(name, args, ctx).await
    }

    pub async fn get(&self, name: &str) -> Option<Box<dyn Tool>> {
        self.registry.read().await.get(name).await
    }

    pub async fn list(&self) -> Vec<(String, String, bool)> {
        self.registry.read().await.list_filtered(None).await
    }

    pub async fn register<T: Tool + 'static>(&self, tool: T) {
        self.registry.write().await.register(tool).await;
    }

    pub async fn unregister(&self, name: &str) -> bool {
        self.registry.write().await.unregister(name).await
    }

    pub async fn set_disabled(&self, tools: std::collections::HashSet<String>) {
        self.registry.write().await.set_disabled(tools);
    }

    pub fn registry(&self) -> Arc<RwLock<ToolRegistry>> {
        Arc::clone(&self.registry)
    }
}

fn validate_json_schema(
    args: &serde_json::Value,
    schema: &serde_json::Value,
) -> Option<Vec<String>> {
    let mut errors = Vec::new();

    if let Some(obj) = schema.as_object() {
        if let Some(required_fields) = obj.get("required").and_then(|r| r.as_array()) {
            for field in required_fields {
                if let Some(field_name) = field.as_str() {
                    if args.get(field_name).is_none_or(|v| v.is_null()) {
                        errors.push(format!("Missing required field: '{}'", field_name));
                    }
                }
            }
        }

        if let Some(properties) = obj.get("properties").and_then(|p| p.as_object()) {
            for (field, field_schema) in properties {
                if let Some(args_val) = args.get(field) {
                    if !args_val.is_null() {
                        if let Some(expected_type) = field_schema
                            .as_object()
                            .and_then(|fo| fo.get("type"))
                            .and_then(|t| t.as_str())
                        {
                            let actual_type = json_type_name(args_val);
                            if actual_type != expected_type {
                                errors.push(format!(
                                    "Field '{}' expected type '{}', got '{}'",
                                    field, expected_type, actual_type
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        None
    } else {
        Some(errors)
    }
}

fn json_type_name(val: &serde_json::Value) -> &'static str {
    match val {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

impl Clone for RuntimeFacadeToolRouter {
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
        }
    }
}

impl Default for RuntimeFacadeToolRouter {
    fn default() -> Self {
        Self::new(ToolRegistry::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_deny_first_unknown_tool() {
        let router = RuntimeFacadeToolRouter::default();
        let result = router.execute_with_validation("nonexistent_tool", json!({}), None).await;
        let err_result = result.unwrap();
        assert!(!err_result.success);
        assert!(err_result.error.is_some());
        let error_msg = err_result.error.as_ref().unwrap();
        assert!(error_msg.contains("not found or not available"), "Got: {}", error_msg);
    }

    #[test]
    fn test_validate_json_schema_valid() {
        let schema = json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "content": {"type": "string"}
            },
            "required": ["path", "content"]
        });
        let args = json!({"path": "/foo", "content": "bar"});
        assert!(validate_json_schema(&args, &schema).is_none());
    }

    #[test]
    fn test_validate_json_schema_missing_required() {
        let schema = json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "content": {"type": "string"}
            },
            "required": ["path", "content"]
        });
        let args = json!({"path": "/foo"});
        let errors = validate_json_schema(&args, &schema);
        assert!(errors.is_some());
        assert!(errors.unwrap().iter().any(|e| e.contains("content")));
    }

    #[test]
    fn test_validate_json_schema_type_mismatch() {
        let schema = json!({
            "type": "object",
            "properties": {
                "count": {"type": "number"}
            },
            "required": ["count"]
        });
        let args = json!({"count": "not a number"});
        let errors = validate_json_schema(&args, &schema);
        assert!(errors.is_some());
        assert!(errors.unwrap().iter().any(|e| e.contains("number")));
    }
}
