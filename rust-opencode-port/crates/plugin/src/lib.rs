use libloading::{Library, Symbol};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("plugin already registered: {0}")]
    DuplicatePlugin(String),
    #[error("plugin not found: {0}")]
    NotFound(String),
    #[error("plugin IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("plugin metadata parse error: {0}")]
    MetadataParse(#[from] serde_json::Error),
    #[error("plugin load error: {0}")]
    Load(String),
    #[error("plugin init failed ({0}): {1}")]
    Init(String, String),
    #[error("plugin shutdown failed ({0}): {1}")]
    Shutdown(String, String),
}

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn init(&mut self) -> Result<(), PluginError>;
    fn shutdown(&mut self) -> Result<(), PluginError>;
    fn description(&self) -> &str;
}

#[derive(Debug, Deserialize)]
struct PluginMetadata {
    name: String,
    version: String,
    description: String,
    #[serde(default)]
    library_path: Option<PathBuf>,
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    libraries: Vec<Library>,
    discovered_metadata: Vec<PathBuf>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            libraries: Vec::new(),
            discovered_metadata: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<(), PluginError> {
        let key = plugin.name().to_string();
        if self.plugins.contains_key(&key) {
            return Err(PluginError::DuplicatePlugin(key));
        }
        self.plugins.insert(key, plugin);
        Ok(())
    }

    pub fn discover_default_dirs(&mut self) -> Result<(), PluginError> {
        let mut paths = Vec::new();
        if let Some(home) = std::env::var_os("HOME") {
            paths.push(PathBuf::from(home).join(".config/opencode/plugins"));
        }
        paths.push(PathBuf::from(".opencode/plugins"));
        self.discover_from_dirs(&paths)
    }

    pub fn discover_from_dirs(&mut self, paths: &[PathBuf]) -> Result<(), PluginError> {
        for path in paths {
            for metadata_path in find_metadata_files(path)? {
                self.discovered_metadata.push(metadata_path.clone());
                let metadata = read_metadata(&metadata_path)?;
                let _ = (&metadata.name, &metadata.version, &metadata.description);
                if let Some(lib_path) = metadata.library_path {
                    let absolute = if lib_path.is_absolute() {
                        lib_path
                    } else {
                        metadata_path
                            .parent()
                            .map(|p| p.join(lib_path))
                            .unwrap_or_default()
                    };

                    let plugin = unsafe { self.load_plugin_library(&absolute)? };
                    self.register(plugin)?;
                }
            }
        }

        Ok(())
    }

    pub fn init_all(&mut self) -> Result<(), PluginError> {
        for plugin in self.plugins.values_mut() {
            plugin
                .init()
                .map_err(|e| PluginError::Init(plugin.name().to_string(), e.to_string()))?;
        }
        Ok(())
    }

    pub fn shutdown_all(&mut self) -> Result<(), PluginError> {
        for plugin in self.plugins.values_mut() {
            plugin
                .shutdown()
                .map_err(|e| PluginError::Shutdown(plugin.name().to_string(), e.to_string()))?;
        }
        Ok(())
    }

    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    pub fn discovered_metadata(&self) -> &[PathBuf] {
        &self.discovered_metadata
    }

    unsafe fn load_plugin_library(&mut self, path: &Path) -> Result<Box<dyn Plugin>, PluginError> {
        let lib = Library::new(path).map_err(|e| PluginError::Load(e.to_string()))?;

        type PluginCreate = unsafe fn() -> *mut dyn Plugin;
        let constructor: Symbol<PluginCreate> = lib
            .get(b"_create_plugin")
            .map_err(|e| PluginError::Load(e.to_string()))?;

        let plugin_ptr = constructor();
        let plugin = Box::from_raw(plugin_ptr);
        self.libraries.push(lib);
        Ok(plugin)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

fn read_metadata(path: &Path) -> Result<PluginMetadata, PluginError> {
    let content = fs::read_to_string(path)?;
    let metadata = serde_json::from_str::<PluginMetadata>(&content)?;
    Ok(metadata)
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

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        initialized: bool,
        shutdown: bool,
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            "test-plugin"
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn init(&mut self) -> Result<(), PluginError> {
            self.initialized = true;
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), PluginError> {
            self.shutdown = true;
            Ok(())
        }

        fn description(&self) -> &str {
            "test plugin"
        }
    }

    #[test]
    fn test_register_and_get_plugin() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown: false,
            }))
            .unwrap();

        assert!(manager.get_plugin("test-plugin").is_some());
    }

    #[test]
    fn test_init_and_shutdown_lifecycle() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin {
                initialized: false,
                shutdown: false,
            }))
            .unwrap();

        manager.init_all().unwrap();
        manager.shutdown_all().unwrap();
    }

    #[test]
    fn test_discover_metadata_files() {
        let temp = tempfile::tempdir().unwrap();
        let plugin_dir = temp.path().join("plugins");
        fs::create_dir_all(&plugin_dir).unwrap();

        let metadata = plugin_dir.join("demo.plugin.json");
        fs::write(
            &metadata,
            r#"{"name":"demo","version":"1.0.0","description":"demo plugin"}"#,
        )
        .unwrap();

        let files = find_metadata_files(&plugin_dir).unwrap();
        assert_eq!(files.len(), 1);

        let mut manager = PluginManager::new();
        manager
            .discover_from_dirs(&[plugin_dir])
            .expect("metadata discovery should succeed without loading libraries");
        assert_eq!(manager.discovered_metadata().len(), 1);
    }
}
