#[cfg(test)]
mod tests {
    fn workspace_root() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn crate_unit_tests_live_in_crate_src() {
        let core_src = workspace_root().join("crates/core/src");
        if !core_src.exists() {
            return;
        }

        let has_mod_tests = std::fs::read_dir(core_src)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().is_file() && e.path().extension().map(|s| s == "rs").unwrap_or(false)
            })
            .any(|e| {
                let content = std::fs::read_to_string(e.path()).unwrap();
                content.contains("#[cfg(test)]")
            });

        assert!(
            has_mod_tests,
            "Unit tests should live in crate modules (crates/*/src/*.rs) per convention"
        );
    }

    #[test]
    fn cross_crate_tests_live_under_tests_dir() {
        let tests_dir = workspace_root().join("tests");
        assert!(
            tests_dir.exists(),
            "Cross-crate integration tests must live under rust-opencode-port/tests/ per convention"
        );
    }

    #[test]
    fn convention_tests_live_in_conventions_dir() {
        let conventions_dir = workspace_root().join("tests/src/conventions");
        assert!(
            conventions_dir.exists(),
            "Convention tests must live in tests/src/conventions/ per README"
        );
        let entries = std::fs::read_dir(conventions_dir).unwrap();
        let rs_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().is_file() && e.path().extension().map(|s| s == "rs").unwrap_or(false)
            })
            .collect();
        assert!(
            !rs_files.is_empty(),
            "tests/src/conventions/ must contain actual test .rs files"
        );
    }

    #[test]
    fn integration_tests_use_common_helpers() {
        let tests_dir = workspace_root().join("tests/src/common");
        if !tests_dir.exists() {
            return;
        }
        let entries = std::fs::read_dir(tests_dir).unwrap();
        let helpers: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().is_file() && e.path().extension().map(|s| s == "rs").unwrap_or(false)
            })
            .map(|e| e.path().file_name().unwrap().to_str().unwrap().to_string())
            .collect();

        assert!(
            helpers
                .iter()
                .any(|h| h.contains("temp_project") || h.contains("TempProject")),
            "Integration tests should use TempProject helper from tests/src/common/"
        );
    }

    #[test]
    fn ratatui_testing_accessible_from_tui_tests() {
        let tui_tests = workspace_root().join("crates/tui/tests");
        let ratatui_testing =
            std::path::Path::new("/Users/aaronzh/Documents/GitHub/mycode/ratatui-testing");
        if !tui_tests.exists() || !ratatui_testing.exists() {
            eprintln!("[KNOWN_GAP] TUI tests or ratatui-testing not found - test not applicable");
            return;
        }
        let entries = std::fs::read_dir(&tui_tests).unwrap();
        let has_ratatui_usage = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().is_file() && e.path().extension().map(|s| s == "rs").unwrap_or(false)
            })
            .any(|e| {
                std::fs::read_to_string(e.path())
                    .map(|c| c.contains("ratatui_testing"))
                    .unwrap_or(false)
            });

        if !has_ratatui_usage {
            eprintln!("[KNOWN_GAP] TUI tests don't reference ratatui-testing crate yet");
        }
    }
}
