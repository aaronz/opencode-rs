use opencode_tui::dialogs::{
    Dialog, DialogAction, HomeAction, HomeView, HomeViewSection, ModelInfo, ProviderInfo,
    ProviderStatus,
};

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
fn test_home_action_all() {
    let actions = HomeAction::all();
    assert_eq!(actions.len(), 5);
}

#[test]
fn test_home_action_labels() {
    assert_eq!(HomeAction::NewSession.label(), "New Session");
    assert_eq!(HomeAction::ContinueLast.label(), "Continue Last Session");
    assert_eq!(HomeAction::ViewSessions.label(), "View All Sessions");
    assert_eq!(HomeAction::Settings.label(), "Settings");
    assert_eq!(HomeAction::Quit.label(), "Quit");
}

#[test]
fn test_home_action_keys() {
    assert_eq!(HomeAction::NewSession.key(), "n");
    assert_eq!(HomeAction::ContinueLast.key(), "c");
    assert_eq!(HomeAction::ViewSessions.key(), "s");
    assert_eq!(HomeAction::Settings.key(), ",");
    assert_eq!(HomeAction::Quit.key(), "q");
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
fn test_provider_status_variants() {
    assert!(matches!(
        ProviderStatus::Connected,
        ProviderStatus::Connected
    ));
    assert!(matches!(
        ProviderStatus::Disconnected,
        ProviderStatus::Disconnected
    ));
    assert!(matches!(ProviderStatus::Error, ProviderStatus::Error));
}

#[test]
fn test_provider_info_creation() {
    let info = ProviderInfo {
        name: "OpenAI".to_string(),
        status: ProviderStatus::Connected,
        id: "openai".to_string(),
        api_key_set: true,
    };
    assert_eq!(info.name, "OpenAI");
    assert!(matches!(info.status, ProviderStatus::Connected));
    assert!(info.api_key_set);
}

#[test]
fn test_provider_info_not_api_key_set() {
    let info = ProviderInfo {
        name: "Test Provider".to_string(),
        status: ProviderStatus::Disconnected,
        id: "test".to_string(),
        api_key_set: false,
    };
    assert!(!info.api_key_set);
    assert!(matches!(info.status, ProviderStatus::Disconnected));
}

#[test]
fn test_model_info_creation() {
    let info = ModelInfo {
        id: "gpt-4".to_string(),
        name: "GPT-4".to_string(),
        provider: "OpenAI".to_string(),
        is_paid: true,
        is_available: true,
    };
    assert_eq!(info.id, "gpt-4");
    assert_eq!(info.name, "GPT-4");
    assert_eq!(info.provider, "OpenAI");
    assert!(info.is_paid);
    assert!(info.is_available);
}

#[test]
fn test_model_info_not_available() {
    let info = ModelInfo {
        id: "gpt-5".to_string(),
        name: "GPT-5".to_string(),
        provider: "OpenAI".to_string(),
        is_paid: true,
        is_available: false,
    };
    assert!(!info.is_available);
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
