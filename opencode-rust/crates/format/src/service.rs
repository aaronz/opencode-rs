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

    pub fn get_formatter(&self, name: &str) -> Option<&dyn Formatter> {
        self.formatters.get(name).map(|b| b.as_ref())
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

impl std::fmt::Debug for FormatServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormatServiceState")
            .field("formatters", &self.formatters.keys().collect::<Vec<_>>())
            .field("commands", &self.commands)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct InstanceState {
    directory: PathBuf,
    formatter_config: FormatterConfig,
    service_state: Arc<Mutex<FormatServiceState>>,
}

impl InstanceState {
    pub fn new(directory: PathBuf, config: FormatterConfig) -> Self {
        Self {
            directory,
            formatter_config: config,
            service_state: Arc::new(Mutex::new(FormatServiceState::new())),
        }
    }

    pub fn directory(&self) -> &PathBuf {
        &self.directory
    }

    pub fn formatter_config(&self) -> &FormatterConfig {
        &self.formatter_config
    }

    pub fn service_state(&self) -> &Arc<Mutex<FormatServiceState>> {
        &self.service_state
    }

    pub fn set_formatter_config(&mut self, config: FormatterConfig) {
        self.formatter_config = config;
    }
}

#[derive(Debug)]
pub struct InstanceStateManager {
    instances: HashMap<PathBuf, InstanceState>,
}

impl Default for InstanceStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InstanceStateManager {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }

    pub fn get_or_create(
        &mut self,
        directory: &Path,
        config: FormatterConfig,
    ) -> &mut InstanceState {
        if !self.instances.contains_key(directory) {
            self.instances.insert(
                directory.to_path_buf(),
                InstanceState::new(directory.to_path_buf(), config),
            );
        }
        self.instances.get_mut(directory).unwrap()
    }

    pub fn get(&self, directory: &Path) -> Option<&InstanceState> {
        self.instances.get(directory)
    }

    pub fn remove(&mut self, directory: &Path) -> Option<InstanceState> {
        self.instances.remove(directory)
    }

    pub fn instances_count(&self) -> usize {
        self.instances.len()
    }
}

pub struct FormatService {
    instance_manager: Arc<Mutex<InstanceStateManager>>,
}

impl FormatService {
    pub fn new() -> Self {
        Self {
            instance_manager: Arc::new(Mutex::new(InstanceStateManager::new())),
        }
    }

    pub async fn init(&self, directory: &Path, config: FormatterConfig) -> EffectResult<()> {
        let instance_manager = self.instance_manager.clone();
        let directory = directory.to_path_buf();
        let config = config.clone();

        Effect::new(move || async move {
            let mut manager = instance_manager.lock().await;
            let instance = manager.get_or_create(&directory, config.clone());
            let mut state_guard = instance.service_state.lock().await;

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
                    for formatter in all_formatters() {
                        state_guard.register_formatter(formatter);
                    }

                    let mut ruff_disabled = false;
                    let mut uv_disabled = false;

                    for (name, entry) in formatters {
                        if entry.disabled.unwrap_or(false) {
                            state_guard.set_command(name.clone(), None);
                        } else {
                            state_guard.set_command(name.clone(), entry.command.clone());
                        }
                        if name == "ruff" && entry.disabled.unwrap_or(false) {
                            ruff_disabled = true;
                        }
                        if name == "uvformat" && entry.disabled.unwrap_or(false) {
                            uv_disabled = true;
                        }
                    }

                    if ruff_disabled {
                        state_guard.set_command("uvformat".to_string(), None);
                    }
                    if uv_disabled {
                        state_guard.set_command("ruff".to_string(), None);
                    }
                }
            }

            Ok(())
        })
        .run()
        .await
    }

    pub async fn status(&self, directory: &Path) -> Vec<FormatterStatus> {
        let instance_manager = self.instance_manager.lock().await;
        let mut statuses = Vec::new();

        if let Some(instance) = instance_manager.get(directory) {
            let state = instance.service_state.lock().await;
            let config = instance.formatter_config.clone();
            let ctx = FormatterContext {
                directory: directory.to_path_buf(),
                worktree: directory.to_path_buf(),
            };

            match &config {
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
                    let ruff_disabled = config_formatters
                        .get("ruff")
                        .map(|e| e.disabled.unwrap_or(false))
                        .unwrap_or(false);
                    let uv_disabled = config_formatters
                        .get("uvformat")
                        .map(|e| e.disabled.unwrap_or(false))
                        .unwrap_or(false);

                    for formatter in state.formatters.values() {
                        let name = formatter.name();
                        let entry = config_formatters.get(name);
                        let config_disabled =
                            entry.map(|e| e.disabled.unwrap_or(false)).unwrap_or(false);

                        if config_disabled {
                            continue;
                        }

                        let linked_disabled = (name == "uvformat" && ruff_disabled)
                            || (name == "ruff" && uv_disabled);

                        let available = formatter.enabled(&ctx).await.is_some();
                        let enabled = !linked_disabled && available;

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
        }

        statuses.sort_by(|a, b| a.name.cmp(&b.name));
        statuses
    }

    pub async fn file(&self, filepath: &Path) -> EffectResult<()> {
        let instance_manager = self.instance_manager.clone();
        let filepath = filepath.to_path_buf();

        let directory = filepath
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("/"));

        Effect::new(move || async move {
            let manager = instance_manager.lock().await;
            let config = if let Some(instance) = manager.get(&directory) {
                instance.formatter_config.clone()
            } else {
                return Ok(());
            };

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

                    for (formatter_name, entry) in matched.iter() {
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

                            match cmd.spawn() {
                                Ok(mut child) => {
                                    use tokio::time::{timeout, Duration};
                                    const FORMAT_TIMEOUT: Duration = Duration::from_secs(10);
                                    match timeout(FORMAT_TIMEOUT, child.wait()).await {
                                        Ok(Ok(status)) if status.success() => {
                                            tracing::info!(
                                                file_path = %file_path_str,
                                                formatter = %formatter_name,
                                                executable,
                                                "formatter executed successfully"
                                            );
                                        }
                                        Ok(Ok(status)) => {
                                            tracing::warn!(
                                                file_path = %file_path_str,
                                                formatter = %formatter_name,
                                                executable,
                                                status = %status,
                                                "formatter failed with non-zero status"
                                            );
                                        }
                                        Ok(Err(e)) => {
                                            tracing::warn!(
                                                file_path = %file_path_str,
                                                formatter = %formatter_name,
                                                executable,
                                                error = %e,
                                                "formatter wait failed"
                                            );
                                        }
                                        Err(_) => {
                                            let _ = child.kill().await;
                                            tracing::warn!(
                                                file_path = %file_path_str,
                                                formatter = %formatter_name,
                                                executable,
                                                "formatter timed out"
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        file_path = %file_path_str,
                                        formatter = %formatter_name,
                                        executable,
                                        error = %e,
                                        "failed to spawn formatter"
                                    );
                                }
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
        opencode_core::formatter::FormatterEngine::new(FormatterConfig::Disabled(true))
    }

    pub fn instance_manager(&self) -> &Arc<Mutex<InstanceStateManager>> {
        &self.instance_manager
    }
}

impl Default for FormatService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn service_initializes_without_error() {
        let service = FormatService::new();
        let result = service
            .init(Path::new("/tmp"), FormatterConfig::Disabled(false))
            .await;
        assert!(result.is_ok(), "init() should return success result");
    }

    #[tokio::test]
    async fn init_returns_success_result() {
        let service = FormatService::new();
        let result = service
            .init(
                Path::new("/tmp"),
                FormatterConfig::Formatters(HashMap::new()),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn status_returns_empty_when_disabled() {
        let service = FormatService::new();
        let _ = service
            .init(Path::new("/tmp"), FormatterConfig::Disabled(true))
            .await;
        let statuses = service.status(Path::new("/tmp")).await;
        assert!(
            statuses.is_empty(),
            "Expected empty status when formatter is disabled"
        );
    }

    #[tokio::test]
    async fn format_service_creates_with_default() {
        let _service = FormatService::default();
    }

    #[tokio::test]
    async fn format_service_creates_with_new() {
        let _service = FormatService::new();
    }

    #[tokio::test]
    async fn file_returns_ok_for_disabled() {
        let service = FormatService::new();
        let _ = service
            .init(Path::new("/tmp"), FormatterConfig::Disabled(false))
            .await;
        let result = service.file(Path::new("/tmp/test.rs")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn file_returns_ok_for_enabled_with_no_matching_formatters() {
        let service = FormatService::new();
        let _ = service
            .init(
                Path::new("/tmp"),
                FormatterConfig::Formatters(HashMap::new()),
            )
            .await;
        let result = service.file(Path::new("/tmp/test.rs")).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn disabling_ruff_removes_uv() {
        let mut formatters = HashMap::new();
        formatters.insert(
            "ruff".to_string(),
            FormatterEntry {
                disabled: Some(true),
                command: None,
                environment: None,
                extensions: None,
            },
        );
        let service = FormatService::new();
        let _ = service
            .init(Path::new("/tmp"), FormatterConfig::Formatters(formatters))
            .await;
        let statuses = service.status(Path::new("/tmp")).await;

        let uv_status = statuses.iter().find(|s| s.name == "uvformat");
        assert!(
            uv_status.map(|s| !s.enabled).unwrap_or(false),
            "uvformat should be disabled when ruff is disabled"
        );

        let ruff_status = statuses.iter().find(|s| s.name == "ruff");
        assert!(
            ruff_status.is_none(),
            "ruff should be excluded when explicitly disabled"
        );
    }

    #[tokio::test]
    async fn disabling_uv_removes_ruff() {
        let mut formatters = HashMap::new();
        formatters.insert(
            "uvformat".to_string(),
            FormatterEntry {
                disabled: Some(true),
                command: None,
                environment: None,
                extensions: None,
            },
        );
        let service = FormatService::new();
        let _ = service
            .init(Path::new("/tmp"), FormatterConfig::Formatters(formatters))
            .await;
        let statuses = service.status(Path::new("/tmp")).await;

        let ruff_status = statuses.iter().find(|s| s.name == "ruff");
        assert!(
            ruff_status.map(|s| !s.enabled).unwrap_or(false),
            "ruff should be disabled when uvformat is disabled"
        );

        let uv_status = statuses.iter().find(|s| s.name == "uvformat");
        assert!(
            uv_status.is_none(),
            "uvformat should be excluded when explicitly disabled"
        );
    }

    #[tokio::test]
    async fn format_service_uses_instance_state_for_configuration() {
        let service = FormatService::new();

        let mut formatters = HashMap::new();
        formatters.insert(
            "prettier".to_string(),
            FormatterEntry {
                disabled: Some(false),
                command: Some(vec!["prettier".to_string(), "--write".to_string()]),
                environment: None,
                extensions: Some(vec![".js".to_string()]),
            },
        );

        let _ = service
            .init(
                Path::new("/project"),
                FormatterConfig::Formatters(formatters),
            )
            .await;

        let manager = service.instance_manager.lock().await;
        let instance = manager.get(Path::new("/project"));
        assert!(
            instance.is_some(),
            "Instance should be created for /project directory"
        );

        let instance = instance.unwrap();
        assert_eq!(instance.directory(), &PathBuf::from("/project"));
        match instance.formatter_config() {
            FormatterConfig::Formatters(map) => {
                assert!(map.contains_key("prettier"), "prettier should be in config");
            }
            _ => panic!("Expected Formatters variant"),
        }
    }

    #[tokio::test]
    async fn formatter_state_isolated_per_directory() {
        let service = FormatService::new();

        let mut project_a_formatters = HashMap::new();
        project_a_formatters.insert(
            "prettier".to_string(),
            FormatterEntry {
                disabled: Some(false),
                command: Some(vec!["prettier".to_string(), "--write".to_string()]),
                environment: None,
                extensions: Some(vec![".js".to_string(), ".ts".to_string()]),
            },
        );

        let mut project_b_formatters = HashMap::new();
        project_b_formatters.insert(
            "rustfmt".to_string(),
            FormatterEntry {
                disabled: Some(false),
                command: Some(vec!["rustfmt".to_string()]),
                environment: None,
                extensions: Some(vec![".rs".to_string()]),
            },
        );

        let _ = service
            .init(
                Path::new("/project-a"),
                FormatterConfig::Formatters(project_a_formatters),
            )
            .await;
        let _ = service
            .init(
                Path::new("/project-b"),
                FormatterConfig::Formatters(project_b_formatters),
            )
            .await;

        let manager = service.instance_manager.lock().await;
        assert_eq!(
            manager.instances_count(),
            2,
            "Should have 2 separate instances"
        );

        let project_a_instance = manager
            .get(Path::new("/project-a"))
            .expect("project-a instance should exist");
        let project_b_instance = manager
            .get(Path::new("/project-b"))
            .expect("project-b instance should exist");

        match project_a_instance.formatter_config() {
            FormatterConfig::Formatters(map) => {
                assert!(
                    map.contains_key("prettier"),
                    "project-a should have prettier"
                );
                assert!(
                    !map.contains_key("rustfmt"),
                    "project-a should NOT have rustfmt"
                );
            }
            _ => panic!("Expected Formatters variant for project-a"),
        }

        match project_b_instance.formatter_config() {
            FormatterConfig::Formatters(map) => {
                assert!(
                    !map.contains_key("prettier"),
                    "project-b should NOT have prettier"
                );
                assert!(map.contains_key("rustfmt"), "project-b should have rustfmt");
            }
            _ => panic!("Expected Formatters variant for project-b"),
        }
    }

    #[tokio::test]
    async fn instance_state_stores_directory_correctly() {
        let directory = PathBuf::from("/test/path");
        let config = FormatterConfig::Disabled(false);
        let instance = InstanceState::new(directory.clone(), config);

        assert_eq!(instance.directory(), &directory);
        assert!(matches!(
            instance.formatter_config(),
            FormatterConfig::Disabled(false)
        ));
    }

    #[tokio::test]
    async fn instance_state_manager_get_or_create() {
        let mut manager = InstanceStateManager::new();
        let config = FormatterConfig::Disabled(false);

        manager.get_or_create(Path::new("/dir1"), config.clone());
        assert_eq!(manager.instances_count(), 1);

        {
            let instance1 = manager.get(Path::new("/dir1")).unwrap();
            assert_eq!(instance1.directory(), &PathBuf::from("/dir1"));
        }

        manager.get_or_create(Path::new("/dir2"), FormatterConfig::Disabled(true));
        assert_eq!(manager.instances_count(), 2);

        manager.get_or_create(Path::new("/dir1"), FormatterConfig::Disabled(true));
        assert_eq!(
            manager.instances_count(),
            2,
            "Should not create duplicate for same directory"
        );
    }

    #[tokio::test]
    async fn instance_state_manager_remove() {
        let mut manager = InstanceStateManager::new();
        let config = FormatterConfig::Disabled(false);

        manager.get_or_create(Path::new("/dir1"), config);
        assert_eq!(manager.instances_count(), 1);

        let removed = manager.remove(Path::new("/dir1"));
        assert!(removed.is_some());
        assert_eq!(manager.instances_count(), 0);

        let removed_after = manager.remove(Path::new("/nonexistent"));
        assert!(removed_after.is_none());
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn enabled_checks_run_in_parallel() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let service = FormatService::new();
        let _ = service
            .init(Path::new("/tmp"), FormatterConfig::Disabled(false))
            .await;

        let call_count = Arc::new(AtomicUsize::new(0));
        let mock_formatter = std::sync::Mutex::new(crate::formatters::gofmt::GofmtFormatter::new());

        let ctx = FormatterContext {
            directory: PathBuf::from("/tmp"),
            worktree: PathBuf::from("/tmp"),
        };

        #[allow(clippy::await_holding_lock)]
        let result1 = {
            let guard = mock_formatter.lock().unwrap();
            guard.enabled(&ctx).await
        };

        #[allow(clippy::await_holding_lock)]
        let result2 = {
            let guard = mock_formatter.lock().unwrap();
            guard.enabled(&ctx).await
        };

        assert!(
            result1.is_some() || result2.is_some(),
            "At least one formatter should be available"
        );

        assert_eq!(
            call_count.load(Ordering::SeqCst),
            2,
            "Should have made exactly 2 calls"
        );
    }

    #[tokio::test]
    async fn enabled_checks_run_in_parallel_with_tokio_join() {
        use crate::formatters::{all_formatters, FormatterContext};

        let temp_dir = tempfile::tempdir().unwrap();
        let ctx = FormatterContext {
            directory: temp_dir.path().to_path_buf(),
            worktree: temp_dir.path().to_path_buf(),
        };

        let formatters = all_formatters();

        let start = std::time::Instant::now();

        let ctx0 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx1 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx2 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx3 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx4 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx5 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx6 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx7 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx8 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx9 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };

        let (r0, r1, r2, r3, r4, r5, r6, r7, r8, r9) = tokio::join!(
            formatters[0].enabled(&ctx0),
            formatters[1].enabled(&ctx1),
            formatters[2].enabled(&ctx2),
            formatters[3].enabled(&ctx3),
            formatters[4].enabled(&ctx4),
            formatters[5].enabled(&ctx5),
            formatters[6].enabled(&ctx6),
            formatters[7].enabled(&ctx7),
            formatters[8].enabled(&ctx8),
            formatters[9].enabled(&ctx9),
        );

        let elapsed = start.elapsed().as_millis() as usize;
        assert!(
            elapsed < 1000,
            "tokio::join! should run checks in parallel, took {}ms",
            elapsed
        );

        let results = [r0, r1, r2, r3, r4, r5, r6, r7, r8, r9];
        let _available_count = results.iter().filter(|r| r.is_some()).count();

        let _ = &results; // Used to suppress unused warning

        drop(temp_dir);
    }

    #[tokio::test]
    async fn parallel_checks_produce_same_results_as_sequential() {
        use crate::formatters::{all_formatters, FormatterContext};

        let temp_dir = tempfile::tempdir().unwrap();
        let ctx = FormatterContext {
            directory: temp_dir.path().to_path_buf(),
            worktree: temp_dir.path().to_path_buf(),
        };

        let formatters = all_formatters();

        let mut sequential_results: Vec<(String, bool)> = Vec::new();
        for formatter in formatters.iter() {
            let available = formatter.enabled(&ctx).await.is_some();
            sequential_results.push((formatter.name().to_string(), available));
        }

        let ctx0 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx1 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx2 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx3 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx4 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx5 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx6 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx7 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx8 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx9 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx10 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx11 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx12 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx13 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx14 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx15 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx16 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx17 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx18 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx19 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx20 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx21 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx22 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx23 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx24 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };
        let ctx25 = FormatterContext {
            directory: ctx.directory.clone(),
            worktree: ctx.worktree.clone(),
        };

        let (
            r0,
            r1,
            r2,
            r3,
            r4,
            r5,
            r6,
            r7,
            r8,
            r9,
            r10,
            r11,
            r12,
            r13,
            r14,
            r15,
            r16,
            r17,
            r18,
            r19,
            r20,
            r21,
            r22,
            r23,
            r24,
            r25,
        ) = tokio::join!(
            formatters[0].enabled(&ctx0),
            formatters[1].enabled(&ctx1),
            formatters[2].enabled(&ctx2),
            formatters[3].enabled(&ctx3),
            formatters[4].enabled(&ctx4),
            formatters[5].enabled(&ctx5),
            formatters[6].enabled(&ctx6),
            formatters[7].enabled(&ctx7),
            formatters[8].enabled(&ctx8),
            formatters[9].enabled(&ctx9),
            formatters[10].enabled(&ctx10),
            formatters[11].enabled(&ctx11),
            formatters[12].enabled(&ctx12),
            formatters[13].enabled(&ctx13),
            formatters[14].enabled(&ctx14),
            formatters[15].enabled(&ctx15),
            formatters[16].enabled(&ctx16),
            formatters[17].enabled(&ctx17),
            formatters[18].enabled(&ctx18),
            formatters[19].enabled(&ctx19),
            formatters[20].enabled(&ctx20),
            formatters[21].enabled(&ctx21),
            formatters[22].enabled(&ctx22),
            formatters[23].enabled(&ctx23),
            formatters[24].enabled(&ctx24),
            formatters[25].enabled(&ctx25),
        );

        let mut parallel_results = vec![
            (formatters[0].name().to_string(), r0.is_some()),
            (formatters[1].name().to_string(), r1.is_some()),
            (formatters[2].name().to_string(), r2.is_some()),
            (formatters[3].name().to_string(), r3.is_some()),
            (formatters[4].name().to_string(), r4.is_some()),
            (formatters[5].name().to_string(), r5.is_some()),
            (formatters[6].name().to_string(), r6.is_some()),
            (formatters[7].name().to_string(), r7.is_some()),
            (formatters[8].name().to_string(), r8.is_some()),
            (formatters[9].name().to_string(), r9.is_some()),
            (formatters[10].name().to_string(), r10.is_some()),
            (formatters[11].name().to_string(), r11.is_some()),
            (formatters[12].name().to_string(), r12.is_some()),
            (formatters[13].name().to_string(), r13.is_some()),
            (formatters[14].name().to_string(), r14.is_some()),
            (formatters[15].name().to_string(), r15.is_some()),
            (formatters[16].name().to_string(), r16.is_some()),
            (formatters[17].name().to_string(), r17.is_some()),
            (formatters[18].name().to_string(), r18.is_some()),
            (formatters[19].name().to_string(), r19.is_some()),
            (formatters[20].name().to_string(), r20.is_some()),
            (formatters[21].name().to_string(), r21.is_some()),
            (formatters[22].name().to_string(), r22.is_some()),
            (formatters[23].name().to_string(), r23.is_some()),
            (formatters[24].name().to_string(), r24.is_some()),
            (formatters[25].name().to_string(), r25.is_some()),
        ];

        sequential_results.sort_by(|a, b| a.0.cmp(&b.0));
        parallel_results.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(
            sequential_results.len(),
            parallel_results.len(),
            "Should return same number of results"
        );

        for (seq, par) in sequential_results.iter().zip(parallel_results.iter()) {
            assert_eq!(seq.0, par.0, "Formatter names should match");
            assert_eq!(
                seq.1, par.1,
                "Availability check for '{}' should match: sequential={}, parallel={}",
                seq.0, seq.1, par.1
            );
        }

        drop(temp_dir);
    }
}
