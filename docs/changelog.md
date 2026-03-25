# Changelog

## v0.1.0

Initial release of pitop.

### Board Detection (Epic 1)

- Auto-detect Raspberry Pi model from `/proc/device-tree/compatible`
- Support for Pi 5 (BCM2712), Pi 4B (BCM2711), and Zero 2W (BCM2710)
- Fallback to generic Linux mode on unrecognized boards
- `--board` CLI flag to force a specific board type

### Core Collectors (Epic 2)

- **CPU** -- per-core usage from `/proc/stat`, frequency from cpufreq sysfs
- **Memory** -- RAM and swap usage from `/proc/meminfo`
- **Thermal** -- SoC temperature from thermal zones, hwmon discovery by name
- **Network** -- per-interface throughput from `/proc/net/dev`
- **Disk** -- partition usage and I/O throughput from `/proc/diskstats`
- **Process** -- process table from `/proc/[pid]/` scanning
- **Throttle** -- throttle state from `vcgencmd get_throttled`
- sysfs reading utilities with graceful error handling

### TUI Framework (Epic 3)

- ratatui + crossterm terminal application with alternate screen
- Tab bar with 6 tabs: Overview, Processes, Power, Network, Disk, System
- Overview dashboard with CPU gauges, memory bars, temperature, network throughput
- Sparkline history graphs (60-sample ring buffers)
- Keyboard navigation: number keys, Tab/Shift+Tab, vim bindings
- Pause/resume with Space bar
- Panic hook for terminal restoration

### Additional Tabs (Epic 4)

- **Processes tab** -- sortable table (PID, Name, CPU%, MEM, User) with kill support
- **Network tab** -- interface details with IPv4/IPv6, MAC, throughput sparklines
- **Disk tab** -- partition usage gauges, I/O throughput
- **System tab** -- board model, kernel, architecture, uptime, capabilities

### Pi-Specific Features (Epic 5)

- **PMIC power rails** (Pi 5) -- 12-rail voltage/current/power table via `vcgencmd pmic_read_adc`
- **Estimated wattage** with correction factor and sparkline
- **Fan monitoring** (Pi 5) -- RPM and duty cycle from hwmon
- **PCIe link status** (Pi 5) -- generation, lane width, downgrade detection
- **PoE HAT detection** (Pi 5, Pi 4B) -- power supply status
- **Voltage readings** (Pi 4B, Zero 2W) -- core and SDRAM voltages via `vcgencmd measure_volts`

### Polish and Release (Epic 6)

- Zero `unwrap()`/`expect()` in production code (enforced by clippy)
- CLI argument parsing via clap with derive macros
- `--interval`, `--tab`, `--board`, `--verbose`, `--version`, `--help` flags
- Graceful error handling throughout all collectors
- Terminal restoration on panic

### Advanced Features (Epic 7)

- **Color themes** -- default, monochrome, solarized built-in themes
- **Custom themes** -- user-defined colors via `[custom_theme]` config section
- **Configuration file** -- TOML-based at `~/.config/pitop/config.toml`
- **Config validation** -- `--config-check` and `--generate-config` commands
- **Help overlay** -- `?` key shows scrollable keyboard shortcut reference
- **GPU monitoring** -- frequency, memory, temperature, codec status via vcgencmd
- **Stress testing** -- built-in CPU load generator with `--stress` flag

### Feature Polish (Epic 8)

- Theme-aware UI -- all modules read colors from active theme
- Runtime theme cycling with `t` key
- Stress test runtime controls: `Ctrl+S` toggle, `Ctrl+Up/Down` worker adjustment
- Footer bar with context-sensitive hints and stress test status
- Custom theme support with hex color parsing
