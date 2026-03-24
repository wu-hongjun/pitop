// Policy: No unwrap() or expect() in production code.

use anyhow::{bail, Result};
use ratatui::style::Color;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub general: GeneralConfig,
    pub thresholds: ThresholdConfig,
    pub custom_theme: Option<CustomThemeConfig>,
}

/// Custom theme configuration — all fields are optional.
/// Missing fields fall back to the default theme's colors.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CustomThemeConfig {
    pub border: Option<String>,
    pub border_highlight: Option<String>,
    pub title: Option<String>,
    pub text: Option<String>,
    pub text_dim: Option<String>,
    pub highlight_bg: Option<String>,
    pub gauge_low: Option<String>,
    pub gauge_warn: Option<String>,
    pub gauge_crit: Option<String>,
    pub sparkline_cpu: Option<String>,
    pub sparkline_mem: Option<String>,
    pub sparkline_temp: Option<String>,
    pub sparkline_power: Option<String>,
    pub cpu_border: Option<String>,
    pub mem_border: Option<String>,
    pub temp_border: Option<String>,
    pub net_border: Option<String>,
    pub power_border: Option<String>,
    pub throttle_ok: Option<String>,
    pub throttle_warn: Option<String>,
    pub throttle_crit: Option<String>,
}

/// Parse a color string into a ratatui `Color`.
///
/// Supports named colors (`"red"`, `"green"`, etc.) and hex `"#RRGGBB"` format.
/// Returns `None` for unrecognized strings.
pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color::Rgb(r, g, b));
        }
        return None;
    }
    match s.to_lowercase().as_str() {
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "blue" => Some(Color::Blue),
        "yellow" => Some(Color::Yellow),
        "cyan" => Some(Color::Cyan),
        "magenta" => Some(Color::Magenta),
        "white" => Some(Color::White),
        "gray" => Some(Color::Gray),
        "darkgray" => Some(Color::DarkGray),
        _ => None,
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GeneralConfig {
    pub interval_ms: u64,
    pub default_tab: u8,
    pub history_size: usize,
    pub theme: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ThresholdConfig {
    pub cpu: Threshold,
    pub memory: Threshold,
    pub temperature: Threshold,
    pub disk: Threshold,
    pub swap: Threshold,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Threshold {
    pub warning: f64,
    pub critical: f64,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            interval_ms: 1000,
            default_tab: 1,
            history_size: 60,
            theme: String::from("default"),
        }
    }
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            cpu: Threshold {
                warning: 60.0,
                critical: 85.0,
            },
            memory: Threshold {
                warning: 70.0,
                critical: 90.0,
            },
            temperature: Threshold {
                warning: 60.0,
                critical: 75.0,
            },
            disk: Threshold {
                warning: 70.0,
                critical: 90.0,
            },
            swap: Threshold {
                warning: 25.0,
                critical: 50.0,
            },
        }
    }
}

impl Default for Threshold {
    fn default() -> Self {
        Self {
            warning: 0.0,
            critical: 0.0,
        }
    }
}

impl Config {
    /// Load config from the given path, or the default XDG path.
    /// Returns default config if no file is found. Does NOT validate.
    pub fn load_raw(path: Option<&Path>) -> Result<Self> {
        let config_path = path.map(PathBuf::from).or_else(default_config_path);

        match config_path {
            Some(p) if p.exists() => {
                let content = std::fs::read_to_string(&p)?;
                let config: Config = toml::from_str(&content)?;
                Ok(config)
            }
            _ => Ok(Config::default()),
        }
    }

    /// Load config and validate it. Returns error on invalid config.
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let config = Self::load_raw(path)?;
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration values are within acceptable ranges.
    pub fn validate(&self) -> Result<()> {
        if self.general.interval_ms < 100 {
            bail!(
                "interval_ms must be >= 100, got {}",
                self.general.interval_ms
            );
        }
        if self.general.default_tab < 1 || self.general.default_tab > 6 {
            bail!(
                "default_tab must be between 1 and 6, got {}",
                self.general.default_tab
            );
        }
        if self.general.history_size < 10 || self.general.history_size > 600 {
            bail!(
                "history_size must be between 10 and 600, got {}",
                self.general.history_size
            );
        }
        // Validate theme name
        let valid_themes = ["default", "monochrome", "solarized", "custom"];
        if !valid_themes.contains(&self.general.theme.as_str()) {
            bail!(
                "theme must be one of default/monochrome/solarized/custom, got '{}'",
                self.general.theme
            );
        }
        if self.general.theme == "custom" && self.custom_theme.is_none() {
            bail!("theme is set to 'custom' but no [custom_theme] section is defined");
        }
        Ok(())
    }
}

/// Generate a fully-commented sample config.toml string.
pub fn generate_sample() -> String {
    r#"# pitop configuration file
# Place this file at ~/.config/pitop/config.toml

[general]
# Refresh interval in milliseconds (minimum 100)
interval_ms = 1000

# Starting tab number (1-6)
# 1=Overview, 2=Processes, 3=Power, 4=Network, 5=Disk, 6=System
default_tab = 1

# Number of samples to keep in sparkline history (10-600)
history_size = 60

# Color theme: "default", "monochrome", or "solarized"
theme = "default"

[thresholds.cpu]
# CPU usage percentage thresholds
warning = 60.0
critical = 85.0

[thresholds.memory]
# Memory usage percentage thresholds
warning = 70.0
critical = 90.0

[thresholds.temperature]
# Temperature thresholds in degrees Celsius
warning = 60.0
critical = 75.0

[thresholds.disk]
# Disk usage percentage thresholds
warning = 70.0
critical = 90.0

[thresholds.swap]
# Swap usage percentage thresholds
warning = 25.0
critical = 50.0
"#
    .to_string()
}

fn default_config_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".config/pitop/config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_matches_default_toml_values() {
        let config = Config::default();
        assert_eq!(config.general.interval_ms, 1000);
        assert_eq!(config.general.default_tab, 1);
        assert_eq!(config.general.history_size, 60);
        assert_eq!(config.general.theme, "default");

        assert!((config.thresholds.cpu.warning - 60.0).abs() < f64::EPSILON);
        assert!((config.thresholds.cpu.critical - 85.0).abs() < f64::EPSILON);
        assert!((config.thresholds.memory.warning - 70.0).abs() < f64::EPSILON);
        assert!((config.thresholds.memory.critical - 90.0).abs() < f64::EPSILON);
        assert!((config.thresholds.temperature.warning - 60.0).abs() < f64::EPSILON);
        assert!((config.thresholds.temperature.critical - 75.0).abs() < f64::EPSILON);
        assert!((config.thresholds.disk.warning - 70.0).abs() < f64::EPSILON);
        assert!((config.thresholds.disk.critical - 90.0).abs() < f64::EPSILON);
        assert!((config.thresholds.swap.warning - 25.0).abs() < f64::EPSILON);
        assert!((config.thresholds.swap.critical - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_load_from_toml_string() {
        let toml_str = r#"
[general]
interval_ms = 500
default_tab = 3
history_size = 120
theme = "dark"

[thresholds.cpu]
warning = 50
critical = 80

[thresholds.memory]
warning = 60
critical = 85
"#;
        let config: Config = toml::from_str(toml_str).unwrap_or_default();
        assert_eq!(config.general.interval_ms, 500);
        assert_eq!(config.general.default_tab, 3);
        assert_eq!(config.general.history_size, 120);
        assert_eq!(config.general.theme, "dark");
        assert!((config.thresholds.cpu.warning - 50.0).abs() < f64::EPSILON);
        assert!((config.thresholds.cpu.critical - 80.0).abs() < f64::EPSILON);
        assert!((config.thresholds.memory.warning - 60.0).abs() < f64::EPSILON);
        assert!((config.thresholds.memory.critical - 85.0).abs() < f64::EPSILON);
        // Unspecified thresholds should use defaults
        assert!((config.thresholds.temperature.warning - 60.0).abs() < f64::EPSILON);
        assert!((config.thresholds.disk.critical - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_partial_toml_uses_defaults_for_missing_fields() {
        let toml_str = r#"
[general]
interval_ms = 2000
"#;
        let config: Config = toml::from_str(toml_str).unwrap_or_default();
        assert_eq!(config.general.interval_ms, 2000);
        // Missing fields fall back to defaults
        assert_eq!(config.general.default_tab, 1);
        assert_eq!(config.general.history_size, 60);
        assert_eq!(config.general.theme, "default");
        assert!((config.thresholds.cpu.warning - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_empty_toml_gives_defaults() {
        let config: Config = toml::from_str("").unwrap_or_default();
        assert_eq!(config.general.interval_ms, 1000);
        assert_eq!(config.general.default_tab, 1);
    }

    #[test]
    fn test_load_missing_file_returns_default() {
        let result = Config::load(Some(Path::new("/nonexistent/path/config.toml")));
        assert!(result.is_ok());
        let config = result.unwrap_or_default();
        assert_eq!(config.general.interval_ms, 1000);
    }

    #[test]
    fn test_load_from_actual_file() {
        let dir = tempfile::tempdir().unwrap_or_else(|_| {
            // Fallback: skip the test if we can't create a temp dir
            panic!("test requires tempfile support");
        });
        let config_path = dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            r#"
[general]
interval_ms = 750
default_tab = 4

[thresholds.cpu]
warning = 55
critical = 90
"#,
        )
        .unwrap_or_default();

        let result = Config::load(Some(&config_path));
        assert!(result.is_ok());
        let config = result.unwrap_or_default();
        assert_eq!(config.general.interval_ms, 750);
        assert_eq!(config.general.default_tab, 4);
        assert!((config.thresholds.cpu.warning - 55.0).abs() < f64::EPSILON);
        assert!((config.thresholds.cpu.critical - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_load_none_path_with_no_home_config() {
        // When passing None and no config file exists at ~/, should return defaults
        // We can't easily control $HOME in tests, but we can verify the function doesn't panic
        let result = Config::load(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_valid_config_passes() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_interval_ms_too_low() {
        let mut config = Config::default();
        config.general.interval_ms = 50;
        let err = config.validate().unwrap_err();
        assert!(
            err.to_string()
                .contains("interval_ms must be >= 100, got 50"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_validate_default_tab_zero() {
        let mut config = Config::default();
        config.general.default_tab = 0;
        let err = config.validate().unwrap_err();
        assert!(
            err.to_string()
                .contains("default_tab must be between 1 and 6, got 0"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_validate_history_size_too_small() {
        let mut config = Config::default();
        config.general.history_size = 5;
        let err = config.validate().unwrap_err();
        assert!(
            err.to_string()
                .contains("history_size must be between 10 and 600, got 5"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_generate_sample_contains_general_section() {
        let sample = super::generate_sample();
        assert!(!sample.is_empty());
        assert!(
            sample.contains("[general]"),
            "sample config should contain [general] section"
        );
    }

    #[test]
    fn test_parse_toml_with_custom_theme() {
        let toml_str = r##"
[custom_theme]
border = "cyan"
title = "#FF8800"
gauge_crit = "red"
sparkline_cpu = "#0000FF"
"##;
        let config: Config = toml::from_str(toml_str).unwrap_or_default();
        let ct = config.custom_theme;
        assert!(ct.is_some());
        let ct = ct.unwrap_or_default();
        assert_eq!(ct.border.as_deref(), Some("cyan"));
        assert_eq!(ct.title.as_deref(), Some("#FF8800"));
        assert_eq!(ct.gauge_crit.as_deref(), Some("red"));
        assert_eq!(ct.sparkline_cpu.as_deref(), Some("#0000FF"));
        // Unspecified fields should be None
        assert!(ct.text.is_none());
        assert!(ct.highlight_bg.is_none());
    }

    #[test]
    fn test_parse_color_named() {
        assert_eq!(parse_color("red"), Some(Color::Red));
        assert_eq!(parse_color("Green"), Some(Color::Green));
        assert_eq!(parse_color("BLUE"), Some(Color::Blue));
        assert_eq!(parse_color("yellow"), Some(Color::Yellow));
        assert_eq!(parse_color("cyan"), Some(Color::Cyan));
        assert_eq!(parse_color("magenta"), Some(Color::Magenta));
        assert_eq!(parse_color("white"), Some(Color::White));
        assert_eq!(parse_color("gray"), Some(Color::Gray));
        assert_eq!(parse_color("darkgray"), Some(Color::DarkGray));
    }

    #[test]
    fn test_parse_color_hex() {
        assert_eq!(parse_color("#FF0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_color("#00FF00"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(parse_color("#0000FF"), Some(Color::Rgb(0, 0, 255)));
        assert_eq!(parse_color("#ABCDEF"), Some(Color::Rgb(0xAB, 0xCD, 0xEF)));
    }

    #[test]
    fn test_parse_color_invalid() {
        assert_eq!(parse_color("unknown"), None);
        assert_eq!(parse_color("#GG0000"), None);
        assert_eq!(parse_color("#FF00"), None);
        assert_eq!(parse_color(""), None);
    }

    #[test]
    fn test_config_without_custom_theme_has_none() {
        let config = Config::default();
        assert!(config.custom_theme.is_none());
    }
}
