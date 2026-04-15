use clap::Args;
use serde_json::json;

#[derive(Args, Debug)]
pub struct UpgradeArgs {
    #[arg(short, long)]
    pub version: Option<String>,

    #[arg(short, long)]
    pub force: bool,

    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: UpgradeArgs) {
    let current_version = env!("CARGO_PKG_VERSION");
    let target_version = args.version.as_deref().unwrap_or("latest");

    if args.dry_run {
        if args.json {
            let result = json!({
                "action": "upgrade",
                "current_version": current_version,
                "target_version": target_version,
                "force": args.force,
                "dry_run": true,
                "status": "not_implemented"
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&result).expect("failed to serialize JSON output")
            );
            return;
        }
        println!("[DRY RUN] opencode-rs upgrade");
        println!("  Current version : {}", current_version);
        println!("  Target version  : {}", target_version);
        if args.force {
            println!("  Mode: force");
        }
        println!();
        println!("[DRY RUN] To upgrade, run:");
        println!("  cargo install opencode-rs");
        return;
    }

    if args.json {
        let result = json!({
            "action": "upgrade",
            "current_version": current_version,
            "target_version": target_version,
            "force": args.force,
            "status": "not_implemented"
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&result).expect("failed to serialize JSON output")
        );
        return;
    }

    println!("opencode-rs upgrade");
    println!("  Current version : {}", current_version);
    println!("  Target version  : {}", target_version);
    if args.force {
        println!("  Mode: force");
    }
    println!();
    println!("To upgrade, run:");
    println!("  cargo install opencode-rs");
}
