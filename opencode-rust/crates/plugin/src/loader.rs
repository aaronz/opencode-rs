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

    /// Loads a plugin from a dynamic library file.
    ///
    /// # Safety
    ///
    /// The library must be a valid plugin compiled with the same ABI version.
    /// The plugin must implement the `Plugin` trait correctly. Loading malformed
    /// plugins may cause undefined behavior.
    pub unsafe fn load_plugin<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<Box<dyn Plugin>, PluginError> {
        let lib = Library::new(path.as_ref())
            .map_err(|e| PluginError::Load(format!("Failed to load library: {}", e)))?;
        // SAFETY: The unsafe fn type is required because _create_plugin is an extern
        // function that returns a raw pointer. Callers must ensure the plugin library
        // is compiled with the same ABI version and returns a valid vtable pointer.
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

impl Drop for PluginLoader {
    fn drop(&mut self) {
        while let Some(lib) = self.libraries.pop() {
            if let Err(e) = lib.close() {
                tracing::warn!(error = %e, "Failed to close plugin library");
            }
        }
    }
}
