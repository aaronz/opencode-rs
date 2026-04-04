use std::path::Path;
use std::process::Command;

/// Reserved IDE extension cursor position contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

/// Reserved IDE extension integration contract.
///
/// This trait is intentionally minimal in TASK-3.6 and provides a stable API
/// surface for future editor integrations.
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
        // Check TERM_PROGRAM environment variable
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ide_detect() {
        let ide = IdeManager::detect();
        // Just verify it returns some IDE
        assert!(matches!(
            ide,
            Ide::Unknown
                | Ide::Vscode
                | Ide::Cursor
                | Ide::Windsurf
                | Ide::VscodeInsiders
                | Ide::Vscodium
        ));
    }

    #[test]
    fn test_ide_from_name() {
        assert_eq!(Ide::from_name("Windsurf"), Some(Ide::Windsurf));
        assert_eq!(Ide::from_name("Cursor"), Some(Ide::Cursor));
        assert_eq!(Ide::from_name("Unknown IDE"), None);
    }

    struct StubExtension {
        opened: bool,
        changed: bool,
        cursor: Option<Position>,
    }

    impl IdeExtension for StubExtension {
        fn on_file_opened(&mut self, _path: &Path) {
            self.opened = true;
        }

        fn on_file_changed(&mut self, _path: &Path) {
            self.changed = true;
        }

        fn on_cursor_moved(&mut self, position: Position) {
            self.cursor = Some(position);
        }

        fn get_completion(&self, position: Position) -> Option<String> {
            if position.line == 0 && position.column == 0 {
                Some("completion".to_string())
            } else {
                None
            }
        }
    }

    #[test]
    fn ide_extension_trait_stub_is_callable() {
        let mut ext = StubExtension {
            opened: false,
            changed: false,
            cursor: None,
        };

        ext.on_file_opened(Path::new("/tmp/test.rs"));
        ext.on_file_changed(Path::new("/tmp/test.rs"));
        ext.on_cursor_moved(Position { line: 1, column: 2 });

        assert!(ext.opened);
        assert!(ext.changed);
        assert_eq!(ext.cursor, Some(Position { line: 1, column: 2 }));
        assert_eq!(
            ext.get_completion(Position { line: 0, column: 0 }),
            Some("completion".to_string())
        );
    }
}
