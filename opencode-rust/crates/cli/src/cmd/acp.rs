use clap::Args;
use serde_json::json;

#[derive(Args, Debug)]
pub struct AcpArgs {
    #[arg(short, long)]
    pub url: Option<String>,

    #[arg(long)]
    pub json: bool,

    #[arg(long, default_value = "start")]
    pub action: String,
}

pub fn run(args: AcpArgs) {
    if args.json {
        let result = json!({
            "component": "acp",
            "action": args.action,
            "url": args.url,
            "status": "ready"
        });
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
        return;
    }

    println!("ACP Protocol Manager");
    println!("  Action: {}", args.action);
    if let Some(url) = &args.url {
        println!("  Target URL: {}", url);
    }
    println!("  Status: Ready");
}
