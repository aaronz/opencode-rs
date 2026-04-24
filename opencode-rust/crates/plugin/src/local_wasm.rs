use opencode_core::OpenCodeError;
use std::fmt;
use std::path::Path;
use wasmi::{ImportsBuilder, Module, ModuleInstance, ModuleRef, NopExternals};

pub struct LocalWasmPlugin {
    name: String,
    version: String,
    module: Option<Module>,
    instance: Option<ModuleRef>,
}

impl LocalWasmPlugin {
    pub fn new(name: String, version: String) -> Result<Self, OpenCodeError> {
        Ok(Self {
            name,
            version,
            module: None,
            instance: None,
        })
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), OpenCodeError> {
        let wasm_bytes = std::fs::read(path.as_ref()).map_err(OpenCodeError::Io)?;
        self.load_bytes(&wasm_bytes)
    }

    pub fn load_bytes(&mut self, wasm_bytes: &[u8]) -> Result<(), OpenCodeError> {
        let module = Module::from_buffer(wasm_bytes)
            .map_err(|e| OpenCodeError::Config(format!("Failed to compile WASM module: {}", e)))?;

        let instance = ModuleInstance::new(&module, &ImportsBuilder::default())
            .map_err(|e| OpenCodeError::Config(format!("Failed to instantiate WASM: {}", e)))?;

        let module_ref = instance.assert_no_start();
        self.module = Some(module);
        self.instance = Some(module_ref);

        Ok(())
    }

    pub fn execute(&mut self, func_name: &str) -> Result<(), OpenCodeError> {
        let instance = self
            .instance
            .as_mut()
            .ok_or_else(|| OpenCodeError::Config("Plugin not loaded".to_string()))?;

        instance
            .invoke_export(func_name, &[], &mut NopExternals)
            .map_err(|e| OpenCodeError::Tool(format!("Function call failed: {}", e)))?;

        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn is_loaded(&self) -> bool {
        self.instance.is_some()
    }
}

impl fmt::Debug for LocalWasmPlugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalWasmPlugin")
            .field("name", &self.name)
            .field("version", &self.version)
            .field("instance", &self.instance.is_some())
            .finish()
    }
}

pub fn load_plugin<P: AsRef<Path>>(path: P) -> Result<LocalWasmPlugin, OpenCodeError> {
    let path = path.as_ref();
    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let mut plugin = LocalWasmPlugin::new(file_name.to_string(), "1.0.0".to_string())?;
    plugin.load(path)?;
    Ok(plugin)
}

pub fn load_plugin_with_version<P: AsRef<Path>>(
    path: P,
    name: String,
    version: String,
) -> Result<LocalWasmPlugin, OpenCodeError> {
    let mut plugin = LocalWasmPlugin::new(name, version)?;
    plugin.load(path)?;
    Ok(plugin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_plugin_with_invalid_wasm_handles_gracefully() {
        let mut temp_file = NamedTempFile::with_suffix(".wasm").unwrap();
        temp_file.write_all(b"not valid wasm").unwrap();
        let _ = temp_file.flush();

        let result = load_plugin(temp_file.path());
        assert!(result.is_err(), "Loading invalid WASM should fail");

        let err = result.unwrap_err();
        let err_str = format!("{}", err);
        assert!(
            err_str.contains("compile") || err_str.contains("validation"),
            "Error should indicate compilation failure, got: {}",
            err_str
        );
    }

    #[test]
    fn test_load_plugin_with_non_existent_file_fails() {
        let result = load_plugin("/non/existent/path.wasm");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            matches!(err, OpenCodeError::Io(_)),
            "Error should be an IO error for non-existent file, got: {}",
            err
        );
    }

    #[test]
    fn test_local_wasm_plugin_creation() {
        let plugin = LocalWasmPlugin::new("test".to_string(), "1.0.0".to_string());
        assert!(plugin.is_ok());

        let plugin = plugin.unwrap();
        assert_eq!(plugin.name(), "test");
        assert_eq!(plugin.version(), "1.0.0");
        assert!(!plugin.is_loaded());
    }

    #[test]
    fn test_local_wasm_plugin_execute_before_load() {
        let mut plugin = LocalWasmPlugin::new("test".to_string(), "1.0.0".to_string()).unwrap();
        assert!(!plugin.is_loaded());

        let result = plugin.execute("some_func");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            format!("{}", err).contains("not loaded"),
            "Error should indicate plugin not loaded, got: {}",
            err
        );
    }

    #[test]
    fn test_load_valid_wasm_file() {
        let wat_code = r#"
            (module
                (func (export "greet") (result i32)
                    i32.const 42
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat_code).expect("failed to parse WAT");

        let mut temp_file = NamedTempFile::with_suffix(".wasm").unwrap();
        temp_file.write_all(&wasm_bytes).unwrap();
        let _ = temp_file.flush();

        let result = load_plugin(temp_file.path());
        assert!(result.is_ok(), "Loading valid WASM should succeed");

        let plugin = result.unwrap();
        assert!(plugin.is_loaded());
        assert!(
            plugin.name().contains("tmp"),
            "plugin name should be derived from temp file"
        );
    }

    #[test]
    fn test_plugin_execute_on_loaded_plugin() {
        let wat_code = r#"
            (module
                (func (export "greet") (result i32)
                    i32.const 42
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat_code).expect("failed to parse WAT");

        let mut temp_file = NamedTempFile::with_suffix(".wasm").unwrap();
        temp_file.write_all(&wasm_bytes).unwrap();
        let _ = temp_file.flush();

        let mut plugin = load_plugin(temp_file.path()).expect("failed to load plugin");
        assert!(plugin.is_loaded());

        let result = plugin.execute("greet");
        assert!(result.is_ok(), "execute should succeed for valid function");
    }

    #[test]
    fn test_plugin_execute_unknown_function() {
        let wat_code = r#"
            (module
                (func (export "known") (result i32)
                    i32.const 1
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat_code).expect("failed to parse WAT");

        let mut temp_file = NamedTempFile::with_suffix(".wasm").unwrap();
        temp_file.write_all(&wasm_bytes).unwrap();
        let _ = temp_file.flush();

        let mut plugin = load_plugin(temp_file.path()).expect("failed to load plugin");

        let result = plugin.execute("unknown_function");
        assert!(result.is_err(), "calling unknown function should fail");
        let err = result.unwrap_err();
        assert!(
            format!("{}", err).contains("Function call failed"),
            "Error should indicate function call failure, got: {}",
            err
        );
    }
}
