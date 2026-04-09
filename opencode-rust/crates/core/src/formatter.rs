use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use glob::Pattern;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{info, warn};

use crate::config::{FormatterConfig, FormatterEntry};

#[derive(Debug, thiserror::Error)]
pub enum FormatterError {
    #[error("Formatter disabled")]
    Disabled,
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Timeout after {0:?}")]
    Timeout(Duration),
    #[error("No matching formatter for {0}")]
    NoMatch(String),
}

pub struct FormatterEngine {
    config: HashMap<String, FormatterEntry>,
    timeout: Duration,
    enabled: bool,
}

impl FormatterEngine {
    pub fn new(config: FormatterConfig) -> Self {
        let (enabled, config) = match config {
            FormatterConfig::Disabled(value) => (value, HashMap::new()),
            FormatterConfig::Formatters(formatters) => (true, formatters),
        };

        Self {
            config,
            timeout: Duration::from_secs(10),
            enabled,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn match_formatters(&self, file_path: &str) -> Vec<&FormatterEntry> {
        if !self.is_enabled() {
            return vec![];
        }

        let mut matches = self
            .config
            .iter()
            .filter(|(_, entry)| !entry.disabled.unwrap_or(false))
            .filter(|(_, entry)| formatter_matches_file(entry, file_path))
            .collect::<Vec<_>>();

        matches.sort_by(|(left_name, _), (right_name, _)| left_name.cmp(right_name));

        matches.into_iter().map(|(_, entry)| entry).collect()
    }

    pub async fn format_file(
        &self,
        file_path: &str,
        project_root: &Path,
    ) -> Result<(), FormatterError> {
        if !self.is_enabled() {
            return Err(FormatterError::Disabled);
        }

        let matched = self.match_formatters(file_path);
        if matched.is_empty() {
            return Err(FormatterError::NoMatch(file_path.to_string()));
        }

        for formatter in matched {
            let Some(command) = formatter.command.as_ref() else {
                warn!(file_path, "formatter missing command; skipping");
                continue;
            };

            if command.is_empty() {
                warn!(file_path, "formatter command is empty; skipping");
                continue;
            }

            let executable = &command[0];
            let args = command[1..]
                .iter()
                .map(|arg| arg.replace("$FILE", file_path))
                .collect::<Vec<_>>();

            let mut cmd = Command::new(executable);
            cmd.args(&args).current_dir(project_root);

            if let Some(environment) = formatter.environment.as_ref() {
                cmd.envs(environment);
            }

            match cmd.spawn() {
                Ok(mut child) => match timeout(self.timeout, child.wait()).await {
                    Ok(Ok(status)) if status.success() => {
                        info!(file_path, executable, "formatter executed successfully");
                    }
                    Ok(Ok(status)) => {
                        warn!(
                            file_path,
                            executable,
                            status = %status,
                            "formatter failed with non-zero status; continuing"
                        );
                    }
                    Ok(Err(err)) => {
                        warn!(
                            file_path,
                            executable,
                            error = %err,
                            "formatter process wait failed; continuing"
                        );
                    }
                    Err(_) => {
                        let _ = child.kill().await;
                        warn!(
                            file_path,
                            executable,
                            timeout = ?self.timeout,
                            "formatter timed out; continuing"
                        );
                    }
                },
                Err(err) => {
                    warn!(
                        file_path,
                        executable,
                        error = %err,
                        "failed to spawn formatter command; continuing"
                    );
                }
            }
        }

        Ok(())
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }
}

fn formatter_matches_file(formatter: &FormatterEntry, file_path: &str) -> bool {
    let Some(patterns) = formatter.extensions.as_ref() else {
        return false;
    };

    let path = Path::new(file_path);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();

    patterns.iter().any(|pattern| {
        let pattern = pattern.trim();
        if pattern.is_empty() {
            return false;
        }

        if pattern == extension || pattern.trim_start_matches('.') == extension {
            return true;
        }

        Pattern::new(pattern)
            .map(|glob| glob.matches(file_name) || glob.matches(file_path))
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use tempfile::tempdir;

    fn formatter_entry(command: Vec<String>, extensions: Vec<String>) -> FormatterEntry {
        FormatterEntry {
            disabled: None,
            command: Some(command),
            environment: None,
            extensions: Some(extensions),
        }
    }

    #[test]
    fn match_formatters_matches_typescript_extension() {
        let mut config = HashMap::new();
        config.insert(
            "typescript".to_string(),
            formatter_entry(
                vec!["true".to_string()],
                vec!["ts".to_string(), "tsx".to_string()],
            ),
        );
        config.insert(
            "python".to_string(),
            formatter_entry(vec!["true".to_string()], vec!["py".to_string()]),
        );

        let engine = FormatterEngine::new(FormatterConfig::Formatters(config));
        let matched = engine.match_formatters("src/app.ts");

        assert_eq!(matched.len(), 1);
        assert_eq!(
            matched[0].extensions.as_ref().unwrap(),
            &vec!["ts".to_string(), "tsx".to_string()]
        );
    }

    #[tokio::test]
    async fn format_file_executes_command_with_file_replaced() {
        let temp = tempdir().unwrap();
        let source_file = temp.path().join("main.ts");
        let marker = temp.path().join("replaced.txt");
        fs::write(&source_file, "const x = 1;\n").unwrap();

        let mut config = HashMap::new();
        config.insert(
            "typescript".to_string(),
            formatter_entry(
                vec![
                    "sh".to_string(),
                    "-c".to_string(),
                    format!(
                        "if [ \"$1\" = \"{}\" ]; then touch \"{}\"; fi",
                        source_file.display(),
                        marker.display()
                    ),
                    "sh".to_string(),
                    "$FILE".to_string(),
                ],
                vec!["ts".to_string()],
            ),
        );

        let engine = FormatterEngine::new(FormatterConfig::Formatters(config));
        engine
            .format_file(source_file.to_str().unwrap(), temp.path())
            .await
            .unwrap();

        assert!(marker.exists());
    }

    #[test]
    fn disabled_config_reports_not_enabled() {
        let engine = FormatterEngine::new(FormatterConfig::Disabled(false));
        assert!(!engine.is_enabled());
    }

    #[tokio::test]
    async fn multiple_formatters_execute_in_order() {
        let temp = tempdir().unwrap();
        let source_file = temp.path().join("main.ts");
        let order_file = temp.path().join("order.log");
        fs::write(&source_file, "const x = 1;\n").unwrap();

        let mut config = HashMap::new();
        config.insert(
            "a-first".to_string(),
            formatter_entry(
                vec![
                    "sh".to_string(),
                    "-c".to_string(),
                    format!("echo first >> \"{}\"", order_file.display()),
                ],
                vec!["ts".to_string()],
            ),
        );
        config.insert(
            "b-second".to_string(),
            formatter_entry(
                vec![
                    "sh".to_string(),
                    "-c".to_string(),
                    format!("echo second >> \"{}\"", order_file.display()),
                ],
                vec!["ts".to_string()],
            ),
        );

        let engine = FormatterEngine::new(FormatterConfig::Formatters(config));
        engine
            .format_file(source_file.to_str().unwrap(), temp.path())
            .await
            .unwrap();

        let content = fs::read_to_string(order_file).unwrap();
        let lines = content.lines().collect::<Vec<_>>();
        assert_eq!(lines, vec!["first", "second"]);
    }

    #[tokio::test]
    async fn timeout_is_enforced_and_failure_is_not_fatal() {
        let temp = tempdir().unwrap();
        let source_file = temp.path().join("main.ts");
        let marker = temp.path().join("should_not_exist.txt");
        fs::write(&source_file, "const x = 1;\n").unwrap();

        let mut config = HashMap::new();
        config.insert(
            "slow".to_string(),
            formatter_entry(
                vec![
                    "sh".to_string(),
                    "-c".to_string(),
                    format!("sleep 1; touch \"{}\"", marker.display()),
                ],
                vec!["ts".to_string()],
            ),
        );

        let mut engine = FormatterEngine::new(FormatterConfig::Formatters(config));
        engine.set_timeout(Duration::from_millis(50));

        let result = engine
            .format_file(source_file.to_str().unwrap(), temp.path())
            .await;

        assert!(result.is_ok());
        assert!(!marker.exists());
    }
}
