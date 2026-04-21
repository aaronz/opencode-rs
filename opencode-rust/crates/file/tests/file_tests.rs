use opencode_file::FileError;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn test_file_error_variants() {
    let not_found = FileError::NotFound(PathBuf::from("/nonexistent"));
    assert!(not_found.to_string().contains("Path not found"));
    assert_eq!(not_found.to_string(), "Path not found: /nonexistent");

    let not_a_file = FileError::NotAFile(PathBuf::from("/some/dir"));
    assert!(not_a_file.to_string().contains("Not a file"));
    assert_eq!(not_a_file.to_string(), "Not a file: /some/dir");

    let not_a_dir = FileError::NotADirectory(PathBuf::from("/some/file.txt"));
    assert!(not_a_dir.to_string().contains("Not a directory"));
    assert_eq!(not_a_dir.to_string(), "Not a directory: /some/file.txt");

    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let io = FileError::Io {
        context: String::from("reading file"),
        source: Arc::new(io_error),
    };
    assert!(io.to_string().contains("IO error"));
    assert!(io.to_string().contains("reading file"));

    let watch_err = FileError::Watch(String::from("Failed to start watcher"));
    assert!(watch_err.to_string().contains("Watch error"));
    assert_eq!(watch_err.to_string(), "Watch error: Failed to start watcher");

    let watch_not_found = FileError::WatchNotFound(String::from("abc123"));
    assert!(watch_not_found.to_string().contains("Watch not found"));
    assert_eq!(watch_not_found.to_string(), "Watch not found: abc123");

    let path_too_long = FileError::PathTooLong(PathBuf::from("/very/long/path/that/exceeds/max"));
    assert!(path_too_long.to_string().contains("Path too long"));
    assert_eq!(path_too_long.to_string(), "Path too long: /very/long/path/that/exceeds/max");
}

#[test]
fn test_file_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission denied");
    let file_err: FileError = io_err.into();

    match file_err {
        FileError::Io { context, source } => {
            assert_eq!(context, "");
            assert_eq!(source.kind(), std::io::ErrorKind::PermissionDenied);
        }
        _ => panic!("Expected FileError::Io"),
    }
}

#[test]
fn test_file_error_debug() {
    let err = FileError::NotFound(PathBuf::from("/test"));
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("NotFound"));
    assert!(debug_str.contains("/test"));
}

#[test]
fn test_file_error_different_variants_are_different() {
    let err_not_found = FileError::NotFound(PathBuf::from("/test"));
    let err_not_a_file = FileError::NotAFile(PathBuf::from("/test"));
    let err_not_a_dir = FileError::NotADirectory(PathBuf::from("/test"));

    match err_not_found {
        FileError::NotFound(_) => {}
        _ => panic!("Expected NotFound"),
    }

    match err_not_a_file {
        FileError::NotAFile(_) => {}
        _ => panic!("Expected NotAFile"),
    }

    match err_not_a_dir {
        FileError::NotADirectory(_) => {}
        _ => panic!("Expected NotADirectory"),
    }
}