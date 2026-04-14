use opencode_agent::{Agent, BuildAgent, GeneralAgent};
use opencode_core::tool::{
    build_default_registry, ToolDefinition, ToolExecutor, ToolParameter,
    ToolRegistry as CoreToolRegistry, ToolResult,
};
use opencode_core::{Message, Session};
use opencode_tools::registry::ToolSource;
use opencode_tools::{read::ReadTool, write::WriteTool, Tool, ToolRegistry};
use std::sync::Arc;

use crate::common::{MockLLMProvider, TempProject};

#[tokio::test]
async fn test_agent_runtime_uses_opencode_tools_tool_registry() {
    let provider = MockLLMProvider::new().with_response("Agent response");
    let mut session = Session::new();
    session.add_message(Message::user("Test".to_string()));

    let agent = BuildAgent::new();
    let tools = opencode_tools::ToolRegistry::new();

    let result = agent.run(&mut session, &provider, &tools).await;
    assert!(
        result.is_ok(),
        "Agent should run with opencode_tools::ToolRegistry"
    );
}

#[tokio::test]
async fn test_general_agent_uses_opencode_tools_tool_registry() {
    let provider = MockLLMProvider::new().with_response("Search results");
    let mut session = Session::new();
    session.add_message(Message::user("Find files".to_string()));

    let agent = GeneralAgent::new();
    let tools = opencode_tools::ToolRegistry::new();

    let result = agent.run(&mut session, &provider, &tools).await;
    assert!(
        result.is_ok(),
        "GeneralAgent should run with opencode_tools::ToolRegistry"
    );
}

#[tokio::test]
async fn test_tool_registry_execute_read_tool() {
    let project = TempProject::new();
    project.create_file("test.txt", "Hello, World!");

    let registry = ToolRegistry::new();
    registry.register(ReadTool::new()).await;

    let result = registry
        .execute(
            "read",
            serde_json::json!({"path": project.path().join("test.txt").to_string_lossy()}),
            None,
        )
        .await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.success);
    assert!(result.content.contains("Hello, World!"));
}

#[tokio::test]
async fn test_tool_registry_execute_write_tool() {
    let project = TempProject::new();

    let registry = ToolRegistry::new();
    registry.register(WriteTool).await;

    let file_path = project
        .path()
        .join("output.txt")
        .to_string_lossy()
        .to_string();
    let result = registry
        .execute(
            "write",
            serde_json::json!({"path": file_path, "content": "Test content"}),
            None,
        )
        .await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.success);

    let output_file = project.path().join("output.txt");
    assert!(output_file.exists());
    assert_eq!(
        std::fs::read_to_string(&output_file).unwrap(),
        "Test content"
    );
}

#[tokio::test]
async fn test_tool_registry_list_filtered() {
    let registry = ToolRegistry::new();
    registry.register(ReadTool::new()).await;

    let tools = registry.list_filtered(None).await;
    let tool_names: Vec<&str> = tools.iter().map(|(n, _, _)| n.as_str()).collect();

    assert!(
        tool_names.contains(&"read"),
        "Registry should contain 'read' tool. Found: {:?}",
        tool_names
    );
}

#[tokio::test]
async fn test_tool_registry_get_tool() {
    let registry = ToolRegistry::new();
    registry.register(ReadTool::new()).await;

    let tool = registry.get("read").await;
    assert!(tool.is_some(), "Should be able to get 'read' tool");

    let tool = registry.get("nonexistent").await;
    assert!(tool.is_none(), "Should not find nonexistent tool");
}

#[tokio::test]
async fn test_tool_registry_disabled_tool() {
    let mut registry = ToolRegistry::new();
    registry.register(ReadTool::new()).await;

    let mut disabled = std::collections::HashSet::new();
    disabled.insert("read".to_string());
    registry.set_disabled(disabled);

    let result = registry
        .execute("read", serde_json::json!({"path": "/test.txt"}), None)
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("disabled"),
        "Error should mention disabled: {}",
        err
    );
}

#[tokio::test]
async fn test_tool_registry_collision_resolution_builtin_overrides_custom() {
    #[derive(Clone)]
    struct CustomTool;
    #[async_trait::async_trait]
    impl Tool for CustomTool {
        fn name(&self) -> &str {
            "echo"
        }
        fn description(&self) -> &str {
            "Custom echo"
        }
        fn clone_tool(&self) -> Box<dyn Tool> {
            Box::new(self.clone())
        }
        async fn execute(
            &self,
            _args: serde_json::Value,
            _ctx: Option<opencode_tools::ToolContext>,
        ) -> Result<opencode_tools::ToolResult, opencode_core::OpenCodeError> {
            Ok(opencode_tools::ToolResult::ok("custom"))
        }
    }

    let registry = ToolRegistry::new();

    registry.register(CustomTool).await;
    registry.register(ReadTool::new()).await;

    let result = registry
        .execute("echo", serde_json::json!({"path": "/test.txt"}), None)
        .await
        .unwrap();

    assert_eq!(result.content, "custom");
}

#[tokio::test]
async fn test_tool_registry_multiple_tools_execution() {
    let registry = ToolRegistry::new();
    registry.register(ReadTool::new()).await;
    registry.register(WriteTool).await;

    let tools = registry.list_filtered(None).await;
    let tool_names: Vec<&str> = tools.iter().map(|(n, _, _)| n.as_str()).collect();

    assert!(tool_names.contains(&"read"));
    assert!(tool_names.contains(&"write"));
}

#[tokio::test]
async fn test_tool_registry_not_found_error() {
    let registry = ToolRegistry::new();

    let result = registry
        .execute("nonexistent", serde_json::json!({}), None)
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("not found") || err.to_string().contains("Tool"),
        "Error should indicate tool not found: {}",
        err
    );
}

#[tokio::test]
async fn test_tool_registry_tool_with_status() {
    let mut registry = ToolRegistry::new();
    registry.register(ReadTool::new()).await;

    let (tool, disabled) = registry.get_with_status("read").await.unwrap();
    assert!(!disabled, "Tool should not be disabled by default");
    assert_eq!(tool.name(), "read");

    let mut disabled_set = std::collections::HashSet::new();
    disabled_set.insert("read".to_string());
    registry.set_disabled(disabled_set);

    let (_tool, disabled) = registry.get_with_status("read").await.unwrap();
    assert!(disabled, "Tool should be marked as disabled");
}

mod core_tool_registry_tests {
    use super::*;

    #[test]
    fn test_core_tool_registry_new() {
        let registry = CoreToolRegistry::new();
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_core_tool_registry_register_and_get() {
        let mut registry = CoreToolRegistry::new();
        let executor: ToolExecutor = Arc::new(|_| Ok("test".to_string()));
        registry.register(
            ToolDefinition {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            executor.clone(),
        );

        assert!(registry.contains("test_tool"));
        assert_eq!(registry.get("test_tool").unwrap().name, "test_tool");
        assert!(registry.get_executor("test_tool").is_some());
    }

    #[test]
    fn test_core_tool_registry_disabled_tools() {
        let mut registry = CoreToolRegistry::new();
        let executor: ToolExecutor = Arc::new(|_| Ok("test".to_string()));
        registry.register(
            ToolDefinition {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            executor,
        );

        registry.set_disabled(std::collections::HashSet::from(["test_tool".to_string()]));

        assert!(registry.is_disabled("test_tool"));
        assert!(registry.get_executor("test_tool").is_none());
    }

    #[test]
    fn test_core_tool_registry_list_with_status() {
        let mut registry = CoreToolRegistry::new();
        let executor: ToolExecutor = Arc::new(|_| Ok("test".to_string()));
        registry.register(
            ToolDefinition {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            executor,
        );

        registry.set_disabled(std::collections::HashSet::from(["test_tool".to_string()]));

        let listed = registry.list_with_status();
        let (name, disabled) = listed.iter().find(|(n, _)| *n == "test_tool").unwrap();
        assert_eq!(*name, "test_tool");
        assert!(*disabled);
    }

    #[test]
    fn test_core_tool_registry_executor_execution() {
        let mut registry = CoreToolRegistry::new();
        let executor: ToolExecutor = Arc::new(|args: serde_json::Value| {
            let input = args.get("input").and_then(|v| v.as_str()).unwrap_or("");
            Ok(format!("echo: {}", input))
        });
        registry.register(
            ToolDefinition {
                name: "echo".to_string(),
                description: "Echo back input".to_string(),
                parameters: vec![ToolParameter {
                    name: "input".to_string(),
                    description: "Input to echo".to_string(),
                    required: true,
                    schema: serde_json::json!({"type": "string"}),
                }],
                ..Default::default()
            },
            executor,
        );

        let exec = registry.get_executor("echo").unwrap();
        let result = exec(serde_json::json!({"input": "hello"})).unwrap();
        assert_eq!(result, "echo: hello");
    }

    #[test]
    fn test_core_tool_registry_requires_approval() {
        let mut registry = CoreToolRegistry::new();
        let executor: ToolExecutor = Arc::new(|_| Ok("test".to_string()));
        registry.register(
            ToolDefinition {
                name: "approval_tool".to_string(),
                description: "Tool requiring approval".to_string(),
                parameters: vec![],
                requires_approval: true,
                ..Default::default()
            },
            executor,
        );

        assert!(registry.requires_approval("approval_tool"));
        assert!(!registry.requires_approval("nonexistent"));
    }

    #[test]
    fn test_core_tool_registry_get_executor_with_status() {
        let mut registry = CoreToolRegistry::new();
        let executor: ToolExecutor = Arc::new(|_| Ok("test".to_string()));
        registry.register(
            ToolDefinition {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            executor.clone(),
        );

        let (exec, disabled) = registry.get_executor_with_status("test_tool").unwrap();
        assert!(!disabled);

        registry.set_disabled(std::collections::HashSet::from(["test_tool".to_string()]));
        let (exec_after, disabled_after) = registry.get_executor_with_status("test_tool").unwrap();
        assert!(disabled_after);
    }

    #[test]
    fn test_core_tool_registry_get_all() {
        let mut registry = CoreToolRegistry::new();
        let executor: ToolExecutor = Arc::new(|_| Ok("test".to_string()));

        registry.register(
            ToolDefinition {
                name: "tool1".to_string(),
                description: "First tool".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            executor.clone(),
        );
        registry.register(
            ToolDefinition {
                name: "tool2".to_string(),
                description: "Second tool".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            executor,
        );

        let all = registry.get_all();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_core_tool_registry_build_default_has_builtins() {
        let registry = build_default_registry();
        assert!(registry.contains("read"));
        assert!(registry.contains("write"));
        assert!(registry.contains("grep"));
        assert!(registry.contains("bash"));
    }

    #[test]
    fn test_core_tool_registry_multiple_tool_registration() {
        let mut registry = CoreToolRegistry::new();
        let executor: ToolExecutor = Arc::new(|_| Ok("test".to_string()));

        registry.register(
            ToolDefinition {
                name: "tool1".to_string(),
                description: "First".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            executor.clone(),
        );
        registry.register(
            ToolDefinition {
                name: "tool2".to_string(),
                description: "Second".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            executor,
        );

        assert_eq!(registry.list().len(), 2);
    }
}

mod opencode_tools_caching_tests {
    use super::*;

    #[derive(Clone)]
    struct SafeCachingTestTool {
        call_count: std::sync::Arc<std::sync::Mutex<u32>>,
    }

    impl SafeCachingTestTool {
        fn new() -> Self {
            Self {
                call_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
            }
        }
    }

    #[async_trait::async_trait]
    impl Tool for SafeCachingTestTool {
        fn name(&self) -> &str {
            "safe_caching_tool"
        }
        fn description(&self) -> &str {
            "A safe tool for testing caching"
        }
        fn clone_tool(&self) -> Box<dyn Tool> {
            let count = std::sync::Arc::clone(&self.call_count);
            Box::new(SafeCachingTestTool { call_count: count })
        }
        fn is_safe(&self) -> bool {
            true
        }
        async fn execute(
            &self,
            args: serde_json::Value,
            _ctx: Option<opencode_tools::ToolContext>,
        ) -> Result<opencode_tools::ToolResult, opencode_core::OpenCodeError> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;
            let input = args
                .get("input")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            Ok(opencode_tools::ToolResult::ok(format!(
                "call {}: {}",
                *count, input
            )))
        }
    }

    #[tokio::test]
    async fn test_caching_safe_tool_caches_result() {
        let registry = ToolRegistry::new();
        registry.register(SafeCachingTestTool::new()).await;

        let args = serde_json::json!({"input": "test"});

        let result1 = registry
            .execute("safe_caching_tool", args.clone(), None)
            .await
            .unwrap();
        let result2 = registry
            .execute("safe_caching_tool", args.clone(), None)
            .await
            .unwrap();

        assert!(result1.content.contains("call 1:"));
        assert!(result2.content.contains("call 1:"), "Second call should use cached result");
    }

    #[tokio::test]
    async fn test_caching_different_args_different_cache() {
        let registry = ToolRegistry::new();
        registry.register(SafeCachingTestTool::new()).await;

        let args1 = serde_json::json!({"input": "test1"});
        let args2 = serde_json::json!({"input": "test2"});

        let result1 = registry
            .execute("safe_caching_tool", args1.clone(), None)
            .await
            .unwrap();
        let result2 = registry
            .execute("safe_caching_tool", args2.clone(), None)
            .await
            .unwrap();

        assert!(result1.content.contains("call 1:"));
        assert!(result2.content.contains("call 2:"), "Different args should not use cached result");
    }

    #[tokio::test]
    async fn test_caching_ttl_expiration() {
        use std::time::Duration;
        let registry = ToolRegistry::with_ttl(Duration::from_millis(50));
        registry.register(SafeCachingTestTool::new()).await;

        let args = serde_json::json!({});

        let result1 = registry
            .execute("safe_caching_tool", args.clone(), None)
            .await
            .unwrap();
        assert!(result1.content.contains("call 1:"));

        tokio::time::sleep(Duration::from_millis(60)).await;

        let result2 = registry
            .execute("safe_caching_tool", args.clone(), None)
            .await
            .unwrap();
        assert!(result2.content.contains("call 2:"), "After TTL, should re-execute");
    }

    #[tokio::test]
    async fn test_caching_invalidation_for_tool() {
        let registry = ToolRegistry::new();
        registry.register(SafeCachingTestTool::new()).await;

        let args = serde_json::json!({"input": "test"});

        let result1 = registry
            .execute("safe_caching_tool", args.clone(), None)
            .await
            .unwrap();
        assert!(result1.content.contains("call 1:"));

        registry.invalidate_cache_for_tool("safe_caching_tool").await;

        let result2 = registry
            .execute("safe_caching_tool", args.clone(), None)
            .await
            .unwrap();
        assert!(result2.content.contains("call 2:"), "After invalidation, should re-execute");
    }

    #[tokio::test]
    async fn test_caching_invalidation_all() {
        #[derive(Clone)]
        struct AnotherSafeTool;
        #[async_trait::async_trait]
        impl Tool for AnotherSafeTool {
            fn name(&self) -> &str {
                "another_safe_tool"
            }
            fn description(&self) -> &str {
                "Another safe tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(AnotherSafeTool)
            }
            fn is_safe(&self) -> bool {
                true
            }
            async fn execute(
                &self,
                args: serde_json::Value,
                _ctx: Option<opencode_tools::ToolContext>,
            ) -> Result<opencode_tools::ToolResult, opencode_core::OpenCodeError> {
                let input = args
                    .get("input")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                Ok(opencode_tools::ToolResult::ok(format!("result: {}", input)))
            }
        }

        let registry = ToolRegistry::new();
        registry.register(SafeCachingTestTool::new()).await;
        registry.register(AnotherSafeTool).await;

        registry
            .execute("safe_caching_tool", serde_json::json!({"input": "test"}), None)
            .await
            .unwrap();
        registry
            .execute(
                "another_safe_tool",
                serde_json::json!({"input": "test"}),
                None,
            )
            .await
            .unwrap();

        registry.invalidate_all_cache().await;

        let result = registry
            .execute("safe_caching_tool", serde_json::json!({"input": "test"}), None)
            .await
            .unwrap();
        assert!(result.content.contains("call"));
    }

    #[tokio::test]
    async fn test_async_execution_parallel() {
        use opencode_tools::registry::ToolCall as ToolsToolCall;

        #[derive(Clone)]
        struct ParallelTestTool;
        #[async_trait::async_trait]
        impl Tool for ParallelTestTool {
            fn name(&self) -> &str {
                "parallel_tool"
            }
            fn description(&self) -> &str {
                "Tool for parallel execution test"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(ParallelTestTool)
            }
            async fn execute(
                &self,
                args: serde_json::Value,
                _ctx: Option<opencode_tools::ToolContext>,
            ) -> Result<opencode_tools::ToolResult, opencode_core::OpenCodeError> {
                let input = args
                    .get("input")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                Ok(opencode_tools::ToolResult::ok(format!("processed: {}", input)))
            }
        }

        let registry = ToolRegistry::new();
        registry.register(ParallelTestTool).await;

        let calls = vec![
            ToolsToolCall {
                name: "parallel_tool".to_string(),
                args: serde_json::json!({"input": "first"}),
                ctx: None,
            },
            ToolsToolCall {
                name: "parallel_tool".to_string(),
                args: serde_json::json!({"input": "second"}),
                ctx: None,
            },
        ];

        let results = registry.execute_parallel(calls).await;
        assert_eq!(results.len(), 2);
    }
}

mod registry_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_opencode_tools_registry_with_multiple_agents() {
        let provider = MockLLMProvider::new().with_response("Agent response");
        let mut session = Session::new();
        session.add_message(Message::user("Test".to_string()));

        let agent = BuildAgent::new();
        let tools = ToolRegistry::new();
        tools.register(ReadTool::new()).await;
        tools.register(WriteTool).await;

        let result = agent.run(&mut session, &provider, &tools).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_both_registries_can_coexist() {
        let mut core_registry = CoreToolRegistry::new();
        let tools_registry = ToolRegistry::new();

        tools_registry.register(ReadTool::new()).await;

        let executor: ToolExecutor = Arc::new(|_| Ok("core".to_string()));
        core_registry.register(
            ToolDefinition {
                name: "core_tool".to_string(),
                description: "Core tool".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            executor,
        );

        assert!(core_registry.contains("core_tool"));
        assert!(!core_registry.contains("read"));

        assert!(tools_registry.get("read").await.is_some());
        assert!(tools_registry.get("core_tool").await.is_none());

        let core_list = core_registry.list();
        let tools_list = tools_registry.list_filtered(None).await;
        assert!(core_list.contains(&"core_tool".to_string()));
        assert!(tools_list.iter().any(|(n, _, _)| n == "read"));
    }
}

mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_opencode_tools_disabled_tool_error_message() {
        let mut registry = ToolRegistry::new();
        registry.register(ReadTool::new()).await;

        let mut disabled = std::collections::HashSet::new();
        disabled.insert("read".to_string());
        registry.set_disabled(disabled);

        let result = registry
            .execute("read", serde_json::json!({"path": "/test.txt"}), None)
            .await;

        match result {
            Err(opencode_core::OpenCodeError::Tool(msg)) => {
                assert!(
                    msg.contains("disabled") || msg.contains("disabled"),
                    "Error should mention disabled"
                );
            }
            other => panic!("Expected disabled tool error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_opencode_tools_nonexistent_tool_error() {
        let registry = ToolRegistry::new();

        let result = registry
            .execute("nonexistent", serde_json::json!({}), None)
            .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_core_registry_nonexistent_tool_executor() {
        let registry = CoreToolRegistry::new();
        assert!(registry.get_executor("nonexistent").is_none());
    }

    #[test]
    fn test_core_registry_disabled_executor_returns_none() {
        let mut registry = CoreToolRegistry::new();
        let executor: ToolExecutor = Arc::new(|_| Ok("test".to_string()));
        registry.register(
            ToolDefinition {
                name: "test".to_string(),
                description: "Test".to_string(),
                parameters: vec![],
                ..Default::default()
            },
            executor,
        );

        registry.set_disabled(std::collections::HashSet::from(["test".to_string()]));
        assert!(registry.get_executor("test").is_none());
    }

    #[tokio::test]
    async fn test_safe_tool_failure_not_cached() {
        #[derive(Clone)]
        struct SafeFailingTool;
        #[async_trait::async_trait]
        impl Tool for SafeFailingTool {
            fn name(&self) -> &str {
                "safe_failing_tool"
            }
            fn description(&self) -> &str {
                "A safe tool that fails"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(SafeFailingTool)
            }
            fn is_safe(&self) -> bool {
                true
            }
            async fn execute(
                &self,
                _args: serde_json::Value,
                _ctx: Option<opencode_tools::ToolContext>,
            ) -> Result<opencode_tools::ToolResult, opencode_core::OpenCodeError> {
                Ok(opencode_tools::ToolResult::err("intentional failure"))
            }
        }

        let registry = ToolRegistry::new();
        registry.register(SafeFailingTool).await;

        let args = serde_json::json!({});

        let result1 = registry
            .execute("safe_failing_tool", args.clone(), None)
            .await
            .unwrap();
        assert!(!result1.success);

        let result2 = registry
            .execute("safe_failing_tool", args.clone(), None)
            .await
            .unwrap();
        assert!(!result2.success);
        assert!(result2.error.unwrap().contains("intentional failure"));
    }
}

mod collision_and_priority_tests {
    use super::*;

    #[tokio::test]
    async fn test_collision_resolution_plugin_overrides_custom_global() {
        #[derive(Clone)]
        struct GlobalTool;
        #[async_trait::async_trait]
        impl Tool for GlobalTool {
            fn name(&self) -> &str {
                "collision_tool"
            }
            fn description(&self) -> &str {
                "Global tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<opencode_tools::ToolContext>,
            ) -> Result<opencode_tools::ToolResult, opencode_core::OpenCodeError> {
                Ok(opencode_tools::ToolResult::ok("global"))
            }
        }

        #[derive(Clone)]
        struct PluginTool;
        #[async_trait::async_trait]
        impl Tool for PluginTool {
            fn name(&self) -> &str {
                "collision_tool"
            }
            fn description(&self) -> &str {
                "Plugin tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<opencode_tools::ToolContext>,
            ) -> Result<opencode_tools::ToolResult, opencode_core::OpenCodeError> {
                Ok(opencode_tools::ToolResult::ok("plugin"))
            }
        }

        let registry = ToolRegistry::new();

        registry
            .register_with_source(GlobalTool, ToolSource::CustomGlobal)
            .await;
        registry
            .register_with_source(PluginTool, ToolSource::Plugin)
            .await;

        let result = registry
            .execute("collision_tool", serde_json::json!({}), None)
            .await
            .unwrap();
        assert_eq!(result.content, "plugin");
    }

    #[tokio::test]
    async fn test_collision_resolution_custom_project_overrides_custom_global() {
        #[derive(Clone)]
        struct GlobalTool;
        #[async_trait::async_trait]
        impl Tool for GlobalTool {
            fn name(&self) -> &str {
                "collision_tool"
            }
            fn description(&self) -> &str {
                "Global tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<opencode_tools::ToolContext>,
            ) -> Result<opencode_tools::ToolResult, opencode_core::OpenCodeError> {
                Ok(opencode_tools::ToolResult::ok("global"))
            }
        }

        #[derive(Clone)]
        struct ProjectTool;
        #[async_trait::async_trait]
        impl Tool for ProjectTool {
            fn name(&self) -> &str {
                "collision_tool"
            }
            fn description(&self) -> &str {
                "Project tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<opencode_tools::ToolContext>,
            ) -> Result<opencode_tools::ToolResult, opencode_core::OpenCodeError> {
                Ok(opencode_tools::ToolResult::ok("project"))
            }
        }

        let registry = ToolRegistry::new();

        registry
            .register_with_source(GlobalTool, ToolSource::CustomGlobal)
            .await;
        registry
            .register_with_source(ProjectTool, ToolSource::CustomProject)
            .await;

        let result = registry
            .execute("collision_tool", serde_json::json!({}), None)
            .await
            .unwrap();
        assert_eq!(result.content, "project");
    }

    #[tokio::test]
    async fn test_builtin_cannot_be_overridden() {
        #[derive(Clone)]
        struct BuiltinTool;
        #[async_trait::async_trait]
        impl Tool for BuiltinTool {
            fn name(&self) -> &str {
                "my_tool"
            }
            fn description(&self) -> &str {
                "Builtin"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<opencode_tools::ToolContext>,
            ) -> Result<opencode_tools::ToolResult, opencode_core::OpenCodeError> {
                Ok(opencode_tools::ToolResult::ok("builtin"))
            }
        }

        #[derive(Clone)]
        struct CustomTool;
        #[async_trait::async_trait]
        impl Tool for CustomTool {
            fn name(&self) -> &str {
                "my_tool"
            }
            fn description(&self) -> &str {
                "Custom"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<opencode_tools::ToolContext>,
            ) -> Result<opencode_tools::ToolResult, opencode_core::OpenCodeError> {
                Ok(opencode_tools::ToolResult::ok("custom"))
            }
        }

        let registry = ToolRegistry::new();

        registry.register(BuiltinTool).await;
        registry
            .register_with_source(CustomTool, ToolSource::CustomGlobal)
            .await;

        let result = registry
            .execute("my_tool", serde_json::json!({}), None)
            .await
            .unwrap();
        assert_eq!(result.content, "builtin");
    }

    #[tokio::test]
    async fn test_tool_source_ordering() {
        assert!(ToolSource::Builtin < ToolSource::Plugin);
        assert!(ToolSource::Plugin < ToolSource::CustomProject);
        assert!(ToolSource::CustomProject < ToolSource::CustomGlobal);
    }
}
