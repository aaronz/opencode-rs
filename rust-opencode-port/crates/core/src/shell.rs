use std::process::{Command, Stdio};

pub struct Shell;

impl Shell {
    pub fn execute(cmd: &str) -> Result<String, crate::OpenCodeError> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| crate::OpenCodeError::Tool(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(stdout.to_string())
        } else {
            Err(crate::OpenCodeError::Tool(stderr.to_string()))
        }
    }

    pub fn which(program: &str) -> Option<String> {
        Self::execute(&format!("which {}", program)).ok()
    }
}
