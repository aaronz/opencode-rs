#[macro_export]
macro_rules! log_fields {
    () => {
        $crate::event::LogFields::default()
    };
    ($($field:ident = $value:expr),*) => {
        {
            let mut fields = $crate::event::LogFields::default();
            $(
                fields.$field = Some($value.into());
            )*
            fields
        }
    };
}

#[macro_export]
macro_rules! log_tool {
    ($logger:expr, $tool:expr, $status:expr, $($field:tt)*) => {
        $logger.info(
            &format!("tool.{}", $tool),
            &format!("Tool {} completed", $status),
            $crate::log_fields!($($field)*)
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
                provider: Some($provider.to_string()),
                model: Some($model.to_string()),
                token_count: Some($tokens),
                latency_ms: Some($latency),
                ..Default::default()
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
        let fields = log_fields!(latency_ms = 42u64);
        assert_eq!(fields.latency_ms, Some(42));
    }
}
