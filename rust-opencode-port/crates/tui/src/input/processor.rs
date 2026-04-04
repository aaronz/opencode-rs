use std::fmt::{Display, Formatter};
use std::io::Read;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;

use crate::command::CommandRegistry;

const MAX_FILE_SIZE: u64 = 100 * 1024;

#[derive(Debug)]
pub enum InputProcessorError {
    Io(std::io::Error),
    CommandNotFound(String),
    ShellTimeout,
    ShellFailed(String),
}

impl Display for InputProcessorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InputProcessorError::Io(error) => write!(f, "io error: {error}"),
            InputProcessorError::CommandNotFound(name) => write!(f, "unknown command: /{name}"),
            InputProcessorError::ShellTimeout => write!(f, "shell command timed out"),
            InputProcessorError::ShellFailed(error) => write!(f, "shell command failed: {error}"),
        }
    }
}

impl std::error::Error for InputProcessorError {}

impl From<std::io::Error> for InputProcessorError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone)]
pub struct InputProcessor {
    timeout: Duration,
}

impl Default for InputProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl InputProcessor {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(15),
        }
    }

    pub fn process_files(&self, files: &[PathBuf]) -> Result<String, InputProcessorError> {
        let mut context = String::new();
        for file in files {
            let metadata = std::fs::metadata(file)?;
            if !metadata.is_file() {
                continue;
            }

            let mut content = String::new();
            let mut warning = None;
            if metadata.len() > MAX_FILE_SIZE {
                let mut handle = std::fs::File::open(file)?;
                let mut buffer = vec![0_u8; MAX_FILE_SIZE as usize];
                let bytes_read = handle.read(&mut buffer)?;
                content = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
                warning = Some(format!(
                    "[warning] file truncated to {}KB (original {} bytes)",
                    MAX_FILE_SIZE / 1024,
                    metadata.len()
                ));
            } else {
                content = std::fs::read_to_string(file)?;
            }

            context.push_str(&format!("\nFile: @{}\n", file.display()));
            if let Some(warning) = warning {
                context.push_str(&format!("{warning}\n"));
            }
            context.push_str(&content);
            context.push('\n');
        }
        Ok(context)
    }

    pub fn process_shell(&self, cmd: &str) -> Result<String, InputProcessorError> {
        Ok(format!(
            "Shell command preview:\n$ {cmd}\nConfirm with /confirm-shell"
        ))
    }

    pub fn process_shell_confirmed(&self, cmd: &str) -> Result<String, InputProcessorError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(InputProcessorError::Io)?;

        runtime.block_on(async {
            let child = if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .args(["/C", cmd])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
            } else {
                Command::new("sh")
                    .args(["-c", cmd])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
            }
            .map_err(InputProcessorError::Io)?;

            let output = tokio::time::timeout(self.timeout, child.wait_with_output())
                .await
                .map_err(|_| InputProcessorError::ShellTimeout)?
                .map_err(InputProcessorError::Io)?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let code = output.status.code().unwrap_or(-1);
            if output.status.success() {
                Ok(format!("$ {cmd}\n{stdout}"))
            } else {
                Err(InputProcessorError::ShellFailed(format!(
                    "exit code {code}\n{stderr}"
                )))
            }
        })
    }

    pub fn process_command(
        &self,
        registry: &CommandRegistry,
        name: &str,
        args: &str,
    ) -> Result<String, InputProcessorError> {
        if registry.get_by_name(name).is_none() {
            return Err(InputProcessorError::CommandNotFound(name.to_string()));
        }
        if args.is_empty() {
            Ok(format!("/{name}"))
        } else {
            Ok(format!("/{name} {args}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn process_files_reads_content() {
        let temp = tempdir().expect("tempdir");
        let file = temp.path().join("a.txt");
        fs::write(&file, "hello").expect("write file");

        let processor = InputProcessor::new();
        let output = processor
            .process_files(&[file])
            .expect("process files output");
        assert!(output.contains("hello"));
    }

    #[test]
    fn process_command_routes_to_registry() {
        let processor = InputProcessor::new();
        let registry = CommandRegistry::new();
        let output = processor
            .process_command(&registry, "help", "")
            .expect("process command output");
        assert_eq!(output, "/help");
    }
}
