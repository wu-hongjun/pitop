# CLI Options

## Usage

```
pitop [OPTIONS]
```

---

## Options

### `-i`, `--interval <MS>`

Refresh interval in milliseconds. Controls how frequently pitop collects data and updates the display.

- **Type**: integer
- **Default**: 1000
- **Minimum**: 100

```sh
pitop -i 500      # Update twice per second
pitop -i 2000     # Update every 2 seconds
```

!!! tip
    Lower intervals provide smoother updates but increase CPU usage. On a Pi Zero 2W, consider using 2000ms or higher.

---

### `-t`, `--tab <N>`

Starting tab number. pitop will open directly on this tab.

- **Type**: integer (1-6)
- **Default**: 1

| Value | Tab |
|-------|-----|
| 1 | Overview |
| 2 | Processes |
| 3 | Power |
| 4 | Network |
| 5 | Disk |
| 6 | System |

```sh
pitop -t 3        # Start on the Power tab
```

---

### `-c`, `--config <PATH>`

Path to a custom config file. Overrides the default location (`~/.config/pitop/config.toml`).

```sh
pitop --config /path/to/config.toml
```

---

### `--board <TYPE>`

Force a specific board type instead of auto-detection.

- **Type**: string
- **Default**: `auto`
- **Values**: `auto`, `pi5`, `pi4b`, `zero2w`

```sh
pitop --board pi5     # Force Pi 5 mode
pitop --board auto    # Auto-detect (default)
```

!!! warning
    Forcing a board type on incompatible hardware will cause the corresponding collectors to return empty data. The app will not crash, but Pi-specific sections will show no data.

---

### `--theme <NAME>`

Color theme to use.

- **Type**: string
- **Default**: value from config file, or `"default"`
- **Values**: `default`, `monochrome`, `solarized`

```sh
pitop --theme solarized
pitop --theme monochrome
```

You can also cycle themes at runtime by pressing `t`.

---

### `-v`, `--verbose`

Enable verbose error output. When enabled, collector errors are printed to stderr. Useful for debugging why a specific feature is not displaying data.

```sh
pitop -v
```

---

### `--stress`

Start the CPU stress test on launch. Workers begin running immediately and can be toggled with `Ctrl+S` at runtime.

```sh
pitop --stress
```

---

### `--stress-workers <N>`

Number of stress test worker threads. Only meaningful when combined with `--stress`.

- **Type**: integer
- **Default**: number of CPU cores
- **Minimum**: 1

```sh
pitop --stress --stress-workers 2    # Start with 2 workers
```

---

### `--generate-config`

Print a fully commented sample config.toml to stdout and exit. Does not launch the TUI.

```sh
pitop --generate-config > ~/.config/pitop/config.toml
```

---

### `--config-check`

Load and validate the config file, print the result, and exit. Does not launch the TUI.

```sh
pitop --config-check
pitop --config /path/to/config.toml --config-check
```

Prints either "Configuration is valid." or a specific error message.

---

### `-V`, `--version`

Print the version number and exit.

```sh
pitop --version
```

---

### `-h`, `--help`

Print usage information and exit.

```sh
pitop --help
```

---

## Examples

```sh
# Default usage
pitop

# Fast refresh, solarized theme, start on Power tab
pitop -i 500 --theme solarized -t 3

# Stress test with 2 workers and verbose output
pitop --stress --stress-workers 2 -v

# Force Pi 5 mode on an unrecognized board
pitop --board pi5

# Generate and validate a config file
pitop --generate-config > ~/.config/pitop/config.toml
pitop --config-check
```

---

## Precedence

CLI arguments always override config file values:

1. **CLI arguments** (highest priority)
2. **Config file** (`~/.config/pitop/config.toml`)
3. **Built-in defaults** (lowest priority)
