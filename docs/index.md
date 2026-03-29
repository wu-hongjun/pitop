# pitop

**A terminal-based system monitor for Raspberry Pi, written in Rust.**

Think `htop` meets `mactop` -- purpose-built for Raspberry Pi hardware with board-specific features like PMIC power rails, fan speed, PCIe link status, and PoE detection.

---

## Features

- **Overview dashboard** -- CPU gauges, memory, temperature, GPU, network throughput, and sparkline history
- **Process manager** -- sortable table with vim-style navigation and kill support
- **Power monitoring** -- Pi 5 PMIC rails with per-rail voltage/current/wattage, Pi 4B voltage readings
- **Fan speed** -- RPM and duty cycle display for Pi 5 with official fan
- **PCIe link status** -- generation and lane width with downgrade detection (Pi 5)
- **PoE HAT** -- status and current draw detection (Pi 5, Pi 4B)
- **Network interfaces** -- throughput sparklines, IPv4/IPv6, MAC addresses
- **Disk partitions** -- usage and I/O throughput
- **System info** -- board model, kernel, architecture, uptime
- **Three color themes** -- default, monochrome, solarized, plus custom theme support
- **Built-in stress testing** -- CPU load generator with adjustable worker count
- **Configuration file** -- TOML-based persistent settings

---

## Quick install

```sh
curl -sL https://pitop.hongjunwu.com/install.sh | sh
```

Auto-detects your Pi model and architecture, downloads the right binary, and installs to `/usr/local/bin`.

[Full installation guide](getting-started/installation.md){ .md-button }

---

## Supported boards

| Board | SoC | Features |
|-------|-----|----------|
| Raspberry Pi 5 | BCM2712 | Full: PMIC, fan, PCIe, PoE |
| Raspberry Pi 4 Model B | BCM2711 | Voltages, PoE |
| Raspberry Pi Zero 2 W | BCM2710 | Basic monitoring |
| Generic Linux | -- | CPU, memory, network, disk |

pitop degrades gracefully on any Linux system. Pi-specific features are activated only when the corresponding hardware is detected.

---

## Quick start

```sh
pitop                          # Launch with defaults
pitop -i 500                   # 500ms refresh interval
pitop -t 3                     # Start on the Power tab
pitop --theme solarized        # Use the solarized theme
pitop --stress                 # Launch with CPU stress test
```

Use number keys `1`-`6` to switch tabs, `q` to quit, `?` for help.

[Quick start guide](getting-started/quick-start.md){ .md-button }

---

## Architecture

pitop reads directly from `/proc/` and `/sys/` files -- no external system-info libraries. Board-specific features (PMIC, fan, PCIe) are activated based on hardware detection at startup. Every Pi-specific feature handles missing hardware gracefully, so the app runs on any Linux system with reduced features.

Built with:

- **ratatui** + **crossterm** -- TUI rendering and terminal backend
- **tokio** -- async runtime for tick loop and subprocess calls
- **clap** -- CLI argument parsing
- **anyhow** -- error handling

[Architecture details](development/architecture.md){ .md-button }

---

## License

pitop is released under the [MIT License](https://github.com/wu-hongjun/pitop/blob/main/LICENSE).
