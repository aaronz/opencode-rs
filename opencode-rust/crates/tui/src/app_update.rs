use crate::action::{
    Action, ActionResult, AppMode, AppState, DialogAction, ForkAction, SessionsAction,
    SlashCommandAction,
};

pub fn update(state: &mut AppState, action: Action) -> ActionResult {
    match action {
        Action::Quit => {
            state.should_quit = true;
            ActionResult::Handled
        }
        Action::ModeTransition(mode_action) => update_mode(state, mode_action),
        Action::Dialog(dialog_action) => update_dialog(state, dialog_action),
        Action::Timeline(timeline_action) => update_timeline(state, timeline_action),
        Action::Input(input_action) => update_input(state, input_action),
        Action::Connect(connect_action) => update_connect(state, connect_action),
        Action::Fork(fork_action) => update_fork(state, fork_action),
        Action::Sessions(sessions_action) => update_sessions(state, sessions_action),
        Action::SlashCommand(slash_command_action) => {
            update_slash_command(state, slash_command_action)
        }
        Action::FileDrop(paths) => {
            state.pending_file_drop = Some(paths);
            ActionResult::Handled
        }
    }
}

fn update_fork(state: &mut AppState, action: ForkAction) -> ActionResult {
    match action {
        ForkAction::Cancel => {
            state.mode = AppMode::Timeline;
            ActionResult::Handled
        }
        ForkAction::Confirm => ActionResult::Handled,
        ForkAction::InputChar(c) => {
            state.input_buffer.push(c);
            ActionResult::Handled
        }
        ForkAction::Backspace => {
            state.input_buffer.pop();
            ActionResult::Handled
        }
    }
}

fn update_sessions(state: &mut AppState, action: SessionsAction) -> ActionResult {
    match action {
        SessionsAction::Close => {
            state.mode = AppMode::Chat;
            ActionResult::Handled
        }
        SessionsAction::SelectPrevious => ActionResult::Handled,
        SessionsAction::SelectNext => ActionResult::Handled,
        SessionsAction::Confirm => ActionResult::Handled,
        SessionsAction::CreateNew => ActionResult::Handled,
    }
}

fn update_slash_command(state: &mut AppState, action: SlashCommandAction) -> ActionResult {
    match action {
        SlashCommandAction::Cancel => {
            state.mode = AppMode::Chat;
            ActionResult::Handled
        }
        SlashCommandAction::Confirm => ActionResult::Handled,
        SlashCommandAction::InputChar(c) => {
            state.input_buffer.push(c);
            ActionResult::Handled
        }
        SlashCommandAction::Backspace => {
            state.input_buffer.pop();
            ActionResult::Handled
        }
    }
}

fn update_mode(state: &mut AppState, action: crate::action::AppModeAction) -> ActionResult {
    match action {
        crate::action::AppModeAction::ShowHome => {
            state.mode = AppMode::Home;
            ActionResult::Handled
        }
        crate::action::AppModeAction::ShowChat => {
            state.mode = AppMode::Chat;
            ActionResult::Handled
        }
        crate::action::AppModeAction::ShowTimeline => {
            state.mode = AppMode::Timeline;
            ActionResult::Handled
        }
        crate::action::AppModeAction::ShowSettings => {
            state.mode = AppMode::Settings;
            ActionResult::Handled
        }
        crate::action::AppModeAction::ShowSessions => {
            state.mode = AppMode::Sessions;
            ActionResult::Handled
        }
        crate::action::AppModeAction::ShowProviderManagement => {
            state.mode = AppMode::ProviderManagement;
            ActionResult::Handled
        }
    }
}

fn update_dialog(state: &mut AppState, action: DialogAction) -> ActionResult {
    match action {
        DialogAction::Close => {
            close_current_dialog(state);
            ActionResult::Handled
        }
        DialogAction::NavigateUp => {
            dialog_navigate_up(state);
            ActionResult::Handled
        }
        DialogAction::NavigateDown => {
            dialog_navigate_down(state);
            ActionResult::Handled
        }
        DialogAction::Confirm => {
            dialog_confirm(state);
            ActionResult::Handled
        }
        DialogAction::Cancel => {
            dialog_cancel(state);
            ActionResult::Handled
        }
        DialogAction::Home => {
            dialog_go_home(state);
            ActionResult::Handled
        }
        DialogAction::End => dialog_go_end(state),
        DialogAction::PageUp => dialog_page_up(state),
        DialogAction::PageDown => dialog_page_down(state),
        DialogAction::InputChar(c) => {
            dialog_input_char(state, c);
            ActionResult::Handled
        }
        DialogAction::Backspace => {
            dialog_backspace(state);
            ActionResult::Handled
        }
        DialogAction::Clear => {
            dialog_clear(state);
            ActionResult::Handled
        }
        DialogAction::Select(index) => {
            dialog_select(state, index);
            ActionResult::Handled
        }
        DialogAction::Filter(filter) => {
            dialog_filter(state, filter);
            ActionResult::Handled
        }
        DialogAction::NavigateLeft => dialog_navigate_left(state),
        DialogAction::NavigateRight => dialog_navigate_right(state),
        DialogAction::Open(_) => ActionResult::NotHandled,
        DialogAction::ConfirmWithData(_) => ActionResult::Handled,
    }
}

fn update_timeline(state: &mut AppState, action: crate::action::TimelineAction) -> ActionResult {
    match action {
        crate::action::TimelineAction::SelectPrevious => {
            timeline_select_previous(state);
            ActionResult::Handled
        }
        crate::action::TimelineAction::SelectNext => {
            timeline_select_next(state);
            ActionResult::Handled
        }
        crate::action::TimelineAction::ToggleMetadata => {
            state.show_metadata = !state.show_metadata;
            ActionResult::Handled
        }
        crate::action::TimelineAction::ForkAtSelected => {
            fork_at_selected(state);
            ActionResult::Handled
        }
    }
}

fn update_input(state: &mut AppState, action: crate::action::InputAction) -> ActionResult {
    match action {
        crate::action::InputAction::Submit => {
            submit_input(state);
            ActionResult::Handled
        }
        crate::action::InputAction::MoveCursorLeft => {
            input_move_cursor_left(state);
            ActionResult::Handled
        }
        crate::action::InputAction::MoveCursorRight => {
            input_move_cursor_right(state);
            ActionResult::Handled
        }
        crate::action::InputAction::MoveToStart => {
            input_move_to_start(state);
            ActionResult::Handled
        }
        crate::action::InputAction::MoveToEnd => {
            input_move_to_end(state);
            ActionResult::Handled
        }
        crate::action::InputAction::DeleteChar => {
            input_delete_char(state);
            ActionResult::Handled
        }
        crate::action::InputAction::DeleteWord => {
            input_delete_word(state);
            ActionResult::Handled
        }
        crate::action::InputAction::HistoryPrevious => {
            input_history_previous(state);
            ActionResult::Handled
        }
        crate::action::InputAction::HistoryNext => {
            input_history_next(state);
            ActionResult::Handled
        }
        crate::action::InputAction::Complete => {
            input_complete(state);
            ActionResult::Handled
        }
    }
}

fn update_connect(state: &mut AppState, action: crate::action::ConnectAction) -> ActionResult {
    match action {
        crate::action::ConnectAction::SelectProvider(provider_id) => {
            state.pending_connect_provider = Some(provider_id);
            ActionResult::Handled
        }
        crate::action::ConnectAction::SelectMethod(method) => {
            state.pending_connect_method = Some(method);
            ActionResult::Handled
        }
        crate::action::ConnectAction::ConfirmApiKey(api_key) => {
            state.pending_api_key_for_validation = Some(api_key);
            ActionResult::Handled
        }
        crate::action::ConnectAction::Cancel => {
            state.pending_connect_provider = None;
            state.pending_connect_method = None;
            state.pending_api_key_for_validation = None;
            ActionResult::Handled
        }
        crate::action::ConnectAction::StartOAuth(provider) => {
            start_oauth(state, provider);
            ActionResult::Handled
        }
        crate::action::ConnectAction::ValidationComplete(result) => {
            state.validation_in_progress = false;
            if result.success {
                connect_validation_success(state, result.models);
            } else {
                connect_validation_failure(state, result.error_message);
            }
            ActionResult::Handled
        }
    }
}

fn close_current_dialog(state: &mut AppState) {
    match state.mode {
        AppMode::ConnectProvider => {
            state.mode = AppMode::Chat;
        }
        AppMode::ConnectMethod => {
            state.mode = AppMode::ConnectProvider;
        }
        AppMode::ConnectApiKey => {
            state.mode = AppMode::ConnectMethod;
        }
        AppMode::ConnectProgress => {
            state.mode = AppMode::ConnectApiKey;
        }
        AppMode::ConnectModel => {
            state.mode = AppMode::ConnectApiKey;
        }
        AppMode::ConnectApiKeyError => {
            state.mode = AppMode::ConnectApiKey;
        }
        AppMode::ModelSelection => {
            state.mode = AppMode::Chat;
        }
        AppMode::ForkDialog => {
            state.mode = AppMode::Timeline;
        }
        AppMode::CommandPalette => {
            state.mode = AppMode::Chat;
        }
        AppMode::SlashCommand => {
            state.mode = AppMode::Chat;
        }
        AppMode::Search => {
            state.mode = AppMode::Chat;
        }
        AppMode::FileSelection => {
            state.mode = AppMode::Chat;
        }
        AppMode::DirectorySelection => {
            state.mode = AppMode::Chat;
        }
        AppMode::DiffReview => {
            state.mode = AppMode::Chat;
        }
        AppMode::ReleaseNotes => {
            state.mode = AppMode::Home;
        }
        _ => {}
    }
}

fn dialog_navigate_up(state: &mut AppState) {
    if let Some(idx) = state.dialog_selected_index {
        if idx > 0 {
            state.dialog_selected_index = Some(idx - 1);
        }
    }
}

fn dialog_navigate_down(state: &mut AppState) {
    state.dialog_selected_index = Some(state.dialog_selected_index.unwrap_or(0) + 1);
}

fn dialog_confirm(_state: &mut AppState) {}
fn dialog_cancel(state: &mut AppState) {
    close_current_dialog(state);
}
fn dialog_go_home(state: &mut AppState) {
    state.dialog_selected_index = Some(0);
}
fn dialog_go_end(_state: &mut AppState) -> ActionResult {
    ActionResult::NotHandled
}

fn dialog_page_up(_state: &mut AppState) -> ActionResult {
    ActionResult::NotHandled
}

fn dialog_page_down(_state: &mut AppState) -> ActionResult {
    ActionResult::NotHandled
}

fn dialog_navigate_left(_state: &mut AppState) -> ActionResult {
    ActionResult::NotHandled
}

fn dialog_navigate_right(_state: &mut AppState) -> ActionResult {
    ActionResult::NotHandled
}
fn dialog_input_char(state: &mut AppState, c: char) {
    state.input_buffer.push(c);
}
fn dialog_backspace(state: &mut AppState) {
    state.input_buffer.pop();
}
fn dialog_clear(state: &mut AppState) {
    state.input_buffer.clear();
}
fn dialog_select(state: &mut AppState, index: usize) {
    state.dialog_selected_index = Some(index);
}
fn dialog_filter(state: &mut AppState, filter: String) {
    state.dialog_filter = Some(filter);
}

fn timeline_select_previous(_state: &mut AppState) {}
fn timeline_select_next(_state: &mut AppState) {}
fn fork_at_selected(_state: &mut AppState) {}

fn submit_input(_state: &mut AppState) {}
fn input_move_cursor_left(state: &mut AppState) {
    if state.input_cursor_position > 0 {
        state.input_cursor_position -= 1;
    }
}
fn input_move_cursor_right(state: &mut AppState) {
    if state.input_cursor_position < state.input_buffer.len() {
        state.input_cursor_position += 1;
    }
}
fn input_move_to_start(state: &mut AppState) {
    state.input_cursor_position = 0;
}
fn input_move_to_end(state: &mut AppState) {
    state.input_cursor_position = state.input_buffer.len();
}
fn input_delete_char(_state: &mut AppState) {}
fn input_delete_word(_state: &mut AppState) {}
fn input_history_previous(_state: &mut AppState) {}
fn input_history_next(_state: &mut AppState) {}
fn input_complete(_state: &mut AppState) {}

fn start_oauth(_state: &mut AppState, _provider: crate::action::OAuthProvider) {}
fn connect_validation_success(
    _state: &mut AppState,
    _models: Option<Vec<crate::action::BrowserAuthModelInfo>>,
) {
}
fn connect_validation_failure(_state: &mut AppState, _error: Option<String>) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{AppModeAction, DialogAction};

    #[test]
    fn test_update_quit() {
        let mut state = AppState::new();
        assert!(!state.should_quit);
        let result = update(&mut state, Action::Quit);
        assert_eq!(result, ActionResult::Handled);
        assert!(state.should_quit);
    }

    #[test]
    fn test_update_mode_transition() {
        let mut state = AppState::new();
        assert_eq!(state.mode, AppMode::Home);
        let result = update(&mut state, Action::ModeTransition(AppModeAction::ShowChat));
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.mode, AppMode::Chat);
    }

    #[test]
    fn test_update_dialog_navigate() {
        let mut state = AppState::new();
        state.dialog_selected_index = Some(5);

        let result = update(&mut state, Action::Dialog(DialogAction::NavigateUp));
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.dialog_selected_index, Some(4));

        let result = update(&mut state, Action::Dialog(DialogAction::NavigateDown));
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.dialog_selected_index, Some(5));
    }

    #[test]
    fn test_update_dialog_navigate_down_from_none() {
        let mut state = AppState::new();
        state.dialog_selected_index = None;

        let result = update(&mut state, Action::Dialog(DialogAction::NavigateDown));
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.dialog_selected_index, Some(1));
    }

    #[test]
    fn test_update_toggle_metadata() {
        let mut state = AppState::new();
        assert!(!state.show_metadata);
        let result = update(
            &mut state,
            Action::Timeline(crate::action::TimelineAction::ToggleMetadata),
        );
        assert_eq!(result, ActionResult::Handled);
        assert!(state.show_metadata);
    }

    #[test]
    fn test_update_input_char() {
        let mut state = AppState::new();
        assert_eq!(state.input_buffer, "");
        let result = update(&mut state, Action::Dialog(DialogAction::InputChar('h')));
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.input_buffer, "h");
    }

    #[test]
    fn test_update_input_backspace() {
        let mut state = AppState::new();
        state.input_buffer = "hello".to_string();
        let result = update(&mut state, Action::Dialog(DialogAction::Backspace));
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.input_buffer, "hell");
    }

    #[test]
    fn test_update_cursor_movement() {
        let mut state = AppState::new();
        state.input_buffer = "hello".to_string();
        state.input_cursor_position = 5;

        update(
            &mut state,
            Action::Input(crate::action::InputAction::MoveCursorLeft),
        );
        assert_eq!(state.input_cursor_position, 4);

        update(
            &mut state,
            Action::Input(crate::action::InputAction::MoveCursorLeft),
        );
        assert_eq!(state.input_cursor_position, 3);

        update(
            &mut state,
            Action::Input(crate::action::InputAction::MoveCursorRight),
        );
        assert_eq!(state.input_cursor_position, 4);
    }

    #[test]
    fn test_update_cursor_move_to_start_end() {
        let mut state = AppState::new();
        state.input_buffer = "hello".to_string();
        state.input_cursor_position = 3;

        update(
            &mut state,
            Action::Input(crate::action::InputAction::MoveToStart),
        );
        assert_eq!(state.input_cursor_position, 0);

        update(
            &mut state,
            Action::Input(crate::action::InputAction::MoveToEnd),
        );
        assert_eq!(state.input_cursor_position, 5);
    }

    #[test]
    fn test_update_connect_select_provider() {
        let mut state = AppState::new();
        assert!(state.pending_connect_provider.is_none());
        let result = update(
            &mut state,
            Action::Connect(crate::action::ConnectAction::SelectProvider(
                "openai".to_string(),
            )),
        );
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.pending_connect_provider, Some("openai".to_string()));
    }

    #[test]
    fn test_update_connect_cancel() {
        let mut state = AppState::new();
        state.pending_connect_provider = Some("openai".to_string());
        state.pending_connect_method = Some("api_key".to_string());
        state.pending_api_key_for_validation = Some("sk-123".to_string());

        let result = update(
            &mut state,
            Action::Connect(crate::action::ConnectAction::Cancel),
        );
        assert_eq!(result, ActionResult::Handled);
        assert!(state.pending_connect_provider.is_none());
        assert!(state.pending_connect_method.is_none());
        assert!(state.pending_api_key_for_validation.is_none());
    }

    #[test]
    fn test_update_fork_cancel() {
        let mut state = AppState::new();
        state.mode = AppMode::ForkDialog;

        let result = update(&mut state, Action::Fork(ForkAction::Cancel));
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.mode, AppMode::Timeline);
    }

    #[test]
    fn test_update_fork_input_char() {
        let mut state = AppState::new();
        assert_eq!(state.input_buffer, "");

        let result = update(&mut state, Action::Fork(ForkAction::InputChar('a')));
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.input_buffer, "a");

        update(&mut state, Action::Fork(ForkAction::InputChar('b')));
        assert_eq!(state.input_buffer, "ab");
    }

    #[test]
    fn test_update_fork_backspace() {
        let mut state = AppState::new();
        state.input_buffer = "hello".to_string();

        let result = update(&mut state, Action::Fork(ForkAction::Backspace));
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.input_buffer, "hell");
    }

    #[test]
    fn test_update_fork_confirm() {
        let mut state = AppState::new();
        state.mode = AppMode::ForkDialog;

        let result = update(&mut state, Action::Fork(ForkAction::Confirm));
        assert_eq!(result, ActionResult::Handled);
    }

    #[test]
    fn test_update_sessions_close() {
        let mut state = AppState::new();
        state.mode = AppMode::Sessions;

        let result = update(&mut state, Action::Sessions(SessionsAction::Close));
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.mode, AppMode::Chat);
    }

    #[test]
    fn test_update_sessions_select_previous() {
        let mut state = AppState::new();
        let result = update(&mut state, Action::Sessions(SessionsAction::SelectPrevious));
        assert_eq!(result, ActionResult::Handled);
    }

    #[test]
    fn test_update_sessions_select_next() {
        let mut state = AppState::new();
        let result = update(&mut state, Action::Sessions(SessionsAction::SelectNext));
        assert_eq!(result, ActionResult::Handled);
    }

    #[test]
    fn test_update_sessions_confirm() {
        let mut state = AppState::new();
        let result = update(&mut state, Action::Sessions(SessionsAction::Confirm));
        assert_eq!(result, ActionResult::Handled);
    }

    #[test]
    fn test_update_sessions_create_new() {
        let mut state = AppState::new();
        let result = update(&mut state, Action::Sessions(SessionsAction::CreateNew));
        assert_eq!(result, ActionResult::Handled);
    }

    #[test]
    fn test_update_slash_command_cancel() {
        let mut state = AppState::new();
        state.mode = AppMode::SlashCommand;

        let result = update(&mut state, Action::SlashCommand(SlashCommandAction::Cancel));
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.mode, AppMode::Chat);
    }

    #[test]
    fn test_update_slash_command_input_char() {
        let mut state = AppState::new();
        assert_eq!(state.input_buffer, "");

        let result = update(
            &mut state,
            Action::SlashCommand(SlashCommandAction::InputChar('/')),
        );
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.input_buffer, "/");
    }

    #[test]
    fn test_update_slash_command_backspace() {
        let mut state = AppState::new();
        state.input_buffer = "/help".to_string();

        let result = update(
            &mut state,
            Action::SlashCommand(SlashCommandAction::Backspace),
        );
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.input_buffer, "/hel");
    }

    #[test]
    fn test_update_slash_command_confirm() {
        let mut state = AppState::new();
        let result = update(
            &mut state,
            Action::SlashCommand(SlashCommandAction::Confirm),
        );
        assert_eq!(result, ActionResult::Handled);
    }

    #[test]
    fn test_update_mode_show_home() {
        let mut state = AppState::new();
        state.mode = AppMode::Chat;

        let result = update(&mut state, Action::ModeTransition(AppModeAction::ShowHome));
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.mode, AppMode::Home);
    }

    #[test]
    fn test_update_mode_show_timeline() {
        let mut state = AppState::new();
        state.mode = AppMode::Chat;

        let result = update(
            &mut state,
            Action::ModeTransition(AppModeAction::ShowTimeline),
        );
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.mode, AppMode::Timeline);
    }

    #[test]
    fn test_update_mode_show_settings() {
        let mut state = AppState::new();
        state.mode = AppMode::Home;

        let result = update(
            &mut state,
            Action::ModeTransition(AppModeAction::ShowSettings),
        );
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.mode, AppMode::Settings);
    }

    #[test]
    fn test_update_mode_show_sessions() {
        let mut state = AppState::new();
        state.mode = AppMode::Chat;

        let result = update(
            &mut state,
            Action::ModeTransition(AppModeAction::ShowSessions),
        );
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.mode, AppMode::Sessions);
    }

    #[test]
    fn test_update_mode_show_provider_management() {
        let mut state = AppState::new();
        state.mode = AppMode::Home;

        let result = update(
            &mut state,
            Action::ModeTransition(AppModeAction::ShowProviderManagement),
        );
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.mode, AppMode::ProviderManagement);
    }

    #[test]
    fn test_update_dialog_confirm() {
        let mut state = AppState::new();
        let result = update(&mut state, Action::Dialog(DialogAction::Confirm));
        assert_eq!(result, ActionResult::Handled);
    }

    #[test]
    fn test_update_dialog_cancel() {
        let mut state = AppState::new();
        state.mode = AppMode::ConnectMethod;

        let result = update(&mut state, Action::Dialog(DialogAction::Cancel));
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.mode, AppMode::ConnectProvider);
    }

    #[test]
    fn test_update_dialog_go_home() {
        let mut state = AppState::new();
        state.dialog_selected_index = Some(5);

        let result = update(&mut state, Action::Dialog(DialogAction::Home));
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.dialog_selected_index, Some(0));
    }

    #[test]
    fn test_update_dialog_go_end() {
        let mut state = AppState::new();
        let result = update(&mut state, Action::Dialog(DialogAction::End));
        assert_eq!(result, ActionResult::NotHandled);
    }

    #[test]
    fn test_update_dialog_page_up() {
        let mut state = AppState::new();
        let result = update(&mut state, Action::Dialog(DialogAction::PageUp));
        assert_eq!(result, ActionResult::NotHandled);
    }

    #[test]
    fn test_update_dialog_page_down() {
        let mut state = AppState::new();
        let result = update(&mut state, Action::Dialog(DialogAction::PageDown));
        assert_eq!(result, ActionResult::NotHandled);
    }

    #[test]
    fn test_update_dialog_clear() {
        let mut state = AppState::new();
        state.input_buffer = "hello world".to_string();

        let result = update(&mut state, Action::Dialog(DialogAction::Clear));
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.input_buffer, "");
    }

    #[test]
    fn test_update_dialog_select() {
        let mut state = AppState::new();

        let result = update(&mut state, Action::Dialog(DialogAction::Select(3)));
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.dialog_selected_index, Some(3));

        update(&mut state, Action::Dialog(DialogAction::Select(7)));
        assert_eq!(state.dialog_selected_index, Some(7));
    }

    #[test]
    fn test_update_dialog_filter() {
        let mut state = AppState::new();
        assert!(state.dialog_filter.is_none());

        let result = update(
            &mut state,
            Action::Dialog(DialogAction::Filter("test filter".to_string())),
        );
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.dialog_filter, Some("test filter".to_string()));
    }

    #[test]
    fn test_update_dialog_close_current_dialog_fork() {
        let mut state = AppState::new();
        state.mode = AppMode::ForkDialog;

        close_current_dialog(&mut state);
        assert_eq!(state.mode, AppMode::Timeline);
    }

    #[test]
    fn test_update_dialog_close_current_dialog_search() {
        let mut state = AppState::new();
        state.mode = AppMode::Search;

        close_current_dialog(&mut state);
        assert_eq!(state.mode, AppMode::Chat);
    }

    #[test]
    fn test_update_dialog_close_current_dialog_release_notes() {
        let mut state = AppState::new();
        state.mode = AppMode::ReleaseNotes;

        close_current_dialog(&mut state);
        assert_eq!(state.mode, AppMode::Home);
    }

    #[test]
    fn test_update_dialog_close_current_dialog_model_selection() {
        let mut state = AppState::new();
        state.mode = AppMode::ModelSelection;

        close_current_dialog(&mut state);
        assert_eq!(state.mode, AppMode::Chat);
    }

    #[test]
    fn test_update_timeline_select_previous() {
        let mut state = AppState::new();
        let result = update(
            &mut state,
            Action::Timeline(crate::action::TimelineAction::SelectPrevious),
        );
        assert_eq!(result, ActionResult::Handled);
    }

    #[test]
    fn test_update_timeline_select_next() {
        let mut state = AppState::new();
        let result = update(
            &mut state,
            Action::Timeline(crate::action::TimelineAction::SelectNext),
        );
        assert_eq!(result, ActionResult::Handled);
    }

    #[test]
    fn test_update_timeline_fork_at_selected() {
        let mut state = AppState::new();
        let result = update(
            &mut state,
            Action::Timeline(crate::action::TimelineAction::ForkAtSelected),
        );
        assert_eq!(result, ActionResult::Handled);
    }

    #[test]
    fn test_update_connect_select_method() {
        let mut state = AppState::new();
        assert!(state.pending_connect_method.is_none());

        let result = update(
            &mut state,
            Action::Connect(crate::action::ConnectAction::SelectMethod(
                "oauth".to_string(),
            )),
        );
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.pending_connect_method, Some("oauth".to_string()));
    }

    #[test]
    fn test_update_connect_confirm_api_key() {
        let mut state = AppState::new();
        assert!(state.pending_api_key_for_validation.is_none());

        let result = update(
            &mut state,
            Action::Connect(crate::action::ConnectAction::ConfirmApiKey(
                "sk-test123".to_string(),
            )),
        );
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(
            state.pending_api_key_for_validation,
            Some("sk-test123".to_string())
        );
    }

    #[test]
    fn test_update_file_drop() {
        use std::path::PathBuf;

        let mut state = AppState::new();
        assert!(state.pending_file_drop.is_none());

        let paths = vec![
            PathBuf::from("/path/to/file1.rs"),
            PathBuf::from("/path/to/file2.rs"),
        ];
        let result = update(&mut state, Action::FileDrop(paths.clone()));
        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.pending_file_drop, Some(paths));
    }

    #[test]
    fn test_update_dialog_navigate_left() {
        let mut state = AppState::new();
        let result = update(&mut state, Action::Dialog(DialogAction::NavigateLeft));
        assert_eq!(result, ActionResult::NotHandled);
    }

    #[test]
    fn test_update_dialog_navigate_right() {
        let mut state = AppState::new();
        let result = update(&mut state, Action::Dialog(DialogAction::NavigateRight));
        assert_eq!(result, ActionResult::NotHandled);
    }

    #[test]
    fn test_update_dialog_open_returns_not_handled() {
        let mut state = AppState::new();
        let result = update(
            &mut state,
            Action::Dialog(DialogAction::Open("test".to_string())),
        );
        assert_eq!(result, ActionResult::NotHandled);
    }

    #[test]
    fn test_update_dialog_confirm_with_data() {
        let mut state = AppState::new();
        let result = update(
            &mut state,
            Action::Dialog(DialogAction::ConfirmWithData("data".to_string())),
        );
        assert_eq!(result, ActionResult::Handled);
    }
}
