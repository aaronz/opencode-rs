use crate::cmd::session::get_session_sharing_for_quick as get_session_sharing;
use clap::Args;

#[derive(Args, Debug)]
pub(crate) struct ExportArgs {
    #[arg(short, long)]
    pub output: Option<String>,

    #[arg(short, long)]
    pub format: Option<String>,
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_args_default() {
        let args = ExportArgs {
            output: None,
            format: None,
        };
        assert!(args.output.is_none());
        assert!(args.format.is_none());
    }

    #[test]
    fn test_export_args_with_output() {
        let args = ExportArgs {
            output: Some("export.json".to_string()),
            format: None,
        };
        assert_eq!(args.output.as_deref(), Some("export.json"));
    }

    #[test]
    fn test_export_args_with_format() {
        let args = ExportArgs {
            output: None,
            format: Some("json".to_string()),
        };
        assert_eq!(args.format.as_deref(), Some("json"));
    }

    #[test]
    fn test_export_args_full() {
        let args = ExportArgs {
            output: Some("sessions.json".to_string()),
            format: Some("json".to_string()),
        };
        assert_eq!(args.output.as_deref(), Some("sessions.json"));
        assert_eq!(args.format.as_deref(), Some("json"));
    }
}

pub(crate) fn run(args: ExportArgs) {
    let sharing = get_session_sharing();
    let sessions = sharing.list_sessions().unwrap_or_default();

    let export_data = serde_json::json!({
        "sessions": sessions,
        "count": sessions.len(),
        "exported_at": chrono::Utc::now().to_rfc3339(),
    });

    let output =
        serde_json::to_string_pretty(&export_data).expect("failed to serialize JSON output");

    if let Some(path) = &args.output {
        if let Err(e) = std::fs::write(path, &output) {
            eprintln!("Error writing to file '{}': {}", path, e);
            std::process::exit(1);
        }
        println!("Exported {} sessions to '{}'", sessions.len(), path);
    } else {
        println!("{}", output);
    }
}
