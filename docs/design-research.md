# pitop — Raspberry Pi System Monitor TUI (Rust)

## Design Research Document

---

## 1. Lessons from existing projects

### pitop (Go) — PierreKieffer/pitop (archived Oct 2025)

**What they did right:**
- Zero external dependencies for data collection — all system data parsed directly from procfs/sysfs using Go's stdlib. This keeps the binary tiny and avoids runtime dependency headaches on the Pi.
- Ships prebuilt binaries for both 32-bit and 64-bit ARM, with one-line curl install scripts. Users don't need Go installed.
- Uses `gizak/termui` with `nsf/termbox` for rendering — clean widget-based layout with sparklines and gauges.

**What we can improve:**
- Only targets RPi 3/4, no Pi 5-specific features (no PMIC, no fan, no RP1 thermals, no PCIe).
- Now archived — the project has been abandoned, which validates the need for a replacement.
- No board detection — it assumes a generic Pi and misses board-specific telemetry.
- Single-view layout — no tabs, no way to dive deeper into specific subsystems.

**Key takeaway:** The direct procfs/sysfs parsing approach (no external libraries for data) is the right call for a Pi-specific tool. We should follow this pattern in Rust.

### PI Dashboard (Python) — emphyri0/pi_dashboard

**What they did right:**
- Tabbed interface (CPU, Processes, Disk, Network, System) — great UX for SSH sessions.
- Lazy data fetching — Disk and Network tab data only refreshes when those tabs are active. This is important for reducing load on a Zero 2W.
- Color-coded thresholds — CPU/RAM/Temp go yellow at warning levels, red at critical. Instant visual feedback.
- Pi-specific GPU monitoring via `vcgencmd` for temperature and VRAM usage.
- Pause/resume with spacebar — useful when you want to snapshot values.
- History graphs using Unicode block characters for sparklines.

**What we can improve:**
- Python + psutil is heavyweight for a Zero 2W (psutil alone pulls ~15MB RSS). A Rust binary will be ~2-5MB total.
- No power monitoring, no PCIe info, no fan speed, no throttle state display.
- curses rendering is limited compared to what ratatui can do.

**Key takeaway:** The tabbed architecture with lazy data fetching per tab is the right pattern. We should replicate the threshold-based color coding and the spacebar pause feature.

### s-tui (Python) — amanusk/s-tui

**What they did right:**
- Integrated stress testing — can toggle between monitoring and load testing in the same UI.
- Hook system — runs user-defined shell scripts when thresholds are exceeded (e.g., temp > 80°C triggers a notification).
- Configuration file at `~/.config/s-tui/s-tui.conf` for persisting settings.
- Smooth graphs using urwid's UTF-8 Braille patterns.

**What we can improve:**
- CPU-only focus — no memory, disk, network, or process monitoring.
- Intel-centric power reading (RAPL) doesn't work on ARM at all.

**Key takeaway:** The threshold hook system is a nice-to-have for v2. The config file pattern is good practice. For v1, focus on monitoring breadth rather than stress testing.

### hwtop (Rust) — Arnavion/hwtop

**What they did right:**
- Already in Rust with a Pi-specific config profile.
- Per-device TOML configuration files for defining which sensors to read.
- Minimal, compact output — fits in a very small terminal.

**What we can improve:**
- Very minimal — just CPU, temps, and network in a single text view. No charts, no process list, no interactivity.
- Config-file driven sensor selection is powerful but complex for new users.

**Key takeaway:** The TOML config approach for sensor paths is useful if we want users to customize which hwmon devices to read. Good fallback strategy.

---

## 2. Board detection and hardware profiles

### Detection method

Read `/proc/device-tree/compatible` (recommended by the Raspberry Pi team as the canonical, distro-agnostic method):

| Board | compatible string | SoC |
|-------|------------------|-----|
| Pi 5 | `raspberrypi,5-model-b` + `brcm,bcm2712` | BCM2712 |
| Pi 4B | `raspberrypi,4-model-b` + `brcm,bcm2711` | BCM2711 |
| Zero 2W | `raspberrypi,model-zero-2-w` + `brcm,bcm2710` | BCM2710A1 |

Fallback: read `/sys/firmware/devicetree/base/model` for the human-readable string (e.g., "Raspberry Pi 5 Model B Rev 1.0").

### Hardware capability matrix

| Feature | Pi 5 | Pi 4B | Zero 2W |
|---------|------|-------|---------|
| CPU cores | 4× A76 @ 2.4GHz | 4× A72 @ 1.8GHz | 4× A53 @ 1.0GHz |
| Thermal zones | SoC + PMIC + RP1 | SoC only | SoC only |
| PMIC power rails | 12 rails via `vcgencmd pmic_read_adc` | ✗ | ✗ |
| Fan control | PWM fan header (hwmon) | ✗ (GPIO fan only) | ✗ |
| External PCIe | x1 Gen 2.0 (configurable to Gen 3.0) | ✗ (internal only) | ✗ |
| PoE HAT support | Yes (PoE+ HAT) | Yes (PoE/PoE+ HAT) | ✗ |
| USB 3.0 | 2 ports | 2 ports | ✗ |
| Voltage monitoring | PMIC rails + EXT5V_V | `measure_volts` (core/sdram) | `measure_volts` (core/sdram) |
| Throttle detection | `get_throttled` bitmask | `get_throttled` bitmask | `get_throttled` bitmask |
| RP1 southbridge | Yes (temps + USB VBus ADC) | ✗ | ✗ |

---

## 3. Data sources — full sysfs/procfs reference

### Universal (all boards)

| Metric | Source | Notes |
|--------|--------|-------|
| CPU usage (per-core) | `/proc/stat` | Parse `cpu0..cpuN` lines, compute delta of jiffies |
| CPU frequency | `/sys/devices/system/cpu/cpufreq/policy0/scaling_cur_freq` | Value in kHz |
| CPU min/max freq | `scaling_min_freq`, `scaling_max_freq` | Same directory |
| CPU governor | `scaling_governor` | Same directory |
| SoC temperature | `/sys/class/thermal/thermal_zone0/temp` | Millidegrees C (÷1000) |
| Memory | `/proc/meminfo` | MemTotal, MemFree, MemAvailable, Buffers, Cached, SwapTotal, SwapFree |
| Disk I/O | `/proc/diskstats` | Sector reads/writes per device, compute deltas |
| Disk usage | `/proc/mounts` + `statvfs()` | Total/used/free per mount |
| Network I/O | `/proc/net/dev` | Bytes rx/tx per interface, compute deltas |
| Processes | `/proc/[pid]/stat` + `/proc/[pid]/status` | PID, name, CPU%, RSS, user |
| Load average | `/proc/loadavg` | 1, 5, 15 minute |
| Uptime | `/proc/uptime` | Seconds since boot |
| Hostname | `/proc/sys/kernel/hostname` | |
| Kernel version | `/proc/version` | |
| Throttle state | `vcgencmd get_throttled` | Hex bitmask (see below) |
| CPU temp (alt) | `vcgencmd measure_temp` | Returns `temp=XX.X'C` |

### Throttle bitmask (`vcgencmd get_throttled`)

```
Bit  Meaning (current)
 0   Under-voltage detected
 1   ARM frequency capped
 2   Currently throttled
 3   Soft temperature limit active

Bit  Meaning (since boot)
16   Under-voltage has occurred
17   ARM frequency capping has occurred
18   Throttling has occurred
19   Soft temperature limit has occurred
```

### Pi 5 specific

| Metric | Source | Notes |
|--------|--------|-------|
| PMIC power rails | `vcgencmd pmic_read_adc` | 12 current + 12 voltage readings |
| PMIC temperature | `vcgencmd measure_temp pmic` | Separate from SoC temp |
| RP1 temperature | `/sys/class/hwmon/hwmonN/temp1_input` (where name = `rp1_adc`) | RP1 southbridge temp |
| RP1 ADC voltages | Same hwmon device, `in1_input` through `in4_input` | AIN1 = USB VBus/2 |
| Fan speed (RPM) | `/sys/devices/platform/cooling_fan/hwmon/hwmonN/fan1_input` | |
| Fan PWM duty | Same directory, `pwm1` | 0-255 |
| PCIe link speed | `/sys/bus/pci/devices/0000:01:00.0/current_link_speed` | e.g., "5.0 GT/s" (Gen2) or "8.0 GT/s" (Gen3) |
| PCIe link width | `/sys/bus/pci/devices/0000:01:00.0/current_link_width` | e.g., "1" (x1) |
| PCIe max speed | Same device, `max_link_speed` | Device capability |
| PCIe max width | Same device, `max_link_width` | Device capability |
| PCIe device name | Same device, read via class/vendor/device files | Or parse `lspci` output |
| PCIe AER errors | `/sys/devices/platform/axi/1000110000.pcie/pci0000:00/0000:00:00.0/aer_dev_correctable` | Error counters |

**PMIC power estimation formula:**
```
total_pmic_watts = sum(voltage[i] * current[i]) for i in 0..11
estimated_real_watts = total_pmic_watts * 1.1451 + 0.5879
```
Note: This does not include 5V rail power (USB peripherals, HATs, NVMe).

**PCIe speed mapping:**
| GT/s | Generation | Max throughput (x1) |
|------|-----------|-------------------|
| 2.5 GT/s | Gen 1.0 | ~250 MB/s |
| 5.0 GT/s | Gen 2.0 | ~500 MB/s |
| 8.0 GT/s | Gen 3.0 | ~985 MB/s |

### Pi 4B specific

| Metric | Source | Notes |
|--------|--------|-------|
| Core voltage | `vcgencmd measure_volts core` | e.g., `volt=0.8688V` |
| SDRAM voltages | `vcgencmd measure_volts sdram_c/sdram_i/sdram_p` | Three separate rails |
| USB/Ethernet | Internal VL805 via PCIe Gen 2 x1 | Not exposed as external PCIe |

### PoE HAT detection (Pi 5 + Pi 4B)

The PoE HAT driver exposes a `power_supply` device in sysfs when connected:
- Look for `/sys/class/power_supply/rpi-poe*/` 
- Properties: `POWER_SUPPLY_PROP_HEALTH`, `POWER_SUPPLY_PROP_ONLINE`, `POWER_SUPPLY_PROP_CURRENT_NOW`, `POWER_SUPPLY_PROP_CURRENT_AVG`, `POWER_SUPPLY_PROP_CURRENT_MAX`
- The PoE HAT also has its own fan controlled via firmware/I2C

Detection: Check if the power_supply device exists. If present, display PoE status and current draw. Also, the EXT5V_V reading from PMIC on Pi 5 can indicate PoE power source (PoE typically provides slightly different voltage than USB-C PSU).

---

## 4. Rust technology stack

### Core dependencies

| Crate | Purpose | Why |
|-------|---------|-----|
| `ratatui` | TUI rendering | 19k+ stars, charts/sparklines/tables/gauges built-in, immediate-mode rendering, crossterm backend works over SSH |
| `crossterm` | Terminal backend | Cross-platform, works on all terminals including SSH, handles raw mode and alternate screen |
| `tokio` | Async runtime | Needed for async tick loop, subprocess calls to `vcgencmd`, and non-blocking I/O |
| `clap` | CLI argument parsing | Refresh interval, color theme, default tab, board override |

### Data collection strategy: Direct procfs/sysfs (no sysinfo crate)

Rationale: Following the original pitop's approach, read files directly. This gives us:
- Full control over Pi-specific paths
- Smaller binary size (~2MB vs ~8MB with sysinfo)
- No unnecessary cross-platform abstraction
- We know exactly what we're reading and can handle Pi quirks

The `sysinfo` crate recently added Pi temperature fixes, but it still doesn't know about PMIC, fan, PCIe, or PoE — we'd need custom collectors for those anyway. Going all-custom keeps the architecture uniform.

### vcgencmd integration

Shell out to `vcgencmd` for data that isn't available via sysfs:
- `pmic_read_adc` (Pi 5 power rails)
- `get_throttled` (throttle bitmask)
- `measure_temp` / `measure_temp pmic`
- `measure_volts`
- `measure_clock arm`

Use `tokio::process::Command` for async subprocess execution with a timeout. Cache results for 1 second minimum to avoid hammering the firmware mailbox.

### Cross-compilation

| Target board | Rust target | Toolchain |
|-------------|-------------|-----------|
| Pi 5 (64-bit) | `aarch64-unknown-linux-gnu` | `aarch64-linux-gnu-gcc` |
| Pi 4B (64-bit) | `aarch64-unknown-linux-gnu` | Same as Pi 5 |
| Pi 4B (32-bit) | `armv7-unknown-linux-gnueabihf` | `arm-linux-gnueabihf-gcc` |
| Zero 2W (32-bit) | `armv7-unknown-linux-gnueabihf` | Same |
| Zero 2W (64-bit) | `aarch64-unknown-linux-gnu` | Same as Pi 5 |

Recommend using `cross` (Docker-based) or GitHub Actions with matrix builds for all targets.

---

## 5. UI layout design

### Tab structure

```
[1:Overview] [2:Processes] [3:Power] [4:Network] [5:Disk] [6:System]
```

**Tab 1 — Overview** (default view, always-on)
- Board name + SoC + kernel in header bar
- CPU: Per-core usage gauges + aggregate sparkline history
- Memory: Used/Total gauge + sparkline
- Temperature: SoC temp gauge (+ PMIC + RP1 on Pi 5)
- Throttle status: Color-coded indicator (green/yellow/red)
- Network: Aggregate rx/tx throughput
- Fan: RPM + PWM% (Pi 5 only)

**Tab 2 — Processes**
- Sortable table: PID, Name, CPU%, MEM%, User
- Sort by CPU (default), toggle with keys
- Vim-style navigation (j/k or arrows)
- Kill process with 'K' key (with confirmation)

**Tab 3 — Power** (Pi 5: full PMIC breakdown; Pi 4B/Zero: basic voltage)
- Pi 5: Per-rail voltage + current table, total wattage estimate, sparkline
- Pi 4B: Core + SDRAM voltages
- Zero 2W: Core voltage only
- PoE status (if HAT detected): Online/offline, current draw
- PCIe info (Pi 5): Link speed (Gen 2/3), width (x1), device name, AER error count

**Tab 4 — Network**
- Per-interface: status (Up/Down), IP addresses, rx/tx rates
- Sparkline per active interface

**Tab 5 — Disk**
- Partition table: device, mountpoint, total/used/free, usage%
- Per-disk I/O rates (read/write KB/s)

**Tab 6 — System**
- Board model, SoC, revision
- Kernel version, OS info
- Uptime
- CPU: model name, architecture, frequency range, governor
- Memory: total RAM, total swap
- PCIe topology (Pi 5): connected devices with link status

### Keyboard controls

| Key | Action |
|-----|--------|
| 1-6 | Switch tabs |
| Tab / Shift+Tab | Next / previous tab |
| q / Ctrl+C | Quit |
| Space | Pause / resume updates |
| j/k or ↑/↓ | Navigate lists |
| s | Cycle sort column (Processes tab) |
| K | Kill selected process (with confirm) |
| ? | Help overlay |

---

## 6. Project structure

```
pitop/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs                 # Entry point, arg parsing, terminal init
│   ├── app.rs                  # App state, tick handler, event loop
│   ├── event.rs                # Keyboard/resize event handling
│   │
│   ├── board/
│   │   ├── mod.rs              # BoardProfile trait + detection logic
│   │   ├── pi5.rs              # Pi 5 capabilities and collector config
│   │   ├── pi4b.rs             # Pi 4B capabilities
│   │   ├── zero2w.rs           # Zero 2W capabilities
│   │   └── unknown.rs          # Graceful fallback for unsupported boards
│   │
│   ├── collectors/
│   │   ├── mod.rs              # Collector trait, refresh scheduling
│   │   ├── cpu.rs              # /proc/stat, cpufreq
│   │   ├── memory.rs           # /proc/meminfo
│   │   ├── thermal.rs          # thermal_zone, hwmon enumeration
│   │   ├── network.rs          # /proc/net/dev
│   │   ├── disk.rs             # /proc/diskstats, mount info
│   │   ├── process.rs          # /proc/[pid]/ scanning
│   │   ├── throttle.rs         # vcgencmd get_throttled
│   │   ├── power.rs            # vcgencmd pmic_read_adc, measure_volts
│   │   ├── fan.rs              # cooling_fan hwmon (Pi 5)
│   │   ├── pcie.rs             # /sys/bus/pci/devices/*/current_link_*
│   │   └── poe.rs              # /sys/class/power_supply/rpi-poe*
│   │
│   ├── ui/
│   │   ├── mod.rs              # Tab routing, layout framework
│   │   ├── header.rs           # Top bar: board name, time, throttle status
│   │   ├── overview.rs         # Tab 1: dashboard gauges + sparklines
│   │   ├── processes.rs        # Tab 2: sortable process table
│   │   ├── power.rs            # Tab 3: PMIC rails, voltages, PCIe, PoE
│   │   ├── network.rs          # Tab 4: interface list + throughput
│   │   ├── disk.rs             # Tab 5: partitions + I/O
│   │   ├── system.rs           # Tab 6: board info, uptime, kernel
│   │   └── widgets/
│   │       ├── gauge_bar.rs    # Color-coded gauge with thresholds
│   │       └── sparkline.rs    # Ring-buffer backed sparkline
│   │
│   └── util/
│       ├── ring_buffer.rs      # Fixed-size circular buffer for history
│       ├── format.rs           # Human-readable bytes, temps, watts, durations
│       ├── vcgencmd.rs         # Async subprocess wrapper with caching
│       └── sysfs.rs            # Helper for reading/parsing sysfs files
│
├── config/
│   └── default.toml            # Default thresholds and refresh intervals
│
└── .cargo/
    └── config.toml             # Cross-compilation linker configs
```

---

## 7. Implementation priorities

### Phase 1 — MVP
1. Board detection (Pi 5, Pi 4B, Zero 2W)
2. CPU usage (per-core) + frequency + temperature
3. Memory + swap
4. Throttle status
5. Overview tab with gauges and sparklines
6. Basic keyboard navigation

### Phase 2 — Full monitoring
7. Process list with sorting
8. Network interfaces + throughput
9. Disk partitions + I/O
10. System info tab
11. Tabbed UI with all 6 tabs

### Phase 3 — Pi-specific features
12. PMIC power rails (Pi 5)
13. Fan speed/PWM (Pi 5)
14. PCIe link info (Pi 5)
15. PoE HAT detection
16. Per-rail voltage display (Pi 4B)

### Phase 4 — Polish
17. Color themes (configurable)
18. Config file for thresholds
19. Cross-compilation CI pipeline
20. Prebuilt binary releases
21. AUR / deb / RPM packaging
