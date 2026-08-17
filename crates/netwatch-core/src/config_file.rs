use std::fs;
use std::path::{Path, PathBuf};

use crate::config::NetWatchConfig;
use crate::error::{ConfigError, NetWatchError};

pub struct ConfigFile;

impl ConfigFile {
    pub fn load(path: &Path) -> Result<NetWatchConfig, NetWatchError> {
        let contents = fs::read_to_string(path).map_err(|source| {
            NetWatchError::Config(ConfigError::Io {
                path: path.to_path_buf(),
                source,
            })
        })?;

        toml::from_str(&contents).map_err(|source| {
            NetWatchError::Config(ConfigError::Parse {
                path: path.to_path_buf(),
                source,
            })
        })
    }

    pub fn save(path: &Path, config: &NetWatchConfig) -> Result<(), NetWatchError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| {
                NetWatchError::Config(ConfigError::Io {
                    path: parent.to_path_buf(),
                    source,
                })
            })?;
        }

        let contents = toml::to_string_pretty(config).map_err(|source| {
            NetWatchError::Config(ConfigError::Serialize {
                path: path.to_path_buf(),
                source,
            })
        })?;

        fs::write(path, contents).map_err(|source| {
            NetWatchError::Config(ConfigError::Io {
                path: path.to_path_buf(),
                source,
            })
        })?;

        Ok(())
    }

    pub fn default_path() -> Option<PathBuf> {
        NetWatchConfig::default_config_directory().map(|directory| directory.join("config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StorageConfig;

    fn test_config_path(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join("netwatch-config-tests")
            .join(name)
            .join("config.toml")
    }

    fn cleanup_test_directory(path: &Path) {
        if let Some(parent) = path.parent()
            && parent.exists()
        {
            std::fs::remove_dir_all(parent).expect("failed to clean test directory");
        }
    }

    #[test]
    fn saves_and_loads_configuration() {
        let path = test_config_path("round-trip");
        cleanup_test_directory(&path);

        let original = NetWatchConfig::new(PathBuf::from("/mnt/example/netwatch-data"), None);

        ConfigFile::save(&path, &original).expect("failed to save configuration");
        assert!(path.is_file());

        let loaded = ConfigFile::load(&path).expect("failed to load configuration");
        assert_eq!(loaded, original);

        cleanup_test_directory(&path);
    }

    #[test]
    fn load_fails_for_missing_file() {
        let path = test_config_path("missing");

        cleanup_test_directory(&path);

        let result = ConfigFile::load(&path);

        assert!(matches!(
            result,
            Err(NetWatchError::Config(ConfigError::Io { .. }))
        ));
    }

    #[test]
    fn load_fails_for_invalid_toml() {
        let path = test_config_path("invalid");

        cleanup_test_directory(&path);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("failed to create test directory");
        }

        std::fs::write(&path, "[storage\npath = ").expect("failed to write invalid TOML");

        let result = ConfigFile::load(&path);

        assert!(matches!(
            result,
            Err(NetWatchError::Config(ConfigError::Parse { .. }))
        ));

        cleanup_test_directory(&path);
    }

    #[test]
    fn save_creates_parent_directories() {
        let path = test_config_path("nested-save");
        cleanup_test_directory(&path);

        let config = NetWatchConfig {
            storage: StorageConfig {
                path: PathBuf::from("/mnt/example/netwatch-data"),
                expected_mount_point: None,
            },
        };

        ConfigFile::save(&path, &config).expect("failed to save configuration");
        assert!(path.is_file());

        cleanup_test_directory(&path);
    }

    #[test]
    fn default_path_points_to_config_toml() {
        let path = ConfigFile::default_path().expect("expected a default config path");

        assert_eq!(path.file_name(), Some(std::ffi::OsStr::new("config.toml")));
    }
}
