use opencode_file::{Debouncer, FileError, FileService, Normalizer};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

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
    assert_eq!(
        watch_err.to_string(),
        "Watch error: Failed to start watcher"
    );

    let watch_not_found = FileError::WatchNotFound(String::from("abc123"));
    assert!(watch_not_found.to_string().contains("Watch not found"));
    assert_eq!(watch_not_found.to_string(), "Watch not found: abc123");

    let path_too_long = FileError::PathTooLong(PathBuf::from("/very/long/path/that/exceeds/max"));
    assert!(path_too_long.to_string().contains("Path too long"));
    assert_eq!(
        path_too_long.to_string(),
        "Path too long: /very/long/path/that/exceeds/max"
    );
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

#[test]
fn test_file_service_creation() {
    let svc = FileService::new();
    assert!(svc
        .normalize_path(PathBuf::from("/a/b").as_path())
        .is_absolute());
}

#[test]
fn test_file_service_thread_safety() {
    let svc = Arc::new(FileService::new());
    let svc2 = svc.clone();

    let handle = std::thread::spawn(move || {
        let svc = svc2;
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.txt");
        std::fs::write(&path, "content").unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(svc.exists(path.as_path()))
    });

    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("test.txt");
    std::fs::write(&path, "content").unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(svc.exists(path.as_path()));

    let from_thread = handle.join().unwrap();
    assert!(result);
    assert!(from_thread);
}

#[test]
fn test_resolve_path_simple_relative() {
    let svc = FileService::new();
    let base = Path::new("/base");
    let relative = Path::new("./foo");
    let resolved = svc.resolve_path(base, relative);
    assert_eq!(resolved, Path::new("/base/foo"));
}

#[test]
fn test_resolve_path_parent_reference() {
    let svc = FileService::new();
    let base = Path::new("/parent/bar");
    let relative = Path::new("../foo");
    let resolved = svc.resolve_path(base, relative);
    assert_eq!(resolved, Path::new("/parent/foo"));
}

#[test]
fn test_resolve_path_already_absolute() {
    let svc = FileService::new();
    let base = Path::new("/foo/bar");
    let absolute = Path::new("/baz/qux");
    let resolved = svc.resolve_path(base, absolute);
    assert_eq!(resolved, Path::new("/baz/qux"));
}

#[test]
fn test_resolve_path_various_base_formats() {
    let svc = FileService::new();

    let base = Path::new("/a/b/c");
    let relative = Path::new("./d");
    let resolved = svc.resolve_path(base, relative);
    assert_eq!(resolved, Path::new("/a/b/c/d"));

    let base = Path::new("/a/b/c");
    let relative = Path::new("../d");
    let resolved = svc.resolve_path(base, relative);
    assert_eq!(resolved, Path::new("/a/b/d"));

    let base = Path::new("/a/b/c");
    let relative = Path::new("../../d");
    let resolved = svc.resolve_path(base, relative);
    assert_eq!(resolved, Path::new("/a/d"));

    let base = Path::new("/a/b/c");
    let relative = Path::new(".");
    let resolved = svc.resolve_path(base, relative);
    assert_eq!(resolved, Path::new("/a/b/c"));
}

#[test]
fn test_resolve_path_relative() {
    let svc = FileService::new();

    let base = Path::new("/base");
    let relative = Path::new("./foo");
    let resolved = svc.resolve_path(base, relative);
    assert_eq!(resolved, Path::new("/base/foo"));

    let base = Path::new("/parent/bar");
    let relative = Path::new("../foo");
    let resolved = svc.resolve_path(base, relative);
    assert_eq!(resolved, Path::new("/parent/foo"));

    let base = Path::new("/foo/bar");
    let absolute = Path::new("/baz/qux");
    let resolved = svc.resolve_path(base, absolute);
    assert_eq!(resolved, Path::new("/baz/qux"));
}

#[test]
fn test_normalize_collapse_dots() {
    let svc = FileService::new();

    let p = svc.normalize(Path::new("/a/b/./c"));
    assert_eq!(p, Path::new("/a/b/c"));

    let p = svc.normalize(Path::new("/a/b/../c/./d"));
    assert_eq!(p, Path::new("/a/c/d"));

    let p = svc.normalize(Path::new("/a/./b/./c"));
    assert_eq!(p, Path::new("/a/b/c"));

    let p = svc.normalize(Path::new("/a/b/c/../d"));
    assert_eq!(p, Path::new("/a/b/d"));

    let p = svc.normalize(Path::new("/a/b/c/../../d"));
    assert_eq!(p, Path::new("/a/d"));

    let p = svc.normalize(Path::new("./a/b/../c"));
    assert_eq!(p, Path::new("a/c"));

    let p = svc.normalize(Path::new("/a/../b/../c"));
    assert_eq!(p, Path::new("/c"));
}

#[test]
fn test_normalize_handles_parent_directory() {
    let svc = FileService::new();
    let p = svc.normalize(Path::new("/a/b/../c"));
    assert_eq!(p, Path::new("/a/c"));
}

#[test]
fn test_normalize_fixes_separators() {
    let svc = FileService::new();
    let p = svc.normalize(Path::new("a/b\\c/d"));
    if cfg!(windows) {
        assert!(p.as_os_str().to_string_lossy().contains('\\'));
    } else {
        assert!(p.as_os_str().to_string_lossy().contains('/'));
    }
}

#[test]
fn test_normalize_edge_case_excess_parent_components() {
    let svc = FileService::new();
    let p = svc.normalize(Path::new("/a/b/c/../../d"));
    assert_eq!(p, Path::new("/a/d"));
}

#[test]
fn test_normalizer_collapse_dots() {
    let normalizer = Normalizer::new();
    let p = normalizer.normalize(Path::new("/a/b/./c/./d"));
    assert_eq!(p, Path::new("/a/b/c/d"));
}

#[test]
fn test_normalizer_handles_parent_directory() {
    let normalizer = Normalizer::new();
    let p = normalizer.normalize(Path::new("/a/b/c/../d"));
    assert_eq!(p, Path::new("/a/b/d"));
}

#[test]
fn test_normalizer_excess_parent_components() {
    let normalizer = Normalizer::new();
    let p = normalizer.normalize(Path::new("/a/b/../../c"));
    assert_eq!(p, Path::new("/c"));
}

#[tokio::test]
async fn test_canonicalize_resolves_symlinks() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let real_file = tmp.path().join("real.txt");
    tokio::fs::write(&real_file, "content").await.unwrap();
    let symlink = tmp.path().join("link.txt");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&real_file, &symlink).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&real_file, &symlink).unwrap();

    let canonical = svc.canonicalize(&symlink).await.unwrap();
    let real_canonical = svc.canonicalize(&real_file).await.unwrap();
    assert_eq!(canonical, real_canonical);
}

#[tokio::test]
async fn test_canonicalize_makes_path_absolute() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("test.txt");
    tokio::fs::write(&file, "content").await.unwrap();

    let canonical = svc.canonicalize(&file).await.unwrap();
    assert!(canonical.is_absolute());
}

#[tokio::test]
async fn test_canonicalize_nonexistent_path_returns_error() {
    let svc = FileService::new();
    let result = svc
        .canonicalize(Path::new("/nonexistent/path/that/does/not/exist"))
        .await;
    assert!(result.is_err());
    match result.unwrap_err() {
        FileError::Io { .. } => {}
        _ => panic!("Expected FileError::Io"),
    }
}

#[tokio::test]
async fn test_canonicalize_broken_symlink() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let broken_symlink = tmp.path().join("broken.txt");
    #[cfg(unix)]
    std::os::unix::fs::symlink("/nonexistent/target", &broken_symlink).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_file("/nonexistent/target", &broken_symlink).unwrap();

    let result = svc.canonicalize(&broken_symlink).await;
    assert!(result.is_err());
}

#[test]
fn test_normalize_path_returns_absolute_paths() {
    let svc = FileService::new();

    let absolute_input = Path::new("/a/b/c");
    let result = svc.normalize_path(absolute_input);
    assert!(
        result.is_absolute(),
        "Absolute input should remain absolute"
    );
    assert_eq!(result, Path::new("/a/b/c"));
}

#[test]
fn test_normalize_path_handles_relative_input() {
    let original_cwd = std::env::current_dir().unwrap();
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();

    let relative_input = Path::new("./foo/bar");
    let result = svc.normalize_path(relative_input);
    assert!(
        result.is_absolute(),
        "Relative input should become absolute"
    );
    assert!(
        result.to_string_lossy().ends_with("foo/bar"),
        "Should resolve to foo/bar under cwd"
    );

    std::env::set_current_dir(original_cwd).unwrap();
}

#[test]
fn test_normalize_path_collapses_dots() {
    let original_cwd = std::env::current_dir().unwrap();
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();

    let input = Path::new("/a/b/../c/./d");
    let result = svc.normalize_path(input);
    assert_eq!(result, Path::new("/a/c/d"));

    std::env::set_current_dir(original_cwd).unwrap();
}

#[test]
fn test_normalize_path_platform_aware() {
    let svc = FileService::new();

    #[cfg(unix)]
    {
        let input = Path::new("/a/b/c");
        let result = svc.normalize_path(input);
        assert!(
            result.to_string_lossy().contains('/'),
            "Unix paths should use forward slash"
        );
    }

    #[cfg(windows)]
    {
        let input = Path::new("C:\\a\\b\\c");
        let result = svc.normalize_path(input);
        assert!(
            result.is_absolute(),
            "Windows absolute paths should be recognized"
        );
    }
}

#[test]
fn test_normalize_path_consistent_across_calls() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();

    let input = tmp.path().join("foo/./bar/../baz");
    let result1 = svc.normalize_path(&input);
    let result2 = svc.normalize_path(&input);
    assert_eq!(
        result1, result2,
        "normalize_path should return consistent results"
    );
}

#[test]
fn test_normalize_path_with_parent_directory() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();

    let input = Path::new("./foo/bar/../baz");
    let result = svc.normalize_path(input);
    assert!(result.is_absolute());
    assert!(
        result.to_string_lossy().ends_with("foo/baz"),
        "Should resolve .. correctly"
    );
}

#[test]
fn test_normalize_path_excess_parent_components() {
    let original_cwd = std::env::current_dir().unwrap();
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();

    let input = Path::new("/a/b/c/../../d");
    let result = svc.normalize_path(input);
    assert_eq!(result, Path::new("/a/d"));

    std::env::set_current_dir(original_cwd).unwrap();
}

#[test]
fn test_debouncer_instantiation_with_delay() {
    let delay = std::time::Duration::from_millis(100);
    let debouncer = Debouncer::new(delay);
    assert_eq!(debouncer.delay(), delay);
}

#[tokio::test]
async fn test_debouncer_queue_collapses_rapid_events() {
    let debounce = std::time::Duration::from_millis(50);
    let debouncer = Debouncer::new(debounce);
    let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count2 = count.clone();

    debouncer
        .queue(PathBuf::from("a.txt"), move || {
            count2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        })
        .await;

    let count3 = count.clone();
    debouncer
        .queue(PathBuf::from("a.txt"), move || {
            count3.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        })
        .await;

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let final_count = count.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(
        final_count, 1,
        "Expected 1 callback but got {} - rapid events should be collapsed",
        final_count
    );
}

#[tokio::test]
async fn test_debouncer_merges_rapid_events() {
    let debounce = Duration::from_millis(100);
    let debouncer = Debouncer::new(debounce);
    let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count2 = count.clone();

    debouncer
        .queue(PathBuf::from("a.txt"), move || {
            count2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        })
        .await;

    let count3 = count.clone();
    debouncer
        .queue(PathBuf::from("a.txt"), move || {
            count3.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        })
        .await;

    tokio::time::sleep(Duration::from_millis(300)).await;
    let final_count = count.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(
        final_count, 1,
        "Expected 1 callback but got {}",
        final_count
    );
}

#[tokio::test]
async fn test_watch_fires_callback_on_file_change() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    let file = dir.join("watched.txt");
    std::fs::write(&file, "v1").unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count2 = call_count.clone();

    let watch_id = svc
        .watch(dir, 100, move |_p| {
            call_count2.fetch_add(1, Ordering::SeqCst);
        })
        .await
        .unwrap();

    std::fs::write(&file, "v2").unwrap();
    std::thread::sleep(Duration::from_millis(1500));

    let count_after = call_count.load(Ordering::SeqCst);
    assert!(
        count_after >= 1,
        "Callback should have been called at least once, got {}",
        count_after
    );

    svc.unwatch(&watch_id).await.unwrap();
}

#[tokio::test]
async fn test_watch_returns_unique_watch_id() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("watched.txt");
    tokio::fs::write(&file, "content").await.unwrap();

    let (tx, _rx) = tokio::sync::mpsc::channel(1);
    let watch_id1 = svc
        .watch(tmp.path(), 50, move |_p| {
            let _ = tx.clone().blocking_send(());
        })
        .await
        .unwrap();

    let (tx2, _rx2) = tokio::sync::mpsc::channel(1);
    let watch_id2 = svc
        .watch(tmp.path(), 50, move |_p| {
            let _ = tx2.clone().blocking_send(());
        })
        .await
        .unwrap();

    assert_ne!(watch_id1, watch_id2, "Each watch should return a unique ID");
}

#[tokio::test]
async fn test_exists_returns_bool() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("exists.txt");
    tokio::fs::write(&file, "content").await.unwrap();

    assert!(
        svc.exists(&file).await,
        "exists() should return true for existing file"
    );
    assert!(
        !svc.exists(&tmp.path().join("nonexistent.txt")).await,
        "exists() should return false for non-existent path"
    );
}

#[tokio::test]
async fn test_exists_does_not_throw_error_on_missing_path() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let missing_path = tmp.path().join("this_file_does_not_exist.txt");

    let result = tokio::fs::metadata(&missing_path).await;
    assert!(
        result.is_err(),
        "metadata() should return error for missing path"
    );

    let exists_result = svc.exists(&missing_path).await;
    assert!(
        !exists_result,
        "exists() should return false, not throw error"
    );
}

#[tokio::test]
async fn test_create_dir_all_async_creates_nested_directories() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("nested").join("deeply").join("dir");

    svc.create_dir_all(&path).await.unwrap();
    assert!(
        path.exists(),
        "create_dir_all should create nested directories"
    );
    assert!(path.is_dir(), "created path should be a directory");
}

#[tokio::test]
async fn test_create_dir_all_async_succeeds_silently_if_exists() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("already").join("exists");

    tokio::fs::create_dir_all(&path).await.unwrap();
    let result = svc.create_dir_all(&path).await;
    assert!(
        result.is_ok(),
        "create_dir_all should succeed silently if directory exists"
    );
}

#[tokio::test]
async fn test_create_dir_all_async_permission_error() {
    let svc = FileService::new();
    let result = svc
        .create_dir_all(Path::new("/nonexistent/root/deep/path"))
        .await;
    assert!(
        result.is_err(),
        "create_dir_all should return error for permission denied"
    );
    match result.unwrap_err() {
        FileError::Io { .. } => {}
        _ => panic!("Expected FileError::Io for permission errors"),
    }
}

#[tokio::test]
async fn test_remove_file_deletes_existing_file() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("to_delete.txt");
    tokio::fs::write(&file, "content").await.unwrap();
    assert!(file.exists(), "File should exist before deletion");

    let result = svc.remove_file(&file).await;
    assert!(
        result.is_ok(),
        "remove_file should succeed for existing file"
    );
    assert!(!file.exists(), "File should not exist after deletion");
}

#[tokio::test]
async fn test_remove_file_not_found_error() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let nonexistent = tmp.path().join("nonexistent.txt");

    let result = svc.remove_file(&nonexistent).await;
    assert!(
        result.is_err(),
        "remove_file should return error for non-existent path"
    );
    match result.unwrap_err() {
        FileError::NotFound(p) => assert_eq!(p, nonexistent),
        _ => panic!("Expected FileError::NotFound"),
    }
}

#[tokio::test]
async fn test_remove_file_not_a_file_error() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("a_directory");
    tokio::fs::create_dir(&dir).await.unwrap();
    assert!(dir.is_dir(), "Path should be a directory");

    let result = svc.remove_file(&dir).await;
    assert!(
        result.is_err(),
        "remove_file should return error for directory"
    );
    match result.unwrap_err() {
        FileError::NotAFile(p) => assert_eq!(p, dir),
        _ => panic!("Expected FileError::NotAFile"),
    }
}

#[tokio::test]
async fn test_copy_file_copies_file_and_returns_byte_count() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("source.txt");
    let dst = tmp.path().join("dest.txt");
    tokio::fs::write(&src, "hello world").await.unwrap();

    let bytes = svc.copy_file(&src, &dst).await.unwrap();
    assert_eq!(bytes, 11);
    assert!(dst.exists());
    let content = tokio::fs::read_to_string(&dst).await.unwrap();
    assert_eq!(content, "hello world");
}

#[tokio::test]
async fn test_copy_file_creates_parent_dirs() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("src.txt");
    let dst = tmp.path().join("deep").join("nested").join("dst.txt");
    tokio::fs::write(&src, "content").await.unwrap();

    let n = svc.copy_file(&src, &dst).await.unwrap();
    assert_eq!(n, 7);
    assert!(dst.exists());
}

#[tokio::test]
async fn test_copy_file_not_found_error() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let missing = tmp.path().join("nonexistent.txt");
    let dst = tmp.path().join("dest.txt");

    let result = svc.copy_file(&missing, &dst).await;
    assert!(
        result.is_err(),
        "copy_file should return error for missing source"
    );
    match result.unwrap_err() {
        FileError::NotFound(p) => assert_eq!(p, missing),
        _ => panic!("Expected FileError::NotFound"),
    }
}

#[tokio::test]
async fn test_copy_dir_recursive() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("source");
    let dst = tmp.path().join("dest");

    tokio::fs::create_dir_all(&src).await.unwrap();
    tokio::fs::write(src.join("file1.txt"), "content1")
        .await
        .unwrap();
    tokio::fs::write(src.join("file2.txt"), "content2")
        .await
        .unwrap();
    tokio::fs::create_dir_all(src.join("subdir")).await.unwrap();
    tokio::fs::write(src.join("subdir").join("file3.txt"), "content3")
        .await
        .unwrap();

    let n = svc.copy_dir(&src, &dst).await.unwrap();
    assert!(n > 0, "Should copy some bytes");
    assert!(
        dst.join("file1.txt").exists(),
        "file1.txt should exist in dest"
    );
    assert!(
        dst.join("file2.txt").exists(),
        "file2.txt should exist in dest"
    );
    assert!(
        dst.join("subdir").join("file3.txt").exists(),
        "subdir/file3.txt should exist in dest"
    );

    let content1 = tokio::fs::read_to_string(dst.join("file1.txt"))
        .await
        .unwrap();
    assert_eq!(content1, "content1");
}

#[tokio::test]
async fn test_copy_dir_preserves_directory_structure() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("source");
    let dst = tmp.path().join("dest");

    tokio::fs::create_dir_all(src.join("a/b/c")).await.unwrap();
    tokio::fs::write(src.join("a/file.txt"), "a").await.unwrap();
    tokio::fs::write(src.join("a/b/file.txt"), "b")
        .await
        .unwrap();
    tokio::fs::write(src.join("a/b/c/file.txt"), "c")
        .await
        .unwrap();

    svc.copy_dir(&src, &dst).await.unwrap();

    assert!(dst.join("a").is_dir(), "a should be a directory");
    assert!(dst.join("a/b").is_dir(), "a/b should be a directory");
    assert!(dst.join("a/b/c").is_dir(), "a/b/c should be a directory");
    assert!(
        dst.join("a/file.txt").is_file(),
        "a/file.txt should be a file"
    );
    assert!(
        dst.join("a/b/file.txt").is_file(),
        "a/b/file.txt should be a file"
    );
    assert!(
        dst.join("a/b/c/file.txt").is_file(),
        "a/b/c/file.txt should be a file"
    );
}

#[tokio::test]
async fn test_copy_dir_not_a_directory_error() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("a_file.txt");
    tokio::fs::write(&file, "content").await.unwrap();

    let result = svc.copy_dir(&file, &tmp.path().join("dest")).await;
    assert!(
        result.is_err(),
        "copy_dir should return error for file source"
    );
    match result.unwrap_err() {
        FileError::NotADirectory(p) => assert_eq!(p, file),
        _ => panic!("Expected FileError::NotADirectory"),
    }
}

#[tokio::test]
async fn test_unwatch_removes_watcher_from_registry() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("watched.txt");
    std::fs::write(&file, "v1").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(300));

    let (tx, _rx) = tokio::sync::mpsc::channel(1);
    let watch_id = svc
        .watch(tmp.path(), 100, move |_p| {
            let _ = tx.clone().blocking_send(());
        })
        .await
        .unwrap();

    let result = svc.unwatch(&watch_id).await;
    assert!(result.is_ok(), "unwatch should succeed for valid watch_id");

    let result_invalid = svc.unwatch(&watch_id).await;
    assert!(
        result_invalid.is_err(),
        "second unwatch should fail since watcher was removed"
    );
    match result_invalid.unwrap_err() {
        FileError::WatchNotFound(id) => assert_eq!(id, watch_id),
        _ => panic!("Expected FileError::WatchNotFound"),
    }
}

#[tokio::test]
async fn test_unwatch_invalid_watch_id_returns_error() {
    let svc = FileService::new();
    let invalid_id = "nonexistent-watch-id-12345";

    let result = svc.unwatch(invalid_id).await;
    assert!(result.is_err(), "unwatch should fail for invalid watch_id");
    match result.unwrap_err() {
        FileError::WatchNotFound(id) => assert_eq!(id, invalid_id),
        _ => panic!("Expected FileError::WatchNotFound"),
    }
}
