use crate::common::{MockLLMProvider, TempProject};
use opencode_agent::{Agent, BuildAgent};
use opencode_core::{checkpoint::CheckpointManager, message::Message, revert::RevertManager, Session};
use opencode_mcp::{JsonRpcRequest, JsonRpcResponse};
use opencode_plugin::{Plugin, PluginConfig, PluginDomain, PluginManager, PluginError};
use opencode_tools::build_default_registry;
use opencode_tools::{Tool, ToolRegistry};
use uuid::Uuid;

#[tokio::test]
async fn test_phase6_regression_session_agent_integration() {
    let provider = MockLLMProvider::new()
        .with_response("Integration test response")
        .with_model("test-model");

    let mut session = Session::new();
    session.add_message(Message::user("Phase 6 integration test".to_string()));

    let agent = BuildAgent::new();
    let tools = ToolRegistry::new();

    let response = agent
        .run(&mut session, &provider, &tools)
        .await
        .expect("Agent should run successfully");

    assert!(!response.content.is_empty());
    assert_eq!(session.messages.len(), 2);
}

#[test]
fn test_phase6_regression_session_checkpoint_revert_integration() {
    let project = TempProject::new();

    let mut session = Session::new();
    session.add_message(Message::user("Original message".to_string()));
    session.add_message(Message::assistant("Original response".to_string()));

    let checkpoints_dir = project.path().join("checkpoints");
    let checkpoint_manager = CheckpointManager::new()
        .with_checkpoints_dir(checkpoints_dir.clone())
        .with_max_checkpoints(10);

    let _checkpoint = checkpoint_manager
        .create(&session, "Initial checkpoint")
        .expect("Checkpoint should be created");

    session.add_message(Message::user("New message after checkpoint".to_string()));

    let checkpoints = checkpoint_manager
        .list(&session.id)
        .expect("Should list checkpoints");
    assert!(!checkpoints.is_empty(), "Checkpoints should exist");

    let restored = checkpoint_manager
        .load(&session.id, 0)
        .expect("Should load checkpoint");
    assert_eq!(restored.messages.len(), 2);

    let mut revert_manager = RevertManager::new(5);
    let revert_point = revert_manager.create_point(2, "Revert point".to_string());

    session.add_message(Message::assistant("Extra response".to_string()));
    
    revert_manager
        .revert_to(&mut session, &revert_point.id)
        .expect("Revert should succeed");

    assert_eq!(session.messages.len(), 2);
}

#[tokio::test]
async fn test_phase6_regression_tool_registry_agent_integration() {
    let provider = MockLLMProvider::new()
        .with_response("Tool execution completed")
        .with_model("test-model");

    let mut session = Session::new();
    session.add_message(Message::user("Execute a tool".to_string()));

    let registry = build_default_registry(None).await;

    let tools = registry.list_filtered(None).await;
    assert!(!tools.is_empty(), "Default tools should be registered");

    let agent = BuildAgent::new();

    let response = agent
        .run(&mut session, &provider, &registry)
        .await
        .expect("Agent should run with tools");

    assert!(!response.content.is_empty());
}

#[test]
fn test_phase6_regression_mcp_protocol_session_integration() {
    let request = JsonRpcRequest::new("tools/list", Some(serde_json::json!({})));
    
    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "tools/list");
    
    let response = JsonRpcResponse::success(
        Some(serde_json::json!(1)),
        serde_json::json!({"tools": []}),
    );
    
    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[tokio::test]
async fn test_phase6_regression_message_processing_flow() {
    let provider = MockLLMProvider::new()
        .with_response("First response")
        .with_response("Second response")
        .with_response("Third response");

    let mut session = Session::new();
    
    session.add_message(Message::user("First message".to_string()));
    let agent = BuildAgent::new();
    let tools = ToolRegistry::new();
    
    let response1 = agent.run(&mut session, &provider, &tools).await.unwrap();
    assert_eq!(response1.content, "First response");
    
    session.add_message(Message::user("Second message".to_string()));
    let response2 = agent.run(&mut session, &provider, &tools).await.unwrap();
    assert_eq!(response2.content, "Second response");
    
    session.add_message(Message::user("Third message".to_string()));
    let response3 = agent.run(&mut session, &provider, &tools).await.unwrap();
    assert_eq!(response3.content, "Third response");
    
    assert_eq!(session.messages.len(), 6);
}

#[test]
fn test_phase6_regression_plugin_manager_lifecycle() {
    let mut manager = PluginManager::new();
    
    struct SimplePlugin;
    
    impl Plugin for SimplePlugin {
        fn name(&self) -> &str { "simple-plugin" }
        fn version(&self) -> &str { "1.0.0" }
        fn description(&self) -> &str { "Simple plugin for testing" }
        fn init(&mut self) -> Result<(), PluginError> { Ok(()) }
        fn shutdown(&mut self) -> Result<(), PluginError> { Ok(()) }
    }
    
    manager.register(Box::new(SimplePlugin)).expect("Plugin should register");
    manager.startup().expect("Startup should succeed");
    
    let plugin = manager.get_plugin("simple-plugin");
    assert!(plugin.is_some());
    
    manager.on_start_all().expect("on_start_all should succeed");
    manager.on_message_all("test message", "test-session").expect("on_message should succeed");
    manager.on_session_end_all("test-session").expect("on_session_end should succeed");
    
    manager.shutdown_all().expect("Shutdown should succeed");
}

#[test]
fn test_phase6_regression_full_session_lifecycle() {
    let project = TempProject::new();

    let mut session = Session::new();
    session.add_message(Message::system("You are a helpful assistant".to_string()));
    session.add_message(Message::user("Start a new task".to_string()));
    session.add_message(Message::assistant("I'll help you with that.".to_string()));

    let initial_id = session.id;
    let original_message_count = session.messages.len();

    let checkpoints_dir = project.path().join("checkpoints");
    let checkpoint_manager = CheckpointManager::new()
        .with_checkpoints_dir(checkpoints_dir.clone())
        .with_max_checkpoints(5);

    let _checkpoint = checkpoint_manager
        .create(&session, "Initial checkpoint")
        .unwrap();

    let mut exploration = session.fork(Uuid::new_v4());
    exploration.add_message(Message::user("Explore alternative".to_string()));
    exploration.add_message(Message::assistant("Alternative approach".to_string()));

    let main_path = project.path().join("main_session.json");
    session.save(&main_path).expect("Save main");

    let checkpoints = checkpoint_manager
        .list(&initial_id)
        .expect("List checkpoints");
    assert!(!checkpoints.is_empty());

    let loaded_main = Session::load(&main_path).expect("Load main");
    assert_eq!(loaded_main.id, initial_id);

    let exploration_path = project.path().join("exploration.json");
    exploration.save(&exploration_path).expect("Save exploration");
    let loaded_exploration = Session::load(&exploration_path).expect("Load exploration");

    assert_eq!(
        loaded_exploration.parent_session_id.as_deref(),
        Some(initial_id.to_string().as_str()),
        "Exploration should reference main as parent"
    );

    let mut revert_manager = RevertManager::new(5);
    let revert_point = revert_manager.create_point(original_message_count, "Before exploration".to_string());

    let mut to_revert = Session::load(&main_path).expect("Load for revert");
    to_revert.add_message(Message::user("Extra message".to_string()));

    revert_manager
        .revert_to(&mut to_revert, &revert_point.id)
        .expect("Revert should succeed");

    assert_eq!(to_revert.messages.len(), original_message_count);
}

#[tokio::test]
async fn test_phase6_regression_tool_execution_pipeline() {
    let project = TempProject::new();
    project.create_file("test_file.txt", "Hello, World!");

    let registry = build_default_registry(None).await;
    
    let tools = registry.list_filtered(None).await;
    let tool_names: Vec<&str> = tools.iter().map(|(n, _, _)| n.as_str()).collect();
    
    assert!(
        tool_names.contains(&"read"),
        "Read tool should be available. Found: {:?}",
        tool_names
    );

    let result = registry
        .execute(
            "read",
            serde_json::json!({"path": project.path().join("test_file.txt").to_string_lossy()}),
            None,
        )
        .await;

    assert!(result.is_ok(), "Tool execution should succeed");
    let result = result.unwrap();
    assert!(result.success, "Tool should report success");
    assert!(result.content.contains("Hello, World!"));
}

#[tokio::test]
async fn test_phase6_regression_agent_tool_session_integration() {
    let provider = MockLLMProvider::new()
        .with_response("Response with tool call")
        .with_model("test-model");

    let mut session = Session::new();
    session.add_message(Message::user("Execute tool".to_string()));

    let registry = ToolRegistry::new();
    let agent = BuildAgent::new();

    let response = agent
        .run(&mut session, &provider, &registry)
        .await
        .expect("Agent should run");

    assert!(!response.content.is_empty());
    assert_eq!(session.messages.len(), 2);
}

#[test]
fn test_phase6_regression_multiple_session_operations() {
    let project = TempProject::new();

    let mut session1 = Session::new();
    session1.add_message(Message::user("Session 1 message".to_string()));
    let id1 = session1.id;

    let mut session2 = Session::new();
    session2.add_message(Message::user("Session 2 message".to_string()));
    let id2 = session2.id;

    let path1 = project.path().join("session1.json");
    let path2 = project.path().join("session2.json");

    session1.save(&path1).expect("Session 1 should save");
    session2.save(&path2).expect("Session 2 should save");

    let loaded1 = Session::load(&path1).expect("Session 1 should load");
    let loaded2 = Session::load(&path2).expect("Session 2 should load");

    assert_eq!(loaded1.id, id1);
    assert_eq!(loaded2.id, id2);

    let child1 = session1.fork(Uuid::new_v4());
    assert_ne!(child1.id, session1.id);
    assert_eq!(child1.parent_session_id, Some(id1.to_string()));
}

#[test]
fn test_phase6_regression_compaction_preserves_integrity() {
    let project = TempProject::new();

    let mut session = Session::new();
    session.add_message(Message::user("First message".to_string()));

    for i in 0..50 {
        session.add_message(Message::assistant(format!("Important response {}", i)));
    }

    let original_count = session.messages.len();
    assert!(original_count > 30);

    let result = session.compact_messages(50);

    assert!(result.was_compacted);

    let has_recent = session
        .messages
        .iter()
        .any(|m| m.content.contains("Important response"));
    assert!(has_recent, "Recent messages should be preserved");

    let save_path = project.path().join("compacted.json");
    session.save(&save_path).expect("Should save");

    let loaded = Session::load(&save_path).expect("Should load");
    assert_eq!(
        loaded.messages.len(),
        session.messages.len(),
        "Message count preserved"
    );
}

#[test]
fn test_phase6_regression_fork_message_history_preserved() {
    let mut session = Session::new();
    session.add_message(Message::system("System prompt".to_string()));
    session.add_message(Message::user("User message 1".to_string()));
    session.add_message(Message::assistant("Assistant response 1".to_string()));
    session.add_message(Message::user("User message 2".to_string()));
    session.add_message(Message::assistant("Assistant response 2".to_string()));

    let parent_message_count = session.messages.len();
    let parent_id = session.id;

    let mut child = session.fork(Uuid::new_v4());

    assert_eq!(child.messages.len(), parent_message_count);
    assert_eq!(
        child.parent_session_id.as_deref(),
        Some(parent_id.to_string().as_str())
    );

    child.add_message(Message::assistant("Child continuation".to_string()));

    assert_eq!(session.messages.len(), parent_message_count);
    assert_eq!(child.messages.len(), parent_message_count + 1);
}

#[test]
fn test_phase6_regression_share_mode_persists() {
    let project = TempProject::new();

    let mut session = Session::new();
    session.add_message(Message::user("Share me".to_string()));
    
    use opencode_core::config::ShareMode;
    session.set_share_mode(ShareMode::Manual);
    let _share_link = session.generate_share_link().expect("Should generate link");
    
    let original_share_id = session.shared_id.clone();
    assert!(session.is_shared());

    let save_path = project.path().join("shared.json");
    session.save(&save_path).expect("Should save");

    let loaded = Session::load(&save_path).expect("Should load");
    assert_eq!(
        loaded.shared_id, original_share_id,
        "Share ID should persist"
    );
    assert!(loaded.is_shared());

    session.set_share_mode(ShareMode::Disabled);
    assert!(!session.is_shared());
}

#[test]
fn test_phase6_regression_checkpoints_list_and_load() {
    let project = TempProject::new();

    let mut session = Session::new();
    session.add_message(Message::user("Checkpoint test".to_string()));

    let checkpoints_dir = project.path().join("checkpoints");
    let checkpoint_manager = CheckpointManager::new()
        .with_checkpoints_dir(checkpoints_dir.clone())
        .with_max_checkpoints(10);

    let _cp1 = checkpoint_manager
        .create(&session, "Checkpoint 1")
        .expect("cp1");
    
    session.add_message(Message::assistant("Response".to_string()));

    let _cp2 = checkpoint_manager
        .create(&session, "Checkpoint 2")
        .expect("cp2");

    let checkpoints = checkpoint_manager
        .list(&session.id)
        .expect("Should list checkpoints");
    assert_eq!(checkpoints.len(), 2);

    let restored_cp1 = checkpoint_manager
        .load(&session.id, 0)
        .expect("Should load cp1");
    assert_eq!(restored_cp1.messages.len(), 1);

    let restored_cp2 = checkpoint_manager
        .load(&session.id, 1)
        .expect("Should load cp2");
    assert_eq!(restored_cp2.messages.len(), 2);
}

#[test]
fn test_phase6_regression_undo_redo() {
    let mut session = Session::new();

    session.add_message(Message::user("First".to_string()));
    assert_eq!(session.messages.len(), 1);

    session.add_message(Message::assistant("Second".to_string()));
    assert_eq!(session.messages.len(), 2);

    session.undo(1).expect("Should undo");
    assert_eq!(session.messages.len(), 1);

    session.redo(1).expect("Should redo");
    assert_eq!(session.messages.len(), 2);
}

#[test]
fn test_phase6_regression_export_formats() {
    let mut session = Session::new();
    session.add_message(Message::user("Export me".to_string()));
    session.add_message(Message::assistant("Exported!".to_string()));

    let json = session.export_json().expect("Should export JSON");
    assert!(json.contains("Export me"));
    assert!(json.contains("Exported!"));
    assert!(json.contains("User"));
    assert!(json.contains("Assistant"));

    let md = session.export_markdown().expect("Should export Markdown");
    assert!(md.contains("Export me"));
    assert!(md.contains("**User**"));
    assert!(md.contains("**Assistant**"));
}

#[tokio::test]
async fn test_phase6_regression_multiple_tool_execution() {
    let project = TempProject::new();
    project.create_file("file1.txt", "Content 1");
    project.create_file("file2.txt", "Content 2");

    let registry = build_default_registry(None).await;

    let result1 = registry
        .execute(
            "read",
            serde_json::json!({"path": project.path().join("file1.txt").to_string_lossy()}),
            None,
        )
        .await
        .expect("First read should succeed");

    let result2 = registry
        .execute(
            "read",
            serde_json::json!({"path": project.path().join("file2.txt").to_string_lossy()}),
            None,
        )
        .await
        .expect("Second read should succeed");

    assert!(result1.success);
    assert!(result2.success);
    assert!(result1.content.contains("Content 1"));
    assert!(result2.content.contains("Content 2"));
}

#[test]
fn test_phase6_regression_message_sanitization() {
    let mut session = Session::new();
    let content_with_key = "My API key is sk-1234567890abcdefghij";
    session.add_message(Message::user(content_with_key.to_string()));

    let json = session.export_json().expect("Should export JSON");

    assert!(!json.contains("sk-1234567890"));
    assert!(json.contains("[REDACTED"));
}

#[test]
fn test_phase6_regression_truncate_preserves_recent() {
    let mut session = Session::new();
    session.add_message(Message::user("A".repeat(100)));
    session.add_message(Message::assistant("B".repeat(100)));
    session.add_message(Message::user("C".repeat(100)));
    session.add_message(Message::assistant("D".repeat(100)));

    let original_len = session.messages.len();
    session.truncate_for_context(10);

    assert!(session.messages.len() < original_len);
    assert!(session.messages.len() >= 1);
}

#[tokio::test]
async fn test_phase6_regression_all_default_tools_available() {
    let registry = build_default_registry(None).await;

    let tools = registry.list_filtered(None).await;
    let tool_names: Vec<&str> = tools.iter().map(|(n, _, _)| n.as_str()).collect();

    assert!(tool_names.contains(&"read"), "read tool should be available");
    assert!(tool_names.contains(&"write"), "write tool should be available");
}
