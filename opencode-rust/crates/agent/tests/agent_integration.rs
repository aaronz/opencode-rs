use opencode_agent::agent::{Agent, AgentType};
use opencode_agent::build_agent::BuildAgent;
use opencode_agent::plan_agent::PlanAgent;
use opencode_agent::system_agents::{CompactionAgent, SummaryAgent, TitleAgent};

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

#[test]
fn test_hidden_agent_compaction_not_visible() {
    let agent = CompactionAgent::new();
    assert!(
        !agent.is_visible(),
        "CompactionAgent should not be visible in standard agent lists"
    );
}

#[test]
fn test_hidden_agent_title_not_visible() {
    let agent = TitleAgent::new();
    assert!(
        !agent.is_visible(),
        "TitleAgent should not be visible in standard agent lists"
    );
}

#[test]
fn test_hidden_agent_summary_not_visible() {
    let agent = SummaryAgent::new();
    assert!(
        !agent.is_visible(),
        "SummaryAgent should not be visible in standard agent lists"
    );
}

#[test]
fn test_visible_agent_build_is_visible() {
    let agent = BuildAgent::new();
    assert!(
        agent.is_visible(),
        "BuildAgent should be visible in standard agent lists"
    );
}

#[test]
fn test_visible_agent_plan_is_visible() {
    let agent = PlanAgent::new();
    assert!(
        agent.is_visible(),
        "PlanAgent should be visible in standard agent lists"
    );
}

#[test]
fn test_hidden_agents_still_have_correct_types() {
    let compaction = CompactionAgent::new();
    let title = TitleAgent::new();
    let summary = SummaryAgent::new();

    assert_eq!(compaction.agent_type(), AgentType::Compaction);
    assert_eq!(title.agent_type(), AgentType::Title);
    assert_eq!(summary.agent_type(), AgentType::Summary);
}

#[test]
fn test_hidden_agents_still_have_names() {
    let compaction = CompactionAgent::new();
    let title = TitleAgent::new();
    let summary = SummaryAgent::new();

    assert_eq!(compaction.name(), "compaction");
    assert_eq!(title.name(), "title");
    assert_eq!(summary.name(), "summary");
}

#[test]
fn test_hidden_agents_have_descriptions() {
    let compaction = CompactionAgent::new();
    let title = TitleAgent::new();
    let summary = SummaryAgent::new();

    assert!(!compaction.description().is_empty());
    assert!(!title.description().is_empty());
    assert!(!summary.description().is_empty());
}
