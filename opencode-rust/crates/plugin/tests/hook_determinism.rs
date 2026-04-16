use indexmap::IndexMap;
use opencode_plugin::{
    sealed, Plugin, PluginConfig, PluginDomain, PluginError, PluginManager, PluginPermissions,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

struct PriorityTestPlugin {
    name: String,
}

impl PriorityTestPlugin {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl sealed::SealedPlugin for PriorityTestPlugin {}

impl Plugin for PriorityTestPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn init(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn description(&self) -> &str {
        "priority test plugin"
    }

    fn on_start(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

fn register_priority_plugin(manager: &mut PluginManager, name: &str, priority: i32) {
    let plugin = PriorityTestPlugin::new(name);
    let config = PluginConfig {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        enabled: true,
        priority,
        domain: PluginDomain::Runtime,
        options: IndexMap::new(),
        permissions: PluginPermissions::default(),
    };
    manager
        .register_with_config(Box::new(plugin), config)
        .unwrap();
}

#[test]
fn hook_determinism() {
    let mut manager = PluginManager::new();

    register_priority_plugin(&mut manager, "plugin-a", 10);
    register_priority_plugin(&mut manager, "plugin-b", 20);
    register_priority_plugin(&mut manager, "plugin-c", 30);

    let first_call = manager.sorted_plugin_names();
    assert_eq!(
        first_call,
        vec!["plugin-a", "plugin-b", "plugin-c"],
        "First call should return plugins in ascending priority order"
    );

    for i in 0..100 {
        let result = manager.sorted_plugin_names();
        assert_eq!(
            result, first_call,
            "Iteration {}: sorted_plugin_names() should return consistent ordering",
            i
        );
    }
}

#[test]
fn hook_order_is_deterministic() {
    let call_order = Arc::new(AtomicUsize::new(0));
    let call_sequence: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    struct OrderedPlugin {
        name: String,
        call_count: Arc<AtomicUsize>,
        call_sequence: Arc<Mutex<Vec<String>>>,
    }

    impl OrderedPlugin {
        fn new(
            name: &str,
            call_count: Arc<AtomicUsize>,
            call_sequence: Arc<Mutex<Vec<String>>>,
        ) -> Self {
            Self {
                name: name.to_string(),
                call_count,
                call_sequence,
            }
        }
    }

    impl sealed::SealedPlugin for OrderedPlugin {}

    impl Plugin for OrderedPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn init(&mut self) -> Result<(), PluginError> {
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), PluginError> {
            Ok(())
        }

        fn description(&self) -> &str {
            "ordered plugin for testing"
        }

        fn on_start(&mut self) -> Result<(), PluginError> {
            let order = self.call_count.fetch_add(1, Ordering::SeqCst);
            let mut seq = self.call_sequence.lock().unwrap();
            if seq.len() == order as usize {
                seq.push(self.name.clone());
            } else {
                seq.push(format!("OUT_OF_ORDER:{}", self.name));
            }
            Ok(())
        }
    }

    let mut manager = PluginManager::new();

    let plugin_alpha = OrderedPlugin::new("alpha", call_order.clone(), call_sequence.clone());
    let plugin_beta = OrderedPlugin::new("beta", call_order.clone(), call_sequence.clone());
    let plugin_gamma = OrderedPlugin::new("gamma", call_order.clone(), call_sequence.clone());

    manager.register(Box::new(plugin_alpha)).unwrap();
    manager.register(Box::new(plugin_beta)).unwrap();
    manager.register(Box::new(plugin_gamma)).unwrap();

    manager.on_start_all().unwrap();

    let sequence = call_sequence.lock().unwrap();
    assert_eq!(sequence.len(), 3, "Expected 3 plugins to be called");
    assert_eq!(sequence[0], "alpha", "First plugin should be alpha");
    assert_eq!(sequence[1], "beta", "Second plugin should be beta");
    assert_eq!(sequence[2], "gamma", "Third plugin should be gamma");
}

#[test]
fn hook_order_is_consistent_across_invocations() {
    for iteration in 0..3 {
        let call_order = Arc::new(AtomicUsize::new(0));
        let call_sequence: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        struct OrderedPlugin {
            name: String,
            call_count: Arc<AtomicUsize>,
            call_sequence: Arc<Mutex<Vec<String>>>,
        }

        impl OrderedPlugin {
            fn new(
                name: &str,
                call_count: Arc<AtomicUsize>,
                call_sequence: Arc<Mutex<Vec<String>>>,
            ) -> Self {
                Self {
                    name: name.to_string(),
                    call_count,
                    call_sequence,
                }
            }
        }

        impl sealed::SealedPlugin for OrderedPlugin {}

        impl Plugin for OrderedPlugin {
            fn name(&self) -> &str {
                &self.name
            }

            fn version(&self) -> &str {
                "1.0.0"
            }

            fn init(&mut self) -> Result<(), PluginError> {
                Ok(())
            }

            fn shutdown(&mut self) -> Result<(), PluginError> {
                Ok(())
            }

            fn description(&self) -> &str {
                "ordered plugin for testing"
            }

            fn on_start(&mut self) -> Result<(), PluginError> {
                let order = self.call_count.fetch_add(1, Ordering::SeqCst);
                let mut seq = self.call_sequence.lock().unwrap();
                if seq.len() == order as usize {
                    seq.push(self.name.clone());
                } else {
                    seq.push(format!("OUT_OF_ORDER:{}", self.name));
                }
                Ok(())
            }
        }

        let mut manager = PluginManager::new();

        let plugin_a = OrderedPlugin::new("plugin-a", call_order.clone(), call_sequence.clone());
        let plugin_b = OrderedPlugin::new("plugin-b", call_order.clone(), call_sequence.clone());
        let plugin_c = OrderedPlugin::new("plugin-c", call_order.clone(), call_sequence.clone());

        manager.register(Box::new(plugin_a)).unwrap();
        manager.register(Box::new(plugin_b)).unwrap();
        manager.register(Box::new(plugin_c)).unwrap();

        manager.on_start_all().unwrap();

        let sequence = call_sequence.lock().unwrap();
        assert_eq!(
            sequence.len(),
            3,
            "Iteration {}: Expected 3 plugins",
            iteration
        );
        assert_eq!(
            sequence[0], "plugin-a",
            "Iteration {}: First plugin should be plugin-a",
            iteration
        );
        assert_eq!(
            sequence[1], "plugin-b",
            "Iteration {}: Second plugin should be plugin-b",
            iteration
        );
        assert_eq!(
            sequence[2], "plugin-c",
            "Iteration {}: Third plugin should be plugin-c",
            iteration
        );
    }
}
