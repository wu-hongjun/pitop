use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Serialize)]
pub struct ThermalReading {
    pub zone_name: String,
    pub temp_celsius: f64,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct ThermalData {
    pub zones: Vec<ThermalReading>,
    pub soc_temp_celsius: f64,
    pub nvme_temp_celsius: Option<f64>,
}

pub struct ThermalCollector {
    root: PathBuf,
}

impl ThermalCollector {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    pub fn collect(&self, data: &mut ThermalData) -> Result<()> {
        data.zones.clear();
        data.soc_temp_celsius = 0.0;
        data.nvme_temp_celsius = None;

        let thermal_dir = self.root.join("sys/class/thermal");
        let entries = match std::fs::read_dir(&thermal_dir) {
            Ok(e) => e,
            Err(_) => {
                // No thermal zones; still try NVMe hwmon below
                self.collect_nvme_temp(data);
                return Ok(());
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if !name.starts_with("thermal_zone") {
                continue;
            }

            let zone_type = std::fs::read_to_string(path.join("type"))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| name.clone());

            let temp_millideg = std::fs::read_to_string(path.join("temp"))
                .ok()
                .and_then(|s| s.trim().parse::<i64>().ok())
                .unwrap_or(0);

            let temp_celsius = temp_millideg as f64 / 1000.0;

            // First zone or "cpu-thermal" is the SoC temp
            if data.zones.is_empty() || zone_type.contains("cpu") {
                data.soc_temp_celsius = temp_celsius;
            }

            data.zones.push(ThermalReading {
                zone_name: zone_type,
                temp_celsius,
            });
        }

        // Scan hwmon for NVMe temperature
        self.collect_nvme_temp(data);

        Ok(())
    }

    /// Scan hwmon devices for an NVMe drive and read its temperature.
    fn collect_nvme_temp(&self, data: &mut ThermalData) {
        if let Some(hwmon_path) = crate::util::sysfs::discover_hwmon(&self.root, "nvme") {
            let temp_millideg = std::fs::read_to_string(hwmon_path.join("temp1_input"))
                .ok()
                .and_then(|s| s.trim().parse::<i64>().ok());
            if let Some(millideg) = temp_millideg {
                data.nvme_temp_celsius = Some(millideg as f64 / 1000.0);
            }
        }
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

    #[test]
    fn reads_thermal_zones() {
        let tmp = TempDir::new().unwrap();
        write_fixture(
            tmp.path(),
            "sys/class/thermal/thermal_zone0/type",
            "cpu-thermal\n",
        );
        write_fixture(
            tmp.path(),
            "sys/class/thermal/thermal_zone0/temp",
            "52300\n",
        );

        let collector = ThermalCollector::new(tmp.path());
        let mut data = ThermalData::default();
        collector.collect(&mut data).unwrap();

        assert_eq!(data.zones.len(), 1);
        assert!((data.soc_temp_celsius - 52.3).abs() < 0.01);
        assert_eq!(data.zones[0].zone_name, "cpu-thermal");
    }

    #[test]
    fn handles_no_thermal_dir() {
        let tmp = TempDir::new().unwrap();
        let collector = ThermalCollector::new(tmp.path());
        let mut data = ThermalData::default();
        collector.collect(&mut data).unwrap();
        assert!(data.zones.is_empty());
        assert!(data.nvme_temp_celsius.is_none());
    }

    #[test]
    fn reads_nvme_temp_from_hwmon() {
        let tmp = TempDir::new().unwrap();
        write_fixture(
            tmp.path(),
            "sys/class/thermal/thermal_zone0/type",
            "cpu-thermal\n",
        );
        write_fixture(
            tmp.path(),
            "sys/class/thermal/thermal_zone0/temp",
            "52300\n",
        );
        // NVMe hwmon device
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon1/name", "nvme\n");
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon1/temp1_input", "57850\n");

        let collector = ThermalCollector::new(tmp.path());
        let mut data = ThermalData::default();
        collector.collect(&mut data).unwrap();

        assert!(data.nvme_temp_celsius.is_some());
        assert!((data.nvme_temp_celsius.unwrap_or(0.0) - 57.85).abs() < 0.01);
    }

    #[test]
    fn nvme_temp_none_when_no_hwmon() {
        let tmp = TempDir::new().unwrap();
        write_fixture(
            tmp.path(),
            "sys/class/thermal/thermal_zone0/type",
            "cpu-thermal\n",
        );
        write_fixture(
            tmp.path(),
            "sys/class/thermal/thermal_zone0/temp",
            "52300\n",
        );
        // No nvme hwmon device
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon0/name", "cpu_thermal\n");

        let collector = ThermalCollector::new(tmp.path());
        let mut data = ThermalData::default();
        collector.collect(&mut data).unwrap();

        assert!(data.nvme_temp_celsius.is_none());
    }
}
