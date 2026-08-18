pub mod accounting;
pub mod devices;
pub mod users;

use rusqlite::Connection;
use std::path::{Path, PathBuf};

use crate::error::DatabaseError;

const DB_FILE_NAME: &str = "netwatch.sqlite";

pub struct DatabaseManager {
    db_path: PathBuf,
}

impl DatabaseManager {
    pub fn initialize(storage_path: &Path) -> Result<Self, DatabaseError> {
        let db_path = storage_path.join(DB_FILE_NAME);

        let mut connection = Self::open_connection(&db_path)?;
        Self::run_migrations(&mut connection)?;

        Ok(Self { db_path })
    }

    pub fn get_connection(&self) -> Result<Connection, DatabaseError> {
        Self::open_connection(&self.db_path)
    }

    pub fn connect(storage_path: &Path) -> Result<Connection, DatabaseError> {
        let db_path = storage_path.join(DB_FILE_NAME);
        Self::open_connection(&db_path)
    }

    fn open_connection(db_path: &Path) -> Result<Connection, DatabaseError> {
        let connection =
            Connection::open(db_path).map_err(|source| DatabaseError::ConnectionFailed {
                path: db_path.to_path_buf(),
                source,
            })?;

        connection
            .execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA foreign_keys = ON;",
            )
            .map_err(DatabaseError::PragmaError)?;

        Ok(connection)
    }

    pub(crate) fn run_migrations(connection: &mut Connection) -> Result<(), DatabaseError> {
        let tx = connection
            .transaction()
            .map_err(DatabaseError::MigrationError)?;

        tx.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(DatabaseError::MigrationError)?;

        let current_version: i32 = tx
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let migrations = [
            // Version 1
            "
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS devices (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mac_address TEXT UNIQUE NOT NULL,
                ip_address TEXT,
                hostname TEXT,
                display_name TEXT,
                user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
                is_online BOOLEAN DEFAULT 0,
                first_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_seen DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            ",
            // Version 2
            "
            ALTER TABLE devices ADD COLUMN last_rx_bytes INTEGER DEFAULT 0;
            ALTER TABLE devices ADD COLUMN last_tx_bytes INTEGER DEFAULT 0;
            ALTER TABLE devices ADD COLUMN last_counter_update DATETIME;

            CREATE TABLE IF NOT EXISTS device_usage_daily (
                device_id INTEGER NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
                date TEXT NOT NULL,
                rx_bytes INTEGER DEFAULT 0,
                tx_bytes INTEGER DEFAULT 0,
                PRIMARY KEY (device_id, date)
            );
            ",
            // Version 3
            "
            ALTER TABLE devices ADD COLUMN vendor TEXT;
            ",
        ];

        for (i, &migration_sql) in migrations.iter().enumerate() {
            let version = (i + 1) as i32;
            if version > current_version {
                tx.execute_batch(migration_sql)
                    .map_err(DatabaseError::MigrationError)?;
                tx.execute(
                    "INSERT INTO schema_migrations (version) VALUES (?1)",
                    [version],
                )
                .map_err(DatabaseError::MigrationError)?;
            }
        }

        tx.commit().map_err(DatabaseError::MigrationError)?;
        Ok(())
    }
}

#[cfg(test)]
pub(crate) fn setup_test_db() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    DatabaseManager::run_migrations(&mut conn).unwrap();
    conn
}
