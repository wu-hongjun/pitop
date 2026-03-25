# Quick Start

## Launching pitop

After [installation](installation.md), start pitop from any terminal:

```sh
pitop
```

pitop will auto-detect your board, initialize collectors, and display the Overview dashboard.

---

## Navigating tabs

pitop has six tabs, each showing a different aspect of your system:

| Tab | Key | Content |
|-----|-----|---------|
| Overview | `1` | CPU, memory, temperature, GPU, network, fan |
| Processes | `2` | Sortable process table with kill support |
| Power | `3` | PMIC rails, voltages, PCIe, PoE |
| Network | `4` | Interface details, throughput sparklines |
| Disk | `5` | Partition usage, I/O throughput |
| System | `6` | Board info, kernel, uptime, capabilities |

Switch tabs by pressing the number key (`1`-`6`), or cycle through them with `Tab` and `Shift+Tab`.

---

## Basic keyboard shortcuts

| Key | Action |
|-----|--------|
| `1`-`6` | Switch to tab |
| `Tab` / `Shift+Tab` | Next / previous tab |
| `q` or `Ctrl+C` | Quit |
| `Space` | Pause / resume data collection |
| `t` | Cycle through color themes |
| `?` | Open the help overlay |

!!! tip
    Press `?` at any time to see all available keyboard shortcuts for the current context.

---

## Understanding the Overview dashboard

The Overview tab (tab 1) is the default view and provides a summary of your system:

### CPU section

- **Per-core gauges** showing utilization percentage for each CPU core
- **Aggregate CPU usage** across all cores
- **Frequency** display showing current clock speed
- **Sparkline** showing CPU usage history (last 60 samples)

### Memory section

- **RAM usage** with used/total and percentage gauge
- **Swap usage** with used/total and percentage gauge
- **Sparkline** showing memory usage history

### Thermal section

- **SoC temperature** in degrees Celsius
- Color-coded by threshold: green (normal), yellow (warning at 60C), red (critical at 75C)
- **Sparkline** showing temperature history

### Network section

- **Per-interface throughput** showing download and upload rates
- **Sparklines** for network activity history

### Pi-specific sections

Depending on your board, you may also see:

- **Fan speed** (Pi 5) -- RPM and duty cycle percentage
- **GPU** -- frequency, memory allocation, temperature, codec status
- **Throttle status** -- current and historical throttle events

---

## Common launch options

```sh
# Faster refresh (500ms instead of default 1000ms)
pitop -i 500

# Start on the Power tab
pitop -t 3

# Use the solarized color theme
pitop --theme solarized

# Force Pi 5 mode on an unrecognized board
pitop --board pi5

# Run with verbose error output for debugging
pitop -v
```

---

## Next steps

- [Configure pitop](configuration.md) with a persistent config file
- Learn about each tab in the [User Guide](../guide/overview.md)
- See all [CLI options](../reference/cli.md) and [keyboard shortcuts](../reference/keybindings.md)
