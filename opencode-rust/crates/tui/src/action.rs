use crossterm::event::KeyCode;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Quit,

    ModeTransition(AppModeAction),

    Dialog(DialogAction),

    Timeline(TimelineAction),

    Input(InputAction),

    Connect(ConnectAction),

    Fork(ForkAction),

    Sessions(SessionsAction),

    SlashCommand(SlashCommandAction),

    FileDrop(Vec<std::path::PathBuf>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppModeAction {
    ShowHome,
    ShowChat,
    ShowTimeline,
    ShowSettings,
    ShowSessions,
    ShowProviderManagement,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DialogAction {
    Open(String),
    Close,
    Confirm,
    ConfirmWithData(String),
    Cancel,
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    PageUp,
    PageDown,
    Select(usize),
    Filter(String),
    InputChar(char),
    Backspace,
    Clear,
    Home,
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimelineAction {
    SelectPrevious,
    SelectNext,
    ToggleMetadata,
    ForkAtSelected,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForkAction {
    Cancel,
    Confirm,
    InputChar(char),
    Backspace,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionsAction {
    Close,
    SelectPrevious,
    SelectNext,
    Confirm,
    CreateNew,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SlashCommandAction {
    Cancel,
    Confirm,
    InputChar(char),
    Backspace,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputAction {
    Submit,
    MoveCursorLeft,
    MoveCursorRight,
    MoveToStart,
    MoveToEnd,
    DeleteChar,
    DeleteWord,
    HistoryPrevious,
    HistoryNext,
    Complete,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectAction {
    SelectProvider(String),
    SelectMethod(String),
    ConfirmApiKey(String),
    Cancel,
    StartOAuth(OAuthProvider),
    ValidationComplete(ValidationResult),
}

#[derive(Debug, Clone, PartialEq)]
pub enum OAuthProvider {
    OpenAI,
    Google,
    Copilot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    pub success: bool,
    pub error_message: Option<String>,
    pub models: Option<Vec<BrowserAuthModelInfo>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowserAuthModelInfo {
    pub id: String,
    pub name: String,
    pub variants: Vec<String>,
}

impl Action {
    pub fn from_key_event(key: &crossterm::event::KeyEvent) -> Option<Self> {
        use crossterm::event::KeyCode::*;

        if key.kind != crossterm::event::KeyEventKind::Press {
            return None;
        }

        match key.code {
            Char('c')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                Some(Action::Quit)
            }

            Char('j') | KeyCode::Down => Some(Action::Dialog(DialogAction::NavigateDown)),
            Char('k') | KeyCode::Up => Some(Action::Dialog(DialogAction::NavigateUp)),
            Char('h') | KeyCode::Left => Some(Action::Dialog(DialogAction::NavigateLeft)),
            Char('l') | KeyCode::Right => Some(Action::Dialog(DialogAction::NavigateRight)),

            KeyCode::Esc => Some(Action::Dialog(DialogAction::Cancel)),
            KeyCode::Enter => Some(Action::Dialog(DialogAction::Confirm)),

            KeyCode::Home => Some(Action::Dialog(DialogAction::Home)),
            KeyCode::End => Some(Action::Dialog(DialogAction::End)),

            KeyCode::PageUp => Some(Action::Dialog(DialogAction::PageUp)),
            KeyCode::PageDown => Some(Action::Dialog(DialogAction::PageDown)),

            KeyCode::Backspace => Some(Action::Dialog(DialogAction::Backspace)),

            Char(c) => Some(Action::Dialog(DialogAction::InputChar(c))),

            _ => None,
        }
    }
}

pub struct ActionHandler;

impl ActionHandler {
    pub fn handle(action: Action, state: &mut AppState) -> ActionResult {
        crate::app_update::update(state, action)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionResult {
    Handled,
    NotHandled,
    Quit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Home,
    Chat,
    Timeline,
    ForkDialog,
    CommandPalette,
    SlashCommand,
    DiffReview,
    Sessions,
    Settings,
    ModelSelection,
    ProviderManagement,
    ConnectProvider,
    ConnectMethod,
    ConnectApiKey,
    ConnectProgress,
    ConnectApiKeyError,
    ConnectModel,
    FileSelection,
    DirectorySelection,
    ReleaseNotes,
    Search,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub mode: AppMode,
    pub should_quit: bool,
    pub show_metadata: bool,
    pub pending_file_drop: Option<Vec<std::path::PathBuf>>,
    pub pending_connect_provider: Option<String>,
    pub pending_connect_method: Option<String>,
    pub pending_api_key_for_validation: Option<String>,
    pub validation_in_progress: bool,
    pub timeline_selected_index: Option<usize>,
    pub dialog_selected_index: Option<usize>,
    pub dialog_filter: Option<String>,
    pub input_buffer: String,
    pub input_cursor_position: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mode: AppMode::Home,
            should_quit: false,
            show_metadata: false,
            pending_file_drop: None,
            pending_connect_provider: None,
            pending_connect_method: None,
            pending_api_key_for_validation: None,
            validation_in_progress: false,
            timeline_selected_index: None,
            dialog_selected_index: None,
            dialog_filter: None,
            input_buffer: String::new(),
            input_cursor_position: 0,
        }
    }
}

impl Default for AppMode {
    fn default() -> Self {
        AppMode::Home
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;
    use crossterm::event::KeyModifiers;

    #[test]
    fn test_action_from_key_event_escape() {
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::empty(),
        );
        let action = Action::from_key_event(&key);
        assert_eq!(action, Some(Action::Dialog(DialogAction::Cancel)));
    }

    #[test]
    fn test_action_from_key_event_enter() {
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        );
        let action = Action::from_key_event(&key);
        assert_eq!(action, Some(Action::Dialog(DialogAction::Confirm)));
    }

    #[test]
    fn test_action_from_key_event_navigation() {
        let up_key = crossterm::event::KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        assert_eq!(
            Action::from_key_event(&up_key),
            Some(Action::Dialog(DialogAction::NavigateUp))
        );

        let down_key = crossterm::event::KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
        assert_eq!(
            Action::from_key_event(&down_key),
            Some(Action::Dialog(DialogAction::NavigateDown))
        );
    }

    #[test]
    fn test_action_from_key_event_ctrl_c() {
        let key = crossterm::event::KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(Action::from_key_event(&key), Some(Action::Quit));
    }

    #[test]
    fn test_action_from_key_event_char_input() {
        let key = crossterm::event::KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());
        assert_eq!(
            Action::from_key_event(&key),
            Some(Action::Dialog(DialogAction::InputChar('a')))
        );
    }

    #[test]
    fn test_action_from_key_event_press_kind_required() {
        let key =
            crossterm::event::KeyEvent::new(KeyCode::Esc, crossterm::event::KeyModifiers::empty());

        assert_eq!(
            Action::from_key_event(&key),
            Some(Action::Dialog(DialogAction::Cancel))
        );
    }

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert_eq!(state.mode, AppMode::Home);
        assert!(!state.should_quit);
        assert!(!state.show_metadata);
        assert!(state.pending_file_drop.is_none());
    }

    #[test]
    fn test_app_state_mode_transitions() {
        let mut state = AppState::new();
        ActionHandler::handle(Action::ModeTransition(AppModeAction::ShowChat), &mut state);
        assert_eq!(state.mode, AppMode::Chat);
    }

    #[test]
    fn test_app_state_quit() {
        let mut state = AppState::new();
        assert!(!state.should_quit);
        ActionHandler::handle(Action::Quit, &mut state);
        assert!(state.should_quit);
    }

    #[test]
    fn test_action_handler_mode_transitions() {
        let mut state = AppState::new();

        ActionHandler::handle(Action::ModeTransition(AppModeAction::ShowHome), &mut state);
        assert_eq!(state.mode, AppMode::Home);

        ActionHandler::handle(Action::ModeTransition(AppModeAction::ShowChat), &mut state);
        assert_eq!(state.mode, AppMode::Chat);

        ActionHandler::handle(
            Action::ModeTransition(AppModeAction::ShowTimeline),
            &mut state,
        );
        assert_eq!(state.mode, AppMode::Timeline);

        ActionHandler::handle(
            Action::ModeTransition(AppModeAction::ShowSettings),
            &mut state,
        );
        assert_eq!(state.mode, AppMode::Settings);

        ActionHandler::handle(
            Action::ModeTransition(AppModeAction::ShowSessions),
            &mut state,
        );
        assert_eq!(state.mode, AppMode::Sessions);

        ActionHandler::handle(
            Action::ModeTransition(AppModeAction::ShowProviderManagement),
            &mut state,
        );
        assert_eq!(state.mode, AppMode::ProviderManagement);
    }

    #[test]
    fn test_action_handler_dialog_close_transitions_mode() {
        let mut state = AppState::new();

        state.mode = AppMode::ConnectMethod;
        ActionHandler::handle(Action::Dialog(DialogAction::Close), &mut state);
        assert_eq!(state.mode, AppMode::ConnectProvider);

        state.mode = AppMode::ConnectApiKey;
        ActionHandler::handle(Action::Dialog(DialogAction::Close), &mut state);
        assert_eq!(state.mode, AppMode::ConnectMethod);

        state.mode = AppMode::ConnectProgress;
        ActionHandler::handle(Action::Dialog(DialogAction::Close), &mut state);
        assert_eq!(state.mode, AppMode::ConnectApiKey);

        state.mode = AppMode::ConnectModel;
        ActionHandler::handle(Action::Dialog(DialogAction::Close), &mut state);
        assert_eq!(state.mode, AppMode::ConnectApiKey);

        state.mode = AppMode::ConnectApiKeyError;
        ActionHandler::handle(Action::Dialog(DialogAction::Close), &mut state);
        assert_eq!(state.mode, AppMode::ConnectApiKey);

        state.mode = AppMode::ForkDialog;
        ActionHandler::handle(Action::Dialog(DialogAction::Close), &mut state);
        assert_eq!(state.mode, AppMode::Timeline);

        state.mode = AppMode::CommandPalette;
        ActionHandler::handle(Action::Dialog(DialogAction::Close), &mut state);
        assert_eq!(state.mode, AppMode::Chat);

        state.mode = AppMode::SlashCommand;
        ActionHandler::handle(Action::Dialog(DialogAction::Close), &mut state);
        assert_eq!(state.mode, AppMode::Chat);

        state.mode = AppMode::Search;
        ActionHandler::handle(Action::Dialog(DialogAction::Close), &mut state);
        assert_eq!(state.mode, AppMode::Chat);

        state.mode = AppMode::FileSelection;
        ActionHandler::handle(Action::Dialog(DialogAction::Close), &mut state);
        assert_eq!(state.mode, AppMode::Chat);

        state.mode = AppMode::DirectorySelection;
        ActionHandler::handle(Action::Dialog(DialogAction::Close), &mut state);
        assert_eq!(state.mode, AppMode::Chat);

        state.mode = AppMode::DiffReview;
        ActionHandler::handle(Action::Dialog(DialogAction::Close), &mut state);
        assert_eq!(state.mode, AppMode::Chat);

        state.mode = AppMode::ReleaseNotes;
        ActionHandler::handle(Action::Dialog(DialogAction::Close), &mut state);
        assert_eq!(state.mode, AppMode::Home);
    }

    #[test]
    fn test_action_handler_dialog_navigate_up() {
        let mut state = AppState::new();
        state.dialog_selected_index = Some(5);
        ActionHandler::handle(Action::Dialog(DialogAction::NavigateUp), &mut state);
        assert_eq!(state.dialog_selected_index, Some(4));
    }

    #[test]
    fn test_action_handler_dialog_navigate_up_at_zero() {
        let mut state = AppState::new();
        state.dialog_selected_index = Some(0);
        ActionHandler::handle(Action::Dialog(DialogAction::NavigateUp), &mut state);
        assert_eq!(state.dialog_selected_index, Some(0));
    }

    #[test]
    fn test_action_handler_dialog_navigate_down() {
        let mut state = AppState::new();
        state.dialog_selected_index = Some(5);
        ActionHandler::handle(Action::Dialog(DialogAction::NavigateDown), &mut state);
        assert_eq!(state.dialog_selected_index, Some(6));
    }

    #[test]
    fn test_action_handler_dialog_navigate_down_from_none() {
        let mut state = AppState::new();
        assert!(state.dialog_selected_index.is_none());
        ActionHandler::handle(Action::Dialog(DialogAction::NavigateDown), &mut state);
        assert_eq!(state.dialog_selected_index, Some(1));
    }

    #[test]
    fn test_action_handler_dialog_home() {
        let mut state = AppState::new();
        state.dialog_selected_index = Some(10);
        ActionHandler::handle(Action::Dialog(DialogAction::Home), &mut state);
        assert_eq!(state.dialog_selected_index, Some(0));
    }

    #[test]
    fn test_action_handler_dialog_select() {
        let mut state = AppState::new();
        ActionHandler::handle(Action::Dialog(DialogAction::Select(42)), &mut state);
        assert_eq!(state.dialog_selected_index, Some(42));
    }

    #[test]
    fn test_action_handler_dialog_input_char() {
        let mut state = AppState::new();
        ActionHandler::handle(Action::Dialog(DialogAction::InputChar('a')), &mut state);
        assert_eq!(state.input_buffer, "a");
        ActionHandler::handle(Action::Dialog(DialogAction::InputChar('b')), &mut state);
        assert_eq!(state.input_buffer, "ab");
        ActionHandler::handle(Action::Dialog(DialogAction::InputChar('c')), &mut state);
        assert_eq!(state.input_buffer, "abc");
    }

    #[test]
    fn test_action_handler_dialog_input_char_unicode() {
        let mut state = AppState::new();
        ActionHandler::handle(Action::Dialog(DialogAction::InputChar('好')), &mut state);
        assert_eq!(state.input_buffer, "好");
        ActionHandler::handle(Action::Dialog(DialogAction::InputChar('👋')), &mut state);
        assert_eq!(state.input_buffer, "好👋");
    }

    #[test]
    fn test_action_handler_dialog_backspace() {
        let mut state = AppState::new();
        state.input_buffer = "abc".to_string();
        ActionHandler::handle(Action::Dialog(DialogAction::Backspace), &mut state);
        assert_eq!(state.input_buffer, "ab");
        ActionHandler::handle(Action::Dialog(DialogAction::Backspace), &mut state);
        assert_eq!(state.input_buffer, "a");
    }

    #[test]
    fn test_action_handler_dialog_backspace_empty() {
        let mut state = AppState::new();
        assert!(state.input_buffer.is_empty());
        ActionHandler::handle(Action::Dialog(DialogAction::Backspace), &mut state);
        assert_eq!(state.input_buffer, "");
    }

    #[test]
    fn test_action_handler_dialog_clear() {
        let mut state = AppState::new();
        state.input_buffer = "test input".to_string();
        ActionHandler::handle(Action::Dialog(DialogAction::Clear), &mut state);
        assert!(state.input_buffer.is_empty());
    }

    #[test]
    fn test_action_handler_dialog_filter() {
        let mut state = AppState::new();
        ActionHandler::handle(
            Action::Dialog(DialogAction::Filter("test".to_string())),
            &mut state,
        );
        assert_eq!(state.dialog_filter, Some("test".to_string()));
    }

    #[test]
    fn test_action_handler_timeline_toggle_metadata() {
        let mut state = AppState::new();
        assert!(!state.show_metadata);
        ActionHandler::handle(Action::Timeline(TimelineAction::ToggleMetadata), &mut state);
        assert!(state.show_metadata);
        ActionHandler::handle(Action::Timeline(TimelineAction::ToggleMetadata), &mut state);
        assert!(!state.show_metadata);
    }

    #[test]
    fn test_action_handler_connect_select_provider() {
        let mut state = AppState::new();
        ActionHandler::handle(
            Action::Connect(ConnectAction::SelectProvider("openai".to_string())),
            &mut state,
        );
        assert_eq!(state.pending_connect_provider, Some("openai".to_string()));
    }

    #[test]
    fn test_action_handler_connect_select_method() {
        let mut state = AppState::new();
        ActionHandler::handle(
            Action::Connect(ConnectAction::SelectMethod("api_key".to_string())),
            &mut state,
        );
        assert_eq!(state.pending_connect_method, Some("api_key".to_string()));
    }

    #[test]
    fn test_action_handler_connect_confirm_api_key() {
        let mut state = AppState::new();
        ActionHandler::handle(
            Action::Connect(ConnectAction::ConfirmApiKey("sk-test123".to_string())),
            &mut state,
        );
        assert_eq!(
            state.pending_api_key_for_validation,
            Some("sk-test123".to_string())
        );
    }

    #[test]
    fn test_action_handler_connect_cancel_clears_pending() {
        let mut state = AppState::new();
        state.pending_connect_provider = Some("openai".to_string());
        state.pending_connect_method = Some("api_key".to_string());
        state.pending_api_key_for_validation = Some("sk-test".to_string());

        ActionHandler::handle(Action::Connect(ConnectAction::Cancel), &mut state);

        assert!(state.pending_connect_provider.is_none());
        assert!(state.pending_connect_method.is_none());
        assert!(state.pending_api_key_for_validation.is_none());
    }

    #[test]
    fn test_action_handler_connect_validation_complete_success() {
        let mut state = AppState::new();
        state.validation_in_progress = true;

        let result = ValidationResult {
            success: true,
            error_message: None,
            models: Some(vec![BrowserAuthModelInfo {
                id: "gpt-4".to_string(),
                name: "GPT-4".to_string(),
                variants: vec![],
            }]),
        };

        ActionHandler::handle(
            Action::Connect(ConnectAction::ValidationComplete(result)),
            &mut state,
        );

        assert!(!state.validation_in_progress);
    }

    #[test]
    fn test_action_handler_connect_validation_complete_failure() {
        let mut state = AppState::new();
        state.validation_in_progress = true;

        let result = ValidationResult {
            success: false,
            error_message: Some("Invalid API key".to_string()),
            models: None,
        };

        ActionHandler::handle(
            Action::Connect(ConnectAction::ValidationComplete(result)),
            &mut state,
        );

        assert!(!state.validation_in_progress);
    }

    #[test]
    fn test_action_handler_file_drop() {
        let mut state = AppState::new();
        let paths = vec![std::path::PathBuf::from("/tmp/test.txt")];
        ActionHandler::handle(Action::FileDrop(paths.clone()), &mut state);
        assert_eq!(state.pending_file_drop, Some(paths));
    }

    #[test]
    fn test_action_handler_returns_correct_results() {
        let mut state = AppState::new();

        assert_eq!(
            ActionHandler::handle(Action::Quit, &mut state),
            ActionResult::Handled
        );
        assert_eq!(
            ActionHandler::handle(Action::ModeTransition(AppModeAction::ShowHome), &mut state),
            ActionResult::Handled
        );
        assert_eq!(
            ActionHandler::handle(Action::Dialog(DialogAction::Close), &mut state),
            ActionResult::Handled
        );
        assert_eq!(
            ActionHandler::handle(Action::Timeline(TimelineAction::ToggleMetadata), &mut state),
            ActionResult::Handled
        );
    }

    #[test]
    fn test_action_result_equality() {
        assert_eq!(ActionResult::Handled, ActionResult::Handled);
        assert_eq!(ActionResult::NotHandled, ActionResult::NotHandled);
        assert_eq!(ActionResult::Quit, ActionResult::Quit);
        assert_ne!(ActionResult::Handled, ActionResult::NotHandled);
        assert_ne!(ActionResult::Handled, ActionResult::Quit);
    }

    #[test]
    fn test_app_mode_equality() {
        assert_eq!(AppMode::Home, AppMode::Home);
        assert_eq!(AppMode::Chat, AppMode::Chat);
        assert_eq!(AppMode::Timeline, AppMode::Timeline);
        assert_ne!(AppMode::Home, AppMode::Chat);
    }

    #[test]
    fn test_app_mode_clone() {
        let mode = AppMode::ConnectProvider;
        let cloned = mode.clone();
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_dialog_action_variants() {
        use DialogAction::*;
        assert_eq!(Close, Close);
        assert_eq!(Confirm, Confirm);
        assert_eq!(Cancel, Cancel);
        assert_eq!(NavigateUp, NavigateUp);
        assert_eq!(NavigateDown, NavigateDown);
        assert_eq!(NavigateLeft, NavigateLeft);
        assert_eq!(NavigateRight, NavigateRight);
        assert_eq!(PageUp, PageUp);
        assert_eq!(PageDown, PageDown);
        assert_eq!(Select(0), Select(0));
        assert_eq!(Select(1), Select(1));
        assert_ne!(Select(0), Select(1));
    }

    #[test]
    fn test_timeline_action_variants() {
        use TimelineAction::*;
        assert_eq!(SelectPrevious, SelectPrevious);
        assert_eq!(SelectNext, SelectNext);
        assert_eq!(ToggleMetadata, ToggleMetadata);
        assert_eq!(ForkAtSelected, ForkAtSelected);
    }

    #[test]
    fn test_input_action_variants() {
        use InputAction::*;
        assert_eq!(Submit, Submit);
        assert_eq!(MoveCursorLeft, MoveCursorLeft);
        assert_eq!(MoveCursorRight, MoveCursorRight);
        assert_eq!(MoveToStart, MoveToStart);
        assert_eq!(MoveToEnd, MoveToEnd);
        assert_eq!(DeleteChar, DeleteChar);
        assert_eq!(DeleteWord, DeleteWord);
        assert_eq!(HistoryPrevious, HistoryPrevious);
        assert_eq!(HistoryNext, HistoryNext);
        assert_eq!(Complete, Complete);
    }

    #[test]
    fn test_connect_action_variants() {
        use ConnectAction::*;
        assert_eq!(
            SelectProvider("test".to_string()),
            SelectProvider("test".to_string())
        );
        assert_eq!(
            SelectMethod("api_key".to_string()),
            SelectMethod("api_key".to_string())
        );
        assert_ne!(
            SelectProvider("test".to_string()),
            SelectProvider("other".to_string())
        );
    }

    #[test]
    fn test_oauth_provider_variants() {
        assert_eq!(OAuthProvider::OpenAI, OAuthProvider::OpenAI);
        assert_eq!(OAuthProvider::Google, OAuthProvider::Google);
        assert_eq!(OAuthProvider::Copilot, OAuthProvider::Copilot);
        assert_ne!(OAuthProvider::OpenAI, OAuthProvider::Google);
    }

    #[test]
    fn test_validation_result_creation() {
        let result = ValidationResult {
            success: true,
            error_message: None,
            models: None,
        };
        assert!(result.success);
        assert!(result.error_message.is_none());
        assert!(result.models.is_none());
    }

    #[test]
    fn test_browser_auth_model_info_creation() {
        let model = BrowserAuthModelInfo {
            id: "gpt-4".to_string(),
            name: "GPT-4".to_string(),
            variants: vec!["gpt-4-turbo".to_string()],
        };
        assert_eq!(model.id, "gpt-4");
        assert_eq!(model.name, "GPT-4");
        assert_eq!(model.variants.len(), 1);
    }

    #[test]
    fn test_dialog_action_confirm_with_data() {
        use DialogAction::*;
        assert_eq!(
            ConfirmWithData("retry".to_string()),
            ConfirmWithData("retry".to_string())
        );
        assert_eq!(
            ConfirmWithData("cancel".to_string()),
            ConfirmWithData("cancel".to_string())
        );
        assert_ne!(
            ConfirmWithData("retry".to_string()),
            ConfirmWithData("cancel".to_string())
        );
        assert_ne!(ConfirmWithData("retry".to_string()), Confirm);
        assert_ne!(ConfirmWithData("retry".to_string()), Close);
    }

    #[test]
    fn test_action_handler_confirm_with_data() {
        let mut state = AppState::new();
        let result = ActionHandler::handle(
            Action::Dialog(DialogAction::ConfirmWithData("retry".to_string())),
            &mut state,
        );
        assert_eq!(result, ActionResult::Handled);
    }

    #[test]
    fn test_dialog_action_confirm_with_data_clone() {
        let action = DialogAction::ConfirmWithData("test".to_string());
        let cloned = action.clone();
        assert_eq!(action, cloned);
    }

    #[test]
    fn test_fork_action_cancel() {
        let mut state = AppState::new();
        state.mode = AppMode::ForkDialog;

        ActionHandler::handle(Action::Fork(ForkAction::Cancel), &mut state);

        assert_eq!(state.mode, AppMode::Timeline);
    }

    #[test]
    fn test_fork_action_confirm() {
        let mut state = AppState::new();
        state.mode = AppMode::ForkDialog;

        let result = ActionHandler::handle(Action::Fork(ForkAction::Confirm), &mut state);

        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.mode, AppMode::ForkDialog);
    }

    #[test]
    fn test_fork_action_input_char() {
        let mut state = AppState::new();
        assert!(state.input_buffer.is_empty());

        ActionHandler::handle(Action::Fork(ForkAction::InputChar('a')), &mut state);
        assert_eq!(state.input_buffer, "a");

        ActionHandler::handle(Action::Fork(ForkAction::InputChar('b')), &mut state);
        assert_eq!(state.input_buffer, "ab");
    }

    #[test]
    fn test_fork_action_backspace() {
        let mut state = AppState::new();
        state.input_buffer = "test".to_string();

        ActionHandler::handle(Action::Fork(ForkAction::Backspace), &mut state);
        assert_eq!(state.input_buffer, "tes");

        ActionHandler::handle(Action::Fork(ForkAction::Backspace), &mut state);
        assert_eq!(state.input_buffer, "te");
    }

    #[test]
    fn test_fork_action_backspace_empty() {
        let mut state = AppState::new();
        assert!(state.input_buffer.is_empty());

        ActionHandler::handle(Action::Fork(ForkAction::Backspace), &mut state);
        assert!(state.input_buffer.is_empty());
    }

    #[test]
    fn test_action_from_key_event_enter_maps_to_confirm() {
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        );
        let action = Action::from_key_event(&key);
        assert_eq!(action, Some(Action::Dialog(DialogAction::Confirm)));
    }

    #[test]
    fn test_sessions_action_close() {
        let mut state = AppState::new();
        state.mode = AppMode::Sessions;

        ActionHandler::handle(Action::Sessions(SessionsAction::Close), &mut state);

        assert_eq!(state.mode, AppMode::Chat);
    }

    #[test]
    fn test_sessions_action_select_previous() {
        let mut state = AppState::new();
        let result =
            ActionHandler::handle(Action::Sessions(SessionsAction::SelectPrevious), &mut state);
        assert_eq!(result, ActionResult::Handled);
    }

    #[test]
    fn test_sessions_action_select_next() {
        let mut state = AppState::new();
        let result =
            ActionHandler::handle(Action::Sessions(SessionsAction::SelectNext), &mut state);
        assert_eq!(result, ActionResult::Handled);
    }

    #[test]
    fn test_sessions_action_confirm() {
        let mut state = AppState::new();
        state.mode = AppMode::Sessions;

        let result = ActionHandler::handle(Action::Sessions(SessionsAction::Confirm), &mut state);
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.mode, AppMode::Sessions);
    }

    #[test]
    fn test_sessions_action_create_new() {
        let mut state = AppState::new();
        let result = ActionHandler::handle(Action::Sessions(SessionsAction::CreateNew), &mut state);
        assert_eq!(result, ActionResult::Handled);
    }

    #[test]
    fn test_sessions_action_variants() {
        use SessionsAction::*;
        assert_eq!(Close, Close);
        assert_eq!(SelectPrevious, SelectPrevious);
        assert_eq!(SelectNext, SelectNext);
        assert_eq!(Confirm, Confirm);
        assert_eq!(CreateNew, CreateNew);
    }

    #[test]
    fn test_slash_command_action_cancel() {
        let mut state = AppState::new();
        state.mode = AppMode::SlashCommand;

        ActionHandler::handle(Action::SlashCommand(SlashCommandAction::Cancel), &mut state);

        assert_eq!(state.mode, AppMode::Chat);
    }

    #[test]
    fn test_slash_command_action_confirm() {
        let mut state = AppState::new();
        state.mode = AppMode::SlashCommand;

        let result = ActionHandler::handle(
            Action::SlashCommand(SlashCommandAction::Confirm),
            &mut state,
        );
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.mode, AppMode::SlashCommand);
    }

    #[test]
    fn test_slash_command_action_input_char() {
        let mut state = AppState::new();
        assert!(state.input_buffer.is_empty());

        ActionHandler::handle(
            Action::SlashCommand(SlashCommandAction::InputChar('/')),
            &mut state,
        );
        assert_eq!(state.input_buffer, "/");

        ActionHandler::handle(
            Action::SlashCommand(SlashCommandAction::InputChar('p')),
            &mut state,
        );
        assert_eq!(state.input_buffer, "/p");
    }

    #[test]
    fn test_slash_command_action_backspace() {
        let mut state = AppState::new();
        state.input_buffer = "test".to_string();

        ActionHandler::handle(
            Action::SlashCommand(SlashCommandAction::Backspace),
            &mut state,
        );
        assert_eq!(state.input_buffer, "tes");

        ActionHandler::handle(
            Action::SlashCommand(SlashCommandAction::Backspace),
            &mut state,
        );
        assert_eq!(state.input_buffer, "te");
    }

    #[test]
    fn test_slash_command_action_backspace_empty() {
        let mut state = AppState::new();
        assert!(state.input_buffer.is_empty());

        ActionHandler::handle(
            Action::SlashCommand(SlashCommandAction::Backspace),
            &mut state,
        );
        assert!(state.input_buffer.is_empty());
    }

    #[test]
    fn test_slash_command_action_variants() {
        use SlashCommandAction::*;
        assert_eq!(Cancel, Cancel);
        assert_eq!(Confirm, Confirm);
        assert_eq!(InputChar('a'), InputChar('a'));
        assert_eq!(Backspace, Backspace);
    }

    #[test]
    fn test_input_buffer_unicode_handling() {
        let mut state = AppState::new();
        assert!(state.input_buffer.is_empty());

        ActionHandler::handle(Action::Dialog(DialogAction::InputChar('a')), &mut state);
        assert_eq!(state.input_buffer, "a");

        ActionHandler::handle(Action::Dialog(DialogAction::InputChar('b')), &mut state);
        assert_eq!(state.input_buffer, "ab");
    }

    #[test]
    fn test_input_buffer_clear() {
        let mut state = AppState::new();
        state.input_buffer = "test content".to_string();

        ActionHandler::handle(Action::Dialog(DialogAction::Clear), &mut state);
        assert!(state.input_buffer.is_empty());
    }

    #[test]
    fn test_close_current_dialog_all_dialog_modes() {
        let mut state = AppState::new();

        let dialog_modes = vec![
            (AppMode::ConnectProvider, AppMode::Chat),
            (AppMode::ConnectMethod, AppMode::ConnectProvider),
            (AppMode::ConnectApiKey, AppMode::ConnectMethod),
            (AppMode::ConnectProgress, AppMode::ConnectApiKey),
            (AppMode::ConnectModel, AppMode::ConnectApiKey),
            (AppMode::ConnectApiKeyError, AppMode::ConnectApiKey),
            (AppMode::ModelSelection, AppMode::Chat),
            (AppMode::ForkDialog, AppMode::Timeline),
            (AppMode::CommandPalette, AppMode::Chat),
            (AppMode::SlashCommand, AppMode::Chat),
            (AppMode::Search, AppMode::Chat),
            (AppMode::FileSelection, AppMode::Chat),
            (AppMode::DirectorySelection, AppMode::Chat),
            (AppMode::DiffReview, AppMode::Chat),
            (AppMode::ReleaseNotes, AppMode::Home),
        ];

        for (initial_mode, expected_mode) in dialog_modes {
            state.mode = initial_mode.clone();
            ActionHandler::handle(Action::Dialog(DialogAction::Close), &mut state);
            assert_eq!(
                state.mode, expected_mode,
                "Close from {:?} should go to {:?}, got {:?}",
                initial_mode, expected_mode, state.mode
            );
        }
    }

    #[test]
    fn test_app_state_dialog_navigate_down_from_none() {
        let mut state = AppState::new();
        assert!(state.dialog_selected_index.is_none());

        ActionHandler::handle(Action::Dialog(DialogAction::NavigateDown), &mut state);
        assert_eq!(state.dialog_selected_index, Some(1));
    }

    #[test]
    fn test_app_state_dialog_navigate_down_wraps() {
        let mut state = AppState::new();
        state.dialog_selected_index = Some(5);

        ActionHandler::handle(Action::Dialog(DialogAction::NavigateDown), &mut state);
        assert_eq!(state.dialog_selected_index, Some(6));
    }

    #[test]
    fn test_app_state_dialog_select() {
        let mut state = AppState::new();
        assert!(state.dialog_selected_index.is_none());

        ActionHandler::handle(Action::Dialog(DialogAction::Select(42)), &mut state);
        assert_eq!(state.dialog_selected_index, Some(42));
    }

    #[test]
    fn test_app_state_dialog_filter() {
        let mut state = AppState::new();
        assert!(state.dialog_filter.is_none());

        ActionHandler::handle(
            Action::Dialog(DialogAction::Filter("test filter".to_string())),
            &mut state,
        );
        assert_eq!(state.dialog_filter, Some("test filter".to_string()));
    }

    #[test]
    fn test_action_result_variants() {
        assert_eq!(ActionResult::Handled, ActionResult::Handled);
        assert_eq!(ActionResult::NotHandled, ActionResult::NotHandled);
        assert_eq!(ActionResult::Quit, ActionResult::Quit);
        assert_ne!(ActionResult::Handled, ActionResult::NotHandled);
        assert_ne!(ActionResult::Handled, ActionResult::Quit);
        assert_ne!(ActionResult::NotHandled, ActionResult::Quit);
    }

    #[test]
    fn test_app_state_new_has_default_values() {
        let state = AppState::new();
        assert_eq!(state.mode, AppMode::Home);
        assert!(!state.should_quit);
        assert!(!state.show_metadata);
        assert!(state.pending_file_drop.is_none());
        assert!(state.pending_connect_provider.is_none());
        assert!(state.pending_connect_method.is_none());
        assert!(state.pending_api_key_for_validation.is_none());
        assert!(!state.validation_in_progress);
        assert!(state.timeline_selected_index.is_none());
        assert!(state.dialog_selected_index.is_none());
        assert!(state.dialog_filter.is_none());
        assert_eq!(state.input_buffer, "");
        assert_eq!(state.input_cursor_position, 0);
    }
}
