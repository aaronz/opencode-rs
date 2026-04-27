use std::process::{Command, Stdio};

pub struct Shell;

pub(crate) fn contains_shell_injection(cmd: &str) -> bool {
    let chars: Vec<char> = cmd.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        match c {
            ';' | '|' | '&' | '<' | '>' | '#' | '\n' => {
                return true;
            }
            '$' => {
                if i + 1 < chars.len() {
                    let next = chars[i + 1];
                    if next == '(' || next == '{' {
                        return true;
                    }
                }
            }
            '`' => {
                return true;
            }
            _ => {}
        }
    }

    if cmd.contains("&&") || cmd.contains("||") {
        return true;
    }

    let lower = cmd.to_lowercase();
    if lower.contains("rm -rf") || lower.contains("dd if=") || lower.contains(":(){:|:&}:") {
        return true;
    }

    false
}

impl Shell {
    pub fn execute(cmd: &str) -> Result<String, crate::OpenCodeError> {
        if contains_shell_injection(cmd) {
            return Err(crate::OpenCodeError::Tool(
                "Shell command contains forbidden characters or patterns".to_string(),
            ));
        }

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
