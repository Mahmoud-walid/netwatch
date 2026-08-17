//! NetWatch runtime configuration.

use crate::error::StorageError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const DEFAULT_STORAGE_DIRECTORY_NAME: &str = "netwatch";
const DEFAULT_CONFIG_DIRECTORY_NAME: &str = "netwatch";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetWatchConfig {
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageConfig {
    pub path: PathBuf,
    pub expected_mount_point: Option<PathBuf>,
}

impl NetWatchConfig {
    pub fn new(storage_path: PathBuf, expected_mount_point: Option<PathBuf>) -> Self {
        Self {
            storage: StorageConfig {
                path: storage_path,
                expected_mount_point,
            },
        }
    }

    pub fn default_storage_path() -> Option<PathBuf> {
        dirs::data_local_dir().map(|base| base.join(DEFAULT_STORAGE_DIRECTORY_NAME))
    }

    pub fn default_config_directory() -> Option<PathBuf> {
        dirs::config_dir().map(|base| base.join(DEFAULT_CONFIG_DIRECTORY_NAME))
    }

    pub fn validate_storage_path(&self) -> Result<(), StorageError> {
        validate_storage_path(&self.storage.path)
    }
}

pub fn validate_storage_path(path: &Path) -> Result<(), StorageError> {
    if path.as_os_str().is_empty() {
        return Err(StorageError::EmptyPath);
    }

    if path.exists() && !path.is_dir() {
        return Err(StorageError::PathIsNotDirectory(path.to_path_buf()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_non_existing_storage_directory() {
        let config = NetWatchConfig::new(PathBuf::from("/mnt/stor/netwatch-data"), None);
        assert!(matches!(config.validate_storage_path(), Ok(())));
    }

    #[test]
    fn rejects_empty_storage_path() {
        let config = NetWatchConfig::new(PathBuf::new(), None);
        assert!(matches!(
            config.validate_storage_path(),
            Err(StorageError::EmptyPath)
        ));
    }

    #[test]
    fn default_config_directory_uses_config_directory() {
        let path = NetWatchConfig::default_config_directory().unwrap();
        assert_eq!(path.file_name(), Some(std::ffi::OsStr::new("netwatch")));
    }
}
