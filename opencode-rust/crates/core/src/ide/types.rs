use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

pub trait IdeExtension {
    fn on_file_opened(&mut self, path: &Path);
    fn on_file_changed(&mut self, path: &Path);
    fn on_cursor_moved(&mut self, position: Position);
    fn get_completion(&self, position: Position) -> Option<String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ide {
    Windsurf,
    VscodeInsiders,
    Vscode,
    Cursor,
    Vscodium,
    Unknown,
}

impl Ide {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Windsurf" => Some(Ide::Windsurf),
            "Visual Studio Code - Insiders" => Some(Ide::VscodeInsiders),
            "Visual Studio Code" => Some(Ide::Vscode),
            "Cursor" => Some(Ide::Cursor),
            "VSCodium" => Some(Ide::Vscodium),
            _ => None,
        }
    }

    pub fn command(&self) -> &'static str {
        match self {
            Ide::Windsurf => "windsurf",
            Ide::VscodeInsiders => "code-insiders",
            Ide::Vscode => "code",
            Ide::Cursor => "cursor",
            Ide::Vscodium => "codium",
            Ide::Unknown => "",
        }
    }
}

pub struct IdeManager;

impl IdeManager {
    pub fn detect() -> Ide {
        if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
            if term_program == "vscode" {
                if let Ok(v) = std::env::var("GIT_ASKPASS") {
                    if v.contains("Visual Studio Code - Insiders") {
                        return Ide::VscodeInsiders;
                    }
                    if v.contains("Visual Studio Code") {
                        return Ide::Vscode;
                    }
                    if v.contains("Cursor") {
                        return Ide::Cursor;
                    }
                    if v.contains("Windsurf") {
                        return Ide::Windsurf;
                    }
                }
            }
        }
        Ide::Unknown
    }

    pub fn is_already_installed() -> bool {
        std::env::var("OPENCODE_CALLER")
            .map(|v| v == "vscode" || v == "vscode-insiders")
            .unwrap_or(false)
    }

    pub fn install(ide: Ide) -> Result<String, String> {
        let cmd = ide.command();
        if cmd.is_empty() {
            return Err("Unknown IDE".to_string());
        }

        let output = Command::new(cmd)
            .args(["--install-extension", "sst-dev.opencode"])
            .output()
            .map_err(|e| e.to_string())?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(stderr);
        }

        if stdout.contains("already installed") {
            return Err("Extension already installed".to_string());
        }

        Ok(stdout)
    }
}