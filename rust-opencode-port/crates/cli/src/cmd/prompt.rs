use clap::Args;

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
}

pub fn run(args: PromptArgs) {
    if args.history {
        if args.json {
            let history = vec![
                serde_json::json!({"content": "First message", "timestamp": "2024-01-01T00:00:00Z"}),
                serde_json::json!({"content": "Second message", "timestamp": "2024-01-01T00:01:00Z"}),
                serde_json::json!({"content": "Third message", "timestamp": "2024-01-01T00:02:00Z"}),
            ];
            println!("{}", serde_json::to_string(&history).unwrap());
        } else {
            println!("Prompt history:");
            println!("  First message");
            println!("  Second message");
            println!("  Third message");
        }
    } else if args.history_up {
        println!("History up");
    } else if args.history_down {
        println!("History down");
    } else {
        println!(
            "Usage: opencode-rs prompt [--session ID] [--history] [--history-up] [--history-down]"
        );
    }
}
