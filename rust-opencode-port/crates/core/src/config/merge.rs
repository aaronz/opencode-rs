use crate::config::Config;
use serde_json::{Map, Value};

pub fn deep_merge(base: &Value, override_val: &Value) -> Value {
    match (base, override_val) {
        (Value::Object(base_map), Value::Object(override_map)) => {
            let mut result = base_map.clone();
            for (key, override_value) in override_map {
                let base_value = result.get(key);
                let merged = match base_value {
                    Some(base_val) => deep_merge(base_val, override_value),
                    None => override_value.clone(),
                };
                result.insert(key.clone(), merged);
            }
            Value::Object(result)
        }
        _ => override_val.clone(),
    }
}

pub fn merge_configs(base: &Config, override_val: &Config) -> Config {
    // Intentionally merge through serde_json::Value to preserve existing deep-merge
    // semantics across nested/flattened config structures without maintaining a
    // brittle field-by-field merger. Optimize to struct-level merging only when
    // we have exhaustive parity tests for all config fields.
    let base_json = serde_json::to_value(base).unwrap_or(Value::Object(Map::new()));
    let override_json = serde_json::to_value(override_val).unwrap_or(Value::Object(Map::new()));

    let merged = deep_merge(&base_json, &override_json);

    serde_json::from_value(merged).unwrap_or(Config::default())
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

        let result = deep_merge(&base, &override_val);

        assert_eq!(result["server"]["port"], 3000);
        assert_eq!(result["server"]["host"], "localhost");
        assert_eq!(result["model"], "gpt-4");
        assert_eq!(result["small_model"], "gpt-3.5");
    }

    #[test]
    fn test_deep_merge_arrays_replace() {
        let base = json!({"tags": ["a", "b"]});
        let override_val = json!({"tags": ["c"]});

        let result = deep_merge(&base, &override_val);

        assert_eq!(result["tags"], json!(["c"]));
    }

    #[test]
    fn test_deep_merge_non_objects() {
        let base = json!({"key": "value"});
        let override_val = json!({"key": "new_value"});

        let result = deep_merge(&base, &override_val);

        assert_eq!(result["key"], "new_value");
    }
}
