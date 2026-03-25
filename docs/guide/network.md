# Network Tab

The Network tab (tab 4) provides detailed information about all network interfaces on the system.

---

## Interface list

Each network interface is displayed with:

| Field | Description |
|-------|-------------|
| Name | Interface name (e.g., `eth0`, `wlan0`) |
| Status | Link state (Up / Down) |
| IPv4 | IPv4 address |
| IPv6 | IPv6 address (if configured) |
| MAC | Hardware MAC address |
| RX rate | Current receive throughput (bytes/sec) |
| TX rate | Current transmit throughput (bytes/sec) |

The loopback interface (`lo`) is filtered out by default.

---

## Throughput sparklines

Each interface includes RX and TX sparklines showing throughput history over the last 60 samples (configurable via `history_size` in the config file).

Rates are displayed in human-readable format:

- Bytes/sec for low traffic
- KB/s, MB/s, GB/s as appropriate

---

## Data source

Network data is parsed from `/proc/net/dev`, which provides cumulative byte and packet counters for each interface. pitop calculates the per-second rate by comparing consecutive samples at each tick interval.

---

## Interface discovery

Interfaces are discovered dynamically each tick. New interfaces (e.g., a USB Ethernet adapter plugged in) appear automatically. Removed interfaces are pruned from the display and their sparkline history is cleaned up to prevent unbounded memory growth.

!!! tip
    On a typical Raspberry Pi, you will see:

    - `eth0` -- wired Ethernet (Pi 4B, Pi 5)
    - `wlan0` -- Wi-Fi
    - `end0` -- wired Ethernet (newer naming on some Pi 5 images)
