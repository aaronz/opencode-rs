use opencode_tui::dialogs::home_view::{HomeAction, HomeView};
use opencode_tui::session::SessionManager;
use opencode_tui::theme::Theme;

#[test]
fn test_home_view_new() {
    let theme = Theme::default();
    let view = HomeView::new(theme);
    assert_eq!(view.get_selected_action(), HomeAction::NewSession);
}

#[test]
fn test_home_action_all() {
    let actions = HomeAction::all();
    assert_eq!(actions.len(), 5);
    assert!(actions.contains(&HomeAction::NewSession));
    assert!(actions.contains(&HomeAction::ContinueLast));
    assert!(actions.contains(&HomeAction::ViewSessions));
    assert!(actions.contains(&HomeAction::Settings));
    assert!(actions.contains(&HomeAction::Quit));
}

#[test]
fn test_home_action_label() {
    assert_eq!(HomeAction::NewSession.label(), "New Session");
    assert_eq!(HomeAction::ContinueLast.label(), "Continue Last Session");
    assert_eq!(HomeAction::ViewSessions.label(), "View All Sessions");
    assert_eq!(HomeAction::Settings.label(), "Settings");
    assert_eq!(HomeAction::Quit.label(), "Quit");
}

#[test]
fn test_home_action_key() {
    assert_eq!(HomeAction::NewSession.key(), "n");
    assert_eq!(HomeAction::ContinueLast.key(), "c");
    assert_eq!(HomeAction::ViewSessions.key(), "s");
    assert_eq!(HomeAction::Settings.key(), ",");
    assert_eq!(HomeAction::Quit.key(), "q");
}

#[test]
fn test_home_view_with_model() {
    let theme = Theme::default();
    let view = HomeView::new(theme).with_model("gpt-4".to_string());
    assert_eq!(view.get_selected_action(), HomeAction::NewSession);
}

#[test]
fn test_home_view_with_directory() {
    let theme = Theme::default();
    let view = HomeView::new(theme).with_directory("/tmp".to_string());
    assert_eq!(view.get_selected_action(), HomeAction::NewSession);
}

#[test]
fn test_home_action_to_string() {
    assert_eq!(HomeAction::NewSession.to_string(), "new_session");
    assert_eq!(HomeAction::ContinueLast.to_string(), "continue_last");
    assert_eq!(HomeAction::ViewSessions.to_string(), "view_sessions");
    assert_eq!(HomeAction::Settings.to_string(), "settings");
    assert_eq!(HomeAction::Quit.to_string(), "quit");
}

#[test]
fn test_home_action_partial_eq() {
    assert_eq!(HomeAction::NewSession, HomeAction::NewSession);
    assert_ne!(HomeAction::NewSession, HomeAction::Quit);
}

#[test]
fn test_home_view_move_selection() {
    let theme = Theme::default();
    let mut view = HomeView::new(theme);

    assert_eq!(view.get_selected_action(), HomeAction::NewSession);

    view.move_selection(1);
    assert_eq!(view.get_selected_action(), HomeAction::ContinueLast);

    view.move_selection(1);
    assert_eq!(view.get_selected_action(), HomeAction::ViewSessions);

    view.move_selection(-1);
    assert_eq!(view.get_selected_action(), HomeAction::ContinueLast);
}

#[test]
fn test_home_view_selection_bounds() {
    let theme = Theme::default();
    let mut view = HomeView::new(theme);

    for _ in 0..10 {
        view.move_selection(1);
    }
    assert_eq!(view.get_selected_action(), HomeAction::Quit);

    for _ in 0..10 {
        view.move_selection(-1);
    }
    assert_eq!(view.get_selected_action(), HomeAction::NewSession);
}

#[test]
fn test_home_view_with_session_manager() {
    let theme = Theme::default();
    let mut session_manager = SessionManager::new();
    session_manager.add_session("Test Session 1");
    session_manager.add_session("Test Session 2");

    let view = HomeView::new(theme).with_session_manager(&session_manager);
    assert_eq!(view.get_selected_action(), HomeAction::NewSession);
}
