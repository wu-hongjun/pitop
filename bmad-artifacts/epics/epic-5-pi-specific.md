# Epic 5: Pi-Specific Features

## Goal
Implement hardware features unique to specific Pi boards.
All features must degrade gracefully on boards that lack them.

---

## Story 5.1: PMIC power rails (Pi 5)

### Acceptance criteria
- [ ] Calls `vcgencmd pmic_read_adc` and parses all 12 current + 12 voltage rails
- [ ] Displays per-rail table: rail name, voltage (V), current (A), power (W)
- [ ] Computes total estimated wattage: `sum(V×I) × 1.1451 + 0.5879`
- [ ] Total wattage sparkline in history
- [ ] Shows EXT5V_V (input voltage) and BATT_V (RTC battery)
- [ ] Gracefully returns empty data on Pi 4B / Zero 2W
- [ ] Renders in the Power tab

### Files: `src/collectors/power.rs`, `src/ui/power.rs`

---

## Story 5.2: Fan monitoring (Pi 5)

### Acceptance criteria
- [ ] Discovers fan hwmon device by enumerating `/sys/class/hwmon/` for name `cooling_fan` or by checking `/sys/devices/platform/cooling_fan/`
- [ ] Reads `fan1_input` (RPM) and `pwm1` (duty 0-255, convert to %)
- [ ] Displays on Overview tab: "Fan: 3054 RPM (45%)"
- [ ] Returns None on boards without fan header
- [ ] Does not error when fan device hwmon number changes

### Files: `src/collectors/fan.rs`

---

## Story 5.3: PCIe link detection (Pi 5)

### Acceptance criteria
- [ ] Scans `/sys/bus/pci/devices/` for endpoint devices (skip bridges)
- [ ] For each device: reads `current_link_speed`, `current_link_width`, `max_link_speed`, `max_link_width`
- [ ] Maps speed strings to generation labels: "2.5 GT/s" → Gen 1, "5.0 GT/s" → Gen 2, "8.0 GT/s" → Gen 3
- [ ] Reads device name from `vendor` and `device` files (hex IDs)
- [ ] Detects downgraded links: current < max → show "(downgraded)" label
- [ ] Displays in Power tab: "PCIe: NVMe SSD — Gen 3 x1 (8.0 GT/s)"
- [ ] Returns empty list on boards without PCIe (Zero 2W, Pi 4B)

### Files: `src/collectors/pcie.rs`

---

## Story 5.4: PoE HAT detection (Pi 5, Pi 4B)

### Acceptance criteria
- [ ] Checks for `/sys/class/power_supply/rpi-poe*/` or similar PoE power supply device
- [ ] If present: reads `online` (boolean), `current_now` (µA), `current_max`
- [ ] Displays in Power tab: "PoE: Active, 1.2A draw"
- [ ] Returns None when no PoE HAT detected
- [ ] Works on both Pi 5 and Pi 4B PoE/PoE+ HATs

### Files: `src/collectors/poe.rs`

---

## Story 5.5: Pi 4B voltage readings

### Acceptance criteria
- [ ] Calls `vcgencmd measure_volts` for: core, sdram_c, sdram_i, sdram_p
- [ ] Parses `volt=X.XXXXV` format
- [ ] Displays in Power tab as simple voltage table
- [ ] Only runs on Pi 4B (and unknown boards as fallback)
- [ ] Pi 5 uses PMIC data instead (Story 5.1)

### Files: extend `src/collectors/power.rs`
