use crate::cmd::session::load_session_records;
use clap::{ArgAction, Args};
use serde_json::json;

#[derive(Args, Debug)]
pub(crate) struct ListArgs {
    #[arg(short, long)]
    pub all: bool,

    #[arg(short, long)]
    pub limit: Option<u32>,

    #[arg(short, long, action = ArgAction::Count)]
    pub json: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_args_default() {
        let args = ListArgs {
            all: false,
            limit: None,
            json: 0,
        };
        assert!(!args.all);
        assert!(args.limit.is_none());
        assert_eq!(args.json, 0);
    }

    #[test]
    fn test_list_args_with_all() {
        let args = ListArgs {
            all: true,
            limit: None,
            json: 0,
        };
        assert!(args.all);
    }

    #[test]
    fn test_list_args_with_limit() {
        let args = ListArgs {
            all: false,
            limit: Some(10),
            json: 0,
        };
        assert_eq!(args.limit, Some(10));
    }

    #[test]
    fn test_list_args_with_json() {
        let args = ListArgs {
            all: false,
            limit: None,
            json: 1,
        };
        assert_eq!(args.json, 1);
    }

    #[test]
    fn test_list_args_with_all_and_limit() {
        let args = ListArgs {
            all: true,
            limit: Some(5),
            json: 0,
        };
        assert!(args.all);
        assert_eq!(args.limit, Some(5));
    }

    #[test]
    fn test_list_args_with_json_and_limit() {
        let args = ListArgs {
            all: false,
            limit: Some(20),
            json: 2,
        };
        assert_eq!(args.limit, Some(20));
        assert_eq!(args.json, 2);
    }
}

pub(crate) fn run(args: ListArgs) {
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
        println!(
            "{}",
            serde_json::to_string_pretty(&result).expect("failed to serialize JSON output")
        );
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
