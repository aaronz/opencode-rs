use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use opencode_core::{Message, Session};
use opencode_storage::database::StoragePool;
use opencode_storage::migration::MigrationManager;
use opencode_storage::sqlite_repository::SqliteSessionRepository;
use opencode_storage::SessionRepository;
use std::sync::Arc;
use std::time::Duration;

fn create_test_session(msg_count: usize) -> Session {
    let mut session = Session::new();
    for i in 0..msg_count {
        session.add_message(Message::user(format!("User message {}", i)));
        session.add_message(Message::assistant(format!("Assistant response {}", i)));
    }
    session
}

fn bench_session_create(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_load");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(100);

    for msg_count in [10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("session_create", msg_count),
            &msg_count,
            |b, &msg_count| {
                b.iter(|| {
                    let session = create_test_session(msg_count);
                    black_box(session)
                });
            },
        );
    }
}

fn bench_session_save(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_load");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(100);

    for msg_count in [10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("session_save", msg_count),
            &msg_count,
            |b, &msg_count| {
                let temp_dir = tempfile::tempdir().unwrap();
                let db_path = temp_dir.path().join("bench.db");
                let pool = StoragePool::new(&db_path).unwrap();
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let migration = MigrationManager::new(pool.clone(), 2);
                    migration.migrate().await.unwrap();
                });
                let repo = Arc::new(SqliteSessionRepository::new(pool));

                b.iter(|| {
                    let mut session = Session::new();
                    for i in 0..msg_count {
                        session.add_message(Message::user(format!("User message {}", i)));
                        session.add_message(Message::assistant(format!("Assistant response {}", i)));
                    }
                    let id = session.id;
                    rt.block_on(repo.save(&session)).unwrap();
                    black_box(id);
                });

                drop(temp_dir);
            },
        );
    }
}

fn bench_session_resume(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_load");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(100);

    for msg_count in [10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("session_resume", msg_count),
            &msg_count,
            |b, &msg_count| {
                let temp_dir = tempfile::tempdir().unwrap();
                let db_path = temp_dir.path().join("bench.db");
                let pool = StoragePool::new(&db_path).unwrap();
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let migration = MigrationManager::new(pool.clone(), 2);
                    migration.migrate().await.unwrap();
                });
                let repo = Arc::new(SqliteSessionRepository::new(pool));

                let session = create_test_session(msg_count);
                let session_id = session.id.to_string();
                rt.block_on(repo.save(&session)).unwrap();

                b.iter(|| {
                    let id = session_id.clone();
                    let loaded = rt.block_on(repo.find_by_id(&id)).unwrap();
                    black_box(loaded.is_some());
                });

                drop(temp_dir);
            },
        );
    }
}

fn bench_session_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_load");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(100);

    for msg_count in [10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("session_delete", msg_count),
            &msg_count,
            |b, &msg_count| {
                let temp_dir = tempfile::tempdir().unwrap();
                let db_path = temp_dir.path().join("bench.db");
                let pool = StoragePool::new(&db_path).unwrap();
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let migration = MigrationManager::new(pool.clone(), 2);
                    migration.migrate().await.unwrap();
                });
                let repo = Arc::new(SqliteSessionRepository::new(pool));

                let session = create_test_session(msg_count);
                let session_id = session.id.to_string();
                rt.block_on(repo.save(&session)).unwrap();

                b.iter(|| {
                    let new_session = create_test_session(msg_count);
                    let new_id = new_session.id.to_string();
                    rt.block_on(repo.delete(&session_id)).unwrap();
                    rt.block_on(repo.save(&new_session)).unwrap();
                    black_box(new_id);
                });

                drop(temp_dir);
            },
        );
    }
}

fn bench_cold_vs_warm(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_load");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(50);
    let msg_count = 50;

    group.bench_function("cold_start_save", |b| {
        b.iter(|| {
            let temp_dir = tempfile::tempdir().unwrap();
            let db_path = temp_dir.path().join("bench.db");
            let pool = StoragePool::new(&db_path).unwrap();
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let migration = MigrationManager::new(pool.clone(), 2);
                migration.migrate().await.unwrap();
            });
            let repo = Arc::new(SqliteSessionRepository::new(pool));

            let mut session = Session::new();
            for i in 0..msg_count {
                session.add_message(Message::user(format!("User message {}", i)));
                session.add_message(Message::assistant(format!("Assistant response {}", i)));
            }
            let id = session.id;
            rt.block_on(repo.save(&session)).unwrap();
            black_box(id);

            drop(temp_dir);
        });
    });

    group.bench_function("warm_save", |b| {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("bench.db");
        let pool = StoragePool::new(&db_path).unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let migration = MigrationManager::new(pool.clone(), 2);
            migration.migrate().await.unwrap();
        });
        let repo = Arc::new(SqliteSessionRepository::new(pool));

        b.iter(|| {
            let mut session = Session::new();
            for i in 0..msg_count {
                session.add_message(Message::user(format!("User message {}", i)));
                session.add_message(Message::assistant(format!("Assistant response {}", i)));
            }
            let id = session.id;
            rt.block_on(repo.save(&session)).unwrap();
            black_box(id);
        });

        drop(temp_dir);
    });
}

fn bench_percentiles(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_load");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(200);
    let msg_count = 50;

    group.bench_function("save_percentiles", |b| {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("bench.db");
        let pool = StoragePool::new(&db_path).unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let migration = MigrationManager::new(pool.clone(), 2);
            migration.migrate().await.unwrap();
        });
        let repo = Arc::new(SqliteSessionRepository::new(pool));

        b.iter(|| {
            let mut session = Session::new();
            for i in 0..msg_count {
                session.add_message(Message::user(format!("User message {}", i)));
                session.add_message(Message::assistant(format!("Assistant response {}", i)));
            }
            let id = session.id;
            rt.block_on(repo.save(&session)).unwrap();
            black_box(id);
        });

        drop(temp_dir);
    });

    group.bench_function("resume_percentiles", |b| {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("bench.db");
        let pool = StoragePool::new(&db_path).unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let migration = MigrationManager::new(pool.clone(), 2);
            migration.migrate().await.unwrap();
        });
        let repo = Arc::new(SqliteSessionRepository::new(pool));

        let mut session = Session::new();
        for i in 0..msg_count {
            session.add_message(Message::user(format!("User message {}", i)));
            session.add_message(Message::assistant(format!("Assistant response {}", i)));
        }
        let session_id = session.id.to_string();
        rt.block_on(repo.save(&session)).unwrap();

        b.iter(|| {
            let id = session_id.clone();
            let loaded = rt.block_on(repo.find_by_id(&id)).unwrap();
            black_box(loaded.is_some());
        });

        drop(temp_dir);
    });
}

criterion_group!(
    benches,
    bench_session_create,
    bench_session_save,
    bench_session_resume,
    bench_session_delete,
    bench_cold_vs_warm,
    bench_percentiles
);
criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_session() {
        let mut session = Session::new();
        for i in 0..10 {
            session.add_message(Message::user(format!("User message {}", i)));
            session.add_message(Message::assistant(format!("Assistant response {}", i)));
        }
        assert_eq!(session.messages.len(), 20);
    }
}