mod types;

pub use types::Storage;

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    #[test]
    fn test_storage_save_load() {
        let tmp = TempDir::new().unwrap();
        let storage = Storage::new(tmp.path().to_path_buf());

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Data {
            value: i32,
        }

        storage.save("test", &Data { value: 42 }).unwrap();
        let loaded: Data = storage.load("test").unwrap();

        assert_eq!(loaded.value, 42);
    }

    #[test]
    fn test_storage_exists() {
        let tmp = TempDir::new().unwrap();
        let storage = Storage::new(tmp.path().to_path_buf());

        assert!(!storage.exists("test"));
        storage.save("test", &"data").unwrap();
        assert!(storage.exists("test"));
    }

    #[test]
    fn test_storage_delete() {
        let tmp = TempDir::new().unwrap();
        let storage = Storage::new(tmp.path().to_path_buf());

        storage.save("test", &"data").unwrap();
        assert!(storage.exists("test"));

        storage.delete("test").unwrap();
        assert!(!storage.exists("test"));
    }

    #[test]
    fn test_storage_list_keys() {
        let tmp = TempDir::new().unwrap();
        let storage = Storage::new(tmp.path().to_path_buf());

        storage.save("key1", &"data1").unwrap();
        storage.save("key2", &"data2").unwrap();

        let keys = storage.list_keys().unwrap();
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }

    #[test]
    fn test_storage_load_not_found() {
        let tmp = TempDir::new().unwrap();
        let storage = Storage::new(tmp.path().to_path_buf());

        let result: Result<String, _> = storage.load("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_storage_delete_nonexistent() {
        let tmp = TempDir::new().unwrap();
        let storage = Storage::new(tmp.path().to_path_buf());

        let result = storage.delete("nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_storage_save_and_load_struct() {
        let tmp = TempDir::new().unwrap();
        let storage = Storage::new(tmp.path().to_path_buf());

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestStruct {
            name: String,
            value: i32,
            items: Vec<String>,
        }

        let data = TestStruct {
            name: "test".to_string(),
            value: 42,
            items: vec!["a".to_string(), "b".to_string()],
        };

        storage.save("struct_test", &data).unwrap();
        let loaded: TestStruct = storage.load("struct_test").unwrap();

        assert_eq!(loaded.name, "test");
        assert_eq!(loaded.value, 42);
        assert_eq!(loaded.items, vec!["a", "b"]);
    }

    #[test]
    fn test_storage_overwrite() {
        let tmp = TempDir::new().unwrap();
        let storage = Storage::new(tmp.path().to_path_buf());

        storage.save("key", &"original").unwrap();
        assert_eq!(storage.load::<String>("key").unwrap(), "original");

        storage.save("key", &"updated").unwrap();
        assert_eq!(storage.load::<String>("key").unwrap(), "updated");
    }

    #[test]
    fn test_storage_exists_false_for_nonexistent() {
        let tmp = TempDir::new().unwrap();
        let storage = Storage::new(tmp.path().to_path_buf());

        assert!(!storage.exists("nonexistent_key"));
    }
}