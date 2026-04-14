use anyhow::{Context, Result};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::cell::RefCell;
use std::fmt::Debug;
use std::io::{BufRead, BufReader, Write};

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
    pub fn new(command: &[&str]) -> Result<Self> {
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

    pub fn read_output(&mut self, _timeout: std::time::Duration) -> Result<String> {
        match &mut self.reader {
            Some(reader) => {
                let mut buffer = String::new();
                let _ = reader.read_line(&mut buffer);
                Ok(buffer)
            }
            None => anyhow::bail!("PTY reader not available"),
        }
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

    pub fn inject_key_event(&mut self, _event: crossterm::event::KeyEvent) -> Result<()> {
        Ok(())
    }

    pub fn inject_mouse_event(&mut self, _event: crossterm::event::MouseEvent) -> Result<()> {
        Ok(())
    }

    pub fn is_child_running(&self) -> bool {
        match self.child.borrow_mut().as_mut() {
            Some(child) => child.try_wait().ok().flatten().is_none(),
            None => false,
        }
    }
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
