//! Sensitive file detection module (FR-226).
//!
//! Implements default-deny security policy for sensitive files.
//!
//! ## Sensitive File Patterns
//!
//! | Pattern | Default Behavior | Can Override |
//! |---------|------------------|--------------|
//! | `.env` | DENY | Yes |
//! | `*.pem`, `*.key` | DENY | Yes |
//! | `credentials.json` | DENY | Yes |
//! | `secrets.*` | DENY | Yes |
//!
//! ## Usage
//!
//! ```rust
//! use opencode_permission::sensitive_file::is_sensitive_path;
//!
//! // Check if a path is sensitive
//! if is_sensitive_path("/path/to/.env") {
//!     // Deny access by default
//! }
//! ```

use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

/// List of sensitive file name patterns (basename only).
static SENSITIVE_NAMES: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // .env files (all .env variants)
        Regex::new(r"(?:^|[/\\])\.env(?:\.\w+)?$").unwrap(),
        // PEM certificates and keys
        Regex::new(r"\.pem$").unwrap(),
        Regex::new(r"\.key$").unwrap(),
        // Credentials files
        Regex::new(r"^credentials\.json$").unwrap(),
        Regex::new(r"^credentials\.yaml$").unwrap(),
        Regex::new(r"^credentials\.yml$").unwrap(),
        // Secret files
        Regex::new(r"(?:^|[/\\])\.secrets$").unwrap(),
        Regex::new(r"^secrets\.\w+$").unwrap(),
        Regex::new(r"^secret\.\w+$").unwrap(),
        Regex::new(r"(?:^|[/\\])\.secret$").unwrap(),
        // API key files
        Regex::new(r"^api_key$").unwrap(),
        Regex::new(r"^apikey$").unwrap(),
        // Token files
        Regex::new(r"^token$").unwrap(),
        Regex::new(r"^oauth\b").unwrap(),
        // Private key files
        Regex::new(r"^id_rsa$").unwrap(),
        Regex::new(r"^id_ed25519$").unwrap(),
        Regex::new(r"^id_ecdsa$").unwrap(),
        // .git-credentials (contains auth info)
        Regex::new(r"^\.git-credentials$").unwrap(),
        Regex::new(r"^\.gitconfig$").unwrap(),
        // S3 config files
        Regex::new(r"^s3cfg$").unwrap(),
        Regex::new(r"^\.s3cfg$").unwrap(),
        // Database credentials
        Regex::new(r"^database\.json$").unwrap(),
        Regex::new(r"^\.db_credentials$").unwrap(),
        // Docker config
        Regex::new(r"^\.dockerconfigjson$").unwrap(),
        // Azure credentials
        Regex::new(r"^azure\.json$").unwrap(),
        // Google cloud credentials
        Regex::new(r"application_credentials\.json$").unwrap(),
    ]
});

/// List of sensitive path patterns (full path patterns).
static SENSITIVE_PATHS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // AWS credentials
        Regex::new(r"\.aws/credentials$").unwrap(),
        Regex::new(r"\.aws/config$").unwrap(),
        // GCP credentials
        Regex::new(r"gcloud/application_credentials\.json$").unwrap(),
        Regex::new(r"gcloud/configurations/config_\w+$").unwrap(),
        // Azure credentials
        Regex::new(r"\.azure/azureProfile\.json$").unwrap(),
        Regex::new(r"\.azure/accessTokens\.json$").unwrap(),
        // GitHub tokens
        Regex::new(r"\.config/ghHosts$").unwrap(),
        // Kubernetes secrets
        Regex::new(r"kubernetes/secrets\.yaml$").unwrap(),
        // SSH config with sensitive data
        Regex::new(r"ssh/config$").unwrap(),
        // Maven settings with passwords
        Regex::new(r"\.m2/settings\.xml$").unwrap(),
        // npmrc with auth tokens
        Regex::new(r"\.npmrc$").unwrap(),
        // pip cache with credentials
        Regex::new(r"\.pip/pip\.conf$").unwrap(),
        // Netrc (FTP/HTTP credentials)
        Regex::new(r"^\.netrc$").unwrap(),
        // Shell history containing secrets
        Regex::new(r"\.bash_history$").unwrap(),
        Regex::new(r"\.zsh_history$").unwrap(),
        Regex::new(r"\.history$").unwrap(),
    ]
});

/// Known secret file extensions that should be denied.
static SENSITIVE_EXTENSIONS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        ".pem",
        ".key",
        ".p12",
        ".pfx",
        ".crt",
        ".der",
        ".jks",
        ".keystore",
        ".truststore",
    ]
});

/// Checks if a file path is sensitive and should be denied by default.
///
/// ## Arguments
///
/// * `path` - The file path to check
///
/// ## Returns
///
/// * `true` if the path is sensitive and should be denied by default
/// * `false` if the path is not sensitive
///
/// ## Examples
///
/// ```rust
/// use opencode_permission::sensitive_file::is_sensitive_path;
///
/// assert!(is_sensitive_path(".env"));
/// assert!(is_sensitive_path("/path/to/.env.local"));
/// assert!(is_sensitive_path("credentials.json"));
/// assert!(!is_sensitive_path("source_code.rs"));
/// ```
pub fn is_sensitive_path<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    // Get the filename (basename)
    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
        // Check against sensitive name patterns
        for pattern in SENSITIVE_NAMES.iter() {
            if pattern.is_match(filename) {
                return true;
            }
        }
    }

    // Check if the path ends with a sensitive extension
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_with_dot = format!(".{}", ext);
        if SENSITIVE_EXTENSIONS.contains(&ext_with_dot.as_str()) {
            return true;
        }
    }

    // Check against sensitive path patterns
    let path_str = path.to_string_lossy();
    for pattern in SENSITIVE_PATHS.iter() {
        if pattern.is_match(&path_str) {
            return true;
        }
    }

    false
}

/// Checks if a file path is in a sensitive directory.
///
/// ## Arguments
///
/// * `path` - The file path to check
///
/// ## Returns
///
/// * `true` if the path is in a sensitive directory
///
/// ## Examples
///
/// ```rust
/// use opencode_permission::sensitive_file::is_sensitive_directory;
///
/// assert!(is_sensitive_directory("/etc/ssh/some_file"));
/// assert!(!is_sensitive_directory("/home/user/project/source.rs"));
/// ```
pub fn is_sensitive_directory<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    // Common sensitive directories
    let sensitive_dirs = [
        "/etc/ssh",
        "/etc/ssl",
        "/etc/certs",
        "/root/.ssh",
        "/home/.ssh",
        "/.ssh",
        "/tmp/secrets",
        "/var/secrets",
        "/run/secrets",
    ];

    for dir in sensitive_dirs {
        if path.starts_with(dir) {
            return true;
        }
    }

    false
}

/// Checks if a path should be blocked due to being an external directory.
///
/// External directories are paths outside the allowed working directory.
/// This is separate from sensitive file checking.
///
/// ## Arguments
///
/// * `path` - The file path to check
/// * `allowed_base` - The allowed base directory
///
/// ## Returns
///
/// * `true` if the path is outside the allowed base directory
pub fn is_external_directory<P1: AsRef<Path>, P2: AsRef<Path>>(path: P1, allowed_base: P2) -> bool {
    let path = path.as_ref();
    let allowed_base = allowed_base.as_ref();

    // Normalize paths and check if path is within allowed_base
    let path_abs = if path.is_relative() {
        std::env::current_dir().ok().map(|cwd| cwd.join(path))
    } else {
        Some(path.to_path_buf())
    };

    let base_abs = if allowed_base.is_relative() {
        std::env::current_dir()
            .ok()
            .map(|cwd| cwd.join(allowed_base))
    } else {
        Some(allowed_base.to_path_buf())
    };

    match (path_abs, base_abs) {
        (Some(p), Some(b)) => {
            // Check if path starts with allowed base
            !p.starts_with(&b)
        }
        _ => false, // If we can't determine, don't block
    }
}

/// Gets the reason why a path is considered sensitive.
///
/// ## Arguments
///
/// * `path` - The file path to check
///
/// ## Returns
///
/// * `Some(&str)` with the reason if the path is sensitive
/// * `None` if the path is not sensitive
pub fn get_sensitive_reason<P: AsRef<Path>>(path: P) -> Option<&'static str> {
    let path = path.as_ref();

    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
        if filename.starts_with(".env") {
            return Some(".env files contain environment variables and secrets");
        }
        if filename.ends_with(".pem") || filename.ends_with(".key") {
            return Some("PEM and key files contain cryptographic keys");
        }
        if filename == "credentials.json" || filename == "credentials.yaml" {
            return Some("Credentials files contain authentication data");
        }
        if filename.starts_with("secrets.") || filename.starts_with("secret.") {
            return Some("Secret files contain sensitive configuration");
        }
    }

    let path_str = path.to_string_lossy();
    if path_str.contains(".aws/credentials") {
        return Some("AWS credentials files contain access keys");
    }
    if path_str.contains("gcloud/application_credentials") {
        return Some("GCP credentials files contain service account keys");
    }

    None
}

/// Represents a sensitive file check result with details.
#[derive(Debug, Clone)]
pub struct SensitiveCheckResult {
    /// Whether the file is sensitive.
    pub is_sensitive: bool,

    /// Reason for denial (if sensitive).
    pub reason: Option<&'static str>,

    /// Whether the check can be overridden.
    pub can_override: bool,
}

impl SensitiveCheckResult {
    /// Creates a deny result.
    pub fn deny(reason: &'static str) -> Self {
        Self {
            is_sensitive: true,
            reason: Some(reason),
            can_override: true, // Can be overridden with explicit permission
        }
    }

    /// Creates an allow result.
    pub fn allow() -> Self {
        Self {
            is_sensitive: false,
            reason: None,
            can_override: false,
        }
    }
}

/// Performs a comprehensive sensitive file check.
///
/// ## Arguments
///
/// * `path` - The file path to check
///
/// ## Returns
///
/// A `SensitiveCheckResult` with the details of the check.
pub fn check_sensitive<P: AsRef<Path>>(path: P) -> SensitiveCheckResult {
    let path = path.as_ref();

    if is_sensitive_path(path) {
        let reason = get_sensitive_reason(path).unwrap_or("Sensitive file pattern matched");
        return SensitiveCheckResult::deny(reason);
    }

    if is_sensitive_directory(path) {
        return SensitiveCheckResult::deny("Path is in a sensitive directory");
    }

    SensitiveCheckResult::allow()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sensitive_path_env() {
        assert!(is_sensitive_path(".env"));
        assert!(is_sensitive_path(".env.local"));
        assert!(is_sensitive_path(".env.production"));
        assert!(is_sensitive_path("/path/to/.env"));
        assert!(is_sensitive_path("C:\\Users\\test\\.env"));
    }

    #[test]
    fn test_is_sensitive_path_pem() {
        assert!(is_sensitive_path("server.pem"));
        assert!(is_sensitive_path("private.key"));
        assert!(is_sensitive_path("/etc/ssl/certs/server.crt"));
        assert!(is_sensitive_path("/path/to/credentials.pem"));
    }

    #[test]
    fn test_is_sensitive_path_credentials() {
        assert!(is_sensitive_path("credentials.json"));
        assert!(is_sensitive_path("credentials.yaml"));
        assert!(is_sensitive_path("/home/user/.config/credentials.json"));
    }

    #[test]
    fn test_is_sensitive_path_secrets() {
        assert!(is_sensitive_path("secrets.json"));
        assert!(is_sensitive_path("secrets.yml"));
        assert!(is_sensitive_path("secret.token"));
        assert!(is_sensitive_path(".secrets"));
    }

    #[test]
    fn test_is_not_sensitive() {
        assert!(!is_sensitive_path("source_code.rs"));
        assert!(!is_sensitive_path("README.md"));
        assert!(!is_sensitive_path("Cargo.toml"));
        assert!(!is_sensitive_path("src/main.rs"));
        assert!(!is_sensitive_path("tests/test_file.py"));
    }

    #[test]
    fn test_sensitive_extensions() {
        assert!(is_sensitive_path("keystore.jks"));
        assert!(is_sensitive_path("truststore.keystore"));
        assert!(is_sensitive_path("cert.p12"));
    }

    #[test]
    fn test_aws_credentials_path() {
        assert!(is_sensitive_path("/home/user/.aws/credentials"));
        assert!(is_sensitive_path("/root/.aws/config"));
        assert!(is_sensitive_path(".aws/credentials"));
    }

    #[test]
    fn test_gcp_credentials_path() {
        assert!(is_sensitive_path(
            "/home/user/.config/gcloud/application_credentials.json"
        ));
        assert!(is_sensitive_path("~/.gcloud/application_credentials.json"));
    }

    #[test]
    fn test_is_sensitive_directory() {
        assert!(is_sensitive_directory("/etc/ssh/some_file"));
        assert!(is_sensitive_directory("/etc/ssl/private/key"));
        assert!(is_sensitive_directory("/root/.ssh/authorized_keys"));
        assert!(is_sensitive_directory("/run/secrets/secret"));
    }

    #[test]
    fn test_is_not_sensitive_directory() {
        assert!(!is_sensitive_directory("/home/user/project/src/main.rs"));
        assert!(!is_sensitive_directory("/var/www/html/index.html"));
        assert!(!is_sensitive_directory("/tmp/project/file.txt"));
    }

    #[test]
    fn test_get_sensitive_reason() {
        assert_eq!(
            get_sensitive_reason(".env"),
            Some(".env files contain environment variables and secrets")
        );
        assert_eq!(
            get_sensitive_reason("credentials.json"),
            Some("Credentials files contain authentication data")
        );
        assert_eq!(get_sensitive_reason("source_code.rs"), None);
    }

    #[test]
    fn test_check_sensitive_allow() {
        let result = check_sensitive("source_code.rs");
        assert!(!result.is_sensitive);
        assert!(result.reason.is_none());
    }

    #[test]
    fn test_check_sensitive_deny() {
        let result = check_sensitive(".env");
        assert!(result.is_sensitive);
        assert!(result.reason.is_some());
        assert!(result.can_override);
    }

    #[test]
    fn test_can_override_for_sensitive() {
        let result = check_sensitive(".env");
        assert!(result.can_override);
    }
}
