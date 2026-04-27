//! Tool registry implementation.

use crate::config::{DirectoryScanner, ToolInfo};
use crate::session::Session;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use super::types::{ToolDefinition, ToolExecutor, ToolParameter};

#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>,
    executors: HashMap<String, ToolExecutor>,
    disabled: HashSet<String>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            executors: HashMap::new(),
            disabled: HashSet::new(),
        }
    }

    pub fn set_disabled(&mut self, tools: HashSet<String>) {
        self.disabled = tools;
    }

    pub fn is_disabled(&self, name: &str) -> bool {
        self.disabled.contains(name)
    }

    pub fn requires_approval(&self, name: &str) -> bool {
        self.tools
            .get(name)
            .map(|def| def.requires_approval)
            .unwrap_or(false)
    }

    pub fn register(&mut self, definition: ToolDefinition, executor: ToolExecutor) {
        let name = definition.name.clone();
        self.tools.insert(name.clone(), definition);
        self.executors.insert(name, executor);
    }

    pub fn get(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    pub fn get_executor(&self, name: &str) -> Option<&ToolExecutor> {
        if self.is_disabled(name) {
            return None;
        }
        self.executors.get(name)
    }

    pub fn get_executor_with_status(&self, name: &str) -> Option<(&ToolExecutor, bool)> {
        self.executors
            .get(name)
            .map(|executor| (executor, self.is_disabled(name)))
    }

    pub fn get_all(&self) -> Vec<&ToolDefinition> {
        self.tools.values().collect()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    pub fn list(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    pub fn list_with_status(&self) -> Vec<(&String, bool)> {
        self.tools
            .keys()
            .map(|name| (name, self.is_disabled(name)))
            .collect()
    }
}

pub(crate) fn register_custom_tool(registry: &mut ToolRegistry, tool_info: ToolInfo) {
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&tool_info.content) else {
        tracing::warn!(
            "Failed to parse tool content for {}: not valid JSON",
            tool_info.name
        );
        return;
    };

    let name = parsed
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&tool_info.name)
        .to_string();

    let description = parsed
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let args = parsed.get("args");
    let parameters = if let Some(obj) = args.and_then(|v| v.as_object()) {
        let props = obj.get("properties").and_then(|v| v.as_object());
        props
            .map(|properties| {
                properties
                    .iter()
                    .map(|(param_name, param_def)| {
                        let required = obj
                            .get("required")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().any(|r| r.as_str() == Some(param_name)))
                            .unwrap_or(false);

                        ToolParameter {
                            name: param_name.clone(),
                            description: param_def
                                .get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            required,
                            schema: param_def.clone(),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let file_path = tool_info.path.clone();
    let tool_def = ToolDefinition {
        name: name.clone(),
        description,
        parameters,
        ..Default::default()
    };

    let executor = Arc::new(move |args: serde_json::Value| {
        let args_str =
            serde_json::to_string(&args).map_err(|e| format!("Failed to serialize args: {}", e))?;

        let extension = file_path
            .extension()
            .and_then(|e: &std::ffi::OsStr| e.to_str())
            .ok_or_else(|| "Missing file extension".to_string())?;

        let node_cmd = match extension {
            "js" | "ts" | "mjs" | "cjs" => "node",
            _ => {
                return Err(format!(
                    "Unsupported file extension for tool execution: {}",
                    extension
                ));
            }
        };

        let output = std::process::Command::new(node_cmd)
            .arg(&file_path)
            .arg("--args")
            .arg(&args_str)
            .output()
            .map_err(|e| format!("Failed to execute tool: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    });

    registry.register(tool_def, executor);
}

fn register_discovered_custom_tools(registry: &mut ToolRegistry, project_root: Option<PathBuf>) {
    let scanner = DirectoryScanner::new();

    if let Some(ref root) = project_root {
        let opencode_path = root.join(".opencode-rs");
        if opencode_path.exists() {
            let tools = scanner.scan_tools(&opencode_path);
            for tool_info in tools {
                register_custom_tool(registry, tool_info);
            }
        }
    }

    if let Some(global_path) = dirs::config_dir() {
        let global_opencode = global_path.join("opencode");
        if global_opencode.exists() {
            let tools = scanner.scan_tools(&global_opencode);
            for tool_info in tools {
                register_custom_tool(registry, tool_info);
            }
        }
    }
}

pub fn build_default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    registry.register(
        ToolDefinition {
            name: "read".to_string(),
            description: "Read contents of a file".to_string(),
            parameters: vec![ToolParameter {
                name: "file_path".to_string(),
                description: "Path to the file to read".to_string(),
                required: true,
                schema: serde_json::json!({ "type": "string" }),
            }],
            ..Default::default()
        },
        Arc::new(|args| {
            let file_path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: file_path")?;
            std::fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))
        }),
    );

    registry.register(
        ToolDefinition {
            name: "write".to_string(),
            description: "Write content to a file".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "file_path".to_string(),
                    description: "Path to the file to write".to_string(),
                    required: true,
                    schema: serde_json::json!({ "type": "string" }),
                },
                ToolParameter {
                    name: "content".to_string(),
                    description: "Content to write to the file".to_string(),
                    required: true,
                    schema: serde_json::json!({ "type": "string" }),
                },
            ],
            ..Default::default()
        },
        Arc::new(|args| {
            let file_path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: file_path")?;
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: content")?;

            if let Some(parent) = std::path::Path::new(file_path).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directories: {}", e))?;
            }
            std::fs::write(file_path, content)
                .map_err(|e| format!("Failed to write file: {}", e))?;
            Ok(format!("Successfully wrote to {}", file_path))
        }),
    );

    registry.register(
        ToolDefinition {
            name: "grep".to_string(),
            description: "Search for pattern in files".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "pattern".to_string(),
                    description: "Pattern to search for".to_string(),
                    required: true,
                    schema: serde_json::json!({ "type": "string" }),
                },
                ToolParameter {
                    name: "path".to_string(),
                    description: "Path to search in".to_string(),
                    required: false,
                    schema: serde_json::json!({ "type": "string" }),
                },
            ],
            ..Default::default()
        },
        Arc::new(|args| {
            let pattern = args
                .get("pattern")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: pattern")?;
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or(".");

            let output = std::process::Command::new("grep")
                .args(["-r", "-n", pattern, path])
                .output()
                .map_err(|e| format!("Failed to execute grep: {}", e))?;

            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }),
    );

    registry.register(
        ToolDefinition {
            name: "list_dir".to_string(),
            description: "List directory contents".to_string(),
            parameters: vec![ToolParameter {
                name: "path".to_string(),
                description: "Path to list".to_string(),
                required: false,
                schema: serde_json::json!({ "type": "string" }),
            }],
            ..Default::default()
        },
        Arc::new(|args| {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or(".");
            let entries = std::fs::read_dir(path)
                .map_err(|e| format!("Failed to read directory: {}", e))?
                .filter_map(|e| e.ok())
                .map(|e| {
                    let f_type = if e.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                        "dir"
                    } else {
                        "file"
                    };
                    format!("{} ({})", e.file_name().to_string_lossy(), f_type)
                })
                .collect::<Vec<_>>()
                .join("\n");
            Ok(entries)
        }),
    );

    registry.register(
        ToolDefinition {
            name: "execute".to_string(),
            description: "Execute a shell command".to_string(),
            parameters: vec![ToolParameter {
                name: "command".to_string(),
                description: "Command to execute".to_string(),
                required: true,
                schema: serde_json::json!({ "type": "string" }),
            }],
            requires_approval: true,
        },
        Arc::new(|args| {
            let command = args
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: command")?;

            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .map_err(|e| format!("Failed to execute command: {}", e))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                Ok(format!("{}{}", stdout, if stderr.is_empty() { "" } else { "\nSTDERR:\n" }))
            } else {
                Err(format!(
                    "Command failed with exit code: {}{}{}",
                    output
                        .status
                        .code()
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    if stdout.is_empty() { "" } else { "\nSTDOUT:\n" },
                    stdout
                ))
            }
        }),
    );

    registry.register(
        ToolDefinition {
            name: "web_fetch".to_string(),
            description: "Fetch content from a URL".to_string(),
            parameters: vec![ToolParameter {
                name: "url".to_string(),
                description: "URL to fetch".to_string(),
                required: true,
                schema: serde_json::json!({ "type": "string" }),
            }],
            ..Default::default()
        },
        Arc::new(|args| {
            let url = args
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: url")?;

            let response = reqwest::blocking::get(url)
                .map_err(|e| format!("Failed to fetch URL: {}", e))?
                .text()
                .map_err(|e| format!("Failed to read response: {}", e))?;

            Ok(response)
        }),
    );

    registry.register(
        ToolDefinition {
            name: "websearch".to_string(),
            description: "Search the web for information".to_string(),
            parameters: vec![ToolParameter {
                name: "query".to_string(),
                description: "Search query".to_string(),
                required: true,
                schema: serde_json::json!({ "type": "string" }),
            }],
            ..Default::default()
        },
        Arc::new(|_args| Err("Web search not yet implemented".to_string())),
    );

    registry.register(
        ToolDefinition {
            name: "session_load".to_string(),
            description: "Load a session from local storage".to_string(),
            parameters: vec![ToolParameter {
                name: "session_id".to_string(),
                description: "Session ID to load".to_string(),
                required: true,
                schema: serde_json::json!({ "type": "string" }),
            }],
            ..Default::default()
        },
        Arc::new(|args| {
            let session_id = args
                .get("session_id")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: session_id")?;
            let id = uuid::Uuid::parse_str(session_id)
                .map_err(|e| format!("Invalid session_id: {}", e))?;
            let session = Session::load_by_id(&id).map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&session)
                .map_err(|e| format!("Failed to serialize session: {}", e))
        }),
    );

    registry.register(
        ToolDefinition {
            name: "session_save".to_string(),
            description: "Save current session to local storage".to_string(),
            parameters: vec![ToolParameter {
                name: "session_id".to_string(),
                description: "Optional session ID to persist".to_string(),
                required: false,
                schema: serde_json::json!({ "type": "string" }),
            }],
            ..Default::default()
        },
        Arc::new(|args| {
            let session_id = args.get("session_id").and_then(|v| v.as_str());
            let id = match session_id {
                Some(raw) => {
                    uuid::Uuid::parse_str(raw).map_err(|e| format!("Invalid session_id: {}", e))?
                }
                None => uuid::Uuid::new_v4(),
            };

            let path = Session::session_path(&id);
            let mut session = if path.exists() {
                Session::load(&path).map_err(|e| e.to_string())?
            } else {
                let mut s = Session::new();
                s.id = id;
                s
            };

            session.id = id;
            session.save(&path).map_err(|e| e.to_string())?;

            serde_json::to_string(&serde_json::json!({
                "session_id": id.to_string(),
                "saved": true,
            }))
            .map_err(|e| format!("Failed to serialize response: {}", e))
        }),
    );

    registry.register(
        ToolDefinition {
            name: "bash".to_string(),
            description: "Run a bash command".to_string(),
            parameters: vec![ToolParameter {
                name: "command".to_string(),
                description: "Command to run in bash".to_string(),
                required: true,
                schema: serde_json::json!({ "type": "string" }),
            }],
            requires_approval: true,
        },
        Arc::new(|args| {
            let command = args
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or("Missing required parameter: command")?;

            let output = std::process::Command::new("bash")
                .arg("-c")
                .arg(command)
                .output()
                .map_err(|e| format!("Failed to execute bash command: {}", e))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                Ok(format!("{}{}", stdout, if stderr.is_empty() { "" } else { "\nSTDERR:\n" }))
            } else {
                Err(format!(
                    "Command failed with exit code: {}{}{}",
                    output
                        .status
                        .code()
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    if stdout.is_empty() { "" } else { "\nSTDOUT:\n" },
                    stdout
                ))
            }
        }),
    );

    let project_root = find_project_root();
    register_discovered_custom_tools(&mut registry, project_root);

    registry
}

fn find_project_root() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    for ancestor in cwd.ancestors() {
        let opencode_dir = ancestor.join(".opencode-rs");
        if opencode_dir.exists() && opencode_dir.is_dir() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}
