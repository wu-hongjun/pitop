# pitop

A terminal-based system monitor for Raspberry Pi, written in Rust.

> **Status**: Pre-release. Under active development.

## What is this?

pitop is like [mactop](https://github.com/metaspartan/mactop) but for Raspberry Pi hardware. It provides a real-time TUI dashboard showing CPU, memory, thermals, power, network, disk, and processes — with features specific to Pi hardware like PMIC power rail monitoring, fan speed, PCIe link status, and PoE detection.

## Supported boards

| Board | Status |
|-------|--------|
| Raspberry Pi 5 | 🟡 In progress |
| Raspberry Pi 4 Model B | 🟡 In progress |
| Raspberry Pi Zero 2 W | 🟡 In progress |

## Features (planned)

- **Overview dashboard** — CPU per-core gauges, memory, temperature, throttle status
- **Process monitor** — Sortable table with CPU%, MEM%, kill support
- **Power tab** — PMIC rail voltages/currents (Pi 5), total wattage estimate, PoE status
- **PCIe info** — Link speed (Gen 2/3), width, connected device (Pi 5)
- **Network tab** — Per-interface throughput, IP addresses
- **Disk tab** — Partition usage, I/O rates
- **System info** — Board model, kernel, uptime, fan speed

## Building from source

```bash
# Native build (on the Pi itself)
cargo build --release

# Cross-compile for Pi 5 / Pi 4B (64-bit)
cargo build --release --target aarch64-unknown-linux-gnu

# Cross-compile for Zero 2W (32-bit)
cargo build --release --target armv7-unknown-linux-gnueabihf
```

## Running

```bash
./pitop            # Default: 1 second refresh
./pitop -i 500     # 500ms refresh interval
./pitop --tab 3    # Start on Power tab
```

## Keyboard shortcuts

| Key | Action |
|-----|--------|
| `1`-`6` | Switch tabs |
| `Tab` | Next tab |
| `q` | Quit |
| `Space` | Pause/resume |
| `j`/`k` | Navigate lists |
| `s` | Cycle sort column |
| `?` | Help |

## Architecture

See [CLAUDE.md](./CLAUDE.md) for architecture decisions and coding standards.
See [docs/design-research.md](./docs/design-research.md) for hardware research.

## License

MIT
