use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::warn;

use opencode_config::{FormatterConfig, FormatterEntry};
use opencode_format::entry_matches_file;

const FORMAT_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn format_file_after_write(file_path: &str, project_root: &Path) {
    let config_path = project_root.join("opencode.json");
    let formatters = if config_path.exists() {
        load_formatters_from_config(&config_path)
    } else {
        let config_path = project_root.join("opencode.jsonc");
        if config_path.exists() {
            load_formatters_from_config(&config_path)
        } else {
            Vec::new()
        }
    };

    if formatters.is_empty() {
        return;
    }

    let matching: Vec<&FormatterEntry> = formatters
        .iter()
        .filter(|e| !e.disabled.unwrap_or(false))
        .filter(|e| entry_matches_file(e, file_path))
        .collect();

    for formatter in matching {
        let Some(command) = formatter.command.as_ref() else {
            continue;
        };
        if command.is_empty() {
            continue;
        }

        let executable = &command[0];
        let args: Vec<String> = command[1..]
            .iter()
            .map(|arg| arg.replace("$FILE", file_path))
            .collect();

        let mut cmd = Command::new(executable);
        cmd.args(&args).current_dir(project_root);
        if let Some(env) = formatter.environment.as_ref() {
            cmd.envs(env);
        }

        match cmd.spawn() {
            Ok(mut child) => {
                match timeout(FORMAT_TIMEOUT, child.wait()).await {
                    Ok(Ok(status)) if !status.success() => {
                        warn!(file_path, executable, %status, "formatter failed; write already completed");
                    }
                    Ok(Err(e)) => {
                        warn!(file_path, executable, error = %e, "formatter wait failed; write already completed");
                    }
                    Err(_) => {
                        let _ = child.kill().await;
                        warn!(file_path, executable, "formatter timed out; write already completed");
                    }
                    _ => {}
                }
            }
            Err(e) => {
                warn!(file_path, executable, error = %e, "failed to spawn formatter; write already completed");
            }
        }
    }
}

fn load_formatters_from_config(path: &Path) -> Vec<FormatterEntry> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let Some(formatter_val) = value.get("formatter") else {
        return Vec::new();
    };

    if let Some(obj) = formatter_val.as_object() {
        if obj.contains_key("agents") || obj.contains_key("disabled") {
            return Vec::new();
        }
    }

    match serde_json::from_value::<FormatterConfig>(formatter_val.clone()) {
        Ok(FormatterConfig::Formatters(map)) => map.into_values().collect(),
        _ => Vec::new(),
    }
}
