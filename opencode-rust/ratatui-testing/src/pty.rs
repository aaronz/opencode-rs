use anyhow::{Context, Result};
use portable_pty::{Child, CommandBuilder, MasterPty, PtyPair};
use std::io::{BufRead, Write};

pub struct PtySimulator {
    master: Option<Box<dyn MasterPty>>,
    child: Option<Box<dyn Child>>,
    writer: Option<Box<dyn Write + Send>>,
    reader: Option<Box<dyn BufRead>>,
}

impl PtySimulator {
    pub fn new(command: &[&str]) -> Result<Self> {
        let pair = PtyPair::new().context("Failed to create PTY pair")?;

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
            child: Some(child),
            writer: Some(Box::new(writer)),
            reader: Some(Box::new(reader)),
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

    pub fn read_output(&mut self, timeout: std::time::Duration) -> Result<String> {
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
                master.resize(cols, rows)?;
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
        match &self.child {
            Some(child) => child.try_is_running().unwrap_or(false),
            None => false,
        }
    }
}

impl Default for PtySimulator {
    fn default() -> Self {
        Self {
            master: None,
            child: None,
            writer: None,
            reader: None,
        }
    }
}
