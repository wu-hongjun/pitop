# Themes

pitop ships with three built-in color themes and supports fully custom themes via the config file.

---

## Built-in themes

### Default

The default theme uses bright, distinct colors optimized for dark terminal backgrounds:

- Green/yellow/red gauge colors for threshold levels
- Blue sparklines for CPU, magenta for memory, red for temperature
- Cyan highlighted borders for the active section
- Yellow titles and white text

### Monochrome

A theme using only white and gray shades. Useful for:

- Terminals with limited color support
- High-contrast accessibility
- Screenshots and documentation

### Solarized

A theme based on the [Solarized Dark](https://ethanschoonover.com/solarized/) color palette. Uses the standard Solarized base and accent colors:

- Base0/base01 for text
- Solarized blue, green, yellow, red, magenta for accents
- Base02 for highlight backgrounds

---

## Switching themes

### At runtime

Press `t` to cycle through available themes: default -> monochrome -> solarized -> (custom, if defined) -> default.

### Via CLI

```sh
pitop --theme solarized
```

### Via config file

```toml
[general]
theme = "solarized"
```

---

## Custom themes

Define a custom theme in the config file under `[custom_theme]`. All fields are optional -- any field you omit falls back to the default theme's color.

### Available color fields

#### General

| Field | Description | Default |
|-------|-------------|---------|
| `border` | Border color for panels | white |
| `border_highlight` | Highlighted/active border | cyan |
| `title` | Title text color | yellow |
| `text` | Normal text color | white |
| `text_dim` | Dimmed/secondary text | darkgray |
| `highlight_bg` | Background for selected items | darkgray |

#### Gauges

| Field | Description | Default |
|-------|-------------|---------|
| `gauge_low` | Gauge fill below warning | green |
| `gauge_warn` | Gauge fill at warning level | yellow |
| `gauge_crit` | Gauge fill at critical level | red |

#### Sparklines

| Field | Description | Default |
|-------|-------------|---------|
| `sparkline_cpu` | CPU sparkline color | blue |
| `sparkline_mem` | Memory sparkline color | magenta |
| `sparkline_temp` | Temperature sparkline color | red |
| `sparkline_power` | Power sparkline color | yellow |

#### Section borders

| Field | Description | Default |
|-------|-------------|---------|
| `cpu_border` | CPU section border | blue |
| `mem_border` | Memory section border | magenta |
| `temp_border` | Temperature section border | green |
| `net_border` | Network section border | green |
| `power_border` | Power section border | yellow |

#### Status indicators

| Field | Description | Default |
|-------|-------------|---------|
| `throttle_ok` | No throttle events | green |
| `throttle_warn` | Past throttle events | yellow |
| `throttle_crit` | Active throttling | red |

---

### Color format

Colors can be specified as:

- **Named colors**: `red`, `green`, `blue`, `yellow`, `cyan`, `magenta`, `white`, `gray`, `darkgray`
- **Hex RGB**: `"#RRGGBB"` format (e.g., `"#FF8800"`, `"#2AA198"`)

### Example custom theme

```toml
[general]
theme = "custom"

[custom_theme]
border = "cyan"
border_highlight = "#FF8800"
title = "#FF8800"
text = "white"
text_dim = "gray"
highlight_bg = "#1A1A2E"
gauge_low = "#00FF00"
gauge_warn = "#FFAA00"
gauge_crit = "#FF0000"
sparkline_cpu = "#4488FF"
sparkline_mem = "#FF44FF"
sparkline_temp = "#FF4444"
sparkline_power = "#FFAA44"
cpu_border = "#4488FF"
mem_border = "#FF44FF"
temp_border = "#44FF44"
net_border = "#44FF44"
power_border = "#FFAA44"
throttle_ok = "#00FF00"
throttle_warn = "#FFAA00"
throttle_crit = "#FF0000"
```

!!! tip
    You do not need to specify every field. Only override the colors you want to change:

    ```toml
    [general]
    theme = "custom"

    [custom_theme]
    title = "#FF8800"
    sparkline_cpu = "cyan"
    ```

    All other fields will use the default theme's colors.

!!! warning
    Setting `theme = "custom"` without defining a `[custom_theme]` section will produce a validation error. Use `pitop --config-check` to verify your config.
