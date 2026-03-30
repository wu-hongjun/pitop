# Board Support

pitop auto-detects the Raspberry Pi board at startup and activates the appropriate collectors and UI sections.

---

## Supported boards

| Board | SoC | Detection String | Features |
|-------|-----|-----------------|----------|
| Raspberry Pi 5 | BCM2712 | `brcm,bcm2712` | Full: PMIC, fan, PCIe, PoE, GPU |
| Raspberry Pi 4 Model B | BCM2711 | `brcm,bcm2711` | Voltages, PoE, GPU |
| Raspberry Pi 400 | BCM2711 | `brcm,bcm2711` | Voltages, GPU (same as Pi 4B) |
| Compute Module 4 | BCM2711 | `brcm,bcm2711` | Voltages, GPU (same as Pi 4B) |
| Raspberry Pi 3 Model B+ | BCM2837 | `brcm,bcm2837` | Basic monitoring, GPU |
| Raspberry Pi 3 Model B | BCM2837 | `brcm,bcm2837` | Basic monitoring, GPU |
| Raspberry Pi 3 Model A+ | BCM2837 | `brcm,bcm2837` | Basic monitoring, GPU |
| Raspberry Pi 2 Model B | BCM2836 | `brcm,bcm2836` | Basic monitoring, GPU |
| Raspberry Pi 1 | BCM2835 | `brcm,bcm2835` | Basic monitoring |
| Raspberry Pi Zero 2 W | BCM2710/BCM2837 | `brcm,bcm2710` or `brcm,bcm2837` | Basic monitoring, GPU |
| Raspberry Pi Zero W | BCM2835 | `brcm,bcm2835` | Basic monitoring |
| Raspberry Pi Zero | BCM2835 | `brcm,bcm2835` | Basic monitoring |
| Generic Linux | -- | (no match) | CPU, memory, network, disk |

---

## Board detection

At startup, pitop reads `/proc/device-tree/compatible` to identify the board. This file contains null-separated strings like:

```
raspberrypi,5-model-b\0brcm,bcm2712\0
```

pitop matches against both the SoC identifier and the model string:

| Detection string | Board type |
|-----------|------------|
| `brcm,bcm2712` | Pi 5 |
| `brcm,bcm2711` | Pi 4B / Pi 400 / CM4 |
| `brcm,bcm2837` | Pi 3B+ / Pi 3B / Pi 3A+ / Zero 2W |
| `brcm,bcm2836` | Pi 2B |
| `brcm,bcm2835` | Pi 1 / Zero W / Zero |
| `brcm,bcm2710` | Zero 2W (alternate) |
| `raspberrypi,5-model-b` | Pi 5 |
| `raspberrypi,4-model-b` | Pi 4B |
| `raspberrypi,400` | Pi 400 |
| `raspberrypi,4-compute-module` | CM4 |
| `raspberrypi,3-model-b-plus` | Pi 3B+ |
| `raspberrypi,3-model-b` | Pi 3B |
| `raspberrypi,3-model-a-plus` | Pi 3A+ |
| `raspberrypi,2-model-b` | Pi 2B |
| `raspberrypi,model-zero-2-w` | Zero 2W |
| `raspberrypi,model-zero-w` | Zero W |
| `raspberrypi,model-zero` | Zero |

If no match is found, pitop runs in **generic Linux mode** with only universal collectors active.

### Fallback display name

The human-readable board name (shown in the header bar and System tab) is read from `/sys/firmware/devicetree/base/model`, which contains a string like "Raspberry Pi 5 Model B Rev 1.0".

---

## Forcing a board type

You can override auto-detection with the `--board` flag:

```sh
pitop --board pi5     # Force Pi 5 mode
pitop --board pi4b    # Force Pi 4B mode (also covers Pi 400, CM4)
pitop --board pi3b    # Force Pi 3B/3B+/3A+ mode
pitop --board pi2b    # Force Pi 2B mode
pitop --board zero2w  # Force Zero 2W mode
pitop --board zero    # Force Zero/Zero W mode
pitop --board auto    # Auto-detect (default)
```

!!! warning
    Forcing a board type on hardware that does not match will cause Pi-specific collectors to fail gracefully. Features that depend on hardware not present (e.g., PMIC on a Pi 4B) will show no data rather than crashing.

---

## Feature matrix

| Feature | Pi 5 | Pi 4B / 400 / CM4 | Pi 3B+ / 3B / 3A+ | Pi 2B | Zero 2W | Zero / Zero W | Generic |
|---------|------|--------------------|--------------------|-------|---------|---------------|---------|
| CPU usage & frequency | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Memory & swap | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| SoC temperature | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| PMIC temperature | Yes | -- | -- | -- | -- | -- | -- |
| RP1 temperature | Yes | -- | -- | -- | -- | -- | -- |
| NVMe temperature | Yes | -- | -- | -- | -- | -- | -- |
| Network interfaces | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Disk partitions & I/O | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Process table | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| PMIC power rails | Yes | -- | -- | -- | -- | -- | -- |
| Voltage readings | -- | Yes | Yes | Yes | Yes | Yes | -- |
| Fan speed | Yes | -- | -- | -- | -- | -- | -- |
| PCIe link status | Yes | -- | -- | -- | -- | -- | -- |
| PoE HAT status | Yes | Yes | -- | -- | -- | -- | -- |
| GPU monitoring | Yes | Yes | Yes | Yes | Yes | Yes | -- |
| Throttle detection | Yes | Yes | Yes | Yes | Yes | Yes | -- |

---

## CPU specifications

| Board | Cores | Architecture | Max frequency |
|-------|-------|-------------|---------------|
| Pi 5 | 4x Cortex-A76 (0xd0b) | ARMv8.2-A (aarch64) | 2.4 GHz |
| Pi 4B / Pi 400 / CM4 | 4x Cortex-A72 (0xd08) | ARMv8-A (aarch64/armv7l) | 1.8 GHz |
| Pi 3B+ / Pi 3B / Pi 3A+ | 4x Cortex-A53 (0xd03) | ARMv8-A (aarch64) | 1.4 GHz |
| Pi 2B | 4x Cortex-A7 (0xc07) | ARMv7-A (armv7l) | 900 MHz |
| Zero 2W | 4x Cortex-A53 (0xd03) | ARMv8-A (aarch64) | 1.0 GHz |
| Pi 1 / Zero / Zero W | 1x ARM1176JZF-S (0xb76) | ARMv6 (armv6l) | 1.0 GHz |

pitop detects CPU model names on ARM64 by reading the CPU part number from `/proc/cpuinfo` and mapping it (e.g., `0xd03` = Cortex-A53, `0xd0b` = Cortex-A76).

---

## Graceful degradation

Every hardware-specific feature handles missing sysfs paths and unavailable commands without crashing:

- If a sysfs file does not exist (`ENOENT`), the collector returns empty data
- If a sysfs file is not readable (`EACCES`), the collector skips it
- If `vcgencmd` is not installed, all vcgencmd-dependent features are silently disabled
- UI sections with no data are either hidden or display a "not available" message

This means pitop can run on any Linux system -- it will simply show fewer features on non-Pi hardware.
