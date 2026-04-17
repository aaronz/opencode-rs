use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;

use super::error::{ErrorResponse, FieldError};

#[derive(Debug, Clone, Serialize)]
pub struct ValidationErrors {
    #[serde(skip)]
    pub details: Vec<FieldError>,
}

#[allow(dead_code)]
impl ValidationErrors {
    pub(crate) fn new() -> Self {
        Self {
            details: Vec::new(),
        }
    }

    pub(crate) fn add(&mut self, error: FieldError) {
        self.details.push(error);
    }

    pub(crate) fn add_if(&mut self, condition: bool, error: FieldError) {
        if condition {
            self.add(error);
        }
    }

    pub(crate) fn has_errors(&self) -> bool {
        !self.details.is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        self.details.len()
    }

    pub(crate) fn to_response(&self) -> HttpResponse {
        if self.details.is_empty() {
            return ErrorResponse::validation_error("Request validation failed")
                .to_response(actix_web::http::StatusCode::BAD_REQUEST);
        }

        if self.details.len() == 1 {
            let err = &self.details[0];
            return ErrorResponse::validation_error(format!(
                "Invalid value for '{}': {}",
                err.field, err.message
            ))
            .with_details(vec![err.clone()])
            .to_response(actix_web::http::StatusCode::UNPROCESSABLE_ENTITY);
        }

        ErrorResponse::validation_error(format!(
            "Request validation failed with {} errors",
            self.details.len()
        ))
        .with_details(self.details.clone())
        .to_response(actix_web::http::StatusCode::UNPROCESSABLE_ENTITY)
    }
}

#[allow(dead_code)]
impl Default for ValidationErrors {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Validation error: {} errors", self.details.len())
    }
}

#[allow(dead_code)]
impl ResponseError for ValidationErrors {
    fn error_response(&self) -> HttpResponse {
        self.to_response()
    }
}

#[allow(dead_code)]
pub struct StringValidator<'a> {
    field: &'a str,
    value: &'a str,
    errors: &'a mut ValidationErrors,
}

#[allow(dead_code)]
impl<'a> StringValidator<'a> {
    pub(crate) fn new(field: &'a str, value: &'a str, errors: &'a mut ValidationErrors) -> Self {
        Self {
            field,
            value,
            errors,
        }
    }

    pub(crate) fn not_empty(self) -> Self {
        self.errors.add_if(
            self.value.trim().is_empty(),
            FieldError::with_code(self.field, "cannot be empty"),
        );
        self
    }

    pub(crate) fn min_length(self, min: usize) -> Self {
        self.errors.add_if(
            self.value.len() < min,
            FieldError::too_short(self.field, min),
        );
        self
    }

    pub(crate) fn max_length(self, max: usize) -> Self {
        self.errors.add_if(
            self.value.len() > max,
            FieldError::too_long(self.field, max),
        );
        self
    }

    pub(crate) fn matches(self, pattern: &str, description: &str) -> Self {
        let regex = regex_lite::Regex::new(pattern).ok();
        if let Some(re) = regex {
            self.errors.add_if(
                !re.is_match(self.value),
                FieldError::format(self.field, description),
            );
        }
        self
    }

    pub(crate) fn valid_uuid(self) -> Self {
        self.errors.add_if(
            uuid::Uuid::parse_str(self.value).is_err(),
            FieldError::format(self.field, "valid UUID"),
        );
        self
    }

    pub(crate) fn valid_url(self) -> Self {
        self.errors.add_if(
            url::Url::parse(self.value).is_err(),
            FieldError::format(self.field, "valid URL"),
        );
        self
    }
}

#[allow(dead_code)]
pub struct OptionalStringValidator<'a> {
    field: &'a str,
    value: Option<&'a str>,
    errors: &'a mut ValidationErrors,
}

#[allow(dead_code)]
impl<'a> OptionalStringValidator<'a> {
    pub(crate) fn new(
        field: &'a str,
        value: Option<&'a str>,
        errors: &'a mut ValidationErrors,
    ) -> Self {
        Self {
            field,
            value,
            errors,
        }
    }

    pub(crate) fn if_present_matches(self, pattern: &str, description: &str) -> Self {
        if let Some(v) = self.value {
            let regex = regex_lite::Regex::new(pattern).ok();
            if let Some(re) = regex {
                self.errors
                    .add_if(!re.is_match(v), FieldError::format(self.field, description));
            }
        }
        self
    }

    pub(crate) fn if_present_valid_uuid(self) -> Self {
        if let Some(v) = self.value {
            self.errors.add_if(
                uuid::Uuid::parse_str(v).is_err(),
                FieldError::format(self.field, "valid UUID"),
            );
        }
        self
    }

    pub(crate) fn if_present_valid_url(self) -> Self {
        if let Some(v) = self.value {
            self.errors.add_if(
                url::Url::parse(v).is_err(),
                FieldError::format(self.field, "valid URL"),
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
    pub(crate) fn new(field: String, value: T, errors: &mut ValidationErrors) -> Self {
        Self {
            field,
            value,
            errors,
        }
    }
}

#[allow(dead_code)]
impl NumberValidator<usize> {
    /// # Safety
    ///
    /// `self.errors` must be a valid, non-null pointer derived from a live
    /// `&mut ValidationErrors` reference. The caller guarantees that
    /// `ValidationErrors` outlives this `NumberValidator` and that no other
    /// code holds a mutable reference to it for the lifetime of this validator.
    /// This is trivially satisfied when `NumberValidator::new` is called with
    /// `&mut errors` and the validator is dropped before `errors` is used again.
    pub(crate) fn min(self, min: usize) -> Self {
        // SAFETY: The raw pointer was created from a valid `&mut ValidationErrors`
        // in `NumberValidator::new`. The caller guarantees the referent lives long
        // enough and no aliasing mutable references exist. The dereference produces
        // a `&mut ValidationErrors` matching the original lifetime.
        unsafe {
            // SAFETY: Dereferencing self.errors is safe per the contract above.
            (&mut *self.errors).add_if(
                self.value < min,
                FieldError::out_of_range(&self.field, Some(min), None),
            );
        }
        self
    }

    /// # Safety
    ///
    /// `self.errors` must be a valid, non-null pointer derived from a live
    /// `&mut ValidationErrors` reference. The caller guarantees that
    /// `ValidationErrors` outlives this `NumberValidator` and that no other
    /// code holds a mutable reference to it for the lifetime of this validator.
    pub(crate) fn max(self, max: usize) -> Self {
        // SAFETY: The raw pointer was created from a valid `&mut ValidationErrors`
        // in `NumberValidator::new`. The caller guarantees the referent lives long
        // enough and no aliasing mutable references exist. The dereference produces
        // a `&mut ValidationErrors` matching the original lifetime.
        unsafe {
            // SAFETY: Dereferencing self.errors is safe per the contract above.
            (&mut *self.errors).add_if(
                self.value > max,
                FieldError::out_of_range(&self.field, None, Some(max)),
            );
        }
        self
    }

    pub(crate) fn range(self, min: usize, max: usize) -> Self {
        self.min(min).max(max)
    }
}

pub struct RequestValidator {
    errors: ValidationErrors,
}

#[allow(dead_code)]
impl RequestValidator {
    pub(crate) fn new() -> Self {
        Self {
            errors: ValidationErrors::new(),
        }
    }

    pub(crate) fn errors(&self) -> &ValidationErrors {
        &self.errors
    }

    pub(crate) fn errors_mut(&mut self) -> &mut ValidationErrors {
        &mut self.errors
    }

    pub(crate) fn is_valid(&self) -> bool {
        !self.errors.has_errors()
    }

    pub(crate) fn validate_required_string(&mut self, field: &str, value: Option<&str>) {
        let Some(v) = value else {
            self.errors.add(FieldError::required(field));
            return;
        };
        StringValidator::new(field, v, &mut self.errors)
            .not_empty()
            .max_length(10000);
    }

    pub(crate) fn validate_optional_string(
        &mut self,
        field: &str,
        value: Option<&str>,
        max_length: usize,
    ) {
        if let Some(v) = value {
            StringValidator::new(field, v, &mut self.errors).max_length(max_length);
        }
    }

    pub(crate) fn validate_required_uuid(&mut self, field: &str, value: Option<&str>) {
        let Some(v) = value else {
            self.errors.add(FieldError::required(field));
            return;
        };
        StringValidator::new(field, v, &mut self.errors).valid_uuid();
    }

    pub(crate) fn validate_required_number(
        &mut self,
        field: &str,
        value: Option<usize>,
        min: usize,
        max: usize,
    ) {
        let Some(v) = value else {
            self.errors.add(FieldError::required(field));
            return;
        };
        NumberValidator::new(field.to_string(), v, &mut self.errors).range(min, max);
    }

    pub(crate) fn validate_optional_number(
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

    pub(crate) fn validate_enum(&mut self, field: &str, value: &str, allowed_values: &[&str]) {
        if !allowed_values.iter().any(|v| *v == value) {
            self.errors
                .add(FieldError::invalid_enum(field, allowed_values));
        }
    }

    pub(crate) fn validate(self) -> Result<(), ValidationErrors> {
        if self.errors.has_errors() {
            Err(self.errors)
        } else {
            Ok(())
        }
    }
}

#[allow(dead_code)]
impl Default for RequestValidator {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn validate_session_id(id: &str) -> Result<uuid::Uuid, ValidationErrors> {
    let mut errors = ValidationErrors::new();
    StringValidator::new("session_id", id, &mut errors).valid_uuid();
    if errors.has_errors() {
        Err(errors)
    } else {
        uuid::Uuid::parse_str(id).map_err(|_| {
            let mut errs = ValidationErrors::new();
            errs.add(FieldError::format("session_id", "valid UUID"));
            errs
        })
    }
}

#[allow(dead_code)]
pub(crate) fn validate_message_index(index: usize, max: usize) -> Result<(), ValidationErrors> {
    let mut errors = ValidationErrors::new();
    NumberValidator::new("message_index".to_string(), index, &mut errors)
        .range(0, max.saturating_sub(1));
    if errors.has_errors() {
        Err(errors)
    } else {
        Ok(())
    }
}

pub(crate) fn validate_pagination(limit: Option<usize>, offset: Option<usize>) -> (usize, usize) {
    let limit = limit.unwrap_or(20);
    let offset = offset.unwrap_or(0);
    (limit.min(200), offset.min(10000))
}

#[allow(dead_code)]
pub(crate) fn validate_command(
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
        let err = FieldError::with_code("email", "invalid format");
        assert_eq!(err.field, "email");
        assert_eq!(err.message, "invalid format");
        assert_eq!(err.code, 7001);
    }

    #[test]
    fn test_validation_error_required() {
        let err = FieldError::required("name");
        assert_eq!(err.field, "name");
        assert!(err.message.contains("required"));
        assert_eq!(err.code, 7002);
    }

    #[test]
    fn test_validation_error_format() {
        let err = FieldError::format("email", "valid email");
        assert_eq!(err.field, "email");
        assert!(err.message.contains("invalid format"));
        assert!(err.message.contains("valid email"));
        assert_eq!(err.code, 7003);
    }

    #[test]
    fn test_validation_errors_single() {
        let mut errors = ValidationErrors::new();
        errors.add(FieldError::with_code("field1", "error1"));

        let resp = errors.to_response();
        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[test]
    fn test_validation_errors_multiple() {
        let mut errors = ValidationErrors::new();
        errors.add(FieldError::with_code("field1", "error1"));
        errors.add(FieldError::with_code("field2", "error2"));

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
        let err = FieldError::out_of_range("num", Some(1), Some(10));
        assert!(err.message.contains("between 1 and 10"));

        let err = FieldError::out_of_range("num", Some(1), None);
        assert!(err.message.contains("at least 1"));

        let err = FieldError::out_of_range("num", None, Some(10));
        assert!(err.message.contains("at most 10"));

        let err = FieldError::out_of_range("num", None, None);
        assert!(err.message.contains("invalid value"));
    }

    #[test]
    fn test_validation_error_too_long() {
        let err = FieldError::too_long("field", 100);
        assert!(err.message.contains("100 characters"));
        assert!(err.message.contains("field"));
    }

    #[test]
    fn test_validation_error_too_short() {
        let err = FieldError::too_short("field", 5);
        assert!(err.message.contains("at least 5 characters"));
        assert!(err.message.contains("field"));
    }

    #[test]
    fn test_validation_error_invalid_enum() {
        let err = FieldError::invalid_enum("status", &["active", "inactive", "pending"]);
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
        errors.add(FieldError::with_code("field1", "error1"));
        let resp = errors.to_response();
        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[test]
    fn test_validation_errors_response_with_multiple_errors() {
        let mut errors = ValidationErrors::new();
        errors.add(FieldError::with_code("field1", "error1"));
        errors.add(FieldError::with_code("field2", "error2"));
        errors.add(FieldError::with_code("field3", "error3"));
        let resp = errors.to_response();
        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[test]
    fn test_validation_errors_add_if() {
        let mut errors = ValidationErrors::new();
        errors.add_if(true, FieldError::with_code("field", "error"));
        assert!(errors.has_errors());

        let mut errors = ValidationErrors::new();
        errors.add_if(false, FieldError::with_code("field", "error"));
        assert!(!errors.has_errors());
    }

    #[test]
    fn test_validation_errors_len() {
        let mut errors = ValidationErrors::new();
        assert_eq!(errors.len(), 0);

        errors.add(FieldError::with_code("f1", "e1"));
        assert_eq!(errors.len(), 1);

        errors.add(FieldError::with_code("f2", "e2"));
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
        errors.add(FieldError::with_code("f1", "e1"));
        errors.add(FieldError::with_code("f2", "e2"));
        let display = format!("{}", errors);
        assert!(display.contains("2 errors"));
    }

    #[test]
    fn test_validate_session_id_returns_validation_errors_on_invalid_input() {
        let result = validate_session_id("not-a-valid-uuid");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.has_errors());
        assert_eq!(errors.len(), 1);
        assert_eq!(errors.details[0].field, "session_id");
    }

    #[test]
    fn test_validate_session_id_returns_uuid_on_valid_input() {
        let result = validate_session_id("550e8400-e29b-41d4-a716-446655440000");
        assert!(result.is_ok());
        let uuid = result.unwrap();
        assert_eq!(uuid.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_validate_session_id_returns_error_for_empty_string() {
        let result = validate_session_id("");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.has_errors());
    }

    #[test]
    fn test_validate_session_id_returns_error_for_partial_uuid() {
        let result = validate_session_id("550e8400-e29b-41d4-a716");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_session_id_returns_error_for_malformed_uuid() {
        let malformed = vec![
            "gibberish",
            "12345",
            "ZZZZZZZZ-YYYY-XXXX-WWWW-VVVVVVVVVVVV",
            "550e8400-e29b-41d4-a716-44665544000",
            "550e8400-e29b-41d4-a716-4466554400000",
            "not-a-uuid-at-all",
            "uuid-with-extra-chars-at-end-550e8400-e29b-41d4-a716-446655440000",
        ];
        for input in malformed {
            let result = validate_session_id(input);
            assert!(result.is_err(), "Expected '{}' to be invalid UUID", input);
        }
    }

    #[test]
    fn test_validate_session_id_validates_all_valid_uuid_formats() {
        let valid_uuids = vec![
            "00000000-0000-0000-0000-000000000000",
            "ffffffff-ffff-ffff-ffff-ffffffffffff",
            "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
            "f47ac10b-58cc-4372-a567-0e02b2c3d479",
            "550e8400-e29b-41d4-a716-446655440000",
        ];
        for uuid_str in valid_uuids {
            let result = validate_session_id(uuid_str);
            assert!(result.is_ok(), "Expected '{}' to be valid UUID", uuid_str);
        }
    }

    #[test]
    fn test_validate_session_id_error_response_contains_proper_field_error() {
        let result = validate_session_id("invalid-session");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        let response = errors.to_response();
        assert_eq!(
            response.status(),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[test]
    fn test_validate_message_index_returns_ok_for_valid_index() {
        let result = validate_message_index(5, 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_message_index_returns_error_for_out_of_range() {
        let result = validate_message_index(10, 10);
        assert!(result.is_err());

        let result = validate_message_index(15, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_message_index_allows_zero_index() {
        let result = validate_message_index(0, 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_message_index_handles_empty_collection() {
        let result = validate_message_index(0, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_message_index_returns_error_for_large_index_on_empty_collection() {
        let result = validate_message_index(100, 0);
        assert!(result.is_err());
    }
}
