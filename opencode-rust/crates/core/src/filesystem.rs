#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub struct AppFileSystem;

impl AppFileSystem {
    pub fn is_dir(path: &str) -> bool {
        Path::new(path).is_dir()
    }

    pub fn is_file(path: &str) -> bool {
        Path::new(path).is_file()
    }

    pub fn exists(path: &str) -> bool {
        Path::new(path).exists()
    }

    pub fn read_json(path: &str) -> Result<serde_json::Value, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn write_json(
        path: &str,
        data: &serde_json::Value,
        mode: Option<u32>,
    ) -> Result<(), String> {
        let content = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
        std::fs::write(path, content).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        if let Some(m) = mode {
            std::fs::set_permissions(path, PermissionsExt::from_mode(m)).ok();
        }
        Ok(())
    }

    pub fn ensure_dir(path: &str) -> Result<(), String> {
        std::fs::create_dir_all(path).map_err(|e| e.to_string())
    }

    pub fn write_with_dirs(path: &str, content: &str, mode: Option<u32>) -> Result<(), String> {
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, content).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        if let Some(m) = mode {
            std::fs::set_permissions(path, PermissionsExt::from_mode(m)).ok();
        }
        Ok(())
    }

    pub fn find_up(target: &str, start: &str, stop: Option<&str>) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = PathBuf::from(start);
        let stop = stop.map(PathBuf::from);

        loop {
            let search = current.join(target);
            if search.exists() {
                result.push(search.to_string_lossy().to_string());
            }
            if stop.as_ref() == Some(&current) {
                break;
            }
            if !current.pop() {
                break;
            }
        }
        result
    }

    pub fn up(targets: &[String], start: &str, stop: Option<&str>) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = PathBuf::from(start);
        let stop = stop.map(PathBuf::from);

        loop {
            for target in targets {
                let search = current.join(target);
                if search.exists() {
                    result.push(search.to_string_lossy().to_string());
                }
            }
            if stop.as_ref() == Some(&current) {
                break;
            }
            if !current.pop() {
                break;
            }
        }
        result
    }

    pub fn normalize_path(p: &str) -> std::path::PathBuf {
        PathBuf::from(p)
    }

    fn relative<'a>(a: &'a str, b: &'a str) -> std::borrow::Cow<'a, str> {
        let a = Path::new(a);
        let b = Path::new(b);
        if let Ok(rel) = a.strip_prefix(b) {
            std::borrow::Cow::Borrowed(rel.to_str().unwrap_or(""))
        } else if let Ok(rel) = b.strip_prefix(a) {
            std::borrow::Cow::Borrowed(rel.to_str().unwrap_or(""))
        } else {
            std::borrow::Cow::Owned(a.to_string_lossy().to_string())
        }
    }

    pub fn overlaps(a: &str, b: &str) -> bool {
        let rel_a = Self::relative(a, b);
        let rel_b = Self::relative(b, a);
        rel_a.is_empty() || !rel_a.starts_with("..") || rel_b.is_empty() || !rel_b.starts_with("..")
    }

    pub fn contains(parent: &str, child: &str) -> bool {
        !Self::relative(parent, child).starts_with("..")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dir() {
        assert!(AppFileSystem::is_dir("/tmp"));
        assert!(!AppFileSystem::is_file(
            "/tmp/nonexistent_file_for_test_12345"
        ));
    }

    #[test]
    fn test_find_up() {
        let result = AppFileSystem::find_up(
            "Cargo.toml",
            "/Users/aaronzh/Documents/GitHub/opencode-rs/opencode-rust",
            None,
        );
        assert!(!result.is_empty());
    }
}
