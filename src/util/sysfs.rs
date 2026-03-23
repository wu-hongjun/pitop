use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Read a sysfs/procfs file and return its contents trimmed of whitespace.
pub fn read_sysfs_string(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .with_context(|| format!("Failed to read {}", path.display()))
}

/// Read a sysfs/procfs file and parse it as a u64.
pub fn read_sysfs_u64(path: &Path) -> Result<u64> {
    let content = read_sysfs_string(path)?;
    content.parse::<u64>().with_context(|| {
        format!(
            "Failed to parse '{}' as u64 from {}",
            content,
            path.display()
        )
    })
}

/// Read a sysfs/procfs file and parse it as a f64.
pub fn read_sysfs_f64(path: &Path) -> Result<f64> {
    let content = read_sysfs_string(path)?;
    content.parse::<f64>().with_context(|| {
        format!(
            "Failed to parse '{}' as f64 from {}",
            content,
            path.display()
        )
    })
}

/// Discover an hwmon device by its `name` file content.
///
/// Enumerates `/sys/class/hwmon/hwmon*` and returns the path of the first
/// device whose `name` file matches the given string.
///
/// The `root` parameter enables fixture-based testing.
pub fn discover_hwmon(root: &Path, name: &str) -> Option<PathBuf> {
    let hwmon_dir = root.join("sys/class/hwmon");
    let entries = std::fs::read_dir(&hwmon_dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name_file = path.join("name");
        if let Ok(content) = std::fs::read_to_string(&name_file) {
            if content.trim() == name {
                return Some(path);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_fixture(dir: &Path, relative: &str, content: &str) {
        let path = dir.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, content).unwrap();
    }

    #[test]
    fn read_string_trims_whitespace() {
        let tmp = TempDir::new().unwrap();
        write_fixture(tmp.path(), "value", "  hello world  \n");
        let result = read_sysfs_string(&tmp.path().join("value")).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn read_string_missing_file() {
        let tmp = TempDir::new().unwrap();
        let result = read_sysfs_string(&tmp.path().join("nonexistent"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to read"));
    }

    #[test]
    fn read_u64_valid() {
        let tmp = TempDir::new().unwrap();
        write_fixture(tmp.path(), "freq", "1800000\n");
        assert_eq!(read_sysfs_u64(&tmp.path().join("freq")).unwrap(), 1800000);
    }

    #[test]
    fn read_u64_invalid() {
        let tmp = TempDir::new().unwrap();
        write_fixture(tmp.path(), "bad", "not_a_number\n");
        assert!(read_sysfs_u64(&tmp.path().join("bad")).is_err());
    }

    #[test]
    fn read_f64_valid() {
        let tmp = TempDir::new().unwrap();
        write_fixture(tmp.path(), "temp", "52300\n");
        assert!((read_sysfs_f64(&tmp.path().join("temp")).unwrap() - 52300.0).abs() < 0.01);
    }

    #[test]
    fn discover_hwmon_found() {
        let tmp = TempDir::new().unwrap();
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon0/name", "cpu_thermal\n");
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon1/name", "cooling_fan\n");
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon2/name", "rp1_adc\n");

        let result = discover_hwmon(tmp.path(), "cooling_fan");
        assert!(result.is_some());
        assert!(result.unwrap().ends_with("hwmon1"));
    }

    #[test]
    fn discover_hwmon_not_found() {
        let tmp = TempDir::new().unwrap();
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon0/name", "cpu_thermal\n");

        assert!(discover_hwmon(tmp.path(), "cooling_fan").is_none());
    }

    #[test]
    fn discover_hwmon_no_directory() {
        let tmp = TempDir::new().unwrap();
        assert!(discover_hwmon(tmp.path(), "anything").is_none());
    }
}
