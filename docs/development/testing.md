# Testing

## Running tests

```sh
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests for a specific module
cargo test config::tests
cargo test stress::tests
cargo test event::tests
```

---

## Test architecture

pitop uses a root-relative path design that makes testing straightforward. Every collector accepts a `root: &Path` parameter instead of hardcoding `/proc/` and `/sys/`. In tests, a temporary directory serves as the root, populated with fixture files.

### Example

```rust
#[test]
fn test_cpu_collector() {
    let tmp = tempfile::TempDir::new().unwrap();
    // Create fixture files at tmp.path().join("proc/stat")
    let collector = CpuCollector::new(tmp.path());
    // ...
}
```

---

## Test fixtures

Fixture files live in `tests/fixtures/` organized by board type:

```
tests/fixtures/
+-- pi5/
|   +-- device-tree-compatible
|   +-- proc/
|   +-- sys/
+-- pi4b/
|   +-- device-tree-compatible
|   +-- proc/
|   +-- sys/
+-- zero2w/
    +-- device-tree-compatible
    +-- proc/
    +-- sys/
```

These contain sample data from real Raspberry Pi systems, used to verify parsing logic without needing actual hardware.

---

## What is tested

### Unit tests

Each module includes inline `#[cfg(test)]` test modules:

| Module | Tests |
|--------|-------|
| `config.rs` | Config loading, defaults, validation, TOML parsing, color parsing |
| `event.rs` | Key handling, tab switching, pause, theme cycling |
| `stress.rs` | Worker start/stop, add/remove, clamping, primality |
| `ui/theme.rs` | Theme lookup, custom theme building, fallback behavior |
| `collectors/gpu.rs` | vcgencmd output parsing (clock, memory, temp, codecs) |
| `board/mod.rs` | Board detection from device-tree strings |
| `util/ring_buffer.rs` | Push, capacity, ordering |
| `util/sysfs.rs` | File reading helpers |
| `collectors/*.rs` | Data parsing for each collector |

### Integration tests

The App struct can be instantiated with a temporary root directory for integration-level testing:

```rust
fn test_app() -> App {
    let tmp = TempDir::new().unwrap();
    App::new(BoardType::Unknown, tmp.path(), false, Config::default())
}
```

---

## Adding tests

### For a new collector

1. Add fixture data to the appropriate `tests/fixtures/{board}/` directory
2. Write a test that creates the collector with a temp directory root
3. Populate the temp directory with the fixture data
4. Call `collect()` and assert the parsed values

### For new vcgencmd parsing

1. Add a parsing function (e.g., `parse_something(output: &str) -> Option<T>`)
2. Write tests with sample vcgencmd output strings
3. Test valid output, malformed output, and empty input

### For UI behavior

1. Create an `App` with `test_app()` helper
2. Simulate key events with `handle_key()`
3. Assert the resulting app state

---

## Dev dependencies

| Crate | Purpose |
|-------|---------|
| `tempfile` | Create temporary directories for fixture-based tests |
