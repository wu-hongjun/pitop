use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Serialize)]
pub struct PcieDevice {
    pub address: String,
    pub vendor_id: String,
    pub device_id: String,
    pub current_speed: String,
    pub current_width: u8,
    pub max_speed: String,
    pub max_width: u8,
    pub gen_label: String,
    pub downgraded: bool,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct PcieData {
    pub devices: Vec<PcieDevice>,
}

pub struct PcieCollector {
    root: PathBuf,
}

fn speed_to_gen(speed: &str) -> &str {
    if speed.contains("8.0") {
        "Gen 3"
    } else if speed.contains("5.0") {
        "Gen 2"
    } else if speed.contains("2.5") {
        "Gen 1"
    } else {
        "Unknown"
    }
}

/// Extract the numeric GT/s value from a speed string like "8.0 GT/s".
/// Returns None if the string cannot be parsed.
fn parse_gts(speed: &str) -> Option<f64> {
    speed
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
}

/// Read a sysfs attribute file, returning None if the file doesn't exist or
/// can't be read.
fn read_attr(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
}

/// Check if a PCI class code represents a bridge (class 0x0604xx).
fn is_bridge(class_str: &str) -> bool {
    // Class codes in sysfs look like "0x060400" (with 0x prefix)
    let stripped = class_str.strip_prefix("0x").unwrap_or(class_str);
    stripped.starts_with("0604")
}

impl PcieCollector {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    pub fn collect(&self, data: &mut PcieData) -> Result<()> {
        data.devices.clear();

        let pci_dir = self.root.join("sys/bus/pci/devices");
        let entries = match std::fs::read_dir(&pci_dir) {
            Ok(e) => e,
            Err(_) => return Ok(()), // No PCI bus available
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let address = entry.file_name().to_string_lossy().to_string();

            // Read class code and skip bridges
            let class_code = read_attr(&path.join("class")).unwrap_or_default();
            if is_bridge(&class_code) {
                continue;
            }

            let vendor_id = read_attr(&path.join("vendor")).unwrap_or_default();
            let device_id = read_attr(&path.join("device")).unwrap_or_default();

            let current_speed = read_attr(&path.join("current_link_speed")).unwrap_or_default();
            let current_width_str = read_attr(&path.join("current_link_width")).unwrap_or_default();
            let max_speed = read_attr(&path.join("max_link_speed")).unwrap_or_default();
            let max_width_str = read_attr(&path.join("max_link_width")).unwrap_or_default();

            let current_width = current_width_str.parse::<u8>().unwrap_or(0);
            let max_width = max_width_str.parse::<u8>().unwrap_or(0);

            let gen_label = speed_to_gen(&current_speed).to_string();

            // Detect downgrade: current speed < max speed or current width < max width
            let current_gts = parse_gts(&current_speed).unwrap_or(0.0);
            let max_gts = parse_gts(&max_speed).unwrap_or(0.0);
            let downgraded = (current_gts > 0.0 && max_gts > 0.0 && current_gts < max_gts)
                || (current_width > 0 && max_width > 0 && current_width < max_width);

            data.devices.push(PcieDevice {
                address,
                vendor_id,
                device_id,
                current_speed,
                current_width,
                max_speed,
                max_width,
                gen_label,
                downgraded,
            });
        }

        // Sort by address for deterministic output
        data.devices.sort_by(|a, b| a.address.cmp(&b.address));

        Ok(())
    }
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

    fn create_pci_device(dir: &Path, address: &str, class: &str, vendor: &str, device: &str) {
        let base = format!("sys/bus/pci/devices/{address}");
        write_fixture(dir, &format!("{base}/class"), class);
        write_fixture(dir, &format!("{base}/vendor"), vendor);
        write_fixture(dir, &format!("{base}/device"), device);
    }

    fn set_link_info(
        dir: &Path,
        address: &str,
        cur_speed: &str,
        cur_width: &str,
        max_speed: &str,
        max_width: &str,
    ) {
        let base = format!("sys/bus/pci/devices/{address}");
        write_fixture(dir, &format!("{base}/current_link_speed"), cur_speed);
        write_fixture(dir, &format!("{base}/current_link_width"), cur_width);
        write_fixture(dir, &format!("{base}/max_link_speed"), max_speed);
        write_fixture(dir, &format!("{base}/max_link_width"), max_width);
    }

    #[test]
    fn reads_pcie_endpoint() {
        let tmp = TempDir::new().unwrap();
        create_pci_device(tmp.path(), "0000:01:00.0", "0x020000", "0x1b4b", "0x2241");
        set_link_info(tmp.path(), "0000:01:00.0", "8.0 GT/s", "1", "8.0 GT/s", "4");

        let collector = PcieCollector::new(tmp.path());
        let mut data = PcieData::default();
        collector.collect(&mut data).unwrap();

        assert_eq!(data.devices.len(), 1);
        let dev = &data.devices[0];
        assert_eq!(dev.address, "0000:01:00.0");
        assert_eq!(dev.vendor_id, "0x1b4b");
        assert_eq!(dev.device_id, "0x2241");
        assert_eq!(dev.current_speed, "8.0 GT/s");
        assert_eq!(dev.current_width, 1);
        assert_eq!(dev.max_speed, "8.0 GT/s");
        assert_eq!(dev.max_width, 4);
        assert_eq!(dev.gen_label, "Gen 3");
        // Width downgraded: current 1 < max 4
        assert!(dev.downgraded);
    }

    #[test]
    fn skips_bridges() {
        let tmp = TempDir::new().unwrap();
        // Bridge device — should be skipped
        create_pci_device(tmp.path(), "0000:00:00.0", "0x060400", "0x14e4", "0x2712");
        set_link_info(tmp.path(), "0000:00:00.0", "5.0 GT/s", "4", "5.0 GT/s", "4");
        // Endpoint device — should be included
        create_pci_device(tmp.path(), "0000:01:00.0", "0x010802", "0x1b4b", "0x2241");
        set_link_info(tmp.path(), "0000:01:00.0", "5.0 GT/s", "1", "5.0 GT/s", "1");

        let collector = PcieCollector::new(tmp.path());
        let mut data = PcieData::default();
        collector.collect(&mut data).unwrap();

        assert_eq!(data.devices.len(), 1);
        assert_eq!(data.devices[0].address, "0000:01:00.0");
    }

    #[test]
    fn detects_speed_downgrade() {
        let tmp = TempDir::new().unwrap();
        create_pci_device(tmp.path(), "0000:01:00.0", "0x020000", "0x1234", "0x5678");
        set_link_info(tmp.path(), "0000:01:00.0", "2.5 GT/s", "1", "5.0 GT/s", "1");

        let collector = PcieCollector::new(tmp.path());
        let mut data = PcieData::default();
        collector.collect(&mut data).unwrap();

        assert_eq!(data.devices.len(), 1);
        assert_eq!(data.devices[0].gen_label, "Gen 1");
        assert!(data.devices[0].downgraded);
    }

    #[test]
    fn no_downgrade_when_matching() {
        let tmp = TempDir::new().unwrap();
        create_pci_device(tmp.path(), "0000:01:00.0", "0x020000", "0x1234", "0x5678");
        set_link_info(tmp.path(), "0000:01:00.0", "5.0 GT/s", "4", "5.0 GT/s", "4");

        let collector = PcieCollector::new(tmp.path());
        let mut data = PcieData::default();
        collector.collect(&mut data).unwrap();

        assert_eq!(data.devices.len(), 1);
        assert_eq!(data.devices[0].gen_label, "Gen 2");
        assert!(!data.devices[0].downgraded);
    }

    #[test]
    fn handles_no_pci_bus() {
        let tmp = TempDir::new().unwrap();
        let collector = PcieCollector::new(tmp.path());
        let mut data = PcieData::default();
        collector.collect(&mut data).unwrap();
        assert!(data.devices.is_empty());
    }

    #[test]
    fn handles_missing_link_files() {
        let tmp = TempDir::new().unwrap();
        // Device with class but no link info files
        create_pci_device(tmp.path(), "0000:01:00.0", "0x020000", "0x1234", "0x5678");

        let collector = PcieCollector::new(tmp.path());
        let mut data = PcieData::default();
        collector.collect(&mut data).unwrap();

        assert_eq!(data.devices.len(), 1);
        let dev = &data.devices[0];
        assert_eq!(dev.current_speed, "");
        assert_eq!(dev.current_width, 0);
        assert_eq!(dev.gen_label, "Unknown");
        assert!(!dev.downgraded);
    }

    #[test]
    fn multiple_endpoints_sorted() {
        let tmp = TempDir::new().unwrap();
        create_pci_device(tmp.path(), "0000:03:00.0", "0x020000", "0xaaaa", "0xbbbb");
        set_link_info(tmp.path(), "0000:03:00.0", "8.0 GT/s", "1", "8.0 GT/s", "1");
        create_pci_device(tmp.path(), "0000:01:00.0", "0x010802", "0x1234", "0x5678");
        set_link_info(tmp.path(), "0000:01:00.0", "5.0 GT/s", "4", "5.0 GT/s", "4");

        let collector = PcieCollector::new(tmp.path());
        let mut data = PcieData::default();
        collector.collect(&mut data).unwrap();

        assert_eq!(data.devices.len(), 2);
        // Should be sorted by address
        assert_eq!(data.devices[0].address, "0000:01:00.0");
        assert_eq!(data.devices[1].address, "0000:03:00.0");
    }

    #[test]
    fn speed_to_gen_mapping() {
        assert_eq!(speed_to_gen("2.5 GT/s"), "Gen 1");
        assert_eq!(speed_to_gen("5.0 GT/s"), "Gen 2");
        assert_eq!(speed_to_gen("8.0 GT/s"), "Gen 3");
        assert_eq!(speed_to_gen("16.0 GT/s"), "Unknown");
        assert_eq!(speed_to_gen(""), "Unknown");
    }

    #[test]
    fn parse_gts_values() {
        assert!((parse_gts("8.0 GT/s").unwrap() - 8.0).abs() < 0.01);
        assert!((parse_gts("5.0 GT/s").unwrap() - 5.0).abs() < 0.01);
        assert!((parse_gts("2.5 GT/s").unwrap() - 2.5).abs() < 0.01);
        assert!(parse_gts("").is_none());
        assert!(parse_gts("bad").is_none());
    }
}
