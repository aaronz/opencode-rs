use std::path::Path;
use wasmtime::{Engine, Instance, Module, Store};

#[derive(Debug, thiserror::Error)]
pub enum WasmError {
    #[error("WASM compilation failed: {0}")]
    Compile(String),
    #[error("WASM instantiation failed: {0}")]
    Instantiate(String),
    #[error("WASM function call failed: {0}")]
    Call(String),
    #[error("WASM timeout: {0}")]
    Timeout(String),
    #[error("WASM sandbox violation: {0}")]
    Sandbox(String),
    #[error("WASM IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Clone)]
pub struct WasmCapabilities {
    pub filesystem_scope: Option<String>,
    pub network_allowed: bool,
    pub allowed_env_vars: Vec<String>,
    pub execution_timeout_secs: Option<u64>,
}

impl Default for WasmCapabilities {
    fn default() -> Self {
        Self {
            filesystem_scope: None,
            network_allowed: false,
            allowed_env_vars: Vec::new(),
            execution_timeout_secs: Some(30),
        }
    }
}

pub struct WasmRuntime {
    engine: Engine,
    capabilities: WasmCapabilities,
}

impl WasmRuntime {
    pub fn new(capabilities: WasmCapabilities) -> Result<Self, WasmError> {
        let engine = Engine::default();
        Ok(Self {
            engine,
            capabilities,
        })
    }

    pub fn load_module(&self, path: &Path) -> Result<Module, WasmError> {
        let bytes = std::fs::read(path)?;
        Module::new(&self.engine, bytes).map_err(|e| WasmError::Compile(e.to_string()))
    }

    pub fn instantiate_module(&self, module: &Module) -> Result<WasmInstance, WasmError> {
        let mut store = Store::new(&self.engine, ());

        let instance = Instance::new(&mut store, module, &[])
            .map_err(|e| WasmError::Instantiate(e.to_string()))?;

        Ok(WasmInstance { store, instance })
    }

    pub fn capabilities(&self) -> &WasmCapabilities {
        &self.capabilities
    }
}

pub struct WasmInstance {
    store: Store<()>,
    instance: Instance,
}

impl WasmInstance {
    pub fn call(&mut self, func_name: &str) -> Result<(), WasmError> {
        let func = self
            .instance
            .get_func(&mut self.store, func_name)
            .ok_or_else(|| WasmError::Instantiate(format!("function not found: {}", func_name)))?;

        func.typed::<(), ()>(&mut self.store)
            .map_err(|e| WasmError::Call(e.to_string()))?;

        Ok(())
    }
}

pub struct WasmPlugin {
    name: String,
    version: String,
    runtime: WasmRuntime,
    instance: Option<WasmInstance>,
}

impl WasmPlugin {
    pub fn new(
        name: String,
        version: String,
        capabilities: WasmCapabilities,
    ) -> Result<Self, WasmError> {
        let runtime = WasmRuntime::new(capabilities)?;
        Ok(Self {
            name,
            version,
            runtime,
            instance: None,
        })
    }

    pub fn load(&mut self, module_path: &Path) -> Result<(), WasmError> {
        let module = self.runtime.load_module(module_path)?;
        let instance = self.runtime.instantiate_module(&module)?;
        self.instance = Some(instance);
        Ok(())
    }

    pub fn execute(&mut self, func_name: &str) -> Result<(), WasmError> {
        if let Some(ref mut instance) = self.instance {
            instance.call(func_name)?;
        }
        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_runtime_creation() {
        let caps = WasmCapabilities::default();
        let runtime = WasmRuntime::new(caps);
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_wasm_capabilities_default() {
        let caps = WasmCapabilities::default();
        assert!(!caps.network_allowed);
        assert!(caps.execution_timeout_secs.is_some());
    }
}
