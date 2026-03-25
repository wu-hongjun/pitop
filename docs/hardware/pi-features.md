# Pi-Specific Features

This page details the Raspberry Pi hardware features that pitop monitors, including which boards support them and how the data is collected.

---

## PMIC power rails (Pi 5 only)

The Raspberry Pi 5 includes a Dialog DA9090 PMIC that provides per-rail voltage and current telemetry.

### Data source

```sh
vcgencmd pmic_read_adc
```

This returns 12 voltage rails and 12 current rails, e.g.:

```
VDD_CORE_V=0.8750V
VDD_CORE_A=1.2340A
VDD_CPU_V=0.9000V
VDD_CPU_A=0.5670A
...
```

### Available rails

| Rail | Description |
|------|-------------|
| VDD_CORE | SoC core power |
| VDD_CPU | CPU cluster power |
| VDD_IO | General I/O power |
| EXT5V_V | Input voltage from USB-C power supply |
| BATT_V | RTC backup battery voltage |

### Estimated wattage

Total estimated power draw is calculated using a correction factor:

```
total_watts = sum(voltage * current for all rails) * 1.1451 + 0.5879
```

The correction factor accounts for PMIC conversion losses that are not captured in the rail measurements.

---

## Fan monitoring (Pi 5 only)

The Pi 5 has a dedicated 4-pin PWM fan header.

### Data source

pitop discovers the fan hwmon device by scanning `/sys/class/hwmon/` and matching the device with `name` equal to `cooling_fan`.

!!! warning "No hardcoded hwmon numbers"
    hwmon numbers (e.g., `hwmon2`) change across reboots. pitop always discovers devices by enumerating all hwmon entries and matching by the `name` file content.

### Readings

| File | Description |
|------|-------------|
| `fan1_input` | Current fan speed in RPM |
| `pwm1` | PWM duty cycle (0-255, displayed as 0-100%) |

### Display

The Overview tab shows fan status as: `Fan: 3054 RPM (45%)`

If no fan hwmon device is found, the fan section is hidden.

---

## PCIe link status (Pi 5 only)

The Pi 5 exposes a single-lane PCIe interface (x1 Gen 2.0 by default, configurable to Gen 3.0) for NVMe SSDs and other PCIe devices via the HAT+ connector.

### Data source

PCIe link information is read from:

```
/sys/bus/pci/devices/*/current_link_speed
/sys/bus/pci/devices/*/current_link_width
```

### Display

The Power tab shows:

- **Link speed** (e.g., Gen 2 5.0 GT/s, Gen 3 8.0 GT/s)
- **Lane width** (typically x1)
- **Downgrade warning** if the link is running below its maximum capability

---

## PoE HAT detection (Pi 5, Pi 4B)

pitop detects Power over Ethernet HATs by scanning for power supply devices:

### Data source

```
/sys/class/power_supply/rpi-poe*
```

If a PoE HAT is detected, the Power tab shows whether PoE power is being supplied and the current draw.

### Supported HATs

- Raspberry Pi PoE HAT (Pi 4B)
- Raspberry Pi PoE+ HAT (Pi 5, Pi 4B)

---

## Voltage monitoring (Pi 4B, Zero 2W)

Boards without a PMIC use `vcgencmd measure_volts` for basic voltage readings.

### Data source

```sh
vcgencmd measure_volts core      # SoC core voltage
vcgencmd measure_volts sdram_c   # SDRAM controller voltage
vcgencmd measure_volts sdram_i   # SDRAM I/O voltage
vcgencmd measure_volts sdram_p   # SDRAM PHY voltage
```

Output format: `volt=1.2000V`

---

## Thermal monitoring

### SoC temperature (all boards)

Read from `/sys/class/thermal/thermal_zone0/temp`. The value is in millidegrees Celsius (e.g., `52300` = 52.3C).

### Additional thermal zones (Pi 5)

The Pi 5 exposes additional thermal zones:

| Zone | Description |
|------|-------------|
| SoC | Main processor temperature |
| PMIC | Power management IC temperature |
| RP1 | Southbridge chip temperature |

These are discovered by scanning `/sys/class/thermal/` and matching by the `type` file.

### hwmon temperature sensors

pitop also discovers temperature sensors via `/sys/class/hwmon/` by matching the `name` file content. This catches sensors that are not exposed as thermal zones.

---

## Throttle detection (all Pi boards)

### Data source

```sh
vcgencmd get_throttled
```

Returns a hex bitmask, e.g., `throttled=0x0`.

### Bitmask fields

| Bit | Meaning |
|-----|---------|
| 0 | Under-voltage detected |
| 1 | ARM frequency capped |
| 2 | Currently throttled |
| 3 | Soft temperature limit active |
| 16 | Under-voltage has occurred |
| 17 | ARM frequency capping has occurred |
| 18 | Throttling has occurred |
| 19 | Soft temperature limit has occurred |

Bits 0-3 indicate **current** status. Bits 16-19 indicate events that have occurred **since boot** but may not be active now.

### Display

The header bar shows a color-coded throttle indicator:

- **Green**: `0x0` -- no throttle events
- **Yellow**: bits 16-19 set but bits 0-3 clear -- past events, not currently active
- **Red**: any of bits 0-3 set -- active throttling right now

---

## GPU monitoring (all Pi boards)

GPU data is collected via multiple `vcgencmd` calls:

| Command | Data |
|---------|------|
| `vcgencmd measure_clock core` | GPU core frequency in Hz |
| `vcgencmd get_mem gpu` | GPU memory allocation in MB |
| `vcgencmd measure_temp` | GPU temperature |
| `vcgencmd codec_enabled H264` | H.264 hardware decode status |
| `vcgencmd codec_enabled HEVC` | HEVC/H.265 hardware decode status |

### vcgencmd availability

All vcgencmd calls go through a centralized runner (`VcgencmdRunner`) that:

- Uses `tokio::process::Command` with a 2-second timeout
- Caches results for 1 second minimum to reduce subprocess overhead
- Returns `None` if vcgencmd is not installed or fails
- Never panics on errors
