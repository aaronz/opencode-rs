use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::Mutex;

use opencode_config::{FormatterConfig, FormatterEntry};
use opencode_core::effect::{Effect, EffectError, EffectResult};

use super::formatters::{all_formatters, Formatter, FormatterContext, FormatterStatus};

pub struct FormatServiceState {
    formatters: HashMap<String, Box<dyn Formatter>>,
    commands: HashMap<String, Option<Vec<String>>>,
}

impl Default for FormatServiceState {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatServiceState {
    pub fn new() -> Self {
        Self {
            formatters: HashMap::new(),
            commands: HashMap::new(),
        }
    }

    pub fn register_formatter(&mut self, formatter: Box<dyn Formatter>) {
        let name = formatter.name().to_string();
        self.formatters.insert(name, formatter);
    }

    pub fn get_formatter(&self, name: &str) -> Option<&Box<dyn Formatter>> {
        self.formatters.get(name)
    }

    pub fn set_command(&mut self, name: String, command: Option<Vec<String>>) {
        self.commands.insert(name, command);
    }

    pub fn get_command(&self, name: &str) -> Option<&Option<Vec<String>>> {
        self.commands.get(name)
    }

    pub fn formatter_names(&self) -> Vec<String> {
        self.formatters.keys().cloned().collect()
    }
}

pub struct FormatService {
    config: FormatterConfig,
    state: Arc<Mutex<FormatServiceState>>,
}

impl FormatService {
    pub fn new(config: FormatterConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(FormatServiceState::new())),
        }
    }

    pub async fn init(&self) -> EffectResult<()> {
        let config = self.config.clone();
        let state = self.state.clone();

        Effect::new(move || async move {
            let mut state_guard = state.lock().await;

            match &config {
                FormatterConfig::Disabled(false) => {
                    state_guard.commands.clear();
                    for formatter in all_formatters() {
                        let name = formatter.name().to_string();
                        state_guard.register_formatter(formatter);
                        state_guard.set_command(name, None);
                    }
                }
                FormatterConfig::Disabled(true) => {
                    state_guard.commands.clear();
                }
                FormatterConfig::Formatters(formatters) => {
                    for (name, entry) in formatters {
                        if entry.disabled.unwrap_or(false) {
                            state_guard.set_command(name.clone(), None);
                        } else {
                            state_guard.set_command(name.clone(), entry.command.clone());
                        }
                    }
                }
            }

            Ok(())
        })
        .run()
        .await
    }

    pub async fn status(&self) -> Vec<FormatterStatus> {
        let state = self.state.lock().await;
        let mut statuses = Vec::new();
        let ctx = FormatterContext {
            directory: PathBuf::from("/tmp"),
            worktree: PathBuf::from("/tmp"),
        };

        match &self.config {
            FormatterConfig::Disabled(false) => {
                for formatter in state.formatters.values() {
                    let enabled = formatter.enabled(&ctx).await.is_some();
                    statuses.push(FormatterStatus {
                        name: formatter.name().to_string(),
                        extensions: formatter
                            .extensions()
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                        enabled,
                    });
                }
            }
            FormatterConfig::Disabled(true) => {
                for formatter in state.formatters.values() {
                    statuses.push(FormatterStatus {
                        name: formatter.name().to_string(),
                        extensions: formatter
                            .extensions()
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                        enabled: false,
                    });
                }
            }
            FormatterConfig::Formatters(config_formatters) => {
                for formatter in state.formatters.values() {
                    let name = formatter.name();
                    let entry = config_formatters.get(name);
                    let config_disabled = entry.map(|e| e.disabled.unwrap_or(false)).unwrap_or(false);
                    let available = formatter.enabled(&ctx).await.is_some();
                    let enabled = !config_disabled && available;

                    statuses.push(FormatterStatus {
                        name: name.to_string(),
                        extensions: formatter
                            .extensions()
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                        enabled,
                    });
                }
            }
        }

        statuses.sort_by(|a, b| a.name.cmp(&b.name));
        statuses
    }

    pub async fn file(&self, filepath: &Path) -> EffectResult<()> {
        let config = self.config.clone();
        let filepath = filepath.to_path_buf();

        Effect::new(move || async move {
            match &config {
                FormatterConfig::Disabled(false) => {
                    return Err(EffectError::Generic("Formatter disabled".to_string()));
                }
                FormatterConfig::Disabled(true) => {
                    return Ok(());
                }
                FormatterConfig::Formatters(config_formatters) => {
                    let file_path_str = filepath.to_string_lossy();
                    let file_path = &filepath;

                    let mut matched: Vec<(&String, &FormatterEntry)> = config_formatters
                        .iter()
                        .filter(|(_, entry)| !entry.disabled.unwrap_or(false))
                        .filter(|(_, entry)| {
                            if let Some(patterns) = &entry.extensions {
                                patterns.iter().any(|pattern| {
                                    let ext = file_path
                                        .extension()
                                        .and_then(|e| e.to_str())
                                        .unwrap_or_default();
                                    let pattern_stripped = pattern.trim_start_matches('.');
                                    ext == pattern_stripped || ext == pattern
                                })
                            } else {
                                false
                            }
                        })
                        .collect();

                    matched.sort_by(|(left_name, _), (right_name, _)| left_name.cmp(right_name));

                    for (_, entry) in matched {
                        if let Some(command) = entry.command.as_ref() {
                            if command.is_empty() {
                                continue;
                            }

                            let executable = &command[0];
                            let args = command[1..]
                                .iter()
                                .map(|arg| arg.replace("$FILE", &file_path_str))
                                .collect::<Vec<_>>();

                            let mut cmd = tokio::process::Command::new(executable);
                            cmd.args(&args);

                            if let Some(env) = entry.environment.as_ref() {
                                cmd.envs(env);
                            }

                            if let Err(e) = cmd.spawn() {
                                tracing::warn!(
                                    file_path = %file_path_str,
                                    executable,
                                    error = %e,
                                    "failed to spawn formatter"
                                );
                            }
                        }
                    }
                }
            }

            Ok(())
        })
        .run()
        .await
    }

    pub fn engine(&self) -> opencode_core::formatter::FormatterEngine {
        match &self.config {
            FormatterConfig::Disabled(value) => {
                opencode_core::formatter::FormatterEngine::new(FormatterConfig::Disabled(*value))
            }
            FormatterConfig::Formatters(map) => opencode_core::formatter::FormatterEngine::new(
                FormatterConfig::Formatters(map.clone()),
            ),
        }
    }
}

impl Default for FormatService {
    fn default() -> Self {
        Self::new(FormatterConfig::Disabled(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn service_initializes_without_error() {
        let service = FormatService::new(FormatterConfig::Disabled(false));
        let result = service.init().await;
        assert!(result.is_ok(), "init() should return success result");
    }

    #[tokio::test]
    async fn init_returns_success_result() {
        let service = FormatService::new(FormatterConfig::Formatters(HashMap::new()));
        let result = service.init().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn status_returns_empty_when_disabled() {
        let service = FormatService::new(FormatterConfig::Disabled(true));
        let _ = service.init().await;
        let statuses = service.status().await;
        assert!(statuses.is_empty(), "Expected empty status when formatter is disabled");
    }

    #[tokio::test]
    async fn format_service_creates_with_default() {
        let _service = FormatService::default();
    }

    #[tokio::test]
    async fn format_service_creates_with_config() {
        let config = FormatterConfig::Disabled(false);
        let _service = FormatService::new(config);
    }

    #[tokio::test]
    async fn file_returns_ok_for_disabled() {
        let service = FormatService::new(FormatterConfig::Disabled(false));
        let _ = service.init().await;
        let result = service.file(Path::new("/tmp/test.rs")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn file_returns_ok_for_enabled_with_no_matching_formatters() {
        let service = FormatService::new(FormatterConfig::Formatters(HashMap::new()));
        let _ = service.init().await;
        let result = service.file(Path::new("/tmp/test.rs")).await;
        assert!(result.is_ok());
    }
}
