use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Serialize)]
pub struct PoeData {
    /// PoE HAT detected in sysfs
    pub available: bool,
    /// Currently receiving PoE power
    pub online: bool,
    /// Current draw in Amps (converted from µA)
    pub current_amps: f64,
    /// Max rated current in Amps (converted from µA)
    pub current_max_amps: f64,
    /// Device name, e.g. "rpi-poe-0"
    pub device_name: String,
}

pub struct PoeCollector {
    #[allow(dead_code)]
    root: PathBuf,
    /// Discovered PoE power supply path, or None if not present
    poe_path: Option<PathBuf>,
}

impl PoeCollector {
    pub fn new(root: &Path) -> Self {
        let poe_path = discover_poe_device(root);
        Self {
            root: root.to_path_buf(),
            poe_path,
        }
    }

    pub fn collect(&self, data: &mut PoeData) -> Result<()> {
        // Reset to defaults
        *data = PoeData::default();

        let poe_path = match &self.poe_path {
            Some(p) => p,
            None => return Ok(()),
        };

        // If the path no longer exists (device removed), report unavailable
        if !poe_path.exists() {
            return Ok(());
        }

        data.available = true;
        data.device_name = poe_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Read online status: "1" means PoE is actively powering
        data.online = std::fs::read_to_string(poe_path.join("online"))
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok())
            .map(|v| v == 1)
            .unwrap_or(false);

        // Read current_now in microamps, convert to Amps
        data.current_amps = std::fs::read_to_string(poe_path.join("current_now"))
            .ok()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .map(|ua| ua / 1_000_000.0)
            .unwrap_or(0.0);

        // Read current_max in microamps, convert to Amps
        data.current_max_amps = std::fs::read_to_string(poe_path.join("current_max"))
            .ok()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .map(|ua| ua / 1_000_000.0)
            .unwrap_or(0.0);

        Ok(())
    }
}

/// Enumerate `/sys/class/power_supply/` and return the first path
/// whose name starts with `rpi-poe`.
fn discover_poe_device(root: &Path) -> Option<PathBuf> {
    let ps_dir = root.join("sys/class/power_supply");
    let entries = std::fs::read_dir(&ps_dir).ok()?;

    for entry in entries.flatten() {
        let name = entry.file_name();
        if name.to_string_lossy().starts_with("rpi-poe") {
            return Some(entry.path());
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
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
    }

    #[test]
    fn discovers_poe_device() {
        let tmp = TempDir::new().unwrap();
        write_fixture(tmp.path(), "sys/class/power_supply/rpi-poe-0/online", "1\n");

        let result = discover_poe_device(tmp.path());
        assert!(result.is_some());
        let path = result.unwrap();
        assert!(path.ends_with("rpi-poe-0"));
    }

    #[test]
    fn no_poe_device_when_directory_missing() {
        let tmp = TempDir::new().unwrap();
        assert!(discover_poe_device(tmp.path()).is_none());
    }

    #[test]
    fn no_poe_device_when_no_matching_entries() {
        let tmp = TempDir::new().unwrap();
        write_fixture(tmp.path(), "sys/class/power_supply/ac/online", "1\n");

        assert!(discover_poe_device(tmp.path()).is_none());
    }

    #[test]
    fn collects_poe_data_online() {
        let tmp = TempDir::new().unwrap();
        let poe_dir = "sys/class/power_supply/rpi-poe-0";

        write_fixture(tmp.path(), &format!("{poe_dir}/online"), "1\n");
        write_fixture(tmp.path(), &format!("{poe_dir}/current_now"), "500000\n");
        write_fixture(tmp.path(), &format!("{poe_dir}/current_max"), "2500000\n");

        let collector = PoeCollector::new(tmp.path());
        let mut data = PoeData::default();
        collector.collect(&mut data).unwrap();

        assert!(data.available);
        assert!(data.online);
        assert!((data.current_amps - 0.5).abs() < 0.001);
        assert!((data.current_max_amps - 2.5).abs() < 0.001);
        assert_eq!(data.device_name, "rpi-poe-0");
    }

    #[test]
    fn collects_poe_data_offline() {
        let tmp = TempDir::new().unwrap();
        let poe_dir = "sys/class/power_supply/rpi-poe-0";

        write_fixture(tmp.path(), &format!("{poe_dir}/online"), "0\n");
        write_fixture(tmp.path(), &format!("{poe_dir}/current_now"), "0\n");
        write_fixture(tmp.path(), &format!("{poe_dir}/current_max"), "2500000\n");

        let collector = PoeCollector::new(tmp.path());
        let mut data = PoeData::default();
        collector.collect(&mut data).unwrap();

        assert!(data.available);
        assert!(!data.online);
        assert!((data.current_amps).abs() < 0.001);
        assert!((data.current_max_amps - 2.5).abs() < 0.001);
    }

    #[test]
    fn handles_no_poe_hardware() {
        let tmp = TempDir::new().unwrap();
        let collector = PoeCollector::new(tmp.path());
        let mut data = PoeData::default();
        collector.collect(&mut data).unwrap();

        assert!(!data.available);
        assert!(!data.online);
        assert!((data.current_amps).abs() < 0.001);
    }

    #[test]
    fn handles_missing_current_files() {
        let tmp = TempDir::new().unwrap();
        let poe_dir = "sys/class/power_supply/rpi-poe-0";

        // Only online file exists, no current_now or current_max
        write_fixture(tmp.path(), &format!("{poe_dir}/online"), "1\n");

        let collector = PoeCollector::new(tmp.path());
        let mut data = PoeData::default();
        collector.collect(&mut data).unwrap();

        assert!(data.available);
        assert!(data.online);
        assert!((data.current_amps).abs() < 0.001);
        assert!((data.current_max_amps).abs() < 0.001);
    }

    #[test]
    fn handles_malformed_values() {
        let tmp = TempDir::new().unwrap();
        let poe_dir = "sys/class/power_supply/rpi-poe-0";

        write_fixture(tmp.path(), &format!("{poe_dir}/online"), "garbage\n");
        write_fixture(
            tmp.path(),
            &format!("{poe_dir}/current_now"),
            "not_a_number\n",
        );
        write_fixture(tmp.path(), &format!("{poe_dir}/current_max"), "\n");

        let collector = PoeCollector::new(tmp.path());
        let mut data = PoeData::default();
        collector.collect(&mut data).unwrap();

        assert!(data.available);
        assert!(!data.online);
        assert!((data.current_amps).abs() < 0.001);
        assert!((data.current_max_amps).abs() < 0.001);
    }

    #[test]
    fn resets_data_on_each_collect() {
        let tmp = TempDir::new().unwrap();
        let poe_dir = "sys/class/power_supply/rpi-poe-0";

        write_fixture(tmp.path(), &format!("{poe_dir}/online"), "1\n");
        write_fixture(tmp.path(), &format!("{poe_dir}/current_now"), "500000\n");
        write_fixture(tmp.path(), &format!("{poe_dir}/current_max"), "2500000\n");

        let collector = PoeCollector::new(tmp.path());

        // First collect
        let mut data = PoeData::default();
        collector.collect(&mut data).unwrap();
        assert!(data.available);
        assert!(data.online);

        // Modify data, then collect again to verify it resets
        data.current_amps = 999.0;
        collector.collect(&mut data).unwrap();
        assert!((data.current_amps - 0.5).abs() < 0.001);
    }
}
