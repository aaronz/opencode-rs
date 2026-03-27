use std::path::PathBuf;

pub fn assert_external_directory(target: &str) -> bool {
    if target.is_empty() {
        return false;
    }

    let target_path = PathBuf::from(target);

    if let Ok(current_dir) = std::env::current_dir() {
        if target_path.starts_with(&current_dir) {
            return false;
        }
    }

    true
}
