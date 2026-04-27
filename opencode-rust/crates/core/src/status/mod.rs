mod types;

pub use types::{SessionStatus, Status};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_status_new() {
        let status = SessionStatus::new("test-session".to_string());
        assert_eq!(status.session_id, "test-session");
        assert!(matches!(status.status, Status::Idle));
        assert_eq!(status.message_count, 0);
        assert_eq!(status.tool_calls, 0);
        assert_eq!(status.errors, 0);
    }

    #[test]
    fn test_session_status_update() {
        let mut status = SessionStatus::new("test".to_string());
        status.update(Status::Processing);
        assert!(status.is_processing());
    }

    #[test]
    fn test_session_status_increment_messages() {
        let mut status = SessionStatus::new("test".to_string());
        status.increment_messages();
        assert_eq!(status.message_count, 1);
    }

    #[test]
    fn test_session_status_increment_tool_calls() {
        let mut status = SessionStatus::new("test".to_string());
        status.increment_tool_calls();
        assert_eq!(status.tool_calls, 1);
    }

    #[test]
    fn test_session_status_increment_errors() {
        let mut status = SessionStatus::new("test".to_string());
        status.increment_errors();
        assert_eq!(status.errors, 1);
    }

    #[test]
    fn test_session_status_is_idle() {
        let status = SessionStatus::new("test".to_string());
        assert!(status.is_idle());
    }

    #[test]
    fn test_session_status_is_processing() {
        let mut status = SessionStatus::new("test".to_string());
        status.update(Status::Processing);
        assert!(status.is_processing());
    }
}