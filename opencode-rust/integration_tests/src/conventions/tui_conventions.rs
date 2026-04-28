#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn tui_lib_path() -> PathBuf {
        workspace_root().join("crates/tui/src/lib.rs")
    }

    fn tui_plugin_api_path() -> PathBuf {
        workspace_root().join("crates/tui/src/plugin_api.rs")
    }

    #[test]
    fn tui_plugin_api_is_versioned() {
        if !tui_plugin_api_path().exists() {
            let tui_lib = std::fs::read_to_string(tui_lib_path()).unwrap();
            if tui_lib.contains("plugin_api") {
                assert!(
                    tui_lib.contains("VERSION")
                        || tui_lib.contains("version")
                        || tui_lib.contains("ApiVersion"),
                    "TUI plugin API surface should be versioned per PRD 15"
                );
            }
            return;
        }
        let content = std::fs::read_to_string(tui_plugin_api_path()).unwrap();
        assert!(
            content.contains("VERSION")
                || content.contains("version")
                || content.contains("ApiVersion")
                || content.contains("v1")
                || content.contains("v2"),
            "TUI plugin external API surface must be versioned per PRD 15"
        );
    }

    #[test]
    fn tui_plugin_types_not_in_core() {
        let core_lib =
            std::fs::read_to_string(workspace_root().join("crates/core/src/lib.rs")).unwrap();

        let tui_terms = [
            "tui::",
            "opencode_tui",
            "crate::tui",
            "TuiPlugin",
            "Dialog",
            "Slot",
        ];
        let has_tui_leak = tui_terms.iter().any(|term| core_lib.contains(term));

        assert!(
            !has_tui_leak,
            "crates/core/ must not expose TUI plugin types - TUI plugin API is separate per PRD 15"
        );
    }

    #[test]
    fn tui_does_not_depend_on_server() {
        let tui_lib = std::fs::read_to_string(tui_lib_path()).unwrap();

        assert!(
            !tui_lib.contains("crate::server") && !tui_lib.contains("opencode_server::Server"),
            "crates/tui/ must not depend on crates/server/ - separate interface per PRD 09/13"
        );
    }

    #[test]
    fn tui_plugin_api_commands_routes_dialogs_slots() {
        let plugin_api_path = tui_plugin_api_path();
        if !plugin_api_path.exists() {
            return;
        }
        let content = std::fs::read_to_string(plugin_api_path).unwrap();
        let has_api_elements = content.contains("commands")
            || content.contains("routes")
            || content.contains("dialogs")
            || content.contains("slots")
            || content.contains("themes")
            || content.contains("state");

        assert!(
            has_api_elements,
            "TUI plugin API should expose commands, routes, dialogs, slots, themes, state per PRD 15"
        );
    }

    #[test]
    fn plugin_enabled_in_tui_json_not_opencode_json() {
        let opencode_config = workspace_root().join("crates/core/src/config.rs");
        if !opencode_config.exists() {
            return;
        }
        let content = std::fs::read_to_string(opencode_config).unwrap();

        assert!(
            !content.to_lowercase().contains("\"plugin_enabled\"") || content.contains("tui"),
            "plugin_enabled belongs in tui.json config, not opencode.json per PRD 06"
        );
    }
}
