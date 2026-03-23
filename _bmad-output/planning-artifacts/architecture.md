---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
status: 'complete'
completedAt: '2026-03-22'
inputDocuments: [_bmad-output/planning-artifacts/prd.md, bmad-artifacts/product-brief.md, docs/design-research.md]
workflowType: 'architecture'
project_name: 'pitop'
user_name: 'Hongjunwu'
date: '2026-03-22'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**
48 FRs across 9 capability areas. The dominant pattern is independent data collectors feeding a tabbed TUI. Each collector reads from a specific procfs/sysfs path or vcgencmd subprocess, transforms raw data into display-ready structs, and stores history in fixed-size ring buffers. The UI renders the active tab's data each frame. Board detection at startup determines which collectors are activated.

**Non-Functional Requirements:**
The architecture-shaping NFRs are:
- **NFR1-2 (Performance budget):** < 2% CPU, < 10MB RSS on Zero 2W. Drives allocation strategy, ring buffer sizing, and collector scheduling.
- **NFR5 (Non-blocking collectors):** A slow vcgencmd call must not block UI or other collectors. Requires async execution or timeout isolation.
- **NFR6 (vcgencmd caching):** 2-second timeout, 1-second cache minimum. Dedicated caching wrapper module.
- **NFR8 (24hr stability):** Fixed-size data structures only. No unbounded growth.
- **NFR9 (No unwrap):** Pervasive `Result`/`Option` handling via anyhow.
- **NFR10-11 (Terminal restore):** Panic hook + signal handler for clean shutdown.
- **NFR20 (hwmon discovery):** Enumerate-and-match pattern, never hardcode device numbers.

**Scale & Complexity:**
- Primary domain: Systems programming / TUI application
- Complexity level: Low-medium
- Estimated architectural components: ~20 modules (12 collectors + 6 UI tabs + 4 utility modules)

### Technical Constraints & Dependencies

- **No sysinfo crate** — all data collection is custom procfs/sysfs parsing
- **tokio async runtime** — required for vcgencmd subprocess and tick loop
- **ratatui + crossterm** — TUI rendering, terminal backend
- **clap** — CLI argument parsing with derive macros
- **serde + serde_json** — JSON snapshot mode (`--json`)
- **clap_complete** — shell completion generation (build-time only)
- **anyhow** — error handling throughout
- **Cross-compilation targets:** aarch64-unknown-linux-gnu, armv7-unknown-linux-gnueabihf, x86_64-unknown-linux-gnu

### Cross-Cutting Concerns Identified

1. **Graceful degradation** — Every sysfs read, every vcgencmd call, every hwmon discovery must handle absence gracefully. This is not error handling — it's the expected path on unsupported hardware.
2. **Board-conditional behavior** — Board detection result propagates through collector activation, UI layout, and feature availability. Single decision point, many consumers.
3. **Performance discipline** — Every allocation, every clone, every string format matters on Zero 2W. Architecture must make the cheap path the default path.
4. **Testability on x86** — Collector logic must accept injectable file paths (not hardcoded `/proc/...`) to enable fixture-based testing on non-Pi systems.
5. **Terminal safety** — Raw mode + alternate screen must be restored on every exit path: normal quit, Ctrl+C, panic, collector failure.

## Technology Stack & Project Foundation

### Primary Technology Domain

Rust systems programming — terminal-based TUI application. No starter template ecosystem applies. Project is initialized with `cargo init` and manual dependency configuration.

### Technology Decisions (Pre-Established)

These decisions are locked in from the product brief and CLAUDE.md:

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust | Performance ceiling for Zero 2W (< 5MB binary, < 10MB RSS, < 2% CPU) |
| TUI framework | ratatui + crossterm | Industry standard for Rust TUIs, SSH-compatible, charts/sparklines/gauges built-in |
| Async runtime | tokio (multi-thread) | Required for vcgencmd subprocess timeouts and non-blocking tick loop |
| CLI parsing | clap (derive) | Standard Rust CLI library, generates help/version, supports derive macros |
| Error handling | anyhow | Ergonomic error propagation without custom error types per module |
| Serialization | serde + serde_json | `--json` snapshot mode, derive `Serialize` on all collector data structs |
| Shell completion | clap_complete (build dep) | Generates bash/zsh/fish/PowerShell completions at build time |

### Initialization Command

```bash
cargo init pitop
```

### Cargo.toml Dependencies

```toml
[dependencies]
ratatui = "0.29"
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }
clap = { version = "4", features = ["derive"] }
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
libc = "0.2"  # for statvfs syscall (disk usage)

[build-dependencies]
clap_complete = "4"
```

*Note: Exact version numbers should be verified at project creation time. Use latest compatible versions.*

### Project Structure (From CLAUDE.md)

The module structure is pre-established in CLAUDE.md and aligns with the collector-per-subsystem architecture identified in the context analysis. No structural decisions remain for this step.

## Core Architectural Decisions

### Decision Priority Analysis

**Already Decided (from CLAUDE.md + PRD):**
- Language, framework, dependencies (Step 3)
- Module structure (CLAUDE.md)
- No sysinfo crate, no std::process::Command, no hardcoded hwmon
- Collector trait signature: `fn collect(&mut self) -> Result<()>`
- Ring buffer: 60-sample fixed window
- vcgencmd: tokio::process::Command, 2s timeout, 1s cache

### 1. Event Loop Architecture

**Decision:** Single-threaded tokio event loop with tick-based scheduling.

```
loop {
    select! {
        _ = tick_interval.tick() => { run_collectors(); }
        event = crossterm_events.next() => { handle_input(event); }
    }
    render_ui();
}
```

**Rationale:** ratatui is not thread-safe — rendering must happen on the main thread. Collectors are I/O-bound (file reads), not CPU-bound, so single-threaded async is sufficient. vcgencmd subprocess calls use `tokio::process::Command` which yields the thread during the wait.

**Alternative considered:** Multi-threaded with collector workers. Rejected — adds complexity (channels, Arc<Mutex<>>) for no measurable benefit. File reads complete in microseconds. Only vcgencmd takes meaningful time, and tokio handles that with async.

### 2. App State Design

**Decision:** Single `App` struct owns all collector data and UI state.

```rust
pub struct App {
    pub board: Box<dyn BoardProfile>,
    pub cpu: CpuData,
    pub memory: MemoryData,
    pub thermal: ThermalData,
    pub throttle: ThrottleData,
    pub processes: Vec<ProcessInfo>,
    pub network: NetworkData,
    pub disk: DiskData,
    pub power: Option<PowerData>,     // None on boards without PMIC
    pub fan: Option<FanData>,         // None on boards without fan
    pub pcie: Option<PcieData>,       // None on boards without PCIe
    pub poe: Option<PoeData>,         // None when no PoE HAT
    pub system_info: SystemInfo,      // Static, collected once
    pub active_tab: usize,
    pub paused: bool,
    pub sparklines: SparklineHistory, // Ring buffers for all sparkline data
}
```

**Rationale:** Flat struct with `Option<T>` for board-conditional data. No trait objects for data — just plain structs. The `Option` pattern naturally handles graceful degradation: if `fan` is `None`, the UI simply skips the fan section.

### 3. Collector Execution Model

**Decision:** Collectors are plain functions (not trait objects) grouped into always-run and tab-dependent sets.

```rust
// Always run (every tick):
cpu_collector.collect(&mut app.cpu)?;
memory_collector.collect(&mut app.memory)?;
thermal_collector.collect(&mut app.thermal)?;
throttle_collector.collect(&mut app.throttle)?;
network_collector.collect_summary(&mut app.network)?;  // aggregate only

// Tab-dependent (only when tab is active):
match app.active_tab {
    1 => { /* overview: already covered by always-run */ }
    2 => { process_collector.collect(&mut app.processes)?; }
    3 => { power_collector.collect(&mut app.power)?; }
    4 => { network_collector.collect_detail(&mut app.network)?; }
    5 => { disk_collector.collect(&mut app.disk)?; }
    6 => { /* system info: static, no refresh */ }
}
```

**Rationale:** Lazy tab refresh per CLAUDE.md. The overview tab's data (CPU, memory, thermal, throttle, network summary) runs every tick. Expensive collectors (process scanning, disk I/O, full network detail) only run when their tab is visible. This directly supports NFR1-2 (performance on Zero 2W).

### 4. Sysfs Path Injection for Testing

**Decision:** Each collector accepts a base path, defaulting to `/`.

```rust
impl CpuCollector {
    pub fn new(sysfs_root: &Path) -> Self { ... }
    // reads from {sysfs_root}/proc/stat
    // reads from {sysfs_root}/sys/devices/system/cpu/cpufreq/...
}

// Production:
CpuCollector::new(Path::new("/"))

// Test:
CpuCollector::new(Path::new("tests/fixtures/pi5"))
```

**Rationale:** Cross-cutting concern #4 (testability). Every collector must work with fixture files on x86. A path prefix is the simplest injection mechanism — no trait abstraction, no mock framework, just string concatenation.

### 5. Board Profile → Collector Gating

**Decision:** `BoardProfile` returns capability flags; `App::new()` conditionally creates collectors.

```rust
pub trait BoardProfile: Send + Sync {
    fn board_type(&self) -> BoardType;
    fn name(&self) -> &str;
    fn has_pmic(&self) -> bool;
    fn has_fan(&self) -> bool;
    fn has_pcie(&self) -> bool;
    fn has_poe(&self) -> bool;
    fn thermal_zones(&self) -> &[&str];
    fn voltage_sources(&self) -> VoltageSource; // Pmic | MeasureVolts | None
}
```

**Rationale:** Simple boolean flags rather than a collector factory. The `App::new()` function checks each flag and either creates the collector + initializes the `Option<T>` data field, or leaves it as `None`. The UI checks the same `Option` to decide what to render.

### 6. vcgencmd Wrapper — Shared Instance

**Decision:** Single `VcgencmdRunner` instance shared across all collectors that need it (throttle, power, thermal).

```rust
pub struct VcgencmdRunner {
    cache: HashMap<String, (Instant, String)>,
    available: bool,  // set to false on first NotFound error, never retry
}
```

**Rationale:** NFR6 requires 1-second caching. Multiple collectors calling `get_throttled` and `pmic_read_adc` in the same tick should share the cache. Single instance avoids redundant subprocess spawns. The `available` flag implements silent degradation (FR46) — if vcgencmd isn't found on first call, skip all future attempts.

### Deferred Decisions (Post-MVP)

- **Color theme system:** Hardcoded thresholds for v1. Configurable in v2 via TOML config.
- **Plugin/extension architecture:** Not in scope.
- **JSON schema versioning:** Stabilize schema in v1, version in v2 if needed.

### Decision Impact Analysis

**Implementation Sequence:**
1. sysfs utilities + path injection (foundation for everything)
2. Board detection + profiles (gates collector activation)
3. vcgencmd wrapper (shared dependency for multiple collectors)
4. Ring buffer (required before sparkline UI)
5. Individual collectors (independent, can parallelize)
6. App struct + event loop (integrates collectors + UI)
7. Tab UI rendering (consumes App state)
8. CLI args + JSON mode (entry point wiring)

**Cross-Component Dependencies:**
- All collectors depend on sysfs utilities
- Throttle, power, thermal collectors depend on vcgencmd wrapper
- Fan, thermal (RP1) collectors depend on hwmon discovery (in sysfs utilities)
- All UI tabs depend on App state struct
- JSON mode depends on serde derives on all data structs

## Implementation Patterns & Consistency Rules

### Conflict Points for pitop

7 areas where different AI agents implementing different collectors/modules could make incompatible choices.

### Naming Patterns

**Rust Code Conventions (Standard — enforced by clippy):**
- Types: `PascalCase` — `CpuData`, `BoardType`, `ThrottleData`
- Functions/methods: `snake_case` — `collect`, `read_sysfs_u64`, `discover_hwmon`
- Constants: `SCREAMING_SNAKE_CASE` — `DEFAULT_TICK_INTERVAL`, `VCGENCMD_TIMEOUT`
- Modules/files: `snake_case` — `cpu.rs`, `ring_buffer.rs`, `system_info.rs`

**Data Struct Field Naming:**
- Always `snake_case` — matches Rust convention AND serde's default JSON output
- Use descriptive names: `usage_percent` not `usage`, `bytes_per_sec` not `rate`
- Boolean fields: use `is_` prefix — `is_throttled`, `is_online`, `is_available`
- Units in field names when ambiguous: `temp_celsius`, `freq_khz`, `current_ma`

**JSON Output (`--json` mode):**
- `snake_case` field names (serde default, no rename attributes needed)
- Matches Rust struct field names 1:1 — no translation layer
- Example: `{ "soc_temp_celsius": 52.3, "is_throttled": false, "cpu_usage_percent": 12.5 }`

### Structure Patterns

**Module Organization (from CLAUDE.md, enforced):**
```
src/collectors/{name}.rs  — one file per collector
src/board/{name}.rs       — one file per board profile
src/ui/{name}.rs          — one file per tab
src/util/{name}.rs        — shared utilities
```

**Test Organization:**
- Unit tests: inline `#[cfg(test)] mod tests` at the bottom of each source file
- Integration tests: `tests/` directory at crate root
- Fixture files: `tests/fixtures/{board_type}/` — e.g., `tests/fixtures/pi5/proc-stat`
- Fixture file naming: mirror the sysfs/procfs path with dashes — `/proc/stat` → `proc-stat`, `/sys/class/thermal/thermal_zone0/temp` → `sys-class-thermal-thermal_zone0-temp`

### Data Struct Patterns

**Collector Data Structs — every struct MUST:**
```rust
#[derive(Debug, Default, Clone, Serialize)]
pub struct CpuData {
    pub cores: Vec<CoreUsage>,
    pub aggregate_usage_percent: f64,
    pub frequency_khz: u64,
    pub min_frequency_khz: u64,
    pub max_frequency_khz: u64,
    pub governor: String,
}
```

- Derive `Debug`, `Default`, `Clone`, `Serialize` (all four, always)
- `Default` enables zero-initialization before first collection
- `Clone` enables JSON snapshot without borrowing issues
- `Serialize` enables `--json` mode
- All fields `pub` — data structs are plain containers, not encapsulated

**Optional board-specific data:**
- Use `Option<T>` at the `App` level, not inside the data struct
- Data structs assume they exist — the `Option` gating happens at the collector activation level

### Error Handling Patterns

**In collectors:**
```rust
// GOOD: Return Result, let caller decide
pub fn collect(&mut self, data: &mut CpuData) -> Result<()> {
    let content = read_sysfs_string(&self.stat_path)
        .context("Failed to read /proc/stat")?;
    // ...
    Ok(())
}

// GOOD: For optional features, return Option
pub fn read_fan_rpm(hwmon_path: &Path) -> Option<u32> {
    read_sysfs_u64(&hwmon_path.join("fan1_input")).ok().map(|v| v as u32)
}
```

**In the tick loop:**
```rust
// GOOD: Log and continue — never crash on collector failure
if let Err(e) = cpu_collector.collect(&mut app.cpu) {
    if app.verbose { eprintln!("CPU collector: {}", e); }
}
```

**Forbidden patterns:**
- No `unwrap()` or `expect()` anywhere in `src/`
- No `panic!()` except in unreachable code paths (use `unreachable!()`)
- No `.unwrap_or_else(|_| panic!(...))` — use `.unwrap_or_default()` or `?`

### sysfs/procfs Reading Patterns

**Always use the utility functions:**
```rust
// GOOD:
let temp = read_sysfs_u64(&path.join("temp"))?;

// BAD:
let temp: u64 = std::fs::read_to_string(&path.join("temp"))?.trim().parse()?;
```

**hwmon discovery — always by name:**
```rust
// GOOD:
let fan_hwmon = discover_hwmon("cooling_fan");

// BAD:
let fan_hwmon = Path::new("/sys/class/hwmon/hwmon2");
```

**Path construction — always use the injected root:**
```rust
// GOOD:
let stat_path = self.root.join("proc/stat");

// BAD:
let stat_path = Path::new("/proc/stat");
```

### Enforcement Guidelines

**All AI agents implementing pitop modules MUST:**
1. Derive `Debug, Default, Clone, Serialize` on every data struct
2. Use `read_sysfs_*` utility functions — never raw `std::fs::read_to_string`
3. Accept a `root: &Path` parameter — never hardcode absolute paths
4. Return `Result<()>` from collectors, `Option<T>` from optional features
5. Include `#[cfg(test)] mod tests` with fixture-based tests
6. Run `cargo clippy` and `cargo fmt` before considering work complete

**Anti-Patterns (will cause integration failures):**
- Using `unwrap()` anywhere in `src/`
- Hardcoding `/sys/class/hwmon/hwmon2` or any hwmon number
- Using `std::process::Command` instead of `tokio::process::Command`
- Creating a data struct without `Serialize` derive
- Reading sysfs directly instead of through utility functions
- Putting tests in a separate `tests/unit/` directory instead of inline modules

## Project Structure & Boundaries

### Complete Project Directory Structure

```
pitop/
├── Cargo.toml                          # Dependencies, build config, metadata
├── build.rs                            # Shell completion generation (clap_complete)
├── README.md
├── LICENSE
├── .github/
│   └── workflows/
│       ├── ci.yml                      # clippy + test on x86
│       └── release.yml                 # Cross-compile + GitHub Release on tag
├── .cargo/
│   └── config.toml                     # Cross-compilation linker configs
├── config/
│   └── default.toml                    # Color threshold values (green/yellow/red)
├── scripts/
│   ├── install.sh                      # curl one-liner installer
│   └── capture-sysfs.sh               # Capture fixture data from real Pi
├── src/
│   ├── main.rs                         # Entry point: arg parsing, terminal init, mode dispatch
│   ├── app.rs                          # App struct, tick handler, collector orchestration
│   ├── event.rs                        # Keyboard/resize event handling, key mapping
│   ├── board/
│   │   ├── mod.rs                      # BoardType enum, detect(), BoardProfile trait
│   │   ├── pi5.rs                      # Pi5Profile: all capabilities enabled
│   │   ├── pi4b.rs                     # Pi4BProfile: PoE + voltages, no PMIC/fan/PCIe
│   │   ├── zero2w.rs                   # Zero2WProfile: minimal capabilities
│   │   └── unknown.rs                  # UnknownProfile: generic Linux only
│   ├── collectors/
│   │   ├── mod.rs                      # Collector trait definition, scheduling logic
│   │   ├── cpu.rs                      # /proc/stat + cpufreq parsing
│   │   ├── memory.rs                   # /proc/meminfo parsing
│   │   ├── thermal.rs                  # thermal_zone + hwmon enumeration
│   │   ├── network.rs                  # /proc/net/dev parsing, rate computation
│   │   ├── disk.rs                     # /proc/diskstats + statvfs
│   │   ├── process.rs                  # /proc/[pid]/ scanning
│   │   ├── throttle.rs                 # vcgencmd get_throttled decoding
│   │   ├── power.rs                    # vcgencmd pmic_read_adc + measure_volts
│   │   ├── fan.rs                      # cooling_fan hwmon (Pi 5)
│   │   ├── pcie.rs                     # /sys/bus/pci/devices/* parsing
│   │   └── poe.rs                      # /sys/class/power_supply/rpi-poe*
│   ├── ui/
│   │   ├── mod.rs                      # Tab routing, layout framework, render dispatch
│   │   ├── header.rs                   # Top bar: board name, time, throttle status
│   │   ├── overview.rs                 # Tab 1: gauges, sparklines, summary
│   │   ├── processes.rs                # Tab 2: sortable process table
│   │   ├── power.rs                    # Tab 3: PMIC/voltages/PCIe/PoE
│   │   ├── network.rs                  # Tab 4: per-interface detail
│   │   ├── disk.rs                     # Tab 5: partitions + I/O
│   │   ├── system.rs                   # Tab 6: board info, kernel, uptime
│   │   ├── help.rs                     # Help overlay (? key)
│   │   └── widgets/
│   │       ├── gauge_bar.rs            # Color-coded gauge with thresholds
│   │       └── sparkline.rs            # Ring-buffer backed sparkline widget
│   └── util/
│       ├── ring_buffer.rs              # Fixed-size circular buffer (60 samples)
│       ├── format.rs                   # Human-readable bytes, temps, watts, durations
│       ├── vcgencmd.rs                 # Async subprocess wrapper with caching
│       └── sysfs.rs                    # read_sysfs_*, discover_hwmon helpers
├── tests/
│   ├── fixtures/
│   │   ├── pi5/                        # Captured from real Pi 5
│   │   ├── pi4b/                       # Captured from real Pi 4B
│   │   └── zero2w/                     # Captured from real Zero 2W
│   └── integration/
│       └── board_detection.rs          # End-to-end board detection tests
└── completions/                        # Generated at build time
    ├── pitop.bash
    ├── pitop.zsh
    ├── pitop.fish
    └── _pitop.ps1
```

### Architectural Boundaries

**Data Flow:**
```
sysfs/procfs files ──→ util/sysfs.rs ──→ collectors/*.rs ──→ App struct ──→ ui/*.rs ──→ terminal
vcgencmd binary ──→ util/vcgencmd.rs ──→ collectors/*.rs ──↗
                                                    ↘ serde_json ──→ stdout (--json mode)
```

**Module Boundaries:**
- `collectors/` modules never import from `ui/` — data flows one direction
- `ui/` modules never call sysfs/procfs directly — they read from `App` state
- `board/` modules never import from `collectors/` — they return capability flags, not collector instances
- `util/` modules are leaf dependencies — they import nothing from the project

### Requirements to Structure Mapping

| FR Category | Source Files | Test Fixtures |
|-------------|-------------|---------------|
| FR1-5: Board Detection | `board/mod.rs`, `board/pi5.rs`, `pi4b.rs`, `zero2w.rs`, `unknown.rs` | `fixtures/*/proc-device-tree-compatible` |
| FR6-7: CPU | `collectors/cpu.rs`, `ui/overview.rs` | `fixtures/*/proc-stat`, `fixtures/*/cpufreq` |
| FR8-9: Memory/Swap | `collectors/memory.rs`, `ui/overview.rs` | `fixtures/*/proc-meminfo` |
| FR10: Load Average | `collectors/cpu.rs` (or inline in `app.rs`) | `fixtures/*/proc-loadavg` |
| FR11-12: Thermal | `collectors/thermal.rs`, `ui/overview.rs` | `fixtures/*/thermal_zone*` |
| FR13: Throttle | `collectors/throttle.rs`, `ui/header.rs` | N/A (vcgencmd mock) |
| FR14-15: Sparklines | `util/ring_buffer.rs`, `ui/widgets/sparkline.rs` | N/A (unit tests) |
| FR16-19: Processes | `collectors/process.rs`, `ui/processes.rs` | `fixtures/*/proc-pid/` |
| FR20-23: PMIC/Power | `collectors/power.rs`, `ui/power.rs` | N/A (vcgencmd mock) |
| FR24: Voltages | `collectors/power.rs`, `ui/power.rs` | N/A (vcgencmd mock) |
| FR25: PoE | `collectors/poe.rs`, `ui/power.rs` | `fixtures/*/power_supply/` |
| FR26: PCIe | `collectors/pcie.rs`, `ui/power.rs` | `fixtures/pi5/pci-devices/` |
| FR27: Fan | `collectors/fan.rs`, `ui/overview.rs` | `fixtures/pi5/hwmon-fan/` |
| FR28-29: Network | `collectors/network.rs`, `ui/network.rs` | `fixtures/*/proc-net-dev` |
| FR30-31: Disk | `collectors/disk.rs`, `ui/disk.rs` | `fixtures/*/proc-diskstats` |
| FR32-35: System Info | `board/mod.rs`, `ui/system.rs` | `fixtures/*/base-model` |
| FR36-42: UI/Nav | `event.rs`, `ui/mod.rs`, `ui/help.rs` | N/A (unit tests) |
| FR43-45: JSON/Completion | `main.rs`, `build.rs` | N/A (integration tests) |
| FR46-48: Degradation | All collectors, `util/vcgencmd.rs` | All fixtures |

### Tick Cycle Data Flow

```
Startup:
  main.rs → clap parse args
         → board/mod.rs::detect() → BoardProfile
         → App::new(profile, args) → create collectors based on profile
         → if --json: run_once() → serialize → stdout → exit
         → else: enter_tui() → event loop

Each tick:
  app.rs tick() →
    always: cpu.collect(), memory.collect(), thermal.collect(),
            throttle.collect(), network.collect_summary()
    if active_tab == 2: process.collect()
    if active_tab == 3: power.collect(), pcie.collect(), poe.collect()
    if active_tab == 4: network.collect_detail()
    if active_tab == 5: disk.collect()
    → push values to sparkline ring buffers
    → ui/mod.rs render(app) → ratatui frame
```

## Architecture Validation Results

### Coherence Validation

**Decision Compatibility:** All technology choices work together without conflicts. tokio async + ratatui sync rendering coexist correctly (render on main thread, async for vcgencmd only). serde derives + anyhow error handling are orthogonal concerns. clap derive + clap_complete are the same crate family.

**Pattern Consistency:** snake_case naming throughout (Rust code, JSON output, fixture files). Path injection applied uniformly to all collectors. Option<T> used consistently for board-conditional data. Error handling pattern (Result in collectors, log-and-continue in tick loop) is uniform across all modules.

**Structure Alignment:** One-file-per-collector maps 1:1 to FR categories. Module boundaries enforce one-way data flow. Test fixtures mirror production paths.

### Requirements Coverage

- **48/48 Functional Requirements:** All mapped to specific source files (see Requirements to Structure Mapping table)
- **24/24 Non-Functional Requirements:** All addressed by architectural decisions (lazy tab refresh for performance, fixed-size buffers for stability, path injection for testability, panic hook for terminal safety)

### Minor Gaps Identified (Non-Blocking)

1. **Format utility schema** — `src/util/format.rs` needs: `format_bytes()`, `format_temp()`, `format_watts()`, `format_duration()`. Straightforward helpers.
2. **Color threshold config** — `config/default.toml` schema: temp (green < 60, yellow < 80, red >= 80), CPU and memory percentage thresholds.
3. **Process kill confirmation UX** — FR19 "confirmation step" should render as a bottom-bar prompt ("Kill PID 1234? y/n") replacing the status bar temporarily.

### Architecture Readiness Assessment

**Overall Status:** READY FOR IMPLEMENTATION
**Confidence Level:** High

**Implementation order:**
1. `src/util/sysfs.rs` + `src/util/ring_buffer.rs` (foundation)
2. `src/board/` (detection + profiles)
3. `src/util/vcgencmd.rs` (shared dependency)
4. `src/collectors/cpu.rs` + `memory.rs` + `thermal.rs` (first collectors)
5. `src/app.rs` + `src/event.rs` + `src/main.rs` (event loop)
6. `src/ui/overview.rs` (first visible output)
7. Remaining collectors and tabs incrementally
