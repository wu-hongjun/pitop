use serde::Serialize;
use std::collections::HashMap;

/// A single PMIC power rail with voltage, current, and computed power.
#[derive(Debug, Default, Clone, Serialize)]
pub struct PowerRail {
    pub name: String,
    pub voltage: f64,
    pub current: f64,
    pub power: f64,
}

/// Aggregated PMIC data from `vcgencmd pmic_read_adc` (Pi 5 only).
#[derive(Debug, Default, Clone, Serialize)]
pub struct PmicData {
    pub rails: Vec<PowerRail>,
    pub total_pmic_watts: f64,
    pub estimated_real_watts: f64,
    pub ext5v_voltage: f64,
    pub ext5v_current: f64,
}

/// A single voltage reading from `vcgencmd measure_volts` (Pi 4B / Zero 2W).
#[derive(Debug, Default, Clone, Serialize)]
pub struct VoltageReading {
    pub name: String,
    pub voltage: f64,
}

/// Combined power data for all board types.
#[derive(Debug, Default, Clone, Serialize)]
pub struct PowerData {
    pub pmic: Option<PmicData>,
    pub voltages: Vec<VoltageReading>,
}

/// Parse the multi-line output of `vcgencmd pmic_read_adc`.
///
/// Each line is in the format `RAIL_V=X.XXXXV` or `RAIL_A=X.XXXXA`.
/// Rails are grouped by their name prefix (stripping `_V` / `_A` suffix),
/// and power is computed as voltage * current for each rail.
///
/// The estimated real power consumption is:
///   total_pmic_watts * 1.1451 + 0.5879
pub fn parse_pmic_read_adc(output: &str) -> PmicData {
    // Collect voltage and current values keyed by rail name prefix.
    let mut voltages: HashMap<String, f64> = HashMap::new();
    let mut currents: HashMap<String, f64> = HashMap::new();
    // Track insertion order so the output is deterministic.
    let mut rail_names_ordered: Vec<String> = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Split on '=' to get key and value parts.
        let Some((key, raw_value)) = line.split_once('=') else {
            continue;
        };

        // The key may be "RAIL_V" (simple) or "RAIL_A current(N)" (Pi 5 format).
        // Extract just the rail identifier (first whitespace-delimited token).
        let rail_id = key.trim().split_whitespace().next().unwrap_or("");

        // Determine whether this is a voltage (_V suffix) or current (_A suffix) reading.
        if let Some(rail_name) = rail_id.strip_suffix("_V") {
            let v = parse_numeric_with_unit(raw_value);
            if !voltages.contains_key(rail_name) && !currents.contains_key(rail_name) {
                rail_names_ordered.push(rail_name.to_string());
            }
            voltages.insert(rail_name.to_string(), v);
        } else if let Some(rail_name) = rail_id.strip_suffix("_A") {
            let a = parse_numeric_with_unit(raw_value);
            if !voltages.contains_key(rail_name) && !currents.contains_key(rail_name) {
                rail_names_ordered.push(rail_name.to_string());
            }
            currents.insert(rail_name.to_string(), a);
        }
    }

    // Build PowerRail entries in insertion order.
    let mut rails = Vec::new();
    let mut total_pmic_watts = 0.0;
    let mut ext5v_voltage = 0.0;
    let mut ext5v_current = 0.0;

    for name in &rail_names_ordered {
        let voltage = voltages.get(name).copied().unwrap_or(0.0);
        let current = currents.get(name).copied().unwrap_or(0.0);
        let power = voltage * current;

        if name == "EXT5V" {
            ext5v_voltage = voltage;
            ext5v_current = current;
            // EXT5V is the raw input rail — don't include in PMIC total
            // to avoid double-counting (per design-research.md)
        } else {
            total_pmic_watts += power;
        }

        rails.push(PowerRail {
            name: name.clone(),
            voltage,
            current,
            power,
        });
    }

    let estimated_real_watts = total_pmic_watts * 1.1451 + 0.5879;

    PmicData {
        rails,
        total_pmic_watts,
        estimated_real_watts,
        ext5v_voltage,
        ext5v_current,
    }
}

/// Parse the output of `vcgencmd measure_volts <rail>`.
///
/// Expected format: `volt=X.XXXXV`
/// Returns `None` if parsing fails.
pub fn parse_measure_volts(rail_name: &str, output: &str) -> Option<VoltageReading> {
    let trimmed = output.trim();
    let value_str = trimmed.strip_prefix("volt=")?;
    let voltage = parse_numeric_with_unit_opt(value_str)?;

    Some(VoltageReading {
        name: rail_name.to_string(),
        voltage,
    })
}

/// Strip a trailing unit letter (V, A, W, etc.) and parse the remaining string
/// as an f64. Returns 0.0 on failure.
fn parse_numeric_with_unit(s: &str) -> f64 {
    parse_numeric_with_unit_opt(s).unwrap_or(0.0)
}

/// Strip a trailing unit letter and parse as f64. Returns `None` on failure.
fn parse_numeric_with_unit_opt(s: &str) -> Option<f64> {
    let s = s.trim();
    // Remove a single trailing alphabetic character (the unit).
    let numeric = if s.ends_with(|c: char| c.is_ascii_alphabetic()) {
        &s[..s.len() - 1]
    } else {
        s
    };
    numeric.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PMIC_OUTPUT: &str = "\
EXT5V_V=5.0588V
EXT5V_A=0.4294A
3V7_WL_SW_A=0.0000A
3V3_SYS_A=0.0190A
1V8_SYS_A=0.2014A
DDR_VDD2_A=0.0550A
DDR_VDDQ_A=0.0144A
1V1_SYS_A=0.1440A
0V8_SW_A=0.5960A
VDD_CORE_A=0.3330A
3V3_DAC_A=0.0020A
3V3_ADC_A=0.0020A
0V8_AON_A=0.0060A
USB_VDD5V_A=0.0000A
HDMI_A=0.0000A
";

    #[test]
    fn parse_pmic_ext5v_voltage_and_current() {
        let data = parse_pmic_read_adc(PMIC_OUTPUT);
        assert!((data.ext5v_voltage - 5.0588).abs() < 1e-4);
        assert!((data.ext5v_current - 0.4294).abs() < 1e-4);
    }

    #[test]
    fn parse_pmic_rail_count() {
        let data = parse_pmic_read_adc(PMIC_OUTPUT);
        // EXT5V has both V and A; the rest are current-only rails.
        // Unique rail names: EXT5V, 3V7_WL_SW, 3V3_SYS, 1V8_SYS, DDR_VDD2,
        // DDR_VDDQ, 1V1_SYS, 0V8_SW, VDD_CORE, 3V3_DAC, 3V3_ADC, 0V8_AON,
        // USB_VDD5V, HDMI = 14 rails
        assert_eq!(data.rails.len(), 14);
    }

    #[test]
    fn parse_pmic_ext5v_rail_power() {
        let data = parse_pmic_read_adc(PMIC_OUTPUT);
        let ext5v = data
            .rails
            .iter()
            .find(|r| r.name == "EXT5V")
            .expect("EXT5V rail should exist");
        let expected_power = 5.0588 * 0.4294;
        assert!(
            (ext5v.power - expected_power).abs() < 1e-4,
            "EXT5V power: expected {}, got {}",
            expected_power,
            ext5v.power
        );
    }

    #[test]
    fn parse_pmic_current_only_rail_has_zero_voltage() {
        let data = parse_pmic_read_adc(PMIC_OUTPUT);
        let vdd_core = data
            .rails
            .iter()
            .find(|r| r.name == "VDD_CORE")
            .expect("VDD_CORE rail should exist");
        assert_eq!(vdd_core.voltage, 0.0);
        assert!((vdd_core.current - 0.3330).abs() < 1e-4);
        // power = 0 * 0.333 = 0
        assert_eq!(vdd_core.power, 0.0);
    }

    #[test]
    fn parse_pmic_total_watts() {
        let data = parse_pmic_read_adc(PMIC_OUTPUT);
        // EXT5V is excluded from total_pmic_watts (it's the raw input rail).
        // In the fixture, only EXT5V has non-zero voltage, so total is 0.
        assert!(
            data.total_pmic_watts.abs() < 1e-4,
            "total: expected ~0, got {}",
            data.total_pmic_watts
        );
        // But ext5v fields should still be populated
        assert!((data.ext5v_voltage - 5.0588).abs() < 1e-4);
        assert!((data.ext5v_current - 0.4294).abs() < 1e-4);
    }

    #[test]
    fn parse_pmic_estimated_real_watts() {
        let data = parse_pmic_read_adc(PMIC_OUTPUT);
        let expected = data.total_pmic_watts * 1.1451 + 0.5879;
        assert!(
            (data.estimated_real_watts - expected).abs() < 1e-6,
            "estimated real watts: expected {}, got {}",
            expected,
            data.estimated_real_watts
        );
    }

    #[test]
    fn parse_pmic_with_all_voltage_and_current_pairs() {
        // Test a case where multiple rails have both V and A values.
        let output = "\
RAIL1_V=3.3000V
RAIL1_A=0.1000A
RAIL2_V=1.8000V
RAIL2_A=0.2000A
";
        let data = parse_pmic_read_adc(output);
        assert_eq!(data.rails.len(), 2);

        let r1 = &data.rails[0];
        assert_eq!(r1.name, "RAIL1");
        assert!((r1.voltage - 3.3).abs() < 1e-4);
        assert!((r1.current - 0.1).abs() < 1e-4);
        assert!((r1.power - 0.33).abs() < 1e-4);

        let r2 = &data.rails[1];
        assert_eq!(r2.name, "RAIL2");
        assert!((r2.voltage - 1.8).abs() < 1e-4);
        assert!((r2.current - 0.2).abs() < 1e-4);
        assert!((r2.power - 0.36).abs() < 1e-4);

        let expected_total = 0.33 + 0.36;
        assert!((data.total_pmic_watts - expected_total).abs() < 1e-4);
    }

    #[test]
    fn parse_pmic_empty_input() {
        let data = parse_pmic_read_adc("");
        assert!(data.rails.is_empty());
        assert_eq!(data.total_pmic_watts, 0.0);
        assert!((data.estimated_real_watts - 0.5879).abs() < 1e-6);
        assert_eq!(data.ext5v_voltage, 0.0);
        assert_eq!(data.ext5v_current, 0.0);
    }

    #[test]
    fn parse_pmic_garbage_input() {
        let data = parse_pmic_read_adc("this is not valid output\nneither is this");
        assert!(data.rails.is_empty());
    }

    #[test]
    fn parse_pmic_rail_ordering_preserved() {
        let output = "B_V=1.0V\nA_V=2.0V\nC_V=3.0V\n";
        let data = parse_pmic_read_adc(output);
        assert_eq!(data.rails[0].name, "B");
        assert_eq!(data.rails[1].name, "A");
        assert_eq!(data.rails[2].name, "C");
    }

    // --- measure_volts tests ---

    #[test]
    fn parse_measure_volts_core() {
        let reading = parse_measure_volts("core", "volt=0.8688V");
        let reading = reading.expect("should parse successfully");
        assert_eq!(reading.name, "core");
        assert!((reading.voltage - 0.8688).abs() < 1e-4);
    }

    #[test]
    fn parse_measure_volts_sdram() {
        let reading = parse_measure_volts("sdram_c", "volt=1.1000V\n");
        let reading = reading.expect("should parse successfully");
        assert_eq!(reading.name, "sdram_c");
        assert!((reading.voltage - 1.1).abs() < 1e-4);
    }

    #[test]
    fn parse_measure_volts_invalid_format() {
        assert!(parse_measure_volts("core", "garbage").is_none());
    }

    #[test]
    fn parse_measure_volts_empty() {
        assert!(parse_measure_volts("core", "").is_none());
    }

    #[test]
    fn parse_measure_volts_missing_value() {
        assert!(parse_measure_volts("core", "volt=").is_none());
    }

    #[test]
    fn parse_measure_volts_no_unit_suffix() {
        // The format should have a V suffix, but be lenient with bare numbers.
        let reading = parse_measure_volts("core", "volt=0.8688");
        let reading = reading.expect("should parse bare number");
        assert!((reading.voltage - 0.8688).abs() < 1e-4);
    }

    // --- helper function tests ---

    #[test]
    fn parse_numeric_strips_unit() {
        assert!((parse_numeric_with_unit("5.0588V") - 5.0588).abs() < 1e-4);
        assert!((parse_numeric_with_unit("0.4294A") - 0.4294).abs() < 1e-4);
        assert!((parse_numeric_with_unit("1.2345W") - 1.2345).abs() < 1e-4);
    }

    #[test]
    fn parse_numeric_no_unit() {
        assert!((parse_numeric_with_unit("3.14") - 3.14).abs() < 1e-4);
    }

    #[test]
    fn parse_numeric_invalid() {
        assert_eq!(parse_numeric_with_unit("not_a_number"), 0.0);
        assert_eq!(parse_numeric_with_unit(""), 0.0);
    }

    #[test]
    fn power_data_defaults() {
        let pd = PowerData::default();
        assert!(pd.pmic.is_none());
        assert!(pd.voltages.is_empty());
    }
}
