use opencode_storage::database::StoragePool;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[tokio::test]
async fn test_storage_crash_001_can_create_fresh_after_corruption() {
    let temp_dir = tempdir().unwrap();
    let corrupted_db_path = temp_dir.path().join("corrupted.db");
    let fresh_db_path = temp_dir.path().join("fresh.db");

    {
        std::fs::write(&corrupted_db_path, b"corrupted database content").unwrap();
    }

    let corrupted_pool = StoragePool::new(&corrupted_db_path);
    if let Ok(pool) = corrupted_pool {
        let result = pool.get().await;
        if let Err(e) = result {
            let err_str = format!("{}", e);
            assert!(
                err_str.contains("Storage")
                    || err_str.contains("database")
                    || err_str.contains("sqlite"),
                "Error should be descriptive, got: {}",
                err_str
            );
        }
    }

    let fresh_pool = StoragePool::new(&fresh_db_path);
    assert!(
        fresh_pool.is_ok(),
        "Should be able to create fresh database after corruption"
    );

    let conn = fresh_pool.unwrap().get().await.unwrap();
    let result = conn
        .execute(|c| c.query_row("SELECT 1", [], |row| row.get::<_, i32>(0)))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result, 1);
    drop(temp_dir);
}

#[tokio::test]
async fn test_storage_crash_001_process_continues_after_corruption() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    {
        let pool = StoragePool::new(&db_path).unwrap();
        let conn = pool.get().await.unwrap();
        conn.execute(|c| c.execute("CREATE TABLE test_sessions (id TEXT)", []))
            .await
            .unwrap()
            .unwrap();
    }

    let mut file = File::create(&db_path).unwrap();
    file.write_all(b"XXXX").unwrap();
    drop(file);

    let pool = StoragePool::new(&db_path).unwrap();
    let result = pool.get().await;

    match result {
        Ok(_) => {}
        Err(e) => {
            let err_str = format!("{}", e);
            assert!(
                err_str.contains("Storage") || err_str.contains("database"),
                "Error should be descriptive, got: {}",
                err_str
            );
        }
    }

    let fresh_db_path = temp_dir.path().join("fresh2.db");
    let fresh_pool = StoragePool::new(&fresh_db_path);
    assert!(
        fresh_pool.is_ok(),
        "Should be able to create new database after corruption"
    );
    drop(temp_dir);
}

#[tokio::test]
async fn test_storage_crash_001_no_panic_on_corrupted_file() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    {
        let pool = StoragePool::new(&db_path).unwrap();
        let conn = pool.get().await.unwrap();
        conn.execute(|c| c.execute("CREATE TABLE test_sessions (id TEXT)", []))
            .await
            .unwrap()
            .unwrap();
    }

    let mut file = File::create(&db_path).unwrap();
    file.write_all(b"Invalid SQLite").unwrap();
    drop(file);

    let pool = StoragePool::new(&db_path).unwrap();
    let _result = pool.get().await;

    drop(temp_dir);
}
