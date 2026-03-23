---
stepsCompleted: [step-01-init, step-02-discovery, step-02b-vision, step-02c-executive-summary, step-03-success, step-04-journeys, step-05-domain-skipped, step-06-innovation-skipped, step-07-project-type, step-08-scoping, step-09-functional, step-10-nonfunctional, step-11-polish]
inputDocuments: [bmad-artifacts/product-brief.md, docs/design-research.md]
workflowType: 'prd'
documentCounts:
  briefs: 1
  research: 1
  brainstorming: 0
  projectDocs: 0
classification:
  projectType: cli_tool
  domain: developer_hardware_tooling
  complexity: low
  projectContext: greenfield
---

# Product Requirements Document - pitop

**Author:** Hongjunwu
**Date:** 2026-03-22

## Executive Summary

pitop is a terminal-based system monitor purpose-built for Raspberry Pi. It ships as a single static binary with zero runtime dependencies that surfaces board-specific telemetry — PMIC power rails, fan speed, PCIe link negotiation, PoE HAT status — that generic Linux monitors like `htop` cannot access. It targets Pi homelab operators managing headless servers over SSH, IoT developers diagnosing thermal and power issues during development, and hobbyists who want a single tool to replace ad-hoc `vcgencmd` scripts.

pitop detects the board at startup via `/proc/device-tree/compatible` and activates only the collectors relevant to that hardware. Three boards are supported in v1: Raspberry Pi 5 (BCM2712, full feature set), Pi 4 Model B (BCM2711, core monitoring + voltages + PoE), and Pi Zero 2 W (BCM2710A1, lightweight core monitoring). On non-Pi Linux systems, generic CPU/memory/disk/network/process collectors run normally, enabling x86 development and CI testing.

All system data is read directly from procfs/sysfs — no abstraction libraries. Board-specific data unavailable via sysfs (PMIC power rails, throttle state, voltages) is collected through `vcgencmd` subprocess calls with async execution, 2-second timeouts, and 1-second caching. If `vcgencmd` is unavailable for any reason, those features silently degrade.

### What Makes This Special

**Board-aware intelligence.** pitop is not a generic Linux monitor with Pi branding. It detects your specific board and adapts: a Pi 5 user sees 12 PMIC power rails, fan RPM, PCIe Gen 2/3 link status, and RP1 southbridge thermals. A Zero 2W user gets a minimal view tuned for 512MB RAM and 1GHz cores. No other tool does this.

**Gap in the ecosystem.** The original Go-based pitop (PierreKieffer/pitop) validated demand for a Pi-specific monitor, then was archived in October 2025. The Pi 5's introduction of PMIC, fan header, RP1 southbridge, and external PCIe created hardware complexity that no existing tool covers. Python alternatives like pi_dashboard use psutil (~15MB RSS), making them impractical on the Zero 2W. A Rust implementation hits the performance ceiling needed: under 5MB binary, under 10MB RSS, under 2% CPU at 1-second refresh.

**Single binary, zero friction.** Install via `curl` one-liner or `cargo install pitop`. No Python interpreter, no pip dependencies, no Go runtime. One binary that works on all three boards.

## Project Classification

- **Project Type:** CLI Tool — terminal-based TUI application with keyboard-driven interaction
- **Domain:** Developer/Hardware Tooling — Raspberry Pi ecosystem monitoring
- **Complexity:** Low — standard software practices, no regulatory or compliance requirements
- **Project Context:** Greenfield — new Rust implementation, no existing codebase

## Success Criteria

### User Success

- **Instant board recognition:** User launches pitop and immediately sees their specific board identified (e.g., "Raspberry Pi 5 Model B Rev 1.0") with the correct feature set activated — no configuration needed.
- **Single-pane-of-glass:** User replaces their workflow of `htop` + `vcgencmd measure_temp` + `vcgencmd pmic_read_adc` + `cat /sys/class/thermal/...` with one tool that shows everything.
- **SSH-first experience:** User SSHs into a headless Pi, runs `pitop`, and gets a fully functional TUI on an 80x24 terminal. No X11 forwarding, no web browser, no port tunneling.
- **"Aha" moment:** Pi 5 user sees per-rail PMIC power breakdown with total wattage estimate — data they've never seen presented this way outside of raw `vcgencmd` output.

### Business Success

- **Open-source adoption:** 500+ GitHub stars within 6 months of release (the archived Go pitop reached ~200 stars with less functionality).
- **Community validation:** Issues and PRs from real Pi users confirm the tool is used in production homelab and IoT environments.
- **Crate downloads:** Published on crates.io with measurable `cargo install` adoption.
- **Ecosystem recognition:** Mentioned in Raspberry Pi forums, r/raspberry_pi, or Pi-adjacent communities as the recommended monitoring tool.

### Technical Success

- **Binary size:** Under 5MB stripped for both `aarch64` and `armv7` targets.
- **Memory footprint:** Under 10MB RSS on Raspberry Pi Zero 2W (512MB total RAM).
- **CPU overhead:** Under 2% CPU on Zero 2W at default 1-second refresh interval.
- **Board detection accuracy:** 100% correct identification across Pi 5, Pi 4B, and Zero 2W using `/proc/device-tree/compatible`.
- **Data accuracy:** All displayed values match `vcgencmd`, `htop`, `free`, and `df` output on real hardware within rounding tolerance.
- **Graceful degradation:** Zero panics, zero crashes when run on x86 Linux, Docker containers, or Pi boards with missing hardware features.
- **CI green:** `cargo clippy` zero warnings, `cargo test` passes on x86 GitHub Actions runners.

### Measurable Outcomes

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Board detection | 3/3 boards correct | Test on real hardware |
| Binary size (aarch64) | < 5MB | `ls -la` on stripped release binary |
| RSS on Zero 2W | < 10MB | `ps aux` after 60 seconds of running |
| CPU on Zero 2W | < 2% | `top` observation over 60 seconds |
| PMIC data accuracy | ±1% vs vcgencmd | Side-by-side comparison |
| Startup time | < 500ms to first render | Wall clock measurement |
| Crash rate | 0 crashes in 24hr run | Endurance test on each board |

## User Journeys

### Journey 1: Marcus — Homelab Operator (Happy Path)

Marcus runs three Pi 4Bs as a home Kubernetes cluster — PiHole, Home Assistant, and a media server. He's SSH'd into his PiHole node because DNS resolution felt sluggish this morning.

**Opening Scene:** Marcus types `ssh pi@pihole.local` then runs `pitop`. The header bar immediately shows "Raspberry Pi 4 Model B Rev 1.4 — BCM2711" and a green throttle indicator. He's on the Overview tab.

**Rising Action:** CPU gauges show all four cores under 15% — that's normal. But the memory bar is at 87% and yellow. He glances at the sparkline — memory has been climbing steadily. He hits `2` to jump to the Processes tab. Sorts by MEM% with `s`. The `pihole-FTL` process is at 340MB — that's double what it usually runs.

**Climax:** He spots the problem immediately. The FTL database has grown and PiHole is caching more than expected. He doesn't need to kill anything — he just needs to restart the service. But he also notices the temperature gauge is at 62°C with a yellow warning. The throttle indicator still shows green (no throttling yet), but the sparkline shows temp has been rising over the last minute.

**Resolution:** Marcus restarts the PiHole service in another terminal, watches the memory bar drop back to 45% in pitop. Temperature starts falling. He leaves pitop running in a tmux pane so he can check back later. Total diagnosis time: 90 seconds.

**Requirements revealed:** Per-core CPU gauges, memory bar with sparkline history, process table sortable by MEM%, color-coded temperature thresholds, throttle indicator, tab switching.

### Journey 2: Priya — IoT Developer (Pi 5 Power Analysis)

Priya is developing a battery-monitoring IoT gateway on a Pi 5 with an NVMe SSD. She needs to know total system power draw to size her PoE power supply.

**Opening Scene:** Priya runs `pitop` on her Pi 5 dev bench. The header shows "Raspberry Pi 5 Model B" and she sees the Overview tab with fan RPM at 2800 RPM (45% PWM) — the board is warm from her test workload.

**Rising Action:** She hits `3` to go to the Power tab. She sees all 12 PMIC power rails — VDD_CORE at 0.88V/1.2A, 3V3 at 3.29V/0.4A, and so on. The total estimated wattage reads 6.8W with a sparkline showing it spiked to 9.2W during her load test. Below the PMIC table, she sees EXT5V_V at 5.12V (USB-C PSU) and BATT_V at 3.05V (her RTC coin cell is fine).

**Climax:** She scrolls down to the PCIe section: "NVMe SSD — Gen 3 x1 (8.0 GT/s)". She had been worried the NVMe was negotiating Gen 2 — it's not. Then she checks RP1 ADC: USB VBus is reading 4.95V, confirming her peripherals aren't sagging the rail. Total system draw is ~7W idle, ~10W under load. Her PoE+ supply (25.5W budget) has plenty of headroom.

**Resolution:** Priya now has hard numbers for her project's power budget document. She didn't need to run `vcgencmd pmic_read_adc` and manually parse 24 lines of output, or separately check PCIe link speed, or manually calculate total wattage. One tool, one screen.

**Requirements revealed:** PMIC rail table with per-rail V/A/W, total wattage estimate with sparkline, EXT5V_V and BATT_V display, PCIe link speed/width/device name, fan RPM/PWM on overview, RP1 ADC voltages.

### Journey 3: Tom — Hobbyist (Zero 2W, Degraded Hardware)

Tom is running a weather station on a Pi Zero 2W in his garden shed. It's connected via WiFi and he hasn't checked on it in weeks.

**Opening Scene:** Tom SSHs in over a flaky WiFi connection. His terminal is 80x24. He runs `pitop` and the header shows "Raspberry Pi Zero 2 W — BCM2710A1". The overview is compact — no fan section, no PCIe, no PMIC. Just the essentials.

**Rising Action:** CPU is at 8% — the Python weather script is behaving. But the temperature gauge is red at 72°C. The throttle indicator is yellow: "Freq capped" and "Under-voltage has occurred (since boot)". Tom notices the swap bar — 60% used. With only 512MB RAM, the system is swapping.

**Climax:** Tom hits `3` for Power tab. He sees core voltage at 1.20V — normal. But the throttle history shows under-voltage has occurred since boot. His micro-USB power supply in the shed is probably marginal. The throttle is capping his CPU frequency from 1.0GHz down to 600MHz.

**Resolution:** Tom now knows he needs a better power supply and maybe a heatsink. He didn't need to remember the `vcgencmd get_throttled` hex bitmask format — pitop decoded it into plain English. He also sees that RSS for pitop itself is 6MB — barely a dent on his constrained system.

**Requirements revealed:** Graceful degradation (hide Pi 5 features on Zero 2W), compact layout for 80x24, swap bar, throttle bitmask decoding with human-readable labels, color-coded severity, low memory footprint, core voltage display via vcgencmd.

### Journey 4: Dev on x86 Laptop (Development/CI Path)

A contributor clones the pitop repo on their Ubuntu x86 laptop to fix a bug in the process sorting logic.

**Opening Scene:** They run `cargo run` on their laptop. pitop starts, the header shows "Unknown Board — Generic Linux". No board-specific features appear. The Power tab shows "No board-specific power data available."

**Rising Action:** The Overview tab works — CPU gauges, memory, swap, load average, temperature (from x86 thermal zones), network throughput. They navigate to the Processes tab, reproduce the sorting bug, fix it, and verify.

**Climax:** They run `cargo test` — all tests pass, including board detection tests that use fixture files from `tests/fixtures/`. `cargo clippy` is clean. They didn't need a Raspberry Pi to develop or test the fix.

**Resolution:** The contributor submits a PR with confidence that the fix works. The GitHub Actions CI runner (also x86) confirms all tests pass. The maintainer merges and cross-compiles for aarch64/armv7.

**Requirements revealed:** x86 graceful degradation, Unknown board fallback, generic Linux collectors, fixture-based unit tests, CI compatibility.

### Journey Requirements Summary

| Capability | Marcus (Homelab) | Priya (IoT Dev) | Tom (Hobbyist) | Dev (x86) |
|------------|:---:|:---:|:---:|:---:|
| Board detection + auto-config | x | x | x | x |
| CPU gauges + sparklines | x | x | x | x |
| Memory + swap bars | x | | x | x |
| Temperature with color thresholds | x | x | x | x |
| Throttle status decoding | x | | x | |
| Process table (sortable, killable) | x | | | x |
| PMIC power rails (Pi 5) | | x | | |
| PCIe link info (Pi 5) | | x | | |
| Fan RPM/PWM (Pi 5) | | x | | |
| RP1 ADC voltages (Pi 5) | | x | | |
| Core/SDRAM voltages (Pi 4B/Zero) | | | x | |
| Compact 80x24 layout | | | x | |
| x86 Unknown board fallback | | | | x |
| Tab navigation | x | x | x | x |
| Pause/resume (spacebar) | | x | | |

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

**MVP Approach:** Problem-solving MVP — deliver the minimum that makes a Pi user say "I don't need htop + vcgencmd anymore." The MVP validates two hypotheses: (1) Pi users want a single monitoring tool, and (2) board-aware adaptation is the differentiator that earns adoption over generic alternatives.

**Resource Requirements:** Solo developer, cross-compiling from x86. No CI pipeline needed for MVP — manual builds on real hardware for validation.

### MVP Feature Set (Phase 1)

**Core User Journeys Supported:**
- Marcus (homelab) — full journey: overview + process diagnosis
- Tom (Zero 2W) — partial: overview + throttle status, but no Power tab yet
- Dev (x86) — full: generic collectors work, fixture tests pass

**Must-Have Capabilities:**

| # | Capability | Rationale |
|---|-----------|-----------|
| 1 | Board detection (Pi 5, Pi 4B, Zero 2W, Unknown) | Foundation — everything else depends on this |
| 2 | CPU collector (per-core usage, frequency, governor) | Core monitoring value |
| 3 | Memory + swap collector | Core monitoring value |
| 4 | Thermal collector (SoC temp, hwmon discovery) | Critical for Pi users — thermal throttling is the #1 concern |
| 5 | Throttle status collector (vcgencmd get_throttled) | Decodes the hex bitmask users can never remember |
| 6 | Network collector (per-interface rx/tx rates) | Overview needs aggregate throughput |
| 7 | Process collector (PID, name, CPU%, MEM%, user) | Enables diagnosis workflows like Marcus's journey |
| 8 | Overview tab with gauges, sparklines, color thresholds | The "single pane of glass" experience |
| 9 | Process tab with sorting and kill | The diagnostic workflow |
| 10 | Ring buffer for sparkline history | Required by overview tab |
| 11 | vcgencmd async wrapper with caching/timeout | Required by throttle, thermal |
| 12 | sysfs/procfs read utilities | Foundation for all collectors |
| 13 | Keyboard nav (1-6 tabs, j/k, q, spacebar pause) | Basic interactivity |
| 14 | CLI args (--interval, --tab, --board, --verbose, --json) | User control + scripting |
| 15 | JSON snapshot mode (--json) | Scripting/automation support |
| 16 | Color-coded thresholds (green/yellow/red) | Core UX — immediate visual feedback |
| 17 | Graceful degradation on x86 / missing hardware | Development and CI support |

### Post-MVP Features

**Phase 2 — Full Monitoring (Growth):**

| # | Capability | Depends On |
|---|-----------|------------|
| 18 | Power tab: PMIC rails (Pi 5), voltages (Pi 4B) | vcgencmd wrapper |
| 19 | PCIe link info (Pi 5) | Board detection |
| 20 | PoE HAT detection (Pi 5, Pi 4B) | sysfs utilities |
| 21 | Fan monitoring (Pi 5) | hwmon discovery |
| 22 | RP1 ADC voltages (Pi 5) | hwmon discovery |
| 23 | Network tab: per-interface detail + sparklines | Network collector |
| 24 | Disk tab: partition usage + I/O rates | New disk collector |
| 25 | System info tab: board, kernel, uptime, CPU details | Board detection + sysfs |
| 26 | Help overlay (?) | UI framework |
| 27 | Shell completion (bash/zsh/fish) | clap_complete |
| 28 | Cross-compilation CI + prebuilt releases | GitHub Actions |
| 29 | `cargo install pitop` via crates.io | Publish crate |

This phase completes Priya's journey (power analysis) and Tom's full journey (power tab with voltages).

**Phase 3 — Polish & Ecosystem (Vision / v2+):**

- Configurable color themes
- Config file for custom thresholds (`~/.config/pitop/config.toml`)
- PCIe AER error counts
- GPIO pin monitoring
- Stress testing mode
- Threshold hooks (run shell command on trigger)
- `--once` human-readable snapshot mode
- `--watch --format csv` streaming output
- AUR / deb / RPM packaging

### Risk Mitigation Strategy

**Technical Risks:**
- *vcgencmd unavailability:* Mitigated by design — silent degradation, all vcgencmd features are optional. MVP works without it.
- *hwmon numbering instability:* Mitigated by architecture rule — always discover by name, never hardcode numbers.
- *Cross-compilation breakage:* Deferred to Phase 2. MVP validated on real hardware with manual builds.

**Market Risks:**
- *"Who needs this when htop exists?"* — The board-aware features (PMIC, throttle decoding, PCIe) are the answer. MVP includes throttle decoding; Phase 2 adds the full power story.
- *Low adoption:* Mitigated by publishing to crates.io and including a curl one-liner. Friction is near-zero.

**Resource Risks:**
- *Solo developer:* The architecture is modular (collector trait pattern). Each collector is independent and can be built/tested in isolation. If time is tight, Phase 2 features can ship incrementally.
- *No real Pi hardware for testing:* x86 graceful degradation + fixture-based tests mean 90% of development happens on a laptop. Hardware validation is the final step per phase.

## CLI Tool Specific Requirements

### Command Structure

pitop operates in two modes:

**Interactive mode (default):** Full-screen TUI with tabbed interface, live-updating gauges, keyboard navigation. This is the primary user experience.

**Snapshot mode (`--json`):** Dumps a single collection pass of all active collectors as a JSON object to stdout, then exits. No TUI rendering. Designed for scripting, automation, and integration with tools like `jq`, cron jobs, and monitoring pipelines. The JSON schema mirrors the internal collector struct hierarchy (e.g., `.thermal.soc_temp`, `.cpu.cores[0].usage_percent`, `.power.pmic_rails[]`).

### CLI Arguments (Complete)

| Argument | Short | Default | Description |
|----------|-------|---------|-------------|
| `--interval` | `-i` | `1000` | Refresh interval in milliseconds (TUI mode) |
| `--tab` | `-t` | `1` | Starting tab number, 1–6 (TUI mode) |
| `--board` | | `auto` | Force board type: `pi5`, `pi4b`, `zero2w`, `auto` |
| `--json` | `-j` | | Single snapshot as JSON to stdout, then exit |
| `--verbose` | `-v` | | Log warnings to stderr (e.g., vcgencmd unavailable) |
| `--version` | `-V` | | Print version and exit |
| `--help` | `-h` | | Print usage and exit |

### Output Formats

- **TUI mode:** ratatui rendered frames via crossterm backend. No stdout output during operation.
- **JSON mode:** Single JSON object to stdout. Machine-parseable. Schema derived from collector data structs via `serde::Serialize`. Exit code 0 on success.

### Shell Completion

Generated via `clap_complete` in the build script for bash, zsh, fish, and PowerShell. Completion files included in release tarballs and installable via the install script. Covers all flags and `--board` enum values.

### Scripting Support

- `--json` output is stable and can be relied upon by downstream scripts
- Exit codes: `0` success, `1` general error, `2` invalid arguments
- stderr used for warnings/errors (never stdout, to avoid corrupting JSON output)
- No interactive prompts or confirmations in `--json` mode (kill confirmation is TUI-only)

### Implementation Considerations

- `serde` and `serde_json` are required dependencies for `--json` mode. All collector data structs must derive `Serialize`.
- JSON schema should be documented in the README with example output for each board type.
- `--json` and `--tab`/`--interval` are mutually exclusive (json mode does one pass and exits).
- Shell completion generation happens at build time, not runtime. The `clap_complete` crate is a build dependency only.

## Functional Requirements

### Board Detection & Hardware Profiling

- **FR1:** The system can identify the Raspberry Pi board model at startup by reading the device-tree compatible string
- **FR2:** The system can fall back to reading the device-tree model file for human-readable board name when available
- **FR3:** The system can activate board-specific collectors based on detected hardware capabilities
- **FR4:** The system can operate in a generic Linux mode when no supported board is detected
- **FR5:** Users can override automatic board detection via CLI argument

### System Monitoring

- **FR6:** Users can view per-core CPU usage percentages updated at the configured refresh interval
- **FR7:** Users can view current CPU frequency, min/max frequency range, and active governor
- **FR8:** Users can view total and per-category memory usage (used, free, available, buffers, cached)
- **FR9:** Users can view swap usage
- **FR10:** Users can view system load averages (1, 5, 15 minute)
- **FR11:** Users can view SoC temperature with color-coded severity thresholds
- **FR12:** Users can view additional thermal zones when available (PMIC temp, RP1 temp on Pi 5)
- **FR13:** Users can view decoded throttle status showing current state and since-boot history in human-readable labels
- **FR14:** Users can view aggregate network throughput (all interfaces summed) on the overview
- **FR15:** Users can view sparkline history (last 60 samples) for CPU usage, memory, and temperature

### Process Management

- **FR16:** Users can view a list of running processes with PID, name, CPU%, MEM%, and user
- **FR17:** Users can sort the process list by any displayed column
- **FR18:** Users can navigate the process list using keyboard controls
- **FR19:** Users can send a kill signal to a selected process with a confirmation step

### Power & Hardware Telemetry

- **FR20:** Users can view per-rail PMIC voltage, current, and calculated power on Pi 5
- **FR21:** Users can view estimated total system wattage with sparkline history on Pi 5
- **FR22:** Users can view EXT5V_V (input voltage) and BATT_V (RTC battery) readings on Pi 5
- **FR23:** Users can view RP1 ADC voltages including USB VBus on Pi 5
- **FR24:** Users can view core and SDRAM voltages on Pi 4B and Zero 2W
- **FR25:** Users can view PoE HAT online status and current draw when a PoE HAT is detected
- **FR26:** Users can view PCIe link speed (with generation label), width, and connected device name on Pi 5
- **FR27:** Users can view fan speed (RPM) and PWM duty cycle percentage on Pi 5

### Network Monitoring

- **FR28:** Users can view per-interface status (up/down), IP addresses, and real-time rx/tx throughput
- **FR29:** Users can view per-interface throughput sparklines

### Disk Monitoring

- **FR30:** Users can view mounted partitions with device, mountpoint, total/used/free space, and usage percentage
- **FR31:** Users can view per-disk I/O rates (read/write)

### System Information

- **FR32:** Users can view board model, revision, and SoC name
- **FR33:** Users can view kernel version and OS information
- **FR34:** Users can view system uptime
- **FR35:** Users can view CPU architecture, model name, and frequency range

### User Interface & Navigation

- **FR36:** Users can switch between tabs using number keys (1–6) or Tab/Shift+Tab
- **FR37:** Users can pause and resume live data updates
- **FR38:** Users can view a help overlay showing available keyboard shortcuts
- **FR39:** Users can quit the application via keyboard shortcut
- **FR40:** The system can render a functional layout on terminals as small as 80x24
- **FR41:** The system can display color-coded thresholds (green/yellow/red) for temperature, CPU usage, and memory usage
- **FR42:** The system can hide board-specific UI sections that are not available on the detected hardware

### Data Export & Scripting

- **FR43:** Users can export a single snapshot of all collected metrics as JSON to stdout
- **FR44:** The system can produce stable, machine-parseable JSON output suitable for piping to other tools
- **FR45:** The system can provide shell completion for all CLI arguments in bash, zsh, fish, and PowerShell

### Graceful Degradation

- **FR46:** The system can continue operating when vcgencmd is unavailable, with affected features hidden or showing unavailable status
- **FR47:** The system can continue operating when specific sysfs/procfs paths are missing (board lacks the hardware)
- **FR48:** The system can log degradation warnings to stderr when verbose mode is enabled

## Non-Functional Requirements

### Performance

- **NFR1:** CPU overhead must not exceed 2% on Raspberry Pi Zero 2W at default 1-second refresh interval, measured via `top` over 60 seconds of operation
- **NFR2:** RSS memory usage must remain under 10MB on Raspberry Pi Zero 2W after 60 seconds of operation
- **NFR3:** Stripped release binary must be under 5MB for both `aarch64-unknown-linux-gnu` and `armv7-unknown-linux-gnueabihf` targets
- **NFR4:** Time from launch to first rendered frame must be under 500ms on all supported boards
- **NFR5:** Each collector tick must complete within the configured refresh interval — if a collector exceeds the interval, it must not block other collectors or UI rendering
- **NFR6:** vcgencmd subprocess calls must timeout after 2 seconds and cache results for a minimum of 1 second to avoid hammering the firmware mailbox
- **NFR7:** JSON snapshot mode (`--json`) must complete and exit within 3 seconds on all supported boards

### Reliability

- **NFR8:** The application must run continuously for 24 hours without crashes or memory leaks on each supported board
- **NFR9:** No `unwrap()` or `expect()` calls in any production code path — all errors handled via `anyhow::Result` or `Option`
- **NFR10:** Terminal state must be properly restored on all exit paths, including panics (panic hook must restore terminal)
- **NFR11:** Ctrl+C must be handled gracefully with clean shutdown and terminal restoration
- **NFR12:** Missing or inaccessible sysfs/procfs paths must never cause a crash — the feature is silently unavailable
- **NFR13:** Process disappearance between reads (ENOENT on `/proc/[pid]/`) must be handled without error
- **NFR14:** `cargo clippy` must pass with zero warnings on every commit

### Portability & Compatibility

- **NFR15:** Must compile and run on `aarch64-unknown-linux-gnu` (Pi 5, Pi 4B 64-bit, Zero 2W 64-bit)
- **NFR16:** Must compile and run on `armv7-unknown-linux-gnueabihf` (Pi 4B 32-bit, Zero 2W 32-bit)
- **NFR17:** Must compile and run on `x86_64-unknown-linux-gnu` for development and CI testing
- **NFR18:** TUI must render correctly on terminal sizes from 80x24 (minimum) up to arbitrary larger sizes
- **NFR19:** Must work correctly over SSH connections with no X11 forwarding required
- **NFR20:** hwmon device numbers must never be hardcoded — always discovered by enumerating `/sys/class/hwmon/` and matching the `name` file

### Build & Distribution

- **NFR21:** `cargo fmt` must be applied before every commit
- **NFR22:** All unit tests must pass on x86 GitHub Actions runners using fixture files (no real Pi hardware required for CI)
- **NFR23:** Release binaries must be cross-compiled from an x86 host using `cross` or equivalent Docker-based toolchain
- **NFR24:** Shell completion files for bash, zsh, fish, and PowerShell must be generated at build time and included in release artifacts
