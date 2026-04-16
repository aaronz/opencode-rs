use std::fmt::Debug;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use ratatui::Terminal;
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::BufferDiff;
use crate::PtySimulator;
use crate::StateTester;

#[derive(Debug)]
pub struct TestDsl {
    width: u16,
    height: u16,
    terminal: Option<Terminal<TestBackend>>,
    pty: Option<crate::PtySimulator>,
    buffer_diff: Option<BufferDiff>,
    state_tester: Option<StateTester>,
    last_render: Option<Buffer>,
    predicates: Vec<WaitPredicate>,
}

impl TestDsl {
    pub fn new() -> Self {
        Self {
            width: 80,
            height: 30,
            terminal: None,
            pty: None,
            buffer_diff: None,
            state_tester: None,
            last_render: None,
            predicates: Vec::new(),
        }
    }

    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn init_terminal(mut self) -> Self {
        let backend = TestBackend::new(self.width, self.height);
        match Terminal::new(backend) {
            Ok(term) => {
                self.terminal = Some(term);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize terminal: {}", e);
            }
        }
        self
    }

    pub fn with_pty(mut self) -> Result<Self> {
        let pty = PtySimulator::new().context("Failed to create PTY simulator for TestDsl")?;
        self.pty = Some(pty);
        Ok(self)
    }

    pub fn with_pty_command(mut self, command: &[&str]) -> Result<Self> {
        let pty = PtySimulator::new_with_command(command)
            .context("Failed to create PTY simulator for TestDsl")?;
        self.pty = Some(pty);
        Ok(self)
    }

    pub fn with_buffer_diff(mut self) -> Self {
        self.buffer_diff = Some(BufferDiff::new());
        self
    }

    pub fn with_state_tester(mut self) -> Self {
        self.state_tester = Some(StateTester::new());
        self
    }

    pub fn render(mut self, widget: impl Widget + 'static) -> Self {
        if let Some(ref mut terminal) = self.terminal {
            let area = Rect::new(0, 0, self.width, self.height);
            let result = terminal.draw(|f| {
                f.render_widget(widget, area);
            });
            if result.is_ok() {
                self.last_render = Some(terminal.backend().buffer().clone());
            }
        }
        self
    }

    pub fn render_with_state<S, W, F>(self, state: &S, widget_fn: F) -> Self
    where
        S: 'static,
        W: Widget + 'static,
        F: FnOnce(&S, Rect) -> W + 'static,
    {
        let area = Rect::new(0, 0, self.width, self.height);
        let widget = widget_fn(state, area);
        self.render(widget)
    }

    pub fn capture_buffer(&self) -> Option<Buffer> {
        self.last_render.clone()
    }

    pub fn get_terminal(&self) -> Option<&Terminal<TestBackend>> {
        self.terminal.as_ref()
    }

    pub fn get_terminal_mut(&mut self) -> Option<&mut Terminal<TestBackend>> {
        self.terminal.as_mut()
    }

    pub fn get_pty(&self) -> Option<&PtySimulator> {
        self.pty.as_ref()
    }

    pub fn get_pty_mut(&mut self) -> Option<&mut PtySimulator> {
        self.pty.as_mut()
    }

    pub fn get_buffer_diff(&self) -> Option<&BufferDiff> {
        self.buffer_diff.as_ref()
    }

    pub fn get_buffer_diff_mut(&mut self) -> Option<&mut BufferDiff> {
        self.buffer_diff.as_mut()
    }

    pub fn get_state_tester(&self) -> Option<&StateTester> {
        self.state_tester.as_ref()
    }

    pub fn get_state_tester_mut(&mut self) -> Option<&mut StateTester> {
        self.state_tester.as_mut()
    }

    pub fn add_predicate(mut self, predicate: WaitPredicate) -> Self {
        self.predicates.push(predicate);
        self
    }

    pub fn assert_no_diffs(&self, expected: &Buffer) -> Result<()> {
        let actual = self
            .last_render
            .as_ref()
            .context("No buffer has been rendered yet")?;

        let diff = self
            .buffer_diff
            .as_ref()
            .context("BufferDiff not configured. Use with_buffer_diff()")?;

        let result = diff.diff(expected, actual);

        if result.total_diffs > 0 {
            anyhow::bail!("Buffer differences found:\n{}", result);
        }

        Ok(())
    }

    pub fn assert_buffer_matches(
        &self,
        expected: &Buffer,
        options: crate::diff::IgnoreOptions,
    ) -> Result<()> {
        let actual = self
            .last_render
            .as_ref()
            .context("No buffer has been rendered yet")?;

        let diff = BufferDiff::with_options(options);
        let result = diff.diff(expected, actual);

        if result.total_diffs > 0 {
            anyhow::bail!("Buffer differences found:\n{}", result);
        }

        Ok(())
    }

    pub fn assert_buffer_eq(&self, expected: &Buffer, actual: &Buffer) -> Result<()> {
        let diff = self
            .buffer_diff
            .as_ref()
            .context("BufferDiff not configured. Use with_buffer_diff()")?;

        let result = diff.diff(expected, actual);

        if !result.passed {
            anyhow::bail!("Buffer differences found:\n{}", result);
        }

        Ok(())
    }

    pub fn capture_state<S>(&mut self, state: &S, name: Option<&str>) -> Result<()>
    where
        S: serde::Serialize,
    {
        let tester = self
            .state_tester
            .as_mut()
            .context("StateTester not configured. Use with_state_tester()")?;
        tester.capture_state(state, name)?;
        Ok(())
    }

    pub fn assert_state<S>(&self, state: &S) -> Result<()>
    where
        S: serde::Serialize,
    {
        let tester = self
            .state_tester
            .as_ref()
            .context("StateTester not configured. Use with_state_tester()")?;
        tester.assert_state(state)
    }

    pub fn write_to_pty(&mut self, input: &str) -> Result<()> {
        let pty = self
            .pty
            .as_mut()
            .context("PTY not configured. Use with_pty()")?;
        pty.write_input(input)
    }

    pub fn send_keys(&mut self, keys: &str) -> Result<&mut Self> {
        let pty = self
            .pty
            .as_mut()
            .context("PTY not configured. Use with_pty()")?;

        let parsed_keys = parse_key_sequence(keys)?;
        for key_event in parsed_keys {
            pty.inject_key_event(key_event)?;
        }
        Ok(self)
    }

    pub fn read_from_pty(&mut self, timeout: Duration) -> Result<String> {
        let pty = self
            .pty
            .as_mut()
            .context("PTY not configured. Use with_pty()")?;
        pty.read_output(timeout)
    }

    pub fn resize_pty(&mut self, cols: u16, rows: u16) -> Result<()> {
        let pty = self
            .pty
            .as_mut()
            .context("PTY not configured. Use with_pty()")?;
        pty.resize(cols, rows)
    }

    pub fn is_pty_child_running(&self) -> bool {
        self.pty
            .as_ref()
            .map(|pty| pty.is_child_running())
            .unwrap_or(false)
    }

    pub fn wait_for<F>(self, timeout: Duration, predicate: F) -> Result<Self>
    where
        F: Fn() -> bool + Send + 'static,
    {
        let triggered = Arc::new(AtomicBool::new(false));
        let triggered_clone = triggered.clone();

        let (tx, mut rx) = mpsc::channel(1);

        std::thread::spawn(move || {
            while !triggered_clone.load(Ordering::SeqCst) {
                if predicate() {
                    triggered_clone.store(true, Ordering::SeqCst);
                    let _ = tx.blocking_send(());
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        });

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .context("Failed to create tokio runtime for wait_for")?;

        rt.block_on(async {
            tokio::select! {
                _ = rx.recv() => {
                    Ok(())
                }
                _ = sleep(timeout) => {
                    anyhow::bail!("Wait predicate timed out after {:?}", timeout);
                }
            }
        })?;

        Ok(self)
    }

    pub fn wait_for_async<F, Fut>(self, timeout: Duration, predicate: F) -> Result<Self>
    where
        F: Fn() -> Fut + Send + 'static,
        Fut: Future<Output = bool> + Send,
    {
        let triggered = Arc::new(AtomicBool::new(false));
        let triggered_clone = triggered.clone();

        let (tx, mut rx) = mpsc::channel(1);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .unwrap();

            rt.block_on(async {
                while !triggered_clone.load(Ordering::SeqCst) {
                    if predicate().await {
                        triggered_clone.store(true, Ordering::SeqCst);
                        let _ = tx.send(()).await;
                        break;
                    }
                    sleep(Duration::from_millis(50)).await;
                }
            });
        });

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .context("Failed to create tokio runtime for wait_for_async")?;

        rt.block_on(async {
            tokio::select! {
                _ = rx.recv() => {
                    Ok(())
                }
                _ = sleep(timeout) => {
                    anyhow::bail!("Wait predicate timed out after {:?}", timeout);
                }
            }
        })?;

        Ok(self)
    }

    pub fn wait_with_predicates(mut self, timeout: Duration) -> Result<Self> {
        let predicates = std::mem::take(&mut self.predicates);

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .context("Failed to create tokio runtime for wait_with_predicates")?;

        let start = std::time::Instant::now();
        let check_interval = Duration::from_millis(50);

        rt.block_on(async {
            loop {
                let elapsed = start.elapsed();
                if elapsed >= timeout {
                    let failed: Vec<String> = predicates
                        .iter()
                        .filter(|p| !p.check())
                        .map(|p| p.description())
                        .collect();

                    anyhow::bail!(
                        "Wait predicates timed out after {:?}. Failed predicates: {:?}",
                        timeout,
                        failed
                    );
                }

                let all_passed = predicates.iter().all(|p| p.check());
                if all_passed {
                    break;
                }

                sleep(check_interval).await;
            }
            Ok::<(), anyhow::Error>(())
        })?;

        Ok(self)
    }

    pub fn poll_until<F>(self, timeout: Duration, mut condition: F) -> Result<Self>
    where
        F: FnMut() -> bool,
    {
        let start = std::time::Instant::now();
        let check_interval = Duration::from_millis(50);

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .context("Failed to create tokio runtime for poll_until")?;

        rt.block_on(async {
            loop {
                let elapsed = start.elapsed();
                if elapsed >= timeout {
                    anyhow::bail!("Poll condition timed out after {:?}", timeout);
                }

                if condition() {
                    break;
                }

                sleep(check_interval).await;
            }
            Ok::<(), anyhow::Error>(())
        })?;

        Ok(self)
    }

    pub fn poll_until_async<F, Fut>(self, timeout: Duration, mut condition: F) -> Result<Self>
    where
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = bool> + Send + 'static,
    {
        let start = std::time::Instant::now();
        let check_interval = Duration::from_millis(50);

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .context("Failed to create tokio runtime for poll_until_async")?;

        rt.block_on(async {
            loop {
                let elapsed = start.elapsed();
                if elapsed >= timeout {
                    anyhow::bail!("Poll condition timed out after {:?}", timeout);
                }

                if condition().await {
                    break;
                }

                sleep(check_interval).await;
            }
            Ok::<(), anyhow::Error>(())
        })?;

        Ok(self)
    }

    pub fn then<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }

    pub fn then_result<F>(self, f: F) -> Result<Self>
    where
        F: FnOnce(Self) -> Result<Self>,
    {
        f(self)
    }

    pub fn buffer_content_at(&self, x: u16, y: u16) -> Option<String> {
        self.last_render.as_ref().and_then(|buf| {
            let width = buf.area.width as usize;

            if x < buf.area.width && y < buf.area.height {
                let idx = (y as usize) * width + (x as usize);
                buf.content.get(idx).map(|cell| cell.symbol().to_string())
            } else {
                None
            }
        })
    }

    pub fn buffer_line_at(&self, y: u16) -> Option<String> {
        self.last_render.as_ref().and_then(|buf| {
            if y < buf.area.height {
                let width = buf.area.width as usize;
                let start = (y as usize) * width;
                let end = start + width;

                Some(
                    buf.content[start..end.min(buf.content.len())]
                        .iter()
                        .map(|cell| cell.symbol().to_string())
                        .collect::<String>()
                        .trim_end()
                        .to_string(),
                )
            } else {
                None
            }
        })
    }

    pub fn buffer_lines(&self) -> Option<Vec<String>> {
        self.last_render.as_ref().map(|buf| {
            let width = buf.area.width as usize;

            buf.content
                .chunks(width)
                .map(|chunk| {
                    chunk
                        .iter()
                        .map(|cell| cell.symbol().to_string())
                        .collect::<String>()
                        .trim_end()
                        .to_string()
                })
                .collect()
        })
    }

    pub fn assert_pty_running(&self) -> Result<()> {
        if !self.is_pty_child_running() {
            anyhow::bail!("PTY child process is not running");
        }
        Ok(())
    }

    pub fn assert_pty_stopped(&self) -> Result<()> {
        if self.is_pty_child_running() {
            anyhow::bail!("PTY child process is still running");
        }
        Ok(())
    }

    pub fn snapshot_state(&mut self, name: &str) -> Result<()>
    where
        Self: Sized,
    {
        let buffer = self
            .last_render
            .as_ref()
            .context("No buffer has been rendered yet")?;

        let tester = self
            .state_tester
            .as_mut()
            .context("StateTester not configured. Use with_state_tester()")?;

        let lines = buffer
            .content
            .chunks(buffer.area.width as usize)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|cell| cell.symbol().to_string())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<_>>();

        tester.capture_state(&lines, Some(name))?;
        Ok(())
    }

    pub fn compare_to_snapshot(&self, name: &str) -> Result<()>
    where
        Self: Sized,
    {
        let buffer = self
            .last_render
            .as_ref()
            .context("No buffer has been rendered yet")?;

        let tester = self
            .state_tester
            .as_ref()
            .context("StateTester not configured. Use with_state_tester()")?;

        let lines = buffer
            .content
            .chunks(buffer.area.width as usize)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|cell| cell.symbol().to_string())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<_>>();

        tester.assert_state_named(&lines, name)
    }

    pub fn save_snapshot(&mut self, name: &str) -> Result<&mut Self> {
        let buffer = self.last_render.as_ref().context(
            "No buffer has been rendered yet. Render a widget first before saving snapshot.",
        )?;

        crate::save_snapshot(name, buffer)?;
        Ok(self)
    }

    pub fn load_snapshot(&self, name: &str) -> Result<Buffer> {
        crate::load_snapshot(name)
    }

    pub fn load_snapshot_and_assert_eq(&self, name: &str) -> Result<()>
    where
        Self: Sized,
    {
        let actual = self
            .last_render
            .as_ref()
            .context("No buffer has been rendered yet. Render a widget first before comparing.")?;

        let expected = crate::load_snapshot(name)?;

        let diff = self.buffer_diff.as_ref().context(
            "BufferDiff not configured. Use with_buffer_diff() to enable buffer comparison.",
        )?;

        let result = diff.diff(&expected, actual);

        if !result.passed {
            anyhow::bail!(
                "Loaded snapshot '{}' does not match current buffer:\n{}",
                name,
                result
            );
        }

        Ok(())
    }
}

impl Default for TestDsl {
    fn default() -> Self {
        Self::new()
    }
}

pub struct WaitPredicate {
    description: String,
    check_fn: Box<dyn Fn() -> bool + Send + 'static>,
}

impl WaitPredicate {
    pub fn new<F>(description: impl Into<String>, check_fn: F) -> Self
    where
        F: Fn() -> bool + Send + 'static,
    {
        Self {
            description: description.into(),
            check_fn: Box::new(check_fn),
        }
    }

    pub fn from_buffer_content<F>(description: impl Into<String>, check_fn: F) -> Self
    where
        F: Fn(Option<&Buffer>) -> bool + Send + 'static,
    {
        Self {
            description: description.into(),
            check_fn: Box::new(move || check_fn(None)),
        }
    }

    pub fn check(&self) -> bool {
        (self.check_fn)()
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }
}

impl Debug for WaitPredicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WaitPredicate")
            .field("description", &self.description)
            .finish()
    }
}

fn parse_key_sequence(keys: &str) -> Result<Vec<KeyEvent>> {
    let mut key_events = Vec::new();
    let mut remaining = keys.trim();

    let special_keys: &[(&str, KeyCode)] = &[
        ("enter", KeyCode::Enter),
        ("escape", KeyCode::Esc),
        ("esc", KeyCode::Esc),
        ("tab", KeyCode::Tab),
        ("backspace", KeyCode::Backspace),
        ("left", KeyCode::Left),
        ("right", KeyCode::Right),
        ("up", KeyCode::Up),
        ("down", KeyCode::Down),
        ("home", KeyCode::Home),
        ("end", KeyCode::End),
        ("pageup", KeyCode::PageUp),
        ("pagedown", KeyCode::PageDown),
        ("insert", KeyCode::Insert),
        ("delete", KeyCode::Delete),
        ("f1", KeyCode::F(1)),
        ("f2", KeyCode::F(2)),
        ("f3", KeyCode::F(3)),
        ("f4", KeyCode::F(4)),
        ("f5", KeyCode::F(5)),
        ("f6", KeyCode::F(6)),
        ("f7", KeyCode::F(7)),
        ("f8", KeyCode::F(8)),
        ("f9", KeyCode::F(9)),
        ("f10", KeyCode::F(10)),
        ("f11", KeyCode::F(11)),
        ("f12", KeyCode::F(12)),
    ];

    while !remaining.is_empty() {
        let mut matched = false;

        for (name, code) in special_keys {
            let prefix_lower = remaining.to_lowercase();
            if prefix_lower.starts_with(name) {
                let after_name = &remaining[name.len()..];
                if after_name.is_empty()
                    || after_name.starts_with(' ')
                    || after_name.starts_with('\n')
                {
                    remaining = after_name.trim_start_matches(' ').trim_start_matches('\n');
                    key_events.push(KeyEvent::new(*code, KeyModifiers::NONE));
                    matched = true;
                    break;
                }
            }

            let ctrl_name = format!("ctrl-{}", name);
            if prefix_lower.starts_with(&ctrl_name) {
                let after_ctrl = &remaining[ctrl_name.len()..];
                if after_ctrl.is_empty()
                    || after_ctrl.starts_with(' ')
                    || after_ctrl.starts_with('\n')
                {
                    remaining = after_ctrl.trim_start_matches(' ').trim_start_matches('\n');
                    key_events.push(KeyEvent::new(*code, KeyModifiers::CONTROL));
                    matched = true;
                    break;
                }
            }
        }

        if matched {
            continue;
        }

        let ch = remaining.chars().next().unwrap();
        remaining = &remaining[ch.len_utf8()..];

        if ch == '\\' {
            if !remaining.is_empty() {
                let next = remaining.chars().next().unwrap();
                remaining = &remaining[next.len_utf8()..];
                match next {
                    'n' => key_events.push(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
                    't' => key_events.push(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
                    'r' => key_events.push(KeyEvent::new(KeyCode::Char('\r'), KeyModifiers::NONE)),
                    'e' | 'E' => key_events.push(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
                    '\\' => key_events.push(KeyEvent::new(KeyCode::Char('\\'), KeyModifiers::NONE)),
                    _ => {
                        key_events.push(KeyEvent::new(KeyCode::Char(next), KeyModifiers::NONE));
                    }
                }
            }
        } else {
            key_events.push(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
        }
    }

    Ok(key_events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::text::Text;
    use ratatui::widgets::Paragraph;

    fn create_test_dsl() -> TestDsl {
        TestDsl::new().with_size(20, 5).init_terminal()
    }

    #[test]
    fn test_widget_rendering_to_buffer() {
        let dsl = create_test_dsl();
        let widget = Paragraph::new(Text::from("Hello, World!"));

        let result = dsl.render(widget);

        assert!(
            result.last_render.is_some(),
            "Widget should be rendered to buffer"
        );

        let buffer = result.last_render.unwrap();
        assert_eq!(buffer.area.width, 20);
        assert_eq!(buffer.area.height, 5);
    }

    #[test]
    fn test_widget_rendering_with_different_sizes() {
        let dsl = TestDsl::new().with_size(40, 10).init_terminal();
        let widget = Paragraph::new(Text::from("Test"));

        let result = dsl.render(widget);

        let buffer = result.last_render.unwrap();
        assert_eq!(buffer.area.width, 40);
        assert_eq!(buffer.area.height, 10);
    }

    #[test]
    fn test_pty_bufferdiff_statetester_composition() {
        let dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_pty_command(&["echo", "test"])
            .expect("PTY creation failed")
            .with_buffer_diff()
            .with_state_tester();

        assert!(dsl.pty.is_some(), "PTY should be composed");
        assert!(dsl.buffer_diff.is_some(), "BufferDiff should be composed");
        assert!(dsl.state_tester.is_some(), "StateTester should be composed");
    }

    #[test]
    fn test_fluent_api_method_chaining() {
        let result = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_buffer_diff()
            .with_state_tester()
            .render(Paragraph::new(Text::from("Test content")));

        assert!(result.last_render.is_some());
        assert!(result.buffer_diff.is_some());
        assert!(result.state_tester.is_some());
    }

    #[test]
    fn test_fluent_api_then_method() {
        let result = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .then(|dsl| dsl.render(Paragraph::new(Text::from("First"))))
            .then(|dsl| dsl.render(Paragraph::new(Text::from("Second"))));

        assert!(result.last_render.is_some());
    }

    #[test]
    fn test_fluent_api_then_result_method() {
        let result = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .then_result(|dsl| Ok(dsl.render(Paragraph::new(Text::from("Test")))))
            .expect("then_result should succeed");

        assert!(result.last_render.is_some());
    }

    #[test]
    fn test_wait_for_predicate_immediate_success() {
        let dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .wait_for(Duration::from_secs(1), || true)
            .expect("Wait should succeed immediately");

        assert!(dsl.last_render.is_none());
    }

    #[test]
    fn test_wait_for_predicate_timeout() {
        let result = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .wait_for(Duration::from_millis(100), || false);

        assert!(result.is_err(), "Wait should timeout");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("timed out"));
    }

    #[test]
    fn test_wait_with_predicates() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let triggered = Arc::new(AtomicBool::new(false));
        let triggered_clone = triggered.clone();

        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            triggered_clone.store(true, Ordering::SeqCst);
        });

        let predicate =
            WaitPredicate::new("test predicate", move || triggered.load(Ordering::SeqCst));

        let dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .add_predicate(predicate)
            .wait_with_predicates(Duration::from_secs(1))
            .expect("Wait should succeed");

        assert!(dsl.last_render.is_none());
    }

    #[test]
    fn test_poll_until_success() {
        use std::sync::atomic::{AtomicU32, Ordering};

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        std::thread::spawn(move || {
            for _ in 0..3 {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(20));
            }
        });

        let dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .poll_until(Duration::from_secs(1), move || {
                counter.load(Ordering::SeqCst) >= 3
            })
            .expect("Poll should succeed");

        assert!(dsl.last_render.is_none());
    }

    #[test]
    fn test_poll_until_timeout() {
        let result = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .poll_until(Duration::from_millis(50), || false);

        assert!(result.is_err(), "Poll should timeout");
    }

    #[test]
    fn test_buffer_content_access() {
        let dsl = TestDsl::new()
            .with_size(10, 3)
            .init_terminal()
            .render(Paragraph::new(Text::from("Hello")));

        let lines = dsl.buffer_lines().unwrap();
        assert!(!lines.is_empty());
        assert_eq!(lines[0].trim(), "Hello");
    }

    #[test]
    fn test_buffer_line_at() {
        let dsl = TestDsl::new()
            .with_size(10, 3)
            .init_terminal()
            .render(Paragraph::new(Text::from("Line1\nLine2\nLine3")));

        assert_eq!(dsl.buffer_line_at(0).unwrap().trim(), "Line1");
        assert_eq!(dsl.buffer_line_at(1).unwrap().trim(), "Line2");
        assert_eq!(dsl.buffer_line_at(2).unwrap().trim(), "Line3");
    }

    #[test]
    fn test_buffer_content_at() {
        let dsl = TestDsl::new()
            .with_size(10, 3)
            .init_terminal()
            .render(Paragraph::new(Text::from("Hello")));

        let content = dsl.buffer_content_at(0, 0);
        assert!(content.is_some());
        assert_eq!(content.unwrap(), "H");
    }

    #[test]
    fn test_assert_no_diffs_with_identical_buffers() {
        let dsl1 = TestDsl::new()
            .with_size(10, 3)
            .init_terminal()
            .with_buffer_diff()
            .render(Paragraph::new(Text::from("Test")));

        let dsl2 = TestDsl::new()
            .with_size(10, 3)
            .init_terminal()
            .with_buffer_diff()
            .render(Paragraph::new(Text::from("Test")));

        let buffer1 = dsl1.last_render.unwrap();

        let result = dsl2.assert_no_diffs(&buffer1);
        assert!(result.is_ok(), "Identical buffers should have no diffs");
    }

    #[test]
    fn test_assert_no_diffs_with_different_buffers() {
        let dsl1 = TestDsl::new()
            .with_size(10, 3)
            .init_terminal()
            .with_buffer_diff()
            .render(Paragraph::new(Text::from("Test1")));

        let dsl2 = TestDsl::new()
            .with_size(10, 3)
            .init_terminal()
            .with_buffer_diff()
            .render(Paragraph::new(Text::from("Test2")));

        let buffer1 = dsl1.last_render.unwrap();

        let result = dsl2.assert_no_diffs(&buffer1);
        assert!(result.is_err(), "Different buffers should have diffs");
    }

    #[test]
    fn test_state_capture_and_assert() {
        let mut dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_state_tester();

        #[derive(Debug, serde::Serialize, PartialEq)]
        struct TestState {
            value: String,
            count: u32,
        }

        let state = TestState {
            value: "test".to_string(),
            count: 42,
        };

        dsl.capture_state(&state, Some("initial"))
            .expect("Capture should succeed");
        let result = dsl
            .state_tester
            .as_ref()
            .unwrap()
            .assert_state_named(&state, "initial");
        assert!(
            result.is_ok(),
            "Same state should pass assertion: {:?}",
            result
        );
    }

    #[test]
    fn test_pty_operations() {
        let mut dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_pty_command(&["cat"])
            .expect("PTY creation failed");

        assert!(dsl.is_pty_child_running(), "PTY child should be running");

        dsl.write_to_pty("test input\n")
            .expect("Write should succeed");
    }

    #[test]
    fn test_pty_resize() {
        let mut dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_pty_command(&["cat"])
            .expect("PTY creation failed");

        let result = dsl.resize_pty(120, 40);
        assert!(result.is_ok(), "Resize should succeed");
    }

    #[test]
    fn test_snapshot_and_compare() {
        let dsl1 = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_state_tester()
            .render(Paragraph::new(Text::from("Snapshot test")));

        let lines1 = dsl1.buffer_lines().unwrap();

        let mut dsl2 = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_state_tester()
            .render(Paragraph::new(Text::from("Snapshot test")));

        dsl2.capture_state(&lines1, Some("snap1")).unwrap();

        let lines2 = dsl2.buffer_lines().unwrap();
        let result = dsl2
            .state_tester
            .as_ref()
            .unwrap()
            .assert_state_named(&lines2, "snap1");
        assert!(
            result.is_ok(),
            "Snapshot comparison should succeed: {:?}",
            result
        );
    }

    #[test]
    fn test_render_with_state() {
        #[derive(Debug, serde::Serialize)]
        struct RenderState {
            message: String,
        }

        let state = RenderState {
            message: "Dynamic content".to_string(),
        };

        let dsl = TestDsl::new().with_size(80, 24).init_terminal();

        let msg = state.message.clone();
        let widget = Paragraph::new(Text::from(msg));
        let dsl = dsl.render(widget);

        assert!(dsl.last_render.is_some());
        let lines = dsl.buffer_lines().unwrap();
        assert!(lines.iter().any(|l| l.contains("Dynamic content")));
    }

    #[test]
    fn test_wait_predicate_description() {
        let predicate = WaitPredicate::new("buffer has content", || true);
        assert_eq!(predicate.description(), "buffer has content");
    }

    #[test]
    fn test_add_multiple_predicates() {
        let dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .add_predicate(WaitPredicate::new("pred1", || true))
            .add_predicate(WaitPredicate::new("pred2", || true))
            .add_predicate(WaitPredicate::new("pred3", || true));

        assert_eq!(dsl.predicates.len(), 3);
    }

    #[test]
    fn test_get_terminal_mut() {
        let mut dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .render(Paragraph::new(Text::from("Test")));

        let terminal = dsl.get_terminal_mut();
        assert!(terminal.is_some());

        let terminal = dsl.get_terminal();
        assert!(terminal.is_some());
    }

    #[test]
    fn test_get_pty_mut() {
        let mut dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_pty_command(&["cat"])
            .expect("PTY creation failed");

        let pty = dsl.get_pty_mut();
        assert!(pty.is_some());

        let pty = dsl.get_pty();
        assert!(pty.is_some());
    }

    #[test]
    fn test_assert_buffer_eq_accepts_two_buffer_parameters() {
        let dsl = TestDsl::new()
            .with_size(10, 3)
            .init_terminal()
            .with_buffer_diff();

        let buf1 = ratatui::buffer::Buffer::empty(ratatui::layout::Rect::new(0, 0, 10, 3));
        let buf2 = ratatui::buffer::Buffer::empty(ratatui::layout::Rect::new(0, 0, 10, 3));

        let result = dsl.assert_buffer_eq(&buf1, &buf2);
        assert!(
            result.is_ok(),
            "assert_buffer_eq should accept two Buffer parameters"
        );
    }

    #[test]
    fn test_assert_buffer_eq_returns_ok_when_identical() {
        let dsl = TestDsl::new()
            .with_size(10, 3)
            .init_terminal()
            .with_buffer_diff()
            .render(Paragraph::new(Text::from("Identical content")));

        let buffer1 = dsl.capture_buffer().unwrap();

        let dsl2 = TestDsl::new()
            .with_size(10, 3)
            .init_terminal()
            .with_buffer_diff()
            .render(Paragraph::new(Text::from("Identical content")));

        let buffer2 = dsl2.capture_buffer().unwrap();

        let result = dsl.with_buffer_diff().assert_buffer_eq(&buffer1, &buffer2);
        assert!(
            result.is_ok(),
            "assert_buffer_eq should return Ok for identical buffers"
        );
    }

    #[test]
    fn test_assert_buffer_eq_returns_error_with_diff_details() {
        let dsl1 = TestDsl::new()
            .with_size(10, 3)
            .init_terminal()
            .with_buffer_diff()
            .render(Paragraph::new(Text::from("Content A")));

        let buffer1 = dsl1.capture_buffer().unwrap();

        let dsl2 = TestDsl::new()
            .with_size(10, 3)
            .init_terminal()
            .with_buffer_diff()
            .render(Paragraph::new(Text::from("Content B")));

        let buffer2 = dsl2.capture_buffer().unwrap();

        let result = dsl2.with_buffer_diff().assert_buffer_eq(&buffer1, &buffer2);
        assert!(
            result.is_err(),
            "assert_buffer_eq should return error for different buffers"
        );

        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("Buffer differences found") || err_msg.contains("difference"),
            "Error message should contain diff info: {}",
            err_msg
        );
    }

    #[test]
    fn test_assert_buffer_eq_fluent_chaining() {
        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;

        let buf1 = Buffer::empty(Rect::new(0, 0, 10, 3));
        let buf2 = Buffer::empty(Rect::new(0, 0, 10, 3));

        let result = TestDsl::new()
            .with_size(10, 3)
            .init_terminal()
            .with_buffer_diff()
            .render(Paragraph::new(Text::from("Test")))
            .then(|dsl| {
                dsl.assert_buffer_eq(&buf1, &buf2).unwrap();
                dsl
            });

        assert!(result.last_render.is_some());
    }

    #[test]
    fn test_send_keys_accepts_string_keys_input() {
        let mut dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_pty_command(&["cat"])
            .expect("PTY creation failed");

        let result = dsl.send_keys("hello");
        assert!(result.is_ok(), "send_keys should accept string input");
    }

    #[test]
    fn test_send_keys_returns_mut_self_for_fluent_chaining() {
        let mut dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_pty_command(&["cat"])
            .expect("PTY creation failed");

        let returned = dsl.send_keys("hello");
        assert!(returned.is_ok());

        let dsl_ref = returned.unwrap();
        assert!(dsl_ref.get_pty().is_some());
    }

    #[test]
    fn test_send_keys_parses_common_key_sequences() {
        let mut dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_pty_command(&["cat"])
            .expect("PTY creation failed");

        let result1 = dsl.send_keys("enter");
        assert!(result1.is_ok(), "send_keys should parse 'enter'");

        let result2 = dsl.send_keys("escape");
        assert!(result2.is_ok(), "send_keys should parse 'escape'");

        let result3 = dsl.send_keys("esc");
        assert!(result3.is_ok(), "send_keys should parse 'esc'");

        let result4 = dsl.send_keys("tab");
        assert!(result4.is_ok(), "send_keys should parse 'tab'");

        let result5 = dsl.send_keys("backspace");
        assert!(result5.is_ok(), "send_keys should parse 'backspace'");
    }

    #[test]
    fn test_send_keys_parses_ctrl_x_style_sequences() {
        let mut dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_pty_command(&["cat"])
            .expect("PTY creation failed");

        let result1 = dsl.send_keys("ctrl-c");
        assert!(result1.is_ok(), "send_keys should parse 'ctrl-c'");

        let result2 = dsl.send_keys("ctrl-x");
        assert!(result2.is_ok(), "send_keys should parse 'ctrl-x'");

        let result3 = dsl.send_keys("ctrl-a");
        assert!(result3.is_ok(), "send_keys should parse 'ctrl-a'");

        let result4 = dsl.send_keys("ctrl-z");
        assert!(result4.is_ok(), "send_keys should parse 'ctrl-z'");
    }

    #[test]
    fn test_send_keys_fluent_chaining() {
        let mut dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_pty_command(&["cat"])
            .expect("PTY creation failed");

        let final_dsl = dsl
            .send_keys("hello")
            .and_then(|dsl| {
                let _ = dsl.send_keys("enter")?;
                Ok(dsl)
            })
            .and_then(|dsl| {
                let _ = dsl.send_keys("ctrl-c")?;
                Ok(dsl)
            })
            .unwrap();

        assert!(final_dsl.get_pty().is_some());
    }

    #[test]
    fn test_save_snapshot_method_works() {
        let mut dsl = TestDsl::new()
            .with_size(20, 5)
            .init_terminal()
            .render(Paragraph::new(Text::from("Snapshot content")));

        let snapshot_name = "test_save_snapshot_dsl";
        let result = dsl.save_snapshot(snapshot_name);

        assert!(result.is_ok(), "save_snapshot should succeed after render");

        let loaded = crate::load_snapshot(snapshot_name);
        assert!(loaded.is_ok(), "Snapshot should be loadable after save");

        let loaded_buffer = loaded.unwrap();
        let original_buffer = dsl.capture_buffer().unwrap();
        assert_eq!(loaded_buffer.area.width, original_buffer.area.width);
        assert_eq!(loaded_buffer.area.height, original_buffer.area.height);

        std::fs::remove_file("snapshots/test_save_snapshot_dsl.json").ok();
    }

    #[test]
    fn test_save_snapshot_returns_mut_self() {
        let mut dsl = TestDsl::new()
            .with_size(20, 5)
            .init_terminal()
            .render(Paragraph::new(Text::from("Content")));

        let snapshot_name = "test_return_mut";
        let result = dsl.save_snapshot(snapshot_name);

        assert!(result.is_ok(), "save_snapshot should return Ok");
        let dsl_ref = result.unwrap();
        assert!(dsl_ref.last_render.is_some());

        std::fs::remove_file("snapshots/test_return_mut.json").ok();
    }

    #[test]
    fn test_save_snapshot_without_render_fails() {
        let mut dsl = TestDsl::new().with_size(20, 5).init_terminal();

        let result = dsl.save_snapshot("should_fail");

        assert!(
            result.is_err(),
            "save_snapshot should fail when no render has occurred"
        );
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("No buffer has been rendered yet"),
            "Error should mention missing render: {}",
            err_msg
        );
    }

    #[test]
    fn test_load_snapshot_method_works() {
        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;

        let area = Rect::new(0, 0, 20, 5);
        let mut buffer = Buffer::empty(area);
        for (i, c) in "Load test content".chars().enumerate() {
            if i < buffer.content.len() {
                buffer.content[i].set_symbol(c.to_string().as_str());
            }
        }

        let snapshot_name = "test_load_snapshot_dsl";
        crate::save_snapshot(snapshot_name, &buffer).unwrap();

        let dsl = TestDsl::new()
            .with_size(20, 5)
            .init_terminal()
            .render(Paragraph::new(Text::from("Different content")));

        let result = dsl.load_snapshot(snapshot_name);

        assert!(
            result.is_ok(),
            "load_snapshot should successfully load saved snapshot"
        );

        let loaded = result.unwrap();
        assert_eq!(loaded.area.width, buffer.area.width);
        assert_eq!(loaded.area.height, buffer.area.height);

        std::fs::remove_file("snapshots/test_load_snapshot_dsl.json").ok();
    }

    #[test]
    fn test_load_snapshot_nonexistent_fails() {
        let dsl = TestDsl::new().with_size(20, 5).init_terminal();

        let result = dsl.load_snapshot("nonexistent_snapshot_xyz_12345");

        assert!(
            result.is_err(),
            "load_snapshot should fail for nonexistent snapshot"
        );
    }

    #[test]
    fn test_load_snapshot_and_assert_eq_passes_with_matching() {
        let mut dsl = TestDsl::new()
            .with_size(20, 5)
            .init_terminal()
            .with_buffer_diff()
            .render(Paragraph::new(Text::from("Matching content")));

        let snapshot_name = "test_assert_eq_pass";
        dsl.save_snapshot(snapshot_name).unwrap();

        let result = dsl.load_snapshot_and_assert_eq(snapshot_name);

        assert!(
            result.is_ok(),
            "load_snapshot_and_assert_eq should pass when content matches"
        );

        std::fs::remove_file("snapshots/test_assert_eq_pass.json").ok();
    }

    #[test]
    fn test_load_snapshot_and_assert_eq_fails_with_mismatch() {
        let mut dsl = TestDsl::new()
            .with_size(20, 5)
            .init_terminal()
            .with_buffer_diff()
            .render(Paragraph::new(Text::from("Original content")));

        let snapshot_name = "test_assert_eq_fail";
        dsl.save_snapshot(snapshot_name).unwrap();

        let dsl2 = TestDsl::new()
            .with_size(20, 5)
            .init_terminal()
            .with_buffer_diff()
            .render(Paragraph::new(Text::from("Different content")));

        let result = dsl2.load_snapshot_and_assert_eq(snapshot_name);

        assert!(
            result.is_err(),
            "load_snapshot_and_assert_eq should fail when content differs"
        );

        std::fs::remove_file("snapshots/test_assert_eq_fail.json").ok();
    }

    #[test]
    fn test_snapshot_with_version_name() {
        let mut dsl = TestDsl::new()
            .with_size(20, 5)
            .init_terminal()
            .render(Paragraph::new(Text::from("Versioned content")));

        let snapshot_name = "test_snapshot@v1";
        let result = dsl.save_snapshot(snapshot_name);

        assert!(
            result.is_ok(),
            "save_snapshot should support versioned names like 'name@v1'"
        );

        let loaded = crate::load_snapshot(snapshot_name);
        assert!(
            loaded.is_ok(),
            "load_snapshot should work with versioned names"
        );

        std::fs::remove_file("snapshots/test_snapshot@v1.json").ok();
    }

    #[test]
    fn test_snapshot_fluent_api_chaining() {
        let mut dsl = TestDsl::new()
            .with_size(20, 5)
            .init_terminal()
            .with_buffer_diff()
            .render(Paragraph::new(Text::from("Chain test")));

        let snapshot_name = "test_fluent_chain";

        let result = dsl.save_snapshot(snapshot_name).and_then(|dsl| {
            dsl.load_snapshot_and_assert_eq(snapshot_name)?;
            Ok(dsl)
        });

        assert!(
            result.is_ok(),
            "Fluent chaining with save_snapshot and load_snapshot_and_assert_eq should work"
        );

        std::fs::remove_file("snapshots/test_fluent_chain.json").ok();
    }

    #[test]
    fn test_snapshot_multiple_versions_chaining() {
        let mut dsl = TestDsl::new()
            .with_size(20, 5)
            .init_terminal()
            .with_buffer_diff()
            .render(Paragraph::new(Text::from("Version 1")));

        let v1_name = "multi_version@v1";
        dsl.save_snapshot(v1_name).unwrap();

        dsl = dsl.render(Paragraph::new(Text::from("Version 2")));
        let v2_name = "multi_version@v2";
        dsl.save_snapshot(v2_name).unwrap();

        let v1_loaded = dsl.load_snapshot(v1_name);
        let v2_loaded = dsl.load_snapshot(v2_name);

        assert!(v1_loaded.is_ok(), "Should load v1 snapshot");
        assert!(v2_loaded.is_ok(), "Should load v2 snapshot");

        let v1_lines = v1_loaded
            .unwrap()
            .content
            .iter()
            .map(|c| c.symbol().to_string())
            .collect::<String>();
        let v2_lines = v2_loaded
            .unwrap()
            .content
            .iter()
            .map(|c| c.symbol().to_string())
            .collect::<String>();

        assert!(
            v1_lines.contains("Version 1"),
            "V1 should contain 'Version 1'"
        );
        assert!(
            v2_lines.contains("Version 2"),
            "V2 should contain 'Version 2'"
        );

        std::fs::remove_file("snapshots/multi_version@v1.json").ok();
        std::fs::remove_file("snapshots/multi_version@v2.json").ok();
    }

    #[test]
    fn test_with_pty_without_parameters_creates_pty_with_default_command() {
        let dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_pty()
            .expect("PTY creation with default command should succeed");

        assert!(
            dsl.pty.is_some(),
            "PTY should be created with default command"
        );
        assert!(
            dsl.is_pty_child_running(),
            "PTY child process should be running with default command"
        );
    }

    #[test]
    fn test_with_pty_command_with_explicit_command_still_works() {
        let dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_pty_command(&["echo", "test"])
            .expect("PTY creation with explicit command should succeed");

        assert!(
            dsl.pty.is_some(),
            "PTY should be created with explicit command"
        );
        assert!(
            dsl.is_pty_child_running(),
            "PTY child process should be running with explicit command"
        );
    }

    #[test]
    fn test_with_pty_fluent_chaining_without_parameters() {
        let dsl = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_pty()
            .expect("with_pty() should succeed")
            .with_buffer_diff()
            .with_state_tester();

        assert!(dsl.pty.is_some());
        assert!(dsl.buffer_diff.is_some());
        assert!(dsl.state_tester.is_some());
    }

    #[test]
    fn test_with_pty_and_with_pty_command_are_distinct_operations() {
        let dsl_with_default = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_pty()
            .expect("with_pty() should succeed");

        let dsl_with_explicit = TestDsl::new()
            .with_size(80, 24)
            .init_terminal()
            .with_pty_command(&["cat"])
            .expect("with_pty_command() should succeed");

        assert!(
            dsl_with_default.pty.is_some() && dsl_with_explicit.pty.is_some(),
            "Both methods should create PTY instances"
        );
    }
}
