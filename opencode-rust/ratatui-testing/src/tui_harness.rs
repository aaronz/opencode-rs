use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum FakeProviderValidator {
    Success {
        models: Vec<FakeModel>,
    },
    Returns404,
    Returns401,
    ReturnsError(String),
    HangsForever,
    DelayedSuccess {
        delay: Duration,
        models: Vec<FakeModel>,
    },
    DelayedError {
        delay: Duration,
        error: String,
    },
    ManualControl {
        sender: Arc<Mutex<Option<tokio::sync::oneshot::Sender<FakeValidationResult>>>>,
        receiver: Arc<Mutex<Option<tokio::sync::oneshot::Receiver<FakeValidationResult>>>>,
    },
}

#[derive(Debug, Clone)]
pub struct FakeModel {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum FakeValidationResult {
    Success(Vec<FakeModel>),
    Error(String),
}

impl FakeProviderValidator {
    pub fn success() -> Self {
        FakeProviderValidator::Success {
            models: vec![
                FakeModel {
                    id: "MiniMax-M2.7".to_string(),
                    name: "MiniMax M2.7".to_string(),
                },
                FakeModel {
                    id: "MiniMax-M2.5".to_string(),
                    name: "MiniMax M2.5".to_string(),
                },
            ],
        }
    }

    pub fn returns_404() -> Self {
        FakeProviderValidator::Returns404
    }

    pub fn returns_401() -> Self {
        FakeProviderValidator::Returns401
    }

    pub fn returns_error(error: impl Into<String>) -> Self {
        FakeProviderValidator::ReturnsError(error.into())
    }

    pub fn hangs_forever() -> Self {
        FakeProviderValidator::HangsForever
    }

    pub fn delayed_success(delay: Duration, models: Vec<FakeModel>) -> Self {
        FakeProviderValidator::DelayedSuccess { delay, models }
    }

    pub fn delayed_error(delay: Duration, error: impl Into<String>) -> Self {
        FakeProviderValidator::DelayedError {
            delay,
            error: error.into(),
        }
    }

    pub fn manual_control() -> (
        Self,
        Arc<Mutex<Option<tokio::sync::oneshot::Sender<FakeValidationResult>>>>,
    ) {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let sender = Arc::new(Mutex::new(Some(tx)));
        let receiver = Arc::new(Mutex::new(Some(rx)));
        (
            FakeProviderValidator::ManualControl { sender, receiver },
            Arc::new(Mutex::new(None)),
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct CapturedLog {
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Default)]
pub struct CapturedLogs {
    logs: Arc<Mutex<Vec<CapturedLog>>>,
}

impl CapturedLogs {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn capture(&self, level: &str, message: &str, fields: HashMap<String, serde_json::Value>) {
        let log = CapturedLog {
            level: level.to_string(),
            message: message.to_string(),
            fields,
        };
        if let Ok(mut logs) = self.logs.lock() {
            logs.push(log);
        }
    }

    pub fn get_all(&self) -> Vec<CapturedLog> {
        self.logs.lock().map(|l| l.clone()).unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.clear();
        }
    }

    pub fn contains(&self, substring: &str) -> bool {
        self.get_all().iter().any(|l| l.message.contains(substring))
    }

    pub fn has_level(&self, level: &str) -> bool {
        self.get_all().iter().any(|l| l.level == level)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusOwner {
    #[default]
    MainInput,
    ProviderPicker,
    ApiKeyInput,
    ConnectProgress,
    ValidationError,
    ConnectModel,
    Modal,
    Unknown,
}

impl std::fmt::Display for FocusOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FocusOwner::MainInput => write!(f, "MainInput"),
            FocusOwner::ProviderPicker => write!(f, "ProviderPicker"),
            FocusOwner::ApiKeyInput => write!(f, "ApiKeyInput"),
            FocusOwner::ConnectProgress => write!(f, "ConnectProgress"),
            FocusOwner::ValidationError => write!(f, "ValidationError"),
            FocusOwner::ConnectModel => write!(f, "ConnectModel"),
            FocusOwner::Modal => write!(f, "Modal"),
            FocusOwner::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkflowState {
    #[default]
    Idle,
    ConnectProviderPicker,
    ConnectMethod,
    ConnectApiKeyInput,
    ConnectValidating,
    ConnectError,
    ConnectModelPicker,
    Chat,
    Unknown,
}

impl std::fmt::Display for WorkflowState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowState::Idle => write!(f, "Idle"),
            WorkflowState::ConnectProviderPicker => write!(f, "ConnectProviderPicker"),
            WorkflowState::ConnectMethod => write!(f, "ConnectMethod"),
            WorkflowState::ConnectApiKeyInput => write!(f, "ConnectApiKeyInput"),
            WorkflowState::ConnectValidating => write!(f, "ConnectValidating"),
            WorkflowState::ConnectError => write!(f, "ConnectError"),
            WorkflowState::ConnectModelPicker => write!(f, "ConnectModelPicker"),
            WorkflowState::Chat => write!(f, "Chat"),
            WorkflowState::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct TuiTestDiagnostics {
    pub scenario_name: Option<String>,
    pub terminal_size: (u16, u16),
    pub focus_owner: FocusOwner,
    pub workflow_state: WorkflowState,
    pub input_buffer: String,
    pub selected_provider: Option<String>,
    pub selected_model: Option<String>,
    pub pending_async_tasks: Vec<String>,
    pub recent_key_events: Vec<String>,
    pub captured_logs: Vec<CapturedLog>,
}

pub struct TuiTestHarness {
    backend: TestBackend,
    terminal: Terminal<TestBackend>,
    width: u16,
    height: u16,
    provider_validators: HashMap<String, FakeProviderValidator>,
    captured_logs: CapturedLogs,
    manual_sender: Option<Arc<Mutex<Option<tokio::sync::oneshot::Sender<FakeValidationResult>>>>>,
    recent_key_events: Vec<String>,
    diagnostics: TuiTestDiagnostics,
}

impl TuiTestHarness {
    pub fn builder() -> TuiTestHarnessBuilder {
        TuiTestHarnessBuilder::new()
    }

    pub fn new(width: u16, height: u16) -> Result<Self> {
        let backend = TestBackend::new(width, height);
        let terminal =
            Terminal::new(backend.clone()).context("Failed to create terminal for TUI testing")?;

        Ok(Self {
            backend,
            terminal,
            width,
            height,
            provider_validators: HashMap::new(),
            captured_logs: CapturedLogs::new(),
            manual_sender: None,
            recent_key_events: Vec::new(),
            diagnostics: TuiTestDiagnostics {
                terminal_size: (width, height),
                ..Default::default()
            },
        })
    }

    pub fn with_fake_provider_validator(
        mut self,
        provider_id: impl Into<String>,
        validator: FakeProviderValidator,
    ) -> Self {
        let provider_id = provider_id.into();
        if let FakeProviderValidator::ManualControl { sender, .. } = &validator {
            self.manual_sender = Some(sender.clone());
        }
        self.provider_validators.insert(provider_id, validator);
        self
    }

    pub fn with_fake_provider(mut self, provider_id: impl Into<String>) -> Self {
        self.provider_validators
            .insert(provider_id.into(), FakeProviderValidator::success());
        self
    }

    pub fn with_scenario_name(mut self, name: impl Into<String>) -> Self {
        self.diagnostics.scenario_name = Some(name.into());
        self
    }

    pub fn terminal(&self) -> &Terminal<TestBackend> {
        &self.terminal
    }

    pub fn terminal_mut(&mut self) -> &mut Terminal<TestBackend> {
        &mut self.terminal
    }

    pub fn backend(&self) -> &TestBackend {
        &self.backend
    }

    pub fn buffer(&self) -> Buffer {
        self.backend.buffer().clone()
    }

    pub fn captured_logs(&self) -> &CapturedLogs {
        &self.captured_logs
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.backend = TestBackend::new(width, height);
        self.terminal = Terminal::new(self.backend.clone()).expect("Failed to resize terminal");
    }

    #[allow(dead_code)]
    pub fn draw(&mut self, f: impl FnOnce(&mut ratatui::Frame)) {
        let _ = self.terminal.draw(f);
    }

    pub fn assert_screen_contains(&self, text: &str) -> Result<()> {
        let buffer = self.buffer();
        let content = Self::buffer_to_string(&buffer);
        if content.contains(text) {
            Ok(())
        } else {
            anyhow::bail!(
                "Screen does not contain '{}'. Actual content:\n{}",
                text,
                content
            );
        }
    }

    pub fn assert_screen_not_contains(&self, text: &str) -> Result<()> {
        let buffer = self.buffer();
        let content = Self::buffer_to_string(&buffer);
        if !content.contains(text) {
            Ok(())
        } else {
            anyhow::bail!("Screen contains '{}' but should not", text);
        }
    }

    pub fn assert_focus(&self, expected: FocusOwner) -> Result<()> {
        let actual = self.diagnostics.focus_owner;
        if actual == expected {
            Ok(())
        } else {
            anyhow::bail!("Expected focus {} but got {}", expected, actual);
        }
    }

    pub fn assert_workflow_state(&self, expected: WorkflowState) -> Result<()> {
        let actual = self.diagnostics.workflow_state;
        if actual == expected {
            Ok(())
        } else {
            anyhow::bail!("Expected workflow state {} but got {}", expected, actual);
        }
    }

    pub fn assert_input_equals(&self, expected: &str) -> Result<()> {
        let actual = &self.diagnostics.input_buffer;
        if actual == expected {
            Ok(())
        } else {
            anyhow::bail!("Expected input '{}' but got '{}'", expected, actual);
        }
    }

    pub fn assert_no_pending_modal(&self) -> Result<()> {
        match self.diagnostics.focus_owner {
            FocusOwner::Modal
            | FocusOwner::ApiKeyInput
            | FocusOwner::ValidationError
            | FocusOwner::ConnectModel => {
                anyhow::bail!(
                    "Expected no modal but focus owner is {}",
                    self.diagnostics.focus_owner
                );
            }
            _ => Ok(()),
        }
    }

    pub fn assert_input_usable(&self) -> Result<()> {
        let state = self.diagnostics.workflow_state;
        let focus = self.diagnostics.focus_owner;

        match (state, focus) {
            (WorkflowState::Idle, FocusOwner::MainInput) => Ok(()),
            (WorkflowState::Chat, FocusOwner::MainInput) => Ok(()),
            _ => anyhow::bail!(
                "Input should be usable but workflow_state={}, focus={}",
                state,
                focus
            ),
        }
    }

    pub fn record_key_event(&mut self, key_desc: &str) {
        self.recent_key_events.push(key_desc.to_string());
        self.diagnostics.recent_key_events = self.recent_key_events.clone();
    }

    pub fn get_diagnostics(&self) -> TuiTestDiagnostics {
        let mut diag = self.diagnostics.clone();
        diag.captured_logs = self.captured_logs.get_all();
        diag
    }

    pub fn print_diagnostics(&self) {
        let diag = self.get_diagnostics();
        eprintln!("{}", Self::format_diagnostics(&diag));
    }

    pub fn format_diagnostics(diag: &TuiTestDiagnostics) -> String {
        let mut s = String::new();
        s.push_str("=== TUI TEST FAILURE ===\n\n");

        if let Some(name) = &diag.scenario_name {
            s.push_str(&format!("Scenario:\n{}\n\n", name));
        }

        s.push_str(&format!(
            "Terminal size:\n{}x{}\n\n",
            diag.terminal_size.0, diag.terminal_size.1
        ));

        s.push_str(&format!("Focus owner:\n{}\n\n", diag.focus_owner));

        s.push_str(&format!("Active workflow:\n{}\n\n", diag.workflow_state));

        s.push_str(&format!("Input buffer:\n\"{}\"\n\n", diag.input_buffer));

        if let Some(ref provider) = diag.selected_provider {
            s.push_str(&format!("Selected provider:\n{}\n\n", provider));
        }

        if let Some(ref model) = diag.selected_model {
            s.push_str(&format!("Selected model:\n{}\n\n", model));
        }

        if !diag.pending_async_tasks.is_empty() {
            s.push_str("Pending async tasks:\n");
            for task in &diag.pending_async_tasks {
                s.push_str(&format!("  - {}\n", task));
            }
            s.push('\n');
        }

        if !diag.recent_key_events.is_empty() {
            s.push_str("Recent key events:\n");
            s.push_str(&format!("{:?}\n\n", diag.recent_key_events));
        }

        if !diag.captured_logs.is_empty() {
            s.push_str("Captured logs:\n");
            for log in &diag.captured_logs {
                s.push_str(&format!(
                    "  [{}] {} - {:?}\n",
                    log.level, log.message, log.fields
                ));
            }
        }

        s
    }

    fn buffer_to_string(buffer: &Buffer) -> String {
        let area = buffer.area;
        let mut lines = Vec::with_capacity(area.height as usize);

        for y in 0..area.height as usize {
            let mut line = String::new();
            for x in 0..area.width as usize {
                let idx = y * area.width as usize + x;
                if idx < buffer.content.len() {
                    line.push_str(buffer.content[idx].symbol());
                }
            }
            lines.push(line.trim_end().to_string());
        }

        lines.join("\n")
    }

    pub fn get_validator(&self, provider_id: &str) -> Option<&FakeProviderValidator> {
        self.provider_validators.get(provider_id)
    }

    #[allow(dead_code)]
    pub fn trigger_manual_validation_result(&self, result: FakeValidationResult) -> Result<()> {
        let sender = self
            .manual_sender
            .as_ref()
            .context("No manual control sender configured")?;

        let mut sender_guard = sender
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock sender"))?;

        let tx = sender_guard
            .take()
            .context("Manual control sender already used or not set")?;

        tx.send(result)
            .map_err(|_| anyhow::anyhow!("Failed to send manual validation result"))
    }
}

pub struct TuiTestHarnessBuilder {
    width: u16,
    height: u16,
    providers: Vec<String>,
    validators: HashMap<String, FakeProviderValidator>,
    scenario_name: Option<String>,
}

impl TuiTestHarnessBuilder {
    pub fn new() -> Self {
        Self {
            width: 120,
            height: 40,
            providers: Vec::new(),
            validators: HashMap::new(),
            scenario_name: None,
        }
    }

    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_fake_provider(mut self, provider_id: impl Into<String>) -> Self {
        self.providers.push(provider_id.into());
        self
    }

    pub fn with_fake_provider_validator(
        mut self,
        provider_id: impl Into<String>,
        validator: FakeProviderValidator,
    ) -> Self {
        let provider_id = provider_id.into();
        self.validators.insert(provider_id.clone(), validator);
        if !self.providers.contains(&provider_id) {
            self.providers.push(provider_id);
        }
        self
    }

    pub fn with_scenario_name(mut self, name: impl Into<String>) -> Self {
        self.scenario_name = Some(name.into());
        self
    }

    pub fn build(self) -> Result<TuiTestHarness> {
        let mut harness = TuiTestHarness::new(self.width, self.height)?;

        for provider_id in &self.providers {
            if !self.validators.contains_key(provider_id) {
                harness = harness.with_fake_provider(provider_id);
            }
        }

        for (provider_id, validator) in self.validators {
            harness = harness.with_fake_provider_validator(provider_id, validator);
        }

        if let Some(name) = self.scenario_name {
            harness = harness.with_scenario_name(name);
        }

        Ok(harness)
    }
}

impl Default for TuiTestHarnessBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct KeySimulator {
    harness: TuiTestHarness,
}

impl KeySimulator {
    pub fn new(harness: TuiTestHarness) -> Self {
        Self { harness }
    }

    pub fn press_enter(&mut self) {
        self.harness.record_key_event("Enter");
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        self.simulate_key(key);
    }

    pub fn press_esc(&mut self) {
        self.harness.record_key_event("Esc");
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        self.simulate_key(key);
    }

    pub fn press_up(&mut self) {
        self.harness.record_key_event("Up");
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        self.simulate_key(key);
    }

    pub fn press_down(&mut self) {
        self.harness.record_key_event("Down");
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        self.simulate_key(key);
    }

    pub fn type_text(&mut self, text: &str) {
        for c in text.chars() {
            self.harness.record_key_event(&format!("Char('{}')", c));
            let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
            self.simulate_key(key);
        }
    }

    pub fn press_tab(&mut self) {
        self.harness.record_key_event("Tab");
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        self.simulate_key(key);
    }

    pub fn press_backspace(&mut self) {
        self.harness.record_key_event("Backspace");
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        self.simulate_key(key);
    }

    pub fn type_command(&mut self, cmd: &str) {
        for c in cmd.chars() {
            let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
            self.simulate_key(key);
        }
        self.press_enter();
    }

    fn simulate_key(&mut self, _key: KeyEvent) {}

    pub fn into_harness(self) -> TuiTestHarness {
        self.harness
    }

    pub fn harness(&self) -> &TuiTestHarness {
        &self.harness
    }

    pub fn harness_mut(&mut self) -> &mut TuiTestHarness {
        &mut self.harness
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tui_test_harness_builder() {
        let harness = TuiTestHarness::builder()
            .with_size(80, 24)
            .with_fake_provider("minimax-cn")
            .with_scenario_name("test scenario")
            .build()
            .unwrap();

        assert_eq!(harness.width, 80);
        assert_eq!(harness.height, 24);
        assert!(harness.get_validator("minimax-cn").is_some());
    }

    #[test]
    fn test_fake_provider_validators() {
        let success = FakeProviderValidator::success();
        assert!(matches!(success, FakeProviderValidator::Success { .. }));

        let error = FakeProviderValidator::returns_404();
        assert!(matches!(error, FakeProviderValidator::Returns404));

        let hangs = FakeProviderValidator::hangs_forever();
        assert!(matches!(hangs, FakeProviderValidator::HangsForever));
    }

    #[test]
    fn test_captured_logs() {
        let logs = CapturedLogs::new();
        logs.capture("INFO", "Test message", HashMap::new());

        assert!(logs.contains("Test"));
        assert!(logs.has_level("INFO"));
        assert!(!logs.has_level("ERROR"));
    }

    #[test]
    fn test_diagnostics_format() {
        let diag = TuiTestDiagnostics {
            scenario_name: Some("test".to_string()),
            terminal_size: (80, 24),
            focus_owner: FocusOwner::MainInput,
            workflow_state: WorkflowState::Chat,
            input_buffer: "hello".to_string(),
            selected_provider: Some("minimax-cn".to_string()),
            selected_model: None,
            pending_async_tasks: vec!["task1".to_string()],
            recent_key_events: vec!["Enter".to_string()],
            captured_logs: vec![],
        };

        let formatted = TuiTestHarness::format_diagnostics(&diag);
        assert!(formatted.contains("test"));
        assert!(formatted.contains("80x24"));
        assert!(formatted.contains("MainInput"));
    }
}
