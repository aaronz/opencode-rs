use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::cell::RefCell;
use std::fmt::Debug;
use std::io::{BufRead, BufReader, Read, Write};
use std::time::Duration;

pub struct PtySimulator {
    pub master: Option<Box<dyn MasterPty>>,
    pub child: RefCell<Option<Box<dyn Child>>>,
    pub writer: Option<Box<dyn Write + Send>>,
    pub reader: Option<Box<dyn BufRead>>,
}

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
}

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
    fn default() -> Self {
        Self {
            master: None,
            child: RefCell::new(None),
            writer: None,
            reader: None,
        }
    }
}
