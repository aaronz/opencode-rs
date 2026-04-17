use clap::{ArgAction, Args};
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Args, Debug)]
pub(crate) struct BashArgs {
    #[arg(long)]
    pub command: String,

    #[arg(short, long, action = ArgAction::Count)]
    pub json: u8,

    #[arg(long)]
    pub timeout: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_interactive_with_read() {
        assert!(looks_interactive("read -p 'Enter name: '"));
    }

    #[test]
    fn test_looks_interactive_with_read_space() {
        assert!(looks_interactive("read name"));
    }

    #[test]
    fn test_looks_interactive_false() {
        assert!(!looks_interactive("echo hello"));
        assert!(!looks_interactive("ls -la"));
        assert!(!looks_interactive("grep 'test' file.txt"));
    }

    #[test]
    fn test_looks_interactive_with_various_read_patterns() {
        assert!(looks_interactive("read -p 'Enter: '"));
        assert!(!looks_interactive("readfile"));
        assert!(!looks_interactive("pread"));
    }

    #[test]
    fn test_bash_args_basic() {
        let args = BashArgs {
            command: "echo hello".to_string(),
            json: 0,
            timeout: None,
        };
        assert_eq!(args.command, "echo hello");
        assert_eq!(args.json, 0);
        assert!(args.timeout.is_none());
    }

    #[test]
    fn test_bash_args_with_json() {
        let args = BashArgs {
            command: "ls".to_string(),
            json: 1,
            timeout: None,
        };
        assert_eq!(args.json, 1);
    }

    #[test]
    fn test_bash_args_with_timeout() {
        let args = BashArgs {
            command: "sleep 5".to_string(),
            json: 0,
            timeout: Some(60),
        };
        assert_eq!(args.timeout, Some(60));
    }

    #[test]
    fn test_bash_args_with_json_and_timeout() {
        let args = BashArgs {
            command: "pwd".to_string(),
            json: 2,
            timeout: Some(30),
        };
        assert_eq!(args.json, 2);
        assert_eq!(args.timeout, Some(30));
    }

    #[test]
    fn test_bash_args_command_only() {
        let args = BashArgs {
            command: "git status".to_string(),
            json: 0,
            timeout: None,
        };
        assert_eq!(args.command, "git status");
    }

    #[test]
    fn test_bash_args_all_fields() {
        let args = BashArgs {
            command: "npm test".to_string(),
            json: 3,
            timeout: Some(120),
        };
        assert_eq!(args.command, "npm test");
        assert_eq!(args.json, 3);
        assert_eq!(args.timeout, Some(120));
    }
}

fn looks_interactive(command: &str) -> bool {
    command.contains("read ") || command.contains("read -p")
}

pub(crate) fn run(args: BashArgs) {
    if looks_interactive(&args.command) {
        eprintln!("interactive command detected");
        std::process::exit(1);
    }

    let timeout_secs = args.timeout.unwrap_or(30);
    let mut child = match Command::new("sh")
        .arg("-c")
        .arg(&args.command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            eprintln!("Failed to start command: {}", error);
            std::process::exit(1);
        }
    };

    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() >= Duration::from_secs(timeout_secs) {
                    let _ = child.kill();
                    let _ = child.wait();
                    eprintln!("command timeout after {}s", timeout_secs);
                    std::process::exit(1);
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(error) => {
                eprintln!("Failed while waiting for command: {}", error);
                std::process::exit(1);
            }
        }
    }

    let output = child
        .wait_with_output()
        .expect("failed to wait for child process output");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(1);

    if args.json > 0 {
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({
                "stdout": stdout,
                "stderr": stderr,
                "exit_code": exit_code,
            }))
            .expect("failed to serialize JSON output")
        );
        if exit_code != 0 {
            std::process::exit(exit_code);
        }
        return;
    }

    if !stdout.is_empty() {
        print!("{}", stdout);
    }
    if !stderr.is_empty() {
        eprint!("{}", stderr);
    }
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}
