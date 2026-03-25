# Building

## Prerequisites

- **Rust 1.70+** -- install via [rustup](https://rustup.rs/):

    ```sh
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source ~/.cargo/env
    ```

- **A C linker** -- required for the `libc` crate:

    ```sh
    # Debian/Ubuntu/Raspberry Pi OS
    sudo apt install build-essential
    ```

---

## Build from source

```sh
git clone https://github.com/wu-hongjun/pitop.git
cd pitop
cargo build --release
```

The binary will be at `target/release/pitop`.

### Debug build

```sh
cargo build
```

Debug builds compile faster but run slower. The binary is at `target/debug/pitop`.

---

## Release profile

The release profile in `Cargo.toml` is optimized for binary size, which matters for the Pi Zero 2W:

```toml
[profile.release]
opt-level = "z"     # Optimize for size (not speed)
lto = true          # Link-time optimization
strip = true        # Strip debug symbols
codegen-units = 1   # Better optimization at cost of compile time
panic = "abort"     # Smaller binary (no unwinding)
```

Typical binary sizes:

| Target | Approximate size |
|--------|-----------------|
| aarch64-unknown-linux-gnu | ~2-4 MB |
| armv7-unknown-linux-gnueabihf | ~2-4 MB |
| x86_64-unknown-linux-gnu | ~2-4 MB |

---

## Cross-compilation

### For Pi 5 / Pi 4B (64-bit)

```sh
rustup target add aarch64-unknown-linux-gnu
sudo apt install gcc-aarch64-linux-gnu

cargo build --release --target aarch64-unknown-linux-gnu
```

Add to `~/.cargo/config.toml`:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

### For Pi 4B (32-bit)

```sh
rustup target add armv7-unknown-linux-gnueabihf
sudo apt install gcc-arm-linux-gnueabihf

cargo build --release --target armv7-unknown-linux-gnueabihf
```

Add to `~/.cargo/config.toml`:

```toml
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```

### Deploy to Pi

```sh
scp target/aarch64-unknown-linux-gnu/release/pitop pi@raspberrypi:~/
ssh pi@raspberrypi 'sudo mv ~/pitop /usr/local/bin/'
```

---

## Linting and formatting

These must pass before committing:

```sh
# Check for warnings (project enforces zero warnings)
cargo clippy

# Format code
cargo fmt

# Both together
cargo fmt && cargo clippy
```

!!! note
    CI enforces `cargo clippy -- -D clippy::unwrap_used -D clippy::expect_used` to prevent `unwrap()` and `expect()` in production code.

---

## Running locally on non-Pi systems

pitop works on any Linux system. Pi-specific features (PMIC, fan, PCIe, PoE) will be unavailable, but core monitoring (CPU, memory, network, disk, processes) works.

On macOS, the build will succeed but the binary will not run because it depends on Linux-specific procfs/sysfs paths.

```sh
# Build and run on Linux (even non-Pi)
cargo run --release
```
