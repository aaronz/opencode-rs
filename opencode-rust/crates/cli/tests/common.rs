use std::env;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::Once;
use tempfile::TempDir;

static BUILD: Once = Once::new();

#[allow(dead_code)]
pub static EMPTY_VEC: Vec<serde_json::Value> = Vec::new();

pub struct TestHarness {
    #[allow(dead_code)]
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
            .env("RUST_LOG", "off")
            .env("OPENCODE_LOG_LEVEL", "off")
            .current_dir(self.temp_dir.path());
        cmd
    }

    pub fn run_cli(&self, args: &[&str]) -> Output {
        self.cmd()
            .args(args)
            .output()
            .expect("Failed to execute command")
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn setup_file(&self, path: &str, content: &str) -> PathBuf {
        let full_path = self.temp_dir.path().join(path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&full_path, content).unwrap();
        full_path
    }

    #[allow(dead_code)]
    pub fn read_file(&self, path: &str) -> String {
        let full_path = self.temp_dir.path().join(path);
        std::fs::read_to_string(full_path).expect("Failed to read file")
    }

    #[allow(dead_code)]
    pub fn create_session(&self, name: &str) -> String {
        let output = self.run_cli(&["session", "create", "--name", name]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .find(|l| l.contains("Session ID:"))
            .and_then(|l| l.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| format!("session-{}", name))
    }

    #[allow(dead_code)]
    pub fn send_message(&self, session_id: &str, content: &str) {
        self.run_cli(&[
            "session",
            "message",
            "--id",
            session_id,
            "--content",
            content,
        ]);
    }

    #[allow(dead_code)]
    pub fn get_session_messages(&self, session_id: &str) -> Vec<serde_json::Value> {
        let output = self.run_cli(&["session", "show", "--id", session_id, "--json"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();
        json.get("messages")
            .and_then(|m| m.as_array())
            .cloned()
            .unwrap_or_default()
    }

    #[allow(dead_code)]
    pub fn wait_for_async<F>(&self, timeout_ms: u64, check: F) -> bool
    where
        F: Fn() -> bool,
    {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        while start.elapsed() < timeout {
            if check() {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        false
    }

    #[allow(dead_code)]
    pub fn setup_project(&self, name: &str) -> PathBuf {
        let project_path = self.temp_dir.path().join("projects").join(name);
        std::fs::create_dir_all(&project_path).unwrap();

        std::fs::write(project_path.join("README.md"), format!("# {}", name)).unwrap();
        std::fs::write(project_path.join(".gitignore"), "target/\nnode_modules/\n").unwrap();

        project_path
    }

    #[allow(dead_code)]
    pub fn create_mock_provider(&self, name: &str, models: &[&str]) {
        let provider_dir = self.temp_dir.path().join("providers");
        std::fs::create_dir_all(&provider_dir).unwrap();

        let provider_config = serde_json::json!({
            "name": name,
            "models": models.iter().map(|m| serde_json::json!({
                "id": m,
                "name": m,
                "visible": true
            })).collect::<Vec<_>>()
        });

        std::fs::write(
            provider_dir.join(format!("{}.json", name)),
            serde_json::to_string_pretty(&provider_config).unwrap(),
        )
        .unwrap();
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
