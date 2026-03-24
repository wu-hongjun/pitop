# Epic 8: Feature Polish

## Goal
Improve the five features introduced in Epic 7 — color themes, configuration,
help overlay, GPU monitoring, and stress testing — with better usability,
theme integration, runtime controls, and richer data display.

---

## Story 8.1: Theme-aware UI — eliminate all hardcoded colors

### Problem
The `Theme` struct exists and is stored on `App`, but most UI modules still
hardcode `Color::*` values (e.g. `Color::Blue` in `overview.rs`, `Color::Yellow`
in `header.rs`, `Color::Cyan` in `help.rs`). Switching themes only affects
modules that explicitly read `app.theme`.

### Acceptance criteria
- [ ] `overview.rs`: replace every `Color::*` with the corresponding `app.theme.*` field
      (e.g. `Color::Blue` → `app.theme.cpu_border`, `Color::Magenta` → `app.theme.mem_border`)
- [ ] `header.rs`: use `app.theme.throttle_ok/warn/crit` for throttle indicator;
      use `app.theme.title` for board name, `app.theme.text` for uptime
- [ ] `help.rs`: accept `&Theme` parameter; use `theme.title` for headings,
      `theme.border_highlight` for key labels, `theme.text` for descriptions,
      `theme.border` for the popup border
- [ ] `processes.rs`: use theme colors for table headers, selected row highlight,
      alternating row tints
- [ ] `power.rs`, `network.rs`, `disk.rs`, `system.rs`: use theme colors for
      all borders, labels, and value text
- [ ] `mod.rs` (tab bar & footer): use `app.theme.title` for active tab,
      `app.theme.text_dim` for inactive tabs, `app.theme.gauge_crit` for
      stress indicator
- [ ] `percent_color()` and `temp_color()` helpers accept `&Theme` and read
      `theme.gauge_low/warn/crit` instead of hardcoded Green/Yellow/Red
- [ ] All three built-in themes (default, monochrome, solarized) produce a
      visually consistent result across every tab — verify by inspection

### Files
`src/ui/overview.rs`, `src/ui/header.rs`, `src/ui/help.rs`, `src/ui/mod.rs`,
`src/ui/processes.rs`, `src/ui/power.rs`, `src/ui/network.rs`, `src/ui/disk.rs`,
`src/ui/system.rs`

---

## Story 8.2: Runtime theme cycling and custom theme support

### Problem
The theme can only be selected at startup via `--theme` or the config file.
There is no way to cycle themes at runtime, and users cannot define custom
color schemes.

### Acceptance criteria
- [ ] Press `t` to cycle through available themes at runtime (default → monochrome
      → solarized → default …)
- [ ] The footer hint bar shows `t:Theme` alongside the other shortcuts
- [ ] Help overlay lists the `t` key under a "General" section
- [ ] Config file supports a `[theme.custom]` section with the same fields as the
      `Theme` struct, using standard color names or `#RRGGBB` hex values
- [ ] When a custom theme is defined in the config, it is added to the cycle
      (default → monochrome → solarized → custom → default …)
- [ ] `Theme::from_name("custom")` returns the config-defined theme or `None`
- [ ] Add unit test: parse a TOML string with `[theme.custom]` and verify fields

### Files
`src/ui/theme.rs`, `src/config.rs`, `src/event.rs`, `src/app.rs`,
`src/ui/mod.rs`, `src/ui/help.rs`

---

## Story 8.3: Enhanced configuration — generate, validate, and document

### Problem
Users have no way to discover available config options or generate a starter
config file. Invalid keys in the TOML are silently ignored. There is no
sample config.

### Acceptance criteria
- [ ] `pitop --generate-config` prints a fully-commented sample `config.toml`
      to stdout with every option, default value, and a short description
- [ ] The sample includes all sections: `[general]`, `[thresholds.*]`,
      `[theme.custom]`
- [ ] `Config::load()` validates field ranges: `interval_ms >= 100`,
      `default_tab` between 1-6, `history_size` between 10-600
- [ ] Out-of-range values produce a clear error: "interval_ms must be >= 100,
      got 50"
- [ ] Add `--config-check` flag that loads the config, prints validation
      results, and exits (useful for debugging config issues)
- [ ] Add integration test: write a TOML with out-of-range `interval_ms`,
      verify error message
- [ ] Update help overlay to mention `~/.config/pitop/config.toml` path

### Files
`src/config.rs`, `src/main.rs`, `src/ui/help.rs`

---

## Story 8.4: Richer help overlay with sections and scroll

### Problem
The help overlay is a fixed-size popup that cannot scroll and uses hardcoded
colors. As more keybindings are added, the content may overflow the visible
area on small terminals.

### Acceptance criteria
- [ ] Help content is organized into sections with bold section headers:
      "Navigation", "Processes", "Power User", "Stress Testing" (if active)
- [ ] "Navigation" section: `1-6`, `Tab/Shift+Tab`, `q/Ctrl+C`
- [ ] "Display" section: `Space` (pause), `t` (theme cycle), `?` (help)
- [ ] "Processes" section: `j/k/arrows`, `s` (sort), `K` (kill)
- [ ] "Stress Testing" section (shown only when stress mode is available):
      `Ctrl+S` (toggle), `Ctrl+↑/↓` (adjust workers)
- [ ] Overlay shows pitop version and board name at the bottom
- [ ] `j`/`k` or arrow keys scroll the help content if it overflows the popup
- [ ] Add `help_scroll: usize` field to `App` for scroll offset
- [ ] Popup size adapts: 70% of terminal height, minimum 15 rows
- [ ] Colors use the active theme

### Files
`src/ui/help.rs`, `src/app.rs`, `src/event.rs`

---

## Story 8.5: GPU monitoring improvements — sparkline and V3D status

### Problem
GPU data is shown as a single line of text in the temperature section. There is
no history sparkline and no per-codec decode/encode status (V3D/H.264/HEVC).

### Acceptance criteria
- [ ] Add `gpu_freq_history: RingBuffer<f64>` to `App` for GPU clock sparkline
- [ ] Overview sparkline row expands to a 2×2 grid when GPU is available:
      top row = CPU + MEM, bottom row = TEMP + GPU
- [ ] GPU sparkline shows frequency history (0–max_freq MHz range)
- [ ] Read V3D codec status via `vcgencmd codec_enabled H264` and
      `vcgencmd codec_enabled HEVC` — store as `Vec<(String, bool)>` on `GpuData`
- [ ] Overview temperature section shows codec status: "H264: ✓  HEVC: ✗"
- [ ] Power tab shows GPU section with frequency, memory, temp, and codecs
- [ ] Gracefully handle missing V3D (Pi Zero 2W has no hardware decode)
- [ ] Add parser tests for `codec_enabled` output formats

### Files
`src/collectors/gpu.rs`, `src/app.rs`, `src/ui/overview.rs`, `src/ui/power.rs`

---

## Story 8.6: Stress test improvements — adjustable workers and progress

### Problem
The stress test launches a fixed number of workers (= CPU cores) and can only
be toggled on/off. There is no way to adjust load, see elapsed time, or
stress a subset of cores.

### Acceptance criteria
- [ ] `--stress-workers N` CLI flag sets initial worker count (default = core count)
- [ ] While stress is running, `Ctrl+Up` adds a worker, `Ctrl+Down` removes one
      (minimum 1, maximum 2× core count)
- [ ] Footer shows worker count and elapsed time: `[STRESS 4/4 workers 02:35]`
- [ ] `StressTest` tracks `started_at: Option<Instant>` for elapsed time
- [ ] `StressTest::set_workers(n)` dynamically adjusts worker count by
      spawning/stopping individual workers
- [ ] Help overlay "Stress Testing" section documents `Ctrl+↑/↓` for worker
      adjustment
- [ ] Overview header shows a colored stress indicator: green when worker
      count < core count, yellow at core count, red above core count
- [ ] Unit tests: `set_workers` increases and decreases worker count correctly

### Files
`src/stress.rs`, `src/main.rs`, `src/event.rs`, `src/app.rs`,
`src/ui/mod.rs`, `src/ui/help.rs`

---

## Dependencies

- Story 8.1 should be completed first (all other stories assume theme-aware UI)
- Stories 8.2–8.6 are independent of each other after 8.1
- Story 8.4 should incorporate keybindings from 8.2 and 8.6 if completed concurrently

## Definition of done

- `cargo clippy` passes with zero warnings
- `cargo fmt` applied
- All new code has unit tests
- No `unwrap()` or `expect()` in production code paths
- All three built-in themes visually verified across tabs
