use crate::cmd::session::load_session_records;
use clap::{ArgAction, Args};
use serde_json::json;

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(short, long)]
    pub all: bool,

    #[arg(short, long)]
    pub limit: Option<u32>,

    #[arg(short, long, action = ArgAction::Count)]
    pub json: u8,
}

pub fn run(args: ListArgs) {
    let mut sessions = load_session_records()
        .into_iter()
        .map(|record| {
            json!({
                "id": record.id,
                "name": record.name,
                "created_at": record.created_at,
            })
        })
        .collect::<Vec<_>>();

    if let Some(limit) = args.limit {
        sessions.truncate(limit as usize);
    }

    if args.json > 0 {
        let result = json!({
            "action": "list",
            "sessions": sessions,
        });
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
        return;
    }

    println!(
        "Listing sessions, all: {}, limit: {:?}",
        args.all, args.limit
    );
    for session in sessions {
        println!("{}", session["id"].as_str().unwrap_or_default());
    }
}
