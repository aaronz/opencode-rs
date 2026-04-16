use std::cell::RefCell;
use std::fmt::Debug;
use std::io::{BufRead, BufReader, Read, Write};
use std::time::Duration;

#[cfg(unix)]
use anyhow::{Context, Result};
#[cfg(unix)]
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
#[cfg(unix)]
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};

#[cfg(windows)]
use anyhow::Result;

#[cfg(unix)]
pub struct PtySimulator {
    pub master: Option<Box<dyn MasterPty>>,
    pub child: RefCell<Option<Box<dyn Child>>>,
    pub writer: Option<Box<dyn Write + Send>>,
    pub reader: Option<Box<dyn BufRead>>,
}

#[cfg(windows)]
pub struct PtySimulator {
    pub master: Option<()>,
    pub child: RefCell<Option<()>>,
    pub writer: Option<Box<dyn Write + Send>>,
    pub reader: Option<Box<dyn BufRead>>,
}

#[cfg(unix)]
impl Debug for PtySimulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtySimulator")
            .field("master", &"Box<dyn MasterPty>")
            .field("child", &self.child)
            .field("has_writer", &self.writer.is_some())
            .field("has_reader", &self.reader.is_some())
            .finish()
    }
}

#[cfg(windows)]
impl Debug for PtySimulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtySimulator")
            .field("master", &"()")
            .field("child", &"()")
            .field("has_writer", &self.writer.is_some())
            .field("has_reader", &self.reader.is_some())
            .finish()
    }
}

#[cfg(unix)]
impl PtySimulator {
    pub fn new() -> Result<Self> {
        Self::new_with_command(&["bash", "-c", "echo ready"])
    }

    pub fn new_with_command(command: &[&str]) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to create PTY pair")?;

        let mut cmd = CommandBuilder::new(command[0]);
        for arg in &command[1..] {
            cmd.arg(arg);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .context("Failed to spawn child process")?;

        let writer = pair
            .master
            .take_writer()
            .context("Failed to get PTY writer")?;

        let reader = pair
            .master
            .try_clone_reader()
            .context("Failed to get PTY reader")?;

        Ok(Self {
            master: Some(pair.master),
            child: RefCell::new(Some(child)),
            writer: Some(Box::new(writer)),
            reader: Some(Box::new(BufReader::new(reader))),
        })
    }

    pub fn write_input(&mut self, input: &str) -> Result<()> {
        match &mut self.writer {
            Some(writer) => {
                writer.write_all(input.as_bytes())?;
                writer.flush()?;
                Ok(())
            }
            None => anyhow::bail!("PTY writer not available"),
        }
    }

    pub fn read_output(&mut self, timeout: Duration) -> Result<String> {
        let reader = match &mut self.reader {
            Some(reader) => reader,
            None => anyhow::bail!("PTY reader not available"),
        };

        let start = std::time::Instant::now();
        let mut buffer = Vec::new();
        let mut temp_buf = [0u8; 256];

        loop {
            if start.elapsed() >= timeout {
                break;
            }
            match reader.read(&mut temp_buf) {
                Ok(0) => break,
                Ok(n) => {
                    buffer.extend_from_slice(&temp_buf[..n]);
                    break;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => return Err(anyhow::anyhow!("Read error: {}", e)),
            }
        }

        Ok(String::from_utf8_lossy(&buffer).to_string())
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        match &mut self.master {
            Some(master) => {
                master.resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })?;
                Ok(())
            }
            None => anyhow::bail!("PTY master not available"),
        }
    }

    pub fn inject_key_event(&mut self, event: KeyEvent) -> Result<()> {
        let sequence = Self::encode_key_event(event)
            .ok_or_else(|| anyhow::anyhow!("Unsupported key event"))?;
        match &mut self.writer {
            Some(writer) => {
                writer.write_all(sequence.as_bytes())?;
                writer.flush()?;
                Ok(())
            }
            None => anyhow::bail!("PTY writer not available"),
        }
    }

    pub fn inject_mouse_event(&mut self, event: MouseEvent) -> Result<()> {
        let sequence = Self::encode_mouse_event(event);
        match &mut self.writer {
            Some(writer) => {
                writer.write_all(sequence.as_bytes())?;
                writer.flush()?;
                Ok(())
            }
            None => anyhow::bail!("PTY writer not available"),
        }
    }

    pub fn is_child_running(&self) -> bool {
        match self.child.borrow_mut().as_mut() {
            Some(child) => child.try_wait().ok().flatten().is_none(),
            None => false,
        }
    }

    fn encode_key_event(event: KeyEvent) -> Option<String> {
        let codepoint = match event.code {
            KeyCode::Char(c) => c as u32,
            KeyCode::Enter => 13,
            KeyCode::Tab => 9,
            KeyCode::Backspace => 127,
            KeyCode::Esc => 27,
            KeyCode::Left => 57399,
            KeyCode::Right => 57400,
            KeyCode::Up => 57401,
            KeyCode::Down => 57402,
            KeyCode::Home => 57398,
            KeyCode::End => 57403,
            KeyCode::PageUp => 57404,
            KeyCode::PageDown => 57405,
            KeyCode::Insert => 57406,
            KeyCode::Delete => 57407,
            KeyCode::F(n) => match n {
                1 => 57376,
                2 => 57377,
                3 => 57378,
                4 => 57379,
                5 => 57380,
                6 => 57381,
                7 => 57382,
                8 => 57383,
                9 => 57384,
                10 => 57385,
                11 => 57386,
                12 => 57387,
                _ => return None,
            },
            _ => return None,
        };

        let modifiers = encode_key_modifiers(event.modifiers);
        Some(format!("\x1B[{};{}u", codepoint, modifiers))
    }

    fn encode_mouse_event(event: MouseEvent) -> String {
        let (btn, release) = encode_mouse_button(event.kind);
        let modifiers = encode_mouse_modifiers(event.modifiers);
        let cb = btn | modifiers;
        let cx = event.column + 1;
        let cy = event.row + 1;

        if release {
            format!("\x1B[<{};{};{}m", cb, cx, cy)
        } else {
            format!("\x1B[<{};{};{}M", cb, cx, cy)
        }
    }
}

/// Windows PTY implementation stub.
///
/// This module provides a stub implementation of PtySimulator for Windows platforms.
/// All operations return errors indicating that PTY functionality is not supported.
///
/// ## Windows PTY Limitation
///
/// Windows PTY support is not implemented in this testing framework. The Windows API
/// for PTY operations (ConPTY) has different semantics and behavior compared to Unix
/// pseudo-terminals, making cross-platform testing challenging.
///
/// ## Error Handling
///
/// All methods in this implementation return `anyhow::Error` with messages prefixed
/// by `PTY not supported on Windows:` followed by a specific reason. These errors
/// help developers understand why their tests fail on Windows and provide guidance
/// on workarounds.
///
/// ## PRD Reference
///
/// See [FR-PTY-GAP-001] in the ratatui-testing specification for details on this
/// known limitation. The PRD documents this as a best-effort limitation - Unix PTY
/// testing is fully supported while Windows PTY testing returns descriptive errors.
///
/// ## Workarounds for Windows Testing
///
/// - Run PTY-dependent tests only on Unix CI runners
/// - Use conditional compilation with `#[cfg(unix)]` for PTY tests
/// - Mock PTY interactions when testing cross-platform code
///
/// ## Documentation
///
/// For more information on Windows PTY limitations, see:
/// - [Windows ConPTY Documentation](https://docs.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences)
/// - [ratatui-testing Issue Tracker](https://github.com/anomalyco/ratatui-testing/issues)
///
/// [FR-PTY-GAP-001]: ./ratatui-testing/SPEC.md#FR-PTY-GAP-001
#[cfg(windows)]
impl PtySimulator {
    /// Creates a new PTY simulator on Windows.
    ///
    /// # Returns
    ///
    /// Returns `Err` with message "PTY not supported on Windows: PtySimulator requires
    /// Unix-like environment for full functionality. See ratatui-testing documentation
    /// for Windows limitations and workarounds."
    pub fn new() -> Result<Self> {
        anyhow::bail!(
            "PTY not supported on Windows: PtySimulator requires Unix-like \
            environment for full functionality. See \
            https://github.com/anomalyco/ratatui-testing/blob/main/docs/windows-pty.md \
            for details on Windows limitations and workarounds."
        )
    }

    /// Creates a new PTY simulator with a custom command on Windows.
    ///
    /// # Returns
    ///
    /// Returns `Err` with message indicating Windows PTY is not supported.
    pub fn new_with_command(_command: &[&str]) -> Result<Self> {
        anyhow::bail!(
            "PTY not supported on Windows: PtySimulator::new_with_command requires \
            Unix-like environment. See \
            https://github.com/anomalyco/ratatui-testing/blob/main/docs/windows-pty.md \
            for details on Windows limitations and workarounds."
        )
    }

    /// Writes input to the PTY on Windows.
    ///
    /// # Returns
    ///
    /// Returns `Err` indicating Windows PTY is not supported.
    pub fn write_input(&mut self, _input: &str) -> Result<()> {
        anyhow::bail!(
            "PTY not supported on Windows: write_input requires Unix-like environment. \
            See https://github.com/anomalyco/ratatui-testing/blob/main/docs/windows-pty.md \
            for details on Windows limitations and workarounds."
        )
    }

    /// Reads output from the PTY on Windows.
    ///
    /// # Returns
    ///
    /// Returns `Err` indicating Windows PTY is not supported.
    pub fn read_output(&mut self, _timeout: Duration) -> Result<String> {
        anyhow::bail!(
            "PTY not supported on Windows: read_output requires Unix-like environment. \
            See https://github.com/anomalyco/ratatui-testing/blob/main/docs/windows-pty.md \
            for details on Windows limitations and workarounds."
        )
    }

    /// Resizes the PTY window on Windows.
    ///
    /// # Returns
    ///
    /// Returns `Err` indicating Windows PTY is not supported.
    pub fn resize(&mut self, _cols: u16, _rows: u16) -> Result<()> {
        anyhow::bail!(
            "PTY not supported on Windows: resize requires Unix-like environment. \
            See https://github.com/anomalyco/ratatui-testing/blob/main/docs/windows-pty.md \
            for details on Windows limitations and workarounds."
        )
    }

    /// Injects a key event into the PTY on Windows.
    ///
    /// # Returns
    ///
    /// Returns `Err` indicating Windows PTY is not supported.
    pub fn inject_key_event(&mut self, _event: crossterm::event::KeyEvent) -> Result<()> {
        anyhow::bail!(
            "PTY not supported on Windows: inject_key_event requires Unix-like environment. \
            See https://github.com/anomalyco/ratatui-testing/blob/main/docs/windows-pty.md \
            for details on Windows limitations and workarounds."
        )
    }

    /// Injects a mouse event into the PTY on Windows.
    ///
    /// # Returns
    ///
    /// Returns `Err` indicating Windows PTY is not supported.
    pub fn inject_mouse_event(&mut self, _event: crossterm::event::MouseEvent) -> Result<()> {
        anyhow::bail!(
            "PTY not supported on Windows: inject_mouse_event requires Unix-like environment. \
            See https://github.com/anomalyco/ratatui-testing/blob/main/docs/windows-pty.md \
            for details on Windows limitations and workarounds."
        )
    }

    /// Checks if the child process is running on Windows.
    ///
    /// # Returns
    ///
    /// Always returns `false` on Windows since no PTY is created.
    pub fn is_child_running(&self) -> bool {
        false
    }
}

#[cfg(unix)]
fn encode_key_modifiers(modifiers: KeyModifiers) -> u8 {
    let mut m = 0u8;
    if modifiers.contains(KeyModifiers::SHIFT) {
        m |= 1;
    }
    if modifiers.contains(KeyModifiers::CONTROL) {
        m |= 2;
    }
    if modifiers.contains(KeyModifiers::ALT) {
        m |= 4;
    }
    if modifiers.contains(KeyModifiers::SUPER) {
        m |= 8;
    }
    if modifiers.contains(KeyModifiers::HYPER) {
        m |= 16;
    }
    if modifiers.contains(KeyModifiers::META) {
        m |= 32;
    }
    if m == 0 {
        0
    } else {
        m
    }
}

#[cfg(unix)]
fn encode_mouse_button(kind: MouseEventKind) -> (u8, bool) {
    match kind {
        MouseEventKind::Down(MouseButton::Left) => (0, false),
        MouseEventKind::Down(MouseButton::Middle) => (1, false),
        MouseEventKind::Down(MouseButton::Right) => (2, false),
        MouseEventKind::Up(MouseButton::Left) => (0, true),
        MouseEventKind::Up(MouseButton::Middle) => (1, true),
        MouseEventKind::Up(MouseButton::Right) => (2, true),
        MouseEventKind::Drag(MouseButton::Left) => (32, false),
        MouseEventKind::Drag(MouseButton::Middle) => (33, false),
        MouseEventKind::Drag(MouseButton::Right) => (34, false),
        MouseEventKind::Moved => (35, false),
        MouseEventKind::ScrollDown => (64, false),
        MouseEventKind::ScrollUp => (65, false),
        MouseEventKind::ScrollLeft => (66, false),
        MouseEventKind::ScrollRight => (67, false),
    }
}

#[cfg(unix)]
fn encode_mouse_modifiers(modifiers: KeyModifiers) -> u8 {
    let mut m = 0u8;
    if modifiers.contains(KeyModifiers::SHIFT) {
        m |= 4;
    }
    if modifiers.contains(KeyModifiers::CONTROL) {
        m |= 16;
    }
    if modifiers.contains(KeyModifiers::ALT) {
        m |= 8;
    }
    m
}

impl Default for PtySimulator {
    #[cfg(unix)]
    fn default() -> Self {
        Self {
            master: None,
            child: RefCell::new(None),
            writer: None,
            reader: None,
        }
    }

    #[cfg(windows)]
    fn default() -> Self {
        Self {
            master: None,
            child: RefCell::new(None),
            writer: None,
            reader: None,
        }
    }
}
