mod types;

pub use types::{SessionEvent, SessionState, StateTransitionError};

impl SessionState {
    #[allow(clippy::match_like_matches_macro)]
    pub fn can_transition_to(&self, target: &SessionState) -> bool {
        match (self, target) {
            (SessionState::Idle, SessionState::Thinking) => true,
            (SessionState::Idle, SessionState::Summarizing) => true,
            (SessionState::Idle, SessionState::Aborted) => true,
            (SessionState::Idle, SessionState::Paused) => true,
            (SessionState::Idle, SessionState::Error) => true,

            (SessionState::Thinking, SessionState::AwaitingPermission) => true,
            (SessionState::Thinking, SessionState::ExecutingTool) => true,
            (SessionState::Thinking, SessionState::Streaming) => true,
            (SessionState::Thinking, SessionState::Error) => true,

            (SessionState::AwaitingPermission, SessionState::ExecutingTool) => true,
            (SessionState::AwaitingPermission, SessionState::Idle) => true,
            (SessionState::AwaitingPermission, SessionState::Error) => true,

            (SessionState::ExecutingTool, SessionState::Thinking) => true,
            (SessionState::ExecutingTool, SessionState::ApplyingChanges) => true,
            (SessionState::ExecutingTool, SessionState::Error) => true,

            (SessionState::Streaming, SessionState::Thinking) => true,
            (SessionState::Streaming, SessionState::Completed) => true,
            (SessionState::Streaming, SessionState::Idle) => true,
            (SessionState::Streaming, SessionState::Error) => true,

            (SessionState::ApplyingChanges, SessionState::Verifying) => true,
            (SessionState::ApplyingChanges, SessionState::Thinking) => true,
            (SessionState::ApplyingChanges, SessionState::Error) => true,

            (SessionState::Verifying, SessionState::Thinking) => true,
            (SessionState::Verifying, SessionState::Idle) => true,
            (SessionState::Verifying, SessionState::Completed) => true,
            (SessionState::Verifying, SessionState::Error) => true,

            (SessionState::Summarizing, SessionState::Idle) => true,
            (SessionState::Summarizing, SessionState::Completed) => true,

            (SessionState::Aborted, SessionState::Idle) => true,

            (SessionState::Error, SessionState::Idle) => true,
            (SessionState::Error, SessionState::Thinking) => true,

            (SessionState::Completed, SessionState::Idle) => true,
            (SessionState::Completed, SessionState::Thinking) => true,

            (SessionState::Paused, SessionState::Idle) => true,
            (SessionState::Paused, SessionState::Thinking) => true,

            _ => false,
        }
    }

    pub fn get_event(&self) -> SessionEvent {
        match self {
            SessionState::Idle => SessionEvent::PromptReceived,
            SessionState::Thinking => SessionEvent::ToolExecutionRequested,
            SessionState::AwaitingPermission => SessionEvent::PermissionGranted,
            SessionState::ExecutingTool => SessionEvent::ToolExecutionCompleted,
            SessionState::Streaming => SessionEvent::StreamStarted,
            SessionState::ApplyingChanges => SessionEvent::ChangesApplied,
            SessionState::Verifying => SessionEvent::VerificationCompleted,
            SessionState::Summarizing => SessionEvent::SummarizeRequested,
            SessionState::Aborted => SessionEvent::AbortRequested,
            SessionState::Error => SessionEvent::ErrorOccurred,
            SessionState::Completed => SessionEvent::StreamCompleted,
            SessionState::Paused => SessionEvent::PauseRequested,
        }
    }
}

pub fn is_valid_transition(from: SessionState, to: SessionState) -> bool {
    from.can_transition_to(&to)
}

pub fn transition_to(
    state: &mut SessionState,
    target: SessionState,
) -> Result<(), StateTransitionError> {
    if state.can_transition_to(&target) {
        *state = target;
        Ok(())
    } else {
        Err(StateTransitionError {
            from: *state,
            to: target,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_idle_to_thinking() {
        assert!(is_valid_transition(
            SessionState::Idle,
            SessionState::Thinking
        ));
    }

    #[test]
    fn test_valid_thinking_to_awaiting_permission() {
        assert!(is_valid_transition(
            SessionState::Thinking,
            SessionState::AwaitingPermission
        ));
    }

    #[test]
    fn test_valid_awaiting_to_executing() {
        assert!(is_valid_transition(
            SessionState::AwaitingPermission,
            SessionState::ExecutingTool
        ));
    }

    #[test]
    fn test_valid_executing_to_thinking() {
        assert!(is_valid_transition(
            SessionState::ExecutingTool,
            SessionState::Thinking
        ));
    }

    #[test]
    fn test_valid_thinking_to_streaming() {
        assert!(is_valid_transition(
            SessionState::Thinking,
            SessionState::Streaming
        ));
    }

    #[test]
    fn test_valid_streaming_to_completed() {
        assert!(is_valid_transition(
            SessionState::Streaming,
            SessionState::Completed
        ));
    }

    #[test]
    fn test_valid_completed_to_idle() {
        assert!(is_valid_transition(
            SessionState::Completed,
            SessionState::Idle
        ));
    }

    #[test]
    fn test_valid_any_to_error() {
        assert!(is_valid_transition(
            SessionState::Thinking,
            SessionState::Error
        ));
        assert!(is_valid_transition(
            SessionState::ExecutingTool,
            SessionState::Error
        ));
        assert!(is_valid_transition(
            SessionState::Streaming,
            SessionState::Error
        ));
    }

    #[test]
    fn test_valid_error_to_idle() {
        assert!(is_valid_transition(SessionState::Error, SessionState::Idle));
    }

    #[test]
    fn test_valid_idle_to_summarizing() {
        assert!(is_valid_transition(
            SessionState::Idle,
            SessionState::Summarizing
        ));
    }

    #[test]
    fn test_valid_summarizing_to_idle() {
        assert!(is_valid_transition(
            SessionState::Summarizing,
            SessionState::Idle
        ));
    }

    #[test]
    fn test_valid_idle_to_aborted() {
        assert!(is_valid_transition(
            SessionState::Idle,
            SessionState::Aborted
        ));
    }

    #[test]
    fn test_valid_aborted_to_idle() {
        assert!(is_valid_transition(
            SessionState::Aborted,
            SessionState::Idle
        ));
    }

    #[test]
    fn test_valid_idle_to_paused() {
        assert!(is_valid_transition(
            SessionState::Idle,
            SessionState::Paused
        ));
    }

    #[test]
    fn test_valid_paused_to_thinking() {
        assert!(is_valid_transition(
            SessionState::Paused,
            SessionState::Thinking
        ));
    }

    #[test]
    fn test_invalid_idle_to_completed() {
        assert!(!is_valid_transition(
            SessionState::Idle,
            SessionState::Completed
        ));
    }

    #[test]
    fn test_invalid_streaming_to_awaiting() {
        assert!(!is_valid_transition(
            SessionState::Streaming,
            SessionState::AwaitingPermission
        ));
    }

    #[test]
    fn test_invalid_paused_to_completed() {
        assert!(!is_valid_transition(
            SessionState::Paused,
            SessionState::Completed
        ));
    }

    #[test]
    fn test_default_state_is_idle() {
        assert_eq!(SessionState::default(), SessionState::Idle);
    }

    #[test]
    fn test_transition_to_success() {
        let mut state = SessionState::Idle;
        assert!(transition_to(&mut state, SessionState::Thinking).is_ok());
        assert_eq!(state, SessionState::Thinking);
    }

    #[test]
    fn test_transition_to_failure() {
        let mut state = SessionState::Idle;
        let result = transition_to(&mut state, SessionState::Completed);
        assert!(result.is_err());
        assert_eq!(state, SessionState::Idle);
    }

    #[test]
    fn test_event_mapping() {
        assert_eq!(SessionState::Idle.get_event(), SessionEvent::PromptReceived);
        assert_eq!(
            SessionState::Thinking.get_event(),
            SessionEvent::ToolExecutionRequested
        );
        assert_eq!(
            SessionState::AwaitingPermission.get_event(),
            SessionEvent::PermissionGranted
        );
        assert_eq!(
            SessionState::ExecutingTool.get_event(),
            SessionEvent::ToolExecutionCompleted
        );
        assert_eq!(
            SessionState::Streaming.get_event(),
            SessionEvent::StreamStarted
        );
    }
}
