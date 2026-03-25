# Architecture

This page describes the internal architecture of pitop for developers who want to understand or contribute to the codebase.

---

## Design principles

1. **Direct sysfs/procfs parsing only** -- no external system-info libraries (e.g., `sysinfo` crate). All data is read from `/proc/` and `/sys/` files directly, or via `vcgencmd` subprocess.

2. **Graceful degradation** -- every Pi-specific feature handles missing hardware. The app runs on any Linux system with reduced features.

3. **No panics in production** -- no `unwrap()` or `expect()` in any code path that can reach production. Use `anyhow::Result`, `Option`, or `.unwrap_or_default()`.

4. **Lazy tab refresh** -- only the active tab's expensive collectors run each tick. Overview metrics always run.

5. **Dynamic hardware discovery** -- never hardcode hwmon numbers or device paths that change between reboots.

---

## Module structure

```
src/
+-- main.rs              Entry point, arg parsing, terminal init
+-- app.rs               App state, tick handler, event dispatch
+-- event.rs             Keyboard/resize event handling
+-- config.rs            TOML config loading and validation
+-- stress.rs            CPU stress test worker management
+-- board/               Board detection and hardware profiles
|   +-- mod.rs           BoardProfile trait + detect() function
|   +-- pi5.rs           Pi 5 capabilities
|   +-- pi4b.rs          Pi 4B capabilities
|   +-- zero2w.rs        Zero 2W capabilities
+-- collectors/          Data collection modules
|   +-- mod.rs           Module exports
|   +-- cpu.rs           /proc/stat + cpufreq
|   +-- memory.rs        /proc/meminfo
|   +-- thermal.rs       thermal_zone + hwmon enumeration
|   +-- network.rs       /proc/net/dev
|   +-- disk.rs          /proc/diskstats + mount info
|   +-- process.rs       /proc/[pid]/ scanning
|   +-- throttle.rs      vcgencmd get_throttled
|   +-- power.rs         vcgencmd pmic_read_adc + measure_volts
|   +-- fan.rs           cooling_fan hwmon (Pi 5)
|   +-- gpu.rs           vcgencmd GPU metrics
|   +-- pcie.rs          /sys/bus/pci/devices/*/current_link_*
|   +-- poe.rs           /sys/class/power_supply/rpi-poe*
+-- ui/                  TUI rendering
|   +-- mod.rs           Tab routing and layout framework
|   +-- header.rs        Top bar: board name, time, throttle
|   +-- overview.rs      Tab 1: dashboard gauges + sparklines
|   +-- processes.rs     Tab 2: sortable process table
|   +-- power.rs         Tab 3: PMIC, voltages, PCIe, PoE
|   +-- network.rs       Tab 4: interfaces + throughput
|   +-- disk.rs          Tab 5: partitions + I/O
|   +-- system.rs        Tab 6: board info, uptime, kernel
|   +-- help.rs          Help overlay popup
|   +-- theme.rs         Color themes
|   +-- widgets/         Custom widget helpers
+-- util/
    +-- ring_buffer.rs   Fixed-size circular buffer
    +-- format.rs        Human-readable bytes, temps, watts
    +-- vcgencmd.rs      Async subprocess wrapper with caching
    +-- sysfs.rs         Helper for reading/parsing sysfs files
```

---

## Application lifecycle

### Startup

1. Parse CLI arguments via `clap`
2. Load config from `~/.config/pitop/config.toml` (or CLI-specified path)
3. Detect board type from `/proc/device-tree/compatible`
4. Create `App` struct with all collectors and ring buffers
5. Set initial theme and starting tab
6. Initialize terminal (raw mode, alternate screen)
7. Start the tick loop

### Tick loop

```
loop {
    render UI
    poll for keyboard events (with timeout = tick_rate)
    drain any remaining queued events
    if should_quit: break
    if tick interval elapsed:
        run always-on collectors (CPU, memory, thermal, network, fan, GPU, throttle)
        run tab-specific collectors (based on active_tab)
        update sparkline ring buffers
}
```

### Shutdown

1. Signal stress test workers to stop (if active)
2. Restore terminal (raw mode off, alternate screen off, mouse capture off)
3. Exit

---

## Collector pattern

Each collector is a struct that owns its parsing state and provides a `collect()` method:

```rust
pub struct CpuCollector {
    root: PathBuf,
    prev_stats: Vec<CpuStat>,  // previous tick's values for delta
}

impl CpuCollector {
    pub fn new(root: &Path) -> Self { ... }
    pub fn collect(&mut self, data: &mut CpuData) -> Result<()> { ... }
}
```

Key characteristics:

- **Stateful**: collectors keep previous-tick data for computing deltas (CPU usage, network throughput, disk I/O)
- **Root-relative**: all sysfs/procfs paths are relative to a configurable root, enabling testing with fixture directories
- **Error propagation**: `collect()` returns `Result<()>`. Errors are logged in verbose mode but never crash the app

---

## Board detection

The `board` module reads `/proc/device-tree/compatible` at startup:

```rust
pub fn detect(root: &Path) -> BoardType {
    // Read null-separated strings from /proc/device-tree/compatible
    // Match against known SoC identifiers
    // Return BoardType::Pi5, Pi4B, Zero2W, or Unknown
}
```

Each `BoardType` has a corresponding `BoardProfile` that defines:

- Human-readable name
- Available thermal zones
- Voltage source type (`Pmic`, `MeasureVolts`, `None`)
- Whether fan, PCIe, PoE monitoring is supported

---

## Lazy tab refresh

To minimize CPU usage (important on the Pi Zero 2W), expensive collectors only run when their tab is active:

| Tab | Always-on | Tab-specific |
|-----|-----------|-------------|
| Overview | CPU, memory, thermal, network, fan, GPU, throttle | -- |
| Processes | (same) | Process table scan |
| Power | (same) | PMIC/voltage, PCIe, PoE |
| Network | (same) | (network already always-on) |
| Disk | (same) | Disk partitions & I/O |
| System | (same) | (static data, no refresh) |

---

## Ring buffers

Sparkline history is stored in fixed-size ring buffers (`RingBuffer<T>`):

- Default capacity: 60 samples (configurable via `history_size`)
- Valid range: 10 to 600 samples
- When full, the oldest sample is overwritten
- The `as_vec()` method returns samples in chronological order for rendering

Separate ring buffers exist for CPU usage, memory usage, temperature, power draw, GPU frequency, and per-interface network throughput.

---

## vcgencmd wrapper

All `vcgencmd` subprocess calls go through `VcgencmdRunner`:

- Uses `tokio::process::Command` (async, never blocks the tick loop)
- 2-second timeout per call
- 1-second minimum cache TTL (avoids re-running the same command within 1 second)
- Returns `Option<String>` -- `None` if vcgencmd is missing, times out, or fails
- Never panics

---

## Theme system

The `Theme` struct holds all UI colors. Three built-in themes are available (`default`, `monochrome`, `solarized`), plus a user-defined `custom` theme loaded from the config file.

All UI rendering modules read colors from the active `Theme` instance rather than hardcoding `Color::*` values. Theme cycling at runtime updates the `Theme` on the `App` struct.

---

## Technology stack

| Dependency | Purpose |
|-----------|---------|
| `ratatui` 0.29 | TUI widget library |
| `crossterm` 0.28 | Terminal backend (raw mode, events, alternate screen) |
| `tokio` 1.x | Async runtime (multi-thread, timers, process, fs) |
| `clap` 4 | CLI argument parsing (derive macros) |
| `anyhow` 1 | Error handling |
| `serde` + `toml` | Config file parsing |
| `libc` | Process kill (SIGTERM) |

### Release profile

The release binary is optimized for size, which is important for the Pi Zero 2W:

```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
strip = true        # Strip debug symbols
codegen-units = 1   # Better optimization
panic = "abort"     # Smaller binary
```
