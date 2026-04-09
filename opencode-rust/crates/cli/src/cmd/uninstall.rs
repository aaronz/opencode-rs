use clap::Args;
use serde_json::json;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct UninstallArgs {
    #[arg(short, long)]
    pub force: bool,

    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: UninstallArgs) {
    let data_dir = directories::ProjectDirs::from("com", "opencode", "rs")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("~/.local/share/opencode-rs"));

    let config_dir = directories::ProjectDirs::from("com", "opencode", "rs")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("~/.config/opencode-rs"));

    if args.dry_run || (!args.force) {
        if args.json {
            let result = json!({
                "action": "uninstall",
                "force": args.force,
                "dry_run": true,
                "data_dir": data_dir,
                "config_dir": config_dir,
                "status": "dry_run"
            });
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
            return;
        }
        println!("[DRY RUN] Uninstalling opencode-rs");
        println!("The following directories will be removed:");
        println!("  - Data  : {}", data_dir.display());
        println!("  - Config: {}", config_dir.display());
        return;
    }

    if args.json {
        let result = json!({
            "action": "uninstall",
            "force": args.force,
            "data_dir": data_dir,
            "config_dir": config_dir,
            "status": if args.force { "executed" } else { "dry_run" }
        });
        println!("{}", serde_json::to_string_pretty(&result).unwrap());

        if args.force {
            if data_dir.exists() {
                let _ = std::fs::remove_dir_all(&data_dir);
            }
            if config_dir.exists() {
                let _ = std::fs::remove_dir_all(&config_dir);
            }
        }
        return;
    }

    println!("Uninstalling opencode-rs");
    println!("The following directories will be removed:");
    println!("  - Data  : {}", data_dir.display());
    println!("  - Config: {}", config_dir.display());

    if args.force {
        println!("\nExecuting removal...");
        if data_dir.exists() {
            let _ = std::fs::remove_dir_all(&data_dir);
            println!("Removed data directory.");
        }
        if config_dir.exists() {
            let _ = std::fs::remove_dir_all(&config_dir);
            println!("Removed config directory.");
        }
        println!("Uninstallation complete.");
    } else {
        println!("\nRun with --force to actually remove these files.");
    }
}
