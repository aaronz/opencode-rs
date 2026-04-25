use crossterm::event::KeyCode;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Quit,

    ModeTransition(AppModeAction),

    Dialog(DialogAction),

    Timeline(TimelineAction),

    Input(InputAction),

    Connect(ConnectAction),

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
        match action {
            Action::Quit => {
                state.should_quit = true;
                ActionResult::Handled
            }
            Action::ModeTransition(mode_action) => Self::handle_mode_action(mode_action, state),
            Action::Dialog(dialog_action) => Self::handle_dialog_action(dialog_action, state),
            Action::Timeline(timeline_action) => {
                Self::handle_timeline_action(timeline_action, state)
            }
            Action::Input(input_action) => Self::handle_input_action(input_action, state),
            Action::Connect(connect_action) => Self::handle_connect_action(connect_action, state),
            Action::FileDrop(paths) => {
                state.pending_file_drop = Some(paths);
                ActionResult::Handled
            }
        }
    }

    fn handle_mode_action(action: AppModeAction, state: &mut AppState) -> ActionResult {
        match action {
            AppModeAction::ShowHome => {
                state.mode = AppMode::Home;
                ActionResult::Handled
            }
            AppModeAction::ShowChat => {
                state.mode = AppMode::Chat;
                ActionResult::Handled
            }
            AppModeAction::ShowTimeline => {
                state.mode = AppMode::Timeline;
                ActionResult::Handled
            }
            AppModeAction::ShowSettings => {
                state.mode = AppMode::Settings;
                ActionResult::Handled
            }
            AppModeAction::ShowSessions => {
                state.mode = AppMode::Sessions;
                ActionResult::Handled
            }
            AppModeAction::ShowProviderManagement => {
                state.mode = AppMode::ProviderManagement;
                ActionResult::Handled
            }
        }
    }

    fn handle_dialog_action(action: DialogAction, state: &mut AppState) -> ActionResult {
        match action {
            DialogAction::Close => {
                state.close_current_dialog();
                ActionResult::Handled
            }
            DialogAction::NavigateUp => {
                state.dialog_navigate_up();
                ActionResult::Handled
            }
            DialogAction::NavigateDown => {
                state.dialog_navigate_down();
                ActionResult::Handled
            }
            DialogAction::Confirm => {
                state.dialog_confirm();
                ActionResult::Handled
            }
            DialogAction::Cancel => {
                state.dialog_cancel();
                ActionResult::Handled
            }
            DialogAction::Home => {
                state.dialog_go_home();
                ActionResult::Handled
            }
            DialogAction::End => {
                state.dialog_go_end();
                ActionResult::Handled
            }
            DialogAction::PageUp => {
                state.dialog_page_up();
                ActionResult::Handled
            }
            DialogAction::PageDown => {
                state.dialog_page_down();
                ActionResult::Handled
            }
            DialogAction::InputChar(c) => {
                state.dialog_input_char(c);
                ActionResult::Handled
            }
            DialogAction::Backspace => {
                state.dialog_backspace();
                ActionResult::Handled
            }
            DialogAction::Clear => {
                state.dialog_clear();
                ActionResult::Handled
            }
            DialogAction::Select(index) => {
                state.dialog_select(index);
                ActionResult::Handled
            }
            DialogAction::Filter(filter) => {
                state.dialog_filter(filter);
                ActionResult::Handled
            }
            _ => ActionResult::NotHandled,
        }
    }

    fn handle_timeline_action(action: TimelineAction, state: &mut AppState) -> ActionResult {
        match action {
            TimelineAction::SelectPrevious => {
                state.timeline_select_previous();
                ActionResult::Handled
            }
            TimelineAction::SelectNext => {
                state.timeline_select_next();
                ActionResult::Handled
            }
            TimelineAction::ToggleMetadata => {
                state.show_metadata = !state.show_metadata;
                ActionResult::Handled
            }
            TimelineAction::ForkAtSelected => {
                state.fork_at_selected();
                ActionResult::Handled
            }
        }
    }

    fn handle_input_action(action: InputAction, state: &mut AppState) -> ActionResult {
        match action {
            InputAction::Submit => {
                state.submit_input();
                ActionResult::Handled
            }
            InputAction::MoveCursorLeft => {
                state.input_move_cursor_left();
                ActionResult::Handled
            }
            InputAction::MoveCursorRight => {
                state.input_move_cursor_right();
                ActionResult::Handled
            }
            InputAction::MoveToStart => {
                state.input_move_to_start();
                ActionResult::Handled
            }
            InputAction::MoveToEnd => {
                state.input_move_to_end();
                ActionResult::Handled
            }
            InputAction::DeleteChar => {
                state.input_delete_char();
                ActionResult::Handled
            }
            InputAction::DeleteWord => {
                state.input_delete_word();
                ActionResult::Handled
            }
            InputAction::HistoryPrevious => {
                state.input_history_previous();
                ActionResult::Handled
            }
            InputAction::HistoryNext => {
                state.input_history_next();
                ActionResult::Handled
            }
            InputAction::Complete => {
                state.input_complete();
                ActionResult::Handled
            }
        }
    }

    fn handle_connect_action(action: ConnectAction, state: &mut AppState) -> ActionResult {
        match action {
            ConnectAction::SelectProvider(provider_id) => {
                state.pending_connect_provider = Some(provider_id);
                ActionResult::Handled
            }
            ConnectAction::SelectMethod(method) => {
                state.pending_connect_method = Some(method);
                ActionResult::Handled
            }
            ConnectAction::ConfirmApiKey(api_key) => {
                state.pending_api_key_for_validation = Some(api_key);
                ActionResult::Handled
            }
            ConnectAction::Cancel => {
                state.pending_connect_provider = None;
                state.pending_connect_method = None;
                state.pending_api_key_for_validation = None;
                ActionResult::Handled
            }
            ConnectAction::StartOAuth(provider) => {
                state.start_oauth(provider);
                ActionResult::Handled
            }
            ConnectAction::ValidationComplete(result) => {
                state.validation_in_progress = false;
                if result.success {
                    state.connect_validation_success(result.models);
                } else {
                    state.connect_validation_failure(result.error_message);
                }
                ActionResult::Handled
            }
        }
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
    SlashCommandDialog,
    SearchDialog,
    Sessions,
    Settings,
    ConnectProvider,
    ConnectMethod,
    ConnectApiKey,
    ConnectProgress,
    ConnectModel,
    ValidationError,
    ProviderManagement,
    FileSelection,
    DirectorySelection,
    DiffReview,
    ReleaseNotes,
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

    fn close_current_dialog(&mut self) {
        match self.mode {
            AppMode::ConnectMethod => {
                self.mode = AppMode::ConnectProvider;
            }
            AppMode::ConnectApiKey => {
                self.mode = AppMode::ConnectMethod;
            }
            AppMode::ConnectProgress => {
                self.mode = AppMode::ConnectApiKey;
            }
            AppMode::ConnectModel => {
                self.mode = AppMode::ConnectApiKey;
            }
            AppMode::ValidationError => {
                self.mode = AppMode::ConnectApiKey;
            }
            AppMode::ForkDialog => {
                self.mode = AppMode::Timeline;
            }
            AppMode::CommandPalette => {
                self.mode = AppMode::Chat;
            }
            AppMode::SlashCommandDialog => {
                self.mode = AppMode::Chat;
            }
            AppMode::SearchDialog => {
                self.mode = AppMode::Chat;
            }
            AppMode::FileSelection => {
                self.mode = AppMode::Chat;
            }
            AppMode::DirectorySelection => {
                self.mode = AppMode::Chat;
            }
            AppMode::DiffReview => {
                self.mode = AppMode::Chat;
            }
            AppMode::ReleaseNotes => {
                self.mode = AppMode::Home;
            }
            _ => {}
        }
    }

    fn dialog_navigate_up(&mut self) {
        if let Some(idx) = self.dialog_selected_index {
            if idx > 0 {
                self.dialog_selected_index = Some(idx - 1);
            }
        }
    }

    fn dialog_navigate_down(&mut self) {
        self.dialog_selected_index = Some(self.dialog_selected_index.unwrap_or(0) + 1);
    }

    fn dialog_confirm(&mut self) {}
    fn dialog_cancel(&mut self) {
        self.close_current_dialog();
    }
    fn dialog_go_home(&mut self) {
        self.dialog_selected_index = Some(0);
    }
    fn dialog_go_end(&mut self) {}
    fn dialog_page_up(&mut self) {}
    fn dialog_page_down(&mut self) {}
    fn dialog_input_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }
    fn dialog_backspace(&mut self) {
        self.input_buffer.pop();
    }
    fn dialog_clear(&mut self) {
        self.input_buffer.clear();
    }
    fn dialog_select(&mut self, index: usize) {
        self.dialog_selected_index = Some(index);
    }
    fn dialog_filter(&mut self, filter: String) {
        self.dialog_filter = Some(filter);
    }

    fn timeline_select_previous(&mut self) {}
    fn timeline_select_next(&mut self) {}
    fn fork_at_selected(&mut self) {}

    fn submit_input(&mut self) {}
    fn input_move_cursor_left(&mut self) {
        if self.input_cursor_position > 0 {
            self.input_cursor_position -= 1;
        }
    }
    fn input_move_cursor_right(&mut self) {
        if self.input_cursor_position < self.input_buffer.len() {
            self.input_cursor_position += 1;
        }
    }
    fn input_move_to_start(&mut self) {
        self.input_cursor_position = 0;
    }
    fn input_move_to_end(&mut self) {
        self.input_cursor_position = self.input_buffer.len();
    }
    fn input_delete_char(&mut self) {}
    fn input_delete_word(&mut self) {}
    fn input_history_previous(&mut self) {}
    fn input_history_next(&mut self) {}
    fn input_complete(&mut self) {}

    fn start_oauth(&mut self, _provider: OAuthProvider) {}
    fn connect_validation_success(&mut self, _models: Option<Vec<BrowserAuthModelInfo>>) {}
    fn connect_validation_failure(&mut self, _error: Option<String>) {}
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
}
