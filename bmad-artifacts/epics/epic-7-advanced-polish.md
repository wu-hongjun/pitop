# Epic 7: Advanced Polish

## Goal
Add power-user features: color themes, configuration file support, help overlay,
GPU monitoring via vcgencmd, and optional stress testing mode.

---

## Story 7.1: Color themes

### Acceptance criteria
- [ ] Define a `Theme` struct with named colors for: border, title, text, highlight,
      gauge_low, gauge_warn, gauge_crit, sparkline_cpu, sparkline_mem, sparkline_temp
- [ ] Ship 3 built-in themes: `default` (current colors), `monochrome` (white/gray),
      `solarized` (solarized-dark palette)
- [ ] `--theme` CLI arg selects theme (default/monochrome/solarized)
- [ ] Theme also loadable from config file `[theme]` section
- [ ] All UI modules read colors from the active theme rather than hardcoding `Color::*`
- [ ] Theme is stored on `App` and passed to draw functions

### Files: `src/ui/theme.rs`, update all `src/ui/*.rs`, `src/main.rs`

---

## Story 7.2: Configuration file support

### Acceptance criteria
- [ ] Loads config from `~/.config/pitop/config.toml` (XDG) if it exists
- [ ] Falls back to built-in defaults if no config file found
- [ ] Config fields: interval_ms, default_tab, history_size, theme, threshold values
- [ ] CLI args override config file values (CLI > config > default)
- [ ] `--config` CLI flag to specify a custom config file path
- [ ] Invalid config produces a human-readable error, not a panic

### Files: `src/config.rs`, `src/main.rs`

---

## Story 7.3: Help overlay

### Acceptance criteria
- [ ] Press `?` to toggle a centered overlay showing all keyboard shortcuts
- [ ] Overlay is semi-transparent (dark background with border)
- [ ] Shows all keys: 1-6 tabs, Tab/Shift+Tab, q/Ctrl+C, Space, j/k, s, K, ?
- [ ] Press `?` again or `Esc` to dismiss
- [ ] Overlay renders on top of the current tab content
- [ ] Add `show_help: bool` field to `App`

### Files: `src/ui/help.rs`, `src/ui/mod.rs`, `src/event.rs`, `src/app.rs`

---

## Story 7.4: GPU monitoring

### Acceptance criteria
- [ ] Read GPU frequency via `vcgencmd measure_clock core` (parse `frequency(NN)=XXXXX`)
- [ ] Read GPU memory allocation via `vcgencmd get_mem gpu` (parse `gpu=128M`)
- [ ] Read GPU temperature via `vcgencmd measure_temp` (parse `temp=XX.X'C`)
- [ ] Add GPU section to Overview tab: "GPU: 500MHz / 128M / 52.3°C"
- [ ] Gracefully returns N/A when vcgencmd is unavailable
- [ ] Data collected always-on (same as CPU, for overview display)

### Files: `src/collectors/gpu.rs`, `src/collectors/mod.rs`, `src/app.rs`, `src/ui/overview.rs`

---

## Story 7.5: Stress testing mode

### Acceptance criteria
- [ ] `--stress` CLI flag activates stress mode
- [ ] In stress mode: spawns N worker threads (N = CPU core count) doing pure
      computation (e.g., tight loop computing SHA-256 hashes or prime sieve)
- [ ] Status bar shows "STRESS TEST ACTIVE" in red when running
- [ ] Press `Ctrl+S` to stop stress test (workers are dropped)
- [ ] Stress mode is display-only — it does not affect data collection
- [ ] Uses `tokio::task::spawn_blocking` for worker threads
- [ ] Workers check a shared `AtomicBool` to know when to stop

### Files: `src/stress.rs`, `src/main.rs`, `src/event.rs`, `src/app.rs`, `src/ui/mod.rs`
