# Keyboard Shortcuts

## Global shortcuts

These work from any tab:

| Key | Action |
|-----|--------|
| `1` | Switch to Overview tab |
| `2` | Switch to Processes tab |
| `3` | Switch to Power tab |
| `4` | Switch to Network tab |
| `5` | Switch to Disk tab |
| `6` | Switch to System tab |
| `Tab` | Next tab |
| `Shift+Tab` | Previous tab |
| `q` | Quit |
| `Ctrl+C` | Quit (always works, even in overlays) |
| `Space` | Pause / resume data collection |
| `t` | Cycle color theme |
| `?` | Toggle help overlay |

---

## Processes tab

These shortcuts are only active when the Processes tab (tab 2) is selected:

| Key | Action |
|-----|--------|
| `j` or `Down` | Move selection down |
| `k` or `Up` | Move selection up |
| `s` | Cycle sort column (CPU% -> MEM -> PID -> Name -> User) |
| `K` | Kill selected process (shows confirmation) |

### Kill confirmation

When the kill confirmation prompt is active:

| Key | Action |
|-----|--------|
| `y` | Confirm kill (sends SIGTERM) |
| `n` or `Esc` | Cancel |

---

## Help overlay

When the help overlay is open:

| Key | Action |
|-----|--------|
| `?` or `Esc` | Close help |
| `j` or `Down` | Scroll down |
| `k` or `Up` | Scroll up |

---

## Stress testing

These shortcuts are only available when pitop was launched with `--stress`:

| Key | Action |
|-----|--------|
| `Ctrl+S` | Toggle stress test on/off |
| `Ctrl+Up` | Add one worker thread |
| `Ctrl+Down` | Remove one worker thread |

!!! note
    `Ctrl+S` toggles the stress test between running and stopped. `Ctrl+Up/Down` adjusts the worker count only while the stress test is running. The worker count is clamped between 1 and 2x the initial count.

---

## Quick reference

```
Navigation     1-6 / Tab / Shift+Tab
Quit           q / Ctrl+C
Pause          Space
Theme          t
Help           ?
Process nav    j/k / arrows
Process sort   s
Process kill   K -> y/n
Stress toggle  Ctrl+S
Stress workers Ctrl+Up / Ctrl+Down
```
