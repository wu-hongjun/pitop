# Epic 4: Remaining Tabs

## Goal
Implement the Processes, Network, Disk, and System tabs.

---

## Story 4.1: Processes tab

### Acceptance criteria
- [ ] Table with columns: PID, Name, CPU%, MEM%, User
- [ ] Default sort by CPU% descending
- [ ] Press `s` to cycle sort: CPU% → MEM% → PID → Name
- [ ] Vim navigation: j/k or arrow keys to scroll
- [ ] Selected row highlighted
- [ ] Press `K` on selected process → confirmation prompt → kill -15
- [ ] Shows top 50 processes (scrollable)
- [ ] Refreshes only when tab is active

### Files: `src/ui/processes.rs`

---

## Story 4.2: Network tab

### Acceptance criteria
- [ ] Lists each network interface with: name, status (Up/Down), IPv4, IPv6, MAC
- [ ] Per-interface rx/tx throughput sparklines
- [ ] Active connection count
- [ ] Filters out loopback by default
- [ ] Refreshes only when tab is active

### Files: `src/ui/network.rs`

---

## Story 4.3: Disk tab

### Acceptance criteria
- [ ] Partition table: device, mountpoint, total, used, free, usage% (gauge bar)
- [ ] Per-disk I/O section: read KB/s, write KB/s
- [ ] Filters out squashfs, tmpfs, devtmpfs, sysfs, proc
- [ ] Color-coded usage: green < 70%, yellow < 90%, red >= 90%
- [ ] Refreshes only when tab is active

### Files: `src/ui/disk.rs`

---

## Story 4.4: System info tab

### Acceptance criteria
- [ ] Board model and revision
- [ ] SoC name (BCM2712 / BCM2711 / BCM2710)
- [ ] Kernel version
- [ ] OS name and version (from /etc/os-release)
- [ ] Architecture
- [ ] Hostname
- [ ] Uptime (human-readable)
- [ ] CPU: model name, core count, frequency range, governor
- [ ] Total RAM / Total swap
- [ ] Static info — collected once at startup, not refreshed

### Files: `src/ui/system.rs`
