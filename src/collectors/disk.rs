use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Pseudo-filesystem types to filter out from mount listing.
const PSEUDO_FS: &[&str] = &[
    "sysfs",
    "proc",
    "tmpfs",
    "devtmpfs",
    "devpts",
    "squashfs",
    "overlay",
    "cgroup",
    "cgroup2",
    "pstore",
    "debugfs",
    "securityfs",
    "configfs",
    "fusectl",
    "mqueue",
    "hugetlbfs",
    "tracefs",
    "bpf",
    "autofs",
    "binfmt_misc",
    "rpc_pipefs",
    "nfsd",
    "efivarfs",
];

#[derive(Debug, Default, Clone, Serialize)]
pub struct PartitionInfo {
    pub device: String,
    pub mountpoint: String,
    pub fs_type: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct DiskData {
    pub partitions: Vec<PartitionInfo>,
}

pub struct DiskCollector {
    root: PathBuf,
}

impl DiskCollector {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    pub fn collect(&self, data: &mut DiskData) -> Result<()> {
        data.partitions.clear();

        let mounts_path = self.root.join("proc/mounts");
        let content = std::fs::read_to_string(&mounts_path).unwrap_or_default();

        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }

            let device = parts[0];
            let mountpoint = parts[1];
            let fs_type = parts[2];

            // Skip pseudo-filesystems
            if PSEUDO_FS.contains(&fs_type) {
                continue;
            }
            // Skip if device doesn't start with / (e.g., "none", "sunrpc")
            if !device.starts_with('/') {
                continue;
            }

            // Get disk usage via statvfs — only works on real mountpoints
            // On x86 test fixtures, this won't work, so we handle gracefully
            let (total, free, used, percent) = statvfs_usage(mountpoint);

            data.partitions.push(PartitionInfo {
                device: device.to_string(),
                mountpoint: mountpoint.to_string(),
                fs_type: fs_type.to_string(),
                total_bytes: total,
                used_bytes: used,
                free_bytes: free,
                usage_percent: percent,
            });
        }

        Ok(())
    }
}

/// Get disk usage for a mountpoint via libc statvfs.
fn statvfs_usage(mountpoint: &str) -> (u64, u64, u64, f64) {
    use std::ffi::CString;

    let c_path = match CString::new(mountpoint) {
        Ok(p) => p,
        Err(_) => return (0, 0, 0, 0.0),
    };

    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr(), &mut stat) != 0 {
            return (0, 0, 0, 0.0);
        }

        let block_size = stat.f_frsize as u64;
        let total = stat.f_blocks as u64 * block_size;
        let free = stat.f_bfree as u64 * block_size;
        let available = stat.f_bavail as u64 * block_size;
        let used = total.saturating_sub(free);
        let percent = if total > 0 {
            ((total - available) as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        (total, free, used, percent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    const MOUNTS: &str = "\
sysfs /sys sysfs rw,nosuid,nodev,noexec,relatime 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
/dev/mmcblk0p2 / ext4 rw,noatime 0 0
/dev/mmcblk0p1 /boot/firmware vfat rw,relatime 0 0
tmpfs /run tmpfs rw,nosuid,nodev,size=387584k,nr_inodes=819200,mode=755 0 0
";

    #[test]
    fn filters_pseudo_filesystems() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("proc/mounts");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, MOUNTS).unwrap();

        let collector = DiskCollector::new(tmp.path());
        let mut data = DiskData::default();
        collector.collect(&mut data).unwrap();

        // Only real filesystems should remain (ext4, vfat)
        // sysfs, proc, tmpfs should be filtered
        assert_eq!(data.partitions.len(), 2);
        assert_eq!(data.partitions[0].device, "/dev/mmcblk0p2");
        assert_eq!(data.partitions[0].mountpoint, "/");
        assert_eq!(data.partitions[0].fs_type, "ext4");
        assert_eq!(data.partitions[1].device, "/dev/mmcblk0p1");
    }

    #[test]
    fn handles_missing_mounts() {
        let tmp = TempDir::new().unwrap();
        let collector = DiskCollector::new(tmp.path());
        let mut data = DiskData::default();
        collector.collect(&mut data).unwrap();
        assert!(data.partitions.is_empty());
    }
}
