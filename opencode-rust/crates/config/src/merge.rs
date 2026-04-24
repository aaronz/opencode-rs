use crate::Config;
use serde_json::{Map, Value};

#[derive(Debug, Clone)]
pub struct MergeTypeError {
    pub path: String,
    pub base_type: String,
    pub override_type: String,
}

fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

pub fn deep_merge(base: &Value, override_val: &Value) -> Result<Value, MergeTypeError> {
    deep_merge_with_path(base, override_val, "$".to_string())
}

fn deep_merge_with_path(
    base: &Value,
    override_val: &Value,
    path: String,
) -> Result<Value, MergeTypeError> {
    match (base, override_val) {
        (Value::Object(base_map), Value::Object(override_map)) => {
            let mut result = base_map.clone();
            for (key, override_value) in override_map {
                let base_value = result.get(key);
                let child_path = if path == "$" {
                    format!("$.{}", key)
                } else {
                    format!("{}.{}", path, key)
                };
                let merged = match base_value {
                    Some(base_val) => deep_merge_with_path(base_val, override_value, child_path)?,
                    None => override_value.clone(),
                };
                result.insert(key.clone(), merged);
            }
            Ok(Value::Object(result))
        }
        _ => {
            let base_type = value_type_name(base);
            let override_type = value_type_name(override_val);
            if base_type != override_type {
                Err(MergeTypeError {
                    path,
                    base_type: base_type.to_string(),
                    override_type: override_type.to_string(),
                })
            } else {
                Ok(override_val.clone())
            }
        }
    }
}

pub fn merge_configs(base: &Config, override_val: &Config) -> Result<Config, MergeTypeError> {
    // Direct serde_json::Value merge — no struct-level round-trip needed.
    // deep_merge handles nested objects recursively; arrays are replaced.
    let base_json = serde_json::to_value(base).unwrap_or(Value::Object(Map::new()));
    let override_json = serde_json::to_value(override_val).unwrap_or(Value::Object(Map::new()));

    let merged = deep_merge(&base_json, &override_json)?;

    serde_json::from_value(merged).map_err(|_| MergeTypeError {
        path: "$".to_string(),
        base_type: "object".to_string(),
        override_type: "config".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deep_merge_objects() {
        let base = json!({
            "server": {"port": 8080, "host": "localhost"},
            "model": "gpt-4"
        });
        let override_val = json!({
            "server": {"port": 3000},
            "small_model": "gpt-3.5"
        });

        let result = deep_merge(&base, &override_val).unwrap();

        assert_eq!(result["server"]["port"], 3000);
        assert_eq!(result["server"]["host"], "localhost");
        assert_eq!(result["model"], "gpt-4");
        assert_eq!(result["small_model"], "gpt-3.5");
    }

    #[test]
    fn test_deep_merge_arrays_replace() {
        let base = json!({"tags": ["a", "b"]});
        let override_val = json!({"tags": ["c"]});

        let result = deep_merge(&base, &override_val).unwrap();

        assert_eq!(result["tags"], json!(["c"]));
    }

    #[test]
    fn test_deep_merge_non_objects() {
        let base = json!({"key": "value"});
        let override_val = json!({"key": "new_value"});

        let result = deep_merge(&base, &override_val).unwrap();

        assert_eq!(result["key"], "new_value");
    }

    #[test]
    fn test_deep_merge_type_conflict_object_to_primitive() {
        let base = json!({"server": {"port": 3000}});
        let override_val = json!({"server": "not-an-object"});

        let result = deep_merge(&base, &override_val);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.path, "$.server");
        assert_eq!(err.base_type, "object");
        assert_eq!(err.override_type, "string");
    }

    #[test]
    fn test_deep_merge_type_conflict_primitive_to_object() {
        let base = json!({"server": "localhost"});
        let override_val = json!({"server": {"port": 3000}});

        let result = deep_merge(&base, &override_val);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.path, "$.server");
        assert_eq!(err.base_type, "string");
        assert_eq!(err.override_type, "object");
    }

    #[test]
    fn test_deep_merge_type_conflict_array_to_object() {
        let base = json!({"tags": ["a", "b"]});
        let override_val = json!({"tags": {"0": "a"}});

        let result = deep_merge(&base, &override_val);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.path, "$.tags");
    }

    #[test]
    fn test_deep_merge_type_conflict_nested() {
        let base = json!({"server": {"port": 3000}});
        let override_val = json!({"server": {"port": "not-a-number"}});

        let result = deep_merge(&base, &override_val);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.path, "$.server.port");
    }

    #[test]
    fn test_deep_merge_same_type_allowed() {
        let base = json!({"port": 3000});
        let override_val = json!({"port": 4000});

        let result = deep_merge(&base, &override_val).unwrap();
        assert_eq!(result["port"], 4000);
    }

    #[test]
    fn test_merge_configs_returns_error_on_type_conflict() {
        use crate::{Config, ServerConfig};

        let base = Config {
            server: Some(ServerConfig {
                port: Some(3000),
                hostname: Some("localhost".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let override_json: Value = json!({
            "server": {"port": "not-a-number"}
        });

        let base_json: Value = serde_json::to_value(&base).unwrap();
        let result = deep_merge(&base_json, &override_json);
        assert!(result.is_err());
    }
}
