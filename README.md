# pitop

A terminal-based system monitor for Raspberry Pi, written in Rust.

Think `htop` meets `mactop` — purpose-built for Raspberry Pi hardware with board-specific features like PMIC power rails, fan speed, PCIe link status, and PoE detection.

## Features

- **Overview dashboard** in mactop-style layout with gauge blocks, info panels, and embedded process list
- **Automatic update checker** on startup with install command in footer
- **Process manager** with sortable table and kill support (works on Overview tab too)
- **Power monitoring** — Pi 5 PMIC rails with per-rail voltage/current/wattage, Pi 4B voltage readings
- **Fan speed** display (Pi 5 with official fan)
- **PCIe link status** with generation and downgrade detection (Pi 5)
- **PoE HAT** status and current draw (Pi 5, Pi 4B)
- **Network interfaces** with throughput rates, IPv6, MAC addresses
- **Disk partitions** and I/O throughput
- **GPU monitoring** — V3D clock (Pi 5), memory, temperature, hardware HEVC status
- **NVMe temperature** via hwmon discovery (Pi 5)
- **System info** — board model, kernel, architecture, uptime

## Supported Boards

| Board | SoC | Features |
|-------|-----|----------|
| Raspberry Pi 5 | BCM2712 | Full: PMIC, fan, PCIe, PoE, V3D, NVMe temp |
| Raspberry Pi 4 Model B | BCM2711 | Voltages, PoE, GPU |
| Raspberry Pi 400 | BCM2711 | Voltages, GPU |
| Compute Module 4 | BCM2711 | Voltages, GPU |
| Raspberry Pi 3 Model B+ | BCM2837 | Basic monitoring, GPU |
| Raspberry Pi 3 Model B | BCM2837 | Basic monitoring, GPU |
| Raspberry Pi 3 Model A+ | BCM2837 | Basic monitoring, GPU |
| Raspberry Pi 2 Model B | BCM2836 | Basic monitoring, GPU |
| Raspberry Pi 1 | BCM2835 | Basic monitoring |
| Raspberry Pi Zero 2 W | BCM2710/BCM2837 | Basic monitoring, GPU |
| Raspberry Pi Zero W | BCM2835 | Basic monitoring |
| Raspberry Pi Zero | BCM2835 | Basic monitoring |
| Generic Linux | -- | CPU, memory, network, disk |

## Installation

### One-line install (recommended)

```sh
curl -sL https://pitop.hongjunwu.com/install.sh | sh
```

Auto-detects your Pi model and architecture, downloads the right binary, and installs to `/usr/local/bin`. Pin a version with `PITOP_VERSION=v0.1.0`.

### cargo install

```sh
cargo install --git https://github.com/wu-hongjun/pitop
```

### Build from source

```sh
git clone https://github.com/wu-hongjun/pitop.git
cd pitop
cargo build --release
sudo cp target/release/pitop /usr/local/bin/
```

### Cross-compile (from another machine)

```sh
# For Pi 5 / Pi 4B (64-bit)
cargo build --release --target aarch64-unknown-linux-gnu

# For Pi 4B (32-bit)
cargo build --release --target armv7-unknown-linux-gnueabihf
```

### Download binary

Grab a tarball from [GitHub Releases](https://github.com/wu-hongjun/pitop/releases) and extract it:

```sh
tar xzf pitop-v*.tar.gz
sudo mv pitop /usr/local/bin/
```

## Usage

```
pitop                          # Start with defaults
pitop -i 500                   # 500ms refresh interval
pitop -t 3                     # Start on Power tab
pitop --theme solarized        # Use solarized theme
pitop --board pi5              # Force Pi 5 mode
pitop --stress                 # Launch with CPU stress test
pitop --generate-config        # Print sample config to stdout
pitop --config-check           # Validate your config file
```

## Keyboard Controls

| Key | Action |
|-----|--------|
| `1`-`6` | Switch tabs |
| `Tab` / `Shift+Tab` | Next / previous tab |
| `q` / `Ctrl+C` | Quit |
| `Space` | Pause / resume |
| `t` | Cycle color theme |
| `?` | Help overlay |
| `j`/`k` or arrows | Navigate (Overview / Processes / Help) |
| `s` | Cycle sort column (Processes tab) |
| `K` | Kill selected process (with confirm) |
| `Ctrl+S` | Toggle stress test |
| `Ctrl+Up/Down` | Add/remove stress workers |

## Tabs

1. **Overview** — CPU, GPU/load, temperature, memory gauges + power, board, network, disk info + process list
2. **Processes** — Sortable process table with kill support
3. **Power** — PMIC rails (Pi 5), voltages (Pi 4B), PCIe, PoE
4. **Network** — Interface details, throughput sparklines
5. **Disk** — Partition usage, I/O throughput
6. **System** — Board info, kernel, uptime, capabilities

## Architecture

pitop reads directly from `/proc/` and `/sys/` files — no external system-info libraries. Board-specific features (PMIC, fan, PCIe) are activated based on hardware detection at startup. The application degrades gracefully: if a hardware feature is not present, it is simply omitted from the display.

See [CLAUDE.md](./CLAUDE.md) for architecture decisions and coding standards.
See [docs/design-research.md](./docs/design-research.md) for hardware research.

## Requirements

- Raspberry Pi (or any Linux system for basic features)
- Terminal with 256-color support
- `vcgencmd` for throttle/voltage/PMIC data (included in Raspberry Pi OS)

## License

MIT
