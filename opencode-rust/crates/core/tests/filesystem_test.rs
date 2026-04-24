#[cfg(test)]
mod tests {

    use tempfile::tempdir;

    #[test]
    fn test_filesystem_exists_file() {
        let tmp = tempdir().unwrap();
        let filepath = tmp.path().join("test.txt");
        std::fs::write(&filepath, "content").unwrap();

        assert!(opencode_core::filesystem::AppFileSystem::exists(
            &filepath.to_string_lossy()
        ));
    }

    #[test]
    fn test_filesystem_exists_nonexistent() {
        let tmp = tempdir().unwrap();
        let filepath = tmp.path().join("does_not_exist.txt");

        assert!(!opencode_core::filesystem::AppFileSystem::exists(
            &filepath.to_string_lossy()
        ));
    }

    #[test]
    fn test_filesystem_exists_directory() {
        let tmp = tempdir().unwrap();
        let dirpath = tmp.path().join("subdir");
        std::fs::create_dir(&dirpath).unwrap();

        assert!(opencode_core::filesystem::AppFileSystem::exists(
            &dirpath.to_string_lossy()
        ));
    }

    #[test]
    fn test_filesystem_is_dir() {
        let tmp = tempdir().unwrap();
        let dirpath = tmp.path().join("testdir");
        std::fs::create_dir(&dirpath).unwrap();

        assert!(opencode_core::filesystem::AppFileSystem::is_dir(
            &dirpath.to_string_lossy()
        ));
    }

    #[test]
    fn test_filesystem_is_dir_file() {
        let tmp = tempdir().unwrap();
        let filepath = tmp.path().join("test.txt");
        std::fs::write(&filepath, "content").unwrap();

        assert!(!opencode_core::filesystem::AppFileSystem::is_dir(
            &filepath.to_string_lossy()
        ));
    }

    #[test]
    fn test_filesystem_size() {
        let tmp = tempdir().unwrap();
        let filepath = tmp.path().join("test.txt");
        let content = "Hello, World!";
        std::fs::write(&filepath, content).unwrap();

        let metadata = std::fs::metadata(&filepath).unwrap();
        assert_eq!(metadata.len(), content.len() as u64);
    }

    #[test]
    fn test_filesystem_read() {
        let tmp = tempdir().unwrap();
        let filepath = tmp.path().join("test.txt");
        let content = "test content";
        std::fs::write(&filepath, content).unwrap();

        let read_content = std::fs::read_to_string(&filepath).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_filesystem_write() {
        let tmp = tempdir().unwrap();
        let filepath = tmp.path().join("test.txt");
        let content = "new content";

        std::fs::write(&filepath, content).unwrap();
        let read_content = std::fs::read_to_string(&filepath).unwrap();
        assert_eq!(read_content, content);
    }
}
