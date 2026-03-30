# Overview Tab

The Overview tab (tab 1) is the default landing view. It provides a real-time dashboard of key system metrics in a mactop-inspired layout with gauge blocks, info panels, and a process list.

---

## Layout

The Overview tab uses a mactop-style layout divided into three rows:

### Top row: 4 gauge blocks

- **CPU** -- per-core gauges with aggregate usage and clock frequency
- **GPU / Load** -- V3D clock frequency (Pi 5), load averages, GPU temperature
- **Temperature** -- SoC temperature with color-coded gauge, plus PMIC/RP1/NVMe temps when available
- **Memory** -- RAM and swap usage gauges with used/total display

### Middle row: 4 info panels

- **Power** -- total estimated wattage, input voltage, key rail readings (Pi 5 PMIC), or voltage readings (other Pi boards)
- **Board** -- model name, SoC, CPU type, architecture, kernel version
- **Network** -- per-interface throughput rates (RX/TX), excluding loopback
- **Disk** -- partition usage with device name and mountpoint on separate lines (no `/dev/` prefix)

### Bottom row: process list

- Embedded process table showing top processes by CPU usage
- Supports the same keyboard shortcuts as the Processes tab: `j`/`k` to navigate, `s` to cycle sort column, `K` to kill

---

## CPU gauge block

Displays real-time CPU utilization parsed from `/proc/stat`.

- **Per-core gauge bars** show individual core utilization as percentage bars
- **Aggregate usage** is the average across all cores
- **Current frequency** is read from `/sys/devices/system/cpu/cpufreq/`

### Color coding

| Color | Meaning |
|-------|---------|
| Green | Below warning threshold (default: < 60%) |
| Yellow | At or above warning threshold (default: >= 60%) |
| Red | At or above critical threshold (default: >= 85%) |

---

## Memory gauge block

Shows RAM and swap usage parsed from `/proc/meminfo`.

- **RAM gauge** displays used / total and percentage
- **Swap gauge** displays used / total and percentage (hidden if no swap is configured)

### Color coding

| Color | Meaning |
|-------|---------|
| Green | Below warning threshold (default: < 70% RAM, < 25% swap) |
| Yellow | At or above warning (default: >= 70% RAM, >= 25% swap) |
| Red | At or above critical (default: >= 90% RAM, >= 50% swap) |

---

## Temperature gauge block

Displays SoC temperature from thermal zones in `/sys/class/thermal/`.

- **Temperature gauge** with color-coded reading in degrees Celsius

On Pi 5, additional thermal zones may appear for the PMIC, RP1 southbridge, and NVMe drives (discovered via hwmon).

### Color coding

| Color | Meaning |
|-------|---------|
| Green | Below 60C |
| Yellow | 60C -- 74C (warning range) |
| Red | 75C+ (critical) |

!!! warning "Throttling"
    If the SoC temperature reaches the critical threshold, the Pi firmware may throttle the CPU to reduce heat. Check the throttle indicator in the header bar.

---

## Network info panel

Shows per-interface throughput rates parsed from `/proc/net/dev`.

- **Download rate** (RX bytes/sec) for each interface
- **Upload rate** (TX bytes/sec) for each interface

The loopback interface (`lo`) is filtered out by default.

---

## Disk info panel

Shows partition usage with device names and mountpoints.

- **Device name** shown without the `/dev/` prefix (e.g., `sda1` instead of `/dev/sda1`)
- **Mountpoint** displayed on a separate line below the device name
- **Usage percentage** with color-coded gauge

---

## Power info panel

Shows power and voltage readings collected in real time on the Overview tab:

- **Pi 5**: Total estimated wattage, input voltage (EXT5V_V), and key PMIC rail readings
- **Pi 4B and other boards**: Core and SDRAM voltage readings via `vcgencmd measure_volts`

!!! note
    Power/voltage data is collected on the Overview tab as well as the Power tab, so you can monitor power draw without switching tabs.

---

## Board info panel

Shows hardware identification:

- **Board model** and SoC name
- **CPU type** with core names mapped from ARM CPU part numbers (e.g., Cortex-A76)
- **Architecture** and kernel version

---

## GPU / Load gauge block

Displays GPU and system load data:

- **V3D clock frequency** in MHz on Pi 5 (from `vcgencmd measure_clock v3d`), or core clock on other boards
- **GPU memory** shown as "Shared" on Pi 5 (which uses shared system memory), or allocation in MB on other boards
- **GPU temperature** in Celsius (from `vcgencmd measure_temp`)
- **Codec status**: shows "Hardware HEVC (BCM2712)" on Pi 5 instead of disabled codec marks
- **Load averages** (1, 5, 15 minute)

!!! note
    GPU data is only available on Raspberry Pi boards with `vcgencmd` installed. On other systems, this section is hidden.

---

## Throttle indicator

The header bar shows a throttle status indicator:

- **Green** -- no throttle events detected
- **Yellow** -- throttle events occurred in the past but are not currently active
- **Red** -- active throttling is occurring right now

Throttle data comes from `vcgencmd get_throttled`, which reports a bitmask covering under-voltage, frequency capping, and thermal throttling.
