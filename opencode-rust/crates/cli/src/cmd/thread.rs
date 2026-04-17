use clap::Args;
use opencode_tui::App;

#[derive(Args, Debug)]
pub(crate) struct ThreadArgs {
    #[arg(short, long)]
    pub session_id: Option<String>,
}

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
    if let Err(e) = app.run() {
        eprintln!("Error running TUI: {}", e);
    }
}
