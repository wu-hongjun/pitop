/// Format bytes into human-readable string (e.g., "1.5 GiB").
pub fn format_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;

    if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format bytes per second into human-readable throughput.
pub fn format_bytes_per_sec(bps: f64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;

    if bps >= GIB {
        format!("{:.1} GiB/s", bps / GIB)
    } else if bps >= MIB {
        format!("{:.1} MiB/s", bps / MIB)
    } else if bps >= KIB {
        format!("{:.1} KiB/s", bps / KIB)
    } else {
        format!("{:.0} B/s", bps)
    }
}

/// Format temperature in Celsius with one decimal place.
pub fn format_temp(celsius: f64) -> String {
    format!("{:.1}°C", celsius)
}

/// Format watts with two decimal places.
pub fn format_watts(watts: f64) -> String {
    format!("{:.2} W", watts)
}

/// Format a duration in seconds into human-readable uptime.
pub fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let mins = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

/// Format frequency in kHz to a human-readable string.
pub fn format_freq_mhz(khz: u64) -> String {
    if khz >= 1_000_000 {
        format!("{:.2} GHz", khz as f64 / 1_000_000.0)
    } else {
        format!("{} MHz", khz / 1000)
    }
}

/// Format a percentage with one decimal place.
pub fn format_percent(percent: f64) -> String {
    format!("{:.1}%", percent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_formatting() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1_048_576), "1.0 MiB");
        assert_eq!(format_bytes(1_073_741_824), "1.0 GiB");
        assert_eq!(format_bytes(4_294_967_296), "4.0 GiB");
    }

    #[test]
    fn temp_formatting() {
        assert_eq!(format_temp(52.3), "52.3°C");
        assert_eq!(format_temp(0.0), "0.0°C");
    }

    #[test]
    fn duration_formatting() {
        assert_eq!(format_duration(90), "1m");
        assert_eq!(format_duration(3661), "1h 1m");
        assert_eq!(format_duration(90061), "1d 1h 1m");
    }

    #[test]
    fn freq_formatting() {
        assert_eq!(format_freq_mhz(600_000), "600 MHz");
        assert_eq!(format_freq_mhz(1_800_000), "1.80 GHz");
        assert_eq!(format_freq_mhz(2_400_000), "2.40 GHz");
    }
}
