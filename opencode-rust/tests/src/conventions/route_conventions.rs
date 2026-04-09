use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn server_routes_dir() -> PathBuf {
        workspace_root().join("crates/server/src/routes")
    }

    #[test]
    fn routes_exist_under_server_crate() {
        assert!(
            server_routes_dir().exists(),
            "Route modules must live under crates/server/src/routes/ per PRD 07"
        );
    }

    #[test]
    fn route_modules_follow_resource_groups() {
        if !server_routes_dir().exists() {
            return;
        }
        let entries = std::fs::read_dir(server_routes_dir()).unwrap();
        let module_names: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().is_file() && e.path().extension().map(|s| s == "rs").unwrap_or(false)
            })
            .filter_map(|e| {
                e.path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .collect();

        let canonical_groups = [
            "project",
            "session",
            "message",
            "permission",
            "config",
            "provider",
            "mcp",
            "streaming",
        ];
        let has_resource_group = module_names
            .iter()
            .any(|n| canonical_groups.iter().any(|g| n == g));

        assert!(
            has_resource_group || module_names.is_empty(),
            "Route modules should follow canonical resource groups per PRD 07: project, session, message, permission, config, provider, mcp, streaming"
        );
    }

    #[test]
    fn server_lib_declares_routes_module() {
        let server_lib =
            std::fs::read_to_string(workspace_root().join("crates/server/src/lib.rs")).unwrap();

        assert!(
            server_lib.contains("mod routes") || server_lib.contains("pub mod routes"),
            "crates/server/src/lib.rs must declare routes module per PRD 07"
        );
    }

    #[test]
    fn api_error_shape_consistency() {
        let server_lib =
            std::fs::read_to_string(workspace_root().join("crates/server/src/lib.rs")).unwrap();
        if server_lib.contains("ApiError") || server_lib.contains("RouteError") {
            assert!(
                server_lib.contains("serde::Serialize") || server_lib.contains("serde_json"),
                "API error types should implement Serialize for consistent JSON shape per PRD 07"
            );
        }
    }
}
