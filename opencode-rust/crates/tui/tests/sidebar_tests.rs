use opencode_tui::components::sidebar::{Sidebar, SidebarSectionState, SidebarSectionType};
use tempfile::TempDir;

#[test]
fn test_sidebar_visibility_toggle() {
    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);

    assert!(!sidebar.collapsed, "Sidebar should start uncollapsed");

    sidebar.toggle_collapse();
    assert!(
        sidebar.collapsed,
        "Sidebar should be collapsed after toggle"
    );

    sidebar.toggle_collapse();
    assert!(
        !sidebar.collapsed,
        "Sidebar should be uncollapsed after second toggle"
    );
}

#[test]
fn test_sidebar_section_collapse_independently() {
    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);

    assert!(!sidebar.sections()[0].collapsed);
    assert!(!sidebar.sections()[1].collapsed);
    assert!(!sidebar.sections()[2].collapsed);

    sidebar.collapse_section(SidebarSectionType::Files);
    assert!(sidebar.sections()[0].collapsed);
    assert!(!sidebar.sections()[1].collapsed);
    assert!(!sidebar.sections()[2].collapsed);

    sidebar.collapse_section(SidebarSectionType::Diagnostics);
    assert!(sidebar.sections()[0].collapsed);
    assert!(sidebar.sections()[1].collapsed);
    assert!(!sidebar.sections()[2].collapsed);

    sidebar.expand_section(SidebarSectionType::Files);
    assert!(!sidebar.sections()[0].collapsed);
    assert!(sidebar.sections()[1].collapsed);
    assert!(!sidebar.sections()[2].collapsed);
}

#[test]
fn test_sidebar_toggle_section() {
    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);

    assert!(!sidebar.sections()[0].collapsed);

    sidebar.toggle_section(SidebarSectionType::Files);
    assert!(sidebar.sections()[0].collapsed);

    sidebar.toggle_section(SidebarSectionType::Files);
    assert!(!sidebar.sections()[0].collapsed);
}

#[test]
fn test_file_tree_content_display() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir(tmp.path().join("src")).unwrap();
    std::fs::write(tmp.path().join("src").join("main.rs"), "fn main() {}").unwrap();
    std::fs::write(tmp.path().join("Cargo.toml"), "[package]").unwrap();

    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);
    sidebar.set_file_tree_root(tmp.path().to_path_buf());

    let files_section = sidebar
        .sections()
        .iter()
        .find(|s| s.section_type == SidebarSectionType::Files)
        .expect("Files section should exist");

    let file_tree = files_section
        .file_tree
        .as_ref()
        .expect("File tree should exist in Files section");

    assert!(!file_tree.items.is_empty(), "File tree should have items");

    let has_src_dir = file_tree.items.iter().any(|item| item.name == "src");
    let has_main_rs = file_tree.items.iter().any(|item| item.name == "main.rs");
    let has_cargo_toml = file_tree.items.iter().any(|item| item.name == "Cargo.toml");

    assert!(has_src_dir, "File tree should contain src directory");
    assert!(has_main_rs, "File tree should contain main.rs");
    assert!(has_cargo_toml, "File tree should contain Cargo.toml");

    let src_item = file_tree
        .items
        .iter()
        .find(|item| item.name == "src")
        .expect("src directory should exist");
    assert!(src_item.is_dir, "src should be marked as directory");
}

#[test]
fn test_file_tree_navigation() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir(tmp.path().join("subdir")).unwrap();
    std::fs::write(tmp.path().join("file.txt"), "content").unwrap();

    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);

    let files_section = sidebar
        .sections_mut()
        .iter_mut()
        .find(|s| s.section_type == SidebarSectionType::Files)
        .unwrap();

    let file_tree = files_section.file_tree.as_mut().unwrap();

    assert!(
        file_tree.state.selected().is_some(),
        "File tree should have a selection"
    );

    let initial_selected = file_tree.state.selected();

    file_tree.select_next();
    assert_ne!(
        file_tree.state.selected(),
        initial_selected,
        "select_next should change selection"
    );

    file_tree.select_previous();
    assert_eq!(
        file_tree.state.selected(),
        initial_selected,
        "select_previous should return to initial selection"
    );
}

#[test]
fn test_file_tree_toggle_expand() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir(tmp.path().join("expandable_dir")).unwrap();
    std::fs::write(
        tmp.path().join("expandable_dir").join("nested.rs"),
        "nested",
    )
    .unwrap();

    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);

    let files_section = sidebar
        .sections_mut()
        .iter_mut()
        .find(|s| s.section_type == SidebarSectionType::Files)
        .unwrap();

    let file_tree = files_section.file_tree.as_mut().unwrap();

    let dir_item_idx = file_tree
        .items
        .iter()
        .position(|item| item.name == "expandable_dir");

    if let Some(idx) = dir_item_idx {
        file_tree.state.select(Some(idx));
        let item_before_toggle = file_tree.items.get(idx).unwrap();
        let expanded_before = item_before_toggle.is_expanded;

        file_tree.toggle_current();

        let item_after_toggle = file_tree.items.get(idx).unwrap();
        assert_ne!(
            item_after_toggle.is_expanded, expanded_before,
            "Toggle should change expansion state"
        );
    }
}

#[test]
fn test_lsp_status_display_updates() {
    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);

    let lsp_section = sidebar
        .sections()
        .iter()
        .find(|s| s.section_type == SidebarSectionType::LspStatus)
        .expect("LSP Status section should exist");

    assert!(
        lsp_section.lsp_status.is_empty(),
        "LSP status should initially be empty"
    );

    sidebar.set_lsp_status("LSP connected to rust-analyzer".to_string());

    let updated_lsp_section = sidebar
        .sections()
        .iter()
        .find(|s| s.section_type == SidebarSectionType::LspStatus)
        .expect("LSP Status section should exist");

    assert_eq!(
        updated_lsp_section.lsp_status, "LSP connected to rust-analyzer",
        "LSP status should be updated"
    );

    sidebar.set_lsp_status("LSP not connected".to_string());

    let cleared_lsp_section = sidebar
        .sections()
        .iter()
        .find(|s| s.section_type == SidebarSectionType::LspStatus)
        .expect("LSP Status section should exist");

    assert_eq!(
        cleared_lsp_section.lsp_status, "LSP not connected",
        "LSP status should be cleared"
    );
}

#[test]
fn test_lsp_status_section_titles() {
    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);

    sidebar.set_lsp_status("connected".to_string());

    let lsp_section = sidebar
        .sections()
        .iter()
        .find(|s| s.section_type == SidebarSectionType::LspStatus)
        .expect("LSP Status section should exist");

    assert_eq!(
        lsp_section.section_type.title(),
        "LSP Status",
        "LSP Status section should have correct title"
    );
}

#[test]
fn test_diagnostics_display_shows_errors_warnings() {
    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);

    let diagnostics_section = sidebar
        .sections()
        .iter()
        .find(|s| s.section_type == SidebarSectionType::Diagnostics)
        .expect("Diagnostics section should exist");

    assert!(
        diagnostics_section.diagnostics.is_empty(),
        "Diagnostics should initially be empty"
    );

    let diagnostics = vec![
        "[ERROR] unused variable: `x` at src/main.rs:5:10".to_string(),
        "[WARNING] unused import: `std::fmt` at src/main.rs:2:1".to_string(),
        "[ERROR] expected `;` at src/main.rs:10:15".to_string(),
    ];

    sidebar.set_diagnostics(diagnostics.clone());

    let updated_diagnostics_section = sidebar
        .sections()
        .iter()
        .find(|s| s.section_type == SidebarSectionType::Diagnostics)
        .expect("Diagnostics section should exist");

    assert_eq!(
        updated_diagnostics_section.diagnostics.len(),
        3,
        "Diagnostics should have 3 items"
    );

    assert!(
        updated_diagnostics_section
            .diagnostics
            .iter()
            .any(|d: &String| d.contains("ERROR")),
        "Diagnostics should contain errors"
    );
    assert!(
        updated_diagnostics_section
            .diagnostics
            .iter()
            .any(|d: &String| d.contains("WARNING")),
        "Diagnostics should contain warnings"
    );
}

#[test]
fn test_diagnostics_cleared_after_update() {
    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);

    sidebar.set_diagnostics(vec!["[ERROR] test error".to_string()]);

    let section = sidebar
        .sections()
        .iter()
        .find(|s| s.section_type == SidebarSectionType::Diagnostics)
        .unwrap();

    assert!(!section.diagnostics.is_empty());

    sidebar.set_diagnostics(vec![]);

    let cleared_section = sidebar
        .sections()
        .iter()
        .find(|s| s.section_type == SidebarSectionType::Diagnostics)
        .unwrap();

    assert!(
        cleared_section.diagnostics.is_empty(),
        "Diagnostics should be cleared"
    );
}

#[test]
fn test_diagnostics_section_titles() {
    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);

    sidebar.set_diagnostics(vec!["[ERROR] test".to_string()]);

    let section = sidebar
        .sections_mut()
        .iter_mut()
        .find(|s| s.section_type == SidebarSectionType::Diagnostics)
        .unwrap();

    assert_eq!(
        section.section_type.title(),
        "Diagnostics",
        "Diagnostics section should have correct title"
    );
}

#[test]
fn test_sidebar_section_state_serialization() {
    let state = SidebarSectionState::new(SidebarSectionType::Files);
    let json = serde_json::to_string(&state).unwrap();
    let decoded: SidebarSectionState = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.section_type, SidebarSectionType::Files);
    assert_eq!(decoded.collapsed, state.collapsed);
}

#[test]
fn test_sidebar_get_set_section_state() {
    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);

    let states = sidebar.get_section_state();
    assert_eq!(states.len(), 3);
    assert!(states.iter().all(|s| !s.collapsed));

    sidebar.collapse_section(SidebarSectionType::Files);
    sidebar.collapse_section(SidebarSectionType::Diagnostics);

    let collapsed_states: Vec<SidebarSectionState> = vec![
        SidebarSectionState {
            section_type: SidebarSectionType::Files,
            collapsed: true,
        },
        SidebarSectionState {
            section_type: SidebarSectionType::Diagnostics,
            collapsed: true,
        },
        SidebarSectionState {
            section_type: SidebarSectionType::LspStatus,
            collapsed: false,
        },
    ];

    sidebar.set_section_state(&collapsed_states);

    let files = sidebar
        .sections()
        .iter()
        .find(|s| s.section_type == SidebarSectionType::Files)
        .unwrap();
    let diagnostics = sidebar
        .sections()
        .iter()
        .find(|s| s.section_type == SidebarSectionType::Diagnostics)
        .unwrap();
    let lsp = sidebar
        .sections()
        .iter()
        .find(|s| s.section_type == SidebarSectionType::LspStatus)
        .unwrap();

    assert!(files.collapsed);
    assert!(diagnostics.collapsed);
    assert!(!lsp.collapsed);
}

#[test]
fn test_sidebar_state_persistence() {
    let tmp = TempDir::new().unwrap();
    let state_file = tmp.path().join("sidebar_state.json");

    {
        let theme = opencode_tui::theme::Theme::default();
        let mut sidebar = Sidebar::new(theme);

        sidebar.collapse_section(SidebarSectionType::Files);
        sidebar.collapse_section(SidebarSectionType::Diagnostics);

        sidebar
            .save_to_file(&state_file)
            .expect("Should save sidebar state");

        assert!(state_file.exists(), "State file should be created");
    }

    {
        let theme = opencode_tui::theme::Theme::default();
        let mut sidebar = Sidebar::new(theme);

        assert!(!sidebar.sections()[0].collapsed);
        assert!(!sidebar.sections()[1].collapsed);

        sidebar
            .load_from_file(&state_file)
            .expect("Should load sidebar state");

        let files = sidebar
            .sections()
            .iter()
            .find(|s| s.section_type == SidebarSectionType::Files)
            .unwrap();
        let diagnostics = sidebar
            .sections()
            .iter()
            .find(|s| s.section_type == SidebarSectionType::Diagnostics)
            .unwrap();
        let lsp = sidebar
            .sections()
            .iter()
            .find(|s| s.section_type == SidebarSectionType::LspStatus)
            .unwrap();

        assert!(
            files.collapsed,
            "Files section should be collapsed after load"
        );
        assert!(
            diagnostics.collapsed,
            "Diagnostics section should be collapsed after load"
        );
        assert!(!lsp.collapsed, "LSP section should remain uncollapsed");
    }
}

#[test]
fn test_sidebar_state_persistence_empty_file() {
    let tmp = TempDir::new().unwrap();
    let state_file = tmp.path().join("empty_sidebar_state.json");

    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);

    let result = sidebar.load_from_file(&state_file);
    assert!(
        result.is_ok(),
        "Loading from non-existent file should succeed"
    );

    assert!(
        !sidebar.sections()[0].collapsed,
        "Section should remain uncollapsed when file doesn't exist"
    );
}

#[test]
fn test_sidebar_section_type_next_prev() {
    assert_eq!(
        SidebarSectionType::Files.next(),
        SidebarSectionType::Diagnostics
    );
    assert_eq!(
        SidebarSectionType::Diagnostics.next(),
        SidebarSectionType::LspStatus
    );
    assert_eq!(
        SidebarSectionType::LspStatus.next(),
        SidebarSectionType::Files
    );

    assert_eq!(
        SidebarSectionType::Files.prev(),
        SidebarSectionType::LspStatus
    );
    assert_eq!(
        SidebarSectionType::Diagnostics.prev(),
        SidebarSectionType::Files
    );
    assert_eq!(
        SidebarSectionType::LspStatus.prev(),
        SidebarSectionType::Diagnostics
    );
}

#[test]
fn test_sidebar_section_type_titles() {
    assert_eq!(SidebarSectionType::Files.title(), "Files");
    assert_eq!(SidebarSectionType::Diagnostics.title(), "Diagnostics");
    assert_eq!(SidebarSectionType::LspStatus.title(), "LSP Status");
}

#[test]
fn test_sidebar_section_navigation() {
    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);

    assert_eq!(sidebar.active_section_index(), 0);

    sidebar.next_section();
    assert_eq!(sidebar.active_section_index(), 1);

    sidebar.next_section();
    assert_eq!(sidebar.active_section_index(), 2);

    sidebar.next_section();
    assert_eq!(sidebar.active_section_index(), 0);

    sidebar.prev_section();
    assert_eq!(sidebar.active_section_index(), 2);

    sidebar.prev_section();
    assert_eq!(sidebar.active_section_index(), 1);
}

#[test]
fn test_sidebar_set_active_section() {
    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);

    sidebar.set_active_section(2);
    assert_eq!(sidebar.active_section_index(), 2);

    sidebar.set_active_section(1);
    assert_eq!(sidebar.active_section_index(), 1);

    sidebar.set_active_section(0);
    assert_eq!(sidebar.active_section_index(), 0);

    sidebar.set_active_section(99);
    assert_eq!(
        sidebar.active_section_index(),
        0,
        "Setting out of bounds index should not change active section"
    );
}

#[test]
fn test_sidebar_toggle_active_collapse() {
    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);

    assert!(!sidebar.active_section().collapsed);

    sidebar.toggle_active_collapse();
    assert!(sidebar.active_section().collapsed);

    sidebar.toggle_active_collapse();
    assert!(!sidebar.active_section().collapsed);
}

#[test]
fn test_sidebar_all_sections_accessible() {
    let theme = opencode_tui::theme::Theme::default();
    let sidebar = Sidebar::new(theme);

    assert_eq!(sidebar.sections().len(), 3);

    let section_types: Vec<_> = sidebar.sections().iter().map(|s| s.section_type).collect();

    assert!(section_types.contains(&SidebarSectionType::Files));
    assert!(section_types.contains(&SidebarSectionType::Diagnostics));
    assert!(section_types.contains(&SidebarSectionType::LspStatus));
}

#[test]
fn test_sidebar_file_tree_items_structure() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir(tmp.path().join("a")).unwrap();
    std::fs::create_dir(tmp.path().join("b")).unwrap();
    std::fs::write(tmp.path().join("file.rs"), "").unwrap();

    let theme = opencode_tui::theme::Theme::default();
    let sidebar = Sidebar::new(theme);

    let files_section = sidebar
        .sections()
        .iter()
        .find(|s| s.section_type == SidebarSectionType::Files)
        .unwrap();

    let file_tree = files_section.file_tree.as_ref().unwrap();

    for item in &file_tree.items {
        assert!(!item.name.is_empty(), "Each item should have a name");
        assert!(
            item.path.exists() || item.path.to_string_lossy() == tmp.path().to_string_lossy(),
            "Path should be valid"
        );
    }

    let dirs: Vec<_> = file_tree.items.iter().filter(|i| i.is_dir).collect();
    let files: Vec<_> = file_tree.items.iter().filter(|i| !i.is_dir).collect();

    assert!(!dirs.is_empty(), "Should have directory items");
    assert!(!files.is_empty(), "Should have file items");
}

#[test]
fn test_sidebar_multiple_lsp_status_updates() {
    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);

    let statuses = vec![
        "Initializing...".to_string(),
        "Loading packages".to_string(),
        "rust-analyzer v1.0.0".to_string(),
        "Connected".to_string(),
    ];

    for status in &statuses {
        sidebar.set_lsp_status(status.clone());

        let section = sidebar
            .sections()
            .iter()
            .find(|s| s.section_type == SidebarSectionType::LspStatus)
            .unwrap();

        assert_eq!(&section.lsp_status, status);
    }
}

#[test]
fn test_sidebar_multiple_diagnostics_updates() {
    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);

    sidebar.set_diagnostics(vec!["[ERROR] Error 1".to_string()]);
    assert_eq!(
        sidebar
            .sections()
            .iter()
            .find(|s| s.section_type == SidebarSectionType::Diagnostics)
            .unwrap()
            .diagnostics
            .len(),
        1
    );

    sidebar.set_diagnostics(vec![
        "[ERROR] Error 1".to_string(),
        "[ERROR] Error 2".to_string(),
    ]);
    assert_eq!(
        sidebar
            .sections()
            .iter()
            .find(|s| s.section_type == SidebarSectionType::Diagnostics)
            .unwrap()
            .diagnostics
            .len(),
        2
    );

    sidebar.set_diagnostics(vec![
        "[ERROR] Error 1".to_string(),
        "[WARNING] Warning 1".to_string(),
        "[WARNING] Warning 2".to_string(),
        "[ERROR] Error 2".to_string(),
    ]);
    let diagnostics_section = sidebar
        .sections()
        .iter()
        .find(|s| s.section_type == SidebarSectionType::Diagnostics)
        .unwrap();
    assert_eq!(diagnostics_section.diagnostics.len(), 4);
    assert!(
        diagnostics_section
            .diagnostics
            .iter()
            .filter(|d: &&String| d.contains("ERROR"))
            .count()
            == 2
    );
    assert!(
        diagnostics_section
            .diagnostics
            .iter()
            .filter(|d: &&String| d.contains("WARNING"))
            .count()
            == 2
    );
}

#[test]
fn test_sidebar_file_tree_depth_tracking() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join("level1").join("level2")).unwrap();
    std::fs::write(tmp.path().join("root.txt"), "").unwrap();
    std::fs::write(tmp.path().join("level1").join("l1.txt"), "").unwrap();
    std::fs::write(tmp.path().join("level1").join("level2").join("l2.txt"), "").unwrap();

    let theme = opencode_tui::theme::Theme::default();
    let mut sidebar = Sidebar::new(theme);
    sidebar.set_file_tree_root(tmp.path().to_path_buf());

    let files_section = sidebar
        .sections()
        .iter()
        .find(|s| s.section_type == SidebarSectionType::Files)
        .unwrap();

    let file_tree = files_section.file_tree.as_ref().unwrap();

    let root_dir = file_tree
        .items
        .iter()
        .find(|i| i.name == tmp.path().file_name().unwrap().to_string_lossy())
        .unwrap();
    assert_eq!(root_dir.depth, 0, "Root directory should have depth 0");
    assert!(root_dir.is_dir, "Root should be marked as directory");

    let root = file_tree
        .items
        .iter()
        .find(|i| i.name == "root.txt")
        .unwrap();
    assert_eq!(root.depth, 1, "Root file should have depth 1");

    let l1 = file_tree.items.iter().find(|i| i.name == "l1.txt").unwrap();
    assert_eq!(l1.depth, 2, "Level 1 file should have depth 2");

    let l2 = file_tree.items.iter().find(|i| i.name == "l2.txt").unwrap();
    assert_eq!(l2.depth, 3, "Level 2 file should have depth 3");
}
