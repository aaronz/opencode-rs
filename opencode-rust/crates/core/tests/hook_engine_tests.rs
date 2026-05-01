use opencode_core::hook::{
    HookAction, HookDefinition, HookEngine, HookEvent, HookFailurePolicy, HookId, HookPoint,
    HookResult, HookTrigger,
};

fn test_hook_definition() -> HookDefinition {
    HookDefinition::new(
        HookId::User(1),
        "test_hook".to_string(),
        HookTrigger::HookPoint(HookPoint::BeforeToolExecution),
        HookAction::Log {
            message: "test".to_string(),
            level: "info".to_string(),
        },
    )
}

#[tokio::test]
async fn test_hook_engine_register_and_unregister() {
    let engine = HookEngine::new();

    let hook = test_hook_definition();
    let hook_id = hook.id;

    engine.register(hook).await;
    let hooks = engine
        .get_hooks_for_point(HookPoint::BeforeToolExecution)
        .await;
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].id, hook_id);

    let removed = engine.unregister(&hook_id).await;
    assert!(removed);
    let hooks = engine
        .get_hooks_for_point(HookPoint::BeforeToolExecution)
        .await;
    assert!(hooks.is_empty());
}

#[tokio::test]
async fn test_hook_engine_register_disabled_hook() {
    let engine = HookEngine::new();

    let hook = test_hook_definition().disabled();
    engine.register(hook).await;

    let hooks = engine
        .get_hooks_for_point(HookPoint::BeforeToolExecution)
        .await;
    assert!(hooks.is_empty());
}

#[tokio::test]
async fn test_hook_engine_get_hooks_for_event() {
    let engine = HookEngine::new();

    let hook = HookDefinition::new(
        HookId::User(1),
        "event_hook".to_string(),
        HookTrigger::Event("custom_event".to_string()),
        HookAction::Log {
            message: "test".to_string(),
            level: "info".to_string(),
        },
    );
    engine.register(hook).await;

    let hooks = engine.get_hooks_for_event("custom_event").await;
    assert_eq!(hooks.len(), 1);

    let hooks = engine.get_hooks_for_event("other_event").await;
    assert!(hooks.is_empty());
}

#[tokio::test]
async fn test_hook_engine_get_hooks_for_command() {
    let engine = HookEngine::new();

    let hook = HookDefinition::new(
        HookId::User(1),
        "command_hook".to_string(),
        HookTrigger::Command("test_cmd".to_string()),
        HookAction::Log {
            message: "test".to_string(),
            level: "info".to_string(),
        },
    );
    engine.register(hook).await;

    let hooks = engine
        .get_hooks_for_point(HookPoint::AfterCommandCompleted)
        .await;
    assert!(hooks.is_empty());
}

#[tokio::test]
async fn test_hook_engine_trigger_executes_hooks() {
    let engine = HookEngine::new();

    let hook = HookDefinition::new(
        HookId::User(1),
        "trigger_test".to_string(),
        HookTrigger::HookPoint(HookPoint::BeforeToolExecution),
        HookAction::Log {
            message: "hook executed".to_string(),
            level: "info".to_string(),
        },
    );
    engine.register(hook).await;

    let event = HookEvent::ToolExecution {
        tool_name: "test_tool".to_string(),
    };
    let results = engine.trigger(HookPoint::BeforeToolExecution, event).await;

    assert_eq!(results.len(), 1);
    assert!(results[0].success);
    assert_eq!(results[0].output, Some("hook executed".to_string()));
}

#[tokio::test]
async fn test_hook_engine_trigger_with_no_matching_hooks() {
    let engine = HookEngine::new();

    let hook = HookDefinition::new(
        HookId::User(1),
        "other_point_hook".to_string(),
        HookTrigger::HookPoint(HookPoint::AfterToolExecution),
        HookAction::Log {
            message: "should not run".to_string(),
            level: "info".to_string(),
        },
    );
    engine.register(hook).await;

    let event = HookEvent::ToolExecution {
        tool_name: "test_tool".to_string(),
    };
    let results = engine.trigger(HookPoint::BeforeToolExecution, event).await;

    assert!(results.is_empty());
}

#[tokio::test]
async fn test_hook_engine_set_env_action() {
    let engine = HookEngine::new();

    let hook = HookDefinition::new(
        HookId::User(1),
        "set_env_hook".to_string(),
        HookTrigger::HookPoint(HookPoint::BeforeSessionStart),
        HookAction::SetEnv {
            key: "OPENCODE_TEST_VAR".to_string(),
            value: "test_value".to_string(),
        },
    );
    engine.register(hook).await;

    let event = HookEvent::SessionStart {
        session_id: "test_session".to_string(),
    };
    let results = engine.trigger(HookPoint::BeforeSessionStart, event).await;

    assert_eq!(results.len(), 1);
    assert!(results[0].success);
    assert_eq!(
        results[0].output,
        Some("Set OPENCODE_TEST_VAR=test_value".to_string())
    );

    std::env::remove_var("OPENCODE_TEST_VAR");
}

#[tokio::test]
async fn test_hook_engine_block_action() {
    let engine = HookEngine::new();

    let hook = HookDefinition::new(
        HookId::User(1),
        "block_hook".to_string(),
        HookTrigger::HookPoint(HookPoint::BeforeToolExecution),
        HookAction::Block {
            reason: "blocked for testing".to_string(),
        },
    );
    engine.register(hook).await;

    let event = HookEvent::ToolExecution {
        tool_name: "test_tool".to_string(),
    };
    let results = engine.trigger(HookPoint::BeforeToolExecution, event).await;

    assert_eq!(results.len(), 1);
    assert!(!results[0].success);
    assert!(results[0].error.is_some());
    assert!(results[0].error.as_ref().unwrap().contains("Hook blocked"));
}

#[tokio::test]
async fn test_hook_engine_notify_action() {
    let engine = HookEngine::new();

    let hook = HookDefinition::new(
        HookId::User(1),
        "notify_hook".to_string(),
        HookTrigger::HookPoint(HookPoint::AfterSessionStart),
        HookAction::Notify {
            message: "session started".to_string(),
        },
    );
    engine.register(hook).await;

    let event = HookEvent::SessionStart {
        session_id: "test_session".to_string(),
    };
    let results = engine.trigger(HookPoint::AfterSessionStart, event).await;

    assert_eq!(results.len(), 1);
    assert!(results[0].success);
    assert_eq!(results[0].output, Some("session started".to_string()));
}

#[tokio::test]
async fn test_hook_engine_timeout_short() {
    let engine = HookEngine::new();

    let hook = HookDefinition::new(
        HookId::User(1),
        "timeout_hook".to_string(),
        HookTrigger::HookPoint(HookPoint::BeforeSessionStart),
        HookAction::RunCommand {
            command: "sleep 10".to_string(),
        },
    )
    .with_timeout(100)
    .with_failure_policy(HookFailurePolicy::Log);

    engine.register(hook).await;

    let event = HookEvent::SessionStart {
        session_id: "test_session".to_string(),
    };
    let results = engine.trigger(HookPoint::BeforeSessionStart, event).await;

    assert_eq!(results.len(), 1);
    assert!(!results[0].success);
    assert_eq!(results[0].error, Some("Hook timed out".to_string()));
}

#[tokio::test]
async fn test_hook_engine_failure_policy_log() {
    let engine = HookEngine::new();

    let hook = HookDefinition::new(
        HookId::User(1),
        "fail_log_hook".to_string(),
        HookTrigger::HookPoint(HookPoint::BeforeToolExecution),
        HookAction::Block {
            reason: "intentional failure".to_string(),
        },
    )
    .with_failure_policy(HookFailurePolicy::Log);

    engine.register(hook).await;

    let event = HookEvent::ToolExecution {
        tool_name: "test_tool".to_string(),
    };
    let results = engine.trigger(HookPoint::BeforeToolExecution, event).await;

    assert!(!results[0].success);
}

#[tokio::test]
async fn test_hook_engine_execution_history() {
    let engine = HookEngine::new();

    let hook = HookDefinition::new(
        HookId::User(1),
        "history_hook".to_string(),
        HookTrigger::HookPoint(HookPoint::BeforeToolExecution),
        HookAction::Log {
            message: "test".to_string(),
            level: "info".to_string(),
        },
    );
    engine.register(hook).await;

    let event = HookEvent::ToolExecution {
        tool_name: "test_tool".to_string(),
    };
    engine.trigger(HookPoint::BeforeToolExecution, event).await;

    let history = engine.get_execution_history().await;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].hook_name, "history_hook");

    engine.clear_history().await;
    let history = engine.get_execution_history().await;
    assert!(history.is_empty());
}

#[tokio::test]
async fn test_hook_engine_multiple_hooks_same_point() {
    let engine = HookEngine::new();

    for i in 0..3 {
        let hook = HookDefinition::new(
            HookId::User(i),
            format!("hook_{}", i),
            HookTrigger::HookPoint(HookPoint::BeforeSessionStart),
            HookAction::Log {
                message: format!("hook {}", i),
                level: "info".to_string(),
            },
        );
        engine.register(hook).await;
    }

    let event = HookEvent::SessionStart {
        session_id: "test_session".to_string(),
    };
    let results = engine.trigger(HookPoint::BeforeSessionStart, event).await;

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.success));
}

#[tokio::test]
async fn test_hook_engine_from_domain_event_session_started() {
    let event = opencode_core::events::DomainEvent::SessionStarted("session_123".to_string());
    let hook_event = HookEngine::from_domain_event(&event).await;

    assert!(hook_event.is_some());
    match hook_event.unwrap() {
        HookEvent::SessionStart { session_id } => {
            assert_eq!(session_id, "session_123");
        }
        _ => panic!("Expected SessionStart event"),
    }
}

#[tokio::test]
async fn test_hook_engine_from_domain_event_session_ended() {
    let event = opencode_core::events::DomainEvent::SessionEnded("session_456".to_string());
    let hook_event = HookEngine::from_domain_event(&event).await;

    assert!(hook_event.is_some());
    match hook_event.unwrap() {
        HookEvent::SessionEnd { session_id } => {
            assert_eq!(session_id, "session_456");
        }
        _ => panic!("Expected SessionEnd event"),
    }
}

#[tokio::test]
async fn test_hook_engine_from_domain_event_tool_calls_return_none() {
    let event = opencode_core::events::DomainEvent::ToolCallStarted {
        session_id: "session_123".to_string(),
        tool_name: "test_tool".to_string(),
        call_id: "call_123".to_string(),
    };
    let hook_event = HookEngine::from_domain_event(&event).await;
    assert!(hook_event.is_none());
}

#[tokio::test]
async fn test_hook_engine_from_domain_event_llm_events_return_none() {
    let event = opencode_core::events::DomainEvent::LlmRequestStarted {
        session_id: "session_123".to_string(),
        provider: "openai".to_string(),
        model: "gpt-4".to_string(),
    };
    let hook_event = HookEngine::from_domain_event(&event).await;
    assert!(hook_event.is_none());
}

#[tokio::test]
async fn test_hook_engine_hook_definition_builder() {
    let hook = HookDefinition::new(
        HookId::Builtin(1),
        "builder_test".to_string(),
        HookTrigger::HookPoint(HookPoint::BeforeContextBuild),
        HookAction::Notify {
            message: "testing".to_string(),
        },
    )
    .with_description("Test description".to_string())
    .with_timeout(5000)
    .with_failure_policy(HookFailurePolicy::Warn);

    assert_eq!(hook.id, HookId::Builtin(1));
    assert_eq!(hook.name, "builder_test");
    assert_eq!(hook.description, Some("Test description".to_string()));
    assert_eq!(hook.timeout_ms, Some(5000));
    assert_eq!(hook.failure_policy, HookFailurePolicy::Warn);
    assert!(hook.enabled);
}

#[tokio::test]
async fn test_hook_engine_hook_definition_disabled() {
    let hook = HookDefinition::new(
        HookId::User(1),
        "should_be_disabled".to_string(),
        HookTrigger::HookPoint(HookPoint::AfterContextBuild),
        HookAction::Log {
            message: "test".to_string(),
            level: "info".to_string(),
        },
    )
    .disabled();

    assert!(!hook.enabled);
}

#[tokio::test]
async fn test_hook_engine_hook_result_struct() {
    let result = HookResult {
        hook_id: HookId::Plugin(1),
        success: true,
        output: Some("output".to_string()),
        error: None,
    };

    assert_eq!(result.hook_id, HookId::Plugin(1));
    assert!(result.success);
    assert_eq!(result.output, Some("output".to_string()));
    assert!(result.error.is_none());
}

#[tokio::test]
async fn test_hook_engine_hook_result_failure() {
    let result = HookResult {
        hook_id: HookId::User(1),
        success: false,
        output: None,
        error: Some("failed".to_string()),
    };

    assert!(!result.success);
    assert!(result.output.is_none());
    assert_eq!(result.error, Some("failed".to_string()));
}

#[test]
fn test_hook_id_cost_tier() {
    assert_eq!(HookId::Builtin(1).cost_tier(), 0);
    assert_eq!(HookId::Project(1).cost_tier(), 1);
    assert_eq!(HookId::User(1).cost_tier(), 2);
    assert_eq!(HookId::Plugin(1).cost_tier(), 3);
}

#[test]
fn test_hook_id_estimated_context_tokens() {
    assert_eq!(HookId::Builtin(1).estimated_context_tokens(), 0);
    assert_eq!(HookId::Project(1).estimated_context_tokens(), 50);
    assert_eq!(HookId::User(1).estimated_context_tokens(), 150);
    assert_eq!(HookId::Plugin(1).estimated_context_tokens(), 500);
}
