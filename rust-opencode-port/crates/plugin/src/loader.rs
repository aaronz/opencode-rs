use crate::{Plugin, PluginError};
use libloading::{Library, Symbol};
use std::path::Path;

pub struct PluginLoader {
    libraries: Vec<Library>,
}

impl PluginLoader {
    pub fn new() -> Self {
        Self {
            libraries: Vec::new(),
        }
    }

    pub unsafe fn load_plugin<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<Box<dyn Plugin>, PluginError> {
        let lib = Library::new(path.as_ref())
            .map_err(|e| PluginError::Load(format!("Failed to load library: {}", e)))?;

        type PluginCreate = unsafe fn() -> *mut dyn Plugin;
        let constructor: Symbol<PluginCreate> = lib
            .get(b"_create_plugin")
            .map_err(|e| PluginError::Load(format!("Failed to get plugin constructor: {}", e)))?;

        let plugin_ptr = constructor();
        let plugin = Box::from_raw(plugin_ptr);

        self.libraries.push(lib);
        Ok(plugin)
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}
