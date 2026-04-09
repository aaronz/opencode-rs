use std::env;
use std::path::PathBuf;
use std::process::Command;

pub struct EditorLauncher {
    preferred_editor: Option<String>,
}

impl EditorLauncher {
    pub fn new() -> Self {
        Self {
            preferred_editor: None,
        }
    }

    pub fn from_env() -> Self {
        let editor = env::var("EDITOR")
            .or_else(|_| env::var("VISUAL"))
            .ok()
            .filter(|e| !e.is_empty());
        Self {
            preferred_editor: editor,
        }
    }

    pub fn with_editor(mut self, editor: String) -> Self {
        self.preferred_editor = Some(editor);
        self
    }

    pub fn launch(&self, path: &PathBuf, wait: bool) -> Result<(), String> {
        let editor = self.resolve_editor()?;

        let mut cmd = if editor == "vim" || editor == "nvim" || editor == "nano" {
            Command::new(&editor)
        } else {
            Command::new(&editor)
        };

        if wait && !matches!(editor.as_str(), "vim" | "nvim" | "nano" | "emacs") {
            cmd.arg("--wait");
        }

        cmd.arg(path);

        cmd.spawn()
            .map_err(|e| format!("Failed to launch editor '{}': {}", editor, e))?;

        Ok(())
    }

    fn resolve_editor(&self) -> Result<String, String> {
        if let Some(ref editor) = self.preferred_editor {
            return Ok(editor.clone());
        }

        let candidates = [
            "code", "cursor", "windsurf", "nvim", "vim", "nano", "emacs", "nano",
        ];

        for editor in candidates {
            if Self::editor_available(editor) {
                return Ok(editor.to_string());
            }
        }

        Err("No available editor found. Set $EDITOR environment variable.".to_string())
    }

    fn editor_available(editor: &str) -> bool {
        Command::new(editor)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl Default for EditorLauncher {
    fn default() -> Self {
        Self::from_env()
    }
}
