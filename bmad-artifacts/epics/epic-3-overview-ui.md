# Epic 3: TUI Framework and Overview Tab

## Goal
Set up the ratatui application loop, tab navigation, and the
main overview dashboard tab.

---

## Story 3.1: Application scaffold and event loop

### Acceptance criteria
- [ ] Initializes terminal with crossterm (alternate screen, raw mode)
- [ ] Tokio-based tick loop at configurable interval (default 1000ms)
- [ ] Keyboard event handling: q to quit, 1-6 for tabs, Tab/Shift+Tab, Space to pause
- [ ] Graceful cleanup on quit (restore terminal state)
- [ ] Panic hook that restores terminal before printing error
- [ ] `App` struct holds all state: active tab, paused flag, board profile, collectors, history buffers

### Files: `src/main.rs`, `src/app.rs`, `src/event.rs`

---

## Story 3.2: Tab navigation framework

### Acceptance criteria
- [ ] Tab bar rendered at top showing all 6 tabs with active highlight
- [ ] Tabs: Overview, Processes, Power, Network, Disk, System
- [ ] Number keys 1-6 switch directly, Tab/Shift+Tab cycle
- [ ] Active tab content rendered below the tab bar
- [ ] Inactive tab collectors are not refreshed (lazy refresh)
- [ ] Footer bar shows keyboard hints

### Files: `src/ui/mod.rs`, `src/ui/header.rs`

---

## Story 3.3: Overview tab

### Acceptance criteria
- [ ] Header: board name, SoC, uptime, current time
- [ ] CPU section: per-core horizontal gauges (color-coded: green < 60%, yellow < 85%, red >= 85%)
- [ ] CPU aggregate sparkline (60-sample history)
- [ ] Memory gauge + sparkline with used/total label
- [ ] Temperature reading (color-coded: green < 60°C, yellow < 70°C, red >= 70°C)
- [ ] Throttle status: single-line indicator (✓ OK or ⚠ flags)
- [ ] Network: aggregate rx/tx rates
- [ ] Fan section (Pi 5 only): RPM + PWM%
- [ ] Responsive layout: works at 80×24 minimum, uses extra space when available

### Files: `src/ui/overview.rs`, `src/ui/widgets/gauge_bar.rs`, `src/ui/widgets/sparkline.rs`

---

## Story 3.4: Human-readable formatting utilities

### Acceptance criteria
- [ ] `format_bytes(bytes: u64) -> String` — "1.2 GiB", "456 MiB", "789 KiB"
- [ ] `format_bytes_rate(bytes_per_sec: f64) -> String` — "1.2 MB/s"
- [ ] `format_temp(celsius: f64) -> String` — "45.2°C"
- [ ] `format_watts(watts: f64) -> String` — "3.42 W"
- [ ] `format_uptime(seconds: u64) -> String` — "3d 14h 22m"
- [ ] `format_freq(khz: u64) -> String` — "2.4 GHz", "600 MHz"
- [ ] Unit tests for all formatters including edge cases

### Files: `src/util/format.rs`
