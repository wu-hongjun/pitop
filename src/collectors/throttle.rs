use serde::Serialize;

#[derive(Debug, Default, Clone, Serialize)]
pub struct ThrottleData {
    /// Whether throttle data was successfully read from vcgencmd
    pub available: bool,
    // Current state
    pub is_under_voltage: bool,
    pub is_freq_capped: bool,
    pub is_throttled: bool,
    pub is_soft_temp_limit: bool,
    // Since boot
    pub was_under_voltage: bool,
    pub was_freq_capped: bool,
    pub was_throttled: bool,
    pub was_soft_temp_limit: bool,
    /// Raw hex value from vcgencmd
    pub raw_value: u32,
}

impl ThrottleData {
    /// Parse the output of `vcgencmd get_throttled`.
    ///
    /// Expected format: `throttled=0x50005` or just `0x50005`.
    pub fn from_vcgencmd_output(output: &str) -> Self {
        let hex_str = output
            .trim()
            .strip_prefix("throttled=")
            .unwrap_or(output.trim());

        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let value = u32::from_str_radix(hex_str, 16).unwrap_or(0);

        Self::from_bitmask(value)
    }

    /// Decode the throttle bitmask into named flags.
    pub fn from_bitmask(value: u32) -> Self {
        Self {
            available: true,
            is_under_voltage: value & (1 << 0) != 0,
            is_freq_capped: value & (1 << 1) != 0,
            is_throttled: value & (1 << 2) != 0,
            is_soft_temp_limit: value & (1 << 3) != 0,
            was_under_voltage: value & (1 << 16) != 0,
            was_freq_capped: value & (1 << 17) != 0,
            was_throttled: value & (1 << 18) != 0,
            was_soft_temp_limit: value & (1 << 19) != 0,
            raw_value: value,
        }
    }

    /// Returns true if any current throttling condition is active.
    pub fn is_any_active(&self) -> bool {
        self.is_under_voltage || self.is_freq_capped || self.is_throttled || self.is_soft_temp_limit
    }

    /// Returns true if any throttling has occurred since boot.
    pub fn has_any_occurred(&self) -> bool {
        self.was_under_voltage
            || self.was_freq_capped
            || self.was_throttled
            || self.was_soft_temp_limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_throttling() {
        let data = ThrottleData::from_vcgencmd_output("throttled=0x0");
        assert!(!data.is_any_active());
        assert!(!data.has_any_occurred());
        assert_eq!(data.raw_value, 0);
    }

    #[test]
    fn under_voltage_current() {
        let data = ThrottleData::from_bitmask(0x1);
        assert!(data.is_under_voltage);
        assert!(!data.is_freq_capped);
        assert!(!data.is_throttled);
        assert!(data.is_any_active());
    }

    #[test]
    fn all_current_flags() {
        let data = ThrottleData::from_bitmask(0xF);
        assert!(data.is_under_voltage);
        assert!(data.is_freq_capped);
        assert!(data.is_throttled);
        assert!(data.is_soft_temp_limit);
    }

    #[test]
    fn since_boot_flags() {
        let data = ThrottleData::from_bitmask(0x50000);
        assert!(!data.is_any_active());
        assert!(data.was_under_voltage);
        assert!(!data.was_freq_capped);
        assert!(data.was_throttled);
        assert!(data.has_any_occurred());
    }

    #[test]
    fn mixed_current_and_boot() {
        // Under-voltage now + freq capped since boot
        let data = ThrottleData::from_vcgencmd_output("throttled=0x50005");
        assert!(data.is_under_voltage);
        assert!(!data.is_freq_capped);
        assert!(data.is_throttled);
        assert!(data.was_under_voltage);
        assert!(data.was_throttled);
        assert_eq!(data.raw_value, 0x50005);
    }

    #[test]
    fn handles_no_prefix() {
        let data = ThrottleData::from_vcgencmd_output("0x50005");
        assert_eq!(data.raw_value, 0x50005);
    }

    #[test]
    fn handles_invalid_input() {
        let data = ThrottleData::from_vcgencmd_output("garbage");
        assert_eq!(data.raw_value, 0);
        assert!(!data.is_any_active());
    }
}
