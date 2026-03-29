use serde::Serialize;

#[derive(Debug, Default, Clone, Serialize)]
pub struct GpuData {
    pub available: bool,
    pub frequency_mhz: u64,            // GPU core clock in MHz
    pub memory_mb: u64,                // GPU memory allocation in MB
    pub temperature_celsius: f64,      // GPU temp
    pub codecs: Vec<(String, bool)>,   // (codec_name, enabled)
    pub video_decoder: Option<String>, // Hardware decoder description (Pi 5: BCM2712 HEVC)
    pub shared_memory: bool,           // True when GPU uses shared system memory (Pi 5)
}

/// Parse `vcgencmd measure_clock core` output.
/// Format: "frequency(NN)=XXXXXXXXX"
pub fn parse_clock_core(output: &str) -> Option<u64> {
    let freq_str = output.split('=').nth(1)?.trim();
    let hz: u64 = freq_str.parse().ok()?;
    Some(hz / 1_000_000) // Convert Hz to MHz
}

/// Parse `vcgencmd get_mem gpu` output.
/// Format: "gpu=128M"
pub fn parse_get_mem_gpu(output: &str) -> Option<u64> {
    let val = output.split('=').nth(1)?.trim();
    let mb_str = val.strip_suffix('M').or_else(|| val.strip_suffix('m'))?;
    mb_str.parse().ok()
}

/// Parse `vcgencmd measure_temp` output.
/// Format: "temp=52.3'C"
pub fn parse_measure_temp(output: &str) -> Option<f64> {
    let val = output.split('=').nth(1)?.trim();
    let temp_str = val.split('\'').next()?;
    temp_str.parse().ok()
}

/// Parse `vcgencmd codec_enabled <codec>` output.
/// Format: "H264=enabled" or "H264=disabled"
/// Validates the returned codec name matches the expected one (case-insensitive).
pub fn parse_codec_enabled(codec_name: &str, output: &str) -> Option<(String, bool)> {
    let mut parts = output.splitn(2, '=');
    let key = parts.next()?.trim();
    let val = parts.next()?.trim();

    // Verify the response is for the codec we asked about
    if !key.eq_ignore_ascii_case(codec_name) {
        return None;
    }

    match val {
        "enabled" => Some((codec_name.to_string(), true)),
        "disabled" => Some((codec_name.to_string(), false)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_clock_core_valid() {
        assert_eq!(parse_clock_core("frequency(46)=500000000"), Some(500));
    }

    #[test]
    fn parse_clock_core_high_freq() {
        assert_eq!(parse_clock_core("frequency(46)=1000000000"), Some(1000));
    }

    #[test]
    fn parse_clock_core_zero() {
        assert_eq!(parse_clock_core("frequency(46)=0"), Some(0));
    }

    #[test]
    fn parse_clock_core_missing_equals() {
        assert_eq!(parse_clock_core("frequency(46)500000000"), None);
    }

    #[test]
    fn parse_clock_core_empty() {
        assert_eq!(parse_clock_core(""), None);
    }

    #[test]
    fn parse_clock_core_non_numeric() {
        assert_eq!(parse_clock_core("frequency(46)=abc"), None);
    }

    #[test]
    fn parse_get_mem_gpu_valid() {
        assert_eq!(parse_get_mem_gpu("gpu=128M"), Some(128));
    }

    #[test]
    fn parse_get_mem_gpu_lowercase() {
        assert_eq!(parse_get_mem_gpu("gpu=64m"), Some(64));
    }

    #[test]
    fn parse_get_mem_gpu_256() {
        assert_eq!(parse_get_mem_gpu("gpu=256M"), Some(256));
    }

    #[test]
    fn parse_get_mem_gpu_no_suffix() {
        assert_eq!(parse_get_mem_gpu("gpu=128"), None);
    }

    #[test]
    fn parse_get_mem_gpu_empty() {
        assert_eq!(parse_get_mem_gpu(""), None);
    }

    #[test]
    fn parse_get_mem_gpu_non_numeric() {
        assert_eq!(parse_get_mem_gpu("gpu=abcM"), None);
    }

    #[test]
    fn parse_measure_temp_valid() {
        let result = parse_measure_temp("temp=52.3'C");
        assert!((result.unwrap_or(0.0) - 52.3).abs() < 0.001);
    }

    #[test]
    fn parse_measure_temp_integer() {
        let result = parse_measure_temp("temp=45.0'C");
        assert!((result.unwrap_or(0.0) - 45.0).abs() < 0.001);
    }

    #[test]
    fn parse_measure_temp_high() {
        let result = parse_measure_temp("temp=85.5'C");
        assert!((result.unwrap_or(0.0) - 85.5).abs() < 0.001);
    }

    #[test]
    fn parse_measure_temp_empty() {
        assert_eq!(parse_measure_temp(""), None);
    }

    #[test]
    fn parse_measure_temp_missing_quote() {
        // If the quote is missing, split('\'') returns the whole string
        // which should still parse as a float
        let result = parse_measure_temp("temp=52.3C");
        // "52.3C" won't parse as f64, so None
        assert_eq!(result, None);
    }

    #[test]
    fn parse_measure_temp_non_numeric() {
        assert_eq!(parse_measure_temp("temp=abc'C"), None);
    }

    #[test]
    fn gpu_data_default() {
        let data = GpuData::default();
        assert!(!data.available);
        assert_eq!(data.frequency_mhz, 0);
        assert_eq!(data.memory_mb, 0);
        assert!((data.temperature_celsius - 0.0).abs() < f64::EPSILON);
        assert!(data.codecs.is_empty());
        assert!(data.video_decoder.is_none());
        assert!(!data.shared_memory);
    }

    #[test]
    fn parse_codec_enabled_h264_enabled() {
        let result = parse_codec_enabled("H264", "H264=enabled");
        assert_eq!(result, Some(("H264".to_string(), true)));
    }

    #[test]
    fn parse_codec_enabled_h264_disabled() {
        let result = parse_codec_enabled("H264", "H264=disabled");
        assert_eq!(result, Some(("H264".to_string(), false)));
    }

    #[test]
    fn parse_codec_enabled_hevc_enabled() {
        let result = parse_codec_enabled("HEVC", "HEVC=enabled");
        assert_eq!(result, Some(("HEVC".to_string(), true)));
    }

    #[test]
    fn parse_codec_enabled_hevc_disabled() {
        let result = parse_codec_enabled("HEVC", "HEVC=disabled");
        assert_eq!(result, Some(("HEVC".to_string(), false)));
    }

    #[test]
    fn parse_codec_enabled_malformed_no_equals() {
        assert_eq!(parse_codec_enabled("H264", "H264enabled"), None);
    }

    #[test]
    fn parse_codec_enabled_malformed_empty() {
        assert_eq!(parse_codec_enabled("H264", ""), None);
    }

    #[test]
    fn parse_codec_enabled_malformed_unknown_value() {
        assert_eq!(parse_codec_enabled("H264", "H264=maybe"), None);
    }
}
