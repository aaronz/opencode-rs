use clap::Args;
use serde_json::json;

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(short, long)]
    pub all: bool,

    #[arg(short, long)]
    pub limit: Option<u32>,

    #[arg(short, long)]
    pub json: bool,
}

pub fn run(args: ListArgs) {
    if args.json {
        let result = json!({
            "action": "list",
            "sessions": []
        });
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
        return;
    }

    println!(
        "Listing sessions, all: {}, limit: {:?}",
        args.all, args.limit
    );
}
