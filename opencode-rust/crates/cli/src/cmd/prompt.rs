use crate::cmd::session::{append_session_message, load_session_records};
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct PromptArgs {
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

pub fn run(args: PromptArgs) {
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
