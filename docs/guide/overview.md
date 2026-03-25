# Overview Tab

The Overview tab (tab 1) is the default landing view. It provides a real-time dashboard of key system metrics with sparkline history graphs.

---

## Layout

The Overview tab is divided into several sections, arranged to fit within the terminal window:

- **CPU** -- per-core gauges, aggregate usage, clock frequency, sparkline
- **Memory** -- RAM and swap usage with gauges and sparkline
- **Thermal** -- SoC temperature with sparkline
- **Network** -- per-interface throughput rates
- **GPU** -- frequency, memory, temperature, codec status (Pi-specific)
- **Fan** -- RPM and duty cycle (Pi 5 only)

---

## CPU section

Displays real-time CPU utilization parsed from `/proc/stat`.

- **Per-core gauge bars** show individual core utilization as percentage bars
- **Aggregate usage** is the average across all cores
- **Current frequency** is read from `/sys/devices/system/cpu/cpufreq/`
- **Sparkline** plots the last 60 samples (configurable via `history_size`) of aggregate CPU usage

### Color coding

| Color | Meaning |
|-------|---------|
| Green | Below warning threshold (default: < 60%) |
| Yellow | At or above warning threshold (default: >= 60%) |
| Red | At or above critical threshold (default: >= 85%) |

---

## Memory section

Shows RAM and swap usage parsed from `/proc/meminfo`.

- **RAM gauge** displays used / total and percentage
- **Swap gauge** displays used / total and percentage (hidden if no swap is configured)
- **Sparkline** tracks RAM usage percentage over time

### Color coding

| Color | Meaning |
|-------|---------|
| Green | Below warning threshold (default: < 70% RAM, < 25% swap) |
| Yellow | At or above warning (default: >= 70% RAM, >= 25% swap) |
| Red | At or above critical (default: >= 90% RAM, >= 50% swap) |

---

## Thermal section

Displays SoC temperature from thermal zones in `/sys/class/thermal/`.

- **Temperature reading** in degrees Celsius
- **Sparkline** tracks temperature history

On Pi 5, additional thermal zones may appear for the PMIC and RP1 southbridge.

### Color coding

| Color | Meaning |
|-------|---------|
| Green | Below 60C |
| Yellow | 60C -- 74C (warning range) |
| Red | 75C+ (critical) |

!!! warning "Throttling"
    If the SoC temperature reaches the critical threshold, the Pi firmware may throttle the CPU to reduce heat. Check the throttle indicator in the header bar.

---

## Network section

Shows per-interface throughput rates parsed from `/proc/net/dev`.

- **Download rate** (RX bytes/sec) for each interface
- **Upload rate** (TX bytes/sec) for each interface
- **Sparklines** for each interface's RX and TX history

The loopback interface (`lo`) is filtered out by default.

---

## GPU section

Displays GPU data collected via `vcgencmd`:

- **Core frequency** in MHz (from `vcgencmd measure_clock core`)
- **Memory allocation** in MB (from `vcgencmd get_mem gpu`)
- **Temperature** in Celsius (from `vcgencmd measure_temp`)
- **Codec status** showing whether H264 and HEVC hardware decode is enabled

!!! note
    GPU data is only available on Raspberry Pi boards with `vcgencmd` installed. On other systems, this section is hidden.

---

## Fan section (Pi 5 only)

Displays fan speed data from the hwmon subsystem:

- **RPM** -- current fan speed in revolutions per minute
- **Duty cycle** -- PWM duty as a percentage (0--100%)

This section only appears on Pi 5 boards with the official fan connected. The fan hwmon device is discovered dynamically by scanning `/sys/class/hwmon/` for a device named `cooling_fan`.

---

## Throttle indicator

The header bar shows a throttle status indicator:

- **Green** -- no throttle events detected
- **Yellow** -- throttle events occurred in the past but are not currently active
- **Red** -- active throttling is occurring right now

Throttle data comes from `vcgencmd get_throttled`, which reports a bitmask covering under-voltage, frequency capping, and thermal throttling.
