# Changelog

## v0.2.0

### Bug Fixes
- Added MJPG to codec query list (was missing on Zero 2W where MJPG=enabled)

### Verified
- Full 33-point hardware audit passed on Pi 5 16GB (33/33)
- Full 33-point hardware audit passed on Zero 2W (33/33 after MJPG fix)

---

## v0.1.12

### UI
- Redesigned Overview tab to match mactop layout: 4 gauge blocks (CPU, GPU/Load, Temperature, Memory), 4 info panels (Power, Board, Network, Disk), and an embedded process list (no sparklines)
- Process list keyboard shortcuts (`j`/`k`/`s`/`K`) now work on the Overview tab in addition to the Processes tab
- Disk info panel strips `/dev/` prefix and shows device name + mountpoint on separate lines
- GPU section now shows V3D clock frequency (960 MHz) instead of core clock on Pi 5
- GPU memory reported as "Shared" on Pi 5 (which uses shared system memory)
- GPU codec display shows "Hardware HEVC (BCM2712)" instead of disabled codec marks on Pi 5
- Automatic update check on startup with install command shown in footer

### Hardware Detection
- Zero 2W detection fixed: now matches `bcm2837` in addition to `bcm2710`
- CPU model detection on ARM64: maps CPU part numbers to names (0xd03=Cortex-A53, 0xd0b=Cortex-A76, etc.)
- All known Pi models now mapped: Pi 5, Pi 4B, Pi 400, CM4, Pi 3B+, Pi 3B, Pi 3A+, Pi 2B, Pi 1, Zero 2W, Zero W, Zero
- NVMe temperature shown from hwmon sensor discovery

### Bug Fixes
- Fan detection fixed: now matches `pwmfan` hwmon name in addition to `cooling_fan`
- PMIC power parser fixed for Pi 5 format with `RAIL_A current(N)=VALUE` style output
- PCIe downgrade detection: only speed drops are flagged as downgrades, not width mismatches (Pi 5 M.2 HAT is x1 by design)
- PoE detection: infers `online=true` when device exists with `type=Mains`

### New Features
- Power and voltage data now collected on the Overview tab (not just the Power tab)
- Automatic update checker runs at startup and displays available update in footer with install command

---

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
