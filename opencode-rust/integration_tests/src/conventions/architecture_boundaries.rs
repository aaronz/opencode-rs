#[cfg(test)]
mod tests {
    fn workspace_root() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn core_lib_path() -> std::path::PathBuf {
        workspace_root().join("crates/core/src/lib.rs")
    }

    fn server_lib_path() -> std::path::PathBuf {
        workspace_root().join("crates/server/src/lib.rs")
    }

    fn plugin_lib_path() -> std::path::PathBuf {
        workspace_root().join("crates/plugin/src/lib.rs")
    }

    fn tui_lib_path() -> std::path::PathBuf {
        workspace_root().join("crates/tui/src/lib.rs")
    }

    #[test]
    fn core_does_not_import_server() {
        let core_lib = std::fs::read_to_string(core_lib_path()).unwrap();
        assert!(
            !core_lib.contains("opencode_server"),
            "crates/core/ must not import crates/server/ (authority boundary violation - per PRD 01)"
        );
        assert!(
            !core_lib.contains("actix"),
            "crates/core/ must not depend on HTTP server crates (authority boundary - per PRD 01)"
        );
        assert!(
            !core_lib.contains("axum"),
            "crates/core/ must not depend on HTTP server crates (authority boundary - per PRD 01)"
        );
    }

    #[test]
    fn server_does_not_define_core_domain_types() {
        let server_lib = std::fs::read_to_string(server_lib_path()).unwrap();
        let has_session_def = server_lib.contains("struct Session")
            && !server_lib.contains("use opencode_core::Session");
        let has_project_def = server_lib.contains("struct Project")
            && !server_lib.contains("use opencode_core::Project");
        assert!(
            !has_session_def,
            "crates/server/ must not define Session struct - use opencode_core::Session (per PRD 01)"
        );
        assert!(
            !has_project_def,
            "crates/server/ must not define Project struct - use opencode_core::Project (per PRD 01)"
        );
    }

    #[test]
    fn plugin_does_not_import_tui() {
        let plugin_lib = std::fs::read_to_string(plugin_lib_path()).unwrap();
        assert!(
            !plugin_lib.contains("crate::tui")
                && !plugin_lib.contains("opencode_tui"),
            "crates/plugin/ must not import crates/tui/ (plugin boundary - server/runtime vs TUI plugins per PRD 08)"
        );
    }

    #[test]
    fn tui_does_not_import_server() {
        let tui_lib = std::fs::read_to_string(tui_lib_path()).unwrap();
        assert!(
            !tui_lib.contains("crate::server") && !tui_lib.contains("opencode_server"),
            "crates/tui/ must not import crates/server/ (interface boundary per PRD 09)"
        );
    }

    #[test]
    fn agent_does_not_import_tui() {
        let agent_lib =
            std::fs::read_to_string(workspace_root().join("crates/agent/src/lib.rs")).unwrap();
        assert!(
            !agent_lib.contains("crate::tui") && !agent_lib.contains("opencode_tui"),
            "crates/agent/ must not import crates/tui/ (agent/runtime vs TUI boundary per PRD 02)"
        );
    }
}
