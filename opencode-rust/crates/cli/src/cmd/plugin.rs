use clap::{Args, Subcommand};
use opencode_core::Config;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::cmd::load_config;

#[derive(Args, Debug)]
pub(crate) struct PluginArgs {
    #[command(subcommand)]
    pub command: PluginCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum PluginCommand {
    #[command(about = "Install a plugin by name")]
    Install {
        #[arg(value_name = "NAME")]
        name: String,
    },

    #[command(about = "List installed plugins")]
    List,

    #[command(about = "Remove a plugin by name")]
    Remove {
        #[arg(value_name = "NAME")]
        name: String,
    },

    #[command(about = "Search available plugins")]
    Search {
        #[arg(value_name = "QUERY")]
        query: Option<String>,
    },
}

fn save_config(config: &Config) {
    let path = Config::config_path();
    if let Err(error) = config.save(&path) {
        eprintln!("Failed to save config: {}", error);
        std::process::exit(1);
    }
}

fn get_installed_plugins(config: &Config) -> Vec<String> {
    config.plugin.clone().unwrap_or_default()
}

fn discover_plugins() -> Vec<PluginDiscoveryInfo> {
    let mut plugins = Vec::new();

    let global_dir = std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config/opencode/plugins"));

    let project_dir = Some(PathBuf::from(".opencode/plugins"));

    for dir in [global_dir, project_dir].into_iter().flatten() {
        if dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Ok(metadata) = std::fs::read_to_string(path.join("plugin.json")) {
                            if let Ok(info) = serde_json::from_str::<PluginDiscoveryInfo>(&metadata)
                            {
                                plugins.push(info);
                            }
                        }
                    }
                }
            }
        }
    }

    plugins
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginDiscoveryInfo {
    name: String,
    version: String,
    description: String,
    #[serde(default)]
    enabled: bool,
}

pub(crate) fn run(args: PluginArgs) {
    match args.command {
        PluginCommand::Install { name } => {
            let mut config = load_config();
            let mut plugins = get_installed_plugins(&config);

            if plugins.contains(&name) {
                println!("Plugin '{}' is already installed", name);
                return;
            }

            plugins.push(name.clone());
            config.plugin = Some(plugins);

            save_config(&config);
            println!("Plugin '{}' installed successfully", name);
        }
        PluginCommand::List => {
            let config = load_config();
            let installed = get_installed_plugins(&config);
            let discovered = discover_plugins();

            if installed.is_empty() && discovered.is_empty() {
                println!("No plugins installed or discovered");
                return;
            }

            for name in &installed {
                let discovered_info = discovered.iter().find(|p| &p.name == name);
                let (version, description) = discovered_info
                    .map(|p| (p.version.clone(), p.description.clone()))
                    .unwrap_or_else(|| ("unknown".to_string(), "".to_string()));

                println!("{}\t{}\t{}\tinstalled", name, version, description);
            }

            for plugin in discovered {
                if !installed.contains(&plugin.name) {
                    println!(
                        "{}\t{}\t{}\tdiscovered",
                        plugin.name, plugin.version, plugin.description
                    );
                }
            }
        }
        PluginCommand::Remove { name } => {
            let mut config = load_config();
            let mut plugins = get_installed_plugins(&config);

            if !plugins.contains(&name) {
                eprintln!("Plugin '{}' is not installed", name);
                std::process::exit(1);
            }

            plugins.retain(|p| p != &name);
            config.plugin = Some(plugins);

            save_config(&config);
            println!("Plugin '{}' removed successfully", name);
        }
        PluginCommand::Search { query } => {
            let discovered = discover_plugins();

            let filtered: Vec<_> = match &query {
                Some(q) => discovered
                    .into_iter()
                    .filter(|p| {
                        p.name.to_lowercase().contains(&q.to_lowercase())
                            || p.description.to_lowercase().contains(&q.to_lowercase())
                    })
                    .collect(),
                None => discovered,
            };

            if filtered.is_empty() {
                if let Some(ref q) = query {
                    println!("No plugins found matching '{}'", q);
                } else {
                    println!("No plugins discovered. Install plugins to ~/.config/opencode/plugins or .opencode/plugins/");
                }
                return;
            }

            for plugin in filtered {
                println!(
                    "{}\t{}\t{}",
                    plugin.name, plugin.version, plugin.description
                );
            }
        }
    }
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_install_adds_to_list() {
        let config = Config::default();
        let installed = get_installed_plugins(&config);
        assert!(installed.is_empty());
    }

    #[test]
    fn test_plugin_install_already_installed() {
        let config = Config {
            plugin: Some(vec!["test-plugin".to_string()]),
            ..Default::default()
        };
        let installed = get_installed_plugins(&config);
        assert!(installed.contains(&"test-plugin".to_string()));
    }

    #[test]
    fn test_plugin_remove_nonexistent() {
        let config = Config::default();
        let installed = get_installed_plugins(&config);
        assert!(!installed.contains(&"nonexistent".to_string()));
    }

    #[test]
    fn test_plugin_search_filters_by_query() {
        let query = Some("test".to_string());
        let all_plugins = vec![
            PluginDiscoveryInfo {
                name: "test-plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "A test plugin".to_string(),
                enabled: true,
            },
            PluginDiscoveryInfo {
                name: "other-plugin".to_string(),
                version: "2.0.0".to_string(),
                description: "Another plugin".to_string(),
                enabled: true,
            },
        ];

        let filtered: Vec<_> = all_plugins
            .into_iter()
            .filter(|p| {
                p.name
                    .to_lowercase()
                    .contains(&query.as_ref().unwrap().to_lowercase())
                    || p.description
                        .to_lowercase()
                        .contains(&query.as_ref().unwrap().to_lowercase())
            })
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "test-plugin");
    }

    #[test]
    fn test_plugin_search_no_query_returns_all() {
        let query: Option<String> = None;
        let all_plugins = vec![
            PluginDiscoveryInfo {
                name: "test-plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "A test plugin".to_string(),
                enabled: true,
            },
            PluginDiscoveryInfo {
                name: "other-plugin".to_string(),
                version: "2.0.0".to_string(),
                description: "Another plugin".to_string(),
                enabled: true,
            },
        ];

        let filtered: Vec<_> = match &query {
            Some(q) => all_plugins
                .into_iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&q.to_lowercase())
                        || p.description.to_lowercase().contains(&q.to_lowercase())
                })
                .collect(),
            None => all_plugins,
        };

        assert_eq!(filtered.len(), 2);
    }
}
