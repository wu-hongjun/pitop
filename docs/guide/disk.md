# Disk Tab

The Disk tab (tab 5) shows partition usage and I/O throughput.

---

## Partition usage

Each mounted partition is displayed with:

| Field | Description |
|-------|-------------|
| Device | Block device name (e.g., `/dev/mmcblk0p2`) |
| Mount point | Where the partition is mounted (e.g., `/`) |
| Filesystem | Filesystem type (e.g., ext4, vfat) |
| Used / Total | Space used and total capacity |
| Usage % | Percentage of space used, with gauge bar |

### Color coding

| Color | Meaning |
|-------|---------|
| Green | Below warning threshold (default: < 70%) |
| Yellow | At or above warning (default: >= 70%) |
| Red | At or above critical (default: >= 90%) |

Thresholds are configurable in the config file under `[thresholds.disk]`.

---

## I/O throughput

Disk I/O statistics are parsed from `/proc/diskstats` and show:

- **Read rate** -- bytes read per second
- **Write rate** -- bytes written per second

Rates are displayed in human-readable format (B/s, KB/s, MB/s).

---

## Data collection

!!! note "Lazy refresh"
    Disk data is only collected when the Disk tab is active. This is particularly important on the Pi Zero 2W where filesystem operations can be relatively expensive.

Partition information comes from scanning mount points, while I/O statistics are derived from `/proc/diskstats` by computing the difference between consecutive reads.
