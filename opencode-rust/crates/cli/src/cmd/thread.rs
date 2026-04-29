use clap::Args;
use opencode_tui::App;

#[derive(Args, Debug)]
pub(crate) struct ThreadArgs {
    #[arg(short, long)]
    pub session_id: Option<String>,
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_args_default() {
        let args = ThreadArgs { session_id: None };
        assert!(args.session_id.is_none());
    }

    #[test]
    fn test_thread_args_with_session_id() {
        let args = ThreadArgs {
            session_id: Some("thread-session-123".to_string()),
        };
        assert_eq!(args.session_id.as_deref(), Some("thread-session-123"));
    }
}

pub(crate) fn run(_args: ThreadArgs) {
    let mut app = App::new();
    if let Err(message) =
        crate::finalize_tui_run_result(app.run(), App::restore_terminal_after_error)
    {
        tracing::error!(error = %message, "TUI run failed");
    }
}
