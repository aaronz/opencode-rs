use criterion::{black_box, criterion_group, criterion_main, Criterion};
use opencode_plugin::{sealed, Plugin, PluginManager};
use std::time::Duration;

struct DummyPlugin {
    name: String,
}

impl sealed::SealedPlugin for DummyPlugin {}

impl Plugin for DummyPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn init(&mut self) -> Result<(), opencode_plugin::PluginError> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), opencode_plugin::PluginError> {
        Ok(())
    }

    fn description(&self) -> &str {
        "dummy plugin for benchmarking"
    }
}

fn plugin_manager_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("plugin_manager");
    group.measurement_time(Duration::from_secs(3));

    group.bench_function("plugin_manager_new", |b| {
        b.iter(|| {
            let manager = PluginManager::new();
            black_box(manager)
        });
    });

    group.bench_function("plugin_manager_register_single", |b| {
        b.iter(|| {
            let mut manager = PluginManager::new();
            let plugin = Box::new(DummyPlugin {
                name: "test-plugin".to_string(),
            });
            let result = manager.register(plugin);
            black_box(result)
        });
    });

    group.bench_function("plugin_manager_register_10_plugins", |b| {
        b.iter(|| {
            let mut manager = PluginManager::new();
            for i in 0..10 {
                let plugin = Box::new(DummyPlugin {
                    name: format!("test-plugin-{}", i),
                });
                let _ = manager.register(plugin);
            }
            black_box(manager)
        });
    });

    group.bench_function("plugin_manager_register_50_plugins", |b| {
        b.iter(|| {
            let mut manager = PluginManager::new();
            for i in 0..50 {
                let plugin = Box::new(DummyPlugin {
                    name: format!("test-plugin-{}", i),
                });
                let _ = manager.register(plugin);
            }
            black_box(manager)
        });
    });

    group.bench_function("plugin_manager_get_plugin_existing", |b| {
        let mut manager = PluginManager::new();
        for i in 0..10 {
            let plugin = Box::new(DummyPlugin {
                name: format!("test-plugin-{}", i),
            });
            let _ = manager.register(plugin);
        }
        b.iter(|| {
            let plugin = manager.get_plugin("test-plugin-5");
            black_box(plugin)
        });
    });

    group.bench_function("plugin_manager_get_plugin_nonexistent", |b| {
        let mut manager = PluginManager::new();
        for i in 0..10 {
            let plugin = Box::new(DummyPlugin {
                name: format!("test-plugin-{}", i),
            });
            let _ = manager.register(plugin);
        }
        b.iter(|| {
            let plugin = manager.get_plugin("nonexistent-plugin");
            black_box(plugin)
        });
    });

    group.bench_function("plugin_manager_on_start_all", |b| {
        let mut manager = PluginManager::new();
        for i in 0..10 {
            let plugin = Box::new(DummyPlugin {
                name: format!("test-plugin-{}", i),
            });
            let _ = manager.register(plugin);
        }
        b.iter(|| {
            let result = manager.on_start_all();
            black_box(result)
        });
    });

    group.bench_function("plugin_manager_on_tool_call_all", |b| {
        let mut manager = PluginManager::new();
        for i in 0..10 {
            let plugin = Box::new(DummyPlugin {
                name: format!("test-plugin-{}", i),
            });
            let _ = manager.register(plugin);
        }
        let args = serde_json::json!({"file": "test.txt"});
        b.iter(|| {
            let result = manager.on_tool_call_all("read", &args, "session-123");
            black_box(result)
        });
    });

    group.bench_function("plugin_manager_on_message_all", |b| {
        let mut manager = PluginManager::new();
        for i in 0..10 {
            let plugin = Box::new(DummyPlugin {
                name: format!("test-plugin-{}", i),
            });
            let _ = manager.register(plugin);
        }
        b.iter(|| {
            let result = manager.on_message_all("Hello world", "session-123");
            black_box(result)
        });
    });

    group.bench_function("plugin_manager_shutdown", |b| {
        let mut manager = PluginManager::new();
        for i in 0..10 {
            let plugin = Box::new(DummyPlugin {
                name: format!("test-plugin-{}", i),
            });
            let _ = manager.register(plugin);
        }
        b.iter(|| {
            let result = manager.shutdown();
            black_box(result)
        });
    });
}

criterion_group!(benches, plugin_manager_benches);
criterion_main!(benches);
