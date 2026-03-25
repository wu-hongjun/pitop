# Contributing

Contributions to pitop are welcome. This page covers the coding standards, architecture rules, and process for submitting changes.

---

## Coding standards

### Error handling

- **No `unwrap()` or `expect()`** in any code path that can reach production
- Use `anyhow::Result`, `Option`, or `.unwrap_or_default()`
- CI enforces this via `cargo clippy -- -D clippy::unwrap_used -D clippy::expect_used`
- `unwrap()` is acceptable in test code only

### Subprocess calls

- **No `std::process::Command`** -- always use `tokio::process::Command`
- All vcgencmd calls go through the centralized `VcgencmdRunner` in `src/util/vcgencmd.rs`

### Hardware paths

- **No hardcoded hwmon numbers** (e.g., `hwmon2`). Always discover by enumerating `/sys/class/hwmon/` and matching by the `name` file content
- hwmon numbers change across reboots, so hardcoded values will break

### sysfs access

- All sysfs/procfs reads must handle `ENOENT` (file not found) and `EACCES` (permission denied) gracefully
- Return `None`, `Ok(default)`, or an appropriate error -- never panic
- Do not shell out for data available via sysfs (temperature, CPU frequency, etc.)

### Code quality

- `cargo clippy` must pass with zero warnings
- `cargo fmt` must be applied before every commit
- No GUI dependencies -- this is a terminal-only application

---

## Architecture rules

### Direct data collection

pitop reads system data directly from `/proc/` and `/sys/` files. Do not use the `sysinfo` crate or any system-info abstraction library. The only exception is `vcgencmd`, which is called as a subprocess for data not available via sysfs.

### Board detection

Board detection reads `/proc/device-tree/compatible` at startup. Unknown boards run with generic Linux collectors only. Every Pi-specific feature must have a fallback for when the hardware is not present.

### Lazy tab refresh

Only the active tab's expensive collectors run each tick. The Overview tab's metrics (CPU, memory, thermal, network, fan, GPU, throttle) always run. This is important for keeping CPU usage low on the Pi Zero 2W.

### Ring buffers

All sparkline history uses fixed-size ring buffers from `src/util/ring_buffer.rs`. The default size is 60 samples, configurable via the `history_size` config option (range: 10-600).

---

## Project structure

See the [Architecture](architecture.md) page for the full module structure and design details.

---

## Setting up for development

```sh
# Clone the repository
git clone https://github.com/wu-hongjun/pitop.git
cd pitop

# Build and run
cargo run

# Run with verbose output for debugging
cargo run -- -v

# Run tests
cargo test

# Check lints
cargo clippy
```

---

## Pull request process

1. Fork the repository and create a feature branch
2. Make your changes following the coding standards above
3. Ensure all tests pass: `cargo test`
4. Ensure no lint warnings: `cargo clippy`
5. Format your code: `cargo fmt`
6. Write a clear commit message describing the change
7. Open a pull request against `main`

### PR checklist

- [ ] `cargo test` passes
- [ ] `cargo clippy` passes with zero warnings
- [ ] `cargo fmt` applied
- [ ] No `unwrap()` or `expect()` in production code
- [ ] No hardcoded hwmon numbers
- [ ] No `std::process::Command` (use `tokio::process::Command`)
- [ ] Pi-specific features degrade gracefully on non-Pi hardware
- [ ] New collectors include unit tests with fixture data

---

## Adding a new collector

1. Create `src/collectors/new_thing.rs` with a struct and `collect()` method
2. Add the data struct to hold parsed values (with `Default` impl)
3. Add the collector and data fields to `App` in `src/app.rs`
4. Wire up the collector in `App::tick()` (decide: always-on or tab-specific?)
5. Create or update the corresponding `src/ui/*.rs` module to render the data
6. Add unit tests with fixture data
7. Export the module in `src/collectors/mod.rs`

---

## Adding a new UI tab

1. Create `src/ui/new_tab.rs` with a `draw(f: &mut Frame, app: &App, area: Rect)` function
2. Add the module declaration to `src/ui/mod.rs`
3. Add a match arm in `draw_active_tab()` in `src/ui/mod.rs`
4. Update `TAB_COUNT` and `TAB_NAMES` in `src/app.rs`
5. Update key handling in `src/event.rs` if needed
6. Use colors from `app.theme` -- never hardcode `Color::*` values
