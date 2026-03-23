# Epic 1: Board Detection and Hardware Profiles

## Goal
Detect which Raspberry Pi board is running at startup and configure
available collectors accordingly.

---

## Story 1.1: Read device-tree compatible string

### Description
Read `/proc/device-tree/compatible` at startup and parse it to identify
the board. The file contains null-separated strings like
`raspberrypi,5-model-b\0brcm,bcm2712\0`.

### Acceptance criteria
- [ ] Reads `/proc/device-tree/compatible` and parses null-separated values
- [ ] Identifies Pi 5 by `brcm,bcm2712`
- [ ] Identifies Pi 4B by `brcm,bcm2711`
- [ ] Identifies Zero 2W by `brcm,bcm2710`
- [ ] Returns `BoardType::Unknown` if no match
- [ ] Falls back to `/sys/firmware/devicetree/base/model` for display name
- [ ] Unit test with mock file content for each board type

### Files to create/modify
- `src/board/mod.rs` — `BoardType` enum, `detect()` function
- `tests/fixtures/pi5/device-tree-compatible` — mock data
- `tests/fixtures/pi4b/device-tree-compatible` — mock data
- `tests/fixtures/zero2w/device-tree-compatible` — mock data

---

## Story 1.2: Board capability profiles

### Description
Each board type maps to a set of available capabilities that determines
which collectors are activated.

### Acceptance criteria
- [ ] `BoardProfile` trait with methods: `has_pmic()`, `has_fan()`, `has_pcie()`, `has_poe()`, `thermal_zones()`, `name()`
- [ ] `Pi5Profile` implements all capabilities as available
- [ ] `Pi4BProfile` has PoE but no PMIC/fan/PCIe
- [ ] `Zero2WProfile` has minimal capabilities
- [ ] `UnknownProfile` enables only generic Linux collectors
- [ ] Profile is selected based on `detect()` result from Story 1.1
- [ ] Unit test verifying each profile's capability flags

### Files to create/modify
- `src/board/pi5.rs`
- `src/board/pi4b.rs`
- `src/board/zero2w.rs`

---

## Story 1.3: Human-readable board info

### Description
Gather static system information for display on the System tab.

### Acceptance criteria
- [ ] Read board model string from `/sys/firmware/devicetree/base/model`
- [ ] Read kernel version from `/proc/version`
- [ ] Read hostname from `/proc/sys/kernel/hostname`
- [ ] Read architecture from `uname` or `/proc/cpuinfo`
- [ ] Read OS info from `/etc/os-release`
- [ ] All reads gracefully handle missing files
- [ ] Stored in a `SystemInfo` struct available to the UI

### Files to create/modify
- `src/collectors/system_info.rs` (or extend `src/board/mod.rs`)
