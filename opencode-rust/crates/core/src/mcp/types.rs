use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

impl McpTool {
    pub fn validate_args(&self, args: &serde_json::Value) -> Result<(), String> {
        let schema = &self.parameters;

        if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
            let mut missing = Vec::new();
            for field in required {
                if let Some(field_name) = field.as_str() {
                    if args.get(field_name).is_none() {
                        missing.push(field_name.to_string());
                    }
                }
            }
            if !missing.is_empty() {
                return Err(format!("Missing required fields: {}", missing.join(", ")));
            }
        }

        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            for (field, field_schema) in properties {
                if let Some(value) = args.get(field) {
                    if let Some(expected_type) = field_schema.get("type").and_then(|t| t.as_str())
                    {
                        let actual_type = json_type_name(value);
                        if actual_type != expected_type && value != &serde_json::Value::Null {
                            return Err(format!(
                                "Field '{}' expected type '{}', got '{}'",
                                field, expected_type, actual_type
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    pub schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<McpPromptArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCapabilities {
    pub tools: bool,
    pub resources: bool,
    pub prompts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub name: String,
    pub url: String,
    pub tools: Vec<McpTool>,
    pub resources: Vec<McpResource>,
    pub prompts: Vec<McpPrompt>,
    pub capabilities: McpCapabilities,
}

impl McpServer {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            tools: Vec::new(),
            resources: Vec::new(),
            prompts: Vec::new(),
            capabilities: McpCapabilities {
                tools: false,
                resources: false,
                prompts: false,
            },
        }
    }

    pub fn find_resource(&self, uri: &str) -> Option<&McpResource> {
        self.resources.iter().find(|r| r.uri == uri)
    }

    pub fn find_prompt(&self, name: &str) -> Option<&McpPrompt> {
        self.prompts.iter().find(|p| p.name == name)
    }
}