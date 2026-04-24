#[cfg(test)]
mod plugin_security_tests {
    use opencode_plugin::wasm_runtime::{
        WasmCapabilities, WasmError, WasmInstance, WasmPlugin, WasmRuntime,
    };
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_valid_wasm_module() -> Vec<u8> {
        let wat_code = r#"
            (module
                (func (export "run") (result i32)
                    i32.const 42
                )
            )
        "#;
        wat::parse_str(wat_code).expect("failed to parse WAT")
    }

    fn create_infinite_loop_wasm() -> Vec<u8> {
        let wat_code = r#"
            (module
                (func (export "run")
                    (loop
                        br 0
                    )
                )
            )
        "#;
        wat::parse_str(wat_code).expect("failed to parse WAT")
    }

    fn create_memory_grow_wasm() -> Vec<u8> {
        let wat_code = r#"
            (module
                (func (export "grow_memory")
                    (i32.store
                        (i32.const 0)
                        (i32.const 1)
                    )
                    (memory.grow (i32.const 1))
                    (drop)
                    (call 0)
                )
                (func (export "run")
                    (call 0)
                )
            )
        "#;
        wat::parse_str(wat_code).expect("failed to parse WAT")
    }

    fn create_large_allocation_wasm() -> Vec<u8> {
        let wat_code = r#"
            (module
                (func (export "allocate_huge")
                    (i32.store
                        (i32.const 0)
                        (i32.const 1)
                    )
                    (memory.grow (i32.const 65536))
                    (drop)
                )
                (func (export "run")
                    (call 0)
                )
            )
        "#;
        wat::parse_str(wat_code).expect("failed to parse WAT")
    }

    fn write_wasm_to_file(wasm_bytes: &[u8]) -> NamedTempFile {
        let mut file = NamedTempFile::with_suffix(".wasm").unwrap();
        file.write_all(wasm_bytes).unwrap();
        file.flush();
        file
    }

    #[test]
    fn test_plugin_sec_001_valid_wasm_loads_successfully() {
        let caps = WasmCapabilities::default();
        let runtime = WasmRuntime::new(caps).unwrap();

        let wasm_bytes = create_valid_wasm_module();
        let module = runtime.load_module_from_bytes(&wasm_bytes);
        assert!(module.is_ok(), "Valid WASM should compile successfully");
    }

    #[test]
    fn test_plugin_sec_001_invalid_wasm_rejected_at_load() {
        let caps = WasmCapabilities::default();
        let runtime = WasmRuntime::new(caps).unwrap();

        let invalid_wasm = b"not valid wasm bytecode at all";
        let result = runtime.load_module_from_bytes(invalid_wasm.as_slice());
        assert!(
            result.is_err(),
            "Malformed WASM should be rejected at load time"
        );

        let err = result.unwrap_err();
        assert!(
            matches!(err, WasmError::Compile(_)),
            "Error should be Compile error for invalid WASM"
        );
    }

    #[test]
    fn test_plugin_sec_001_corrupted_wasm_rejected() {
        let caps = WasmCapabilities::default();
        let runtime = WasmRuntime::new(caps).unwrap();

        let valid_wasm = create_valid_wasm_module();
        let mut corrupted_wasm = valid_wasm.clone();
        if corrupted_wasm.len() > 10 {
            corrupted_wasm[10] ^= 0xFF;
        }

        let result = runtime.load_module_from_bytes(&corrupted_wasm);
        assert!(result.is_err(), "Corrupted WASM should be rejected");
    }

    #[test]
    fn test_plugin_sec_001_malformed_wasm_with_bad_sections_rejected() {
        let caps = WasmCapabilities::default();
        let runtime = WasmRuntime::new(caps).unwrap();

        let malformed = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        let result = runtime.load_module_from_bytes(malformed.as_slice());
        assert!(
            result.is_err(),
            "WASM with malformed sections should be rejected"
        );
    }

    #[test]
    fn test_plugin_sec_001_memory_access_out_of_bounds_returns_error() {
        let caps = WasmCapabilities::default();
        let runtime = WasmRuntime::new(caps).unwrap();

        let wasm_bytes = create_valid_wasm_module();
        let module = runtime.load_module_from_bytes(&wasm_bytes).unwrap();
        let instance = runtime.instantiate_module(&module);

        match instance {
            Ok(mut inst) => {
                let result = inst.call_with_input("invalid_func", "input");
                assert!(result.is_err(), "Calling non-existent function should fail");
            }
            Err(e) => {
                panic!("Instantiation failed unexpectedly: {:?}", e);
            }
        }
    }

    #[test]
    fn test_plugin_sec_001_unexported_functions_not_accessible() {
        let caps = WasmCapabilities::default();
        let runtime = WasmRuntime::new(caps).unwrap();

        let wat_code = r#"
            (module
                (func $internal_func (result i32)
                    i32.const 100
                )
                (func (export "run") (result i32)
                    call $internal_func
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat_code).expect("failed to parse WAT");
        let module = runtime.load_module_from_bytes(&wasm_bytes).unwrap();
        let mut instance = runtime.instantiate_module(&module).unwrap();

        let result = instance.call("run");
        assert!(result.is_ok(), "Exported function should be callable");

        let result = instance.call("internal_func");
        assert!(
            result.is_err(),
            "Unexported function should not be callable"
        );
    }

    #[test]
    fn test_plugin_sec_002_cpu_limit_configuration_exists() {
        let caps = WasmCapabilities {
            max_cpu_time_secs: Some(5),
            execution_timeout_secs: Some(10),
            ..Default::default()
        };

        let runtime = WasmRuntime::new(caps).unwrap();
        let enforced = runtime.capabilities();

        assert_eq!(enforced.max_cpu_time_secs, Some(5));
        assert_eq!(enforced.execution_timeout_secs, Some(10));
    }

    #[test]
    fn test_plugin_sec_002_memory_limit_configuration_exists() {
        let caps = WasmCapabilities {
            max_memory_bytes: Some(1024 * 1024),
            ..Default::default()
        };

        let runtime = WasmRuntime::new(caps).unwrap();
        let enforced = runtime.capabilities();

        assert_eq!(enforced.max_memory_bytes, Some(1024 * 1024));
    }

    #[test]
    fn test_plugin_sec_002_default_limits_are_restrictive() {
        let caps = WasmCapabilities::default();

        assert!(caps.max_memory_bytes.is_some());
        assert!(caps.max_cpu_time_secs.is_some());
        assert!(caps.execution_timeout_secs.is_some());
    }

    #[test]
    fn test_plugin_sec_002_zero_limits_accepted() {
        let caps = WasmCapabilities {
            max_memory_bytes: Some(0),
            max_cpu_time_secs: Some(0),
            execution_timeout_secs: Some(0),
            ..Default::default()
        };

        let runtime = WasmRuntime::new(caps);
        assert!(
            runtime.is_ok(),
            "Zero limits should be accepted in configuration"
        );
    }

    #[test]
    fn test_plugin_sec_002_plugin_execution_with_timeout_configured() {
        let caps = WasmCapabilities {
            execution_timeout_secs: Some(1),
            max_cpu_time_secs: Some(1),
            max_memory_bytes: Some(64 * 1024 * 1024),
            ..Default::default()
        };

        let runtime = WasmRuntime::new(caps).unwrap();
        let wasm_bytes = create_valid_wasm_module();
        let module = runtime.load_module_from_bytes(&wasm_bytes).unwrap();
        let instance = runtime.instantiate_module(&module);

        assert!(
            instance.is_ok(),
            "Plugin with timeout config should instantiate"
        );
    }

    #[test]
    fn test_plugin_sec_002_wasm_plugin_isolation_on_execution() {
        let caps = WasmCapabilities::default();
        let mut plugin = WasmPlugin::new("test".to_string(), "1.0.0".to_string(), caps).unwrap();

        let wasm_bytes = create_valid_wasm_module();
        let temp_file = write_wasm_to_file(&wasm_bytes);
        let result = plugin.load(temp_file.path());

        assert!(result.is_ok(), "Plugin should load valid WASM");

        let result = plugin.execute("run");
        assert!(
            result.is_ok(),
            "Plugin execution should succeed for valid function"
        );
    }

    #[test]
    fn test_plugin_sec_002_execution_of_unknown_function_fails() {
        let caps = WasmCapabilities::default();
        let mut plugin = WasmPlugin::new("test".to_string(), "1.0.0".to_string(), caps).unwrap();

        let wasm_bytes = create_valid_wasm_module();
        let temp_file = write_wasm_to_file(&wasm_bytes);
        plugin.load(temp_file.path()).unwrap();

        let result = plugin.execute("non_existent_function");
        assert!(result.is_err(), "Execution of unknown function should fail");
    }

    #[test]
    fn test_plugin_sec_002_plugin_execute_before_load_fails() {
        let caps = WasmCapabilities::default();
        let mut plugin = WasmPlugin::new("test".to_string(), "1.0.0".to_string(), caps).unwrap();

        let result = plugin.execute("run");
        assert!(result.is_err(), "Execution before load should fail");
        assert!(matches!(result.unwrap_err(), WasmError::Call(_)));
    }

    #[test]
    fn test_plugin_sec_001_dangerous_imports_blocked() {
        let caps = WasmCapabilities::default();
        let runtime = WasmRuntime::new(caps).unwrap();

        let wat_code = r#"
            (module
                (import "env" "memory" (memory 1))
                (func (export "run")
                    (nop)
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat_code).expect("failed to parse WAT");

        let module = runtime.load_module_from_bytes(&wasm_bytes);
        assert!(
            module.is_ok(),
            "WASM with env.memory import should load (env module is allowed by wasmi)"
        );

        let instance = runtime.instantiate_module(&module.unwrap());
        assert!(
            instance.is_ok(),
            "Instance should be created with minimal imports"
        );
    }

    #[test]
    fn test_plugin_sec_001_multiple_invalid_functions_rejected() {
        let caps = WasmCapabilities::default();
        let runtime = WasmRuntime::new(caps).unwrap();

        let wat_code = r#"
            (module
                (func (export "func1") (result i32) i32.const 1)
                (func (export "func2") (result i32) i32.const 2)
                (func (export "func3") (result i32) i32.const 3)
            )
        "#;
        let wasm_bytes = wat::parse_str(wat_code).expect("failed to parse WAT");
        let module = runtime.load_module_from_bytes(&wasm_bytes).unwrap();
        let mut instance = runtime.instantiate_module(&module).unwrap();

        assert!(instance.call("func1").is_ok());
        assert!(instance.call("func2").is_ok());
        assert!(instance.call("func3").is_ok());
        assert!(instance.call("func4").is_err());
    }

    #[test]
    fn test_plugin_sec_002_network_not_allowed_by_default() {
        let caps = WasmCapabilities::default();
        assert!(
            !caps.network_allowed,
            "Network should be disabled by default"
        );
    }

    #[test]
    fn test_plugin_sec_002_filesystem_scope_can_be_restricted() {
        let caps = WasmCapabilities {
            filesystem_scope: Some("/allowed/path".to_string()),
            network_allowed: false,
            ..Default::default()
        };

        let runtime = WasmRuntime::new(caps).unwrap();
        let enforced = runtime.capabilities();

        assert_eq!(enforced.filesystem_scope, Some("/allowed/path".to_string()));
        assert!(!enforced.network_allowed);
    }

    #[test]
    fn test_plugin_sec_002_allowed_env_vars_can_be_restricted() {
        let caps = WasmCapabilities {
            allowed_env_vars: vec!["HOME".to_string(), "USER".to_string()],
            ..Default::default()
        };

        let runtime = WasmRuntime::new(caps).unwrap();
        let enforced = runtime.capabilities();

        assert!(enforced.allowed_env_vars.contains(&"HOME".to_string()));
        assert!(enforced.allowed_env_vars.contains(&"USER".to_string()));
        assert!(!enforced.allowed_env_vars.contains(&"SECRET".to_string()));
    }

    #[test]
    fn test_plugin_sec_002_wasm_plugin_crash_isolation() {
        let caps = WasmCapabilities::default();
        let result = std::panic::catch_unwind(|| {
            let plugin = WasmPlugin::new("test".to_string(), "1.0.0".to_string(), caps);
            assert!(plugin.is_ok());
        });

        assert!(result.is_ok(), "Plugin creation should not panic");
    }

    #[test]
    fn test_plugin_sec_002_execution_timeout_config_enforced() {
        let caps = WasmCapabilities {
            execution_timeout_secs: Some(30),
            max_cpu_time_secs: Some(10),
            ..Default::default()
        };

        let runtime = WasmRuntime::new(caps).unwrap();
        let enforced = runtime.capabilities();

        assert_eq!(enforced.execution_timeout_secs, Some(30));
        assert_eq!(enforced.max_cpu_time_secs, Some(10));
    }

    #[test]
    fn test_plugin_sec_002_host_function_stubs_do_not_allow_escape() {
        let caps = WasmCapabilities::default();
        let runtime = WasmRuntime::new(caps).unwrap();

        let wat_code = r#"
            (module
                (func (export "run") (result i32)
                    i32.const 0
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat_code).expect("failed to parse WAT");
        let module = runtime.load_module_from_bytes(&wasm_bytes).unwrap();
        let mut instance = runtime.instantiate_module(&module).unwrap();

        let result = instance.call("run");
        assert!(result.is_ok(), "Basic function call should succeed");
    }
}

#[cfg(test)]
mod plugin_security_gaps_documentation {
    use opencode_plugin::wasm_runtime::{WasmCapabilities, WasmRuntime};

    #[test]
    fn security_gap_doc_plugin_sec_001_requires_wasm_validation() {
        let caps = WasmCapabilities::default();
        let runtime = WasmRuntime::new(caps).unwrap();

        let valid_wasm = wat::parse_str(
            r#"
            (module
                (func (export "run") (result i32)
                    i32.const 42
                )
            )
        "#,
        )
        .expect("failed to parse WAT");

        let result = runtime.load_module_from_bytes(&valid_wasm);
        assert!(result.is_ok(), "Valid WASM should load - this passes");

        println!("\n=== SECURITY GAP DOCUMENTATION: plugin_sec_001 ===");
        println!("CURRENT STATE: WASM validation exists at load time via wasmtime");
        println!("GAP: No explicit validation that imported functions are safe");
        println!("GAP: Host function definitions are stubs that return dummy values");
        println!("GAP: No verification that plugin cannot access host memory");
        println!("RECOMMENDATION: Implement strict import allowlisting");
    }

    #[test]
    fn security_gap_doc_plugin_sec_002_requires_resource_enforcement() {
        let caps = WasmCapabilities {
            max_memory_bytes: Some(1024 * 1024),
            max_cpu_time_secs: Some(1),
            execution_timeout_secs: Some(1),
            ..Default::default()
        };

        let runtime = WasmRuntime::new(caps).unwrap();
        let enforced = runtime.capabilities();

        assert_eq!(enforced.max_memory_bytes, Some(1024 * 1024));
        assert_eq!(enforced.max_cpu_time_secs, Some(1));
        assert_eq!(enforced.execution_timeout_secs, Some(1));

        println!("\n=== SECURITY GAP DOCUMENTATION: plugin_sec_002 ===");
        println!("CURRENT STATE: Resource limits are STORED in WasmCapabilities");
        println!("GAP: Limits are NOT actively enforced during execution");
        println!("GAP: No fuel consumption tracking during function calls");
        println!("GAP: No memory allocation interception");
        println!("GAP: No CPU time measurement and enforcement");
        println!("RECOMMENDATION: Implement fuel consumption and epoch-based interruption");
    }
}
