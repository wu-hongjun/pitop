# Stress Testing

pitop includes a built-in CPU stress test for benchmarking and thermal testing your Raspberry Pi.

---

## Starting a stress test

### From the command line

```sh
# Start with stress test enabled (uses all CPU cores)
pitop --stress

# Start with a specific number of worker threads
pitop --stress --stress-workers 2
```

### At runtime

If pitop was launched with `--stress`, you can toggle the stress test on and off:

| Key | Action |
|-----|--------|
| `Ctrl+S` | Toggle stress test on/off |
| `Ctrl+Up` | Add one worker thread |
| `Ctrl+Down` | Remove one worker thread |

!!! note
    The stress test controls (`Ctrl+S`, `Ctrl+Up/Down`) are only available when pitop was launched with the `--stress` flag. Without it, these key bindings have no effect.

---

## How it works

The stress test spawns CPU-intensive worker threads that perform prime number computation via trial division. Each worker runs in a `tokio::task::spawn_blocking` task with its own atomic stop flag, so workers can be individually started and stopped without affecting the rest of the application.

### Worker count

- **Default**: one worker per CPU core (detected from `/proc/cpuinfo`)
- **Minimum**: 1 worker
- **Maximum**: 2x the initial worker count

For example, on a Pi 5 with 4 cores, the default is 4 workers with a maximum of 8.

---

## Monitoring under stress

The stress test is designed to be used alongside pitop's monitoring features:

- **Overview tab** -- watch CPU gauges hit 100% across all cores
- **Overview tab** -- watch temperature climb in real-time via the sparkline
- **Power tab** -- monitor power draw increase on Pi 5 PMIC rails
- **Header bar** -- check for throttle events (under-voltage, thermal throttling)

This makes it easy to test:

- **Thermal performance** -- does your cooling solution keep temperatures safe?
- **Power supply adequacy** -- does the Pi throttle due to under-voltage under load?
- **System stability** -- does the system remain responsive under full load?

---

## Footer indicator

When a stress test is active, the footer bar shows the stress test status:

```
[STRESS 4/8 workers 02:30]
```

This shows:

- Current worker count / maximum workers
- Elapsed time since the stress test started

The indicator is color-coded:

| Color | Meaning |
|-------|---------|
| Green | Fewer workers than the initial count |
| Yellow | Workers equal to the initial count |
| Red | More workers than the initial count |

When the stress test is stopped, the footer shows `[STRESS STOPPED]`.

---

## Stopping the stress test

Press `Ctrl+S` to stop all workers. Workers are signaled to stop via atomic flags and terminate within a few milliseconds. The CPU load should drop almost immediately.

Closing pitop (via `q` or `Ctrl+C`) also stops all stress test workers automatically.
