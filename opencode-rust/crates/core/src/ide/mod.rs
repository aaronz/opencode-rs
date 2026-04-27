mod types;

pub use types::{Ide, IdeExtension, IdeManager, Position};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_ide_detect() {
        let ide = IdeManager::detect();
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
        assert_eq!(Ide::from_name("VSCodium"), Some(Ide::Vscodium));
        assert_eq!(
            Ide::from_name("Visual Studio Code - Insiders"),
            Some(Ide::VscodeInsiders)
        );
        assert_eq!(Ide::from_name("Visual Studio Code"), Some(Ide::Vscode));
    }

    #[test]
    fn test_ide_command() {
        assert_eq!(Ide::Windsurf.command(), "windsurf");
        assert_eq!(Ide::VscodeInsiders.command(), "code-insiders");
        assert_eq!(Ide::Vscode.command(), "code");
        assert_eq!(Ide::Cursor.command(), "cursor");
        assert_eq!(Ide::Vscodium.command(), "codium");
        assert_eq!(Ide::Unknown.command(), "");
    }

    #[test]
    fn test_ide_install_unknown_ide() {
        let result = IdeManager::install(Ide::Unknown);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unknown IDE");
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
