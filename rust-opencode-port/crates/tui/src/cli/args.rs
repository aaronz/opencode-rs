use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "opencode-rs")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct CliArgs {
    #[arg(default_value = ".")]
    pub directory: String,

    #[arg(short, long, alias = "m")]
    pub model: Option<String>,

    #[arg(long, alias = "resume")]
    pub session_id: Option<String>,

    #[arg(long, value_enum, default_value = "workspace-write")]
    pub permission_mode: PermissionMode,

    #[arg(long, hide = true)]
    pub dangerously_skip_permissions: bool,

    #[arg(long, value_enum, default_value = "text")]
    pub output_format: OutputFormat,

    #[arg(long)]
    pub allowed_tools: Option<String>,

    #[arg(short, long)]
    pub non_interactive: bool,

    #[arg(long)]
    pub provider: Option<String>,

    #[arg(long)]
    pub temperature: Option<f64>,

    #[arg(long)]
    pub max_tokens: Option<u32>,
}

impl CliArgs {
    pub fn validate_directory(&self) -> Result<PathBuf, String> {
        let path = PathBuf::from(&self.directory);

        if !path.exists() {
            return Err(format!(
                "Directory does not exist: {}\n\
                \nHint: Use a valid path or create the directory first.\n\
                Example: opencode-rs /path/to/project",
                self.directory
            ));
        }

        if !path.is_dir() {
            return Err(format!(
                "Path is not a directory: {}\n\
                \nHint: Please provide a directory path, not a file.",
                self.directory
            ));
        }

        let _ = match std::fs::canonicalize(&path) {
            Ok(p) => p,
            Err(e) => {
                return Err(format!(
                    "Cannot resolve directory path: {}\n\
                    \nError: {}\n\
                    \nHint: Check if you have proper permissions to access this path.",
                    self.directory, e
                ));
            }
        };

        let path_str = self.directory.replace('\\', "/");
        if path_str.contains("..") {
            return Err(format!(
                "Path traversal detected: {}\n\
                \nHint: Avoid using '..' in paths. Use absolute paths for better reliability.",
                self.directory
            ));
        }

        if path.is_symlink() {
            let target =
                std::fs::read_link(&path).map_err(|e| format!("Symlink is broken: {}", e))?;
            if !target.is_absolute() {
                return Err(format!(
                    "Symlink target must be absolute: {} -> {}",
                    self.directory,
                    target.display()
                ));
            }
        }

        Ok(path)
    }

    pub fn resolved_directory(&self) -> Result<PathBuf, String> {
        let path = self.validate_directory()?;

        match std::fs::canonicalize(&path) {
            Ok(resolved) => Ok(resolved),
            Err(_) => Ok(path),
        }
    }

    pub fn validate_model(&self) -> Result<(), String> {
        if let Some(ref model) = self.model {
            if model.is_empty() {
                return Err("Model name cannot be empty".to_string());
            }
            if model.len() > 100 {
                return Err("Model name too long (max 100 characters)".to_string());
            }
        }
        Ok(())
    }

    pub fn validate_temperature(&self) -> Result<(), String> {
        if let Some(temp) = self.temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err("Temperature must be between 0.0 and 2.0".to_string());
            }
        }
        Ok(())
    }

    pub fn validate_max_tokens(&self) -> Result<(), String> {
        if let Some(tokens) = self.max_tokens {
            if tokens == 0 {
                return Err("max_tokens must be greater than 0".to_string());
            }
            if tokens > 100000 {
                return Err("max_tokens exceeds maximum allowed (100000)".to_string());
            }
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if let Err(e) = self.validate_directory() {
            errors.push(e);
        }
        if let Err(e) = self.validate_model() {
            errors.push(e);
        }
        if let Err(e) = self.validate_temperature() {
            errors.push(e);
        }
        if let Err(e) = self.validate_max_tokens() {
            errors.push(e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum PermissionMode {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}
