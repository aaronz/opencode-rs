use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ignore::WalkBuilder;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_MAX_SIZE: usize = 1024 * 1024;
const GIT_DIR: &str = ".git";

#[derive(Debug, Clone)]
pub struct FileRefResult {
    pub path: String,
    pub content: String,
    pub is_binary: bool,
    pub truncated: bool,
    pub size: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileSearchResult {
    pub path: PathBuf,
    pub score: i64,
    pub display_name: String,
}

pub struct FileRefHandler {
    max_file_size: usize,
    max_content_size: usize,
    base_dir: PathBuf,
}

impl FileRefHandler {
    pub fn new() -> Self {
        Self {
            max_file_size: DEFAULT_MAX_SIZE,
            max_content_size: 5000,
            base_dir: std::env::current_dir().unwrap_or_default(),
        }
    }

    pub fn with_base_dir(mut self, dir: PathBuf) -> Self {
        self.base_dir = dir;
        self
    }

    pub fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    pub fn with_max_content_size(mut self, size: usize) -> Self {
        self.max_content_size = size;
        self
    }

    pub fn fuzzy_search_files(&self, query: &str, max_results: usize) -> Vec<FileSearchResult> {
        let matcher = SkimMatcherV2::default();
        let mut results: Vec<FileSearchResult> = Vec::new();

        let walker = WalkBuilder::new(&self.base_dir)
            .hidden(true)
            .git_ignore(true)
            .require_git(false)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if self.is_excluded(path) {
                continue;
            }

            let relative = path
                .strip_prefix(&self.base_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if let Some(score) = matcher.fuzzy_match(&relative, query) {
                results.push(FileSearchResult {
                    path: path.to_path_buf(),
                    score,
                    display_name: relative,
                });
            }
        }

        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.into_iter().take(max_results).collect()
    }

    fn is_excluded(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name == GIT_DIR {
                return true;
            }
            if name.starts_with('.') {
                return true;
            }
        }

        if let Ok(relative) = path.strip_prefix(&self.base_dir) {
            for component in relative.components() {
                if let Some(name) = component.as_os_str().to_str() {
                    if name == GIT_DIR {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn resolve(&self, path: &str) -> FileRefResult {
        let file_path = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            self.base_dir.join(path)
        };

        if !file_path.exists() {
            return FileRefResult {
                path: path.to_string(),
                content: String::new(),
                is_binary: false,
                truncated: false,
                size: 0,
                error: Some(format!("File not found: {}", path)),
            };
        }

        if let Ok(canonical) = std::fs::canonicalize(&file_path) {
            let base_canonical =
                std::fs::canonicalize(&self.base_dir).unwrap_or_else(|_| self.base_dir.clone());
            if !canonical.starts_with(&base_canonical) {
                return FileRefResult {
                    path: path.to_string(),
                    content: String::new(),
                    is_binary: false,
                    truncated: false,
                    size: 0,
                    error: Some("File is outside project directory".to_string()),
                };
            }
        }

        let metadata = match fs::metadata(&file_path) {
            Ok(m) => m,
            Err(e) => {
                return FileRefResult {
                    path: path.to_string(),
                    content: String::new(),
                    is_binary: false,
                    truncated: false,
                    size: 0,
                    error: Some(format!("Cannot read file metadata: {}", e)),
                };
            }
        };

        let size = metadata.len() as usize;

        if size > self.max_file_size {
            return FileRefResult {
                path: path.to_string(),
                content: String::new(),
                is_binary: false,
                truncated: false,
                size,
                error: Some(format!(
                    "File too large: {} bytes (max: {})",
                    size, self.max_file_size
                )),
            };
        }

        if metadata.is_dir() {
            return FileRefResult {
                path: path.to_string(),
                content: String::new(),
                is_binary: false,
                truncated: false,
                size,
                error: Some("Cannot read directory".to_string()),
            };
        }

        let is_binary = self.detect_binary(&file_path);

        if is_binary {
            return FileRefResult {
                path: path.to_string(),
                content: String::new(),
                is_binary: true,
                truncated: false,
                size,
                error: Some("Binary file detected".to_string()),
            };
        }

        let content = match fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                return FileRefResult {
                    path: path.to_string(),
                    content: String::new(),
                    is_binary: false,
                    truncated: false,
                    size,
                    error: Some(format!("Cannot read file: {}", e)),
                };
            }
        };

        let (truncated_content, truncated) = if content.len() > self.max_content_size {
            (
                format!(
                    "{}...[truncated {} bytes]",
                    &content[..self.max_content_size],
                    content.len() - self.max_content_size
                ),
                true,
            )
        } else {
            (content, false)
        };

        FileRefResult {
            path: path.to_string(),
            content: truncated_content,
            is_binary: false,
            truncated,
            size,
            error: None,
        }
    }

    fn detect_binary(&self, path: &Path) -> bool {
        use std::io::Read;

        let mut file = match fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return false,
        };

        let mut buffer = [0u8; 8192];
        let bytes_read = match file.read(&mut buffer) {
            Ok(n) => n,
            Err(_) => return false,
        };

        for &byte in &buffer[..bytes_read] {
            if byte == 0 {
                return true;
            }
        }

        false
    }

    pub fn format_for_context(&self, result: &FileRefResult) -> String {
        let mut output = format!("File: @{}\n", result.path);
        output.push_str(&format!("Size: {} bytes\n", result.size));

        if result.truncated {
            output.push_str(&format!(
                "(showing first {} bytes)\n",
                self.max_content_size
            ));
        }

        output.push('\n');
        output.push_str(&result.content);

        output
    }

    pub fn get_preview(&self, path: &str, max_lines: usize) -> Option<Vec<String>> {
        let result = self.resolve(path);
        if result.error.is_some() {
            return None;
        }
        if result.is_binary {
            return None;
        }

        let lines: Vec<String> = result
            .content
            .lines()
            .take(max_lines)
            .map(String::from)
            .collect();
        if lines.is_empty() {
            None
        } else {
            Some(lines)
        }
    }

    pub fn is_small_file(&self, path: &str, max_lines: usize) -> bool {
        let result = self.resolve(path);
        if result.error.is_some() {
            return false;
        }
        result.content.lines().count() <= max_lines && !result.is_binary
    }
}

impl Default for FileRefHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn handler() -> FileRefHandler {
        FileRefHandler::new()
    }

    #[test]
    fn test_resolve_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello World").unwrap();

        let handler = FileRefHandler::new().with_base_dir(temp_dir.path().to_path_buf());
        let result = handler.resolve("test.txt");
        assert!(result.error.is_none());
        assert_eq!(result.content, "Hello World");
    }

    #[test]
    fn test_resolve_nonexistent_file() {
        let result = handler().resolve("/nonexistent/path/file.txt");
        assert!(result.error.is_some());
    }

    #[test]
    fn test_file_size_limit() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.txt");
        fs::write(&file_path, "x".repeat(2000)).unwrap();

        let handler = FileRefHandler::new().with_max_file_size(100);
        let result = handler.resolve(file_path.to_str().unwrap());
        assert!(result.error.is_some());
    }

    #[test]
    fn test_content_truncation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "x".repeat(2000)).unwrap();

        let handler = FileRefHandler::new()
            .with_base_dir(temp_dir.path().to_path_buf())
            .with_max_content_size(100);
        let result = handler.resolve("test.txt");
        assert!(result.truncated);
    }

    #[test]
    fn test_format_for_context() {
        let result = FileRefResult {
            path: "test.rs".to_string(),
            content: "fn main() {}".to_string(),
            is_binary: false,
            truncated: false,
            size: 12,
            error: None,
        };

        let output = handler().format_for_context(&result);
        assert!(output.contains("File: @test.rs"));
        assert!(output.contains("fn main() {}"));
    }

    #[test]
    fn test_fuzzy_search_excludes_git() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        fs::write(temp_dir.path().join("main.rs"), "fn main() {}").unwrap();

        let handler = FileRefHandler::new().with_base_dir(temp_dir.path().to_path_buf());
        let results = handler.fuzzy_search_files("main", 10);

        assert!(results.iter().all(|r| !r.display_name.contains(".git")));
        assert!(results.iter().any(|r| r.display_name == "main.rs"));
    }

    #[test]
    fn test_fuzzy_search_scores() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("config.json"), "{}").unwrap();
        fs::write(temp_dir.path().join("config.yaml"), "").unwrap();
        fs::write(temp_dir.path().join("readme.md"), "").unwrap();

        let handler = FileRefHandler::new().with_base_dir(temp_dir.path().to_path_buf());
        let results = handler.fuzzy_search_files("config", 10);

        assert!(results.len() >= 2);
        assert!(results[0].display_name.starts_with("config"));
    }
}
