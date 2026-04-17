use crate::cmd::session::{append_session_message, load_session_records};
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub(crate) struct PromptArgs {
    #[arg(short, long)]
    pub session: Option<String>,

    #[arg(long)]
    pub history: bool,

    #[arg(long)]
    pub history_up: bool,

    #[arg(long)]
    pub history_down: bool,

    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub content: Option<String>,

    #[arg(long)]
    pub context: Option<String>,

    #[arg(long = "async")]
    pub asynchronous: bool,

    #[arg(long)]
    pub cancel: bool,

    #[arg(long = "queue-status")]
    pub queue_status: bool,

    #[arg(long)]
    pub shell: bool,

    #[arg(long)]
    pub terminal: bool,

    #[arg(long)]
    pub multiline: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_args_default() {
        let args = PromptArgs {
            session: None,
            history: false,
            history_up: false,
            history_down: false,
            json: false,
            content: None,
            context: None,
            asynchronous: false,
            cancel: false,
            queue_status: false,
            shell: false,
            terminal: false,
            multiline: false,
        };
        assert!(args.session.is_none());
        assert!(!args.history);
        assert!(!args.history_up);
        assert!(!args.history_down);
        assert!(!args.json);
        assert!(args.content.is_none());
        assert!(args.context.is_none());
        assert!(!args.asynchronous);
        assert!(!args.cancel);
        assert!(!args.queue_status);
        assert!(!args.shell);
        assert!(!args.terminal);
        assert!(!args.multiline);
    }

    #[test]
    fn test_prompt_args_with_session() {
        let args = PromptArgs {
            session: Some("session-123".to_string()),
            history: false,
            history_up: false,
            history_down: false,
            json: false,
            content: None,
            context: None,
            asynchronous: false,
            cancel: false,
            queue_status: false,
            shell: false,
            terminal: false,
            multiline: false,
        };
        assert_eq!(args.session.as_deref(), Some("session-123"));
    }

    #[test]
    fn test_prompt_args_with_content() {
        let args = PromptArgs {
            session: None,
            history: false,
            history_up: false,
            history_down: false,
            json: false,
            content: Some("Hello world".to_string()),
            context: None,
            asynchronous: false,
            cancel: false,
            queue_status: false,
            shell: false,
            terminal: false,
            multiline: false,
        };
        assert_eq!(args.content.as_deref(), Some("Hello world"));
    }

    #[test]
    fn test_prompt_args_with_context() {
        let args = PromptArgs {
            session: None,
            history: false,
            history_up: false,
            history_down: false,
            json: false,
            content: Some("Hello".to_string()),
            context: Some("/path/to/context".to_string()),
            asynchronous: false,
            cancel: false,
            queue_status: false,
            shell: false,
            terminal: false,
            multiline: false,
        };
        assert_eq!(args.context.as_deref(), Some("/path/to/context"));
    }

    #[test]
    fn test_prompt_args_history_flags() {
        let args = PromptArgs {
            session: None,
            history: true,
            history_up: true,
            history_down: false,
            json: false,
            content: None,
            context: None,
            asynchronous: false,
            cancel: false,
            queue_status: false,
            shell: false,
            terminal: false,
            multiline: false,
        };
        assert!(args.history);
        assert!(args.history_up);
        assert!(!args.history_down);
    }

    #[test]
    fn test_prompt_args_with_json() {
        let args = PromptArgs {
            session: None,
            history: true,
            history_up: false,
            history_down: false,
            json: true,
            content: None,
            context: None,
            asynchronous: false,
            cancel: false,
            queue_status: false,
            shell: false,
            terminal: false,
            multiline: false,
        };
        assert!(args.json);
    }

    #[test]
    fn test_prompt_args_async_and_terminal() {
        let args = PromptArgs {
            session: Some("session-123".to_string()),
            history: false,
            history_up: false,
            history_down: false,
            json: false,
            content: Some("async content".to_string()),
            context: None,
            asynchronous: true,
            cancel: false,
            queue_status: false,
            shell: false,
            terminal: true,
            multiline: false,
        };
        assert!(args.asynchronous);
        assert!(args.terminal);
        assert!(!args.multiline);
    }

    #[test]
    fn test_prompt_args_cancel() {
        let args = PromptArgs {
            session: Some("session-123".to_string()),
            history: false,
            history_up: false,
            history_down: false,
            json: false,
            content: None,
            context: None,
            asynchronous: false,
            cancel: true,
            queue_status: false,
            shell: false,
            terminal: false,
            multiline: false,
        };
        assert!(args.cancel);
    }

    #[test]
    fn test_prompt_args_queue_status() {
        let args = PromptArgs {
            session: None,
            history: false,
            history_up: false,
            history_down: false,
            json: true,
            content: None,
            context: None,
            asynchronous: false,
            cancel: false,
            queue_status: true,
            shell: false,
            terminal: false,
            multiline: false,
        };
        assert!(args.queue_status);
        assert!(args.json);
    }

    #[test]
    fn test_prompt_args_shell() {
        let args = PromptArgs {
            session: Some("session-123".to_string()),
            history: false,
            history_up: false,
            history_down: false,
            json: false,
            content: Some("echo hello".to_string()),
            context: None,
            asynchronous: false,
            cancel: false,
            queue_status: false,
            shell: true,
            terminal: false,
            multiline: false,
        };
        assert!(args.shell);
    }

    #[test]
    fn test_prompt_args_multiline() {
        let args = PromptArgs {
            session: None,
            history: false,
            history_up: false,
            history_down: false,
            json: false,
            content: Some("multi\nline\ncontent".to_string()),
            context: None,
            asynchronous: false,
            cancel: false,
            queue_status: false,
            shell: false,
            terminal: false,
            multiline: true,
        };
        assert!(args.multiline);
    }

    #[test]
    fn test_prompt_queue_path_default() {
        std::env::remove_var("OPENCODE_DATA_DIR");
        let path = prompt_queue_path();
        assert_eq!(
            path.file_name().map(|s| s.to_str()),
            Some(Some("prompt-queue.json"))
        );
    }

    #[test]
    fn test_prompt_queue_path_with_data_dir() {
        let temp_dir = std::env::temp_dir();
        std::env::set_var("OPENCODE_DATA_DIR", temp_dir.to_string_lossy().as_ref());
        let path = prompt_queue_path();
        assert!(path.to_string_lossy().contains("prompt-queue.json"));
        std::env::remove_var("OPENCODE_DATA_DIR");
    }
}

fn prompt_queue_path() -> PathBuf {
    if let Ok(data_dir) = std::env::var("OPENCODE_DATA_DIR") {
        let path = PathBuf::from(data_dir);
        let _ = std::fs::create_dir_all(&path);
        return path.join("prompt-queue.json");
    }
    PathBuf::from("./prompt-queue.json")
}

fn load_queue() -> Vec<serde_json::Value> {
    let path = prompt_queue_path();
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<Vec<serde_json::Value>>(&content).ok())
        .unwrap_or_default()
}

fn save_queue(queue: &[serde_json::Value]) {
    let path = prompt_queue_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(
        path,
        serde_json::to_string_pretty(queue).expect("failed to serialize queue"),
    )
    .expect("failed to write queue file");
}

pub(crate) fn run(args: PromptArgs) {
    if args.history {
        let history = if let Some(session_id) = args.session.as_deref() {
            load_session_records()
                .into_iter()
                .find(|record| record.id == session_id)
                .map(|record| {
                    record
                        .messages
                        .into_iter()
                        .map(|message| {
                            serde_json::json!({
                                "content": message.content,
                                "timestamp": "stored",
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            vec![
                serde_json::json!({"content": "First message", "timestamp": "2024-01-01T00:00:00Z"}),
                serde_json::json!({"content": "Second message", "timestamp": "2024-01-01T00:01:00Z"}),
                serde_json::json!({"content": "Third message", "timestamp": "2024-01-01T00:02:00Z"}),
            ]
        };

        if args.json {
            println!(
                "{}",
                serde_json::to_string(&history).expect("failed to serialize JSON output")
            );
        } else {
            println!("Prompt history:");
            for entry in history {
                if let Some(content) = entry["content"].as_str() {
                    println!("  {}", content);
                }
            }
        }
        return;
    }

    if args.history_up {
        println!("History up");
        return;
    }

    if args.history_down {
        println!("History down");
        return;
    }

    if args.cancel {
        if let Some(session_id) = args.session {
            let mut queue = load_queue();
            queue.retain(|entry| entry["session"] != session_id);
            save_queue(&queue);
        }
        println!("Prompt cancelled");
        return;
    }

    if args.queue_status {
        let queue = load_queue();
        let pending = if let Some(session_id) = args.session {
            queue
                .into_iter()
                .filter(|entry| entry["session"] == session_id)
                .count()
        } else {
            queue.len()
        };
        if args.json {
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({"pending": pending}))
                    .expect("failed to serialize JSON output")
            );
        } else {
            println!("Pending prompts: {}", pending);
        }
        return;
    }

    if let Some(mut content) = args.content {
        if let Some(context_path) = args.context {
            if let Ok(context_content) = std::fs::read_to_string(&context_path) {
                content = format!("{}\n{}", context_content, content);
            }
        }

        if args.shell {
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(&content)
                .output();
            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                if !stdout.is_empty() {
                    print!("{}", stdout);
                }
                if let Some(session_id) = args.session.as_deref() {
                    let shell_message = if stdout.is_empty() {
                        content.clone()
                    } else {
                        stdout
                    };
                    let _ = append_session_message(session_id, &shell_message);
                }
                return;
            }
        }

        if args.asynchronous {
            let mut queue = load_queue();
            queue.push(serde_json::json!({
                "session": args.session.clone().unwrap_or_default(),
                "content": content,
                "terminal": args.terminal,
                "multiline": args.multiline,
            }));
            save_queue(&queue);
            println!("Queued prompt");
            return;
        }

        if let Some(session_id) = args.session.as_deref() {
            let _ = append_session_message(session_id, &content);
        }
        println!("Prompt submitted");
        return;
    }

    println!(
        "Usage: opencode-rs prompt [--session ID] [--history] [--history-up] [--history-down]"
    );
}
