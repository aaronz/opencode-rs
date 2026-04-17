use clap::Parser;
use opencode_tui::cli::{CliArgs, OutputFormat, PermissionMode};
use tempfile::TempDir;

#[test]
fn test_cli_args_default_values() {
    let args = CliArgs::parse_from(["opencode-rs"]);
    assert_eq!(args.directory, ".");
    assert!(args.model.is_none());
    assert!(args.session_id.is_none());
    assert!(matches!(
        args.permission_mode,
        PermissionMode::WorkspaceWrite
    ));
    assert!(matches!(args.output_format, OutputFormat::Text));
    assert!(!args.non_interactive);
}

#[test]
fn test_cli_args_with_model() {
    let args = CliArgs::parse_from(["opencode-rs", "-m", "gpt-4"]);
    assert_eq!(args.model, Some("gpt-4".to_string()));
}

#[test]
fn test_cli_args_with_session_id() {
    let args = CliArgs::parse_from(["opencode-rs", "--session-id", "abc123"]);
    assert_eq!(args.session_id, Some("abc123".to_string()));
}

#[test]
fn test_cli_args_permission_mode() {
    let args = CliArgs::parse_from(["opencode-rs", "--permission-mode", "read-only"]);
    assert!(matches!(args.permission_mode, PermissionMode::ReadOnly));
}

#[test]
fn test_cli_args_output_format() {
    let args = CliArgs::parse_from(["opencode-rs", "--output-format", "json"]);
    assert!(matches!(args.output_format, OutputFormat::Json));
}

#[test]
fn test_cli_args_non_interactive() {
    let args = CliArgs::parse_from(["opencode-rs", "--non-interactive"]);
    assert!(args.non_interactive);
}

#[test]
fn test_cli_args_provider() {
    let args = CliArgs::parse_from(["opencode-rs", "--provider", "openai"]);
    assert_eq!(args.provider, Some("openai".to_string()));
}

#[test]
fn test_cli_args_temperature() {
    let args = CliArgs::parse_from(["opencode-rs", "--temperature", "0.7"]);
    assert_eq!(args.temperature, Some(0.7));
}

#[test]
fn test_cli_args_max_tokens() {
    let args = CliArgs::parse_from(["opencode-rs", "--max-tokens", "1000"]);
    assert_eq!(args.max_tokens, Some(1000));
}

#[test]
fn test_cli_args_validate_directory_valid() {
    let tmp = TempDir::new().unwrap();
    let args = CliArgs {
        directory: tmp.path().to_string_lossy().to_string(),
        model: None,
        session_id: None,
        permission_mode: PermissionMode::WorkspaceWrite,
        dangerously_skip_permissions: false,
        output_format: OutputFormat::Text,
        allowed_tools: None,
        non_interactive: false,
        provider: None,
        temperature: None,
        max_tokens: None,
    };
    assert!(args.validate_directory().is_ok());
}

#[test]
fn test_cli_args_validate_directory_not_exists() {
    let args = CliArgs {
        directory: "/nonexistent/path/12345".to_string(),
        model: None,
        session_id: None,
        permission_mode: PermissionMode::WorkspaceWrite,
        dangerously_skip_permissions: false,
        output_format: OutputFormat::Text,
        allowed_tools: None,
        non_interactive: false,
        provider: None,
        temperature: None,
        max_tokens: None,
    };
    let result = args.validate_directory();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_cli_args_validate_directory_is_file() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("file.txt"), "content").unwrap();
    let args = CliArgs {
        directory: tmp.path().join("file.txt").to_string_lossy().to_string(),
        model: None,
        session_id: None,
        permission_mode: PermissionMode::WorkspaceWrite,
        dangerously_skip_permissions: false,
        output_format: OutputFormat::Text,
        allowed_tools: None,
        non_interactive: false,
        provider: None,
        temperature: None,
        max_tokens: None,
    };
    let result = args.validate_directory();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not a directory"));
}

#[test]
fn test_cli_args_validate_directory_path_traversal() {
    let tmp = TempDir::new().unwrap();
    let parent = tmp.path().parent().unwrap();
    let traversal_path = parent.join("..").join("etc").join("passwd");
    let args = CliArgs {
        directory: traversal_path.to_string_lossy().to_string(),
        model: None,
        session_id: None,
        permission_mode: PermissionMode::WorkspaceWrite,
        dangerously_skip_permissions: false,
        output_format: OutputFormat::Text,
        allowed_tools: None,
        non_interactive: false,
        provider: None,
        temperature: None,
        max_tokens: None,
    };
    let result = args.validate_directory();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("traversal") || err.contains("does not exist"));
}

#[test]
fn test_cli_args_resolved_directory() {
    let tmp = TempDir::new().unwrap();
    let args = CliArgs {
        directory: tmp.path().to_string_lossy().to_string(),
        model: None,
        session_id: None,
        permission_mode: PermissionMode::WorkspaceWrite,
        dangerously_skip_permissions: false,
        output_format: OutputFormat::Text,
        allowed_tools: None,
        non_interactive: false,
        provider: None,
        temperature: None,
        max_tokens: None,
    };
    assert!(args.resolved_directory().is_ok());
}

#[test]
fn test_cli_args_validate_model_empty() {
    let args = CliArgs {
        directory: ".".to_string(),
        model: Some("".to_string()),
        session_id: None,
        permission_mode: PermissionMode::WorkspaceWrite,
        dangerously_skip_permissions: false,
        output_format: OutputFormat::Text,
        allowed_tools: None,
        non_interactive: false,
        provider: None,
        temperature: None,
        max_tokens: None,
    };
    let result = args.validate_model();
    assert!(result.is_err());
}

#[test]
fn test_cli_args_validate_model_too_long() {
    let args = CliArgs {
        directory: ".".to_string(),
        model: Some("a".repeat(101)),
        session_id: None,
        permission_mode: PermissionMode::WorkspaceWrite,
        dangerously_skip_permissions: false,
        output_format: OutputFormat::Text,
        allowed_tools: None,
        non_interactive: false,
        provider: None,
        temperature: None,
        max_tokens: None,
    };
    let result = args.validate_model();
    assert!(result.is_err());
}

#[test]
fn test_cli_args_validate_temperature_valid() {
    let args = CliArgs {
        directory: ".".to_string(),
        model: None,
        session_id: None,
        permission_mode: PermissionMode::WorkspaceWrite,
        dangerously_skip_permissions: false,
        output_format: OutputFormat::Text,
        allowed_tools: None,
        non_interactive: false,
        provider: None,
        temperature: Some(1.5),
        max_tokens: None,
    };
    assert!(args.validate_temperature().is_ok());
}

#[test]
fn test_cli_args_validate_temperature_out_of_range() {
    let args = CliArgs {
        directory: ".".to_string(),
        model: None,
        session_id: None,
        permission_mode: PermissionMode::WorkspaceWrite,
        dangerously_skip_permissions: false,
        output_format: OutputFormat::Text,
        allowed_tools: None,
        non_interactive: false,
        provider: None,
        temperature: Some(3.0),
        max_tokens: None,
    };
    let result = args.validate_temperature();
    assert!(result.is_err());
}

#[test]
fn test_cli_args_validate_max_tokens_zero() {
    let args = CliArgs {
        directory: ".".to_string(),
        model: None,
        session_id: None,
        permission_mode: PermissionMode::WorkspaceWrite,
        dangerously_skip_permissions: false,
        output_format: OutputFormat::Text,
        allowed_tools: None,
        non_interactive: false,
        provider: None,
        temperature: None,
        max_tokens: Some(0),
    };
    let result = args.validate_max_tokens();
    assert!(result.is_err());
}

#[test]
fn test_cli_args_validate_max_tokens_too_high() {
    let args = CliArgs {
        directory: ".".to_string(),
        model: None,
        session_id: None,
        permission_mode: PermissionMode::WorkspaceWrite,
        dangerously_skip_permissions: false,
        output_format: OutputFormat::Text,
        allowed_tools: None,
        non_interactive: false,
        provider: None,
        temperature: None,
        max_tokens: Some(200000),
    };
    let result = args.validate_max_tokens();
    assert!(result.is_err());
}

#[test]
fn test_cli_args_validate_success() {
    let tmp = TempDir::new().unwrap();
    let args = CliArgs {
        directory: tmp.path().to_string_lossy().to_string(),
        model: Some("gpt-4".to_string()),
        session_id: None,
        permission_mode: PermissionMode::WorkspaceWrite,
        dangerously_skip_permissions: false,
        output_format: OutputFormat::Text,
        allowed_tools: None,
        non_interactive: false,
        provider: None,
        temperature: Some(0.7),
        max_tokens: Some(1000),
    };
    assert!(args.validate().is_ok());
}

#[test]
fn test_cli_args_validate_multiple_errors() {
    let args = CliArgs {
        directory: "/nonexistent".to_string(),
        model: Some("".to_string()),
        session_id: None,
        permission_mode: PermissionMode::WorkspaceWrite,
        dangerously_skip_permissions: false,
        output_format: OutputFormat::Text,
        allowed_tools: None,
        non_interactive: false,
        provider: None,
        temperature: Some(5.0),
        max_tokens: Some(0),
    };
    let result = args.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.len() >= 3);
}

#[test]
fn test_permission_mode_variants() {
    assert!(matches!(PermissionMode::ReadOnly, PermissionMode::ReadOnly));
    assert!(matches!(
        PermissionMode::WorkspaceWrite,
        PermissionMode::WorkspaceWrite
    ));
    assert!(matches!(
        PermissionMode::DangerFullAccess,
        PermissionMode::DangerFullAccess
    ));
}

#[test]
fn test_output_format_variants() {
    assert!(matches!(OutputFormat::Text, OutputFormat::Text));
    assert!(matches!(OutputFormat::Json, OutputFormat::Json));
    assert!(matches!(OutputFormat::Ndjson, OutputFormat::Ndjson));
}

#[test]
fn test_cli_args_alias_m() {
    let args = CliArgs::parse_from(["opencode-rs", "-m", "claude-3"]);
    assert_eq!(args.model, Some("claude-3".to_string()));
}

#[test]
fn test_cli_args_alias_resume() {
    let args = CliArgs::parse_from(["opencode-rs", "--resume", "session-abc"]);
    assert_eq!(args.session_id, Some("session-abc".to_string()));
}

#[test]
fn test_cli_args_dangerously_skip_permissions() {
    let args = CliArgs::parse_from(["opencode-rs", "--dangerously-skip-permissions"]);
    assert!(args.dangerously_skip_permissions);
}

#[test]
fn test_cli_args_allowed_tools() {
    let args = CliArgs::parse_from(["opencode-rs", "--allowed-tools", "read,write,edit"]);
    assert_eq!(args.allowed_tools, Some("read,write,edit".to_string()));
}
