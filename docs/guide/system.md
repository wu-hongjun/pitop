# System Tab

The System tab (tab 6) displays static system information that does not change during runtime.

---

## Displayed information

| Field | Source | Example |
|-------|--------|---------|
| Board model | `/sys/firmware/devicetree/base/model` | Raspberry Pi 5 Model B Rev 1.0 |
| SoC | Board detection | BCM2712 |
| Architecture | `uname` | aarch64 |
| Kernel | `/proc/version` | Linux 6.6.31-v8+ |
| OS | `/etc/os-release` | Debian GNU/Linux 12 (bookworm) |
| Hostname | `/proc/sys/kernel/hostname` | raspberrypi |
| Uptime | `/proc/uptime` | 3d 14h 22m |
| CPU cores | `/proc/cpuinfo` | 4 |

---

## Hardware capabilities

The tab also shows which hardware-specific features are available on the detected board:

| Feature | Pi 5 | Pi 4B | Zero 2W |
|---------|------|-------|---------|
| PMIC power rails | Yes | -- | -- |
| Fan monitoring | Yes | -- | -- |
| PCIe link | Yes | -- | -- |
| PoE HAT | Yes | Yes | -- |
| Voltage monitoring | PMIC | vcgencmd | vcgencmd |
| Throttle detection | Yes | Yes | Yes |
| GPU monitoring | Yes | Yes | Yes |

---

## Static data

Unlike other tabs, the System tab does not refresh data each tick because the displayed information is static or changes very infrequently (only uptime updates). This means switching to the System tab incurs no additional CPU cost from collectors.
