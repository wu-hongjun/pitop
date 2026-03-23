use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Serialize)]
pub struct FanData {
    pub available: bool,
    pub rpm: u32,
    pub pwm_raw: u8,      // 0-255
    pub pwm_percent: f64, // 0.0 - 100.0
}

pub struct FanCollector {
    hwmon_path: Option<PathBuf>,
}

impl FanCollector {
    pub fn new(root: &Path) -> Self {
        let hwmon_path = crate::util::sysfs::discover_hwmon(root, "cooling_fan");
        Self { hwmon_path }
    }

    pub fn collect(&self, data: &mut FanData) -> Result<()> {
        let hwmon_path = match &self.hwmon_path {
            Some(p) => p,
            None => {
                *data = FanData::default();
                return Ok(());
            }
        };

        let rpm = std::fs::read_to_string(hwmon_path.join("fan1_input"))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok());

        let pwm_raw = std::fs::read_to_string(hwmon_path.join("pwm1"))
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok());

        match (rpm, pwm_raw) {
            (Some(r), Some(p)) => {
                data.available = true;
                data.rpm = r;
                data.pwm_raw = p;
                data.pwm_percent = f64::from(p) / 255.0 * 100.0;
            }
            _ => {
                // At least one file is unreadable — fan not available
                *data = FanData::default();
            }
        }

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

    fn setup_fan_hwmon(tmp: &TempDir, rpm: &str, pwm: &str) {
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon3/name", "cooling_fan\n");
        write_fixture(
            tmp.path(),
            "sys/class/hwmon/hwmon3/fan1_input",
            &format!("{rpm}\n"),
        );
        write_fixture(
            tmp.path(),
            "sys/class/hwmon/hwmon3/pwm1",
            &format!("{pwm}\n"),
        );
    }

    #[test]
    fn reads_fan_data() {
        let tmp = TempDir::new().unwrap();
        setup_fan_hwmon(&tmp, "3500", "128");

        let collector = FanCollector::new(tmp.path());
        let mut data = FanData::default();
        collector.collect(&mut data).unwrap();

        assert!(data.available);
        assert_eq!(data.rpm, 3500);
        assert_eq!(data.pwm_raw, 128);
        assert!((data.pwm_percent - 50.196).abs() < 0.1);
    }

    #[test]
    fn full_speed() {
        let tmp = TempDir::new().unwrap();
        setup_fan_hwmon(&tmp, "6000", "255");

        let collector = FanCollector::new(tmp.path());
        let mut data = FanData::default();
        collector.collect(&mut data).unwrap();

        assert!(data.available);
        assert_eq!(data.rpm, 6000);
        assert_eq!(data.pwm_raw, 255);
        assert!((data.pwm_percent - 100.0).abs() < 0.01);
    }

    #[test]
    fn fan_stopped() {
        let tmp = TempDir::new().unwrap();
        setup_fan_hwmon(&tmp, "0", "0");

        let collector = FanCollector::new(tmp.path());
        let mut data = FanData::default();
        collector.collect(&mut data).unwrap();

        assert!(data.available);
        assert_eq!(data.rpm, 0);
        assert_eq!(data.pwm_raw, 0);
        assert!((data.pwm_percent - 0.0).abs() < 0.01);
    }

    #[test]
    fn no_hwmon_device() {
        let tmp = TempDir::new().unwrap();
        // No cooling_fan hwmon at all
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon0/name", "cpu_thermal\n");

        let collector = FanCollector::new(tmp.path());
        let mut data = FanData::default();
        collector.collect(&mut data).unwrap();

        assert!(!data.available);
        assert_eq!(data.rpm, 0);
    }

    #[test]
    fn no_hwmon_directory() {
        let tmp = TempDir::new().unwrap();
        // No sys/class/hwmon directory at all

        let collector = FanCollector::new(tmp.path());
        let mut data = FanData::default();
        collector.collect(&mut data).unwrap();

        assert!(!data.available);
    }

    #[test]
    fn missing_fan1_input() {
        let tmp = TempDir::new().unwrap();
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon3/name", "cooling_fan\n");
        // pwm1 exists but fan1_input does not
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon3/pwm1", "128\n");

        let collector = FanCollector::new(tmp.path());
        let mut data = FanData::default();
        collector.collect(&mut data).unwrap();

        assert!(!data.available);
    }

    #[test]
    fn missing_pwm1() {
        let tmp = TempDir::new().unwrap();
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon3/name", "cooling_fan\n");
        // fan1_input exists but pwm1 does not
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon3/fan1_input", "3500\n");

        let collector = FanCollector::new(tmp.path());
        let mut data = FanData::default();
        collector.collect(&mut data).unwrap();

        assert!(!data.available);
    }

    #[test]
    fn discovers_among_multiple_hwmon() {
        let tmp = TempDir::new().unwrap();
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon0/name", "cpu_thermal\n");
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon1/name", "rp1_adc\n");
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon5/name", "cooling_fan\n");
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon5/fan1_input", "2200\n");
        write_fixture(tmp.path(), "sys/class/hwmon/hwmon5/pwm1", "64\n");

        let collector = FanCollector::new(tmp.path());
        let mut data = FanData::default();
        collector.collect(&mut data).unwrap();

        assert!(data.available);
        assert_eq!(data.rpm, 2200);
        assert_eq!(data.pwm_raw, 64);
        assert!((data.pwm_percent - 25.098).abs() < 0.1);
    }
}
