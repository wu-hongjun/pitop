# Changelog

All notable changes to pitop will be documented in this file.

## [0.1.0] - 2026-03-25

### Added

#### Core
- Board detection via `/proc/device-tree/compatible` for Pi 5, Pi 4B, and Zero 2W
- Board capability profiles controlling which collectors are activated
- Direct sysfs/procfs parsing for all system metrics (no `sysinfo` crate)
- Async vcgencmd subprocess wrapper with 2-second timeout and result caching
- Ring buffer (60-sample window) for sparkline history
- Human-readable formatting utilities for bytes, temperatures, watts, frequencies, and durations
- sysfs/procfs reading helpers with graceful error handling

#### Hardware
- CPU usage collector parsing `/proc/stat` with per-core and aggregate percentages
- Memory collector parsing `/proc/meminfo` for RAM and swap
- Thermal collector with thermal zone enumeration and hwmon discovery
- Network collector parsing `/proc/net/dev` with per-interface throughput rates
- Disk collector with partition usage via `statvfs` and I/O rates from `/proc/diskstats`
- Process collector scanning `/proc/[pid]/` with per-process CPU and memory stats
- Throttle status collector decoding `vcgencmd get_throttled` bitmask flags
- PMIC power rail monitoring via `vcgencmd pmic_read_adc` (Pi 5)
- Fan speed monitoring via hwmon discovery for `cooling_fan` (Pi 5)
- PCIe link detection and generation reporting (Pi 5)
- PoE HAT detection and current draw monitoring (Pi 5, Pi 4B)
- Pi 4B voltage readings via `vcgencmd measure_volts`
- GPU frequency, memory, temperature, and codec status monitoring

#### UI
- ratatui + crossterm TUI with configurable tick interval
- Six-tab interface: Overview, Processes, Power, Network, Disk, System
- Tab navigation via number keys 1-6, Tab/Shift+Tab cycling
- Overview dashboard with per-core CPU gauges, memory gauge, sparklines, and temperature
- Color-coded gauges (green/yellow/red thresholds for CPU, memory, temperature, disk)
- Processes tab with sortable table (CPU%, MEM%, PID, Name), vim-style navigation, and process kill
- Power tab with PMIC rails, voltage table, PCIe devices, and PoE status
- Network tab with per-interface details, throughput sparklines, and connection count
- Disk tab with partition usage gauges and per-disk I/O rates
- System tab with board info, kernel version, OS details, and hardware summary
- Help overlay toggled with `?` key, with sectioned keyboard shortcut reference
- Scrollable help content with j/k navigation for small terminals
- Three built-in color themes: default, monochrome, solarized
- Runtime theme cycling with `t` key
- Space key to pause/resume data collection

#### Configuration
- TOML configuration file support (`~/.config/pitop/config.toml`)
- CLI arguments override config file values (CLI > config > defaults)
- `--generate-config` flag to print a fully-commented sample config
- `--config-check` flag for config validation
- Field range validation with clear error messages
- Custom theme support via `[theme.custom]` config section

#### Developer Experience
- CI workflow with `cargo fmt`, `cargo clippy`, and `cargo test` on every push
- Release workflow building aarch64 and armv7 binaries on git tag
- Cross-compilation via `cross` for ARM targets
- Clippy enforcement: zero warnings, no `unwrap()` or `expect()` allowed
- Release profile optimized for binary size (LTO, strip, `opt-level = "z"`)
- One-line install script for downloading prebuilt binaries
- Stress testing mode (`--stress`) with adjustable worker count
