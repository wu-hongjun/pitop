# Configuration

pitop supports a TOML configuration file for persisting settings across sessions.

## Config file location

pitop looks for its config file at:

```
~/.config/pitop/config.toml
```

This follows the [XDG Base Directory](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html) convention. If no config file exists, pitop uses built-in defaults.

---

## Generating a config file

Use the `--generate-config` flag to print a fully commented sample config to stdout:

```sh
pitop --generate-config > ~/.config/pitop/config.toml
```

!!! tip
    Create the directory first if it does not exist:

    ```sh
    mkdir -p ~/.config/pitop
    pitop --generate-config > ~/.config/pitop/config.toml
    ```

---

## Validating your config

Check your config file for errors without launching the TUI:

```sh
pitop --config-check
```

This loads, parses, and validates the config, then prints either "Configuration is valid." or a specific error message.

To check a config file at a custom path:

```sh
pitop --config /path/to/config.toml --config-check
```

---

## Config options

### `[general]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `interval_ms` | integer | `1000` | Refresh interval in milliseconds (minimum 100) |
| `default_tab` | integer | `1` | Starting tab number (1-6) |
| `history_size` | integer | `60` | Number of samples in sparkline history (10-600) |
| `theme` | string | `"default"` | Color theme: `"default"`, `"monochrome"`, `"solarized"`, or `"custom"` |

### `[thresholds.cpu]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `warning` | float | `60.0` | CPU usage % at which gauges turn yellow |
| `critical` | float | `85.0` | CPU usage % at which gauges turn red |

### `[thresholds.memory]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `warning` | float | `70.0` | Memory usage % at which gauges turn yellow |
| `critical` | float | `90.0` | Memory usage % at which gauges turn red |

### `[thresholds.temperature]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `warning` | float | `60.0` | Temperature (Celsius) at which display turns yellow |
| `critical` | float | `75.0` | Temperature (Celsius) at which display turns red |

### `[thresholds.disk]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `warning` | float | `70.0` | Disk usage % at which display turns yellow |
| `critical` | float | `90.0` | Disk usage % at which display turns red |

### `[thresholds.swap]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `warning` | float | `25.0` | Swap usage % at which display turns yellow |
| `critical` | float | `50.0` | Swap usage % at which display turns red |

### `[custom_theme]`

Define a custom color theme. All fields are optional -- missing fields fall back to the default theme's colors. See [Themes](../guide/themes.md) for full details.

---

## CLI override behavior

Command-line arguments always take precedence over config file values:

```sh
# Config says interval_ms = 1000, but CLI wins
pitop -i 500

# Config says theme = "default", but CLI wins
pitop --theme solarized

# Config says default_tab = 1, but CLI wins
pitop -t 3
```

The precedence order is:

1. CLI arguments (highest)
2. Config file values
3. Built-in defaults (lowest)

---

## Example config file

```toml
# pitop configuration file
# Place at ~/.config/pitop/config.toml

[general]
# Refresh interval in milliseconds (minimum 100)
interval_ms = 1000

# Starting tab number (1-6)
# 1=Overview, 2=Processes, 3=Power, 4=Network, 5=Disk, 6=System
default_tab = 1

# Number of samples to keep in sparkline history (10-600)
history_size = 60

# Color theme: "default", "monochrome", "solarized", or "custom"
theme = "default"

[thresholds.cpu]
warning = 60.0
critical = 85.0

[thresholds.memory]
warning = 70.0
critical = 90.0

[thresholds.temperature]
warning = 60.0
critical = 75.0

[thresholds.disk]
warning = 70.0
critical = 90.0

[thresholds.swap]
warning = 25.0
critical = 50.0
```

---

## Custom config file path

Use the `--config` flag to specify a different config file:

```sh
pitop --config /path/to/my-config.toml
```

This is useful for testing different configurations or running multiple instances with different settings.
