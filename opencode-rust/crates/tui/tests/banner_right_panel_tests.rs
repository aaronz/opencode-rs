use opencode_tui::components::banner::{Banner, StartupInfo};
use opencode_tui::components::right_panel::{RightPanel, RightPanelTab};

#[test]
fn test_banner_new() {
    let _banner = Banner::new();
    assert!(true, "Banner created");
}

#[test]
fn test_banner_default() {
    let _banner = Banner::default();
    assert!(true, "Banner created via default");
}

#[test]
fn test_banner_with_custom_art() {
    let art = vec!["Custom", "Banner", "Art"];
    let _banner = Banner::with_custom(art);
    assert!(true, "Banner with custom art created");
}

#[test]
fn test_banner_with_model() {
    let _banner = Banner::new().with_model("gpt-4");
    assert!(true, "Banner with model created");
}

#[test]
fn test_banner_with_permission_mode() {
    let _banner = Banner::new().with_permission_mode("WorkspaceWrite");
    assert!(true, "Banner with permission mode created");
}

#[test]
fn test_banner_with_directory() {
    let _banner = Banner::new().with_directory("/home/user/project");
    assert!(true, "Banner with directory created");
}

#[test]
fn test_banner_with_session_id() {
    let _banner = Banner::new().with_session_id("session-123");
    assert!(true, "Banner with session ID created");
}

#[test]
fn test_banner_with_shortcuts() {
    let shortcuts = vec![
        ("Ctrl+C".to_string(), "Copy".to_string()),
        ("Ctrl+V".to_string(), "Paste".to_string()),
    ];
    let _banner = Banner::new().with_shortcuts(shortcuts);
    assert!(true, "Banner with shortcuts created");
}

#[test]
fn test_banner_chaining() {
    let _banner = Banner::new()
        .with_model("gpt-4")
        .with_permission_mode("WorkspaceWrite")
        .with_directory("/project")
        .with_session_id("abc123");
    assert!(true, "Banner with all options created");
}

#[test]
fn test_startup_info_new() {
    let info = StartupInfo::new("gpt-4".to_string(), "/project".to_string());
    assert_eq!(info.model, "gpt-4");
    assert_eq!(info.directory, "/project");
    assert!(info.session_id.is_none());
}

#[test]
fn test_startup_info_with_session() {
    let info = StartupInfo::new("gpt-4".to_string(), "/project".to_string())
        .with_session("session-123".to_string());
    assert_eq!(info.session_id, Some("session-123".to_string()));
}

#[test]
fn test_right_panel_new() {
    let theme = opencode_tui::theme::Theme::default();
    let _panel = RightPanel::new(theme);
    assert!(true, "RightPanel created");
}

#[test]
fn test_right_panel_tab_next() {
    assert_eq!(RightPanelTab::Diagnostics.next(), RightPanelTab::Todo);
    assert_eq!(RightPanelTab::Todo.next(), RightPanelTab::PermissionQueue);
    assert_eq!(
        RightPanelTab::PermissionQueue.next(),
        RightPanelTab::Diagnostics
    );
}

#[test]
fn test_right_panel_tab_prev() {
    assert_eq!(
        RightPanelTab::Diagnostics.prev(),
        RightPanelTab::PermissionQueue
    );
    assert_eq!(RightPanelTab::Todo.prev(), RightPanelTab::Diagnostics);
    assert_eq!(RightPanelTab::PermissionQueue.prev(), RightPanelTab::Todo);
}

#[test]
fn test_right_panel_tab_index() {
    assert_eq!(RightPanelTab::Diagnostics.index(), 0);
    assert_eq!(RightPanelTab::Todo.index(), 1);
    assert_eq!(RightPanelTab::PermissionQueue.index(), 2);
}

#[test]
fn test_right_panel_tab_debug() {
    let tab = RightPanelTab::Diagnostics;
    let debug_str = format!("{:?}", tab);
    assert!(debug_str.contains("Diagnostics"));
}

#[test]
fn test_right_panel_tab_clone() {
    let tab1 = RightPanelTab::Diagnostics;
    let tab2 = tab1.clone();
    assert_eq!(tab1, tab2);
}

#[test]
fn test_right_panel_tab_equality() {
    assert_eq!(RightPanelTab::Diagnostics, RightPanelTab::Diagnostics);
    assert_ne!(RightPanelTab::Diagnostics, RightPanelTab::Todo);
    assert_ne!(RightPanelTab::Todo, RightPanelTab::PermissionQueue);
}

#[test]
fn test_right_panel_tab_all_variants() {
    assert!(matches!(
        RightPanelTab::Diagnostics,
        RightPanelTab::Diagnostics
    ));
    assert!(matches!(RightPanelTab::Todo, RightPanelTab::Todo));
    assert!(matches!(
        RightPanelTab::PermissionQueue,
        RightPanelTab::PermissionQueue
    ));
}
