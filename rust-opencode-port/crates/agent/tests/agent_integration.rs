use opencode_agent::agent::{Agent, AgentType};
use opencode_agent::build_agent::BuildAgent;
use opencode_agent::plan_agent::PlanAgent;

#[test]
fn test_plan_agent_is_read_only() {
    let plan_agent = PlanAgent::new();
    assert!(
        !plan_agent.can_write_files(),
        "PlanAgent should not be able to write files"
    );
    assert!(
        !plan_agent.can_run_commands(),
        "PlanAgent should not be able to run commands"
    );
    assert!(
        !plan_agent.can_execute_tools(),
        "PlanAgent should not execute tools automatically"
    );
}

#[test]
fn test_plan_agent_type() {
    let plan_agent = PlanAgent::new();
    assert_eq!(plan_agent.agent_type(), AgentType::Plan);
}

#[test]
fn test_plan_agent_name() {
    let plan_agent = PlanAgent::new();
    assert_eq!(plan_agent.name(), "plan");
}

#[test]
fn test_build_agent_can_write() {
    let build_agent = BuildAgent::new();
    assert!(
        build_agent.can_write_files(),
        "BuildAgent should be able to write files"
    );
    assert!(
        build_agent.can_run_commands(),
        "BuildAgent should be able to run commands"
    );
    assert!(
        build_agent.can_execute_tools(),
        "BuildAgent should execute tools"
    );
}

#[test]
fn test_build_agent_type() {
    let build_agent = BuildAgent::new();
    assert_eq!(build_agent.agent_type(), AgentType::Build);
}

#[test]
fn test_build_agent_name() {
    let build_agent = BuildAgent::new();
    assert_eq!(build_agent.name(), "build");
}

#[test]
fn test_agent_type_display() {
    assert_eq!(format!("{}", AgentType::Plan), "plan");
    assert_eq!(format!("{}", AgentType::Build), "build");
    assert_eq!(format!("{}", AgentType::General), "general");
    assert_eq!(format!("{}", AgentType::Explore), "explore");
}

#[test]
fn test_plan_agent_with_skill_prompt() {
    let plan_agent = PlanAgent::new().with_skill_prompt("custom skill prompt".to_string());
    assert_eq!(plan_agent.agent_type(), AgentType::Plan);
}

#[test]
fn test_agent_type_equality() {
    assert_eq!(AgentType::Plan, AgentType::Plan);
    assert_eq!(AgentType::Build, AgentType::Build);
    assert_ne!(AgentType::Plan, AgentType::Build);
}
