use clap::{ArgAction, Args};
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Args, Debug)]
pub struct BashArgs {
    #[arg(long)]
    pub command: String,

    #[arg(short, long, action = ArgAction::Count)]
    pub json: u8,

    #[arg(long)]
    pub timeout: Option<u64>,
}

fn looks_interactive(command: &str) -> bool {
    command.contains("read ") || command.contains("read -p")
}

pub fn run(args: BashArgs) {
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
