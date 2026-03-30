use std::fs;
use std::path::Path;

const DEFAULT_MAX_SIZE: usize = 1024 * 1024;

#[derive(Debug, Clone)]
pub struct FileRefResult {
    pub path: String,
    pub content: String,
    pub is_binary: bool,
    pub truncated: bool,
    pub size: usize,
    pub error: Option<String>,
}

pub struct FileRefHandler {
    max_file_size: usize,
    max_content_size: usize,
}

impl FileRefHandler {
    pub fn new() -> Self {
        Self {
            max_file_size: DEFAULT_MAX_SIZE,
            max_content_size: 5000,
        }
    }

    pub fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    pub fn with_max_content_size(mut self, size: usize) -> Self {
        self.max_content_size = size;
        self
    }

    pub fn resolve(&self, path: &str) -> FileRefResult {
        let file_path = Path::new(path);

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

        let metadata = match fs::metadata(file_path) {
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

        let is_binary = self.detect_binary(file_path);

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

        let content = match fs::read_to_string(file_path) {
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

        output.push_str("\n");
        output.push_str(&result.content);

        output
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

        let result = handler().resolve(file_path.to_str().unwrap());
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

        let handler = FileRefHandler::new().with_max_content_size(100);
        let result = handler.resolve(file_path.to_str().unwrap());
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
}
