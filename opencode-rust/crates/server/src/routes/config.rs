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
