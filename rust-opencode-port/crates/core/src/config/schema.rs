use serde_json::Value;

use super::{TuiConfig, ValidationError, ValidationSeverity};

pub fn validate_json_schema(config: &Value, _schema_url: &str) -> super::ValidationResult {
    let mut errors = Vec::new();

    if let Some(obj) = config.as_object() {
        if let Some(server) = obj.get("server") {
            if let Some(port) = server.get("port") {
                if let Some(p) = port.as_u64() {
                    if p == 0 || p > 65535 {
                        errors.push(super::ValidationError {
                            field: "server.port".to_string(),
                            message: "Port must be between 1 and 65535".to_string(),
                            severity: super::ValidationSeverity::Error,
                        });
                    }
                }
            }
        }

        if let Some(temp) = obj.get("temperature") {
            if let Some(t) = temp.as_f64() {
                if !(0.0..=2.0).contains(&t) {
                    errors.push(super::ValidationError {
                        field: "temperature".to_string(),
                        message: "Temperature must be between 0 and 2".to_string(),
                        severity: super::ValidationSeverity::Error,
                    });
                }
            }
        }
    }

    super::ValidationResult {
        valid: errors.is_empty(),
        errors,
    }
}

#[allow(dead_code)]
pub fn get_official_schema_url() -> &'static str {
    "https://opencode.ai/config.json"
}

#[allow(dead_code)]
pub fn validate_tui_config(tui_config: &TuiConfig) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if let Some(scroll_speed) = &tui_config.scroll_speed {
        if *scroll_speed == 0 {
            errors.push(ValidationError {
                field: "tui.scrollSpeed".to_string(),
                message: "scrollSpeed must be greater than 0".to_string(),
                severity: ValidationSeverity::Error,
            });
        }
        if *scroll_speed > 1000 {
            errors.push(ValidationError {
                field: "tui.scrollSpeed".to_string(),
                message: "scrollSpeed seems excessively high (max recommended: 1000)".to_string(),
                severity: ValidationSeverity::Warning,
            });
        }
    }

    if let Some(scroll_accel) = &tui_config.scroll_acceleration {
        if let Some(speed) = &scroll_accel.speed {
            if *speed < 0.0 {
                errors.push(ValidationError {
                    field: "tui.scrollAcceleration.speed".to_string(),
                    message: "scrollAcceleration.speed must be non-negative".to_string(),
                    severity: ValidationSeverity::Error,
                });
            }
            if *speed > 10.0 {
                errors.push(ValidationError {
                    field: "tui.scrollAcceleration.speed".to_string(),
                    message:
                        "scrollAcceleration.speed seems excessively high (max recommended: 10.0)"
                            .to_string(),
                    severity: ValidationSeverity::Warning,
                });
            }
        }
    }

    errors
}
