use crate::config::{parse_color, CustomThemeConfig};
use ratatui::style::Color;
use serde::Deserialize;

/// Color theme for all UI rendering.
///
/// All UI modules should read colors from a `Theme` instance rather than
/// hardcoding `Color::*` values.  Three built-in themes are available:
/// `default`, `monochrome`, and `solarized`.
#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    // General
    pub border: Color,
    pub border_highlight: Color,
    pub title: Color,
    pub text: Color,
    pub text_dim: Color,
    pub highlight_bg: Color,

    // Gauges / thresholds
    pub gauge_low: Color,
    pub gauge_warn: Color,
    pub gauge_crit: Color,

    // Sparklines
    pub sparkline_cpu: Color,
    pub sparkline_mem: Color,
    pub sparkline_temp: Color,
    pub sparkline_power: Color,

    // Tab-specific
    pub cpu_border: Color,
    pub mem_border: Color,
    pub temp_border: Color,
    pub net_border: Color,
    pub power_border: Color,

    // Status
    pub throttle_ok: Color,
    pub throttle_warn: Color,
    pub throttle_crit: Color,
}

impl Theme {
    /// The default theme matching the currently-hardcoded colors.
    pub fn default_theme() -> Self {
        Self {
            // General
            border: Color::White,
            border_highlight: Color::Cyan,
            title: Color::Yellow,
            text: Color::White,
            text_dim: Color::DarkGray,
            highlight_bg: Color::DarkGray,

            // Gauges / thresholds
            gauge_low: Color::Green,
            gauge_warn: Color::Yellow,
            gauge_crit: Color::Red,

            // Sparklines
            sparkline_cpu: Color::Blue,
            sparkline_mem: Color::Magenta,
            sparkline_temp: Color::Red,
            sparkline_power: Color::Yellow,

            // Tab-specific
            cpu_border: Color::Blue,
            mem_border: Color::Magenta,
            temp_border: Color::Green,
            net_border: Color::Green,
            power_border: Color::Yellow,

            // Status
            throttle_ok: Color::Green,
            throttle_warn: Color::Yellow,
            throttle_crit: Color::Red,
        }
    }

    /// A monochrome theme using only white and gray shades.
    pub fn monochrome() -> Self {
        Self {
            // General
            border: Color::White,
            border_highlight: Color::White,
            title: Color::White,
            text: Color::White,
            text_dim: Color::Gray,
            highlight_bg: Color::DarkGray,

            // Gauges / thresholds
            gauge_low: Color::White,
            gauge_warn: Color::Gray,
            gauge_crit: Color::White,

            // Sparklines
            sparkline_cpu: Color::White,
            sparkline_mem: Color::Gray,
            sparkline_temp: Color::White,
            sparkline_power: Color::Gray,

            // Tab-specific
            cpu_border: Color::White,
            mem_border: Color::Gray,
            temp_border: Color::White,
            net_border: Color::Gray,
            power_border: Color::White,

            // Status
            throttle_ok: Color::White,
            throttle_warn: Color::Gray,
            throttle_crit: Color::White,
        }
    }

    /// Solarized-dark theme.
    pub fn solarized() -> Self {
        Self {
            // General
            border: Color::Rgb(131, 148, 150),          // base0
            border_highlight: Color::Rgb(42, 161, 152), // cyan
            title: Color::Rgb(181, 137, 0),             // yellow
            text: Color::Rgb(131, 148, 150),            // base0
            text_dim: Color::Rgb(88, 110, 117),         // base01
            highlight_bg: Color::Rgb(7, 54, 66),        // base02

            // Gauges / thresholds
            gauge_low: Color::Rgb(133, 153, 0),  // green
            gauge_warn: Color::Rgb(181, 137, 0), // yellow
            gauge_crit: Color::Rgb(220, 50, 47), // red

            // Sparklines
            sparkline_cpu: Color::Rgb(38, 139, 210),  // blue
            sparkline_mem: Color::Rgb(211, 54, 130),  // magenta
            sparkline_temp: Color::Rgb(220, 50, 47),  // red
            sparkline_power: Color::Rgb(181, 137, 0), // yellow

            // Tab-specific
            cpu_border: Color::Rgb(38, 139, 210),  // blue
            mem_border: Color::Rgb(211, 54, 130),  // magenta
            temp_border: Color::Rgb(133, 153, 0),  // green
            net_border: Color::Rgb(133, 153, 0),   // green
            power_border: Color::Rgb(181, 137, 0), // yellow

            // Status
            throttle_ok: Color::Rgb(133, 153, 0),   // green
            throttle_warn: Color::Rgb(181, 137, 0), // yellow
            throttle_crit: Color::Rgb(220, 50, 47), // red
        }
    }

    /// Look up a theme by name.  Returns `None` for unknown names.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "default" => Some(Self::default_theme()),
            "monochrome" => Some(Self::monochrome()),
            "solarized" => Some(Self::solarized()),
            _ => None,
        }
    }

    /// Build a Theme from a `CustomThemeConfig`, falling back to the default
    /// theme for any field that is `None` or has an unparseable color string.
    pub fn from_config(config: &CustomThemeConfig) -> Self {
        let d = Self::default_theme();

        /// Helper: resolve an optional color string, falling back to a default.
        fn resolve(opt: &Option<String>, fallback: Color) -> Color {
            opt.as_deref().and_then(parse_color).unwrap_or(fallback)
        }

        Self {
            border: resolve(&config.border, d.border),
            border_highlight: resolve(&config.border_highlight, d.border_highlight),
            title: resolve(&config.title, d.title),
            text: resolve(&config.text, d.text),
            text_dim: resolve(&config.text_dim, d.text_dim),
            highlight_bg: resolve(&config.highlight_bg, d.highlight_bg),
            gauge_low: resolve(&config.gauge_low, d.gauge_low),
            gauge_warn: resolve(&config.gauge_warn, d.gauge_warn),
            gauge_crit: resolve(&config.gauge_crit, d.gauge_crit),
            sparkline_cpu: resolve(&config.sparkline_cpu, d.sparkline_cpu),
            sparkline_mem: resolve(&config.sparkline_mem, d.sparkline_mem),
            sparkline_temp: resolve(&config.sparkline_temp, d.sparkline_temp),
            sparkline_power: resolve(&config.sparkline_power, d.sparkline_power),
            cpu_border: resolve(&config.cpu_border, d.cpu_border),
            mem_border: resolve(&config.mem_border, d.mem_border),
            temp_border: resolve(&config.temp_border, d.temp_border),
            net_border: resolve(&config.net_border, d.net_border),
            power_border: resolve(&config.power_border, d.power_border),
            throttle_ok: resolve(&config.throttle_ok, d.throttle_ok),
            throttle_warn: resolve(&config.throttle_warn, d.throttle_warn),
            throttle_crit: resolve(&config.throttle_crit, d.throttle_crit),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::default_theme()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_name_returns_known_themes() {
        assert!(Theme::from_name("default").is_some());
        assert!(Theme::from_name("monochrome").is_some());
        assert!(Theme::from_name("solarized").is_some());
    }

    #[test]
    fn from_name_returns_none_for_unknown() {
        assert!(Theme::from_name("nope").is_none());
        assert!(Theme::from_name("").is_none());
    }

    #[test]
    fn default_trait_matches_default_theme() {
        let a = Theme::default();
        let b = Theme::default_theme();
        // Compare a representative field to confirm they are the same variant.
        assert!(matches!(a.border, Color::White));
        assert!(matches!(b.border, Color::White));
    }

    #[test]
    fn from_config_hex_color_ff0000_becomes_rgb_255_0_0() {
        let mut cfg = CustomThemeConfig::default();
        cfg.title = Some("#FF0000".to_string());
        let theme = Theme::from_config(&cfg);
        assert!(matches!(theme.title, Color::Rgb(255, 0, 0)));
        // Unset fields fall back to defaults
        assert!(matches!(theme.border, Color::White));
    }

    #[test]
    fn from_config_named_colors_work() {
        let mut cfg = CustomThemeConfig::default();
        cfg.gauge_crit = Some("magenta".to_string());
        cfg.sparkline_cpu = Some("cyan".to_string());
        let theme = Theme::from_config(&cfg);
        assert!(matches!(theme.gauge_crit, Color::Magenta));
        assert!(matches!(theme.sparkline_cpu, Color::Cyan));
    }

    #[test]
    fn from_config_invalid_color_uses_default() {
        let mut cfg = CustomThemeConfig::default();
        cfg.border = Some("not_a_color".to_string());
        let theme = Theme::from_config(&cfg);
        // Should fall back to default theme's border (White)
        assert!(matches!(theme.border, Color::White));
    }
}
