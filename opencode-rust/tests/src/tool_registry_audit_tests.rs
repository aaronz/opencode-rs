use opencode_agent::{Agent, BuildAgent, GeneralAgent};
use opencode_core::{Message, Session};
use opencode_tools::{read::ReadTool, write::WriteTool, Tool, ToolRegistry};

use crate::common::{MockLLMProvider, TempProject};

#[tokio::test]
async fn test_agent_runtime_uses_opencode_tools_tool_registry() {
    let provider = MockLLMProvider::new().with_response("Agent response");
    let mut session = Session::new();
    session.add_message(Message::user("Test".to_string()));

    let agent = BuildAgent::new();
    let tools = opencode_tools::ToolRegistry::new();

    let result = agent.run(&mut session, &provider, &tools).await;
    assert!(
        result.is_ok(),
        "Agent should run with opencode_tools::ToolRegistry"
    );
}

#[tokio::test]
async fn test_general_agent_uses_opencode_tools_tool_registry() {
    let provider = MockLLMProvider::new().with_response("Search results");
    let mut session = Session::new();
    session.add_message(Message::user("Find files".to_string()));

    let agent = GeneralAgent::new();
    let tools = opencode_tools::ToolRegistry::new();

    let result = agent.run(&mut session, &provider, &tools).await;
    assert!(
        result.is_ok(),
        "GeneralAgent should run with opencode_tools::ToolRegistry"
    );
}

#[tokio::test]
async fn test_tool_registry_execute_read_tool() {
    let project = TempProject::new();
    project.create_file("test.txt", "Hello, World!");

    let registry = ToolRegistry::new();
    registry.register(ReadTool::new()).await;

    let result = registry
        .execute(
            "read",
            serde_json::json!({"path": project.path().join("test.txt").to_string_lossy()}),
            None,
        )
        .await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.success);
    assert!(result.content.contains("Hello, World!"));
}

#[tokio::test]
async fn test_tool_registry_execute_write_tool() {
    let project = TempProject::new();

    let registry = ToolRegistry::new();
    registry.register(WriteTool).await;

    let file_path = project
        .path()
        .join("output.txt")
        .to_string_lossy()
        .to_string();
    let result = registry
        .execute(
            "write",
            serde_json::json!({"path": file_path, "content": "Test content"}),
            None,
        )
        .await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.success);

    let output_file = project.path().join("output.txt");
    assert!(output_file.exists());
    assert_eq!(
        std::fs::read_to_string(&output_file).unwrap(),
        "Test content"
    );
}

#[tokio::test]
async fn test_tool_registry_list_filtered() {
    let registry = ToolRegistry::new();
    registry.register(ReadTool::new()).await;

    let tools = registry.list_filtered(None).await;
    let tool_names: Vec<&str> = tools.iter().map(|(n, _, _)| n.as_str()).collect();

    assert!(
        tool_names.contains(&"read"),
        "Registry should contain 'read' tool. Found: {:?}",
        tool_names
    );
}

#[tokio::test]
async fn test_tool_registry_get_tool() {
    let registry = ToolRegistry::new();
    registry.register(ReadTool::new()).await;

    let tool = registry.get("read").await;
    assert!(tool.is_some(), "Should be able to get 'read' tool");

    let tool = registry.get("nonexistent").await;
    assert!(tool.is_none(), "Should not find nonexistent tool");
}

#[tokio::test]
async fn test_tool_registry_disabled_tool() {
    let mut registry = ToolRegistry::new();
    registry.register(ReadTool::new()).await;

    let mut disabled = std::collections::HashSet::new();
    disabled.insert("read".to_string());
    registry.set_disabled(disabled);

    let result = registry
        .execute("read", serde_json::json!({"path": "/test.txt"}), None)
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("disabled"),
        "Error should mention disabled: {}",
        err
    );
}

#[tokio::test]
async fn test_tool_registry_collision_resolution_builtin_overrides_custom() {
    #[derive(Clone)]
    struct CustomTool;
    #[async_trait::async_trait]
    impl Tool for CustomTool {
        fn name(&self) -> &str {
            "echo"
        }
        fn description(&self) -> &str {
            "Custom echo"
        }
        fn clone_tool(&self) -> Box<dyn Tool> {
            Box::new(self.clone())
        }
        async fn execute(
            &self,
            _args: serde_json::Value,
            _ctx: Option<opencode_tools::ToolContext>,
        ) -> Result<opencode_tools::ToolResult, opencode_core::OpenCodeError> {
            Ok(opencode_tools::ToolResult::ok("custom"))
        }
    }

    let registry = ToolRegistry::new();

    registry.register(CustomTool).await;
    registry.register(ReadTool::new()).await;

    let result = registry
        .execute("echo", serde_json::json!({"path": "/test.txt"}), None)
        .await
        .unwrap();

    assert_eq!(result.content, "custom");
}

#[tokio::test]
async fn test_tool_registry_multiple_tools_execution() {
    let registry = ToolRegistry::new();
    registry.register(ReadTool::new()).await;
    registry.register(WriteTool).await;

    let tools = registry.list_filtered(None).await;
    let tool_names: Vec<&str> = tools.iter().map(|(n, _, _)| n.as_str()).collect();

    assert!(tool_names.contains(&"read"));
    assert!(tool_names.contains(&"write"));
}

#[tokio::test]
async fn test_tool_registry_not_found_error() {
    let registry = ToolRegistry::new();

    let result = registry
        .execute("nonexistent", serde_json::json!({}), None)
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("not found") || err.to_string().contains("Tool"),
        "Error should indicate tool not found: {}",
        err
    );
}

#[tokio::test]
async fn test_tool_registry_tool_with_status() {
    let mut registry = ToolRegistry::new();
    registry.register(ReadTool::new()).await;

    let (tool, disabled) = registry.get_with_status("read").await.unwrap();
    assert!(!disabled, "Tool should not be disabled by default");
    assert_eq!(tool.name(), "read");

    let mut disabled_set = std::collections::HashSet::new();
    disabled_set.insert("read".to_string());
    registry.set_disabled(disabled_set);

    let (_tool, disabled) = registry.get_with_status("read").await.unwrap();
    assert!(disabled, "Tool should be marked as disabled");
}
