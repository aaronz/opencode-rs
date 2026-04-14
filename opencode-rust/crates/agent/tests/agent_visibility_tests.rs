use opencode_agent::agent::{Agent, AgentType};
use opencode_agent::build_agent::BuildAgent;
use opencode_agent::debug_agent::DebugAgent;
use opencode_agent::explore_agent::ExploreAgent;
use opencode_agent::general_agent::GeneralAgent;
use opencode_agent::plan_agent::PlanAgent;
use opencode_agent::refactor_agent::RefactorAgent;
use opencode_agent::review_agent::ReviewAgent;
use opencode_agent::system_agents::{CompactionAgent, SummaryAgent, TitleAgent};

fn all_agents() -> Vec<(String, bool)> {
    vec![
        (
            BuildAgent::new().name().to_string(),
            BuildAgent::new().is_visible(),
        ),
        (
            PlanAgent::new().name().to_string(),
            PlanAgent::new().is_visible(),
        ),
        (
            GeneralAgent::new().name().to_string(),
            GeneralAgent::new().is_visible(),
        ),
        (
            ExploreAgent::new().name().to_string(),
            ExploreAgent::new().is_visible(),
        ),
        (
            ReviewAgent::new().name().to_string(),
            ReviewAgent::new().is_visible(),
        ),
        (
            RefactorAgent::new().name().to_string(),
            RefactorAgent::new().is_visible(),
        ),
        (
            DebugAgent::new().name().to_string(),
            DebugAgent::new().is_visible(),
        ),
        (
            CompactionAgent::new().name().to_string(),
            CompactionAgent::new().is_visible(),
        ),
        (
            TitleAgent::new().name().to_string(),
            TitleAgent::new().is_visible(),
        ),
        (
            SummaryAgent::new().name().to_string(),
            SummaryAgent::new().is_visible(),
        ),
    ]
}

fn visible_agents_in_selection() -> Vec<String> {
    all_agents()
        .into_iter()
        .filter(|(_, visible)| *visible)
        .map(|(name, _)| name)
        .collect()
}

fn hidden_agents_not_in_selection() -> Vec<String> {
    all_agents()
        .into_iter()
        .filter(|(_, visible)| !*visible)
        .map(|(name, _)| name)
        .collect()
}

#[test]
fn test_agent_visibility_build_is_visible() {
    let agent = BuildAgent::new();
    assert!(
        agent.is_visible(),
        "BuildAgent must appear in agent selection UI"
    );
}

#[test]
fn test_agent_visibility_plan_is_visible() {
    let agent = PlanAgent::new();
    assert!(
        agent.is_visible(),
        "PlanAgent must appear in agent selection UI"
    );
}

#[test]
fn test_agent_visibility_compaction_is_hidden() {
    let agent = CompactionAgent::new();
    assert!(
        !agent.is_visible(),
        "CompactionAgent must not appear in agent selection UI"
    );
}

#[test]
fn test_agent_visibility_title_is_hidden() {
    let agent = TitleAgent::new();
    assert!(
        !agent.is_visible(),
        "TitleAgent must not appear in agent selection UI"
    );
}

#[test]
fn test_agent_visibility_summary_is_hidden() {
    let agent = SummaryAgent::new();
    assert!(
        !agent.is_visible(),
        "SummaryAgent must not appear in agent selection UI"
    );
}

#[test]
fn test_agent_visibility_all_visible_agents_in_selection_list() {
    let visible = visible_agents_in_selection();
    assert!(
        visible.contains(&"build".to_string()),
        "build agent must be in visible selection list"
    );
    assert!(
        visible.contains(&"plan".to_string()),
        "plan agent must be in visible selection list"
    );
}

#[test]
fn test_agent_visibility_no_hidden_agent_in_selection_list() {
    let visible = visible_agents_in_selection();
    assert!(
        !visible.contains(&"compaction".to_string()),
        "compaction agent must not be in visible selection list"
    );
    assert!(
        !visible.contains(&"title".to_string()),
        "title agent must not be in visible selection list"
    );
    assert!(
        !visible.contains(&"summary".to_string()),
        "summary agent must not be in visible selection list"
    );
}

#[test]
fn test_agent_visibility_hidden_agents_not_in_selection() {
    let hidden = hidden_agents_not_in_selection();
    assert!(hidden.contains(&"compaction".to_string()));
    assert!(hidden.contains(&"title".to_string()));
    assert!(hidden.contains(&"summary".to_string()));

    let visible = visible_agents_in_selection();
    for name in &hidden {
        assert!(
            !visible.contains(name),
            "hidden agent '{}' must not appear in visible selection list",
            name
        );
    }
}

#[test]
fn test_agent_visibility_build_type_matches() {
    let agent = BuildAgent::new();
    assert_eq!(agent.agent_type(), AgentType::Build);
    assert!(agent.is_visible());
}

#[test]
fn test_agent_visibility_plan_type_matches() {
    let agent = PlanAgent::new();
    assert_eq!(agent.agent_type(), AgentType::Plan);
    assert!(agent.is_visible());
}

#[test]
fn test_agent_visibility_compaction_type_matches() {
    let agent = CompactionAgent::new();
    assert_eq!(agent.agent_type(), AgentType::Compaction);
    assert!(!agent.is_visible());
}

#[test]
fn test_agent_visibility_title_type_matches() {
    let agent = TitleAgent::new();
    assert_eq!(agent.agent_type(), AgentType::Title);
    assert!(!agent.is_visible());
}

#[test]
fn test_agent_visibility_summary_type_matches() {
    let agent = SummaryAgent::new();
    assert_eq!(agent.agent_type(), AgentType::Summary);
    assert!(!agent.is_visible());
}

#[test]
fn test_agent_visibility_general_agent_is_visible() {
    let agent = GeneralAgent::new();
    assert!(agent.is_visible(), "GeneralAgent should be visible");
}

#[test]
fn test_agent_visibility_explore_agent_is_visible() {
    let agent = ExploreAgent::new();
    assert!(agent.is_visible(), "ExploreAgent should be visible");
}

#[test]
fn test_agent_visibility_state_persists_after_creation() {
    let build1 = BuildAgent::new();
    let build2 = BuildAgent::new();
    assert_eq!(
        build1.is_visible(),
        build2.is_visible(),
        "BuildAgent visibility state must be consistent across instances"
    );

    let compaction1 = CompactionAgent::new();
    let compaction2 = CompactionAgent::new();
    assert_eq!(
        compaction1.is_visible(),
        compaction2.is_visible(),
        "CompactionAgent visibility state must be consistent across instances"
    );

    let title1 = TitleAgent::new();
    let title2 = TitleAgent::new();
    assert_eq!(
        title1.is_visible(),
        title2.is_visible(),
        "TitleAgent visibility state must be consistent across instances"
    );

    let summary1 = SummaryAgent::new();
    let summary2 = SummaryAgent::new();
    assert_eq!(
        summary1.is_visible(),
        summary2.is_visible(),
        "SummaryAgent visibility state must be consistent across instances"
    );
}

#[test]
fn test_agent_visibility_state_persists_with_model_override() {
    let base = BuildAgent::new();
    let with_model = BuildAgent::new().with_model("gpt-4o");

    assert_eq!(
        base.is_visible(),
        with_model.is_visible(),
        "BuildAgent visibility must not change when model is overridden"
    );

    let base_compaction = CompactionAgent::new();
    let with_model_compaction = CompactionAgent::new().with_model("gpt-4o-mini");

    assert_eq!(
        base_compaction.is_visible(),
        with_model_compaction.is_visible(),
        "CompactionAgent visibility must not change when model is overridden"
    );
}

#[test]
fn test_agent_visibility_ui_correct_agent_list() {
    let visible = visible_agents_in_selection();

    assert!(!visible.is_empty(), "visible agent list must not be empty");

    let hidden_names = ["compaction", "title", "summary"];
    for name in &hidden_names {
        assert!(
            !visible.iter().any(|v| v == name),
            "hidden agent '{}' must not appear in the UI agent list",
            name
        );
    }

    let expected_visible = ["build", "plan"];
    for name in &expected_visible {
        assert!(
            visible.iter().any(|v| v == *name),
            "visible agent '{}' must appear in the UI agent list",
            name
        );
    }
}
