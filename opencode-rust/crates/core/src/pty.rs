use crate::error::OpenCodeError;
use std::io;
use std::process::{Child, Command, Stdio};

pub struct PtySession {
    child: Child,
}

impl PtySession {
    pub fn new(command: &str) -> Result<Self, OpenCodeError> {
        let child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(OpenCodeError::Io)?;

        Ok(Self { child })
    }

    pub fn kill(&mut self) -> io::Result<()> {
        self.child.kill()
    }

    pub fn wait(&mut self) -> io::Result<std::process::ExitStatus> {
        self.child.wait()
    }
}
