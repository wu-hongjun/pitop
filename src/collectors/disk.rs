use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

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

/// Prefixes that indicate virtual/pseudo block devices to skip in I/O stats.
const SKIP_DEVICE_PREFIXES: &[&str] = &["ram", "loop", "dm-"];

/// Sector size in bytes (standard for Linux block devices).
const SECTOR_SIZE: u64 = 512;

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
pub struct DiskIoInfo {
    pub device: String,
    pub read_bytes_per_sec: f64,
    pub write_bytes_per_sec: f64,
    pub total_read_bytes: u64,
    pub total_write_bytes: u64,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct DiskData {
    pub partitions: Vec<PartitionInfo>,
    pub io_stats: Vec<DiskIoInfo>,
}

struct PrevDiskSample {
    sectors_read: u64,
    sectors_written: u64,
    timestamp: Instant,
}

pub struct DiskCollector {
    root: PathBuf,
    prev: HashMap<String, PrevDiskSample>,
}

impl DiskCollector {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            prev: HashMap::new(),
        }
    }

    pub fn collect(&mut self, data: &mut DiskData) -> Result<()> {
        data.partitions.clear();
        data.io_stats.clear();

        self.collect_partitions(data)?;
        self.collect_io_stats(data);

        Ok(())
    }

    fn collect_partitions(&self, data: &mut DiskData) -> Result<()> {
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

    fn collect_io_stats(&mut self, data: &mut DiskData) {
        let diskstats_path = self.root.join("proc/diskstats");
        let content = match std::fs::read_to_string(&diskstats_path) {
            Ok(c) => c,
            Err(_) => return, // /proc/diskstats not available — graceful degradation
        };
        let now = Instant::now();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some((name, sectors_read, sectors_written)) = parse_diskstats_line(line) {
                // Skip virtual/pseudo devices
                if SKIP_DEVICE_PREFIXES
                    .iter()
                    .any(|prefix| name.starts_with(prefix))
                {
                    continue;
                }

                let total_read_bytes = sectors_read * SECTOR_SIZE;
                let total_write_bytes = sectors_written * SECTOR_SIZE;

                let (read_rate, write_rate) = if let Some(prev) = self.prev.get(&name) {
                    let elapsed = now.duration_since(prev.timestamp).as_secs_f64();
                    if elapsed > 0.0 {
                        let read_delta =
                            sectors_read.saturating_sub(prev.sectors_read) * SECTOR_SIZE;
                        let write_delta =
                            sectors_written.saturating_sub(prev.sectors_written) * SECTOR_SIZE;
                        (read_delta as f64 / elapsed, write_delta as f64 / elapsed)
                    } else {
                        (0.0, 0.0)
                    }
                } else {
                    (0.0, 0.0)
                };

                data.io_stats.push(DiskIoInfo {
                    device: name.clone(),
                    read_bytes_per_sec: read_rate,
                    write_bytes_per_sec: write_rate,
                    total_read_bytes,
                    total_write_bytes,
                });

                self.prev.insert(
                    name,
                    PrevDiskSample {
                        sectors_read,
                        sectors_written,
                        timestamp: now,
                    },
                );
            }
        }
    }
}

/// Get disk usage for a mountpoint via libc statvfs.
#[allow(clippy::unnecessary_cast)] // f_frsize type varies by platform
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

/// Parse a line from /proc/diskstats.
/// Format: "major minor name rd_completed rd_merged rd_sectors rd_ms wr_completed wr_merged wr_sectors wr_ms ios_in_progress io_ms weighted_io_ms"
/// Returns (device_name, sectors_read, sectors_written).
fn parse_diskstats_line(line: &str) -> Option<(String, u64, u64)> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    // Need at least 14 fields (major, minor, name, + 11 stat fields)
    if fields.len() < 14 {
        return None;
    }
    let name = fields[2].to_string();
    let sectors_read: u64 = fields[5].parse().ok()?;
    let sectors_written: u64 = fields[9].parse().ok()?;
    Some((name, sectors_read, sectors_written))
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

    const DISKSTATS: &str = "\
   1       0 ram0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
   7       0 loop0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 179       0 mmcblk0 5492 1234 198304 12340 3210 567 95872 8760 0 15600 21100 0 0 0 0 0 0
 179       1 mmcblk0p1 120 0 3840 480 45 0 1536 120 0 480 600 0 0 0 0 0 0
 179       2 mmcblk0p2 5200 1234 190624 11680 3100 567 93312 8520 0 14800 20200 0 0 0 0 0 0
 253       0 dm-0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
";

    #[test]
    fn filters_pseudo_filesystems() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("proc/mounts");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, MOUNTS).unwrap();

        let mut collector = DiskCollector::new(tmp.path());
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
        let mut collector = DiskCollector::new(tmp.path());
        let mut data = DiskData::default();
        collector.collect(&mut data).unwrap();
        assert!(data.partitions.is_empty());
    }

    #[test]
    fn parses_diskstats() {
        let tmp = TempDir::new().unwrap();
        let proc_dir = tmp.path().join("proc");
        fs::create_dir_all(&proc_dir).unwrap();
        fs::write(proc_dir.join("diskstats"), DISKSTATS).unwrap();
        // Also write empty mounts to avoid error
        fs::write(proc_dir.join("mounts"), "").unwrap();

        let mut collector = DiskCollector::new(tmp.path());
        let mut data = DiskData::default();
        collector.collect(&mut data).unwrap();

        // ram0, loop0, dm-0 should be filtered out; mmcblk0, mmcblk0p1, mmcblk0p2 remain
        assert_eq!(data.io_stats.len(), 3);
        assert_eq!(data.io_stats[0].device, "mmcblk0");
        assert_eq!(data.io_stats[0].total_read_bytes, 198304 * 512);
        assert_eq!(data.io_stats[0].total_write_bytes, 95872 * 512);
        assert_eq!(data.io_stats[1].device, "mmcblk0p1");
        assert_eq!(data.io_stats[2].device, "mmcblk0p2");

        // First sample: rates should be 0
        assert_eq!(data.io_stats[0].read_bytes_per_sec, 0.0);
        assert_eq!(data.io_stats[0].write_bytes_per_sec, 0.0);
    }

    #[test]
    fn handles_missing_diskstats() {
        let tmp = TempDir::new().unwrap();
        let proc_dir = tmp.path().join("proc");
        fs::create_dir_all(&proc_dir).unwrap();
        fs::write(proc_dir.join("mounts"), "").unwrap();
        // No diskstats file

        let mut collector = DiskCollector::new(tmp.path());
        let mut data = DiskData::default();
        collector.collect(&mut data).unwrap();
        assert!(data.io_stats.is_empty());
    }

    #[test]
    fn parse_diskstats_line_works() {
        let line =
            " 179       0 mmcblk0 5492 1234 198304 12340 3210 567 95872 8760 0 15600 21100 0 0 0 0 0 0";
        let (name, sectors_read, sectors_written) = parse_diskstats_line(line).unwrap();
        assert_eq!(name, "mmcblk0");
        assert_eq!(sectors_read, 198304);
        assert_eq!(sectors_written, 95872);
    }

    #[test]
    fn parse_diskstats_line_rejects_short() {
        let line = " 179       0 mmcblk0";
        assert!(parse_diskstats_line(line).is_none());
    }
}
