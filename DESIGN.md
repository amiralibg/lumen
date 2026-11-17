# lumen design tokens

This is the single source of truth for how lumen looks. Widgets read tokens only; no widget may contain a literal color, radius, duration or spring constant. Per-widget geometry lives in `docs/widgets-batch1.md`.

All sizes are logical pixels. Hex is the canonical value; the OKLCH triple (`L C H`) is derived from the hex and is there for tuning and for the OKLab blend.

## 1. Principles

1. **Neutral first.** The UI is a cool gray ramp (hue ~265, chroma <= 0.014). Color means something: accent marks selected, primary, progress and focus; status colors mark status. Nothing else is tinted.
2. **Borders at rest, shadows in flight.** Cards, inputs and buttons sit on a 1px border and cast no shadow. Only floating layers (tooltip, menu, dialog) cast shadows. The one exception is the knob shadow on switch and slider thumbs.
3. **Three text grades, three line grades.** If a fourth seems needed, the layout is wrong.
4. **State is never color alone.** Checked has a glyph, on has a thumb position, error has an icon and a message.
5. **Values settle, touches bounce.** A value moving to a new position (switch thumb, slider, selection pill) never overshoots. Press release and hover growth may.
6. **Exits are faster than entrances.**

## 2. Color

### 2.1 Core tokens

Opaque unless the Kind column says overlay. Overlays are straight (non-premultiplied) alpha drawn over whatever is underneath.

| Token | Light | Dark | Kind / use |
|---|---|---|---|
| `BACKGROUND` | 0.985 0.002 248 `#F9FAFB` | 0.155 0.007 271 `#0B0C0F` | opaque; window and sidebar |
| `SURFACE` | 1.000 0.000 0 `#FFFFFF` | 0.190 0.007 258 `#121417` | opaque; cards, inputs |
| `SURFACE_RAISED` | 1.000 0.000 0 `#FFFFFF` | 0.226 0.008 264 `#1A1C20` | opaque; floating layers, secondary button |
| `SURFACE_MUTED` | 0.967 0.003 265 `#F3F4F6` | 0.244 0.008 275 `#1F2024` | opaque; card footers, disabled fills, badges |
| `TRACK` | 0.876 0.006 265 `#D4D6DA` | 0.340 0.010 261 `#35383D` | opaque; unfilled control tracks |
| `INVERSE_SURFACE` | 0.225 0.011 261 `#191C21` | 0.930 0.005 258 `#E6E8EB` | opaque; tooltip fill |
| `INVERSE_FOREGROUND` | 0.970 0.003 265 `#F4F5F7` | 0.204 0.013 264 `#14171D` | opaque; tooltip text |
| `FOREGROUND` | 0.204 0.013 264 `#14171D` | 0.955 0.004 271 `#EFF0F3` | opaque; primary text |
| `TEXT_SECONDARY` | 0.449 0.014 264 `#51555D` | 0.741 0.009 265 `#A8ABB1` | opaque; descriptions, resting nav items |
| `TEXT_MUTED` | 0.531 0.013 264 `#686C74` | 0.639 0.013 264 `#888C94` | opaque; placeholders, captions, resting icons |
| `TEXT_DISABLED` | 0.721 0.010 258 `#A1A5AB` | 0.430 0.009 261 `#4D5055` | opaque; disabled text and icons |
| `BORDER_SUBTLE` | 0.940 0.004 271 `#EAEBEE` | 0.256 0.008 264 `#212327` | opaque; dividers inside surfaces |
| `BORDER` | 0.906 0.005 258 `#DEE0E3` | 0.296 0.008 256 `#2A2D31` | opaque; default control and card border |
| `BORDER_STRONG` | 0.820 0.007 269 `#C2C4C9` | 0.375 0.011 264 `#3E4147` | opaque; hovered border, unchecked box |
| `KNOB` | 1.000 0.000 0 `#FFFFFF` | 1.000 0.000 0 `#FFFFFF` | opaque; switch and slider thumb |
| `HOVER` | `#0F121C` at 5% | `#FFFFFF` at 6% | overlay; hover on neutral and transparent controls |
| `PRESSED` | `#0F121C` at 9% | `#FFFFFF` at 10% | overlay; pressed on neutral and transparent controls |
| `SCRIM` | `#0F121C` at 40% | `#000000` at 60% | overlay; behind dialogs |
| `FOCUS_RING` | active `ACCENT_STRONG` | active `ACCENT_STRONG` | opaque; focus ring, focused input border, caret |
| `FOCUS_HALO` | `FOCUS_RING` at 24% | `FOCUS_RING` at 32% | overlay; input focus halo, slider drag halo |
| `SELECTION` | `FOCUS_RING` at 22% | `FOCUS_RING` at 36% | overlay; text selection |

Notes:

- `SURFACE_RAISED` equals `SURFACE` in light mode on purpose. Light floating layers separate with shadow; dark ones separate with a lighter fill.
- `TRACK` is the fill for unfilled control tracks (switch off, slider rail, progress rail). Borders are never used as fills.
- `INVERSE_*` is used by the tooltip only: dark chip on light UI, light chip on dark UI.
- `KNOB` is white in both modes.
- `FOCUS_RING` is not a fixed color. It equals the active accent's `ACCENT_STRONG`. When the accent is NEUTRAL it equals BLUE's `ACCENT_STRONG` for that mode, so focus is always visibly distinct from text.
- `FOCUS_HALO` and `SELECTION` are `FOCUS_RING` at the listed alpha. The error halo is `DANGER` at the `FOCUS_HALO` alpha.
- `HOVER` and `PRESSED` are composited over the control's own fill, or over the surface behind it when the fill is transparent. Pre-composite them into one opaque color before drawing so the border and anti-aliasing stay crisp.

### 2.2 Accents

Eight accents. The user picks one; it is exposed through the four `ACCENT*` tokens.

- `ACCENT`: solid fill (primary button, checked box, switch on, slider fill).
- `ACCENT_STRONG`: text and icon grade. Use it for any accent-colored text or glyph on a neutral surface or on `ACCENT_SUBTLE`. It is darker than `ACCENT` in light mode and lighter in dark mode.
- `ACCENT_SUBTLE`: opaque tinted background (selected nav item, soft badges).
- `ACCENT_FOREGROUND`: text and icons on `ACCENT`. White for VIOLET, BLUE, GREEN, ROSE. Dark ink for AMBER, ORANGE, CORAL, which stay bright instead of turning brown to carry white text.

**Light**

| Accent | ACCENT | ACCENT_STRONG | ACCENT_SUBTLE | ACCENT_FOREGROUND |
|---|---|---|---|---|
| NEUTRAL | 0.204 0.013 264 `#14171D` | 0.204 0.013 264 `#14171D` | 0.934 0.004 271 `#E8E9EC` | 1.000 0.000 0 `#FFFFFF` |
| VIOLET | 0.575 0.216 290 `#7D55EC` | 0.507 0.183 290 `#6849C4` | 0.957 0.020 289 `#F0EFFE` | 1.000 0.000 0 `#FFFFFF` |
| BLUE | 0.560 0.199 258 `#0E6EE6` | 0.495 0.170 258 `#125DC0` | 0.958 0.020 257 `#E9F2FF` | 1.000 0.000 0 `#FFFFFF` |
| GREEN | 0.539 0.132 155 `#02844B` | 0.479 0.117 155 `#01703F` | 0.958 0.028 155 `#E3F7E9` | 1.000 0.000 0 `#FFFFFF` |
| AMBER | 0.800 0.160 78 `#F5AF20` | 0.495 0.106 72 `#865600` | 0.957 0.040 88 `#FCF0D3` | 0.219 0.046 77 `#261700` |
| ORANGE | 0.715 0.180 52 `#F77C10` | 0.503 0.138 48 `#A04500` | 0.957 0.025 54 `#FFEDE2` | 0.221 0.046 53 `#2B1405` |
| CORAL | 0.695 0.170 28 `#F46C5F` | 0.507 0.150 24 `#AA3838` | 0.957 0.021 28 `#FFECE9` | 0.220 0.044 30 `#2C120E` |
| ROSE | 0.583 0.220 5 `#DB1E69` | 0.512 0.187 5 `#B61F58` | 0.957 0.022 4 `#FFEBEF` | 1.000 0.000 0 `#FFFFFF` |

**Dark**

| Accent | ACCENT | ACCENT_STRONG | ACCENT_SUBTLE | ACCENT_FOREGROUND |
|---|---|---|---|---|
| NEUTRAL | 0.955 0.004 271 `#EFF0F3` | 0.955 0.004 271 `#EFF0F3` | 0.285 0.010 268 `#282A2F` | 0.191 0.009 264 `#121418` |
| VIOLET | 0.583 0.214 290 `#7F58EE` | 0.760 0.125 290 `#B0A3FB` | 0.285 0.054 290 `#2A2543` | 1.000 0.000 0 `#FFFFFF` |
| BLUE | 0.567 0.200 258 `#1270E9` | 0.752 0.125 258 `#7CB0FD` | 0.284 0.055 258 `#182A45` | 1.000 0.000 0 `#FFFFFF` |
| GREEN | 0.545 0.135 155 `#00864B` | 0.746 0.126 155 `#64C489` | 0.285 0.055 155 `#0F321E` | 1.000 0.000 0 `#FFFFFF` |
| AMBER | 0.819 0.160 78 `#FCB52C` | 0.756 0.125 78 `#DBA549` | 0.284 0.045 78 `#36270E` | 0.219 0.046 77 `#261700` |
| ORANGE | 0.734 0.180 52 `#FE821E` | 0.759 0.125 52 `#EF9963` | 0.285 0.046 51 `#3C2314` | 0.221 0.046 53 `#2B1405` |
| CORAL | 0.715 0.169 28 `#FB7365` | 0.763 0.124 28 `#F79285` | 0.285 0.046 29 `#3E211D` | 0.220 0.044 30 `#2C120E` |
| ROSE | 0.589 0.220 5 `#DD226C` | 0.763 0.125 5 `#F48FA7` | 0.285 0.054 6 `#401E26` | 1.000 0.000 0 `#FFFFFF` |

### 2.3 Filled-control states: `ACCENT_HOVER`, `ACCENT_PRESSED`

Solid fills do not use the `HOVER`/`PRESSED` overlays. They shift lightness in OKLCH, keeping C and H, clipping to sRGB by reducing chroma:

- if `L(fill) < 0.30` (NEUTRAL in light mode): hover `L + 0.07`, pressed `L + 0.12`
- otherwise: hover `L - 0.035`, pressed `L - 0.07`

The same rule produces `DANGER_HOVER` and `DANGER_PRESSED`. Compute these once per theme/accent change. Reference results:

| Accent | Light hover | Light pressed | Dark hover | Dark pressed |
|---|---|---|---|---|
| NEUTRAL | `#24272E` | `#30343B` | `#E3E4E7` | `#D8D9DC` |
| VIOLET | `#7349E0` | `#6A3DD4` | `#754DE2` | `#6C41D6` |
| BLUE | `#0063D7` | `#005AC4` | `#0065DB` | `#005CC8` |
| GREEN | `#007843` | `#006D3C` | `#007A44` | `#006F3D` |
| AMBER | `#E9A403` | `#DB9901` | `#F0AA14` | `#E39F01` |
| ORANGE | `#E97200` | `#D96A00` | `#F27700` | `#E26E00` |
| CORAL | `#E86154` | `#DB564A` | `#EE685A` | `#E25C50` |
| ROSE | `#CE0160` | `#BD0057` | `#D00761` | `#C00058` |
| DANGER | `#D0121C` | `#C10013` | `#D3181F` | `#C50014` |

### 2.4 Status

Status reuses accent values so the palette stays small. Each status has the same four roles: base (solid fill, status dot), `_STRONG` (text and icons), `_SUBTLE` (tinted background), `_FOREGROUND` (text on the solid fill).

| Status | Source |
|---|---|
| `SUCCESS`, `SUCCESS_STRONG`, `SUCCESS_SUBTLE`, `SUCCESS_FOREGROUND` | GREEN row of the current mode |
| `WARNING`, `WARNING_STRONG`, `WARNING_SUBTLE`, `WARNING_FOREGROUND` | AMBER row of the current mode |
| `INFO`, `INFO_STRONG`, `INFO_SUBTLE`, `INFO_FOREGROUND` | BLUE row of the current mode |
| `DANGER*` | own values below (hue 27, between CORAL and ROSE but more saturated and darker than CORAL) |

| Mode | DANGER | DANGER_STRONG | DANGER_SUBTLE | DANGER_FOREGROUND |
|---|---|---|---|---|
| Light | 0.580 0.215 27 `#DD2729` | 0.510 0.183 27 `#B82524` | 0.957 0.021 28 `#FFECE9` | 1.000 0.000 0 `#FFFFFF` |
| Dark | 0.588 0.215 27 `#E02B2B` | 0.760 0.124 27 `#F69186` | 0.284 0.055 26 `#411E1B` | 1.000 0.000 0 `#FFFFFF` |

Status colors do not change with the chosen accent.

### 2.5 Chart series

| Token | Light | Contrast on SURFACE | Dark | Contrast on SURFACE |
|---|---|---|---|---|
| `SERIES_0` | 0.601 0.190 258 `#297CEF` | 4.02 | 0.720 0.147 258 `#68A5FF` | 7.39 |
| `SERIES_1` | 0.601 0.154 52 `#C56001` | 4.16 | 0.720 0.144 52 `#EA8846` | 7.12 |
| `SERIES_2` | 0.600 0.136 160 `#009863` | 3.70 | 0.719 0.118 160 `#58BB8B` | 7.82 |
| `SERIES_3` | 0.600 0.189 295 `#8B61E3` | 4.27 | 0.720 0.162 295 `#AD8DFE` | 7.04 |
| `SERIES_4` | 0.600 0.201 5 `#DA3870` | 4.39 | 0.719 0.171 5 `#F97096` | 6.84 |
| `SERIES_5` | 0.601 0.106 215 `#0090A8` | 3.78 | 0.720 0.102 214 `#4AB5CC` | 7.71 |

Order is blue, orange, green, violet, rose, cyan: adjacent series differ in hue by at least 60 degrees. Lightness is constant within a mode (0.60 light, 0.72 dark) so no series looks more important than another.

### 2.6 Contrast (WCAG 2.x ratios, computed from the hex values)

Text on surfaces. AA needs 4.5 for body text.

| Text token | Mode | BACKGROUND | SURFACE | SURFACE_RAISED | SURFACE_MUTED |
|---|---|---|---|---|---|
| `FOREGROUND` | Light | 17.17 | 17.95 | 17.95 | 16.31 |
| `FOREGROUND` | Dark | 17.16 | 16.19 | 14.97 | 14.28 |
| `TEXT_SECONDARY` | Light | 7.16 | 7.48 | 7.48 | 6.80 |
| `TEXT_SECONDARY` | Dark | 8.50 | 8.02 | 7.41 | 7.07 |
| `TEXT_MUTED` | Light | 5.04 | 5.27 | 5.27 | 4.79 |
| `TEXT_MUTED` | Dark | 5.80 | 5.47 | 5.06 | 4.82 |
| `TEXT_DISABLED` | Light | 2.37 | 2.47 | 2.47 | 2.25 |
| `TEXT_DISABLED` | Dark | 2.42 | 2.28 | 2.11 | 2.01 |

- `TEXT_DISABLED` is below AA by design (disabled content is exempt). Never use it for anything a user must read.
- `TEXT_MUTED` passes AA on the four surfaces only. On a `HOVER` or `PRESSED` overlay use `TEXT_SECONDARY` or `FOREGROUND` (worst case `TEXT_SECONDARY` on `PRESSED` over `SURFACE_MUTED`: 5.65 light, 5.24 dark).
- Tooltip: `INVERSE_FOREGROUND` on `INVERSE_SURFACE` is 15.66 light, 14.62 dark.

Accent pairs. "Worst state" is the lower of the hover and pressed fills against the foreground.

| Accent | Mode | FOREGROUND on ACCENT | Worst state | STRONG on SURFACE | STRONG on BACKGROUND | STRONG on SUBTLE | ACCENT vs SURFACE |
|---|---|---|---|---|---|---|---|
| NEUTRAL | Light | 17.95 | 12.50 | 17.95 | 17.17 | 14.78 | 17.95 |
| NEUTRAL | Dark | 16.18 | 13.06 | 16.19 | 17.16 | 12.60 | 16.19 |
| VIOLET | Light | 4.79 | 5.59 | 6.29 | 6.02 | 5.54 | 4.79 |
| VIOLET | Dark | 4.63 | 5.36 | 8.33 | 8.83 | 6.56 | 3.98 |
| BLUE | Light | 4.78 | 5.58 | 6.26 | 5.99 | 5.55 | 4.78 |
| BLUE | Dark | 4.65 | 5.41 | 8.33 | 8.83 | 6.51 | 3.97 |
| GREEN | Light | 4.77 | 5.57 | 6.19 | 5.93 | 5.53 | 4.77 |
| GREEN | Dark | 4.65 | 5.43 | 8.61 | 9.13 | 6.55 | 3.96 |
| AMBER | Light | 9.16 | 7.11 | 6.28 | 6.01 | 5.54 | 1.90 |
| AMBER | Dark | 9.78 | 7.66 | 8.34 | 8.84 | 6.53 | 10.35 |
| ORANGE | Light | 6.51 | 4.99 | 6.28 | 6.01 | 5.52 | 2.68 |
| ORANGE | Dark | 6.99 | 5.36 | 8.26 | 8.76 | 6.52 | 7.40 |
| CORAL | Light | 5.96 | 4.54 | 6.30 | 6.03 | 5.53 | 2.93 |
| CORAL | Dark | 6.45 | 4.90 | 8.26 | 8.76 | 6.53 | 6.80 |
| ROSE | Light | 4.77 | 5.53 | 6.31 | 6.04 | 5.53 | 4.77 |
| ROSE | Dark | 4.65 | 5.41 | 8.20 | 8.69 | 6.52 | 3.97 |
| DANGER | Light | 4.78 | 5.55 | 6.31 | 6.04 | 5.54 | 4.78 |
| DANGER | Dark | 4.62 | 5.36 | 8.18 | 8.67 | 6.53 | 4.00 |

Every text pair above is AA or better. Two known non-text gaps, both accepted:

- In light mode the AMBER, ORANGE and CORAL fills are below 3:1 against `SURFACE` (1.90, 2.68, 2.93). Every control that uses them also shows a glyph or a thumb position, and focus uses `ACCENT_STRONG`, so no state depends on that edge alone. Never use `ACCENT` as a text color.
- Resting borders are decorative (`BORDER` 1.32 light, 1.33 dark on `SURFACE`; `BORDER_STRONG` 1.75 / 1.80). A future high-contrast theme should map `BORDER_STRONG` to `TEXT_MUTED`.

## 3. Shape and space

### 3.1 Radius

| Token | px | Use |
|---|---|---|
| `radius_xs` | 4 | checkbox, selection rects, kbd chips |
| `radius_control_sm` | 6 | sm controls, tooltip |
| `radius_control` | 8 | md controls, inputs, nav items |
| `radius_control_lg` | 10 | lg controls |
| `radius_panel` | 12 | cards, menus, popovers |
| `radius_surface` | 16 | dialogs, sheets |
| `radius_full` | height / 2 | switch, slider, badges, swatches |

Nested corners are concentric: inner radius = outer radius - gap between the two edges, minimum 2. A focus ring at offset 2 around an 8px control has outer radius 12.

### 3.2 Spacing (4px base)

| Token | px | | Token | px |
|---|---|---|---|---|
| `space_0_5` | 2 | | `space_5` | 20 |
| `space_1` | 4 | | `space_6` | 24 |
| `space_1_5` | 6 | | `space_8` | 32 |
| `space_2` | 8 | | `space_10` | 40 |
| `space_3` | 12 | | `space_12` | 48 |
| `space_4` | 16 | | `space_16` | 64 |

Usage: 2 between stacked nav items; 4 between title and description; 6 between a field label and its control, and between icon and text in sm/md controls; 8 between a control and its inline label; 12 between sibling controls in a row; 16 between form rows; 20 for card padding; 48 between page sections. No other values.

### 3.3 Control sizes

| Size | Height | Horizontal padding | Icon | Icon-text gap | Radius | Text |
|---|---|---|---|---|---|---|
| `sm` | 28 | 10 | 14 | 6 | 6 | Inter 500, 12/16 |
| `md` (default) | 32 | 12 | 16 | 6 | 8 | Inter 500, 13/20 |
| `lg` | 40 | 16 | 18 | 8 | 10 | Inter 500, 14/20 |

Padding is measured from the outer edge and includes the border. Icon-only controls are square (width = height). Pointer hit targets extend to at least 28x28 even when the drawn control is smaller (checkbox, swatch). Standalone icon size outside controls is 16; 20 is allowed in empty states only.

### 3.4 Border widths

| Token | px | Use |
|---|---|---|
| `border_hairline` | 1 | every resting border and divider |
| `border_focus` | 2 | focus ring |
| `border_halo` | 3 | input focus halo |

All borders are inside borders: the stated size of a widget is its outer size. Snap rect edges to physical pixels so 1px lines are not blurred at 1x.

## 4. Type

Inter for all UI text. Geist only at 24px and above. Geist Mono for code, keys and IDs. Letter-spacing is in em.

| Role | Font | Weight | Size | Line height | Tracking | Default color | Use |
|---|---|---|---|---|---|---|---|
| `caption` | Inter | 400 | 11 | 16 | +0.005 | `TEXT_MUTED` | timestamps, counts, fine print |
| `label` | Inter | 500 | 12 | 16 | 0 | `FOREGROUND` | field labels, tooltips, sm buttons, section labels |
| `body` | Inter | 400 | 13 | 20 | 0 | `FOREGROUND` | default text, inputs, descriptions |
| `body-strong` | Inter | 500 | 13 | 20 | 0 | `FOREGROUND` | md buttons, nav items, row titles |
| `heading-4` | Inter | 600 | 14 | 20 | -0.006 | `FOREGROUND` | card titles |
| `heading-3` | Inter | 600 | 16 | 24 | -0.011 | `FOREGROUND` | section titles |
| `heading-2` | Inter | 600 | 20 | 28 | -0.017 | `FOREGROUND` | page sub-titles, dialog titles |
| `heading-1` | Geist | 600 | 24 | 32 | -0.020 | `FOREGROUND` | page titles |
| `display` | Geist | 600 | 32 | 40 | -0.025 | `FOREGROUND` | hero numbers, gallery title |
| `mono` | Geist Mono | 400 | 12 | 20 | 0 | `FOREGROUND` | code, keys, IDs, shortcuts |

Rules:

- lg buttons use 14/20 at weight 500 (heading-4 metrics, Medium weight). This is the only off-scale style.
- Weight 600 is for headings only. Emphasis inside body text is 500.
- Numbers that change in place (slider readout, counts) use tabular figures (`tnum`). If the shaper cannot enable features, reserve the width of the widest possible string and right-align.
- Text is vertically centered by line box: baseline = top + (line height - (ascent + descent)) / 2 + ascent, rounded to the nearest physical pixel. Do not center by glyph bounds.
- Secondary text is `TEXT_SECONDARY`, tertiary is `TEXT_MUTED`. Never lower opacity to dim text.
- Single-line text that overflows is truncated with an ellipsis character, never clipped mid-glyph.

## 5. Elevation

Three levels, floating layers only. Shadow layers are listed back to front as `x, y, blur, spread, color`. Shadow ink is `#0F121C` in light mode and `#000000` in dark mode. If the renderer cannot take a negative spread, inset the shadow rect by that amount on each side and reduce its corner radius by the same amount.

| Level | Used by | Light shadows | Light border | Dark shadows | Dark border |
|---|---|---|---|---|---|
| `elevation_1` | tooltip, hover card | `0, 1, 2, 0, ink 6%` + `0, 4, 10, -2, ink 10%` | 1px `BORDER` | `0, 1, 2, 0, black 40%` + `0, 4, 10, -2, black 40%` | 1px `BORDER_STRONG` |
| `elevation_2` | menu, popover, select list | `0, 2, 4, -1, ink 6%` + `0, 10, 24, -4, ink 12%` | 1px `BORDER` | `0, 2, 4, -1, black 40%` + `0, 12, 28, -4, black 50%` | 1px `BORDER_STRONG` |
| `elevation_3` | dialog, sheet, command palette | `0, 4, 8, -2, ink 6%` + `0, 24, 56, -12, ink 22%` | 1px `BORDER` | `0, 4, 8, -2, black 40%` + `0, 28, 64, -12, black 64%` | 1px `BORDER_STRONG` |

- Floating layers fill with `SURFACE_RAISED`. The tooltip fills with `INVERSE_SURFACE` and takes **no** border in either mode.
- `elevation_3` always sits over a full-window `SCRIM`.
- Level 0 (cards, inputs, buttons, sidebar): no shadow, 1px `BORDER` (or `BORDER_SUBTLE` for dividers inside a card).
- `shadow_knob` (switch and slider thumbs only): light `0, 1, 1, 0, ink 10%` + `0, 2, 4, 0, ink 14%`; dark `0, 1, 3, 0, black 45%`. No border.
- Shadow alpha blends with the theme like any other token.

## 6. Focus ring

- Shown only for focus-visible: focus that arrived by keyboard (Tab, arrows, shortcuts). Pointer clicks do not show it. Text inputs are the exception and use the halo below for any focus.
- Geometry: 2px ring, 2px gap between the control's outer edge and the ring. Draw as an unfilled rounded rect expanded by 4px on each side with a 2px inside border and corner radius = control radius + 4. The gap shows whatever is behind the control.
- Color: `FOCUS_RING`, fully opaque. Danger buttons use `DANGER_STRONG`.
- Animates in: alpha 0 to 1 and gap 4 to 2 on the `snappy` spring. Out: alpha to 0 over 100ms linear, no geometry change.
- Text input halo: border color becomes `FOCUS_RING` (still 1px) and a 3px `FOCUS_HALO` band sits directly outside the control with no gap (expanded by 3px, 3px inside border, radius + 3). Halo width animates 0 to 3 on `snappy`; out is a 100ms alpha fade.
- The ring is drawn above siblings and is never clipped by the parent's content clip; scroll containers reserve 4px of inner padding for it.

## 7. Motion

### 7.1 Springs

Mass is always 1. `response = 2*pi / sqrt(stiffness)`, `damping ratio = damping / (2 * sqrt(stiffness))`.

| Name | Stiffness | Damping | Response (s) | Damping ratio | Overshoot | Use |
|---|---|---|---|---|---|---|
| `snappy` | 987.0 | 62.83 | 0.20 | 1.00 | none | press-down inset, switch thumb travel, slider jump to click, checkbox wipe, focus ring in, tooltip in, every exit |
| `smooth` | 322.3 | 35.90 | 0.35 | 1.00 | none | nav selection pill, menu and dialog entrance, sidebar collapse, error message reveal |
| `bouncy` | 438.6 | 29.32 | 0.30 | 0.70 | 4.6% | press release, thumb hover growth, swatch selection |
| `gentle` | 130.5 | 22.85 | 0.55 | 1.00 | none | progress fill, large layout shifts, page-level transitions |

Rules:

- A property that represents a **value** (thumb position, fill width, selected index) uses a critically damped spring: `snappy`, `smooth` or `gentle`. Only `bouncy` overshoots, and only on size or inset feedback.
- Springs are interruptible and keep their velocity when the target changes. Never restart from zero velocity.
- Integration must be frame-rate independent (closed form, or fixed 1/240s substeps). A spring is at rest when it is within 0.01px (or 0.001 for normalized values) of the target and velocity is below 0.01 units/s; then snap to the target and stop requesting frames.
- While the pointer drags a value (slider), the value tracks the pointer 1:1 with no spring.
- Exits use `snappy` for geometry plus a 100ms linear alpha fade, whatever the entrance used.
- Entrances of floating layers: alpha 0 to 1 over 120ms linear, and a 4px translate toward the resting position on the entrance spring. No scale on layers that contain text.

### 7.2 Durations

| What | Value | Curve |
|---|---|---|
| Theme blend (all tokens, shadow alphas included) | 300ms | ease-in-out `cubic-bezier(0.4, 0, 0.2, 1)`, interpolated in OKLab, alpha linear |
| Accent change | 200ms | same as theme blend |
| Hover color in | 60ms | linear, OKLab |
| Hover color out | 160ms | ease-out, OKLab |
| Pressed color in | 0ms | immediate |
| Pressed color out | 160ms | ease-out, OKLab |
| Selected / checked fill color | 120ms | ease-out, OKLab |
| Tooltip delay in | 500ms pointer at rest over the anchor | |
| Tooltip delay out | 100ms | |
| Tooltip warm window | 400ms after a tooltip hides, the next one opens with no delay and no entrance animation | |
| Caret blink | solid for 500ms after any edit or move, then 500ms on / 500ms off, 80ms linear alpha ramps | |

Theme blend details: keep the from and to values of every token and interpolate each frame; widgets always read the blended value. Mid-blend state changes (hover and so on) compose on top. Starting a new blend mid-flight starts from the currently displayed values.

### 7.3 Reduced motion

When the OS asks for reduced motion (or `Style::reduced_motion` is set):

- Every spring that moves or resizes something is replaced by an immediate jump to the target. This includes thumb travel, the nav pill, press inset, hover growth and layer translation.
- Color and alpha transitions stay but are capped at 100ms linear. Theme blend becomes 100ms.
- `bouncy` is never used.
- The caret does not blink; it stays solid.
- Tooltip delays do not change.

## 8. Gallery shell

Minimum window 960 x 600. Window fill `BACKGROUND`.

```
+--------------+-------------------------------------------------+
|              | top bar 56                    [swatches] | [th] |
|  sidebar     +-------------------------------------------------+
|  240         |        <- 48 ->  content, max 720  <- 48 ->     |
|              |   40 top                                        |
|              |   page title                                    |
|              |   48                                            |
|              |   section                                       |
|              |   48                                            |
|              |   section                         96 bottom     |
+--------------+-------------------------------------------------+
```

| Property | Value |
|---|---|
| Sidebar width | 240 (collapsed 56) |
| Sidebar fill / edge | `BACKGROUND`, 1px `BORDER_SUBTLE` on the right edge |
| Sidebar padding | 12 all sides |
| Top bar height | 56, fill `BACKGROUND`, horizontal padding 48 |
| Top bar bottom edge | 1px `BORDER_SUBTLE`, alpha 0 at scroll offset 0, fading to 1 over the first 16px of scroll |
| Content max width | 720, centered in the area right of the sidebar |
| Page padding | 48 left and right (24 when the content area is narrower than 816), 40 top, 96 bottom |
| Page title to first section | 32 |
| Section spacing | 48 |
| Section title to section content | 16 (title plus one-line description, 4 apart) |
| Gap between cards in a section | 16 |
| Scroll | content area only; sidebar and top bar stay fixed |
| Scrollbar | 8px wide overlay, 2px inset, thumb `BORDER_STRONG`, radius full, visible while scrolling or hovered, fades out over 400ms after 1s idle |

The contents of the top bar, sidebar and page are specified in the "Gallery page" section of `docs/widgets-batch1.md`.

## 9. Looks

Everything above describes the **Lumen** look, which is the default. A look (`lumen_core::look::Look`) bundles the parts of the design that are not mode or accent: neutrals for each mode, control heights, radii and padding, per-widget shape parameters, the type scale, preferred fonts, and motion. Widgets read their geometry from the current look, never from constants.

| | Lumen | macOS | Windows | GNOME |
|---|---|---|---|---|
| Control height sm/md/lg | 28/32/40 | 20/24/30 | 24/32/40 | 28/34/42 |
| Control radius md | 8 | 6 | 4 | 6 |
| Card | r12, border | r10, faint border | r8, border | r12, soft shadow |
| Secondary button | raised + border | white, hairline shadow, bevel on filled | bottom edge | flat filled |
| Switch | 36×20 | 38×22, white knob | 40×20, outlined when off | 46×26, white knob |
| Checkbox | 16, r4 | 14, r3.5 | 20, r4 | 16, r4, 2px border |
| Slider thumb | 16, halo | 20 | 20, accent dot | 20, outline |
| Text input | outline + halo | outline + halo | accent underline | filled |
| Focus ring | 2px, gap 4→2 | 3.5px, 60%, hugging | 2px foreground | 2px, 50% |
| Sidebar selection | accent-subtle | neutral | neutral + accent bar | neutral |
| Fonts | Inter, Geist | SF Pro | Segoe UI Variable | Adwaita Sans, Cantarell |
| Native accent | blue (#0E6EE6) | #007AFF / #0A84FF | #005FB8 / #60CDFF | #3584E4 |

Rules:

- **Blending.** Changing look blends every numeric field over `1.5 × motion.theme_blend` with the standard ease, on the same timeline as the palette blend, so corners, sizes and colors morph together. Fonts and the platform tag switch at the start.
- **Accents.** The eight named accents keep their tuned values on Lumen. On platform looks, and for `Accent::Custom` and `Accent::Native`, the four accent tokens are derived: the foreground is white when it reaches 3:1 on the accent, otherwise a dark tint of the hue. `ACCENT_STRONG` is shifted in OKLCH lightness until it reaches 4.5:1 on `SURFACE`. `ACCENT_SUBTLE` is `SURFACE` mixed 12% (light) or 22% (dark) toward the accent. On Windows dark, accents are lifted to L ≥ 0.78, as Fluent does.
- **New tokens.** `CONTROL`, `CONTROL_BORDER` and `CONTROL_EDGE` are the resting fill, outline and bottom edge of neutral controls. On Lumen they equal `SURFACE_RAISED` and `BORDER`.
- **Native.** `Look::native()` picks macOS, Windows or GNOME from the build target (Lumen on the web). The app layer reports the OS accent, the reduce-motion setting and live dark-mode changes.
- **Customizing.** `rounded(f)` scales radii (pills stay pills, checkbox radius is capped at half its size). `density(..)` adds ±4 to control heights. `text_scale(s)` scales text and control labels. `motion(..)` swaps springs and durations, and `Motion::slower(f)` scales them all while keeping each spring's damping ratio. `Ui::override_color(mode, token, color)` replaces a single token.
