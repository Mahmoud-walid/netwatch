use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountInfo {
    pub mount_point: PathBuf,
}

pub trait MountProvider {
    fn mount_for_path(&self, path: &Path) -> std::io::Result<Option<MountInfo>>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LinuxMountProvider;

impl MountProvider for LinuxMountProvider {
    fn mount_for_path(&self, path: &Path) -> std::io::Result<Option<MountInfo>> {
        let output = Command::new("findmnt")
            .args(["-no", "TARGET"])
            .arg("--target")
            .arg(path)
            .output()?;

        if !output.status.success() {
            return Ok(None);
        }

        let mount_point = String::from_utf8_lossy(&output.stdout).trim().to_owned();

        if mount_point.is_empty() {
            return Ok(None);
        }

        Ok(Some(MountInfo {
            mount_point: PathBuf::from(mount_point),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_mount_for_root() {
        let provider = LinuxMountProvider;

        let result = provider
            .mount_for_path(Path::new("/"))
            .expect("failed to query root mount");

        let mount = result.expect("root filesystem should be mounted");

        assert!(!mount.mount_point.as_os_str().is_empty());
    }

    #[test]
    fn finds_mount_for_existing_storage_path() {
        let provider = LinuxMountProvider;

        let result = provider
            .mount_for_path(Path::new("/mnt/stor"))
            .expect("failed to query storage mount");

        let mount = result.expect("/mnt/stor should be mounted");

        assert_eq!(mount.mount_point, PathBuf::from("/mnt/stor"));
    }

    #[test]
    fn returns_none_for_unmounted_path() {
        let provider = LinuxMountProvider;

        let result = provider
            .mount_for_path(Path::new("/definitely-not-mounted/netwatch"))
            .expect("mount query should not fail");

        assert!(result.is_none());
    }
}
