use crate::common::MockLLMProvider;
use opencode_agent::{Agent, AgentType, BuildAgent, GeneralAgent, PlanAgent};
use opencode_core::{Message, Session};

#[tokio::test]
async fn test_build_agent_e2e_conversation() {
    let provider = MockLLMProvider::new()
        .with_response("Hello! How can I help you?")
        .with_model("test-model");

    let mut session = Session::new();
    session.add_message(Message::user("Hi".to_string()));

    let agent = BuildAgent::new();
    let tools = opencode_tools::ToolRegistry::new();

    let response = agent
        .run(&mut session, &provider, &tools)
        .await
        .expect("Agent should run successfully");

    assert_eq!(response.content, "Hello! How can I help you?");
    assert!(response.tool_calls.is_empty());
    assert_eq!(session.messages.len(), 2);
    assert_eq!(provider.call_count(), 1);
}

#[tokio::test]
async fn test_plan_agent_readonly_constraint() {
    let provider = MockLLMProvider::new().with_response("I can only read files, not write them.");

    let mut session = Session::new();
    session.add_message(Message::user("Write a file for me".to_string()));

    let agent = PlanAgent::new();
    let tools = opencode_tools::ToolRegistry::new();

    let response = agent
        .run(&mut session, &provider, &tools)
        .await
        .expect("Plan agent should run");

    assert!(response.content.contains("read") || response.content.contains("only"));
    assert!(!agent.can_write_files());
    assert!(!agent.can_run_commands());
    assert!(!agent.can_execute_tools());
}

#[tokio::test]
async fn test_general_agent_search_focused() {
    let provider = MockLLMProvider::new().with_response("Found 3 matching files.");

    let mut session = Session::new();
    session.add_message(Message::user("Find all Rust files".to_string()));

    let agent = GeneralAgent::new();
    let tools = opencode_tools::ToolRegistry::new();

    let response = agent
        .run(&mut session, &provider, &tools)
        .await
        .expect("General agent should run");

    assert!(!response.content.is_empty());
    assert!(!agent.can_write_files());
    assert!(!agent.can_run_commands());
}

#[tokio::test]
async fn test_agent_switch_between_types() {
    let build_provider = MockLLMProvider::new().with_response("Build agent response");
    let plan_provider = MockLLMProvider::new().with_response("Plan agent response");

    let mut session = Session::new();
    session.add_message(Message::user("Hello".to_string()));

    let build_agent = BuildAgent::new();
    let plan_agent = PlanAgent::new();
    let tools = opencode_tools::ToolRegistry::new();

    let build_response = build_agent
        .run(&mut session, &build_provider, &tools)
        .await
        .expect("Build agent should work");

    session.add_message(Message::user("Switch to plan mode".to_string()));

    let plan_response = plan_agent
        .run(&mut session, &plan_provider, &tools)
        .await
        .expect("Plan agent should work");

    assert_eq!(build_response.content, "Build agent response");
    assert_eq!(plan_response.content, "Plan agent response");
    assert!(build_agent.can_write_files());
    assert!(!plan_agent.can_write_files());
}

#[tokio::test]
async fn test_agent_error_propagation() {
    let provider = MockLLMProvider::new().with_error("Connection failed");

    let mut session = Session::new();
    session.add_message(Message::user("Hi".to_string()));

    let agent = BuildAgent::new();
    let tools = opencode_tools::ToolRegistry::new();

    let result = agent.run(&mut session, &provider, &tools).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Connection failed") || err.to_string().contains("LLM"));
}

#[tokio::test]
async fn test_agent_with_system_message() {
    let provider = MockLLMProvider::new().with_response("Response with system context");

    let mut session = Session::new();
    session.add_message(Message::system("You are a helpful assistant".to_string()));
    session.add_message(Message::user("Hello".to_string()));

    let agent = BuildAgent::new();
    let tools = opencode_tools::ToolRegistry::new();

    let response = agent
        .run(&mut session, &provider, &tools)
        .await
        .expect("Agent should run with system message");

    assert!(!response.content.is_empty());
    assert_eq!(session.messages.len(), 3);
}

#[tokio::test]
async fn test_multiple_turn_conversation() {
    let provider = MockLLMProvider::new()
        .with_response("First response")
        .with_response("Second response")
        .with_response("Third response");

    let mut session = Session::new();
    session.add_message(Message::user("Turn 1".to_string()));

    let agent = BuildAgent::new();
    let tools = opencode_tools::ToolRegistry::new();

    let response1 = agent.run(&mut session, &provider, &tools).await.unwrap();
    assert_eq!(response1.content, "First response");

    session.add_message(Message::user("Turn 2".to_string()));
    let response2 = agent.run(&mut session, &provider, &tools).await.unwrap();
    assert_eq!(response2.content, "Second response");

    session.add_message(Message::user("Turn 3".to_string()));
    let response3 = agent.run(&mut session, &provider, &tools).await.unwrap();
    assert_eq!(response3.content, "Third response");

    assert_eq!(provider.call_count(), 3);
}

#[tokio::test]
async fn test_agent_type_enum_values() {
    assert_eq!(BuildAgent::new().agent_type(), AgentType::Build);
    assert_eq!(PlanAgent::new().agent_type(), AgentType::Plan);
    assert_eq!(GeneralAgent::new().agent_type(), AgentType::General);
}

#[tokio::test]
async fn test_agent_metadata() {
    let build = BuildAgent::new();
    assert_eq!(build.name(), "build");
    assert!(build.description().contains("file"));
    assert!(build.can_execute_tools());
    assert!(build.can_write_files());
    assert!(build.can_run_commands());

    let plan = PlanAgent::new();
    assert_eq!(plan.name(), "plan");
    assert!(!plan.description().is_empty());
    assert!(!plan.can_execute_tools());
    assert!(!plan.can_write_files());
    assert!(!plan.can_run_commands());

    let general = GeneralAgent::new();
    assert_eq!(general.name(), "general");
    assert!(general.can_execute_tools());
    assert!(!general.can_write_files());
    assert!(!general.can_run_commands());
}
