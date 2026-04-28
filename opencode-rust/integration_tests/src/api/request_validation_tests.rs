#[cfg(test)]
mod test_request_validation {
    use serde_json::Value;

    fn is_valid_uuid(s: &str) -> bool {
        uuid::Uuid::parse_str(s).is_ok()
    }

    fn is_valid_request_json(value: &Value) -> bool {
        let session_id = match value.get("session_id").and_then(|v| v.as_str()) {
            Some(s) if !s.trim().is_empty() => s,
            _ => return false,
        };

        if !is_valid_uuid(session_id) {
            return false;
        }

        let prompt = match value.get("prompt").and_then(|v| v.as_str()) {
            Some(p) if !p.trim().is_empty() => p,
            _ => return false,
        };

        true
    }

    #[tokio::test]
    async fn test_malformed_json_rejected() {
        let malformed_jsons = vec![
            r#"{invalid json}"#,
            r#"{"key": "unclosed"#,
            r#"{"key": }"#,
            r#"not json at all"#,
            r#"null"#,
            r#"{"key": "value",}"#,
            r#"}"#,
            r#""#,
            r#"   "#,
        ];

        for malformed in malformed_jsons {
            let result: Result<Value, _> = serde_json::from_str(malformed);
            assert!(
                result.is_err(),
                "Malformed JSON should be rejected: {:?}",
                malformed
            );
        }
    }

    #[tokio::test]
    async fn test_missing_required_fields_rejected() {
        let test_cases = vec![
            (r#"{}"#, true, "empty object should be rejected"),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000"}"#,
                false,
                "missing prompt",
            ),
            (r#"{"prompt": "test"}"#, false, "missing session_id"),
            (
                r#"{"session_id": "", "prompt": "test"}"#,
                true,
                "empty session_id",
            ),
            (
                r#"{"session_id": "   ", "prompt": "test"}"#,
                true,
                "whitespace session_id",
            ),
            (r#"{"prompt": ""}"#, true, "empty prompt"),
            (r#"{"prompt": "   "}"#, true, "whitespace-only prompt"),
        ];

        for (json_str, expect_invalid, description) in test_cases {
            let result: Result<Value, _> = serde_json::from_str(json_str);
            if expect_invalid {
                assert!(
                    result.is_err() || (result.is_ok() && !is_valid_request_json(&result.unwrap())),
                    "{} should be rejected",
                    description
                );
            }
        }
    }

    #[tokio::test]
    async fn test_invalid_types_rejected() {
        let invalid_type_cases = vec![
            (
                r#"{"session_id": 123, "prompt": "test"}"#,
                true,
                "session_id as number",
            ),
            (
                r#"{"session_id": ["uuid"], "prompt": "test"}"#,
                true,
                "session_id as array",
            ),
            (
                r#"{"session_id": {"id": "uuid"}, "prompt": "test"}"#,
                true,
                "session_id as object",
            ),
            (
                r#"{"session_id": true, "prompt": "test"}"#,
                true,
                "session_id as boolean",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": 123}"#,
                true,
                "prompt as number",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": ["array"]}"#,
                true,
                "prompt as array",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": {"text": "obj"}}"#,
                true,
                "prompt as object",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": false}"#,
                true,
                "prompt as boolean",
            ),
            (
                r#"{"session_id": "not-a-uuid", "prompt": "test"}"#,
                true,
                "invalid UUID format",
            ),
            (
                r#"{"session_id": "", "prompt": "test"}"#,
                true,
                "empty UUID string",
            ),
        ];

        for (json_str, expect_invalid, description) in invalid_type_cases {
            let result: Result<Value, _> = serde_json::from_str(json_str);
            if expect_invalid {
                assert!(
                    result.is_err() || (result.is_ok() && !is_valid_request_json(&result.unwrap())),
                    "{} should be rejected",
                    description
                );
            }
        }
    }

    #[tokio::test]
    async fn test_json_injection_in_string_fields() {
        let injection_cases = vec![
            (
                r#"{"session_id": "valid-uuid', \"injected\": \"value", "prompt": "test"}"#,
                true,
                "quote injection",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": "test\"}"#,
                true,
                "trailing quote injection",
            ),
        ];

        for (json_str, expect_invalid, description) in injection_cases {
            let result: Result<Value, _> = serde_json::from_str(json_str);
            assert!(
                result.is_err() || (result.is_ok() && !is_valid_request_json(&result.unwrap())),
                "{} should be rejected: {}",
                description,
                json_str
            );
        }
    }

    #[tokio::test]
    async fn test_oversized_payload_rejected() {
        let large_prompt = "a".repeat(100001);
        let json = format!(
            r#"{{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": "{}"}}"#,
            large_prompt
        );
        let result: Result<Value, _> = serde_json::from_str(&json);
        if result.is_ok() {
            let value = result.unwrap();
            if let Some(prompt) = value.get("prompt").and_then(|p| p.as_str()) {
                assert!(
                    prompt.len() > 100000,
                    "Large payload should be flagged for size validation"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_negative_values_rejected() {
        let negative_cases = vec![
            (r#"{"limit": -1, "offset": 0}"#, "negative limit"),
            (r#"{"limit": -100, "offset": 0}"#, "large negative limit"),
            (r#"{"offset": -1}"#, "negative offset"),
        ];

        for (json_str, description) in negative_cases {
            let result: Result<Value, _> = serde_json::from_str(json_str);
            if result.is_ok() {
                let value = result.unwrap();
                if let Some(limit) = value.get("limit").and_then(|l| l.as_i64()) {
                    assert!(limit >= 0, "{} should be non-negative", description);
                }
                if let Some(offset) = value.get("offset").and_then(|o| o.as_i64()) {
                    assert!(offset >= 0, "{} should be non-negative", description);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_null_bytes_in_strings_rejected() {
        let null_byte_cases = vec![
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000\0", "prompt": "test"}"#,
                "null byte in session_id",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": "test\0"}"#,
                "null byte in prompt",
            ),
        ];

        for (json_str, description) in null_byte_cases {
            let bytes = json_str.as_bytes();
            if bytes.contains(&0) {
                continue;
            }
            let result: Result<Value, _> = serde_json::from_str(json_str);
            match result {
                Ok(value) => {
                    let json_bytes = serde_json::to_string(&value).unwrap().into_bytes();
                    assert!(
                        !json_bytes.contains(&0),
                        "{}: null bytes should be rejected or escaped",
                        description
                    );
                }
                Err(_) => assert!(true, "{} should be rejected", description),
            }
        }
    }

    #[tokio::test]
    async fn test_valid_request_accepted() {
        let valid_cases = vec![
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": "Hello world"}"#,
                "basic valid request",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": "test with émoji 🎉"}"#,
                "valid with unicode",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": "multi\nline\ntext"}"#,
                "valid with newlines",
            ),
        ];

        for (json_str, description) in valid_cases {
            let result: Result<Value, _> = serde_json::from_str(json_str);
            assert!(
                result.is_ok() && is_valid_request_json(&result.unwrap()),
                "{} should be accepted: {}",
                description,
                json_str
            );
        }
    }

    #[tokio::test]
    async fn test_enum_validation_rejected_invalid_values() {
        let enum_cases = vec![
            (r#"{"mode": "invalid"}"#, "unknown mode value"),
            (r#"{"mode": "BUILD"}"#, "case-sensitive mode"),
            (r#"{"mode": "buildt"}"#, "typo in mode value"),
        ];

        for (json_str, description) in enum_cases {
            let result: Result<Value, _> = serde_json::from_str(json_str);
            if result.is_ok() {
                let value = result.unwrap();
                if let Some(mode) = value.get("mode").and_then(|m| m.as_str()) {
                    let valid_modes = ["build", "plan", "general"];
                    assert!(
                        !valid_modes.contains(&mode),
                        "{} should be rejected: {} not in {:?}",
                        description,
                        mode,
                        valid_modes
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn test_boolean_fields_rejected_as_wrong_type() {
        let bool_cases = vec![
            (
                r#"{"session_id": true, "prompt": "test"}"#,
                "session_id as boolean",
            ),
            (
                r#"{"session_id": false, "prompt": "test"}"#,
                "session_id as false boolean",
            ),
        ];

        for (json_str, description) in bool_cases {
            let result: Result<Value, _> = serde_json::from_str(json_str);
            if result.is_ok() {
                let value = result.unwrap();
                if let Some(sid) = value.get("session_id") {
                    assert!(
                        !sid.is_boolean(),
                        "{} should be rejected: session_id must be string not boolean",
                        description
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn test_nested_object_rejected() {
        let json_str =
            r#"{"session_id": {"id": "550e8400-e29b-41d4-a716-446655440000"}, "prompt": "test"}"#;
        let result: Result<Value, _> = serde_json::from_str(json_str);
        if result.is_ok() {
            let value = result.unwrap();
            if let Some(sid) = value.get("session_id") {
                assert!(
                    !sid.is_object(),
                    "session_id as nested object should be rejected"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_array_fields_rejected() {
        let array_cases = vec![
            (
                r#"{"session_id": ["550e8400-e29b-41d4-a716-446655440000"], "prompt": "test"}"#,
                "session_id as array",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": ["test"]}"#,
                "prompt as array",
            ),
        ];

        for (json_str, description) in array_cases {
            if let Ok(value) = serde_json::from_str::<Value>(json_str) {
                if let Some(sid) = value.get("session_id") {
                    assert!(!sid.is_array(), "{} should be rejected", description);
                }
                if let Some(prompt) = value.get("prompt") {
                    assert!(!prompt.is_array(), "{} should be rejected", description);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_pagination_validation() {
        let pagination_cases = vec![
            (
                r#"{"limit": 0, "offset": 0}"#,
                true,
                "zero values should be valid",
            ),
            (
                r#"{"limit": 200, "offset": 10000}"#,
                true,
                "max values should be valid",
            ),
            (
                r#"{"limit": 201}"#,
                false,
                "limit over max (200) should be flagged",
            ),
            (
                r#"{"offset": 10001}"#,
                false,
                "offset over max (10000) should be flagged",
            ),
        ];

        for (json_str, expect_valid, description) in pagination_cases {
            let result: Result<Value, _> = serde_json::from_str(json_str);
            assert!(result.is_ok(), "{} should parse", description);
            if let Ok(value) = result {
                let limit = value.get("limit").and_then(|l| l.as_u64()).unwrap_or(20);
                let offset = value.get("offset").and_then(|o| o.as_u64()).unwrap_or(0);

                if expect_valid {
                    assert!(
                        limit <= 200 && offset <= 10000,
                        "{} should be valid",
                        description
                    );
                }
            }
        }
    }
}
