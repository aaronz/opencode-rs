use serde_json::Value;

pub(crate) struct SchemaValidator;

#[allow(dead_code)]
impl SchemaValidator {
    pub(crate) fn validate_required(args: &Value, required_fields: &[&str]) -> Result<(), String> {
        let mut missing = Vec::new();
        for field in required_fields {
            match args.get(field) {
                None => missing.push(*field),
                Some(Value::Null) => missing.push(*field),
                _ => {}
            }
        }
        if !missing.is_empty() {
            return Err(format!("Missing required field(s): {}", missing.join(", ")));
        }
        Ok(())
    }

    pub(crate) fn validate_string<'a>(args: &'a Value, field: &str) -> Result<&'a str, String> {
        match args.get(field) {
            Some(Value::String(s)) => Ok(s.as_str()),
            Some(v) => Err(format!("Field '{}' must be a string, got {}", field, v)),
            None => Err(format!("Missing required field '{}'", field)),
        }
    }

    pub(crate) fn validate_array<'a>(
        args: &'a Value,
        field: &str,
    ) -> Result<&'a Vec<Value>, String> {
        match args.get(field) {
            Some(Value::Array(a)) => Ok(a),
            Some(v) => Err(format!("Field '{}' must be an array, got {}", field, v)),
            None => Err(format!("Missing required field '{}'", field)),
        }
    }

    pub(crate) fn validate_object<'a>(
        args: &'a Value,
        field: &str,
    ) -> Result<&'a serde_json::Map<String, Value>, String> {
        match args.get(field) {
            Some(Value::Object(o)) => Ok(o),
            Some(v) => Err(format!("Field '{}' must be an object, got {}", field, v)),
            None => Err(format!("Missing required field '{}'", field)),
        }
    }

    pub(crate) fn validate_bool(args: &Value, field: &str) -> Result<bool, String> {
        match args.get(field) {
            Some(Value::Bool(b)) => Ok(*b),
            Some(v) => Err(format!("Field '{}' must be a boolean, got {}", field, v)),
            None => Err(format!("Missing required field '{}'", field)),
        }
    }

    pub(crate) fn validate_number(args: &Value, field: &str) -> Result<f64, String> {
        match args.get(field) {
            Some(Value::Number(n)) => n
                .as_f64()
                .ok_or_else(|| format!("Field '{}' is not a valid number", field)),
            Some(v) => Err(format!("Field '{}' must be a number, got {}", field, v)),
            None => Err(format!("Missing required field '{}'", field)),
        }
    }

    pub(crate) fn validate_optional_string<'a>(
        args: &'a Value,
        field: &str,
    ) -> Result<Option<&'a str>, String> {
        match args.get(field) {
            Some(Value::String(s)) => Ok(Some(s.as_str())),
            Some(Value::Null) | None => Ok(None),
            Some(v) => Err(format!(
                "Field '{}' must be a string or null, got {}",
                field, v
            )),
        }
    }

    pub(crate) fn validate_optional_number(
        args: &Value,
        field: &str,
    ) -> Result<Option<f64>, String> {
        match args.get(field) {
            Some(Value::Number(n)) => {
                Ok(Some(n.as_f64().ok_or_else(|| {
                    format!("Field '{}' is not a valid number", field)
                })?))
            }
            Some(Value::Null) | None => Ok(None),
            Some(v) => Err(format!(
                "Field '{}' must be a number or null, got {}",
                field, v
            )),
        }
    }

    pub(crate) fn validate_schema(args: &Value, schema: &ToolSchema) -> Result<(), String> {
        let mut errors: Vec<String> = Vec::new();

        for field in &schema.required {
            match args.get(field) {
                None | Some(Value::Null) => {
                    errors.push(format!("Missing required field: '{}'", field));
                }
                _ => {}
            }
        }

        for (field, expected_type) in &schema.properties {
            if let Some(val) = args.get(field) {
                if val != &Value::Null {
                    let actual_type = json_type_name(val);
                    if actual_type != expected_type.as_str() {
                        errors.push(format!(
                            "Field '{}' expected type '{}', got '{}'",
                            field, expected_type, actual_type
                        ));
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }
}

#[allow(dead_code)]
fn json_type_name(val: &Value) -> &'static str {
    match val {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[derive(Default)]
pub struct ToolSchema {
    pub required: Vec<String>,
    pub properties: Vec<(String, String)>,
}

impl ToolSchema {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn require(mut self, field: &str, field_type: &str) -> Self {
        self.required.push(field.to_string());
        self.properties
            .push((field.to_string(), field_type.to_string()));
        self
    }

    pub fn optional(mut self, field: &str, field_type: &str) -> Self {
        self.properties
            .push((field.to_string(), field_type.to_string()));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_required_pass() {
        let args = json!({"path": "/foo", "content": "bar"});
        assert!(SchemaValidator::validate_required(&args, &["path", "content"]).is_ok());
    }

    #[test]
    fn test_validate_required_missing() {
        let args = json!({"path": "/foo"});
        let result = SchemaValidator::validate_required(&args, &["path", "content"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("content"));
    }

    #[test]
    fn test_validate_string_ok() {
        let args = json!({"path": "/foo"});
        assert_eq!(
            SchemaValidator::validate_string(&args, "path").unwrap(),
            "/foo"
        );
    }

    #[test]
    fn test_validate_string_wrong_type() {
        let args = json!({"path": 42});
        assert!(SchemaValidator::validate_string(&args, "path").is_err());
    }

    #[test]
    fn test_schema_builder() {
        let schema = ToolSchema::new()
            .require("path", "string")
            .require("content", "string")
            .optional("encoding", "string");
        let args = json!({"path": "/foo", "content": "bar"});
        assert!(SchemaValidator::validate_schema(&args, &schema).is_ok());
    }

    #[test]
    fn test_schema_type_mismatch() {
        let schema = ToolSchema::new().require("count", "number");
        let args = json!({"count": "not a number"});
        let result = SchemaValidator::validate_schema(&args, &schema);
        assert!(result.is_err());
    }
}
