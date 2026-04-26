use crate::cmd::session::get_session_sharing_for_quick as get_session_sharing;
use clap::Args;
use opencode_core::Session;

#[derive(Args, Debug)]
pub(crate) struct ImportArgs {
    #[arg(short, long)]
    pub input: String,

    #[arg(short, long)]
    pub format: Option<String>,
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_args_with_input() {
        let args = ImportArgs {
            input: "import.json".to_string(),
            format: None,
        };
        assert_eq!(args.input, "import.json");
        assert!(args.format.is_none());
    }

    #[test]
    fn test_import_args_with_format() {
        let args = ImportArgs {
            input: "data.json".to_string(),
            format: Some("json".to_string()),
        };
        assert_eq!(args.input, "data.json");
        assert_eq!(args.format.as_deref(), Some("json"));
    }

    #[test]
    fn test_import_args_full() {
        let args = ImportArgs {
            input: "/path/to/import.json".to_string(),
            format: Some("json".to_string()),
        };
        assert_eq!(args.input, "/path/to/import.json");
        assert_eq!(args.format.as_deref(), Some("json"));
    }
}

pub(crate) fn run(args: ImportArgs) {
    let content = match std::fs::read_to_string(&args.input) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", args.input, e);
            std::process::exit(1);
        }
    };

    let sharing = get_session_sharing();

    let imported: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error parsing JSON from '{}': {}", args.input, e);
            std::process::exit(1);
        }
    };

    let mut imported_count = 0;

    if let Some(sessions) = imported.get("sessions").and_then(|s| s.as_array()) {
        for session_value in sessions {
            if let Ok(session) = serde_json::from_value::<Session>(session_value.clone()) {
                match sharing.save_session(&session) {
                    Ok(_) => imported_count += 1,
                    Err(e) => {
                        eprintln!("Warning: Failed to import session {}: {}", session.id, e);
                    }
                }
            }
        }
    } else if let Ok(_session) = serde_json::from_value::<Session>(imported.clone()) {
        if let Ok(session) = serde_json::from_value::<Session>(imported) {
            match sharing.save_session(&session) {
                Ok(_) => imported_count = 1,
                Err(e) => {
                    eprintln!("Error importing session: {}", e);
                    std::process::exit(1);
                }
            }
        }
    } else {
        eprintln!(
            "Error: File '{}' does not contain valid session data",
            args.input
        );
        std::process::exit(1);
    }

    println!("Imported {} sessions from '{}'", imported_count, args.input);
}
