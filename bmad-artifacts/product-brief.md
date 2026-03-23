# Product Brief: pitop

## Vision

A lightweight, fast, terminal-based system monitor purpose-built for Raspberry Pi hardware. The "mactop for Raspberry Pi" — a single binary with zero runtime dependencies that surfaces board-specific telemetry that generic Linux monitors miss.

## Target users

- Pi homelab operators monitoring headless servers over SSH
- IoT developers checking thermals and power during development
- Hobbyists who want a prettier alternative to `htop` + `vcgencmd` scripts
- Anyone running Pi 5 with NVMe who wants to verify PCIe Gen 2 vs Gen 3

## Supported hardware (v1)

- Raspberry Pi 5 (BCM2712) — full feature set including PMIC, fan, PCIe, PoE
- Raspberry Pi 4 Model B (BCM2711) — core monitoring + voltage readings + PoE
- Raspberry Pi Zero 2 W (BCM2710A1) — core monitoring, optimized for low resources

Detection: read `/proc/device-tree/compatible` for SoC string (`brcm,bcm2712`, `brcm,bcm2711`, `brcm,bcm2710`). Fallback: `/sys/firmware/devicetree/base/model` for display name.

## Core features

1. **Overview dashboard**: Per-core CPU gauges, memory gauge with sparkline, swap usage bar, load average (1/5/15m), SoC temperature with color thresholds, throttle status indicator, aggregate network throughput (all interfaces summed), fan RPM/PWM (Pi 5 only)
2. **Process monitor**: Sortable table (PID, name, CPU%, MEM%, user), vim-style navigation (j/k, arrows), kill with confirmation dialog
3. **Power tab**: PMIC rail voltages and currents (Pi 5) with total wattage estimate and sparkline, EXT5V_V input voltage, BATT_V RTC battery voltage (Pi 5), RP1 ADC voltages including USB VBus (Pi 5), core/SDRAM voltages (Pi 4B), PoE HAT status and current draw (Pi 5, Pi 4B)
4. **PCIe info** (Pi 5, shown in Power tab): Link speed with Gen 2/3 label, width (x1), connected device name
5. **Network tab**: Per-interface status (up/down), IP addresses, real-time rx/tx throughput with sparklines
6. **Disk tab**: Mounted partitions with usage bars, per-disk I/O rates (read/write KB/s)
7. **System info tab**: Board model and revision, SoC name, kernel version, OS info, uptime, CPU architecture/frequency range/governor, total RAM/swap

## UX features

- **Color-coded thresholds**: Hard-coded green/yellow/red levels for temperature, CPU, and memory (values from `config/default.toml`). Configurable themes deferred to v2.
- **Pause/resume**: Spacebar freezes the display for value inspection
- **Help overlay**: `?` key shows keyboard shortcut reference
- **Tab navigation**: `1`–`6` direct, `Tab`/`Shift+Tab` cycle
- **Keyboard controls**: `q`/`Ctrl+C` quit, `s` cycle sort column (Processes tab), `K` kill process

## Non-functional requirements

- Single static binary, no runtime dependencies (vcgencmd is optional — features degrade silently without it)
- Binary size under 5MB stripped
- RSS memory usage under 10MB on Zero 2W
- CPU overhead under 2% at default 1-second refresh interval (configurable via `--interval`)
- Works over SSH on 80×24 minimum terminal, scales to larger
- Graceful degradation on any Linux, including x86: CPU, memory, disk, network, and process monitoring work with generic Linux collectors. Pi-specific features (PMIC, fan, PCIe, PoE, vcgencmd data) show "N/A" or are hidden on non-Pi hardware. This enables development and CI testing on x86 GitHub Actions runners.
- vcgencmd failures (missing binary, permission denied, timeout) are silently ignored. Logged to stderr once when `--verbose` flag is passed.

## CLI arguments

- `--interval` / `-i`: Refresh interval in milliseconds (default 1000)
- `--tab` / `-t`: Starting tab number (1–6, default 1)
- `--board`: Force board type (`pi5`/`pi4b`/`zero2w`/`auto`, default `auto`)
- `--verbose` / `-v`: Log warnings (e.g., vcgencmd unavailable) to stderr
- `--version` / `-V`: Print version and exit

## Out of scope for v1

- Stress testing (unlike s-tui)
- Web dashboard or remote access
- GPIO pin monitoring
- Configurable color themes (defer to v2)
- Config file for custom thresholds (defer to v2)
- Plugin/extension system
- Multi-Pi monitoring from a single instance
- PCIe AER error counts (niche, complex root port path — defer to v2)

## Success metrics

- Correctly detects all three board types at startup
- Displays accurate data validated against `vcgencmd` and `htop` on real hardware
- Compiles and runs on x86 Linux with generic collectors (for development)
- Installable via `curl` one-liner (prebuilt binaries for aarch64 + armv7) or `cargo install pitop`
- Under 2% CPU overhead on Zero 2W at default refresh interval

## Technical constraints

- Language: Rust (ratatui + crossterm + tokio)
- No sysinfo crate — direct procfs/sysfs parsing
- Cross-compiled from x86 host, tested on real Pi hardware
- Published on crates.io
- MIT license
