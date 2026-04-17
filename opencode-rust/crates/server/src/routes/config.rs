use crate::routes::error::json_error;
use crate::ServerState;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Responder};
use opencode_core::Config;

fn deep_merge(base: serde_json::Value, patch: serde_json::Value) -> serde_json::Value {
    match (base, patch) {
        (serde_json::Value::Object(mut base_map), serde_json::Value::Object(patch_map)) => {
            for (key, patch_value) in patch_map {
                let merged = match base_map.remove(&key) {
                    Some(base_value) => deep_merge(base_value, patch_value),
                    None => patch_value,
                };
                base_map.insert(key, merged);
            }
            serde_json::Value::Object(base_map)
        }
        (_, patch_value) => patch_value,
    }
}

pub async fn get_config(state: web::Data<ServerState>) -> impl Responder {
    let config = match state.config.read() {
        Ok(cfg) => cfg.clone(),
        Err(_) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "config_lock_error",
                "Failed to access server config",
            );
        }
    };

    HttpResponse::Ok().json(config)
}

pub async fn update_config(
    state: web::Data<ServerState>,
    req: web::Json<Config>,
) -> impl Responder {
    let current = match state.config.read() {
        Ok(cfg) => cfg.clone(),
        Err(_) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "config_lock_error",
                "Failed to access server config",
            );
        }
    };

    let base_json = match serde_json::to_value(&current) {
        Ok(v) => v,
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "config_serialize_error",
                e.to_string(),
            );
        }
    };
    let patch_json = match serde_json::to_value(req.into_inner()) {
        Ok(v) => v,
        Err(e) => {
            return json_error(
                StatusCode::BAD_REQUEST,
                "invalid_config_patch",
                e.to_string(),
            );
        }
    };

    let merged_json = deep_merge(base_json, patch_json);
    let merged_config: Config = match serde_json::from_value(merged_json) {
        Ok(cfg) => cfg,
        Err(e) => {
            return json_error(
                StatusCode::BAD_REQUEST,
                "invalid_config_patch",
                e.to_string(),
            );
        }
    };

    let config_path = Config::config_path();
    if let Err(e) = merged_config.save(&config_path) {
        return json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "config_save_error",
            e.to_string(),
        );
    }

    match state.config.write() {
        Ok(mut cfg) => {
            *cfg = merged_config.clone();
        }
        Err(_) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "config_lock_error",
                "Failed to update server config",
            );
        }
    }

    HttpResponse::Ok().json(merged_config)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_config));
    cfg.route("", web::patch().to(update_config));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deep_merge_nested_objects() {
        let base = serde_json::json!({
            "server": {"port": 8080, "host": "localhost"},
            "model": "gpt-4"
        });
        let patch = serde_json::json!({
            "server": {"port": 9090}
        });
        let result = deep_merge(base, patch);
        let obj = result.as_object().unwrap();
        assert_eq!(
            obj.get("server")
                .unwrap()
                .as_object()
                .unwrap()
                .get("port")
                .unwrap(),
            9090
        );
        assert_eq!(
            obj.get("server")
                .unwrap()
                .as_object()
                .unwrap()
                .get("host")
                .unwrap(),
            "localhost"
        );
        assert_eq!(obj.get("model").unwrap(), "gpt-4");
    }

    #[test]
    fn test_deep_merge_replaces_non_objects() {
        let base = serde_json::json!({"value": 123});
        let patch = serde_json::json!({"value": "string"});
        let result = deep_merge(base, patch);
        assert_eq!(result, "string");
    }

    #[test]
    fn test_deep_merge_adds_new_keys() {
        let base = serde_json::json!({"existing": true});
        let patch = serde_json::json!({"new_key": "added"});
        let result = deep_merge(base, patch);
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("existing"));
        assert!(obj.contains_key("new_key"));
        assert_eq!(obj.get("new_key").unwrap(), "added");
    }

    #[test]
    fn test_deep_merge_empty_patch() {
        let base = serde_json::json!({"key": "value"});
        let patch = serde_json::json!({});
        let result = deep_merge(base, patch);
        assert_eq!(result.as_object().unwrap().get("key").unwrap(), "value");
    }

    #[test]
    fn test_deep_merge_patch_wins_for_scalar() {
        let base = serde_json::json!(42);
        let patch = serde_json::json!("replaced");
        let result = deep_merge(base, patch);
        assert_eq!(result, "replaced");
    }

    #[test]
    fn test_deep_merge_array_replaced() {
        let base = serde_json::json!({"arr": [1, 2, 3]});
        let patch = serde_json::json!({"arr": ["a", "b"]});
        let result = deep_merge(base, patch);
        assert_eq!(result, serde_json::json!({"arr": ["a", "b"]}));
    }
}
