# pitop — Claude Code Project Instructions

## Project

pitop is a terminal-based system monitor for Raspberry Pi, written in Rust.
It targets three boards: Raspberry Pi 5, Raspberry Pi 4B, and Raspberry Pi Zero 2W.
Think "mactop for Raspberry Pi" — a single-binary TUI that shows CPU, memory,
thermals, power, network, disk, and processes, with board-specific features like
PMIC power rails, fan speed, PCIe link status, and PoE detection.

## Architecture rules

- **Direct sysfs/procfs parsing only.** Do not use the `sysinfo` crate. All system
  data is read from `/proc/` and `/sys/` files directly, or via `vcgencmd` subprocess.
- **vcgencmd calls** go through `src/util/vcgencmd.rs`. This module uses
  `tokio::process::Command` with a 2-second timeout, caches results for 1 second
  minimum, and returns `Option<T>` (never panics if vcgencmd is missing).
- **Board detection at startup** reads `/proc/device-tree/compatible` to determine
  which collectors to activate. Unknown boards run with generic Linux collectors only.
- **Lazy tab refresh**: Only the active tab's expensive collectors (processes, disk,
  network details) run each tick. Overview tab metrics always run.
- **Ring buffers** for all sparkline history: fixed 60-sample window, implemented in
  `src/util/ring_buffer.rs`.
- **hwmon discovery**: Never hardcode hwmon numbers (they change across reboots).
  Enumerate `/sys/class/hwmon/` and match by the `name` file content.
- **Graceful degradation**: Every Pi-specific feature must handle the case where
  the hardware/sysfs path doesn't exist. The app must run (with reduced features)
  on any Linux system, not just Raspberry Pi.

## Technology stack

- `ratatui` + `crossterm` — TUI rendering and terminal backend
- `tokio` — Async runtime for tick loop and subprocess calls
- `anyhow` — Error handling (no `unwrap()` in production code)
- `clap` — CLI argument parsing

## Coding standards

- **No `unwrap()` or `expect()`** in any code path that can reach production.
  Use `anyhow::Result`, `Option`, or `.unwrap_or_default()`.
- **No `std::process::Command`** — always use `tokio::process::Command`.
- **No hardcoded paths** for hwmon devices. Discover by enumerating and matching name.
- All sysfs reads must handle `ENOENT` / `EACCES` gracefully (feature not available).
- Collector trait: `fn collect(&mut self) -> Result<()>`
- Tests use fixture files from `tests/fixtures/{pi5,pi4b,zero2w}/`.
- `cargo clippy` must pass with zero warnings.
- `cargo fmt` must be applied before every commit.

## Module structure

```
src/
├── main.rs              Entry point, arg parsing, terminal init
├── app.rs               App state, tick handler, event dispatch
├── event.rs             Keyboard/resize event handling
├── board/               Board detection and hardware profiles
│   ├── mod.rs           BoardProfile trait + detect() function
│   ├── pi5.rs           Pi 5 capabilities
│   ├── pi4b.rs          Pi 4B capabilities
│   └── zero2w.rs        Zero 2W capabilities
├── collectors/          Data collection modules
│   ├── mod.rs           Collector trait definition
│   ├── cpu.rs           /proc/stat + cpufreq
│   ├── memory.rs        /proc/meminfo
│   ├── thermal.rs       thermal_zone + hwmon enumeration
│   ├── network.rs       /proc/net/dev
│   ├── disk.rs          /proc/diskstats + mount info
│   ├── process.rs       /proc/[pid]/ scanning
│   ├── throttle.rs      vcgencmd get_throttled
│   ├── power.rs         vcgencmd pmic_read_adc + measure_volts
│   ├── fan.rs           cooling_fan hwmon (Pi 5)
│   ├── pcie.rs          /sys/bus/pci/devices/*/current_link_*
│   └── poe.rs           /sys/class/power_supply/rpi-poe*
├── ui/                  TUI rendering
│   ├── mod.rs           Tab routing and layout framework
│   ├── header.rs        Top bar: board name, time, throttle status
│   ├── overview.rs      Tab 1: dashboard gauges + sparklines
│   ├── processes.rs     Tab 2: sortable process table
│   ├── power.rs         Tab 3: PMIC, voltages, PCIe, PoE
│   ├── network.rs       Tab 4: interfaces + throughput
│   ├── disk.rs          Tab 5: partitions + I/O
│   ├── system.rs        Tab 6: board info, uptime, kernel
│   └── widgets/         Custom widget helpers
└── util/
    ├── ring_buffer.rs   Fixed-size circular buffer
    ├── format.rs        Human-readable bytes, temps, watts, durations
    ├── vcgencmd.rs      Async subprocess wrapper with caching
    └── sysfs.rs         Helper for reading/parsing sysfs files
```

## Do not

- Do not use the `sysinfo` crate or any system info abstraction library.
- Do not hardcode hwmon numbers (e.g., `hwmon2`). Always discover by name.
- Do not shell out for data available via sysfs (temperature, CPU freq, etc.).
- Do not use `std::process::Command` — use `tokio::process::Command`.
- Do not panic on missing hardware features. Return `None` or skip gracefully.
- Do not add GUI dependencies. This is a terminal-only application.

## Supported boards

| Board | Detection string in /proc/device-tree/compatible |
|-------|--------------------------------------------------|
| Pi 5 | Contains `brcm,bcm2712` |
| Pi 4B | Contains `brcm,bcm2711` |
| Zero 2W | Contains `brcm,bcm2710` |

## Key sysfs/procfs paths

See `docs/design-research.md` for the complete reference of all paths,
parsing formats, and board-specific data sources.

## Current sprint

See `bmad-artifacts/epics/` for current stories and their status.
Start with Epic 1 (board detection) and work sequentially.
