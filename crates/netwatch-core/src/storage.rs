use std::fs;
use std::path::{Path, PathBuf};

use crate::config::StorageConfig;
use crate::error::StorageError;
use crate::storage_mount::MountProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageStatus {
    Available,
    Missing,
    NotMounted,
}

pub struct StorageManager {
    path: PathBuf,
}

impl StorageManager {
    pub fn new(
        config: &StorageConfig,
        mount_provider: &dyn MountProvider,
    ) -> Result<Self, StorageError> {
        crate::config::validate_storage_path(&config.path)?;

        // Storage Safety Requirement: Ensure the target mount is still correct
        if let Some(expected_mount) = &config.expected_mount_point {
            let path_to_check = if config.path.exists() {
                config.path.as_path()
            } else {
                config.path.parent().unwrap_or_else(|| Path::new("/"))
            };

            let current_mount = mount_provider
                .mount_for_path(path_to_check)
                .map_err(StorageError::MountProvider)?;

            let is_mounted_correctly = match current_mount {
                Some(mount) => mount.mount_point == *expected_mount,
                None => false,
            };

            if !is_mounted_correctly {
                return Err(StorageError::MountMismatch {
                    expected: expected_mount.clone(),
                    found: PathBuf::from("unmounted or mismatched"),
                });
            }
        }

        if !config.path.exists() {
            fs::create_dir_all(&config.path).map_err(|source| {
                StorageError::FailedToCreateDirectory {
                    path: config.path.clone(),
                    source,
                }
            })?;
        }

        Ok(Self {
            path: config.path.clone(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn status(
        &self,
        config: &StorageConfig,
        mount_provider: &dyn MountProvider,
    ) -> StorageStatus {
        if let Some(expected_mount) = &config.expected_mount_point {
            let current = mount_provider.mount_for_path(&self.path).unwrap_or(None);
            if current.map_or(true, |m| m.mount_point != *expected_mount) {
                return StorageStatus::NotMounted;
            }
        }

        if !self.path.exists() || !self.path.is_dir() {
            return StorageStatus::Missing;
        }

        StorageStatus::Available
    }
}
