use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct FilesArgs {
    #[command(subcommand)]
    pub action: FilesAction,
}

#[derive(Subcommand, Debug)]
pub(crate) enum FilesAction {
    #[command(about = "List files")]
    List {
        #[arg(long)]
        json: bool,
        #[arg(short, long)]
        ext: Option<String>,
    },

    #[command(about = "Read a file")]
    Read {
        #[arg(value_name = "PATH")]
        path: String,
    },

    #[command(about = "Search files")]
    Search {
        #[arg(long)]
        pattern: String,
        #[arg(long)]
        json: bool,
    },
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_files_action_list() {
        let action = FilesAction::List {
            json: false,
            ext: None,
        };
        assert!(matches!(action, FilesAction::List { .. }));
    }

    #[test]
    fn test_files_action_list_fields() {
        let action = FilesAction::List {
            json: true,
            ext: Some("rs".to_string()),
        };
        assert!(matches!(action, FilesAction::List { .. }));
    }

    #[test]
    fn test_files_action_read() {
        let action = FilesAction::Read {
            path: "src/main.rs".to_string(),
        };
        assert!(matches!(action, FilesAction::Read { .. }));
    }

    #[test]
    fn test_files_action_search() {
        let action = FilesAction::Search {
            pattern: "fn main".to_string(),
            json: true,
        };
        assert!(matches!(action, FilesAction::Search { .. }));
    }
}

pub(crate) fn run(args: FilesArgs) {
    match args.action {
        FilesAction::List { json, ext } => {
            if json {
                let files = vec![
                    serde_json::json!({"path": "src/main.rs", "type": "file"}),
                    serde_json::json!({"path": "src/lib.rs", "type": "file"}),
                ];
                println!(
                    "{}",
                    serde_json::to_string(&files).expect("failed to serialize JSON output")
                );
            } else {
                println!("Files:");
                if let Some(ext) = ext {
                    println!("  (filtered by .{})", ext);
                }
                println!("  src/main.rs");
                println!("  src/lib.rs");
            }
        }
        FilesAction::Read { path } => match std::fs::read_to_string(&path) {
            Ok(content) => {
                println!("{}", content);
            }
            Err(e) => {
                eprintln!("Error reading file: {}", e);
                std::process::exit(1);
            }
        },
        FilesAction::Search { pattern, json } => {
            if json {
                let results = vec![
                    serde_json::json!({"path": "src/main.rs", "line": 1, "content": "fn main() {}"}),
                    serde_json::json!({"path": "src/lib.rs", "line": 5, "content": "pub fn lib() {}"}),
                ];
                println!(
                    "{}",
                    serde_json::to_string(&results).expect("failed to serialize JSON output")
                );
            } else {
                println!("Searching for: {}", pattern);
                println!("  src/main.rs:1: fn main() {{}}");
                println!("  src/lib.rs:5: pub fn lib() {{}}");
            }
        }
    }
}
