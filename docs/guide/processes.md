# Processes Tab

The Processes tab (tab 2) displays a sortable, interactive table of running processes, similar to `htop`.

---

## Table columns

| Column | Description |
|--------|-------------|
| PID | Process ID |
| Name | Process name (from `/proc/[pid]/comm`) |
| CPU% | CPU usage percentage |
| MEM | Resident set size (RSS) in human-readable format |
| User | Owner of the process |

---

## Navigation

| Key | Action |
|-----|--------|
| `j` or `Down` | Move selection down |
| `k` or `Up` | Move selection up |
| `s` | Cycle sort column |
| `K` | Kill selected process |

The selected row is highlighted with a background color matching the current theme.

---

## Sorting

Press `s` to cycle through sort columns:

1. **CPU%** (descending) -- default sort
2. **MEM** (descending)
3. **PID** (ascending)
4. **Name** (alphabetical)
5. **User** (alphabetical)

The active sort column is indicated in the table header.

---

## Killing a process

1. Navigate to the target process using `j`/`k` or arrow keys
2. Press `K` (uppercase)
3. A confirmation prompt appears showing the PID and process name
4. Press `y` to confirm (sends SIGTERM) or `n`/`Esc` to cancel

!!! warning
    Killing system processes may cause instability. The kill operation sends `SIGTERM` (signal 15), which allows the process to clean up. If the kill fails (e.g., insufficient permissions), an error message is displayed.

After a kill attempt, the result message is shown briefly and clears on the next keypress.

---

## Data collection

Process data is parsed from `/proc/[pid]/` directories:

- `/proc/[pid]/stat` -- CPU usage, state
- `/proc/[pid]/status` -- memory (VmRSS), user
- `/proc/[pid]/comm` -- process name

!!! note "Lazy refresh"
    Process data is only collected when the Processes tab is active. This reduces CPU and I/O overhead when viewing other tabs, which is especially important on lower-powered boards like the Pi Zero 2W.
