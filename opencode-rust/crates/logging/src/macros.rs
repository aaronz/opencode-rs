#[macro_export]
macro_rules! log_fields {
    () => {
        $crate::event::LogFields::default()
    };
    ($($field:ident),* $(,)?) => {
        {
            let mut fields = $crate::event::LogFields::default();
            $(
                fields.$field = ::std::option::Option::Some(::std::string::ToString::to_string(stringify!($field)));
            )*
            fields
        }
    };
    ($($field:ident = $value:expr),* $(,)?) => {
        {
            let mut fields = $crate::event::LogFields::default();
            $(
                fields.$field = ::std::option::Option::Some($value);
            )*
            fields
        }
    };
}

#[macro_export]
macro_rules! log_tool {
    ($logger:expr, $tool:expr, $status:expr) => {
        $logger.info(
            &format!("tool.{}", $tool),
            &format!("Tool {} completed", $status),
            {
                let mut fields = $crate::event::LogFields::default();
                fields.session_id = ::std::option::Option::Some($tool.to_string());
                fields.tool_name = ::std::option::Option::Some($tool.to_string());
                fields
            }
        )
    };
    ($logger:expr, $tool:expr, $status:expr, $($field:ident = $value:expr),* $(,)?) => {
        $logger.info(
            &format!("tool.{}", $tool),
            &format!("Tool {} completed", $status),
            {
                let mut fields = $crate::event::LogFields::default();
                fields.session_id = ::std::option::Option::Some($tool.to_string());
                fields.tool_name = ::std::option::Option::Some($tool.to_string());
                $(
                    fields.$field = ::std::option::Option::Some($value.into());
                )*
                fields
            }
        )
    };
}

#[macro_export]
macro_rules! log_llm {
    ($logger:expr, $provider:expr, $model:expr, $tokens:expr, $latency:expr, $status:expr) => {
        $logger.info(
            &format!("llm.{}", $provider),
            &format!("LLM request completed: {}", $status),
            $crate::event::LogFields {
                provider: ::std::option::Option::Some($provider.to_string()),
                model: ::std::option::Option::Some($model.to_string()),
                token_count: ::std::option::Option::Some($tokens),
                latency_ms: ::std::option::Option::Some($latency),
                ..$crate::event::LogFields::default()
            },
        )
    };
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_log_fields_empty() {
        let fields = log_fields!();
        assert_eq!(fields.session_id, None);
        assert_eq!(fields.tool_name, None);
    }

    #[test]
    fn test_log_fields_named() {
        let fields = log_fields!(latency_ms = 42i64);
        assert_eq!(fields.latency_ms, Some(42));
    }

    #[test]
    fn test_log_fields_field_only_form() {
        let fields = log_fields!(session_id);
        assert_eq!(fields.session_id, Some("session_id".to_string()));
    }

    #[test]
    fn test_log_fields_field_only_multiple_fields() {
        let fields = log_fields!(session_id, tool_name);
        assert_eq!(fields.session_id, Some("session_id".to_string()));
        assert_eq!(fields.tool_name, Some("tool_name".to_string()));
    }

    #[test]
    fn test_log_fields_field_only_all_string_fields() {
        let fields = log_fields!(session_id, tool_name, model, provider, error_code, file_path);
        assert_eq!(fields.session_id, Some("session_id".to_string()));
        assert_eq!(fields.tool_name, Some("tool_name".to_string()));
        assert_eq!(fields.model, Some("model".to_string()));
        assert_eq!(fields.provider, Some("provider".to_string()));
        assert_eq!(fields.error_code, Some("error_code".to_string()));
        assert_eq!(fields.file_path, Some("file_path".to_string()));
    }

    #[test]
    fn test_log_fields_key_value_string() {
        let fields = log_fields!(session_id = "my_session".to_string());
        assert_eq!(fields.session_id, Some("my_session".to_string()));
    }

    #[test]
    fn test_log_fields_key_value_i64() {
        let fields = log_fields!(latency_ms = 45i64);
        assert_eq!(fields.latency_ms, Some(45));
    }

    #[test]
    fn test_log_fields_key_value_multiple() {
        let fields = log_fields!(
            session_id = "sess_123".to_string(),
            tool_name = "read".to_string(),
            latency_ms = 100i64
        );
        assert_eq!(fields.session_id, Some("sess_123".to_string()));
        assert_eq!(fields.tool_name, Some("read".to_string()));
        assert_eq!(fields.latency_ms, Some(100));
    }

    #[test]
    fn test_log_fields_expression_evaluation() {
        let latency_value = 42i64;
        let session_name = "test_session".to_string();
        let fields = log_fields!(latency_ms = latency_value, session_id = session_name);
        assert_eq!(fields.latency_ms, Some(42));
        assert_eq!(fields.session_id, Some("test_session".to_string()));
    }

    #[test]
    fn test_log_fields_expression_with_function_call() {
        fn get_session_id() -> String {
            "generated_session".to_string()
        }
        let fields = log_fields!(session_id = get_session_id());
        assert_eq!(fields.session_id, Some("generated_session".to_string()));
    }

    #[test]
    fn test_log_fields_expression_with_arithmetic() {
        let base_latency = 50i64;
        let fields = log_fields!(latency_ms = base_latency + 10);
        assert_eq!(fields.latency_ms, Some(60));
    }

    #[test]
    fn test_log_fields_all_optional_fields() {
        let fields = log_fields!(
            session_id = "sess_all".to_string(),
            tool_name = "all_fields_tool".to_string(),
            latency_ms = 200i64,
            model = "gpt-5".to_string(),
            provider = "openai".to_string(),
            token_count = 5000i64,
            error_code = "ERR_OK".to_string(),
            file_path = "/path/to/file.rs".to_string(),
            line = 42u32
        );
        assert_eq!(fields.session_id, Some("sess_all".to_string()));
        assert_eq!(fields.tool_name, Some("all_fields_tool".to_string()));
        assert_eq!(fields.latency_ms, Some(200));
        assert_eq!(fields.model, Some("gpt-5".to_string()));
        assert_eq!(fields.provider, Some("openai".to_string()));
        assert_eq!(fields.token_count, Some(5000));
        assert_eq!(fields.error_code, Some("ERR_OK".to_string()));
        assert_eq!(fields.file_path, Some("/path/to/file.rs".to_string()));
        assert_eq!(fields.line, Some(42));
    }

    #[test]
    fn test_log_fields_trailing_comma_field_only() {
        let fields = log_fields!(session_id, tool_name,);
        assert_eq!(fields.session_id, Some("session_id".to_string()));
        assert_eq!(fields.tool_name, Some("tool_name".to_string()));
    }

    #[test]
    fn test_log_fields_trailing_comma_key_value() {
        let fields = log_fields!(session_id = "sess".to_string(), latency_ms = 10i64,);
        assert_eq!(fields.session_id, Some("sess".to_string()));
        assert_eq!(fields.latency_ms, Some(10));
    }

    #[test]
    fn test_log_fields_returns_log_fields_type() {
        use crate::event::LogFields;
        let fields: LogFields = log_fields!(session_id);
        assert!(matches!(fields, LogFields { .. }));
    }

    #[test]
    fn test_log_fields_field_only_does_not_set_other_fields() {
        let fields = log_fields!(session_id);
        assert_eq!(fields.session_id, Some("session_id".to_string()));
        assert_eq!(fields.tool_name, None);
        assert_eq!(fields.latency_ms, None);
        assert_eq!(fields.model, None);
        assert_eq!(fields.provider, None);
        assert_eq!(fields.token_count, None);
        assert_eq!(fields.error_code, None);
        assert_eq!(fields.file_path, None);
        assert_eq!(fields.line, None);
        assert!(fields.extra.is_empty());
    }

    #[test]
    fn test_log_fields_key_value_does_not_set_other_fields() {
        let fields = log_fields!(latency_ms = 42i64);
        assert_eq!(fields.latency_ms, Some(42));
        assert_eq!(fields.session_id, None);
        assert_eq!(fields.tool_name, None);
        assert_eq!(fields.model, None);
        assert!(fields.extra.is_empty());
    }
}
