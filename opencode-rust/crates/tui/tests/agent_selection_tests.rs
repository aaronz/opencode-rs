use opencode_tui::command::{CommandAction, CommandRegistry};

fn agent_selection_commands(registry: &CommandRegistry) -> Vec<String> {
    registry
        .all()
        .iter()
        .filter(|cmd| matches!(&cmd.action, CommandAction::SetMode(_)))
        .map(|cmd| cmd.name.clone())
        .collect()
}

fn get_set_mode_value(registry: &CommandRegistry, name: &str) -> Option<String> {
    registry.get_by_name(name).and_then(|cmd| {
        if let CommandAction::SetMode(mode) = &cmd.action {
            Some(mode.clone())
        } else {
            None
        }
    })
}

#[test]
fn test_agent_selection_plan_command_present() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("plan");
    assert!(
        cmd.is_some(),
        "plan agent command must be present in command registry"
    );
}

#[test]
fn test_agent_selection_build_command_present() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("build");
    assert!(
        cmd.is_some(),
        "build agent command must be present in command registry"
    );
}

#[test]
fn test_agent_selection_plan_sets_mode() {
    let registry = CommandRegistry::new();
    let mode = get_set_mode_value(&registry, "plan");
    assert_eq!(
        mode,
        Some("plan".to_string()),
        "plan command must set mode to 'plan'"
    );
}

#[test]
fn test_agent_selection_build_sets_mode() {
    let registry = CommandRegistry::new();
    let mode = get_set_mode_value(&registry, "build");
    assert_eq!(
        mode,
        Some("build".to_string()),
        "build command must set mode to 'build'"
    );
}

#[test]
fn test_agent_selection_compaction_not_in_registry() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("compaction");
    assert!(
        cmd.is_none(),
        "compaction (hidden agent) must not appear in command registry as a selection option"
    );
}

#[test]
fn test_agent_selection_title_not_in_registry() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("title");
    assert!(
        cmd.is_none(),
        "title (hidden agent) must not appear in command registry as a selection option"
    );
}

#[test]
fn test_agent_selection_summary_not_in_registry() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("summary");
    assert!(
        cmd.is_none(),
        "summary (hidden agent) must not appear in command registry as a selection option"
    );
}

#[test]
fn test_agent_selection_visible_agents_in_set_mode_commands() {
    let registry = CommandRegistry::new();
    let selections = agent_selection_commands(&registry);

    assert!(
        selections.contains(&"plan".to_string()),
        "plan must be in agent selection commands"
    );
    assert!(
        selections.contains(&"build".to_string()),
        "build must be in agent selection commands"
    );
}

#[test]
fn test_agent_selection_hidden_agents_not_in_set_mode_commands() {
    let registry = CommandRegistry::new();
    let selections = agent_selection_commands(&registry);

    assert!(
        !selections.contains(&"compaction".to_string()),
        "compaction must not be in agent selection commands"
    );
    assert!(
        !selections.contains(&"title".to_string()),
        "title must not be in agent selection commands"
    );
    assert!(
        !selections.contains(&"summary".to_string()),
        "summary must not be in agent selection commands"
    );
}

#[test]
fn test_agent_selection_ui_correct_list() {
    let registry = CommandRegistry::new();
    let selections = agent_selection_commands(&registry);

    assert!(
        !selections.is_empty(),
        "agent selection list must not be empty"
    );

    let expected = ["build", "plan"];
    for name in &expected {
        assert!(
            selections.iter().any(|s| s == name),
            "expected agent '{}' must appear in UI selection list",
            name
        );
    }

    let hidden = ["compaction", "title", "summary"];
    for name in &hidden {
        assert!(
            !selections.iter().any(|s| s == name),
            "hidden agent '{}' must not appear in UI selection list",
            name
        );
    }
}

#[test]
fn test_agent_selection_plan_alias_available() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("p");
    assert!(cmd.is_some(), "plan agent must be accessible via alias 'p'");
    assert_eq!(cmd.unwrap().name, "plan");
}

#[test]
fn test_agent_selection_build_alias_available() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("b");
    assert!(
        cmd.is_some(),
        "build agent must be accessible via alias 'b'"
    );
    assert_eq!(cmd.unwrap().name, "build");
}

#[test]
fn test_agent_selection_compaction_alias_not_present() {
    let registry = CommandRegistry::new();
    assert!(registry.get_by_name("compact-agent").is_none());
    assert!(registry.get_by_name("ctx-compress").is_none());
}

#[test]
fn test_agent_selection_filter_shows_visible_agents() {
    let registry = CommandRegistry::new();

    let results_p = registry.find("p");
    let plan_found = results_p.iter().any(|c| c.name == "plan");
    assert!(plan_found, "filtering by 'p' should show plan agent");

    let results_b = registry.find("b");
    let build_found = results_b.iter().any(|c| c.name == "build");
    assert!(build_found, "filtering by 'b' should show build agent");
}

#[test]
fn test_agent_selection_filter_does_not_show_hidden_agents() {
    let registry = CommandRegistry::new();

    let results_c = registry.find("compaction");
    assert!(
        results_c.is_empty(),
        "filtering by 'compaction' must return nothing"
    );

    let results_t = registry.find("title");
    assert!(
        results_t.is_empty(),
        "filtering by 'title' must return nothing"
    );

    let results_s = registry.find("summary");
    assert!(
        results_s.is_empty(),
        "filtering by 'summary' must return nothing"
    );
}

#[test]
fn test_agent_selection_plan_description_mentions_agent() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("plan").unwrap();
    let desc_lower = cmd.description.to_lowercase();
    assert!(
        desc_lower.contains("plan") || desc_lower.contains("agent") || desc_lower.contains("read"),
        "plan command description must mention its nature: '{}'",
        cmd.description
    );
}

#[test]
fn test_agent_selection_build_description_mentions_agent() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("build").unwrap();
    let desc_lower = cmd.description.to_lowercase();
    assert!(
        desc_lower.contains("build") || desc_lower.contains("agent") || desc_lower.contains("full"),
        "build command description must mention its nature: '{}'",
        cmd.description
    );
}
