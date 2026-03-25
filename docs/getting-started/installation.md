# Installation

## One-line install (recommended)

```sh
curl -sL https://wu-hongjun.github.io/pitop/install.sh | sh
```

The install script auto-detects your Pi model and CPU architecture, downloads the correct binary from GitHub Releases, and installs it to `/usr/local/bin`.

To pin a specific version:

```sh
curl -sL https://wu-hongjun.github.io/pitop/install.sh | PITOP_VERSION=v0.1.0 sh
```

!!! note "Supported architectures"
    The installer supports `aarch64` (64-bit ARM), `armv7l` (32-bit ARM), and `x86_64`. ARMv6 (original Pi Zero, Pi 1) is not supported.

---

## cargo install

If you have the Rust toolchain installed:

```sh
cargo install --git https://github.com/wu-hongjun/pitop
```

This builds pitop from source and installs the binary to `~/.cargo/bin/`.

---

## Build from source

```sh
git clone https://github.com/wu-hongjun/pitop.git
cd pitop
cargo build --release
sudo cp target/release/pitop /usr/local/bin/
```

### Prerequisites

- **Rust 1.70+** -- install via [rustup](https://rustup.rs/):

    ```sh
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source ~/.cargo/env
    ```

- **A C linker** -- typically provided by `build-essential` on Debian/Raspberry Pi OS:

    ```sh
    sudo apt install build-essential
    ```

---

## Cross-compile

You can build pitop on a faster machine and copy the binary to your Pi.

=== "Pi 5 / Pi 4B (64-bit)"

    ```sh
    rustup target add aarch64-unknown-linux-gnu
    cargo build --release --target aarch64-unknown-linux-gnu
    scp target/aarch64-unknown-linux-gnu/release/pitop pi@raspberrypi:~/
    ```

=== "Pi 4B (32-bit)"

    ```sh
    rustup target add armv7-unknown-linux-gnueabihf
    cargo build --release --target armv7-unknown-linux-gnueabihf
    scp target/armv7-unknown-linux-gnueabihf/release/pitop pi@raspberrypi:~/
    ```

!!! tip "Cross-compilation linker"
    You may need to install a cross-compilation linker. On Ubuntu/Debian:

    ```sh
    # For aarch64
    sudo apt install gcc-aarch64-linux-gnu

    # For armv7
    sudo apt install gcc-arm-linux-gnueabihf
    ```

    Then configure Cargo to use it by adding to `~/.cargo/config.toml`:

    ```toml
    [target.aarch64-unknown-linux-gnu]
    linker = "aarch64-linux-gnu-gcc"

    [target.armv7-unknown-linux-gnueabihf]
    linker = "arm-linux-gnueabihf-gcc"
    ```

---

## Download binary

Download a prebuilt tarball from [GitHub Releases](https://github.com/wu-hongjun/pitop/releases):

```sh
tar xzf pitop-v*.tar.gz
sudo mv pitop /usr/local/bin/
```

---

## Requirements

- **Raspberry Pi** (or any Linux system for basic features)
- **Terminal with 256-color support** (most modern terminals)
- **`vcgencmd`** for throttle/voltage/PMIC data (included in Raspberry Pi OS)

!!! warning "vcgencmd on non-Pi systems"
    If `vcgencmd` is not available (e.g., on a generic Linux system or a non-standard Pi OS), pitop still runs but Pi-specific features like power monitoring and throttle detection will be unavailable.

---

## Verify installation

```sh
pitop --version
```

If the command prints a version number, pitop is installed and ready to use.
