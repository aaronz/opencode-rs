use clap::{Args, Subcommand};
use std::path::Path;

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

#[derive(serde::Serialize)]
struct FileInfo {
    path: String,
    file_type: String,
    size: u64,
}

#[derive(serde::Serialize)]
struct SearchResult {
    path: String,
    line: usize,
    content: String,
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
            list_files(json, ext.as_deref());
        }
        FilesAction::Read { path } => {
            read_file(&path);
        }
        FilesAction::Search { pattern, json } => {
            search_files(&pattern, json);
        }
    }
}

fn list_files(json: bool, ext_filter: Option<&str>) {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());

    let mut files: Vec<FileInfo> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&current_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if file_name.starts_with('.') {
                continue;
            }

            if let Some(ext) = ext_filter {
                if path.extension().and_then(|e| e.to_str()).unwrap_or("") != ext {
                    continue;
                }
            }

            let file_type = if path.is_dir() {
                "directory"
            } else if path.is_file() {
                "file"
            } else {
                "other"
            };

            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);

            files.push(FileInfo {
                path: path.to_string_lossy().to_string(),
                file_type: file_type.to_string(),
                size,
            });
        }
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&files).expect("failed to serialize JSON output")
        );
    } else {
        if files.is_empty() {
            println!("No files found in current directory");
            if let Some(ext) = ext_filter {
                println!("  (filtered by .{})", ext);
            }
        } else {
            for file in &files {
                println!("  {} ({} bytes) - {}", file.path, file.size, file.file_type);
            }
            println!("\n{} files found", files.len());
        }
    }
}

fn read_file(path: &str) {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            println!("{}", content);
        }
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            std::process::exit(1);
        }
    }
}

fn search_files(pattern: &str, json: bool) {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let mut results: Vec<SearchResult> = Vec::new();

    search_directory(&current_dir, pattern, &mut results, 0);

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&results).expect("failed to serialize JSON output")
        );
    } else {
        if results.is_empty() {
            println!("No matches found for '{}'", pattern);
        } else {
            for result in &results {
                println!("{}:{}: {}", result.path, result.line, result.content);
            }
            println!("\n{} matches found", results.len());
        }
    }
}

fn search_directory(dir: &Path, pattern: &str, results: &mut Vec<SearchResult>, depth: usize) {
    if depth > 10 {
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with('.') && name != "target" && name != "node_modules" {
                search_directory(&path, pattern, results, depth + 1);
            }
        } else if path.is_file() {
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for (line_num, line) in content.lines().enumerate() {
                if line.contains(pattern) {
                    results.push(SearchResult {
                        path: path.to_string_lossy().to_string(),
                        line: line_num + 1,
                        content: line.chars().take(100).collect(),
                    });
                }
            }
        }
    }
}
