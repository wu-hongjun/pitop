# Epic 2: Core Collectors (All Boards)

## Goal
Implement the data collectors that work on every supported board.
These parse standard Linux procfs/sysfs files.

---

## Story 2.1: sysfs/procfs reading utilities

### Description
Create helper functions for safely reading sysfs and procfs files.
These are the foundation for every collector.

### Acceptance criteria
- [ ] `read_sysfs_string(path) -> Result<String>` — reads, trims whitespace
- [ ] `read_sysfs_u64(path) -> Result<u64>` — reads and parses integer
- [ ] `read_sysfs_f64(path) -> Result<f64>` — reads and parses float
- [ ] All functions return descriptive errors on ENOENT/EACCES
- [ ] `discover_hwmon(name: &str) -> Option<PathBuf>` — finds hwmon device by name file
- [ ] Unit tests with temp files

### Files to create
- `src/util/sysfs.rs`

---

## Story 2.2: CPU usage collector

### Description
Parse `/proc/stat` to get per-core and aggregate CPU usage percentages.
Also read current frequency from cpufreq.

### Acceptance criteria
- [ ] Parses all `cpuN` lines from `/proc/stat`
- [ ] Computes usage % from delta between two samples (user+nice+system+irq+softirq+steal vs idle+iowait)
- [ ] Reads current frequency from `/sys/devices/system/cpu/cpufreq/policy0/scaling_cur_freq`
- [ ] Reads min/max frequency and governor
- [ ] Returns `CpuData` struct with per-core usage vec, aggregate usage, freq, governor
- [ ] First call returns 0% (no previous sample), subsequent calls return real delta
- [ ] Unit test with fixture `/proc/stat` snapshots (two samples)

### Files to create
- `src/collectors/cpu.rs`
- `tests/fixtures/pi5/proc-stat-sample1`
- `tests/fixtures/pi5/proc-stat-sample2`

---

## Story 2.3: Memory collector

### Description
Parse `/proc/meminfo` for RAM and swap usage.

### Acceptance criteria
- [ ] Reads MemTotal, MemFree, MemAvailable, Buffers, Cached, SwapTotal, SwapFree
- [ ] Computes used memory as: Total - Available (matches htop behavior)
- [ ] Returns `MemoryData` struct with total, used, free, available, swap_total, swap_used
- [ ] All values in bytes
- [ ] Unit test with fixture meminfo file

### Files to create
- `src/collectors/memory.rs`
- `tests/fixtures/pi5/proc-meminfo`

---

## Story 2.4: Thermal collector

### Description
Read SoC temperature and discover additional thermal zones.

### Acceptance criteria
- [ ] Reads `/sys/class/thermal/thermal_zone0/temp` (millidegrees → float °C)
- [ ] Enumerates all thermal zones, reads their `type` and `temp`
- [ ] On Pi 5: discovers RP1 temp via hwmon (name = `rp1_adc`)
- [ ] On Pi 5: reads PMIC temp via `vcgencmd measure_temp pmic`
- [ ] Returns `ThermalData` with named temperature readings
- [ ] Handles missing zones gracefully (Zero 2W has fewer)
- [ ] Unit test with mock thermal_zone files

### Files to create
- `src/collectors/thermal.rs`

---

## Story 2.5: Network collector

### Description
Parse `/proc/net/dev` for per-interface byte counters and compute throughput.

### Acceptance criteria
- [ ] Parses all interfaces from `/proc/net/dev`
- [ ] Computes rx/tx bytes per second from delta between samples
- [ ] Skips `lo` (loopback) interface by default
- [ ] Returns `NetworkData` with vec of `InterfaceData` (name, rx_bytes, tx_bytes, rx_rate, tx_rate)
- [ ] First call returns 0 rates (no previous sample)
- [ ] Unit test with fixture proc/net/dev snapshots

### Files to create
- `src/collectors/network.rs`
- `tests/fixtures/pi5/proc-net-dev`

---

## Story 2.6: Disk collector

### Description
Read mounted partitions usage and disk I/O rates.

### Acceptance criteria
- [ ] Reads `/proc/mounts` for mounted filesystems
- [ ] Filters out pseudo-filesystems (sysfs, proc, tmpfs, devtmpfs, squashfs)
- [ ] Uses `statvfs` (via `libc` or manual syscall) for total/used/free per mount
- [ ] Parses `/proc/diskstats` for read/write sectors, computes I/O rates from delta
- [ ] Returns `DiskData` with partition list and I/O rate list
- [ ] Unit test with fixture files

### Files to create
- `src/collectors/disk.rs`

---

## Story 2.7: Throttle status collector

### Description
Read the throttle bitmask from vcgencmd and decode it.

### Acceptance criteria
- [ ] Calls `vcgencmd get_throttled` via the vcgencmd utility module
- [ ] Parses hex output (e.g., `throttled=0x50005`)
- [ ] Decodes all 8 flags: undervoltage, freq capped, throttled, soft temp limit (current + since boot)
- [ ] Returns `ThrottleData` struct with named boolean fields
- [ ] Returns all-false if vcgencmd is unavailable
- [ ] Unit test with various hex values

### Files to create
- `src/collectors/throttle.rs`

---

## Story 2.8: Process collector

### Description
Scan `/proc/` for running processes and compute per-process stats.

### Acceptance criteria
- [ ] Scans `/proc/[0-9]+/` directories
- [ ] Reads PID, name (from `/proc/PID/comm`), state, RSS, user (from `/proc/PID/status`)
- [ ] Computes per-process CPU% from `/proc/PID/stat` utime+stime deltas
- [ ] Returns top N processes sorted by CPU% (default N=20)
- [ ] Handles processes that disappear between reads (ENOENT)
- [ ] Unit test with mock proc directory structure

### Files to create
- `src/collectors/process.rs`

---

## Story 2.9: vcgencmd utility module

### Description
Async wrapper for calling vcgencmd with caching and timeout.

### Acceptance criteria
- [ ] `vcgencmd(args: &[&str]) -> Result<Option<String>>` — runs command, returns stdout
- [ ] 2-second timeout on subprocess
- [ ] Caches results for 1 second (configurable)
- [ ] Returns `None` (not error) if vcgencmd binary not found
- [ ] Returns `None` on timeout
- [ ] Thread-safe (can be called from multiple collectors)
- [ ] Unit test with mock command (or integration test on real Pi)

### Files to create
- `src/util/vcgencmd.rs`

---

## Story 2.10: Ring buffer for sparkline history

### Description
Fixed-size circular buffer that stores the last N samples for sparkline rendering.

### Acceptance criteria
- [ ] `RingBuffer<T>` with configurable capacity (default 60)
- [ ] `push(value: T)` — adds sample, evicts oldest if full
- [ ] `as_slice() -> &[T]` — returns samples in chronological order
- [ ] `len()`, `is_empty()`, `capacity()`
- [ ] Implements `Default` with capacity 60
- [ ] Unit test covering push, wrap-around, and ordering

### Files to create
- `src/util/ring_buffer.rs`
