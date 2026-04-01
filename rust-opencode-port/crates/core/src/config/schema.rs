use serde_json::Value;

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

pub fn get_official_schema_url() -> &'static str {
    "https://opencode.ai/config.json"
}
