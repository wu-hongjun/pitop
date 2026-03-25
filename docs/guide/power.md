# Power Tab

The Power tab (tab 3) displays power-related data. The content varies significantly depending on which board is detected.

---

## Pi 5: PMIC power rails

The Raspberry Pi 5 includes a PMIC (Power Management IC) that provides detailed per-rail power telemetry via `vcgencmd pmic_read_adc`.

### Rail table

The tab displays a table with all 12 voltage and current rail pairs:

| Column | Description |
|--------|-------------|
| Rail | Rail name (e.g., VDD_CORE, VDD_CPU) |
| Voltage | Rail voltage in volts |
| Current | Rail current in amps |
| Power | Calculated power in watts (V x I) |

### Key rails

- **VDD_CORE** -- main SoC core voltage
- **VDD_CPU** -- CPU power rail
- **VDD_IO** -- I/O voltage
- **EXT5V_V** -- input voltage from the USB-C power supply
- **BATT_V** -- RTC battery voltage (if present)

### Estimated total wattage

The estimated real power draw is calculated as:

```
total_watts = sum(V x I for all rails) * 1.1451 + 0.5879
```

This includes a correction factor to account for PMIC conversion losses.

A **sparkline** tracks total wattage over time.

---

## Pi 4B: Voltage readings

On the Raspberry Pi 4 Model B, power monitoring uses `vcgencmd measure_volts` to read voltage levels for:

- **core** -- SoC core voltage
- **sdram_c** -- SDRAM controller voltage
- **sdram_i** -- SDRAM I/O voltage
- **sdram_p** -- SDRAM PHY voltage

!!! note
    The Pi 4B does not have a PMIC with current sensing, so only voltage values are displayed. Current and wattage calculations are not available.

---

## Pi Zero 2W

The Pi Zero 2W supports the same `measure_volts` readings as the Pi 4B (core and SDRAM voltages).

---

## PCIe link status (Pi 5)

If the Pi 5 has a PCIe device connected (e.g., an NVMe SSD via HAT+), this section shows:

- **Link generation** (e.g., Gen 2, Gen 3)
- **Lane width** (e.g., x1)
- **Downgrade detection** -- warns if the link is running below its maximum capability

PCIe data is read from `/sys/bus/pci/devices/*/current_link_speed` and `/sys/bus/pci/devices/*/current_link_width`.

---

## PoE status (Pi 5, Pi 4B)

If a PoE (Power over Ethernet) HAT is detected, this section shows:

- **PoE status** -- whether power is being supplied via PoE
- **Current draw** -- from `/sys/class/power_supply/rpi-poe*`

!!! note
    PoE detection requires a PoE HAT or PoE+ HAT to be installed. If no PoE hardware is detected, this section is hidden.

---

## Graceful degradation

The Power tab adapts to available hardware:

| Board | PMIC rails | Voltages | PCIe | PoE |
|-------|-----------|----------|------|-----|
| Pi 5 | Yes | -- | Yes | Yes |
| Pi 4B | -- | Yes | -- | Yes |
| Zero 2W | -- | Yes | -- | -- |
| Generic Linux | -- | -- | -- | -- |

On boards without any power features, the tab displays a message indicating that no power data is available.
