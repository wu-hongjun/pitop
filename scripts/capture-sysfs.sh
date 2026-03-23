#!/bin/bash
# capture-sysfs.sh — Run this ON a Raspberry Pi to capture test fixture data
# Usage: ./capture-sysfs.sh [output_dir]
#
# Captures snapshots of procfs/sysfs files used by pitop.
# The output becomes test fixture data for unit tests.

set -euo pipefail

# Detect board
if [ -f /sys/firmware/devicetree/base/model ]; then
    MODEL=$(tr -d '\0' < /sys/firmware/devicetree/base/model)
else
    MODEL="unknown"
fi

echo "Detected board: $MODEL"

# Determine output directory
if [ -n "${1:-}" ]; then
    OUTDIR="$1"
elif echo "$MODEL" | grep -qi "Pi 5"; then
    OUTDIR="pi5"
elif echo "$MODEL" | grep -qi "Pi 4"; then
    OUTDIR="pi4b"
elif echo "$MODEL" | grep -qi "Zero 2"; then
    OUTDIR="zero2w"
else
    OUTDIR="unknown"
fi

mkdir -p "$OUTDIR"
echo "Writing fixtures to: $OUTDIR/"

# --- procfs snapshots ---

cp /proc/stat "$OUTDIR/proc-stat" 2>/dev/null || echo "SKIP: /proc/stat"
sleep 1
cp /proc/stat "$OUTDIR/proc-stat-sample2" 2>/dev/null || echo "SKIP: /proc/stat sample2"
mv "$OUTDIR/proc-stat" "$OUTDIR/proc-stat-sample1" 2>/dev/null || true

cp /proc/meminfo "$OUTDIR/proc-meminfo" 2>/dev/null || echo "SKIP: /proc/meminfo"
cp /proc/net/dev "$OUTDIR/proc-net-dev" 2>/dev/null || echo "SKIP: /proc/net/dev"
cp /proc/diskstats "$OUTDIR/proc-diskstats" 2>/dev/null || echo "SKIP: /proc/diskstats"
cp /proc/mounts "$OUTDIR/proc-mounts" 2>/dev/null || echo "SKIP: /proc/mounts"
cp /proc/loadavg "$OUTDIR/proc-loadavg" 2>/dev/null || echo "SKIP: /proc/loadavg"
cp /proc/uptime "$OUTDIR/proc-uptime" 2>/dev/null || echo "SKIP: /proc/uptime"
cp /proc/version "$OUTDIR/proc-version" 2>/dev/null || echo "SKIP: /proc/version"

# --- device tree ---

if [ -f /proc/device-tree/compatible ]; then
    cp /proc/device-tree/compatible "$OUTDIR/device-tree-compatible"
fi

if [ -f /sys/firmware/devicetree/base/model ]; then
    cat /sys/firmware/devicetree/base/model > "$OUTDIR/device-tree-model"
fi

# --- thermal ---

for zone in /sys/class/thermal/thermal_zone*/; do
    if [ -d "$zone" ]; then
        name=$(basename "$zone")
        mkdir -p "$OUTDIR/thermal/$name"
        cat "$zone/temp" > "$OUTDIR/thermal/$name/temp" 2>/dev/null || true
        cat "$zone/type" > "$OUTDIR/thermal/$name/type" 2>/dev/null || true
    fi
done

# --- CPU freq ---

mkdir -p "$OUTDIR/cpufreq"
POLICY="/sys/devices/system/cpu/cpufreq/policy0"
if [ -d "$POLICY" ]; then
    cat "$POLICY/scaling_cur_freq" > "$OUTDIR/cpufreq/scaling_cur_freq" 2>/dev/null || true
    cat "$POLICY/scaling_min_freq" > "$OUTDIR/cpufreq/scaling_min_freq" 2>/dev/null || true
    cat "$POLICY/scaling_max_freq" > "$OUTDIR/cpufreq/scaling_max_freq" 2>/dev/null || true
    cat "$POLICY/scaling_governor" > "$OUTDIR/cpufreq/scaling_governor" 2>/dev/null || true
fi

# --- hwmon ---

mkdir -p "$OUTDIR/hwmon"
for hw in /sys/class/hwmon/hwmon*/; do
    if [ -d "$hw" ]; then
        name=$(cat "$hw/name" 2>/dev/null || echo "unknown")
        hwdir="$OUTDIR/hwmon/$name"
        mkdir -p "$hwdir"
        for f in "$hw"/*; do
            [ -f "$f" ] && cat "$f" > "$hwdir/$(basename "$f")" 2>/dev/null || true
        done
    fi
done

# --- fan (Pi 5) ---

FAN="/sys/devices/platform/cooling_fan"
if [ -d "$FAN" ]; then
    mkdir -p "$OUTDIR/fan"
    find "$FAN" -name "fan1_input" -exec cat {} \; > "$OUTDIR/fan/fan1_input" 2>/dev/null || true
    find "$FAN" -name "pwm1" -exec cat {} \; > "$OUTDIR/fan/pwm1" 2>/dev/null || true
fi

# --- PCIe (Pi 5) ---

mkdir -p "$OUTDIR/pcie"
for dev in /sys/bus/pci/devices/*/; do
    if [ -f "$dev/current_link_speed" ]; then
        addr=$(basename "$dev")
        mkdir -p "$OUTDIR/pcie/$addr"
        cat "$dev/current_link_speed" > "$OUTDIR/pcie/$addr/current_link_speed" 2>/dev/null || true
        cat "$dev/current_link_width" > "$OUTDIR/pcie/$addr/current_link_width" 2>/dev/null || true
        cat "$dev/max_link_speed" > "$OUTDIR/pcie/$addr/max_link_speed" 2>/dev/null || true
        cat "$dev/max_link_width" > "$OUTDIR/pcie/$addr/max_link_width" 2>/dev/null || true
        cat "$dev/vendor" > "$OUTDIR/pcie/$addr/vendor" 2>/dev/null || true
        cat "$dev/device" > "$OUTDIR/pcie/$addr/device" 2>/dev/null || true
        cat "$dev/class" > "$OUTDIR/pcie/$addr/class" 2>/dev/null || true
    fi
done

# --- PoE ---

for ps in /sys/class/power_supply/rpi*; do
    if [ -d "$ps" ]; then
        mkdir -p "$OUTDIR/poe"
        cat "$ps/online" > "$OUTDIR/poe/online" 2>/dev/null || true
        cat "$ps/current_now" > "$OUTDIR/poe/current_now" 2>/dev/null || true
        cat "$ps/current_max" > "$OUTDIR/poe/current_max" 2>/dev/null || true
    fi
done

# --- vcgencmd outputs ---

mkdir -p "$OUTDIR/vcgencmd"
if command -v vcgencmd &>/dev/null; then
    vcgencmd get_throttled > "$OUTDIR/vcgencmd/get_throttled" 2>/dev/null || true
    vcgencmd measure_temp > "$OUTDIR/vcgencmd/measure_temp" 2>/dev/null || true
    vcgencmd measure_clock arm > "$OUTDIR/vcgencmd/measure_clock_arm" 2>/dev/null || true

    # Pi 5 specific
    vcgencmd pmic_read_adc > "$OUTDIR/vcgencmd/pmic_read_adc" 2>/dev/null || true
    vcgencmd measure_temp pmic > "$OUTDIR/vcgencmd/measure_temp_pmic" 2>/dev/null || true

    # Pi 4B specific
    for rail in core sdram_c sdram_i sdram_p; do
        vcgencmd measure_volts "$rail" > "$OUTDIR/vcgencmd/measure_volts_$rail" 2>/dev/null || true
    done
else
    echo "SKIP: vcgencmd not found"
fi

# --- /etc/os-release ---

cp /etc/os-release "$OUTDIR/os-release" 2>/dev/null || true

echo ""
echo "Done. Captured fixtures in: $OUTDIR/"
echo "Copy this directory to tests/fixtures/ in the pitop repo."
ls -la "$OUTDIR/"
