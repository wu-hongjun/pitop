use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Serialize)]
pub struct MemoryData {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub available_bytes: u64,
    pub buffers_bytes: u64,
    pub cached_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_free_bytes: u64,
    pub usage_percent: f64,
    pub swap_usage_percent: f64,
}

pub struct MemoryCollector {
    root: PathBuf,
}

impl MemoryCollector {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    pub fn collect(&self, data: &mut MemoryData) -> Result<()> {
        let meminfo_path = self.root.join("proc/meminfo");
        let content = std::fs::read_to_string(&meminfo_path).unwrap_or_default();

        let mut total = 0u64;
        let mut free = 0u64;
        let mut available = 0u64;
        let mut buffers = 0u64;
        let mut cached = 0u64;
        let mut swap_total = 0u64;
        let mut swap_free = 0u64;

        for line in content.lines() {
            if let Some((key, value_kb)) = parse_meminfo_line(line) {
                let value_bytes = value_kb * 1024;
                match key {
                    "MemTotal" => total = value_bytes,
                    "MemFree" => free = value_bytes,
                    "MemAvailable" => available = value_bytes,
                    "Buffers" => buffers = value_bytes,
                    "Cached" => cached = value_bytes,
                    "SwapTotal" => swap_total = value_bytes,
                    "SwapFree" => swap_free = value_bytes,
                    _ => {}
                }
            }
        }

        // Used = Total - Available (matches htop behavior)
        let used = total.saturating_sub(available);

        data.total_bytes = total;
        data.used_bytes = used;
        data.free_bytes = free;
        data.available_bytes = available;
        data.buffers_bytes = buffers;
        data.cached_bytes = cached;
        data.swap_total_bytes = swap_total;
        data.swap_free_bytes = swap_free;
        data.swap_used_bytes = swap_total.saturating_sub(swap_free);
        data.usage_percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        data.swap_usage_percent = if swap_total > 0 {
            (data.swap_used_bytes as f64 / swap_total as f64) * 100.0
        } else {
            0.0
        };

        Ok(())
    }
}

/// Parse a line like "MemTotal:        3884292 kB" → ("MemTotal", 3884292)
fn parse_meminfo_line(line: &str) -> Option<(&str, u64)> {
    let mut parts = line.splitn(2, ':');
    let key = parts.next()?.trim();
    let rest = parts.next()?.trim();
    // Remove " kB" suffix and parse
    let value_str = rest.split_whitespace().next()?;
    let value = value_str.parse::<u64>().ok()?;
    Some((key, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    const MEMINFO: &str = "\
MemTotal:        3884292 kB
MemFree:          158940 kB
MemAvailable:    1520648 kB
Buffers:          194732 kB
Cached:          1315260 kB
SwapCached:            0 kB
Active:          2045684 kB
Inactive:        1469844 kB
SwapTotal:        524284 kB
SwapFree:         324284 kB
";

    #[test]
    fn parses_meminfo() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("proc/meminfo");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, MEMINFO).unwrap();

        let collector = MemoryCollector::new(tmp.path());
        let mut data = MemoryData::default();
        collector.collect(&mut data).unwrap();

        assert_eq!(data.total_bytes, 3884292 * 1024);
        assert_eq!(data.free_bytes, 158940 * 1024);
        assert_eq!(data.available_bytes, 1520648 * 1024);
        assert_eq!(data.buffers_bytes, 194732 * 1024);
        assert_eq!(data.cached_bytes, 1315260 * 1024);
        // used = total - available
        assert_eq!(data.used_bytes, (3884292 - 1520648) * 1024);
        assert_eq!(data.swap_total_bytes, 524284 * 1024);
        assert_eq!(data.swap_free_bytes, 324284 * 1024);
        assert_eq!(data.swap_used_bytes, (524284 - 324284) * 1024);
        assert!(data.usage_percent > 0.0);
        assert!(data.swap_usage_percent > 0.0);
    }

    #[test]
    fn handles_missing_meminfo() {
        let tmp = TempDir::new().unwrap();
        let collector = MemoryCollector::new(tmp.path());
        let mut data = MemoryData::default();
        collector.collect(&mut data).unwrap();
        assert_eq!(data.total_bytes, 0);
    }
}
