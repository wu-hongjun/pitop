# Config File Reference

pitop uses a TOML configuration file located at `~/.config/pitop/config.toml`.

---

## Generating a config file

```sh
mkdir -p ~/.config/pitop
pitop --generate-config > ~/.config/pitop/config.toml
```

---

## Validating a config file

```sh
pitop --config-check
```

---

## Full annotated reference

```toml
# pitop configuration file
# Place this file at ~/.config/pitop/config.toml

#
# [general] — Application-wide settings
#
[general]

# Refresh interval in milliseconds.
# Controls how frequently data is collected and the display is updated.
# Minimum: 100
# Default: 1000
interval_ms = 1000

# Starting tab number (1-6).
# 1 = Overview
# 2 = Processes
# 3 = Power
# 4 = Network
# 5 = Disk
# 6 = System
# Default: 1
default_tab = 1

# Number of samples to keep in sparkline history.
# Higher values show more history but use more memory.
# Range: 10-600
# Default: 60
history_size = 60

# Color theme.
# Built-in options: "default", "monochrome", "solarized"
# Use "custom" to enable the [custom_theme] section below.
# Default: "default"
theme = "default"


#
# [thresholds] — Color thresholds for gauges and indicators.
# Values below "warning" are green, between "warning" and "critical"
# are yellow, and at or above "critical" are red.
#

[thresholds.cpu]
# CPU usage percentage thresholds.
# Default: warning = 60.0, critical = 85.0
warning = 60.0
critical = 85.0

[thresholds.memory]
# Memory usage percentage thresholds.
# Default: warning = 70.0, critical = 90.0
warning = 70.0
critical = 90.0

[thresholds.temperature]
# Temperature thresholds in degrees Celsius.
# Default: warning = 60.0, critical = 75.0
warning = 60.0
critical = 75.0

[thresholds.disk]
# Disk usage percentage thresholds.
# Default: warning = 70.0, critical = 90.0
warning = 70.0
critical = 90.0

[thresholds.swap]
# Swap usage percentage thresholds.
# Default: warning = 25.0, critical = 50.0
warning = 25.0
critical = 50.0


#
# [custom_theme] — Define a custom color theme.
# Requires theme = "custom" in [general].
# All fields are optional. Missing fields fall back to the default theme.
#
# Colors can be specified as:
#   - Named: "red", "green", "blue", "yellow", "cyan", "magenta",
#            "white", "gray", "darkgray"
#   - Hex:   "#RRGGBB" (e.g., "#FF8800")
#

# [custom_theme]
# border = "cyan"
# border_highlight = "#FF8800"
# title = "#FF8800"
# text = "white"
# text_dim = "gray"
# highlight_bg = "#1A1A2E"
# gauge_low = "#00FF00"
# gauge_warn = "#FFAA00"
# gauge_crit = "#FF0000"
# sparkline_cpu = "#4488FF"
# sparkline_mem = "#FF44FF"
# sparkline_temp = "#FF4444"
# sparkline_power = "#FFAA44"
# cpu_border = "#4488FF"
# mem_border = "#FF44FF"
# temp_border = "#44FF44"
# net_border = "#44FF44"
# power_border = "#FFAA44"
# throttle_ok = "#00FF00"
# throttle_warn = "#FFAA00"
# throttle_crit = "#FF0000"
```

---

## Sections

### `[general]`

| Key | Type | Default | Range | Description |
|-----|------|---------|-------|-------------|
| `interval_ms` | integer | 1000 | >= 100 | Refresh interval in milliseconds |
| `default_tab` | integer | 1 | 1-6 | Starting tab number |
| `history_size` | integer | 60 | 10-600 | Sparkline history samples |
| `theme` | string | `"default"` | `default`, `monochrome`, `solarized`, `custom` | Color theme |

### `[thresholds.*]`

Each threshold section has two float fields:

| Key | Type | Description |
|-----|------|-------------|
| `warning` | float | Value at which indicator turns yellow |
| `critical` | float | Value at which indicator turns red |

### `[custom_theme]`

See [Themes](../guide/themes.md) for the full list of color fields and examples.

---

## Validation rules

The following validation rules are enforced when loading the config:

- `interval_ms` must be >= 100
- `default_tab` must be between 1 and 6
- `history_size` must be between 10 and 600
- `theme` must be one of: `default`, `monochrome`, `solarized`, `custom`
- If `theme = "custom"`, a `[custom_theme]` section must be defined
- Unknown keys in any section cause a parse error (strict mode via `deny_unknown_fields`)

---

## Environment

The config file path follows XDG conventions. The default location is determined by:

```
$HOME/.config/pitop/config.toml
```

Use `--config /path/to/file.toml` to override the default location.
