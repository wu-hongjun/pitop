# pitop

A terminal-based system monitor for Raspberry Pi, written in Rust.

Think `htop` meets `mactop` — purpose-built for Raspberry Pi hardware with board-specific features like PMIC power rails, fan speed, PCIe link status, and PoE detection.

## Features

- **Overview dashboard** with CPU gauges, memory, temperature, network throughput, and sparkline history
- **Process manager** with sortable table and kill support
- **Power monitoring** — Pi 5 PMIC rails with per-rail voltage/current/wattage, Pi 4B voltage readings
- **Fan speed** display (Pi 5 with official fan)
- **PCIe link status** with generation and downgrade detection (Pi 5)
- **PoE HAT** status and current draw (Pi 5, Pi 4B)
- **Network interfaces** with throughput sparklines, IPv6, MAC addresses
- **Disk partitions** and I/O throughput
- **System info** — board model, kernel, architecture, uptime

## Supported Boards

| Board | SoC | Features |
|-------|-----|----------|
| Raspberry Pi 5 | BCM2712 | Full: PMIC, fan, PCIe, PoE |
| Raspberry Pi 4 Model B | BCM2711 | Voltages, PoE |
| Raspberry Pi Zero 2 W | BCM2710 | Basic monitoring |
| Generic Linux | — | CPU, memory, network, disk |

## Installation

### Quick Install (Raspberry Pi)

```sh
curl -sL https://raw.githubusercontent.com/OWNER/pitop/main/scripts/install.sh | sh
```

To install a specific version:

```sh
PITOP_VERSION=v0.1.0 curl -sL https://raw.githubusercontent.com/OWNER/pitop/main/scripts/install.sh | sh
```

### Build from Source

```bash
git clone https://github.com/OWNER/pitop.git
cd pitop
cargo build --release
sudo cp target/release/pitop /usr/local/bin/
```

### Cross-Compile

```bash
# For Pi 5 / Pi 4B (64-bit)
cargo build --release --target aarch64-unknown-linux-gnu

# For Zero 2W (32-bit)
cargo build --release --target armv7-unknown-linux-gnueabihf
```

## Usage

```
pitop                    # Start with defaults
pitop -i 500             # 500ms refresh interval
pitop -t 3               # Start on Power tab
pitop --board pi5        # Force Pi 5 mode
pitop -v                 # Verbose error output
```

## Keyboard Controls

| Key | Action |
|-----|--------|
| `1`-`6` | Switch tabs |
| `Tab` / `Shift+Tab` | Next / previous tab |
| `q` / `Ctrl+C` | Quit |
| `Space` | Pause / resume |
| `j`/`k` or arrows | Navigate (Processes tab) |
| `s` | Cycle sort column (Processes tab) |
| `K` | Kill selected process (with confirm) |

## Tabs

1. **Overview** — CPU, memory, temperature, network, fan speed
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
