#![allow(static_mut_refs)]
#![allow(unused_imports)]

use serde::{Deserialize, Serialize};
use std::ptr;
use std::slice;

static mut REGISTERED_TOOLS: Vec<ToolDefinition> = Vec::new();
static mut INITIALIZED: bool = false;

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginCommand {
    pub action: String,
    #[serde(default)]
    pub args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

fn register_tool(name: &str, description: &str, input_schema: serde_json::Value) {
    unsafe {
        REGISTERED_TOOLS.push(ToolDefinition {
            name: name.to_string(),
            description: description.to_string(),
            input_schema,
        });
    }
}

#[no_mangle]
pub extern "C" fn plugin_init() -> i32 {
    unsafe {
        if INITIALIZED {
            return 0;
        }

        register_tool(
            "hello_world",
            "A hello world plugin tool that demonstrates plugin tool registration",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name to greet"
                    }
                },
                "required": ["name"]
            }),
        );

        register_tool(
            "echo",
            "Echoes back the input arguments as JSON",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Message to echo back"
                    }
                },
                "required": ["message"]
            }),
        );

        INITIALIZED = true;
    }
    0
}

#[no_mangle]
pub extern "C" fn plugin_execute(command: *const u8, len: usize) -> i32 {
    if command.is_null() || len == 0 {
        return -1;
    }

    let cmd_slice = unsafe { slice::from_raw_parts(command, len) };

    let cmd: PluginCommand = match serde_json::from_slice(cmd_slice) {
        Ok(c) => c,
        Err(_) => return -2,
    };

    match cmd.action.as_str() {
        "hello_world" => {
            let name = cmd.args.get("name").and_then(|v| v.as_str()).unwrap_or("World");
            let _ = format!("Hello, {}!", name);
            0
        }
        "echo" => {
            let _ = cmd.args.get("message").and_then(|v| v.as_str()).unwrap_or("");
            0
        }
        "list_tools" => {
            0
        }
        _ => -3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_init_returns_zero() {
        let result = plugin_init();
        assert_eq!(result, 0, "plugin_init should return 0 for success");
    }

    #[test]
    fn test_plugin_init_called_twice_returns_zero() {
        let first = plugin_init();
        let second = plugin_init();
        assert_eq!(first, 0);
        assert_eq!(second, 0, "plugin_init should return 0 even when called twice");
    }

    #[test]
    fn test_plugin_execute_with_null_pointer() {
        let result = unsafe { plugin_execute(ptr::null(), 0) };
        assert_eq!(result, -1, "plugin_execute should return -1 for null pointer");
    }

    #[test]
    fn test_plugin_execute_with_null_pointer_only() {
        let result = unsafe { plugin_execute(ptr::null(), 100) };
        assert_eq!(result, -1, "plugin_execute should return -1 for null command pointer");
    }

    #[test]
    fn test_plugin_execute_with_zero_length() {
        let dummy_ptr: *const u8 = 0x1 as *const u8;
        let result = unsafe { plugin_execute(dummy_ptr, 0) };
        assert_eq!(result, -1, "plugin_execute should return -1 for zero length");
    }

    #[test]
    fn test_plugin_execute_with_invalid_json() {
        let invalid_json = b"not valid json";
        let result = unsafe {
            plugin_execute(invalid_json.as_ptr(), invalid_json.len())
        };
        assert_eq!(result, -2, "plugin_execute should return -2 for invalid JSON");
    }

    #[test]
    fn test_plugin_execute_hello_world() {
        let cmd = PluginCommand {
            action: "hello_world".to_string(),
            args: serde_json::json!({"name": "Test"}),
        };
        let json = serde_json::to_vec(&cmd).unwrap();
        let result = unsafe { plugin_execute(json.as_ptr(), json.len()) };
        assert_eq!(result, 0, "hello_world action should return 0");
    }

    #[test]
    fn test_plugin_execute_echo() {
        let cmd = PluginCommand {
            action: "echo".to_string(),
            args: serde_json::json!({"message": "Hello"}),
        };
        let json = serde_json::to_vec(&cmd).unwrap();
        let result = unsafe { plugin_execute(json.as_ptr(), json.len()) };
        assert_eq!(result, 0, "echo action should return 0");
    }

    #[test]
    fn test_plugin_execute_unknown_action() {
        let cmd = PluginCommand {
            action: "unknown_action".to_string(),
            args: serde_json::json!({}),
        };
        let json = serde_json::to_vec(&cmd).unwrap();
        let result = unsafe { plugin_execute(json.as_ptr(), json.len()) };
        assert_eq!(result, -3, "unknown action should return -3");
    }

    #[test]
    fn test_plugin_command_deserialize() {
        let json = r#"{"action":"test","args":{"key":"value"}}"#;
        let cmd: PluginCommand = serde_json::from_str(json).unwrap();
        assert_eq!(cmd.action, "test");
        assert_eq!(cmd.args["key"], "value");
    }

    #[test]
    fn test_tool_definition_serialize() {
        let tool = ToolDefinition {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("test_tool"));
        assert!(json.contains("A test tool"));
    }
}