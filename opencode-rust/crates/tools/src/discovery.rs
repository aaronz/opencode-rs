use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

use crate::{Tool, ToolContext, ToolResult};
use opencode_core::OpenCodeError;

const PROJECT_TOOLS_DIR: &str = ".opencode/tools";
const GLOBAL_TOOLS_DIR: &str = "opencode/tools";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct DiscoveredTool {
    pub definition: ToolDefinition,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveredToolSource {
    Project,
    Global,
}

pub struct ToolDiscovery {
    project_tools_path: Option<PathBuf>,
    global_tools_path: Option<PathBuf>,
}

impl ToolDiscovery {
    pub fn new(project_root: Option<PathBuf>) -> Self {
        let project_tools_path = project_root
            .map(|p| p.join(PROJECT_TOOLS_DIR))
            .filter(|p| p.exists());

        let global_tools_path = dirs::config_dir()
            .map(|p| p.join(GLOBAL_TOOLS_DIR))
            .filter(|p| p.exists());

        Self {
            project_tools_path,
            global_tools_path,
        }
    }

    pub fn discover_tools(&self) -> Vec<DiscoveredTool> {
        self.discover_tools_with_source()
            .into_iter()
            .map(|(tool, _)| tool)
            .collect()
    }

    pub fn discover_tools_with_source(&self) -> Vec<(DiscoveredTool, DiscoveredToolSource)> {
        let mut tools = Vec::new();

        if let Some(ref path) = self.project_tools_path {
            let discovered = self.scan_directory(path);
            for tool in discovered {
                tools.push((tool, DiscoveredToolSource::Project));
            }
        }

        if let Some(ref path) = self.global_tools_path {
            let discovered = self.scan_directory(path);
            for tool in discovered {
                tools.push((tool, DiscoveredToolSource::Global));
            }
        }

        tools
    }

    fn scan_directory(&self, dir: &Path) -> Vec<DiscoveredTool> {
        let mut tools = Vec::new();

        for entry in WalkDir::new(dir)
            .follow_links(true)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if self.is_tool_file(path) {
                if let Some(tool_def) = self.parse_tool_file(path) {
                    tools.push(DiscoveredTool {
                        definition: tool_def,
                        file_path: path.to_path_buf(),
                    });
                }
            }
        }

        tools
    }

    fn is_tool_file(&self, path: &Path) -> bool {
        path.extension()
            .map(|ext| ext == "js" || ext == "ts" || ext == "mjs" || ext == "cjs")
            .unwrap_or(false)
    }

    fn parse_tool_file(&self, path: &Path) -> Option<ToolDefinition> {
        let content = std::fs::read_to_string(path).ok()?;

        self.extract_tool_definition(&content)
    }

    fn extract_tool_definition(&self, content: &str) -> Option<ToolDefinition> {
        let var_name_regex = Regex::new(r#"(?:const|let|var)\s+(\w+)\s*=\s*\{"#).ok()?;
        let name_in_obj_regex = Regex::new(r#"name\s*:\s*["']([^"']+)["']"#).ok()?;
        let desc_regex = Regex::new(r#"description\s*:\s*["']([^"']+)["']"#).ok()?;
        let params_regex = Regex::new(r#"parameters\s*:\s*(\{[^}]+\})"#).ok()?;

        let var_name = var_name_regex
            .captures(content)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_lowercase().replace('_', "-"));

        let name = name_in_obj_regex
            .captures(content)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .or(var_name);

        let name = name?;

        let desc_capture = desc_regex.captures(content)?;
        let description = desc_capture.get(1)?.as_str().to_string();

        let params_capture = params_regex.captures(content);
        let parameters = params_capture
            .and_then(|c| serde_json::from_str::<serde_json::Value>(c.get(1)?.as_str()).ok())
            .unwrap_or(serde_json::json!({
                "type": "object",
                "properties": {},
                "additionalProperties": true
            }));

        Some(ToolDefinition {
            name,
            description,
            parameters,
        })
    }
}

pub struct CustomTool {
    definition: ToolDefinition,
    file_path: PathBuf,
}

impl CustomTool {
    pub fn from_discovered(discovered: DiscoveredTool) -> Self {
        Self {
            definition: discovered.definition,
            file_path: discovered.file_path,
        }
    }

    pub fn definition(&self) -> &ToolDefinition {
        &self.definition
    }
}

#[async_trait]
impl Tool for CustomTool {
    fn name(&self) -> &str {
        &self.definition.name
    }

    fn description(&self) -> &str {
        &self.definition.description
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(Self {
            definition: self.definition.clone(),
            file_path: self.file_path.clone(),
        })
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args_str =
            serde_json::to_string(&args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let extension = self
            .file_path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| OpenCodeError::Tool("Missing file extension".to_string()))?;

        let node_cmd = if extension == "mjs" || extension == "js" || extension == "ts" {
            "node"
        } else {
            return Err(OpenCodeError::Tool(format!(
                "Unsupported file extension for tool execution: {}",
                extension
            )));
        };

        let output = Command::new(node_cmd)
            .arg(&self.file_path)
            .arg("--args")
            .arg(&args_str)
            .output()
            .map_err(|e| OpenCodeError::Tool(format!("Failed to execute tool: {}", e)))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(ToolResult::ok(stdout))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(ToolResult::err(stderr))
        }
    }
}

pub async fn register_custom_tools(
    registry: &crate::ToolRegistry,
    project_root: Option<PathBuf>,
) -> Vec<String> {
    let discovery = ToolDiscovery::new(project_root);
    let discovered = discovery.discover_tools_with_source();

    let mut registered = Vec::new();

    for (tool, source) in discovered {
        let custom_tool = CustomTool::from_discovered(tool);
        let name = custom_tool.name().to_string();

        let tool_source = match source {
            DiscoveredToolSource::Project => crate::registry::ToolSource::CustomProject,
            DiscoveredToolSource::Global => crate::registry::ToolSource::CustomGlobal,
        };
        registry
            .register_with_source(custom_tool, tool_source)
            .await;
        registered.push(name);
    }

    registered
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_tool_file(temp_dir: &Path, filename: &str, content: &str) {
        let file_path = temp_dir.join(filename);
        std::fs::write(&file_path, content).unwrap();
    }

    #[test]
    fn test_parse_tool_definition() {
        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode/tools");
        std::fs::create_dir_all(&tools_dir).unwrap();

        create_test_tool_file(
            &tools_dir,
            "my_tool.js",
            r#"
const myTool = {
    name: "my_tool",
    description: "A test tool",
    parameters: {
        type: "object",
        properties: {
            input: { type: "string" }
        }
    }
};
export default myTool;
"#,
        );

        let discovery = ToolDiscovery::new(Some(temp_dir.path().to_path_buf()));
        let tools = discovery.discover_tools();

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].definition.name, "my_tool");
        assert_eq!(tools[0].definition.description, "A test tool");
    }

    #[test]
    fn test_parse_multiple_tools() {
        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode/tools");
        std::fs::create_dir_all(&tools_dir).unwrap();

        create_test_tool_file(
            &tools_dir,
            "tool1.js",
            r#"
const toolOne = {
    description: "First tool",
    parameters: {}
};
"#,
        );

        create_test_tool_file(
            &tools_dir,
            "tool2.ts",
            r#"
const toolTwo = {
    description: "Second tool", 
    parameters: {}
};
"#,
        );

        let discovery = ToolDiscovery::new(Some(temp_dir.path().to_path_buf()));
        let tools = discovery.discover_tools();

        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn test_global_tools_path() {
        let discovery = ToolDiscovery::new(None);
        if let Some(global_path) = &discovery.global_tools_path {
            assert!(global_path.to_string_lossy().contains("opencode/tools"));
        }
    }
}
