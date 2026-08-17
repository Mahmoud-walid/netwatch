use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetWatchError {
    #[error("storage error")]
    Storage(#[from] StorageError),

    #[error("configuration error")]
    Config(#[from] ConfigError),

    #[error("database error")]
    Database(#[from] DatabaseError),

    #[error("traffic engine error")]
    Traffic(#[from] TrafficError),
}

#[derive(Debug, Error)]
pub enum TrafficError {
    #[error("collector error: {0}")]
    Collector(String),

    #[error("io error during traffic collection: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse traffic counter: {0}")]
    Parse(#[from] std::num::ParseIntError),
}

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("failed to connect to database at {path}: {source}")]
    ConnectionFailed {
        path: PathBuf,
        source: rusqlite::Error,
    },

    #[error("failed to apply database pragmas: {0}")]
    PragmaError(rusqlite::Error),

    #[error("database migration failed: {0}")]
    MigrationError(rusqlite::Error),

    #[error("database query failed: {0}")]
    Query(#[from] rusqlite::Error),
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("storage path is empty")]
    EmptyPath,

    #[error("storage path is not a directory: {0}")]
    PathIsNotDirectory(PathBuf),

    #[error("failed to create storage directory {path}: {source}")]
    FailedToCreateDirectory {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error(
        "mount safety triggered: storage was initialized on mount point {expected}, but currently found {found}. To prevent accidental root filesystem writes, NetWatch will not create the storage directory."
    )]
    MountMismatch { expected: PathBuf, found: PathBuf },

    #[error("mount provider error: {0}")]
    MountProvider(std::io::Error),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read configuration file {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to parse configuration file {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[error("failed to serialize configuration file {path}: {source}")]
    Serialize {
        path: PathBuf,
        source: toml::ser::Error,
    },
}
