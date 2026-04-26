use clap::{Args, Subcommand};
use opencode_core::Config;
use std::env;

#[derive(Args, Debug)]
pub(crate) struct DebugArgs {
    #[command(subcommand)]
    pub action: DebugAction,
}

#[derive(Subcommand, Debug)]
pub(crate) enum DebugAction {
    Config,
    Lsp,
    Agent,
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_action_config() {
        let action = DebugAction::Config;
        let debug_str = format!("{:?}", action);
        assert!(debug_str.contains("Config"));
    }

    #[test]
    fn test_debug_action_lsp() {
        let action = DebugAction::Lsp;
        let debug_str = format!("{:?}", action);
        assert!(debug_str.contains("Lsp"));
    }

    #[test]
    fn test_debug_action_agent() {
        let action = DebugAction::Agent;
        let debug_str = format!("{:?}", action);
        assert!(debug_str.contains("Agent"));
    }

    #[test]
    fn test_debug_args_with_config() {
        let args = DebugArgs {
            action: DebugAction::Config,
        };
        match args.action {
            DebugAction::Config => {}
            _ => panic!("Expected Config variant"),
        }
    }

    #[test]
    fn test_debug_args_with_lsp() {
        let args = DebugArgs {
            action: DebugAction::Lsp,
        };
        match args.action {
            DebugAction::Lsp => {}
            _ => panic!("Expected Lsp variant"),
        }
    }

    #[test]
    fn test_debug_args_with_agent() {
        let args = DebugArgs {
            action: DebugAction::Agent,
        };
        match args.action {
            DebugAction::Agent => {}
            _ => panic!("Expected Agent variant"),
        }
    }
}

pub(crate) fn run(args: DebugArgs) {
    match args.action {
        DebugAction::Config => debug_config(),
        DebugAction::Lsp => debug_lsp(),
        DebugAction::Agent => debug_agent(),
    }
}

fn debug_config() {
    let config_path = Config::config_path();
    println!("Config Debug Info:");
    println!("  Config path: {:?}", config_path);

    if config_path.exists() {
        println!("  Config file exists: yes");
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            println!("  Config file size: {} bytes", content.len());
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                println!("  Config is valid JSON: yes");
                if json.get("model").is_some() {
                    println!("  Has model setting: yes");
                } else {
                    println!("  Has model setting: no");
                }
            } else {
                println!("  Config is valid JSON: no (may be TOML or other format)");
            }
        }
    } else {
        println!("  Config file exists: no");
    }

    let home_config = dirs::config_dir()
        .map(|p| p.join("opencode-rs").join("config.json"))
        .map(|p| p.to_string_lossy().to_string());
    if let Some(path) = home_config {
        println!("  Home config path: {}", path);
    }

    println!("  Environment variables checked:");
    let opencode_key = "OPENCODE_CONFIG";
    if env::var(opencode_key).is_ok() {
        println!("    {}: set", opencode_key);
    } else {
        println!("    {}: not set", opencode_key);
    }

    let config = Config::load(&config_path).unwrap_or_default();
    println!("  Loaded config model: {:?}", config.model);
    println!("  Loaded config agent: {:?}", config.agent);
}

fn debug_lsp() {
    println!("LSP Debug Info:");
    println!("  LSP is used for code intelligence");

    if let Ok(lsp_path) = which_lsp_server() {
        println!("  LSP server found: {}", lsp_path);
    } else {
        println!("  LSP server found: no (not in PATH)");
    }

    println!("  Supported LSP servers: rust-analyzer, clangd, pyright, typescript-language-server");
}

fn debug_agent() {
    println!("Agent Debug Info:");

    let config_path = Config::config_path();
    let config = Config::load(&config_path).unwrap_or_default();

    println!(
        "  Default agent: {:?}",
        config
            .default_agent
            .unwrap_or_else(|| "general".to_string())
    );
    println!(
        "  Default model: {:?}",
        config.model.unwrap_or_else(|| "gpt-4o".to_string())
    );

    let session_count = {
        use crate::cmd::session::get_session_sharing_for_quick as get_session_sharing;
        let sharing = get_session_sharing();
        sharing.list_sessions().map(|s| s.len()).unwrap_or(0)
    };
    println!("  Active sessions in memory: {}", session_count);
}

fn which_lsp_server() -> Result<String, String> {
    let servers = [
        "rust-analyzer",
        "clangd",
        "pyright",
        "typescript-language-server",
        "gopls",
    ];
    for server in servers {
        if which::which(server).is_ok() {
            return Ok(server.to_string());
        }
    }
    Err("No LSP server found".to_string())
}
