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
}
