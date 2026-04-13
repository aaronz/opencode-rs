use crate::common::TempProject;
use opencode_tools::{register_custom_tools, read::ReadTool, write::WriteTool, Tool, ToolRegistry};

#[tokio::test]
async fn test_tool_registry_register_and_execute() {
    let registry = ToolRegistry::new();
    registry.register(ReadTool::new()).await;

    let result = registry
        .execute(
            "read",
            serde_json::json!({"path": "/nonexistent.txt"}),
            None,
        )
        .await;

    assert!(result.is_err() || result.as_ref().map(|r| !r.success).unwrap_or(false));
}

#[tokio::test]
async fn test_read_tool_file_exists() {
    let project = TempProject::new();
    project.create_file("test.txt", "Hello, World!");

    let tool = ReadTool::new();
    let result = tool
        .execute(
            serde_json::json!({"path": project.path().join("test.txt").to_string_lossy()}),
            None,
        )
        .await
        .expect("Read tool should execute");

    assert!(result.success);
    assert!(result.content.contains("Hello, World!"));
}

#[tokio::test]
async fn test_read_tool_file_not_found() {
    let tool = ReadTool::new();
    let result = tool
        .execute(serde_json::json!({"path": "/nonexistent/file.txt"}), None)
        .await
        .expect("Read tool should execute");

    assert!(!result.success);
    assert!(result.error.is_some());
}

#[tokio::test]
async fn test_write_tool_creates_file() {
    let project = TempProject::new();
    let file_path = project
        .path()
        .join("new_file.txt")
        .to_string_lossy()
        .to_string();

    let tool = WriteTool;
    let result = tool
        .execute(
            serde_json::json!({"path": file_path, "content": "New content"}),
            None,
        )
        .await
        .expect("Write tool should execute");

    assert!(result.success);
    assert!(result.content.contains("Written to"));

    let file_path = project.path().join("new_file.txt");
    assert!(file_path.exists());
    assert_eq!(std::fs::read_to_string(&file_path).unwrap(), "New content");
}

#[tokio::test]
async fn test_write_tool_overwrites_file() {
    let project = TempProject::new();
    project.create_file("existing.txt", "Old content");

    let tool = WriteTool;
    let file_path = project
        .path()
        .join("existing.txt")
        .to_string_lossy()
        .to_string();

    let result = tool
        .execute(
            serde_json::json!({"path": file_path, "content": "New content"}),
            None,
        )
        .await
        .expect("Write tool should execute");

    assert!(result.success);

    let file_path = project.path().join("existing.txt");
    assert_eq!(std::fs::read_to_string(&file_path).unwrap(), "New content");
}

#[tokio::test]
async fn test_tool_execute_invalid_args() {
    let tool = ReadTool::new();
    let result = tool
        .execute(serde_json::json!({"invalid": "args"}), None)
        .await;

    let err: opencode_core::OpenCodeError = result.unwrap_err();
    assert!(err.to_string().contains("missing field") || err.to_string().contains("Tool"));
}

#[tokio::test]
async fn test_tool_registry_nonexistent_tool() {
    let registry = ToolRegistry::new();

    let result = registry
        .execute("nonexistent_tool", serde_json::json!({}), None)
        .await;

    let err: opencode_core::OpenCodeError = result.unwrap_err();
    assert!(err.to_string().contains("not found") || err.to_string().contains("Tool"));
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

    let err: opencode_core::OpenCodeError = result.unwrap_err();
    assert!(err.to_string().contains("disabled") || err.to_string().contains("Tool"));
}

#[tokio::test]
async fn test_tool_clone_works() {
    let tool = ReadTool::new();
    let cloned = tool.clone_tool();

    assert_eq!(cloned.name(), "read");
    assert!(cloned.description().contains("Read"));
}

#[tokio::test]
async fn test_read_tool_with_offset_and_limit() {
    let project = TempProject::new();
    project.create_file("multiline.txt", "Line 1\nLine 2\nLine 3\nLine 4\nLine 5");

    let tool = ReadTool::new();
    let result = tool
        .execute(
            serde_json::json!({
                "path": project.path().join("multiline.txt").to_string_lossy(),
                "offset": 1,
                "limit": 2
            }),
            None,
        )
        .await
        .expect("Read tool should execute");

    assert!(result.success);
    assert!(result.content.contains("Line 2"));
    assert!(result.content.contains("Line 3"));
    assert!(!result.content.contains("Line 1"));
}

#[tokio::test]
async fn test_write_tool_creates_nested_directories() {
    let project = TempProject::new();
    let file_path = project
        .path()
        .join("nested/deep/dir/file.txt")
        .to_string_lossy()
        .to_string();

    let tool = WriteTool;
    let result = tool
        .execute(
            serde_json::json!({"path": file_path, "content": "Nested content"}),
            None,
        )
        .await
        .expect("Write tool should execute");

    assert!(result.success);

    let file_path = project.path().join("nested/deep/dir/file.txt");
    assert!(file_path.exists());
    assert_eq!(
        std::fs::read_to_string(&file_path).unwrap(),
        "Nested content"
    );
}

#[tokio::test]
async fn test_custom_tool_discovery_registration_execution() {
    let project = TempProject::new();
    
    let tools_dir = project.path().join(".opencode/tools");
    std::fs::create_dir_all(&tools_dir).expect("Failed to create tools dir");
    
    let tool_content = r#"
const echoTool = {
    name: "echo",
    description: "Echoes the input back",
    parameters: { type: "object", properties: { message: { type: "string" } }, required: ["message"] }
};
const argsIdx = process.argv.findIndex(a => a === '--args');
const args = argsIdx >= 0 ? JSON.parse(process.argv[argsIdx + 1] || '{}') : {};
console.log(args.message || '');
export default echoTool;
"#;
    
    std::fs::write(tools_dir.join("echo.js"), tool_content)
        .expect("Failed to write tool file");
    
    let registry = ToolRegistry::new();
    
    let registered = register_custom_tools(&registry, Some(project.path().to_path_buf())).await;
    
    assert_eq!(registered.len(), 1, "Should discover exactly one tool");
    assert_eq!(registered[0], "echo", "Tool name should be 'echo'");
    
    let tools = registry.list_filtered(None).await;
    let tool_names: Vec<&str> = tools.iter().map(|(n, _, _)| n.as_str()).collect();
    assert!(
        tool_names.contains(&"echo"),
        "Custom tool 'echo' should appear in tool listing. Found: {:?}",
        tool_names
    );
    
    let result = registry
        .execute("echo", serde_json::json!({"message": "Hello, World!"}), None)
        .await;
    
    assert!(result.is_ok(), "Tool execution should succeed");
    let result = result.unwrap();
    assert!(result.success, "Tool should execute successfully");
    assert!(
        result.content.contains("Hello, World!"),
        "Tool output should contain echoed message. Got: {}",
        result.content
    );
}

#[tokio::test]
async fn test_custom_tool_discovery_multiple_tools() {
    let project = TempProject::new();
    
    let tools_dir = project.path().join(".opencode/tools");
    std::fs::create_dir_all(&tools_dir).expect("Failed to create tools dir");
    
    std::fs::write(
        tools_dir.join("tool1.js"),
        r#"const toolOne = { name: "tool_one", description: "First tool", parameters: {} }; export default toolOne;"#,
    ).expect("Failed to write tool1");
    
    std::fs::write(
        tools_dir.join("tool2.ts"),
        r#"const toolTwo = { name: "tool_two", description: "Second tool", parameters: {} }; export default toolTwo;"#,
    ).expect("Failed to write tool2");
    
    let registry = ToolRegistry::new();
    
    let registered = register_custom_tools(&registry, Some(project.path().to_path_buf())).await;
    
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
async fn test_custom_tool_discovery_no_tools() {
    let project = TempProject::new();
    
    let registry = ToolRegistry::new();
    
    let registered = register_custom_tools(&registry, Some(project.path().to_path_buf())).await;
    
    assert!(registered.is_empty(), "Should discover no tools when none exist");
}
