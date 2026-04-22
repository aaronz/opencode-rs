#[macro_export]
macro_rules! log_fields {
    () => {
        $crate::event::LogFields::default()
    };
    ($($field:ident),* $(,)?) => {
        {
            let mut fields = $crate::event::LogFields::default();
            $(
                fields.$field = ::std::option::Option::Some(::std::string::ToString::to_string(&::std::string:: stringify!($field)));
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
    }

    #[test]
    fn test_log_fields_named() {
        let fields = log_fields!(latency_ms = 42i64);
        assert_eq!(fields.latency_ms, Some(42));
    }
}
