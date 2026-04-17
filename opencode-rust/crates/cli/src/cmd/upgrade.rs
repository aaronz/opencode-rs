use clap::Args;
use serde_json::json;

#[derive(Args, Debug)]
pub(crate) struct UpgradeArgs {
    #[arg(short, long)]
    pub version: Option<String>,

    #[arg(short, long)]
    pub force: bool,

    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub dry_run: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upgrade_args_default() {
        let args = UpgradeArgs {
            version: None,
            force: false,
            json: false,
            dry_run: false,
        };
        assert!(args.version.is_none());
        assert!(!args.force);
        assert!(!args.json);
        assert!(!args.dry_run);
    }

    #[test]
    fn test_upgrade_args_with_version() {
        let args = UpgradeArgs {
            version: Some("1.2.3".to_string()),
            force: false,
            json: false,
            dry_run: false,
        };
        assert_eq!(args.version.as_deref(), Some("1.2.3"));
    }

    #[test]
    fn test_upgrade_args_with_force() {
        let args = UpgradeArgs {
            version: None,
            force: true,
            json: false,
            dry_run: false,
        };
        assert!(args.force);
    }

    #[test]
    fn test_upgrade_args_with_json() {
        let args = UpgradeArgs {
            version: None,
            force: false,
            json: true,
            dry_run: false,
        };
        assert!(args.json);
    }

    #[test]
    fn test_upgrade_args_with_dry_run() {
        let args = UpgradeArgs {
            version: None,
            force: false,
            json: false,
            dry_run: true,
        };
        assert!(args.dry_run);
    }

    #[test]
    fn test_upgrade_args_full() {
        let args = UpgradeArgs {
            version: Some("2.0.0".to_string()),
            force: true,
            json: true,
            dry_run: true,
        };
        assert_eq!(args.version.as_deref(), Some("2.0.0"));
        assert!(args.force);
        assert!(args.json);
        assert!(args.dry_run);
    }
}

pub(crate) fn run(args: UpgradeArgs) {
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
