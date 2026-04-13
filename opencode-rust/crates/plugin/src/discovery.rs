use crate::config::validate_plugin_options;
use crate::PluginError;
use crate::{PluginCapability, PluginConfig, PluginPermissions};
use indexmap::IndexMap;
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DiscoveredPlugin {
    pub config: PluginConfig,
    pub metadata_path: PathBuf,
    pub library_path: PathBuf,
    pub description: String,
}

pub struct PluginDiscovery {
    global_dir: Option<PathBuf>,
    project_dir: Option<PathBuf>,
}

impl PluginDiscovery {
    pub fn new(project_path: Option<&Path>) -> Self {
        let global_dir = std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(".config/opencode/plugins"));

        let project_dir = project_path
            .map(|path| path.join(".opencode/plugins"))
            .or_else(|| Some(PathBuf::from(".opencode/plugins")));

        Self {
            global_dir,
            project_dir,
        }
    }

    pub fn with_dirs(global_dir: Option<PathBuf>, project_dir: Option<PathBuf>) -> Self {
        Self {
            global_dir,
            project_dir,
        }
    }

    pub fn discover(&self) -> Result<Vec<DiscoveredPlugin>, PluginError> {
        let mut by_name: IndexMap<String, DiscoveredPlugin> = IndexMap::new();

        if let Some(global_dir) = &self.global_dir {
            for plugin in self.discover_in_dir(global_dir)? {
                by_name.insert(plugin.config.name.clone(), plugin);
            }
        }

        if let Some(project_dir) = &self.project_dir {
            for plugin in self.discover_in_dir(project_dir)? {
                by_name.insert(plugin.config.name.clone(), plugin);
            }
        }

        let mut plugins: Vec<DiscoveredPlugin> = by_name.into_values().collect();
        plugins.sort_by(|a, b| a.config.name.cmp(&b.config.name));
        Ok(plugins)
    }

    fn discover_in_dir(&self, root: &Path) -> Result<Vec<DiscoveredPlugin>, PluginError> {
        let mut discovered = Vec::new();
        for metadata_path in find_metadata_files(root)? {
            discovered.push(parse_metadata_file(&metadata_path)?);
        }
        Ok(discovered)
    }
}

#[derive(Debug, Deserialize)]
struct PluginMetadata {
    name: String,
    version: String,
    description: String,
    main: String,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default)]
    priority: i32,
    #[serde(default)]
    options: IndexMap<String, Value>,
    #[serde(default)]
    capabilities: Vec<PluginCapability>,
    #[serde(default)]
    allowed_events: Vec<String>,
    #[serde(default)]
    filesystem_scope: Option<String>,
    #[serde(default)]
    network_allowed: bool,
}

fn default_true() -> bool {
    true
}

pub(crate) fn parse_metadata_file(path: &Path) -> Result<DiscoveredPlugin, PluginError> {
    let content = fs::read_to_string(path)?;
    let metadata: PluginMetadata = serde_json::from_str(&content)?;

    // Validate plugin options don't use reserved config keys
    let validation_result = validate_plugin_options(&metadata.name, &metadata.options);
    if !validation_result.valid {
        return Err(PluginError::ConfigValidation(
            metadata.name,
            validation_result.errors.join("; "),
        ));
    }

    let library_path = {
        let main = PathBuf::from(&metadata.main);
        if main.is_absolute() {
            main
        } else {
            path.parent().unwrap_or_else(|| Path::new(".")).join(main)
        }
    };

    Ok(DiscoveredPlugin {
        config: PluginConfig {
            name: metadata.name,
            version: metadata.version,
            enabled: metadata.enabled,
            priority: metadata.priority,
            domain: crate::PluginDomain::Runtime,
            options: metadata.options,
            permissions: PluginPermissions {
                capabilities: metadata.capabilities,
                allowed_events: metadata.allowed_events,
                filesystem_scope: metadata.filesystem_scope,
                network_allowed: metadata.network_allowed,
            },
        },
        metadata_path: path.to_path_buf(),
        library_path,
        description: metadata.description,
    })
}

fn find_metadata_files(root: &Path) -> Result<Vec<PathBuf>, PluginError> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut stack = vec![root.to_path_buf()];
    let mut files = Vec::new();

    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }

            if path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.ends_with(".plugin.json"))
                .unwrap_or(false)
            {
                files.push(path);
            }
        }
    }

    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plugin_metadata_json() {
        let temp = tempfile::tempdir().unwrap();
        let metadata_path = temp.path().join("demo.plugin.json");
        fs::write(
            &metadata_path,
            r#"{
                "name": "my-plugin",
                "version": "1.0.0",
                "description": "A sample plugin",
                "main": "my_plugin.so",
                "capabilities": ["ListenEvents", "AddTools"],
                "allowed_events": ["*"],
                "filesystem_scope": null,
                "network_allowed": false
            }"#,
        )
        .unwrap();

        let discovered = parse_metadata_file(&metadata_path).unwrap();
        assert_eq!(discovered.config.name, "my-plugin");
        assert_eq!(discovered.config.version, "1.0.0");
        assert_eq!(discovered.config.permissions.allowed_events, vec!["*"]);
        assert_eq!(discovered.library_path, temp.path().join("my_plugin.so"));
        assert_eq!(discovered.description, "A sample plugin");
    }

    #[test]
    fn project_plugins_override_global_plugins_by_name() {
        let temp = tempfile::tempdir().unwrap();
        let global_dir = temp.path().join("global");
        let project_dir = temp.path().join("project");
        fs::create_dir_all(&global_dir).unwrap();
        fs::create_dir_all(&project_dir).unwrap();

        fs::write(
            global_dir.join("demo.plugin.json"),
            r#"{
                "name": "demo",
                "version": "1.0.0",
                "description": "global",
                "main": "global.so",
                "capabilities": [],
                "allowed_events": [],
                "filesystem_scope": null,
                "network_allowed": false
            }"#,
        )
        .unwrap();

        fs::write(
            project_dir.join("demo.plugin.json"),
            r#"{
                "name": "demo",
                "version": "2.0.0",
                "description": "project",
                "main": "project.so",
                "capabilities": [],
                "allowed_events": [],
                "filesystem_scope": null,
                "network_allowed": false
            }"#,
        )
        .unwrap();

        let discovery =
            PluginDiscovery::with_dirs(Some(global_dir.clone()), Some(project_dir.clone()));
        let plugins = discovery.discover().unwrap();

        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].config.name, "demo");
        assert_eq!(plugins[0].config.version, "2.0.0");
        assert_eq!(plugins[0].library_path, project_dir.join("project.so"));
    }
}
