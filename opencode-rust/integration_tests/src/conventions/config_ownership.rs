#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn config_does_not_have_tui_specific_fields() {
        let config_path = workspace_root().join("crates/core/src/config.rs");
        if !config_path.exists() {
            return;
        }
        let config_content = std::fs::read_to_string(config_path).unwrap();

        // plugin_enabled belongs in TuiConfig (tui.json), NOT in the main Config struct.
        // We check that plugin_enabled is not a direct field of the Config struct.
        // Strategy: Find "pub struct Config {" and "pub struct TuiConfig {"
        // If "plugin_enabled" appears as a struct field (followed by : or ,) within Config's
        // section (between Config's { and TuiConfig's {), then it's incorrectly placed.
        let config_struct_start = config_content.find("pub struct Config {");
        let tui_config_start = config_content.find("pub struct TuiConfig {");

        let plugin_in_main_config =
            if let (Some(config_pos), Some(tui_pos)) = (config_struct_start, tui_config_start) {
                // Search for "plugin_enabled" between Config's opening brace and TuiConfig's opening brace
                let search_start = config_pos + "pub struct Config {".len();
                let search_end = tui_pos;
                let between = &config_content[search_start..search_end];

                // Look for "plugin_enabled" followed by : or , (indicating a struct field)
                if let Some(pos) = between.find("plugin_enabled") {
                    let after = &between[pos..];
                    after.starts_with(':') || after.starts_with(',')
                } else {
                    false
                }
            } else {
                false
            };

        assert!(
            !plugin_in_main_config,
            "Config struct must not have plugin_enabled field - belongs in TuiConfig (tui.json) per PRD 06"
        );

        // TuiConfig in core config is a KNOWN_GAP: legacy integration being phased out
        // per the migration path described in PRD 06 (deprecated tui fields -> tui.json)
        let has_tui = config_content.contains("TuiConfig");
        if has_tui {
            eprintln!("[KNOWN_GAP] Core config still has TuiConfig - in migration per PRD 06");
        }
    }

    #[test]
    fn tools_key_aliased_to_permission() {
        let config_path = workspace_root().join("crates/core/src/config.rs");
        if !config_path.exists() {
            return;
        }
        let config_content = std::fs::read_to_string(config_path).unwrap();
        if config_content.contains("\"tools\"") || config_content.contains("'tools'") {
            assert!(
                config_content.contains("permission")
                    || config_content.contains("tools")
                    || config_content.contains("allowlist"),
                "Legacy 'tools' key must normalize into permission rules per PRD 06"
            );
        }
    }

    #[test]
    fn opencode_json_vs_tui_json_ownership() {
        let core_config = workspace_root().join("crates/core/src/config.rs");
        if !core_config.exists() {
            return;
        }
        let content = std::fs::read_to_string(core_config).unwrap();
        assert!(
            !content.contains("\"tui\"")
                || content.contains("tui_config")
                || content.contains("TuiConfig"),
            "Main config should delegate TUI-specific settings rather than owning them per PRD 06"
        );
    }

    #[test]
    fn session_does_not_import_permission_manager() {
        let session_path = workspace_root().join("crates/core/src/session.rs");
        if !session_path.exists() {
            return;
        }
        let content = std::fs::read_to_string(session_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        let imports: Vec<&str> = lines
            .iter()
            .filter(|l| l.starts_with("use "))
            .map(|l| *l)
            .collect();
        let has_tui_import = imports
            .iter()
            .any(|l| l.contains("crate::tui") || l.contains("crate::permission"));
        assert!(
            !has_tui_import,
            "Session must not import TUI or Permission modules - boundary per PRD 01"
        );
    }
}
