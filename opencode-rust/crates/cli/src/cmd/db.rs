use clap::{Args, Subcommand};
use opencode_core::OpenCodeError;
use opencode_storage::database::StoragePool;
use opencode_storage::migration::MigrationManager;
use opencode_storage::service::StorageService;
use opencode_storage::{SqliteProjectRepository, SqliteSessionRepository};
use std::path::PathBuf;
use std::sync::Arc;

const CURRENT_DB_VERSION: i32 = 3;

#[derive(Args, Debug)]
pub(crate) struct DbArgs {
    #[command(subcommand)]
    pub action: DbAction,
}

#[derive(Subcommand, Debug)]
pub(crate) enum DbAction {
    Init,
    Migrate,
    Backup,
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_args_init() {
        let args = DbArgs {
            action: DbAction::Init,
        };
        match args.action {
            DbAction::Init => {}
            _ => panic!("Expected Init"),
        }
    }

    #[test]
    fn test_db_args_migrate() {
        let args = DbArgs {
            action: DbAction::Migrate,
        };
        match args.action {
            DbAction::Migrate => {}
            _ => panic!("Expected Migrate"),
        }
    }

    #[test]
    fn test_db_args_backup() {
        let args = DbArgs {
            action: DbAction::Backup,
        };
        match args.action {
            DbAction::Backup => {}
            _ => panic!("Expected Backup"),
        }
    }
}

pub(crate) fn run(args: DbArgs) {
    match args.action {
        DbAction::Init => run_init(),
        DbAction::Migrate => run_migrate(),
        DbAction::Backup => run_backup(),
    }
}

fn get_db_path() -> PathBuf {
    let project_dirs = directories::ProjectDirs::from("ai", "opencode", "opencode-rs")
        .expect("Failed to determine project directories");
    let data_dir = project_dirs.data_dir();
    std::fs::create_dir_all(data_dir).expect("Failed to create data directory");
    data_dir.join("opencode-rs.db")
}

fn run_init() {
    let db_path = get_db_path();
    println!("Initializing database at: {:?}", db_path);

    if db_path.exists() {
        println!("Database already exists at that location");
        return;
    }

    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    runtime.block_on(async {
        match init_database(&db_path).await {
            Ok(_) => println!("Database initialized successfully"),
            Err(e) => {
                eprintln!("Failed to initialize database: {}", e);
                std::process::exit(1);
            }
        }
    });
}

async fn init_database(db_path: &PathBuf) -> Result<(), OpenCodeError> {
    let pool = StoragePool::new(db_path)?;
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));
    let project_repo = Arc::new(SqliteProjectRepository::new(pool.clone()));
    let _storage = StorageService::new(session_repo, project_repo, pool);

    let manager = MigrationManager::new(
        StoragePool::new(db_path).map_err(|e| OpenCodeError::Storage(e.to_string()))?,
        CURRENT_DB_VERSION,
    );
    manager.migrate().await?;

    Ok(())
}

fn run_migrate() {
    let db_path = get_db_path();
    println!("Running migrations for database at: {:?}", db_path);

    if !db_path.exists() {
        eprintln!("Database does not exist at that location. Run 'opencode db init' first.");
        std::process::exit(1);
    }

    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    runtime.block_on(async {
        match run_migrations(&db_path).await {
            Ok(versions) => {
                if versions.is_empty() {
                    println!("Database is up to date (version {})", CURRENT_DB_VERSION);
                } else {
                    println!("Applied migrations: {:?}", versions);
                    println!("Database is now at version {}", CURRENT_DB_VERSION);
                }
            }
            Err(e) => {
                eprintln!("Failed to run migrations: {}", e);
                std::process::exit(1);
            }
        }
    });
}

async fn run_migrations(db_path: &PathBuf) -> Result<Vec<i32>, OpenCodeError> {
    let pool = StoragePool::new(db_path)?;
    let pool_for_migration = pool.clone();
    let manager = MigrationManager::new(pool_for_migration, CURRENT_DB_VERSION);

    let conn = pool.get().await?;
    let current_version: i32 = conn
        .execute(|c| {
            c.query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
        })
        .await
        .map_err(|e| OpenCodeError::Storage(e.to_string()))??;

    if current_version >= CURRENT_DB_VERSION {
        return Ok(vec![]);
    }

    manager.migrate().await?;
    Ok((current_version + 1..=CURRENT_DB_VERSION).collect())
}

fn run_backup() {
    let db_path = get_db_path();
    println!("Backing up database from: {:?}", db_path);

    if !db_path.exists() {
        eprintln!("Database does not exist at that location.");
        std::process::exit(1);
    }

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = db_path.with_extension(format!("db.backup_{}.sqlite", timestamp));

    match std::fs::copy(&db_path, &backup_path) {
        Ok(_) => println!("Database backed up to: {:?}", backup_path),
        Err(e) => {
            eprintln!("Failed to backup database: {}", e);
            std::process::exit(1);
        }
    }
}
