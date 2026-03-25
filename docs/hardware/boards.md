# Board Support

pitop auto-detects the Raspberry Pi board at startup and activates the appropriate collectors and UI sections.

---

## Supported boards

| Board | SoC | Detection String | Features |
|-------|-----|-----------------|----------|
| Raspberry Pi 5 | BCM2712 | `brcm,bcm2712` | Full: PMIC, fan, PCIe, PoE, GPU |
| Raspberry Pi 4 Model B | BCM2711 | `brcm,bcm2711` | Voltages, PoE, GPU |
| Raspberry Pi Zero 2 W | BCM2710 | `brcm,bcm2710` | Basic monitoring, GPU |
| Generic Linux | -- | (no match) | CPU, memory, network, disk |

---

## Board detection

At startup, pitop reads `/proc/device-tree/compatible` to identify the board. This file contains null-separated strings like:

```
raspberrypi,5-model-b\0brcm,bcm2712\0
```

pitop matches against the SoC identifier:

| SoC string | Board type |
|-----------|------------|
| `brcm,bcm2712` | Pi 5 |
| `brcm,bcm2711` | Pi 4B |
| `brcm,bcm2710` | Zero 2W |

If no match is found, pitop runs in **generic Linux mode** with only universal collectors active.

### Fallback display name

The human-readable board name (shown in the header bar and System tab) is read from `/sys/firmware/devicetree/base/model`, which contains a string like "Raspberry Pi 5 Model B Rev 1.0".

---

## Forcing a board type

You can override auto-detection with the `--board` flag:

```sh
pitop --board pi5     # Force Pi 5 mode
pitop --board pi4b    # Force Pi 4B mode
pitop --board zero2w  # Force Zero 2W mode
pitop --board auto    # Auto-detect (default)
```

!!! warning
    Forcing a board type on hardware that does not match will cause Pi-specific collectors to fail gracefully. Features that depend on hardware not present (e.g., PMIC on a Pi 4B) will show no data rather than crashing.

---

## Feature matrix

| Feature | Pi 5 | Pi 4B | Zero 2W | Generic |
|---------|------|-------|---------|---------|
| CPU usage & frequency | Yes | Yes | Yes | Yes |
| Memory & swap | Yes | Yes | Yes | Yes |
| SoC temperature | Yes | Yes | Yes | Yes |
| PMIC temperature | Yes | -- | -- | -- |
| RP1 temperature | Yes | -- | -- | -- |
| Network interfaces | Yes | Yes | Yes | Yes |
| Disk partitions & I/O | Yes | Yes | Yes | Yes |
| Process table | Yes | Yes | Yes | Yes |
| PMIC power rails | Yes | -- | -- | -- |
| Voltage readings | -- | Yes | Yes | -- |
| Fan speed | Yes | -- | -- | -- |
| PCIe link status | Yes | -- | -- | -- |
| PoE HAT status | Yes | Yes | -- | -- |
| GPU monitoring | Yes | Yes | Yes | -- |
| Throttle detection | Yes | Yes | Yes | -- |

---

## CPU specifications

| Board | Cores | Architecture | Max frequency |
|-------|-------|-------------|---------------|
| Pi 5 | 4x Cortex-A76 | ARMv8.2-A (aarch64) | 2.4 GHz |
| Pi 4B | 4x Cortex-A72 | ARMv8-A (aarch64/armv7l) | 1.8 GHz |
| Zero 2W | 4x Cortex-A53 | ARMv8-A (aarch64) | 1.0 GHz |

---

## Graceful degradation

Every hardware-specific feature handles missing sysfs paths and unavailable commands without crashing:

- If a sysfs file does not exist (`ENOENT`), the collector returns empty data
- If a sysfs file is not readable (`EACCES`), the collector skips it
- If `vcgencmd` is not installed, all vcgencmd-dependent features are silently disabled
- UI sections with no data are either hidden or display a "not available" message

This means pitop can run on any Linux system -- it will simply show fewer features on non-Pi hardware.
