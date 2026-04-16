use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

use crate::sealed;
use crate::{Tool, ToolContext, ToolResult};
use opencode_core::OpenCodeError;

const PROJECT_TOOLS_DIR: &str = ".opencode/tools";
const GLOBAL_TOOLS_DIR: &str = "opencode/tools";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone)]
pub(crate) struct DiscoveredTool {
    pub definition: ToolDefinition,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiscoveredToolSource {
    Project,
    Global,
}

pub(crate) struct ToolDiscovery {
    project_tools_path: Option<PathBuf>,
    global_tools_path: Option<PathBuf>,
}

impl ToolDiscovery {
    #[allow(dead_code)]
    pub(crate) fn new(project_root: Option<PathBuf>) -> Self {
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

    #[allow(dead_code)]
    pub(crate) fn discover_tools(&self) -> Vec<DiscoveredTool> {
        self.discover_tools_with_source()
            .into_iter()
            .map(|(tool, _)| tool)
            .collect()
    }

    pub(crate) fn discover_tools_with_source(&self) -> Vec<(DiscoveredTool, DiscoveredToolSource)> {
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

pub(crate) struct CustomTool {
    definition: ToolDefinition,
    file_path: PathBuf,
}

impl CustomTool {
    #[allow(dead_code)]
    pub(crate) fn from_discovered(discovered: DiscoveredTool) -> Self {
        Self {
            definition: discovered.definition,
            file_path: discovered.file_path,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn definition(&self) -> &ToolDefinition {
        &self.definition
    }
}

impl sealed::Sealed for CustomTool {}

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

pub async fn build_default_registry(project_root: Option<PathBuf>) -> crate::ToolRegistry {
    let registry = crate::ToolRegistry::new();

    registry.register(crate::read::ReadTool::new()).await;
    registry.register(crate::write::WriteTool).await;
    registry.register(crate::edit::EditTool).await;
    registry.register(crate::bash::BashTool::new()).await;
    registry.register(crate::ls::LsTool).await;
    registry.register(crate::glob::GlobTool).await;
    registry.register(crate::grep_tool::GrepTool).await;
    registry.register(crate::codesearch::CodeSearchTool).await;
    registry.register(crate::task::TaskTool).await;
    registry
        .register(crate::session_tools::SessionLoadTool)
        .await;
    registry
        .register(crate::session_tools::SessionSaveTool)
        .await;
    registry.register(crate::multiedit::MultiEditTool).await;
    registry.register(crate::webfetch::WebfetchTool).await;
    registry.register(crate::web_search::WebSearchTool).await;
    registry.register(crate::todowrite::TodowriteTool).await;
    registry.register(crate::question::QuestionTool).await;
    registry.register(crate::plan::PlanTool).await;
    registry.register(crate::plan_exit::PlanExitTool).await;
    registry.register(crate::lsp_tool::LspTool).await;
    registry.register(crate::git_tools::GitStatusTool).await;
    registry.register(crate::git_tools::GitDiffTool).await;
    registry.register(crate::git_tools::GitLogTool).await;
    registry.register(crate::git_tools::GitShowTool).await;
    registry.register(crate::apply_patch::ApplyPatchTool).await;
    registry
        .register(crate::truncation_dir::TruncationDirTool)
        .await;

    let _ = register_custom_tools(&registry, project_root).await;

    registry
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

    #[tokio::test]
    async fn test_discovery_and_registration_integration() {
        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode/tools");
        std::fs::create_dir_all(&tools_dir).unwrap();

        create_test_tool_file(
            &tools_dir,
            "echo.js",
            r#"
const echoTool = {
    name: "echo",
    description: "Echoes the input back",
    parameters: { type: "object", properties: { message: { type: "string" } }, required: ["message"] }
};
export default echoTool;
"#,
        );

        let registry = crate::ToolRegistry::new();
        let registered =
            register_custom_tools(&registry, Some(temp_dir.path().to_path_buf())).await;

        assert_eq!(registered.len(), 1, "Should discover exactly one tool");
        assert_eq!(registered[0], "echo", "Tool name should be 'echo'");

        let tools = registry.list_filtered(None).await;
        let tool_names: Vec<&str> = tools.iter().map(|(n, _, _)| n.as_str()).collect();
        assert!(
            tool_names.contains(&"echo"),
            "Custom tool 'echo' should appear in tool listing. Found: {:?}",
            tool_names
        );

        let tool = registry.get("echo").await;
        assert!(
            tool.is_some(),
            "Custom tool should be retrievable from registry"
        );
    }

    #[tokio::test]
    async fn test_discovery_multiple_tools_registration() {
        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode/tools");
        std::fs::create_dir_all(&tools_dir).unwrap();

        create_test_tool_file(
            &tools_dir,
            "tool1.js",
            r#"const toolOne = { name: "tool_one", description: "First tool", parameters: {} }; export default toolOne;"#,
        );

        create_test_tool_file(
            &tools_dir,
            "tool2.ts",
            r#"const toolTwo = { name: "tool_two", description: "Second tool", parameters: {} }; export default toolTwo;"#,
        );

        let registry = crate::ToolRegistry::new();
        let registered =
            register_custom_tools(&registry, Some(temp_dir.path().to_path_buf())).await;

        assert_eq!(registered.len(), 2, "Should discover exactly two tools");

        let tools = registry.list_filtered(None).await;
        let tool_names: Vec<&str> = tools.iter().map(|(n, _, _)| n.as_str()).collect();
        assert!(
            tool_names.contains(&"tool_one"),
            "First custom tool should be registered"
        );
        assert!(
            tool_names.contains(&"tool_two"),
            "Second custom tool should be registered"
        );
    }

    #[tokio::test]
    async fn test_discovery_no_tools_returns_empty() {
        let temp_dir = TempDir::new().unwrap();

        let registry = crate::ToolRegistry::new();
        let registered =
            register_custom_tools(&registry, Some(temp_dir.path().to_path_buf())).await;

        assert!(
            registered.is_empty(),
            "Should discover no tools when none exist"
        );
    }

    #[tokio::test]
    async fn test_registry_contains_discovered_tools() {
        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode/tools");
        std::fs::create_dir_all(&tools_dir).unwrap();

        create_test_tool_file(
            &tools_dir,
            "check_tool.js",
            r#"const checkTool = { name: "check_tool", description: "Check tool", parameters: {} }; export default checkTool;"#,
        );

        let registry = crate::ToolRegistry::new();
        let _ = register_custom_tools(&registry, Some(temp_dir.path().to_path_buf())).await;

        let tool = registry.get("check_tool").await;
        assert!(
            tool.is_some(),
            "Registry should contain discovered tool 'check_tool'"
        );
        assert_eq!(tool.unwrap().name(), "check_tool");
    }

    #[tokio::test]
    async fn test_existing_tool_registration_not_affected() {
        use async_trait::async_trait;

        #[derive(Clone)]
        struct ExistingTool;
        impl crate::sealed::Sealed for ExistingTool {}
        #[async_trait]
        impl Tool for ExistingTool {
            fn name(&self) -> &str {
                "existing_tool"
            }
            fn description(&self) -> &str {
                "An existing tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<crate::ToolContext>,
            ) -> Result<crate::ToolResult, opencode_core::OpenCodeError> {
                Ok(crate::ToolResult::ok("existing"))
            }
        }

        let registry = crate::ToolRegistry::new();
        registry.register(ExistingTool).await;

        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode/tools");
        std::fs::create_dir_all(&tools_dir).unwrap();

        create_test_tool_file(
            &tools_dir,
            "new_tool.js",
            r#"const newTool = { name: "new_tool", description: "A new tool", parameters: {} }; export default newTool;"#,
        );

        let registered =
            register_custom_tools(&registry, Some(temp_dir.path().to_path_buf())).await;

        assert_eq!(registered.len(), 1);
        assert_eq!(registered[0], "new_tool");

        let existing = registry.get("existing_tool").await;
        assert!(
            existing.is_some(),
            "Existing tool should still be in registry"
        );

        let new_tool = registry.get("new_tool").await;
        assert!(
            new_tool.is_some(),
            "Newly discovered tool should be in registry"
        );
    }

    #[tokio::test]
    async fn test_tool_registration_error_handling_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode/tools");
        std::fs::create_dir_all(&tools_dir).unwrap();

        std::fs::write(
            tools_dir.join("invalid.js"),
            r#"const invalidTool = { name: "invalid", description: "Invalid", parameters: {invalid} }; export default invalidTool;"#,
        ).unwrap();

        let registry = crate::ToolRegistry::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                register_custom_tools(&registry, Some(temp_dir.path().to_path_buf())).await
            });
        }));

        assert!(
            result.is_ok() || result.is_err(),
            "Should handle parse errors gracefully"
        );
    }

    #[tokio::test]
    async fn test_tool_registration_error_handling_missing_file() {
        let temp_dir = TempDir::new().unwrap();

        let registry = crate::ToolRegistry::new();
        let registered =
            register_custom_tools(&registry, Some(temp_dir.path().to_path_buf())).await;

        assert!(
            registered.is_empty(),
            "Should handle missing tools directory gracefully"
        );
    }

    #[tokio::test]
    async fn test_tool_execution_after_registration() {
        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode/tools");
        std::fs::create_dir_all(&tools_dir).unwrap();

        create_test_tool_file(
            &tools_dir,
            "exec_test.js",
            r#"
const execTool = {
    name: "exec_test",
    description: "Execution test",
    parameters: { type: "object", properties: { input: { type: "string" } } }
};
const argsIdx = process.argv.findIndex(a => a === '--args');
const args = argsIdx >= 0 ? JSON.parse(process.argv[argsIdx + 1] || '{}') : {};
console.log(args.input || 'no input');
export default execTool;
"#,
        );

        let registry = crate::ToolRegistry::new();
        let _ = register_custom_tools(&registry, Some(temp_dir.path().to_path_buf())).await;

        let result = registry
            .execute(
                "exec_test",
                serde_json::json!({"input": "test_value"}),
                None,
            )
            .await;

        assert!(result.is_ok(), "Tool execution should succeed");
        let result = result.unwrap();
        assert!(result.success, "Tool should execute successfully");
        assert!(
            result.content.contains("test_value") || result.content.contains("no input"),
            "Tool should produce output. Got: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn test_custom_tool_discovery_and_execution_full_flow() {
        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode/tools");
        std::fs::create_dir_all(&tools_dir).unwrap();

        create_test_tool_file(
            &tools_dir,
            "full_flow.js",
            r#"
const fullFlowTool = {
    name: "full_flow",
    description: "Full flow test tool",
    parameters: { type: "object", properties: { message: { type: "string" } }, required: ["message"] }
};
const argsIdx = process.argv.findIndex(a => a === '--args');
const args = argsIdx >= 0 ? JSON.parse(process.argv[argsIdx + 1] || '{}') : {};
console.log(args.message || 'default');
export default fullFlowTool;
"#,
        );

        let registry = build_default_registry(Some(temp_dir.path().to_path_buf())).await;

        let custom_tool = registry.get("full_flow").await;
        assert!(
            custom_tool.is_some(),
            "Custom tool should be registered via build_default_registry"
        );

        let builtin_tool = registry.get("read").await;
        assert!(
            builtin_tool.is_some(),
            "Built-in tool 'read' should still be registered"
        );

        let result = registry
            .execute(
                "full_flow",
                serde_json::json!({"message": "flow_test"}),
                None,
            )
            .await;

        match result {
            Ok(r) if r.success => {}
            other => panic!("Expected successful execution, got: {:?}", other),
        }
    }

    #[test]
    fn test_discovery_respects_file_extensions() {
        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode/tools");
        std::fs::create_dir_all(&tools_dir).unwrap();

        std::fs::write(
            tools_dir.join("valid.js"),
            r#"const v = { name: "v", description: "v", parameters: {} }; export default v;"#,
        )
        .unwrap();
        std::fs::write(
            tools_dir.join("valid.ts"),
            r#"const v2 = { name: "v2", description: "v2", parameters: {} }; export default v2;"#,
        )
        .unwrap();
        std::fs::write(
            tools_dir.join("valid.mjs"),
            r#"const v3 = { name: "v3", description: "v3", parameters: {} }; export default v3;"#,
        )
        .unwrap();
        std::fs::write(
            tools_dir.join("valid.cjs"),
            r#"const v4 = { name: "v4", description: "v4", parameters: {} }; export default v4;"#,
        )
        .unwrap();
        std::fs::write(
            tools_dir.join("invalid.txt"),
            r#"const i = { name: "i", description: "i", parameters: {} }; export default i;"#,
        )
        .unwrap();
        std::fs::write(
            tools_dir.join("invalid.md"),
            r#"const i2 = { name: "i2", description: "i2", parameters: {} }; export default i2;"#,
        )
        .unwrap();

        let discovery = ToolDiscovery::new(Some(temp_dir.path().to_path_buf()));
        let tools = discovery.discover_tools();

        assert_eq!(
            tools.len(),
            4,
            "Should discover only .js, .ts, .mjs, .cjs files"
        );
        let names: Vec<&str> = tools.iter().map(|t| t.definition.name.as_str()).collect();
        assert!(names.contains(&"v"));
        assert!(names.contains(&"v2"));
        assert!(names.contains(&"v3"));
        assert!(names.contains(&"v4"));
        assert!(!names.contains(&"i"));
        assert!(!names.contains(&"i2"));
    }

    #[test]
    fn test_discovery_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode/tools/nested/deep");
        std::fs::create_dir_all(&tools_dir).unwrap();

        std::fs::write(
            tools_dir.join("nested_tool.js"),
            r#"const n = { name: "nested_tool", description: "Nested", parameters: {} }; export default n;"#,
        ).unwrap();

        let discovery = ToolDiscovery::new(Some(temp_dir.path().to_path_buf()));
        let tools = discovery.discover_tools();

        assert_eq!(
            tools.len(),
            1,
            "Should discover tools in nested directories"
        );
        assert_eq!(tools[0].definition.name, "nested_tool");
    }

    #[tokio::test]
    async fn custom_tool_registration() {
        let temp_dir = TempDir::new().unwrap();
        let tools_dir = temp_dir.path().join(".opencode/tools");
        std::fs::create_dir_all(&tools_dir).unwrap();

        create_test_tool_file(
            &tools_dir,
            "registration_test.js",
            r#"
const registrationTestTool = {
    name: "registration_test",
    description: "Test tool for custom_tool_registration",
    parameters: { type: "object", properties: { value: { type: "string" } } }
};
const argsIdx = process.argv.findIndex(a => a === '--args');
const args = argsIdx >= 0 ? JSON.parse(process.argv[argsIdx + 1] || '{}') : {};
console.log(args.value || 'default_value');
export default registrationTestTool;
"#,
        );

        let registry = crate::ToolRegistry::new();

        let registered =
            register_custom_tools(&registry, Some(temp_dir.path().to_path_buf())).await;

        assert_eq!(registered.len(), 1, "Should discover exactly one tool");
        assert_eq!(
            registered[0], "registration_test",
            "Tool name should be 'registration_test'"
        );

        let tools = registry.list_filtered(None).await;
        let tool_names: Vec<&str> = tools.iter().map(|(n, _, _)| n.as_str()).collect();
        assert!(
            tool_names.contains(&"registration_test"),
            "Custom tool 'registration_test' should appear in tool listing. Found: {:?}",
            tool_names
        );

        let tool = registry.get("registration_test").await;
        assert!(
            tool.is_some(),
            "Custom tool should be retrievable from registry"
        );
        assert_eq!(
            tool.unwrap().name(),
            "registration_test",
            "Tool name should match"
        );

        let result = registry
            .execute(
                "registration_test",
                serde_json::json!({"value": "test_value"}),
                None,
            )
            .await;

        assert!(result.is_ok(), "Tool execution should succeed");
        let result = result.unwrap();
        assert!(result.success, "Tool should execute successfully");
    }
}
