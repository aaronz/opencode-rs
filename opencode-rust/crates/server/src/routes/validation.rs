use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub error_code: u16,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            error_code: 7001,
        }
    }

    pub fn required(field: impl Into<String>) -> Self {
        let f = field.into();
        Self {
            field: f.clone(),
            message: format!("Field '{}' is required", f),
            error_code: 7002,
        }
    }

    pub fn format(field: impl Into<String>, expected: impl Into<String>) -> Self {
        let f = field.into();
        Self {
            field: f.clone(),
            message: format!(
                "Field '{}' has invalid format. Expected: {}",
                f,
                expected.into()
            ),
            error_code: 7003,
        }
    }

    pub fn out_of_range(field: impl Into<String>, min: Option<usize>, max: Option<usize>) -> Self {
        let f = field.into();
        let message = match (min, max) {
            (Some(min), Some(max)) => format!("Field '{}' must be between {} and {}", f, min, max),
            (Some(min), None) => format!("Field '{}' must be at least {}", f, min),
            (None, Some(max)) => format!("Field '{}' must be at most {}", f, max),
            (None, None) => format!("Field '{}' has invalid value", f),
        };
        Self {
            field: f,
            message,
            error_code: 7001,
        }
    }

    pub fn too_long(field: impl Into<String>, max_length: usize) -> Self {
        let f = field.into();
        Self {
            field: f.clone(),
            message: format!(
                "Field '{}' exceeds maximum length of {} characters",
                f, max_length
            ),
            error_code: 7001,
        }
    }

    pub fn too_short(field: impl Into<String>, min_length: usize) -> Self {
        let f = field.into();
        Self {
            field: f.clone(),
            message: format!("Field '{}' must be at least {} characters", f, min_length),
            error_code: 7001,
        }
    }

    pub fn invalid_enum(field: impl Into<String>, valid_values: &[&str]) -> Self {
        let f = field.into();
        Self {
            field: f.clone(),
            message: format!(
                "Field '{}' has invalid value. Valid values: {}",
                f,
                valid_values.join(", ")
            ),
            error_code: 7001,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationErrors {
    pub error: String,
    pub message: String,
    pub code: u16,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<ValidationError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self {
            error: "validation_error".to_string(),
            message: "Request validation failed".to_string(),
            code: 422,
            details: Vec::new(),
        }
    }

    pub fn add(&mut self, error: ValidationError) {
        self.details.push(error);
    }

    pub fn add_if(&mut self, condition: bool, error: ValidationError) {
        if condition {
            self.add(error);
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.details.is_empty()
    }

    pub fn len(&self) -> usize {
        self.details.len()
    }

    pub fn to_response(&self) -> HttpResponse {
        if self.details.is_empty() {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "validation_error",
                "message": "Request validation failed",
                "code": 422
            }));
        }

        if self.details.len() == 1 {
            let err = &self.details[0];
            return HttpResponse::UnprocessableEntity().json(serde_json::json!({
                "error": "validation_error",
                "message": format!("Invalid value for '{}': {}", err.field, err.message),
                "code": err.error_code,
                "field": err.field
            }));
        }

        HttpResponse::UnprocessableEntity().json(serde_json::json!({
            "error": self.error,
            "message": format!("Request validation failed with {} errors", self.details.len()),
            "code": self.code,
            "details": self.details
        }))
    }
}

impl Default for ValidationErrors {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Validation error: {} errors", self.details.len())
    }
}

impl ResponseError for ValidationErrors {
    fn error_response(&self) -> HttpResponse {
        self.to_response()
    }
}

pub struct StringValidator<'a> {
    field: &'a str,
    value: &'a str,
    errors: &'a mut ValidationErrors,
}

impl<'a> StringValidator<'a> {
    pub fn new(field: &'a str, value: &'a str, errors: &'a mut ValidationErrors) -> Self {
        Self {
            field,
            value,
            errors,
        }
    }

    pub fn not_empty(self) -> Self {
        self.errors.add_if(
            self.value.trim().is_empty(),
            ValidationError::new(self.field, "cannot be empty"),
        );
        self
    }

    pub fn min_length(self, min: usize) -> Self {
        self.errors.add_if(
            self.value.len() < min,
            ValidationError::too_short(self.field, min),
        );
        self
    }

    pub fn max_length(self, max: usize) -> Self {
        self.errors.add_if(
            self.value.len() > max,
            ValidationError::too_long(self.field, max),
        );
        self
    }

    pub fn matches(self, pattern: &str, description: &str) -> Self {
        let regex = regex_lite::Regex::new(pattern).ok();
        if let Some(re) = regex {
            self.errors.add_if(
                !re.is_match(self.value),
                ValidationError::format(self.field, description),
            );
        }
        self
    }

    pub fn valid_uuid(self) -> Self {
        self.errors.add_if(
            uuid::Uuid::parse_str(self.value).is_err(),
            ValidationError::format(self.field, "valid UUID"),
        );
        self
    }

    pub fn valid_url(self) -> Self {
        self.errors.add_if(
            url::Url::parse(self.value).is_err(),
            ValidationError::format(self.field, "valid URL"),
        );
        self
    }
}

pub struct OptionalStringValidator<'a> {
    field: &'a str,
    value: Option<&'a str>,
    errors: &'a mut ValidationErrors,
}

impl<'a> OptionalStringValidator<'a> {
    pub fn new(field: &'a str, value: Option<&'a str>, errors: &'a mut ValidationErrors) -> Self {
        Self {
            field,
            value,
            errors,
        }
    }

    pub fn if_present_matches(self, pattern: &str, description: &str) -> Self {
        if let Some(v) = self.value {
            let regex = regex_lite::Regex::new(pattern).ok();
            if let Some(re) = regex {
                self.errors.add_if(
                    !re.is_match(v),
                    ValidationError::format(self.field, description),
                );
            }
        }
        self
    }

    pub fn if_present_valid_uuid(self) -> Self {
        if let Some(v) = self.value {
            self.errors.add_if(
                uuid::Uuid::parse_str(v).is_err(),
                ValidationError::format(self.field, "valid UUID"),
            );
        }
        self
    }

    pub fn if_present_valid_url(self) -> Self {
        if let Some(v) = self.value {
            self.errors.add_if(
                url::Url::parse(v).is_err(),
                ValidationError::format(self.field, "valid URL"),
            );
        }
        self
    }
}

pub struct NumberValidator<T> {
    field: String,
    value: T,
    errors: *mut ValidationErrors,
}

impl<T> NumberValidator<T> {
    pub fn new(field: String, value: T, errors: &mut ValidationErrors) -> Self {
        Self {
            field,
            value,
            errors,
        }
    }
}

impl NumberValidator<usize> {
    pub fn min(self, min: usize) -> Self {
        unsafe {
            (&mut *self.errors).add_if(
                self.value < min,
                ValidationError::out_of_range(&self.field, Some(min), None),
            );
        }
        self
    }

    pub fn max(self, max: usize) -> Self {
        unsafe {
            (&mut *self.errors).add_if(
                self.value > max,
                ValidationError::out_of_range(&self.field, None, Some(max)),
            );
        }
        self
    }

    pub fn range(self, min: usize, max: usize) -> Self {
        self.min(min).max(max)
    }
}

pub struct RequestValidator {
    errors: ValidationErrors,
}

impl RequestValidator {
    pub fn new() -> Self {
        Self {
            errors: ValidationErrors::new(),
        }
    }

    pub fn errors(&self) -> &ValidationErrors {
        &self.errors
    }

    pub fn errors_mut(&mut self) -> &mut ValidationErrors {
        &mut self.errors
    }

    pub fn is_valid(&self) -> bool {
        !self.errors.has_errors()
    }

    pub fn validate_required_string(&mut self, field: &str, value: Option<&str>) {
        let Some(v) = value else {
            self.errors.add(ValidationError::required(field));
            return;
        };
        StringValidator::new(field, v, &mut self.errors)
            .not_empty()
            .max_length(10000);
    }

    pub fn validate_optional_string(
        &mut self,
        field: &str,
        value: Option<&str>,
        max_length: usize,
    ) {
        if let Some(v) = value {
            StringValidator::new(field, v, &mut self.errors).max_length(max_length);
        }
    }

    pub fn validate_required_uuid(&mut self, field: &str, value: Option<&str>) {
        let Some(v) = value else {
            self.errors.add(ValidationError::required(field));
            return;
        };
        StringValidator::new(field, v, &mut self.errors).valid_uuid();
    }

    pub fn validate_required_number(
        &mut self,
        field: &str,
        value: Option<usize>,
        min: usize,
        max: usize,
    ) {
        let Some(v) = value else {
            self.errors.add(ValidationError::required(field));
            return;
        };
        NumberValidator::new(field.to_string(), v, &mut self.errors).range(min, max);
    }

    pub fn validate_optional_number(
        &mut self,
        field: &str,
        value: Option<usize>,
        min: usize,
        max: usize,
    ) {
        if let Some(v) = value {
            NumberValidator::new(field.to_string(), v, &mut self.errors).range(min, max);
        }
    }

    pub fn validate_enum(&mut self, field: &str, value: &str, allowed_values: &[&str]) {
        if !allowed_values.iter().any(|v| *v == value) {
            self.errors
                .add(ValidationError::invalid_enum(field, allowed_values));
        }
    }

    pub fn validate(self) -> Result<(), ValidationErrors> {
        if self.errors.has_errors() {
            Err(self.errors)
        } else {
            Ok(())
        }
    }
}

impl Default for RequestValidator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn validate_session_id(id: &str) -> Result<uuid::Uuid, ValidationErrors> {
    let mut errors = ValidationErrors::new();
    StringValidator::new("session_id", id, &mut errors).valid_uuid();
    if errors.has_errors() {
        Err(errors)
    } else {
        Ok(uuid::Uuid::parse_str(id).unwrap())
    }
}

pub fn validate_message_index(index: usize, max: usize) -> Result<(), ValidationErrors> {
    let mut errors = ValidationErrors::new();
    NumberValidator::new("message_index".to_string(), index, &mut errors)
        .range(0, max.saturating_sub(1));
    if errors.has_errors() {
        Err(errors)
    } else {
        Ok(())
    }
}

pub fn validate_pagination(limit: Option<usize>, offset: Option<usize>) -> (usize, usize) {
    let limit = limit.unwrap_or(20);
    let offset = offset.unwrap_or(0);
    (limit.min(200), offset.min(10000))
}

pub fn validate_command(
    command: &str,
    args: Option<&Vec<String>>,
    workdir: Option<&str>,
) -> Result<(), ValidationErrors> {
    let mut errors = ValidationErrors::new();

    StringValidator::new("command", command, &mut errors)
        .not_empty()
        .max_length(1000);

    if let Some(args) = args {
        for (i, arg) in args.iter().enumerate() {
            StringValidator::new(&format!("args[{}]", i), arg, &mut errors).max_length(10000);
        }
    }

    if let Some(workdir) = workdir {
        StringValidator::new("workdir", workdir, &mut errors).max_length(1000);
    }

    if errors.has_errors() {
        Err(errors)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_creation() {
        let err = ValidationError::new("email", "invalid format");
        assert_eq!(err.field, "email");
        assert_eq!(err.message, "invalid format");
        assert_eq!(err.error_code, 7001);
    }

    #[test]
    fn test_validation_error_required() {
        let err = ValidationError::required("name");
        assert_eq!(err.field, "name");
        assert!(err.message.contains("required"));
        assert_eq!(err.error_code, 7002);
    }

    #[test]
    fn test_validation_error_format() {
        let err = ValidationError::format("email", "valid email");
        assert_eq!(err.field, "email");
        assert!(err.message.contains("invalid format"));
        assert!(err.message.contains("valid email"));
        assert_eq!(err.error_code, 7003);
    }

    #[test]
    fn test_validation_errors_single() {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new("field1", "error1"));

        let resp = errors.to_response();
        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[test]
    fn test_validation_errors_multiple() {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new("field1", "error1"));
        errors.add(ValidationError::new("field2", "error2"));

        let resp = errors.to_response();
        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[test]
    fn test_string_validator_not_empty() {
        let mut errors = ValidationErrors::new();
        StringValidator::new("name", "", &mut errors).not_empty();
        assert!(errors.has_errors());

        let mut errors = ValidationErrors::new();
        StringValidator::new("name", "  ", &mut errors).not_empty();
        assert!(errors.has_errors());

        let mut errors = ValidationErrors::new();
        StringValidator::new("name", "John", &mut errors).not_empty();
        assert!(!errors.has_errors());
    }

    #[test]
    fn test_string_validator_length() {
        let mut errors = ValidationErrors::new();
        StringValidator::new("name", "abc", &mut errors)
            .min_length(5)
            .max_length(2);
        assert!(errors.has_errors());
    }

    #[test]
    fn test_string_validator_uuid() {
        let mut errors = ValidationErrors::new();
        StringValidator::new("id", "not-a-uuid", &mut errors).valid_uuid();
        assert!(errors.has_errors());

        let mut errors = ValidationErrors::new();
        StringValidator::new("id", "550e8400-e29b-41d4-a716-446655440000", &mut errors)
            .valid_uuid();
        assert!(!errors.has_errors());
    }

    #[test]
    fn test_pagination_defaults() {
        let (limit, offset) = validate_pagination(None, None);
        assert_eq!(limit, 20);
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_pagination_clamping() {
        let (limit, offset) = validate_pagination(Some(500), Some(20000));
        assert_eq!(limit, 200);
        assert_eq!(offset, 10000);
    }

    #[test]
    fn test_string_validator_empty_string() {
        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "", &mut errors).not_empty();
        assert!(errors.has_errors());
        assert_eq!(errors.len(), 1);
        assert!(errors.details[0].message.contains("cannot be empty"));
    }

    #[test]
    fn test_string_validator_whitespace_only() {
        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "   \t\n  ", &mut errors).not_empty();
        assert!(errors.has_errors());
    }

    #[test]
    fn test_string_validator_max_length_boundary() {
        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "abc", &mut errors).max_length(3);
        assert!(!errors.has_errors());

        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "abcd", &mut errors).max_length(3);
        assert!(errors.has_errors());
    }

    #[test]
    fn test_string_validator_min_length_boundary() {
        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "abc", &mut errors).min_length(3);
        assert!(!errors.has_errors());

        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "ab", &mut errors).min_length(3);
        assert!(errors.has_errors());
    }

    #[test]
    fn test_string_validator_special_characters() {
        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "hello world!@#$%^&*()", &mut errors)
            .min_length(1)
            .max_length(100);
        assert!(!errors.has_errors());

        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "nöüe", &mut errors)
            .min_length(1)
            .max_length(100);
        assert!(!errors.has_errors());

        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "🎉🔥💻", &mut errors)
            .min_length(1)
            .max_length(100);
        assert!(!errors.has_errors());
    }

    #[test]
    fn test_string_validator_unicode() {
        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "中文测试", &mut errors)
            .min_length(1)
            .max_length(100);
        assert!(!errors.has_errors());

        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "עברית", &mut errors)
            .min_length(1)
            .max_length(100);
        assert!(!errors.has_errors());
    }

    #[test]
    fn test_string_validator_regex_pattern() {
        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "abc123", &mut errors)
            .matches(r"^[a-z0-9]+$", "lowercase alphanumeric");
        assert!(!errors.has_errors());

        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "ABC123", &mut errors)
            .matches(r"^[a-z0-9]+$", "lowercase alphanumeric");
        assert!(errors.has_errors());
    }

    #[test]
    fn test_string_validator_invalid_regex() {
        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "value", &mut errors).matches(r"[invalid", "valid pattern");
        assert!(!errors.has_errors());
    }

    #[test]
    fn test_string_validator_uuid_variants() {
        let valid_uuids = vec![
            "550e8400-e29b-41d4-a716-446655440000",
            "00000000-0000-0000-0000-000000000000",
            "ffffffff-ffff-ffff-ffff-ffffffffffff",
            "123e4567-e89b-12d3-a456-426614174000",
        ];
        for uuid in valid_uuids {
            let mut errors = ValidationErrors::new();
            StringValidator::new("id", uuid, &mut errors).valid_uuid();
            assert!(!errors.has_errors(), "UUID {} should be valid", uuid);
        }

        let invalid_uuids = vec![
            "not-a-uuid",
            "550e8400-e29b-41d4-a716",
            "550e8400-e29b-41d4-a716-44665544000g",
            "",
            "550e8400-e29b-41d4-a716-446655440000-extra",
        ];
        for uuid in invalid_uuids {
            let mut errors = ValidationErrors::new();
            StringValidator::new("id", uuid, &mut errors).valid_uuid();
            assert!(errors.has_errors(), "UUID '{}' should be invalid", uuid);
        }
    }

    #[test]
    fn test_string_validator_url_variants() {
        let valid_urls = vec![
            "http://example.com",
            "https://example.com",
            "https://example.com:8080",
            "https://example.com/path",
            "https://example.com/path?query=1",
            "https://user:pass@example.com",
            "ftp://example.com",
        ];
        for url in valid_urls {
            let mut errors = ValidationErrors::new();
            StringValidator::new("url", url, &mut errors).valid_url();
            assert!(!errors.has_errors(), "URL {} should be valid", url);
        }

        let invalid_urls = vec!["not-a-url", "example.com", "http://", ""];
        for url in invalid_urls {
            let mut errors = ValidationErrors::new();
            StringValidator::new("url", url, &mut errors).valid_url();
            assert!(errors.has_errors(), "URL '{}' should be invalid", url);
        }
    }

    #[test]
    fn test_optional_string_validator() {
        let mut errors = ValidationErrors::new();
        OptionalStringValidator::new("field", None, &mut errors)
            .if_present_matches(r"^[a-z]+$", "lowercase letters");
        assert!(!errors.has_errors());

        let mut errors = ValidationErrors::new();
        OptionalStringValidator::new("field", Some("ABC"), &mut errors)
            .if_present_matches(r"^[a-z]+$", "lowercase letters");
        assert!(errors.has_errors());
    }

    #[test]
    fn test_number_validator_boundary() {
        let mut errors = ValidationErrors::new();
        NumberValidator::new("num".to_string(), 5usize, &mut errors).range(1, 10);
        assert!(!errors.has_errors());

        let mut errors = ValidationErrors::new();
        NumberValidator::new("num".to_string(), 0usize, &mut errors).range(1, 10);
        assert!(errors.has_errors());

        let mut errors = ValidationErrors::new();
        NumberValidator::new("num".to_string(), 11usize, &mut errors).range(1, 10);
        assert!(errors.has_errors());
    }

    #[test]
    fn test_number_validator_min_max() {
        let mut errors = ValidationErrors::new();
        NumberValidator::new("num".to_string(), 1usize, &mut errors).min(1);
        assert!(!errors.has_errors());

        let mut errors = ValidationErrors::new();
        NumberValidator::new("num".to_string(), 0usize, &mut errors).min(1);
        assert!(errors.has_errors());

        let mut errors = ValidationErrors::new();
        NumberValidator::new("num".to_string(), 10usize, &mut errors).max(10);
        assert!(!errors.has_errors());

        let mut errors = ValidationErrors::new();
        NumberValidator::new("num".to_string(), 11usize, &mut errors).max(10);
        assert!(errors.has_errors());
    }

    #[test]
    fn test_validation_error_out_of_range() {
        let err = ValidationError::out_of_range("num", Some(1), Some(10));
        assert!(err.message.contains("between 1 and 10"));

        let err = ValidationError::out_of_range("num", Some(1), None);
        assert!(err.message.contains("at least 1"));

        let err = ValidationError::out_of_range("num", None, Some(10));
        assert!(err.message.contains("at most 10"));

        let err = ValidationError::out_of_range("num", None, None);
        assert!(err.message.contains("invalid value"));
    }

    #[test]
    fn test_validation_error_too_long() {
        let err = ValidationError::too_long("field", 100);
        assert!(err.message.contains("100 characters"));
        assert!(err.message.contains("field"));
    }

    #[test]
    fn test_validation_error_too_short() {
        let err = ValidationError::too_short("field", 5);
        assert!(err.message.contains("at least 5 characters"));
        assert!(err.message.contains("field"));
    }

    #[test]
    fn test_validation_error_invalid_enum() {
        let err = ValidationError::invalid_enum("status", &["active", "inactive", "pending"]);
        assert!(err.message.contains("active, inactive, pending"));
        assert!(err.message.contains("invalid value"));
    }

    #[test]
    fn test_validate_session_id_edge_cases() {
        assert!(validate_session_id("00000000-0000-0000-0000-000000000000").is_ok());
        assert!(validate_session_id("ffffffff-ffff-ffff-ffff-ffffffffffff").is_ok());
        assert!(validate_session_id("").is_err());
        assert!(validate_session_id("invalid").is_err());
        assert!(validate_session_id("123").is_err());
    }

    #[test]
    fn test_validate_message_index_edge_cases() {
        assert!(validate_message_index(0, 10).is_ok());
        assert!(validate_message_index(5, 10).is_ok());
        assert!(validate_message_index(9, 10).is_ok());
        assert!(validate_message_index(10, 10).is_err());
        assert!(validate_message_index(11, 10).is_err());
        assert!(validate_message_index(0, 0).is_ok());
        assert!(validate_message_index(100, 0).is_err());
    }

    #[test]
    fn test_validate_command_edge_cases() {
        assert!(validate_command("ls", None, None).is_ok());
        assert!(validate_command("ls", Some(&vec!["-la".to_string()]), None).is_ok());
        assert!(validate_command("", None, None).is_err());

        let long_command = "a".repeat(1001);
        assert!(validate_command(&long_command, None, None).is_err());

        let long_arg = "a".repeat(10001);
        assert!(validate_command("cmd", Some(&vec![long_arg]), None).is_err());

        let long_workdir = "a".repeat(1001);
        assert!(validate_command("cmd", None, Some(&long_workdir)).is_err());

        let empty_arg = "".to_string();
        assert!(validate_command("cmd", Some(&vec![empty_arg]), None).is_ok());
    }

    #[test]
    fn test_request_validator_required_string_edge_cases() {
        let mut validator = RequestValidator::new();
        validator.validate_required_string("field", None);
        assert!(!validator.is_valid());

        let mut validator = RequestValidator::new();
        validator.validate_required_string("field", Some(""));
        assert!(!validator.is_valid());

        let mut validator = RequestValidator::new();
        validator.validate_required_string("field", Some("   "));
        assert!(!validator.is_valid());

        let mut validator = RequestValidator::new();
        validator.validate_required_string("field", Some("value"));
        assert!(validator.is_valid());
    }

    #[test]
    fn test_request_validator_optional_string_edge_cases() {
        let mut validator = RequestValidator::new();
        validator.validate_optional_string("field", None, 100);
        assert!(validator.is_valid());

        let mut validator = RequestValidator::new();
        validator.validate_optional_string("field", Some("value"), 100);
        assert!(validator.is_valid());

        let long_value = "a".repeat(101);
        let mut validator = RequestValidator::new();
        validator.validate_optional_string("field", Some(&long_value), 100);
        assert!(!validator.is_valid());
    }

    #[test]
    fn test_request_validator_required_uuid_edge_cases() {
        let mut validator = RequestValidator::new();
        validator.validate_required_uuid("id", None);
        assert!(!validator.is_valid());

        let mut validator = RequestValidator::new();
        validator.validate_required_uuid("id", Some("invalid"));
        assert!(!validator.is_valid());

        let mut validator = RequestValidator::new();
        validator.validate_required_uuid("id", Some("550e8400-e29b-41d4-a716-446655440000"));
        assert!(validator.is_valid());
    }

    #[test]
    fn test_request_validator_required_number_edge_cases() {
        let mut validator = RequestValidator::new();
        validator.validate_required_number("num", None, 0, 100);
        assert!(!validator.is_valid());

        let mut validator = RequestValidator::new();
        validator.validate_required_number("num", Some(50), 0, 100);
        assert!(validator.is_valid());

        let mut validator = RequestValidator::new();
        validator.validate_required_number("num", Some(0), 0, 100);
        assert!(validator.is_valid());

        let mut validator = RequestValidator::new();
        validator.validate_required_number("num", Some(100), 0, 100);
        assert!(validator.is_valid());

        let mut validator = RequestValidator::new();
        validator.validate_required_number("num", Some(101), 0, 100);
        assert!(!validator.is_valid());

        let mut validator = RequestValidator::new();
        validator.validate_required_number("num", Some(200), 0, 100);
        assert!(!validator.is_valid());
    }

    #[test]
    fn test_request_validator_optional_number_edge_cases() {
        let mut validator = RequestValidator::new();
        validator.validate_optional_number("num", None, 0, 100);
        assert!(validator.is_valid());

        let mut validator = RequestValidator::new();
        validator.validate_optional_number("num", Some(50), 0, 100);
        assert!(validator.is_valid());

        let mut validator = RequestValidator::new();
        validator.validate_optional_number("num", Some(101), 0, 100);
        assert!(!validator.is_valid());
    }

    #[test]
    fn test_request_validator_enum_edge_cases() {
        let mut validator = RequestValidator::new();
        validator.validate_enum("status", "active", &["active", "inactive"]);
        assert!(validator.is_valid());

        let mut validator = RequestValidator::new();
        validator.validate_enum("status", "unknown", &["active", "inactive"]);
        assert!(!validator.is_valid());

        let mut validator = RequestValidator::new();
        validator.validate_enum("status", "ACTIVE", &["active", "inactive"]);
        assert!(!validator.is_valid());
    }

    #[test]
    fn test_validation_errors_response_with_empty_details() {
        let errors = ValidationErrors::new();
        let resp = errors.to_response();
        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_validation_errors_response_with_single_error() {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new("field1", "error1"));
        let resp = errors.to_response();
        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[test]
    fn test_validation_errors_response_with_multiple_errors() {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new("field1", "error1"));
        errors.add(ValidationError::new("field2", "error2"));
        errors.add(ValidationError::new("field3", "error3"));
        let resp = errors.to_response();
        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[test]
    fn test_validation_errors_add_if() {
        let mut errors = ValidationErrors::new();
        errors.add_if(true, ValidationError::new("field", "error"));
        assert!(errors.has_errors());

        let mut errors = ValidationErrors::new();
        errors.add_if(false, ValidationError::new("field", "error"));
        assert!(!errors.has_errors());
    }

    #[test]
    fn test_validation_errors_len() {
        let mut errors = ValidationErrors::new();
        assert_eq!(errors.len(), 0);

        errors.add(ValidationError::new("f1", "e1"));
        assert_eq!(errors.len(), 1);

        errors.add(ValidationError::new("f2", "e2"));
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_validate_pagination_edge_cases() {
        assert_eq!(validate_pagination(None, None), (20, 0));
        assert_eq!(validate_pagination(Some(0), None), (0, 0));
        assert_eq!(validate_pagination(None, Some(0)), (20, 0));
        assert_eq!(validate_pagination(Some(1), Some(1)), (1, 1));
        assert_eq!(validate_pagination(Some(200), Some(10000)), (200, 10000));
        assert_eq!(validate_pagination(Some(201), Some(10001)), (200, 10000));
        assert_eq!(validate_pagination(Some(1000), Some(20000)), (200, 10000));
    }

    #[test]
    fn test_string_validator_max_length_zero() {
        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "", &mut errors).max_length(0);
        assert!(!errors.has_errors());

        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "a", &mut errors).max_length(0);
        assert!(errors.has_errors());
    }

    #[test]
    fn test_string_validator_min_length_zero() {
        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "", &mut errors).min_length(0);
        assert!(!errors.has_errors());

        let mut errors = ValidationErrors::new();
        StringValidator::new("field", "a", &mut errors).min_length(0);
        assert!(!errors.has_errors());
    }

    #[test]
    fn test_string_validator_emoji_and_symbols() {
        let emoji = "👨‍👩‍👧‍👦 🔥 💻 42";
        let mut errors = ValidationErrors::new();
        StringValidator::new("field", emoji, &mut errors)
            .min_length(1)
            .max_length(50);
        assert!(!errors.has_errors());

        let sql_injection = "'; DROP TABLE users; --";
        let mut errors = ValidationErrors::new();
        StringValidator::new("field", sql_injection, &mut errors)
            .min_length(1)
            .max_length(1000);
        assert!(!errors.has_errors());

        let xss = "<script>alert('xss')</script>";
        let mut errors = ValidationErrors::new();
        StringValidator::new("field", xss, &mut errors)
            .min_length(1)
            .max_length(1000);
        assert!(!errors.has_errors());
    }

    #[test]
    fn test_validate_command_with_special_chars_in_args() {
        let special_args = vec![
            "arg with spaces".to_string(),
            "arg\twith\ttabs".to_string(),
            "arg\nwith\nnewlines".to_string(),
            "arg\"with\"quotes".to_string(),
            "arg'with'singlequotes".to_string(),
            "arg$with$dollars".to_string(),
            "arg`with`backticks".to_string(),
        ];
        for arg in special_args {
            let result = validate_command("cmd", Some(&vec![arg.clone()]), None);
            assert!(result.is_ok(), "arg '{}' should be valid", arg);
        }
    }

    #[test]
    fn test_validate_workdir_edge_cases() {
        assert!(validate_command("ls", None, Some("/tmp")).is_ok());
        assert!(validate_command("ls", None, Some("/")).is_ok());
        assert!(validate_command("ls", None, Some("")).is_ok());
        assert!(validate_command("ls", None, Some("   ")).is_ok());

        let long_path = "/".repeat(1001);
        let mut errors = ValidationErrors::new();
        StringValidator::new("workdir", &long_path, &mut errors).max_length(1000);
        assert!(errors.has_errors());

        let max_path = "/".repeat(1000);
        let mut errors = ValidationErrors::new();
        StringValidator::new("workdir", &max_path, &mut errors).max_length(1000);
        assert!(!errors.has_errors());
    }

    #[test]
    fn test_request_validator_multiple_field_validation() {
        let mut validator = RequestValidator::new();
        validator.validate_required_string("field1", Some("value1"));
        validator.validate_required_string("field2", Some(""));
        validator.validate_required_string("field3", None);
        assert!(!validator.is_valid());
        assert_eq!(validator.errors().len(), 2);
    }

    #[test]
    fn test_validation_error_display() {
        let errors = ValidationErrors::new();
        let display = format!("{}", errors);
        assert!(display.contains("0 errors"));

        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new("f1", "e1"));
        errors.add(ValidationError::new("f2", "e2"));
        let display = format!("{}", errors);
        assert!(display.contains("2 errors"));
    }
}
