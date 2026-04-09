use glob::glob;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_MAX_FILE_SIZE: usize = 100 * 1024;
const DEFAULT_MAX_TOTAL_SIZE: usize = 20 * 1024;

pub struct InstructionsLoader {
    project_root: PathBuf,
    max_file_size: usize,
    max_total_size: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum InstructionsError {
    #[error("File not found: {0}")]
    NotFound(PathBuf),
    #[error("File too large: {0} ({1} bytes, max {2})")]
    FileTooLarge(PathBuf, usize, usize),
    #[error("Total instructions too large: {0} bytes, max {1}")]
    TotalTooLarge(usize, usize),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Glob pattern error: {0}")]
    GlobError(String),
}

impl InstructionsLoader {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            max_total_size: DEFAULT_MAX_TOTAL_SIZE,
        }
    }

    pub fn set_max_file_size(&mut self, size: usize) {
        self.max_file_size = size;
    }

    pub fn set_max_total_size(&mut self, size: usize) {
        self.max_total_size = size;
    }

    pub fn load_instructions(&self, paths: &[String]) -> Result<String, InstructionsError> {
        let mut combined = String::new();
        let mut total_size = 0usize;

        for path in paths {
            let resolved = self.resolve_path(path)?;
            for file_path in resolved {
                let content = self.load_single_file(&file_path)?;
                total_size += content.len();
                if total_size > self.max_total_size {
                    return Err(InstructionsError::TotalTooLarge(
                        total_size,
                        self.max_total_size,
                    ));
                }
                combined.push_str(&content);
            }
        }

        Ok(combined)
    }

    pub fn resolve_path(&self, path: &str) -> Result<Vec<PathBuf>, InstructionsError> {
        let has_glob = path.contains('*');

        if has_glob {
            let pattern = if Path::new(path).is_absolute() {
                path.to_string()
            } else {
                self.project_root.join(path).to_string_lossy().to_string()
            };

            let entries =
                glob(&pattern).map_err(|e| InstructionsError::GlobError(e.to_string()))?;
            let mut files = Vec::new();

            for entry in entries {
                match entry {
                    Ok(p) if p.is_file() => files.push(p),
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!(
                            "Skipping unreadable glob match for pattern {}: {}",
                            pattern,
                            e
                        )
                    }
                }
            }

            files.sort();

            if files.is_empty() {
                tracing::warn!("Instruction path did not match any files: {}", path);
            }

            return Ok(files);
        }

        let resolved = if path.starts_with('/') {
            PathBuf::from(path)
        } else {
            self.project_root.join(path)
        };

        if resolved.exists() {
            Ok(vec![resolved])
        } else {
            tracing::warn!(
                "Instruction file not found, skipping: {}",
                resolved.display()
            );
            Ok(Vec::new())
        }
    }

    pub fn load_single_file(&self, path: &PathBuf) -> Result<String, InstructionsError> {
        if !path.exists() {
            return Err(InstructionsError::NotFound(path.clone()));
        }

        let metadata = fs::metadata(path)?;
        let size = metadata.len() as usize;
        if size > self.max_file_size {
            return Err(InstructionsError::FileTooLarge(
                path.clone(),
                size,
                self.max_file_size,
            ));
        }

        Ok(fs::read_to_string(path)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn resolves_absolute_path() {
        let dir = tempdir().expect("create temp dir");
        let file = dir.path().join("abs.md");
        fs::write(&file, "absolute").expect("write file");

        let loader = InstructionsLoader::new(dir.path().to_path_buf());
        let resolved = loader
            .resolve_path(file.to_string_lossy().as_ref())
            .expect("resolve path");

        assert_eq!(resolved, vec![file]);
    }

    #[test]
    fn resolves_relative_path() {
        let dir = tempdir().expect("create temp dir");
        let file = dir.path().join("instructions.md");
        fs::write(&file, "relative").expect("write file");

        let loader = InstructionsLoader::new(dir.path().to_path_buf());
        let resolved = loader
            .resolve_path("./instructions.md")
            .expect("resolve path");

        assert_eq!(resolved, vec![file]);
    }

    #[test]
    fn expands_glob_pattern() {
        let dir = tempdir().expect("create temp dir");
        let instructions_dir = dir.path().join("instructions");
        fs::create_dir_all(&instructions_dir).expect("create instructions dir");
        let a = instructions_dir.join("a.md");
        let b = instructions_dir.join("b.md");
        fs::write(&a, "A").expect("write a");
        fs::write(&b, "B").expect("write b");

        let loader = InstructionsLoader::new(dir.path().to_path_buf());
        let resolved = loader
            .resolve_path("instructions/*.md")
            .expect("expand glob");

        assert_eq!(resolved, vec![a, b]);
    }

    #[test]
    fn enforces_file_size_limit() {
        let dir = tempdir().expect("create temp dir");
        let file = dir.path().join("big.md");
        fs::write(&file, "abcdef").expect("write file");

        let mut loader = InstructionsLoader::new(dir.path().to_path_buf());
        loader.set_max_file_size(5);

        let err = loader
            .load_single_file(&file)
            .expect_err("should fail on size");
        assert!(matches!(err, InstructionsError::FileTooLarge(_, 6, 5)));
    }

    #[test]
    fn enforces_total_size_limit() {
        let dir = tempdir().expect("create temp dir");
        let f1 = dir.path().join("one.md");
        let f2 = dir.path().join("two.md");
        fs::write(&f1, "12345").expect("write one");
        fs::write(&f2, "67890").expect("write two");

        let mut loader = InstructionsLoader::new(dir.path().to_path_buf());
        loader.set_max_total_size(9);

        let err = loader
            .load_instructions(&["./one.md".to_string(), "./two.md".to_string()])
            .expect_err("should fail total size");
        assert!(matches!(err, InstructionsError::TotalTooLarge(10, 9)));
    }

    #[test]
    fn missing_files_do_not_error() {
        let dir = tempdir().expect("create temp dir");
        let existing = dir.path().join("exists.md");
        fs::write(&existing, "present").expect("write existing");

        let loader = InstructionsLoader::new(dir.path().to_path_buf());
        let result = loader
            .load_instructions(&["./missing.md".to_string(), "./exists.md".to_string()])
            .expect("missing files should be skipped");

        assert_eq!(result, "present");
    }

    #[test]
    fn concatenates_multiple_files_in_order() {
        let dir = tempdir().expect("create temp dir");
        fs::write(dir.path().join("first.md"), "first-").expect("write first");
        fs::write(dir.path().join("second.md"), "second").expect("write second");

        let loader = InstructionsLoader::new(dir.path().to_path_buf());
        let result = loader
            .load_instructions(&["./first.md".to_string(), "./second.md".to_string()])
            .expect("load instructions");

        assert_eq!(result, "first-second");
    }
}
