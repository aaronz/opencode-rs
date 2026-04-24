use opencode_tui::dialogs::{Dialog, DialogAction, HomeAction, HomeView, HomeViewSection};

#[test]
fn test_dialog_action_variants() {
    assert!(matches!(DialogAction::None, DialogAction::None));
    assert!(matches!(DialogAction::Close, DialogAction::Close));
}

#[test]
fn test_dialog_action_confirm() {
    let action = DialogAction::Confirm("value".to_string());
    assert!(matches!(action, DialogAction::Confirm(ref s) if s == "value"));
}

#[test]
fn test_dialog_action_confirm_multiple() {
    let values = vec!["a".to_string(), "b".to_string()];
    let action = DialogAction::ConfirmMultiple(values.clone());
    assert!(matches!(action, DialogAction::ConfirmMultiple(ref v) if v == &values));
}

#[test]
fn test_dialog_action_navigate() {
    let action = DialogAction::Navigate("next".to_string());
    assert!(matches!(action, DialogAction::Navigate(ref s) if s == "next"));
}

#[test]
fn test_dialog_action_clone() {
    let action1 = DialogAction::Confirm("test".to_string());
    let action2 = action1.clone();
    assert_eq!(action1, action2);
}

#[test]
fn test_dialog_action_debug() {
    let action = DialogAction::None;
    let debug_str = format!("{:?}", action);
    assert!(debug_str.contains("None"));
}

#[test]
fn test_home_view_section_variants() {
    assert!(matches!(
        HomeViewSection::QuickActions,
        HomeViewSection::QuickActions
    ));
    assert!(matches!(
        HomeViewSection::RecentSessions,
        HomeViewSection::RecentSessions
    ));
}

#[test]
fn test_home_action_variants() {
    assert!(matches!(HomeAction::NewSession, HomeAction::NewSession));
    assert!(matches!(HomeAction::ContinueLast, HomeAction::ContinueLast));
    assert!(matches!(HomeAction::ViewSessions, HomeAction::ViewSessions));
    assert!(matches!(HomeAction::Settings, HomeAction::Settings));
    assert!(matches!(HomeAction::Quit, HomeAction::Quit));
}

#[test]
fn test_home_view_dialog_trait() {
    let home_view = HomeView::new(opencode_tui::theme::Theme::default());
    assert!(!home_view.is_modal());
}

#[test]
fn test_dialog_trait_signature() {
    fn assert_dialog<D: Dialog>() {}
    assert_dialog::<HomeView>();
}

#[test]
fn test_home_view_section_copy() {
    let section = HomeViewSection::QuickActions;
    let section2 = section;
    assert_eq!(section, section2);
}

#[test]
fn test_home_action_copy() {
    let action = HomeAction::NewSession;
    let action2 = action;
    assert_eq!(action, action2);
}
