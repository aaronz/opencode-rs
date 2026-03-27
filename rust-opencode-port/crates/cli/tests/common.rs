use std::env;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::Once;
use tempfile::TempDir;

static BUILD: Once = Once::new();

pub struct TestHarness {
    pub workspace_root: PathBuf,
    pub binary_path: PathBuf,
    pub temp_dir: TempDir,
}

impl TestHarness {
    pub fn setup() -> Self {
        // Build the binary once per test run
        BUILD.call_once(|| {
            let status = Command::new("cargo")
                .arg("build")
                .arg("--release")
                .status()
                .expect("Failed to build opencode-rs binary");
            assert!(status.success(), "Cargo build failed");
        });

        let mut workspace_root = env::current_dir().unwrap();
        // Adjust if we're running from inside a crate
        if workspace_root.ends_with("tests")
            || workspace_root
                .components()
                .any(|c| c.as_os_str() == "crates")
        {
            let mut p = workspace_root.clone();
            while p.pop() {
                if p.join("Cargo.toml").exists() && !p.ends_with("crates") {
                    workspace_root = p;
                    break;
                }
            }
        }

        let binary_path = workspace_root
            .join("target")
            .join("release")
            .join("opencode-rs");
        assert!(
            binary_path.exists(),
            "Binary not found at {:?}",
            binary_path
        );

        Self {
            workspace_root,
            binary_path,
            temp_dir: TempDir::new().unwrap(),
        }
    }

    pub fn cmd(&self) -> Command {
        let mut cmd = Command::new(&self.binary_path);
        cmd.env("OPENCODE_DATA_DIR", self.temp_dir.path().join("data"))
            .env("OPENCODE_CONFIG_DIR", self.temp_dir.path().join("config"))
            .env("NO_COLOR", "1")
            .current_dir(self.temp_dir.path());
        cmd
    }

    pub fn run_cli(&self, args: &[&str]) -> Output {
        self.cmd()
            .args(args)
            .output()
            .expect("Failed to execute command")
    }

    pub fn run_cli_json(&self, args: &[&str]) -> serde_json::Value {
        let mut all_args = Vec::new();
        if !args.is_empty() {
            all_args.push(args[0]);
            all_args.push("--json");
            for &arg in &args[1..] {
                all_args.push(arg);
            }
        } else {
            all_args.push("--json");
        }

        let output = self.run_cli(&all_args);
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!("CLI command failed: {}\nStderr: {}", output.status, stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let start_idx = stdout.find('{').or_else(|| stdout.find('[')).unwrap_or(0);
        let json_str = &stdout[start_idx..];

        serde_json::from_str(json_str)
            .unwrap_or_else(|e| panic!("Failed to parse JSON output: {}\nOutput: {}", e, stdout))
    }

    pub fn setup_file(&self, path: &str, content: &str) -> PathBuf {
        let full_path = self.temp_dir.path().join(path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&full_path, content).unwrap();
        full_path
    }

    pub fn read_file(&self, path: &str) -> String {
        let full_path = self.temp_dir.path().join(path);
        std::fs::read_to_string(full_path).expect("Failed to read file")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_setup() {
        let harness = TestHarness::setup();
        assert!(harness.binary_path.exists());
    }
}
