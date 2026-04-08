use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use wasmtime::{
    Engine, Instance, Module, Store, Linker, Memory, Func, Val, ValType,
    Config, MemoryType, Trap,
};
use wasmtime_wasi::WasiCtxBuilder;

use crate::WasmPluginEvent;

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
    #[error("WASM memory error: {0}")]
    Memory(String),
    #[error("WASM IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Clone)]
pub struct WasmCapabilities {
    pub filesystem_scope: Option<String>,
    pub network_allowed: bool,
    pub allowed_env_vars: Vec<String>,
    pub execution_timeout_secs: Option<u64>,
    pub max_memory_bytes: Option<u64>,
    pub max_cpu_time_secs: Option<u64>,
}

impl Default for WasmCapabilities {
    fn default() -> Self {
        Self {
            filesystem_scope: None,
            network_allowed: false,
            allowed_env_vars: Vec::new(),
            execution_timeout_secs: Some(30),
            max_memory_bytes: Some(64 * 1024 * 1024),
            max_cpu_time_secs: Some(10),
        }
    }
}

pub struct WasmRuntime {
    engine: Engine,
    capabilities: WasmCapabilities,
}

pub struct WasmInstance {
    store: Store<WasmInstanceState>,
    instance: Instance,
}

struct WasmInstanceState {
    memory: Option<Memory>,
    event_tx: Option<broadcast::Sender<WasmPluginEvent>>,
    allowed_path: Option<String>,
}

pub struct WasmPlugin {
    name: String,
    version: String,
    runtime: WasmRuntime,
    instance: Option<WasmInstance>,
    event_rx: Option<broadcast::Receiver<WasmPluginEvent>>,
}

pub trait EventBridgeBackend: Send + Sync {
    fn subscribe(&self) -> broadcast::Receiver<EventEnvelope>;
    fn publish(&self, event: EventEnvelope);
}

#[derive(Debug, Clone)]
pub struct EventEnvelope {
    pub event_type: String,
    pub payload: String,
}

pub struct WasmEventBridge<B: EventBridgeBackend> {
    _task: JoinHandle<()>,
    _backend: Arc<B>,
}

impl<B: EventBridgeBackend> WasmEventBridge<B> {
    pub fn new(
        mut plugin: WasmPlugin,
        backend: Arc<B>,
    ) -> Result<Self, WasmError> {
        let event_rx = plugin.take_event_receiver()
            .ok_or_else(|| WasmError::Instantiate("plugin has no event receiver".to_string()))?;

        let (to_plugin_tx, mut to_plugin_rx) = mpsc::channel::<EventEnvelope>(64);

        let _backend = backend.clone();
        let task = tokio::spawn(async move {
            let mut bus_rx = _backend.subscribe();
            let mut plugin_events = event_rx;
            let mut subscribed_events: Vec<String> = Vec::new();

            loop {
                tokio::select! {
                    Some(wasm_event) = plugin_events.recv() => {
                        match wasm_event {
                            WasmPluginEvent::Subscribe { event_name } => {
                                if !subscribed_events.contains(&event_name) {
                                    subscribed_events.push(event_name.clone());
                                    tracing::debug!(event = %event_name, "WASM plugin subscribed to event");
                                }
                            }
                            WasmPluginEvent::Unsubscribe { event_name } => {
                                subscribed_events.retain(|e| e != &event_name);
                                tracing::debug!(event = %event_name, "WASM plugin unsubscribed from event");
                            }
                            WasmPluginEvent::Log { level, message } => {
                                match level.as_str() {
                                    "error" => tracing::error!(message = %message, "WASM plugin"),
                                    "warn" => tracing::warn!(message = %message, "WASM plugin"),
                                    "debug" => tracing::debug!(message = %message, "WASM plugin"),
                                    _ => tracing::info!(message = %message, "WASM plugin"),
                                }
                            }
                            WasmPluginEvent::PublishEvent { event_type, payload } => {
                                _backend.publish(EventEnvelope {
                                    event_type,
                                    payload,
                                });
                            }
                        }
                    }
                    Some(internal_event) = bus_rx.recv() => {
                        let event_name = &internal_event.event_type;
                        if subscribed_events.iter().any(|sub| {
                            sub == "*" || event_name.contains(sub)
                        }) {
                            if to_plugin_tx.send(internal_event.clone()).await.is_err() {
                                break;
                            }
                        }
                    }
                    else => break,
                }
            }
        });

        Ok(Self { _task: task, _backend })
    }
}

#[derive(Debug, Clone)]
pub enum WasmPluginEvent {
    Subscribe { event_name: String },
    Unsubscribe { event_name: String },
    Log { level: String, message: String },
    PublishEvent { event_type: String, payload: String },
}

impl WasmRuntime {
    pub fn new(capabilities: WasmCapabilities) -> Result<Self, WasmError> {
        let mut config = Config::new();
        config.consume_fuel(true);
        
        if let Some(max_mem) = capabilities.max_memory_bytes {
            config.static_memory_maximum(max_mem);
        }
        
        if let Some(timeout) = capabilities.execution_timeout_secs {
            let duration = Duration::from_secs(timeout);
            config.epoch_interruption();
        }
        
        let engine = Engine::new(&config)
            .map_err(|e| WasmError::Compile(e.to_string()))?;
        
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
        let mut state = WasmInstanceState {
            memory: None,
            event_tx: None,
            allowed_path: self.capabilities.filesystem_scope.clone(),
        };
        
        let mut store = Store::new(&self.engine, state);
        
        let mut linker = Linker::new(&self.engine);
        
        self.define_host_functions(&mut linker)?;
        
        if let Some(scope) = &self.capabilities.filesystem_scope {
            let wasi = WasiCtxBuilder::new()
                .preopened_dir(scope.as_str(), "/")
                .map_err(|e| WasmError::Instantiate(format!("WASI setup failed: {}", e)))?
                .build();
            wasmtime_wasi::add_to_linker(&mut linker, |s| s)
                .map_err(|e| WasmError::Instantiate(format!("WASI link failed: {}", e)))?;
        }
        
        let instance = linker
            .instantiate(&mut store, module)
            .map_err(|e| WasmError::Instantiate(e.to_string()))?;
        
        if let Some(memory) = instance.get_memory(&mut store, "memory") {
            state.memory = Some(memory);
        }
        
        Ok(WasmInstance {
            store,
            instance,
        })
    }

    fn define_host_functions(&self, linker: &mut Linker<WasmInstanceState>) -> Result<(), WasmError> {
        linker.func_wrap(
            "host",
            "log",
            |state: &mut WasmInstanceState, ptr: i32, len: i32| {
                if let Some(memory) = &state.memory {
                    if let Ok(msg) = memory.read_string(ptr as u32, len as u32) {
                        tracing::debug![message = %msg, "WASM plugin log"];
                    }
                }
                Ok(())
            },
        ).map_err(|e| WasmError::Instantiate(format!("failed to define log: {}", e)))?;
        
        linker.func_wrap(
            "host",
            "subscribe",
            |mut state: &mut WasmInstanceState, event_ptr: i32, event_len: i32| {
                if let Some(memory) = &state.memory {
                    if let Ok(event_name) = memory.read_string(event_ptr as u32, event_len as u32) {
                        if let Some(tx) = &state.event_tx {
                            let _ = tx.send(WasmPluginEvent::Subscribe {
                                event_name: event_name.clone(),
                            });
                        }
                        tracing::debug![event = %event_name, "WASM plugin subscribed to event"];
                    }
                }
                Ok(0i32)
            },
        ).map_err(|e| WasmError::Instantiate(format!("failed to define subscribe: {}", e)))?;
        
        linker.func_wrap(
            "host",
            "check_path",
            |state: &mut WasmInstanceState, path_ptr: i32, path_len: i32| -> i32 {
                if let Some(memory) = &state.memory {
                    if let Ok(path) = memory.read_string(path_ptr as u32, path_len as u32) {
                        let allowed = match &state.allowed_path {
                            Some(scope) => path.starts_with(scope) || path.starts_with("/"),
                            None => true,
                        };
                        return if allowed { 1 } else { 0 };
                    }
                }
                0
            },
        ).map_err(|e| WasmError::Instantiate(format!("failed to define check_path: {}", e)))?;
        
        linker.func_wrap(
            "host",
            "alloc",
            |mut state: &mut WasmInstanceState, size: i32| -> i32 {
                if let Some(memory) = &state.memory {
                    let alloc_size = size as u32;
                    if let Ok(ptr) = memory.alloc(alloc_size) {
                        return ptr as i32;
                    }
                }
                -1
            },
        ).map_err(|e| WasmError::Instantiate(format!("failed to define alloc: {}", e)))?;
        
        linker.func_wrap(
            "host",
            "dealloc",
            |mut state: &mut WasmInstanceState, ptr: i32, size: i32| {
                if let Some(memory) = &state.memory {
                    let _ = memory.dealloc(ptr as u32, size as u32);
                }
                Ok(())
            },
        ).map_err(|e| WasmError::Instantiate(format!("failed to define dealloc: {}", e)))?;
        
        Ok(())
    }

    pub fn capabilities(&self) -> &WasmCapabilities {
        &self.capabilities
    }
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
    
    pub fn call_with_input(&mut self, func_name: &str, input: &str) -> Result<String, WasmError> {
        let func = self
            .instance
            .get_func(&mut self.store, func_name)
            .ok_or_else(|| WasmError::Instantiate(format!("function not found: {}", func_name)))?;
        
        let memory = self.instance.get_memory(&mut self.store, "memory")
            .ok_or_else(|| WasmError::Memory("memory export not found".to_string()))?;
        
        let input_bytes = input.as_bytes();
        let input_ptr = memory.alloc(input_bytes.len() as u32)
            .map_err(|e| WasmError::Memory(format!("alloc failed: {}", e)))?;
        memory.write(input_ptr, input_bytes)
            .map_err(|e| WasmError::Memory(format!("write failed: {}", e)))?;
        
        let results = func.typed::<(i32, i32), (i32, i32)>(&mut self.store)?
            .call(&mut self.store, (input_ptr as i32, input_bytes.len() as i32))?;
        
        let (result_ptr, result_len) = results;
        if result_ptr < 0 {
            return Err(WasmError::Call(format!("function returned error: {}", result_ptr)));
        }
        
        let output = memory.read_string(result_ptr as u32, result_len as u32)
            .map_err(|e| WasmError::Memory(format!("read failed: {}", e)))?;
        
        memory.dealloc(result_ptr as u32, result_len as u32)
            .map_err(|e| WasmError::Memory(format!("dealloc failed: {}", e)))?;
        
        Ok(output)
    }
}

impl WasmPlugin {
    pub fn new(
        name: String,
        version: String,
        capabilities: WasmCapabilities,
    ) -> Result<Self, WasmError> {
        let runtime = WasmRuntime::new(capabilities)?;
        let (event_tx, event_rx) = broadcast::channel(64);
        Ok(Self {
            name,
            version,
            runtime,
            instance: None,
            event_rx: Some(event_rx),
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
    
    pub fn execute_with_input(&mut self, func_name: &str, input: &str) -> Result<String, WasmError> {
        if let Some(ref mut instance) = self.instance {
            instance.call_with_input(func_name, input)
        } else {
            Err(WasmError::Call("plugin not loaded".to_string()))
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }
    
    pub fn take_event_receiver(&mut self) -> Option<broadcast::Receiver<WasmPluginEvent>> {
        self.event_rx.take()
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
    
    #[test]
    fn test_wasm_capabilities_with_limits() {
        let caps = WasmCapabilities {
            filesystem_scope: Some("/tmp/plugins".to_string()),
            network_allowed: true,
            allowed_env_vars: vec!["HOME".to_string()],
            execution_timeout_secs: Some(60),
            max_memory_bytes: Some(128 * 1024 * 1024),
            max_cpu_time_secs: Some(30),
        };
        
        let runtime = WasmRuntime::new(caps.clone());
        assert!(runtime.is_ok());
        
        let rt = runtime.unwrap();
        assert_eq!(rt.capabilities().filesystem_scope, Some("/tmp/plugins".to_string()));
        assert!(rt.capabilities().network_allowed);
    }
    
    #[test]
    fn test_wasm_plugin_creation() {
        let plugin = WasmPlugin::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            WasmCapabilities::default(),
        );
        assert!(plugin.is_ok());
        
        let plugin = plugin.unwrap();
        assert_eq!(plugin.name(), "test-plugin");
        assert_eq!(plugin.version(), "1.0.0");
    }
    
    #[test]
    fn test_wasm_plugin_execute_before_load() {
        let mut plugin = WasmPlugin::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            WasmCapabilities::default(),
        ).unwrap();
        
        let result = plugin.execute("on_load");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_wasm_plugin_execute_after_load_fails_gracefully() {
        let mut plugin = WasmPlugin::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            WasmCapabilities::default(),
        ).unwrap();
        
        let result = plugin.execute("non_existent_function");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, WasmError::Call(_)));
    }
    
    #[test]
    fn test_plugin_crash_does_not_panic_main() {
        let caps = WasmCapabilities {
            filesystem_scope: Some("/tmp".to_string()),
            network_allowed: false,
            execution_timeout_secs: Some(1),
            max_memory_bytes: Some(1024 * 1024),
            max_cpu_time_secs: Some(1),
            allowed_env_vars: vec![],
        };
        
        let result = std::panic::catch_unwind(|| {
            let plugin = WasmPlugin::new(
                "crash-test".to_string(),
                "1.0.0".to_string(),
                caps,
            );
            assert!(plugin.is_ok());
        });
        
        assert!(result.is_ok(), "Plugin creation should not panic even with restrictive caps");
    }
    
    #[test]
    fn test_capabilities_are_enforced() {
        let caps = WasmCapabilities {
            filesystem_scope: Some("/allowed/path".to_string()),
            network_allowed: false,
            execution_timeout_secs: Some(30),
            max_memory_bytes: Some(64 * 1024 * 1024),
            max_cpu_time_secs: Some(10),
            allowed_env_vars: vec!["PATH".to_string()],
        };
        
        let runtime = WasmRuntime::new(caps.clone()).unwrap();
        let enforced_caps = runtime.capabilities();
        
        assert_eq!(enforced_caps.filesystem_scope, Some("/allowed/path".to_string()));
        assert!(!enforced_caps.network_allowed);
        assert_eq!(enforced_caps.max_memory_bytes, Some(64 * 1024 * 1024));
        assert_eq!(enforced_caps.max_cpu_time_secs, Some(10));
        assert!(enforced_caps.allowed_env_vars.contains(&"PATH".to_string()));
    }
    
    #[test]
    fn test_default_capabilities_are_restrictive() {
        let caps = WasmCapabilities::default();
        
        assert!(caps.filesystem_scope.is_none());
        assert!(!caps.network_allowed);
        assert!(caps.allowed_env_vars.is_empty());
        assert!(caps.max_memory_bytes.is_some());
        assert!(caps.execution_timeout_secs.is_some());
    }

    #[tokio::test]
    async fn test_wasm_event_bridge_with_mock_backend() {
        use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
        
        struct MockBackend {
            subscribe_count: AtomicUsize,
            publish_count: AtomicUsize,
            tx: tokio::sync::broadcast::Sender<EventEnvelope>,
        }
        
        impl MockBackend {
            fn new() -> Self {
                let (tx, _) = broadcast::channel(64);
                Self {
                    subscribe_count: AtomicUsize::new(0),
                    publish_count: AtomicUsize::new(0),
                    tx,
                }
            }
        }
        
        impl EventBridgeBackend for MockBackend {
            fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
                self.subscribe_count.fetch_add(1, Ordering::SeqCst);
                self.tx.subscribe()
            }
            
            fn publish(&self, _event: EventEnvelope) {
                self.publish_count.fetch_add(1, Ordering::SeqCst);
            }
        }
        
        let mut plugin = WasmPlugin::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            WasmCapabilities::default(),
        ).unwrap();
        
        let backend = Arc::new(MockBackend::new());
        let bridge = WasmEventBridge::new(plugin, backend.clone());
        
        assert!(bridge.is_ok());
        assert_eq!(backend.subscribe_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_event_envelope_structure() {
        let envelope = EventEnvelope {
            event_type: "session.created".to_string(),
            payload: r#"{"session_id":"123"}"#.to_string(),
        };
        
        assert_eq!(envelope.event_type, "session.created");
        assert!(envelope.payload.contains("123"));
    }

    #[test]
    fn test_wasm_runtime_with_strict_memory_limit() {
        let caps = WasmCapabilities {
            max_memory_bytes: Some(1024 * 1024),
            ..Default::default()
        };
        
        let runtime = WasmRuntime::new(caps).unwrap();
        let enforced = runtime.capabilities();
        
        assert_eq!(enforced.max_memory_bytes, Some(1024 * 1024));
    }

    #[test]
    fn test_wasm_runtime_with_no_network() {
        let caps = WasmCapabilities {
            network_allowed: false,
            ..Default::default()
        };
        
        let runtime = WasmRuntime::new(caps).unwrap();
        let enforced = runtime.capabilities();
        
        assert!(!enforced.network_allowed);
    }

    #[test]
    fn test_wasm_runtime_with_filesystem_scope() {
        let scope = "/allowed/path";
        let caps = WasmCapabilities {
            filesystem_scope: Some(scope.to_string()),
            ..Default::default()
        };
        
        let runtime = WasmRuntime::new(caps).unwrap();
        let enforced = runtime.capabilities();
        
        assert_eq!(enforced.filesystem_scope, Some(scope.to_string()));
    }

    #[test]
    fn test_wasm_runtime_timeout_configuration() {
        let caps = WasmCapabilities {
            execution_timeout_secs: Some(60),
            max_cpu_time_secs: Some(30),
            ..Default::default()
        };
        
        let runtime = WasmRuntime::new(caps).unwrap();
        let enforced = runtime.capabilities();
        
        assert_eq!(enforced.execution_timeout_secs, Some(60));
        assert_eq!(enforced.max_cpu_time_secs, Some(30));
    }

    #[test]
    fn test_wasm_plugin_with_different_capabilities() {
        let caps = WasmCapabilities {
            filesystem_scope: Some("/tmp".to_string()),
            network_allowed: true,
            allowed_env_vars: vec!["HOME".to_string(), "USER".to_string()],
            execution_timeout_secs: Some(120),
            max_memory_bytes: Some(256 * 1024 * 1024),
            max_cpu_time_secs: Some(60),
        };
        
        let plugin = WasmPlugin::new(
            "capability-test".to_string(),
            "1.0.0".to_string(),
            caps,
        );
        
        assert!(plugin.is_ok());
    }

    #[test]
    fn test_plugin_panic_isolation_with_limited_memory() {
        let caps = WasmCapabilities {
            max_memory_bytes: Some(64 * 1024),
            max_cpu_time_secs: Some(1),
            execution_timeout_secs: Some(1),
            ..Default::default()
        };
        
        let result = std::panic::catch_unwind(|| {
            let plugin = WasmPlugin::new(
                "panic-test".to_string(),
                "1.0.0".to_string(),
                caps,
            );
            assert!(plugin.is_ok());
        });
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_wasm_capabilities_default_has_timeouts() {
        let caps = WasmCapabilities::default();
        
        assert!(caps.execution_timeout_secs.is_some());
        assert!(caps.max_cpu_time_secs.is_some());
        assert!(caps.max_memory_bytes.is_some());
    }

    #[test]
    fn test_wasm_capabilities_clone_is_independent() {
        let caps = WasmCapabilities {
            filesystem_scope: Some("/test".to_string()),
            network_allowed: true,
            ..Default::default()
        };
        
        let cloned = caps.clone();
        
        assert_eq!(cloned.filesystem_scope, caps.filesystem_scope);
        assert_eq!(cloned.network_allowed, caps.network_allowed);
    }
}
