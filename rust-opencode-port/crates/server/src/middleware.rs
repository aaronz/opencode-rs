use crate::ServerState;
use actix_cors::Cors;
use actix_web::dev::ServiceRequest;

pub struct AuthMiddleware;

pub fn cors_middleware(origins: &[String]) -> Cors {
    let mut cors = Cors::default()
        .allow_any_method()
        .allow_any_header()
        .max_age(3600);

    if origins.is_empty() {
        cors = cors.allow_any_origin();
    } else {
        for origin in origins {
            cors = cors.allowed_origin(origin);
        }
    }

    cors
}

pub fn is_api_key_authorized(req: &ServiceRequest) -> bool {
    let Some(state) = req.app_data::<actix_web::web::Data<ServerState>>() else {
        return true;
    };

    let config_guard = match state.config.read() {
        Ok(guard) => guard,
        Err(_) => return false,
    };

    let Some(expected_key) = config_guard.api_key.as_ref() else {
        return true;
    };

    if expected_key.is_empty() {
        return true;
    }

    req.headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|provided| provided == expected_key)
        .unwrap_or(false)
}
