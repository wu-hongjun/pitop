mod pi4b;
mod pi5;
mod unknown;
mod zero2w;

pub use pi4b::Pi4BProfile;
pub use pi5::Pi5Profile;
pub use unknown::UnknownProfile;
pub use zero2w::Zero2WProfile;

use anyhow::Result;
use serde::Serialize;
use std::path::Path;

/// Detected board type based on `/proc/device-tree/compatible`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BoardType {
    Pi5,
    Pi4B,
    Zero2W,
    Unknown,
}

/// Voltage data source available on this board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum VoltageSource {
    /// Pi 5: full PMIC rail data via `vcgencmd pmic_read_adc`
    Pmic,
    /// Pi 4B / Zero 2W: basic voltages via `vcgencmd measure_volts`
    MeasureVolts,
    /// No voltage data available
    None,
}

/// Capability flags that determine which collectors and UI sections are active.
pub trait BoardProfile: Send + Sync + std::fmt::Debug {
    fn board_type(&self) -> BoardType;
    fn name(&self) -> &str;
    fn soc_name(&self) -> &str;
    fn has_pmic(&self) -> bool;
    fn has_fan(&self) -> bool;
    fn has_pcie(&self) -> bool;
    fn has_poe(&self) -> bool;
    fn thermal_zones(&self) -> &[&str];
    fn voltage_source(&self) -> VoltageSource;
}

/// Detect the board type by reading the device-tree compatible string.
///
/// Reads `{root}/proc/device-tree/compatible` which contains null-separated
/// strings like `raspberrypi,5-model-b\0brcm,bcm2712\0`.
///
/// The `root` parameter enables fixture-based testing on x86.
pub fn detect(root: &Path) -> BoardType {
    let compatible_path = root.join("proc/device-tree/compatible");
    let content = match std::fs::read(&compatible_path) {
        Ok(bytes) => bytes,
        Err(_) => return BoardType::Unknown,
    };

    // Parse null-separated strings
    let entries: Vec<&str> = content
        .split(|&b| b == 0)
        .filter_map(|s| std::str::from_utf8(s).ok())
        .filter(|s| !s.is_empty())
        .collect();

    for entry in &entries {
        if entry.contains("bcm2712") {
            return BoardType::Pi5;
        }
        if entry.contains("bcm2711") {
            return BoardType::Pi4B;
        }
        if entry.contains("bcm2710") {
            return BoardType::Zero2W;
        }
    }

    BoardType::Unknown
}

/// Create the appropriate board profile for the detected (or overridden) board type.
pub fn create_profile(board_type: BoardType) -> Box<dyn BoardProfile> {
    match board_type {
        BoardType::Pi5 => Box::new(Pi5Profile),
        BoardType::Pi4B => Box::new(Pi4BProfile),
        BoardType::Zero2W => Box::new(Zero2WProfile),
        BoardType::Unknown => Box::new(UnknownProfile),
    }
}

/// Read the human-readable board model string.
///
/// Falls back to "Unknown" if the file doesn't exist (e.g., on x86).
pub fn read_model_name(root: &Path) -> String {
    let model_path = root.join("sys/firmware/devicetree/base/model");
    std::fs::read_to_string(&model_path)
        .map(|s| s.trim_end_matches('\0').trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string())
}

/// Static system information gathered once at startup.
#[derive(Debug, Default, Clone, Serialize)]
pub struct SystemInfo {
    pub board_type: String,
    pub model_name: String,
    pub kernel_version: String,
    pub hostname: String,
    pub architecture: String,
    pub os_name: String,
    pub os_version: String,
    pub cpu_model: String,
}

/// Collect static system information from procfs/sysfs.
///
/// All reads gracefully handle missing files — this runs on any Linux.
pub fn collect_system_info(root: &Path) -> SystemInfo {
    SystemInfo {
        board_type: String::new(), // Set by caller from BoardProfile
        model_name: read_model_name(root),
        kernel_version: read_trimmed(root, "proc/version"),
        hostname: read_trimmed(root, "proc/sys/kernel/hostname"),
        architecture: read_uname_machine(),
        os_name: read_os_release_field(root, "NAME"),
        os_version: read_os_release_field(root, "VERSION_ID"),
        cpu_model: read_cpu_model(root),
    }
}

/// Read a file, trim whitespace, return empty string on failure.
fn read_trimmed(root: &Path, relative: &str) -> String {
    std::fs::read_to_string(root.join(relative))
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// Get machine architecture from the uname syscall (e.g., "aarch64", "armv7l").
fn read_uname_machine() -> String {
    #[cfg(unix)]
    {
        unsafe {
            let mut info: libc::utsname = std::mem::zeroed();
            if libc::uname(&mut info) == 0 {
                let machine = std::ffi::CStr::from_ptr(info.machine.as_ptr());
                return machine.to_string_lossy().into_owned();
            }
        }
    }
    String::new()
}

/// Read CPU model name from `/proc/cpuinfo`.
///
/// On ARM, looks for "model name" first, then "Hardware".
fn read_cpu_model(root: &Path) -> String {
    let cpuinfo_path = root.join("proc/cpuinfo");
    let content = match std::fs::read_to_string(&cpuinfo_path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    for line in content.lines() {
        if line.starts_with("model name") {
            if let Some(value) = line.split(':').nth(1) {
                return value.trim().to_string();
            }
        }
    }

    // Fall back to "Hardware" (shows on 32-bit ARM)
    for line in content.lines() {
        if line.starts_with("Hardware") {
            if let Some(value) = line.split(':').nth(1) {
                return value.trim().to_string();
            }
        }
    }

    String::new()
}

/// Read a specific field from `/etc/os-release`.
fn read_os_release_field(root: &Path, field: &str) -> String {
    let path = root.join("etc/os-release");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    let prefix = format!("{}=", field);
    for line in content.lines() {
        if line.starts_with(&prefix) {
            let value = line[prefix.len()..].trim();
            // Strip surrounding quotes if present
            return value.trim_matches('"').to_string();
        }
    }

    String::new()
}

/// Parse a board type from a CLI override string.
pub fn parse_board_override(s: &str) -> Result<BoardType> {
    match s.to_lowercase().as_str() {
        "pi5" => Ok(BoardType::Pi5),
        "pi4b" | "pi4" => Ok(BoardType::Pi4B),
        "zero2w" | "zero2" => Ok(BoardType::Zero2W),
        "auto" => Ok(BoardType::Unknown), // Will be re-detected
        _ => anyhow::bail!("Unknown board type: '{}'. Use: pi5, pi4b, zero2w, auto", s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_fixture(dir: &Path, relative: &str, content: &[u8]) {
        let path = dir.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, content).unwrap();
    }

    #[test]
    fn detect_pi5() {
        let tmp = TempDir::new().unwrap();
        let content = b"raspberrypi,5-model-b\0brcm,bcm2712\0";
        create_fixture(tmp.path(), "proc/device-tree/compatible", content);

        assert_eq!(detect(tmp.path()), BoardType::Pi5);
    }

    #[test]
    fn detect_pi4b() {
        let tmp = TempDir::new().unwrap();
        let content = b"raspberrypi,4-model-b\0brcm,bcm2711\0";
        create_fixture(tmp.path(), "proc/device-tree/compatible", content);

        assert_eq!(detect(tmp.path()), BoardType::Pi4B);
    }

    #[test]
    fn detect_zero2w() {
        let tmp = TempDir::new().unwrap();
        let content = b"raspberrypi,model-zero-2-w\0brcm,bcm2710\0";
        create_fixture(tmp.path(), "proc/device-tree/compatible", content);

        assert_eq!(detect(tmp.path()), BoardType::Zero2W);
    }

    #[test]
    fn detect_unknown_when_no_file() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(detect(tmp.path()), BoardType::Unknown);
    }

    #[test]
    fn detect_unknown_when_no_match() {
        let tmp = TempDir::new().unwrap();
        let content = b"some-other-board\0brcm,bcm9999\0";
        create_fixture(tmp.path(), "proc/device-tree/compatible", content);

        assert_eq!(detect(tmp.path()), BoardType::Unknown);
    }

    #[test]
    fn read_model_name_found() {
        let tmp = TempDir::new().unwrap();
        create_fixture(
            tmp.path(),
            "sys/firmware/devicetree/base/model",
            b"Raspberry Pi 5 Model B Rev 1.0\0",
        );

        assert_eq!(
            read_model_name(tmp.path()),
            "Raspberry Pi 5 Model B Rev 1.0"
        );
    }

    #[test]
    fn read_model_name_missing() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(read_model_name(tmp.path()), "Unknown");
    }

    #[test]
    fn system_info_collects_available_data() {
        let tmp = TempDir::new().unwrap();
        create_fixture(
            tmp.path(),
            "sys/firmware/devicetree/base/model",
            b"Raspberry Pi 4 Model B Rev 1.4\0",
        );
        create_fixture(
            tmp.path(),
            "proc/version",
            b"Linux version 6.1.0-rpi7-rpi-v8 (debian-kernel@lists.debian.org)\n",
        );
        create_fixture(tmp.path(), "proc/sys/kernel/hostname", b"pihole\n");
        create_fixture(
            tmp.path(),
            "etc/os-release",
            b"NAME=\"Raspberry Pi OS\"\nVERSION_ID=\"12\"\n",
        );

        let info = collect_system_info(tmp.path());
        assert_eq!(info.model_name, "Raspberry Pi 4 Model B Rev 1.4");
        assert!(info.kernel_version.contains("Linux version 6.1.0"));
        assert_eq!(info.hostname, "pihole");
        assert_eq!(info.os_name, "Raspberry Pi OS");
        assert_eq!(info.os_version, "12");
    }

    #[test]
    fn system_info_handles_missing_files() {
        let tmp = TempDir::new().unwrap();
        let info = collect_system_info(tmp.path());
        assert_eq!(info.model_name, "Unknown");
        assert!(info.kernel_version.is_empty());
        assert!(info.hostname.is_empty());
    }

    #[test]
    fn parse_board_override_valid() {
        assert_eq!(parse_board_override("pi5").unwrap(), BoardType::Pi5);
        assert_eq!(parse_board_override("PI4B").unwrap(), BoardType::Pi4B);
        assert_eq!(parse_board_override("zero2w").unwrap(), BoardType::Zero2W);
        assert_eq!(parse_board_override("auto").unwrap(), BoardType::Unknown);
    }

    #[test]
    fn parse_board_override_invalid() {
        assert!(parse_board_override("pi3").is_err());
    }

    #[test]
    fn profile_capabilities() {
        let pi5 = create_profile(BoardType::Pi5);
        assert!(pi5.has_pmic());
        assert!(pi5.has_fan());
        assert!(pi5.has_pcie());
        assert!(pi5.has_poe());
        assert_eq!(pi5.voltage_source(), VoltageSource::Pmic);

        let pi4b = create_profile(BoardType::Pi4B);
        assert!(!pi4b.has_pmic());
        assert!(!pi4b.has_fan());
        assert!(!pi4b.has_pcie());
        assert!(pi4b.has_poe());
        assert_eq!(pi4b.voltage_source(), VoltageSource::MeasureVolts);

        let zero2w = create_profile(BoardType::Zero2W);
        assert!(!zero2w.has_pmic());
        assert!(!zero2w.has_fan());
        assert!(!zero2w.has_pcie());
        assert!(!zero2w.has_poe());
        assert_eq!(zero2w.voltage_source(), VoltageSource::MeasureVolts);

        let unknown = create_profile(BoardType::Unknown);
        assert!(!unknown.has_pmic());
        assert!(!unknown.has_fan());
        assert!(!unknown.has_pcie());
        assert!(!unknown.has_poe());
        assert_eq!(unknown.voltage_source(), VoltageSource::None);
    }
}
