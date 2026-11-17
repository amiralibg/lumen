# Widgets, batch 1

Label, heading, button, switch, checkbox, slider, text input, card, sidebar nav, tooltip, plus the gallery page that shows them. All tokens, springs and durations are defined in `/DESIGN.md`; nothing here introduces a new value for color, radius or motion.

## 0. Conventions that apply to every widget

- **Geometry** is in logical px and is the outer size, borders included (borders are inside borders). Snap rect edges to physical pixels.
- **State priority** for fill, border and text: disabled > pressed > hover > rest. Selected/checked and error are independent axes that change the base look. Focus-visible only adds the ring; it never changes fill or text.
- **Focus-visible** means keyboard focus. The ring is the one in DESIGN.md section 6 (2px `FOCUS_RING`, 2px gap, radius + 4, `snappy` in, 100ms fade out). "Ring" below means exactly that.
- **Color transitions** use DESIGN.md section 7.2: hover in 60ms, hover out 160ms, pressed in 0ms, pressed out 160ms, checked/selected 120ms. Not repeated per widget.
- **Neutral state layers.** "+ `HOVER`" means the fill is the rest fill with the `HOVER` overlay composited on top (pre-composite to one opaque color). Same for "+ `PRESSED`". On transparent controls the overlay is drawn alone.
- **Filled state layers.** Accent and danger fills use `ACCENT_HOVER` / `ACCENT_PRESSED` (`DANGER_HOVER` / `DANGER_PRESSED`), DESIGN.md section 2.3.
- **Press inset.** Pressing a control does not scale it. The background rect insets by a fixed amount on all four sides and its radius shrinks by the same amount; content (text, icons) does not move or scale. Down uses `snappy`, release uses `bouncy`. Inset amounts: 1px for sm and md controls and the checkbox box, 1.5px for lg. The hit area and the focus ring keep the un-inset geometry.
- **Disabled** controls ignore pointer and keyboard, show no hover, take no focus, and show the default cursor. Disabled looks use tokens, never opacity.
- **Cursor**: pointer (hand) on buttons, switches, checkboxes, nav items, swatches; text I-beam on inputs; default elsewhere.
- **Icons** are Lucide font glyphs, named by their current Lucide names. Center them by their em box in a square of the stated size.
- **Reduced motion**: DESIGN.md section 7.3 applies everywhere. Every spring below becomes a jump.

---

## 1. Label

Why: one text primitive with fixed roles, so nobody picks a font size by hand.

| Property | Value |
|---|---|
| Default role | `body` (Inter 400, 13/20), `FOREGROUND` |
| Variants | `secondary` -> `TEXT_SECONDARY`; `muted` -> `TEXT_MUTED`; `strong` -> `body-strong`; `caption`, `label`, `mono` -> that type role |
| Field label | `label` role (12/16, 500), `FOREGROUND`, 6px above its control. Required marker: ` *` in `DANGER_STRONG`, separated by a 2px gap |
| Wrapping | wraps at the available width; line height stays 20; paragraph spacing 8 |
| Truncation | single-line mode ends with an ellipsis character |
| Disabled | `TEXT_DISABLED` |
| States | none. Labels are not interactive. Clicking a label that belongs to a checkbox or switch toggles that control (the control shows the hover and press, not the label) |
| Link (inline) | `ACCENT_STRONG`, weight 500, no underline at rest; underline 1px, offset 2px below baseline, on hover; ring radius 4 on focus-visible |

## 2. Heading

Why: a fixed hierarchy; headings carry structure through size and weight, never through color.

| Level | Role | Font | Size / line | Space above | Space below |
|---|---|---|---|---|---|
| display | `display` | Geist 600 | 32/40 | 0 | 8 |
| 1 | `heading-1` | Geist 600 | 24/32 | 0 | 8 |
| 2 | `heading-2` | Inter 600 | 20/28 | 32 | 8 |
| 3 | `heading-3` | Inter 600 | 16/24 | 24 | 4 |
| 4 | `heading-4` | Inter 600 | 14/20 | 16 | 4 |

- Color is always `FOREGROUND`. Tracking per DESIGN.md section 4.
- Space above is skipped when the heading is the first child of its container.
- Optional description directly under a heading: `body`, `TEXT_SECONDARY`, max width 560, at the "space below" distance.
- No states, no animation.

---

## 3. Button

Why: four levels of emphasis and no more. A view has at most one primary button; everything else is secondary or ghost. Danger is for destructive, irreversible actions only.

```
 md, text only                md, leading icon              md, icon only
+----------------------+     +-------------------------+    +--------+
|<12> Save changes <12>|  32 |<10>[16]<6> New key  <12>| 32 |  [16]  | 32
+----------------------+     +-------------------------+    +--------+
  radius 8                     icon side padding -2           32 x 32
```

### Sizes

| Size | Height | Padding left/right | With leading icon: left padding | Icon | Gap | Radius | Text | Icon-only |
|---|---|---|---|---|---|---|---|---|
| sm | 28 | 10 | 8 | 14 | 6 | 6 | Inter 500 12/16 | 28 x 28 |
| md | 32 | 12 | 10 | 16 | 6 | 8 | Inter 500 13/20 | 32 x 32 |
| lg | 40 | 16 | 14 | 18 | 8 | 10 | Inter 500 14/20 | 40 x 40 |

- Width hugs content. Minimum width for text buttons is 2 x height. `full_width` stretches to the container and centers the content.
- Content is centered horizontally and vertically. Label is single-line and never wraps.
- A trailing icon mirrors the leading rule (right padding reduced by 2).
- Icon-only buttons must be given a tooltip label.
- Buttons in a row are 8 apart. In a footer the primary action is rightmost.

### Variants and states

| Variant | State | Fill | Border (1px) | Text | Icon |
|---|---|---|---|---|---|
| **Primary** | rest | `ACCENT` | none | `ACCENT_FOREGROUND` | `ACCENT_FOREGROUND` |
| | hover | `ACCENT_HOVER` | none | same | same |
| | pressed | `ACCENT_PRESSED` + press inset | none | same | same |
| **Secondary** | rest | `SURFACE_RAISED` | `BORDER` | `FOREGROUND` | `TEXT_SECONDARY` |
| | hover | `SURFACE_RAISED` + `HOVER` | `BORDER_STRONG` | `FOREGROUND` | `FOREGROUND` |
| | pressed | `SURFACE_RAISED` + `PRESSED` + press inset | `BORDER_STRONG` | `FOREGROUND` | `FOREGROUND` |
| **Ghost** | rest | transparent | none | `TEXT_SECONDARY` | `TEXT_SECONDARY` |
| | hover | `HOVER` | none | `FOREGROUND` | `FOREGROUND` |
| | pressed | `PRESSED` + press inset | none | `FOREGROUND` | `FOREGROUND` |
| **Danger** | rest | `DANGER` | none | `DANGER_FOREGROUND` | `DANGER_FOREGROUND` |
| | hover | `DANGER_HOVER` | none | same | same |
| | pressed | `DANGER_PRESSED` + press inset | none | same | same |
| Primary, Secondary, Danger | disabled | `SURFACE_MUTED` | `BORDER_SUBTLE` | `TEXT_DISABLED` | `TEXT_DISABLED` |
| Ghost | disabled | transparent | none | `TEXT_DISABLED` | `TEXT_DISABLED` |
| any | focus-visible | unchanged | unchanged, plus ring | unchanged | unchanged |

- Ring color is `FOCUS_RING`; on Danger it is `DANGER_STRONG`. Ring radius = button radius + 4.
- Buttons are flat: no gradient, no shadow, no inner highlight.
- Press: inset per section 0 (`snappy` down, `bouncy` up), fill switches to the pressed color immediately and fades back over 160ms. The action fires on release inside the button; dragging out cancels and releases the inset.
- Keyboard: Space and Enter activate. Show the pressed look while the key is held.
- Text and icon colors on Secondary and Ghost follow the hover timing (60ms in, 160ms out).

Glyphs used in the gallery: `plus`, `download`, `copy`, `trash-2`, `arrow-right`, `settings`.

---

## 4. Switch

Why: an on/off setting that applies immediately. If the change needs a Save button, use a checkbox.

```
  off                          on                         pressed (off)
+------------------+        +------------------+        +------------------+
| (####)           | 20     |           (####) |        | (######)         |
+------------------+        +------------------+        +------------------+
 <2> 16 thumb                36 x 20, radius 10          thumb 20 x 16
```

| Part | Geometry |
|---|---|
| Track | 36 x 20, radius 10 |
| Thumb | 16 x 16 circle, 2px from the track edge on three sides. Thumb x travels 16px between off and on |
| Hit area | the track, extended to 36 x 28, plus the associated label |
| Label | `body`, `FOREGROUND`. In a settings row the label is on the left and the switch is right-aligned; inline use puts the label 8px to the right of the track |

| State | Track fill | Thumb fill | Thumb shadow |
|---|---|---|---|
| off, rest | `TRACK` | `KNOB` | `shadow_knob` |
| off, hover | `BORDER_STRONG` | `KNOB` | `shadow_knob` |
| on, rest | `ACCENT` | `ACCENT_FOREGROUND` | `shadow_knob` |
| on, hover | `ACCENT_HOVER` | `ACCENT_FOREGROUND` | `shadow_knob` |
| pressed (either) | the hover fill of that state | same as that state | `shadow_knob` |
| off, disabled | `SURFACE_MUTED` with 1px `BORDER` | `BORDER_STRONG` | none |
| on, disabled | `TRACK` | `SURFACE` | none |
| focus-visible | unchanged, ring around the track (radius 14) | | |

No border on the track except in off-disabled.

Animation:

- One spring value `t` in 0..1 on `snappy` drives everything: thumb x = lerp(off x, on x, t), track fill = OKLab mix(off fill, on fill, t), thumb fill = OKLab mix(`KNOB`, `ACCENT_FOREGROUND`, t). No overshoot. The value flips on release.
- Press feedback: while the pointer or Space is held, the thumb widens from 16 to 20 (height stays 16, radius 8), growing toward the track center, on `snappy`. On release it returns to 16 on `snappy` as it travels. There is no press inset on the track and no bounce anywhere on a switch.
- Dragging is not supported; a click anywhere on the track or label toggles.
- Keyboard: Space toggles. Enter does nothing.

---

## 5. Checkbox

Why: a choice that is part of a form or a multi-select list. The indeterminate state exists only for a parent that summarizes mixed children.

```
 unchecked      checked        indeterminate     with label
 +------+       +------+       +------+          +------+
 |      | 16    | [ck] |       | [--] |          | [ck] |<8> Email me a summary
 +------+       +------+       +------+          +------+
 radius 4       check 14px     bar 8 x 2         box centered on the first text line
```

| Part | Geometry |
|---|---|
| Box | 16 x 16, radius 4 |
| Check | Lucide `check` at 14px, centered in the box |
| Indeterminate bar | a drawn rounded rect, 8 x 2, radius 1, centered. Not a glyph |
| Label | `body` 13/20 `FOREGROUND`, 8px right of the box. The box is vertically centered on the first line (2px below the top of a 20px line). Optional description under the label: `body`, `TEXT_SECONDARY`, 2px gap |
| Hit area | box plus label plus description, at least 28px tall |
| Rows in a group | 12 apart; children of an indeterminate parent are indented 24 |

| State | Box fill | Box border (1px) | Mark | Label |
|---|---|---|---|---|
| unchecked, rest | `SURFACE` | `BORDER_STRONG` | none | `FOREGROUND` |
| unchecked, hover | `SURFACE` | `TEXT_MUTED` | none | `FOREGROUND` |
| unchecked, pressed | `SURFACE` + `PRESSED`, press inset 1 | `TEXT_MUTED` | none | `FOREGROUND` |
| checked or indeterminate, rest | `ACCENT` | none | `ACCENT_FOREGROUND` | `FOREGROUND` |
| checked or indeterminate, hover | `ACCENT_HOVER` | none | `ACCENT_FOREGROUND` | `FOREGROUND` |
| checked or indeterminate, pressed | `ACCENT_PRESSED`, press inset 1 | none | `ACCENT_FOREGROUND` | `FOREGROUND` |
| unchecked, disabled | `SURFACE_MUTED` | `BORDER` | none | `TEXT_DISABLED` |
| checked or indeterminate, disabled | `TRACK` | none | `SURFACE` | `TEXT_DISABLED` |
| focus-visible | unchanged, ring around the box only (radius 8) | | | |

Animation:

- Fill and border color: 120ms ease-out.
- Check appears by a left-to-right wipe: a clip rect over the glyph grows from width 0 to 14 on `snappy`. It disappears with an 80ms linear alpha fade, no wipe.
- Indeterminate bar: width grows 0 to 8 from the center on `snappy`; disappears with the same 80ms fade.
- Checked to indeterminate (or back): the outgoing mark fades 80ms while the incoming one animates in. The fill stays `ACCENT` throughout.
- Press: box inset 1px (`snappy`), release `bouncy`. The mark does not move.
- Clicking an indeterminate box makes it checked. Keyboard: Space toggles.

---

## 6. Slider

Why: choosing a value from a continuous or stepped range where the feel of dragging matters more than typing an exact number.

```
                         thumb 16 (18 on hover/drag)
                              v
 |=======================(  )--------------------------|  <12>  64%
 ^ fill: ACCENT, 4 tall        ^ rail: TRACK, 4 tall            ^ readout, 44 wide
 |<------------------- hit area 20 tall ---------------|
```

| Part | Geometry |
|---|---|
| Hit area | full width x 20 (the whole 20px band starts a drag) |
| Rail | full width x 4, radius 2, vertically centered |
| Fill | from the rail's left end to the thumb center, 4 tall, radius 2 on the left end, square on the right (hidden under the thumb) |
| Thumb | 16 x 16 circle. Its center travels from rail left + 8 to rail right - 8, so the thumb never leaves the rail's bounds |
| Readout (optional) | 12px right of the rail, fixed 44 wide, right-aligned, `label` role (12/16, 500) with tabular figures. The caller formats the string (`64%`, `13 px`) |
| Default width | 240; minimum 120 |
| Label | field label 6px above, as for inputs |

| State | Rail | Fill | Thumb | Readout |
|---|---|---|---|---|
| rest | `TRACK` | `ACCENT` | 16, `KNOB`, `shadow_knob` | `TEXT_SECONDARY` |
| hover (anywhere in hit area) | `TRACK` | `ACCENT` | 18, `KNOB`, `shadow_knob` | `TEXT_SECONDARY` |
| dragging | `TRACK` | `ACCENT` | 18, `KNOB`, `shadow_knob`, plus a halo: 26 x 26 circle of `FOCUS_HALO` centered behind the thumb | `FOREGROUND` |
| focus-visible | unchanged | unchanged | ring around the thumb (2px `FOCUS_RING`, 2px gap, so 24 across for a 16 thumb) | unchanged |
| disabled | `BORDER` | `TEXT_DISABLED` | 16, `SURFACE_MUTED` with 1px `BORDER_STRONG`, no shadow | `TEXT_DISABLED` |

Animation and input:

- While dragging, the value follows the pointer 1:1. No spring, no lag.
- Clicking the rail away from the thumb moves the value there on `snappy` (thumb x and fill width share the one spring value; no overshoot), and the drag continues from that point if the button stays down.
- Thumb size 16 to 18 on hover: `bouncy`. Back to 16: `snappy`.
- Halo: alpha 0 to 1 over 60ms on drag start, 1 to 0 over 160ms on release. Its size does not animate.
- Press feedback is the halo plus the readout turning `FOREGROUND`. There is no inset and no fill darkening.
- Stepped sliders snap the value; the thumb moves between steps on `snappy`. No tick marks in batch 1.
- Keyboard: Left/Down and Right/Up move one step (1% of the range if continuous); with Shift, ten steps; Home and End go to the ends. Keyboard changes animate on `snappy`.
- The readout text changes immediately, never animated or counted up.

---

## 7. Text input

Why: single-line text entry that feels native. Focus is the loudest state in the field because that is where the user's eyes are.

```
 Email                                    <- label 12/16 500, 6 below
+------------------------------------------+
|<10>[16]<8> mara@northwind.dev|      <12> | 32, radius 8, 1px border
+------------------------------------------+
 (!) Enter a valid email address.         <- 6 below; icon 14, 6 gap, text 12/16
```

| Part | Geometry |
|---|---|
| Field | height 32 (sm 28, lg 40), radius 8 (6, 10), 1px border, default width 280, minimum 120 |
| Text inset | 12 from the left and right outer edges |
| Leading icon (optional) | 16px, its left edge 10 from the field edge; text then starts 8 after the icon (34 from the edge). `TEXT_MUTED` |
| Text | `body` 13/20 (lg: 14/20), `FOREGROUND`, vertically centered |
| Placeholder | same metrics, `TEXT_MUTED`. Stays visible on focus until the first character is typed |
| Caret | 2px wide, 16px tall (line height - 4), corner radius 1, color `FOCUS_RING`, vertically centered on the line |
| Selection | `SELECTION` rect, full line height (20), radius 3; selected text stays `FOREGROUND`. Not drawn when the field is unfocused |
| Label | `label` role, 6 above the field |
| Message slot | 6 below the field; `label` metrics at weight 400 (12/16). Helper text is `TEXT_MUTED`. Error text is `DANGER_STRONG` with a leading `circle-alert` glyph at 14px in `DANGER_STRONG`, 6px gap. An error replaces the helper |
| Overflow | text scrolls horizontally inside the 12px insets, clipped to the content rect; the caret stays at least 12px inside the visible edge while typing |

| State | Fill | Border | Halo | Text | Icon |
|---|---|---|---|---|---|
| rest | `SURFACE` | 1px `BORDER` | none | `FOREGROUND` / placeholder `TEXT_MUTED` | `TEXT_MUTED` |
| hover | `SURFACE` | 1px `BORDER_STRONG` | none | same | `TEXT_MUTED` |
| focused (pointer or keyboard) | `SURFACE` | 1px `FOCUS_RING` | 3px `FOCUS_HALO`, directly outside, radius 11 | same | `TEXT_SECONDARY` |
| error | `SURFACE` | 1px `DANGER` | none | same | `TEXT_MUTED` |
| error + focused | `SURFACE` | 1px `DANGER` | 3px `DANGER` at the `FOCUS_HALO` alpha | same | `TEXT_SECONDARY` |
| disabled | `SURFACE_MUTED` | 1px `BORDER_SUBTLE` | none | `TEXT_DISABLED` (value and placeholder) | `TEXT_DISABLED` |

- There is no pressed state and no press inset. Pointer down places the caret immediately.
- Inputs do not use the offset focus ring. The halo is their focus indicator for every kind of focus.
- Halo in: width 0 to 3 on `snappy`. Halo out: 100ms alpha fade. Border color: 60ms in, 160ms out.
- Caret: appears immediately on focus. Solid for 500ms after any keystroke, caret move or click; then 500ms on, 500ms off, with 80ms linear alpha ramps. When the caret moves by typing or arrow keys it jumps; it does not slide.
- Error message reveal: slot height animates 0 to 22 (6 gap + 16 line) on `smooth`, text alpha 0 to 1 over 120ms. Hiding: alpha to 0 over 100ms, height to 0 on `snappy`. Layout below moves with it.
- The error state is set by the caller. Show it after blur or submit, not on every keystroke; clear it as soon as the value becomes valid.
- Editing behavior expected: click to place caret, drag to select, double-click word, triple-click all, Shift+arrows, word jump with Alt (macOS) or Ctrl, Cmd/Ctrl+A/C/X/V/Z.

Glyphs used in the gallery: `mail`, `search`, `at-sign`, `circle-alert`.

---

## 8. Card

Why: groups one topic's controls on a quiet surface. Cards rest on a border; they never cast a shadow and never lift on hover.

```
+--------------------------------------------------------+  radius 12, 1px BORDER, fill SURFACE
| <20>                                                   |
|  Profile                                 [ action ]    |  header: title heading-4
|  How you appear to people in this workspace.           |          description body, TEXT_SECONDARY, 4 below title
|                                                        |  20 between header and body
|  body ...                                              |
| <20>                                                   |
+--------------------------------------------------------+  1px BORDER_SUBTLE
| <20> Changes apply to all workspaces.  [Cancel] [Save] |  footer: fill SURFACE_MUTED, padding 12 / 20
+--------------------------------------------------------+
```

| Part | Spec |
|---|---|
| Container | fill `SURFACE`, 1px `BORDER`, radius 12, no shadow. Width fills the content column |
| Padding | 20 on all sides of the header + body region |
| Header | title: `heading-4` (Inter 600 14/20), `FOREGROUND`. Description: `body`, `TEXT_SECONDARY`, 4 below the title, max width 520. Optional header action (one sm or md button) right-aligned, top-aligned with the title |
| Header to body | 20 |
| Body | free layout. Form rows are 16 apart |
| Settings row (body pattern) | title `body-strong` + description `body` `TEXT_SECONDARY` (2 apart) on the left, control right-aligned and vertically centered. Rows have 12 vertical padding and a 1px `BORDER_SUBTLE` divider between them, inset 0 (divider spans the body width). First row has no top padding, last row no bottom padding |
| Footer (optional) | separated by a 1px `BORDER_SUBTLE` line across the full card width. Fill `SURFACE_MUTED`. Padding 12 vertical, 20 horizontal. Left: hint text, `body`, `TEXT_SECONDARY`. Right: buttons, md, 8 apart, primary rightmost. Bottom corners radius 11 so the fill sits inside the card border |
| Danger variant | identical card; only the action inside is a Danger button. Do not tint the card or its border |
| States | none. A card is not interactive and has no hover, pressed or focus look. Disabled content is disabled per control |
| Animation | none |

---

## 9. Sidebar nav

Why: persistent wayfinding. It stays quiet; the only color in it is the selected item.

```
+--------------------------+
| <12>                     |
|  [N] Northwind Labs      |  header 40: 24 avatar, 8 gap, body-strong
|  <16>                    |
|  Workspace               |  section label 28 tall, label role, TEXT_MUTED
|  [ic] General            |  item 32 tall, radius 8
| ([ic] Members       12 ) |  selected: ACCENT_SUBTLE pill
|  [ic] Billing            |  items 2 apart
|  <16>                    |
|  Account                 |
|  [ic] Notifications   3  |
|  ...                     |
|  [<<] Collapse           |  footer item, pinned to the bottom
+--------------------------+
  240 wide, 12 padding, 1px BORDER_SUBTLE right edge
```

| Part | Geometry and tokens |
|---|---|
| Container | 240 wide, full height, fill `BACKGROUND`, 1px `BORDER_SUBTLE` on the right edge, padding 12 |
| Header | 40 tall. Avatar 24 x 24, radius 6, fill `ACCENT`, initial in `ACCENT_FOREGROUND` (`label` role, centered). Name: `body-strong`, `FOREGROUND`, 8 after the avatar, truncated. 8 horizontal padding to align with item content |
| Section | first section starts 16 below the header; later sections 16 below the previous one |
| Section label | 28 tall, text vertically centered, 8 left padding, `label` role (12/16, 500), `TEXT_MUTED`, sentence case, not uppercase |
| Item | 32 tall, full inner width (216), radius 8, items 2 apart |
| Item content | icon 16 at 8 from the left; label `body-strong` 8 after the icon, truncated; optional badge right-aligned, 8 from the right edge |
| Badge / count | 18 tall, min width 18, horizontal padding 5, radius 9, fill `SURFACE_MUTED`, text `caption` metrics at weight 500 (11/16), tabular figures, `TEXT_SECONDARY`. Show `99+` above 99 |
| Footer | pinned to the bottom, 12 above the edge; holds the collapse item (`panel-left-close` when expanded, `panel-left-open` when collapsed) |

| Item state | Fill | Icon | Label | Badge |
|---|---|---|---|---|
| rest | transparent | `TEXT_MUTED` | `TEXT_SECONDARY` | fill `SURFACE_MUTED`, text `TEXT_SECONDARY` |
| hover | `HOVER` | `FOREGROUND` | `FOREGROUND` | unchanged |
| pressed | `PRESSED`, press inset 1 | `FOREGROUND` | `FOREGROUND` | unchanged |
| selected | `ACCENT_SUBTLE` | `ACCENT_STRONG` | `ACCENT_STRONG` | fill transparent, text `ACCENT_STRONG` |
| selected + hover | `ACCENT_SUBTLE` (no overlay) | `ACCENT_STRONG` | `ACCENT_STRONG` | same |
| disabled | transparent | `TEXT_DISABLED` | `TEXT_DISABLED` | text `TEXT_DISABLED` |
| focus-visible | unchanged, ring radius 12; the ring may overlap the 12px sidebar padding | | | |

With the NEUTRAL accent the selected item is a gray pill with `FOREGROUND` text. That is intended.

Animation:

- The selected pill is one shared rect, not per-item fills. When selection changes it slides to the new item's y on `smooth` (no overshoot) and keeps its size. Icon and label colors of the old and new item cross-fade over 120ms.
- If the new item is in a different scroll position or the sidebar was just created, the pill appears in place with no slide.
- Selection changes on release. Keyboard: Up/Down move focus between items, Enter or Space selects.

Collapsed mode (optional):

- Width 56. Items become 32 x 32, centered, icon only. Labels, badges and the workspace name are hidden; the avatar stays, centered. A count becomes a 6 x 6 `ACCENT` dot at the icon's top-right corner, overlapping it by 2px.
- Section labels become a 1px `BORDER_SUBTLE` divider, 16 wide inset from each side, with 8 above and below.
- Each item shows a tooltip with its label, placed to the right.
- Transition: width 240 to 56 on `smooth`; label and badge alpha to 0 over 100ms at the start of the collapse and 0 to 1 over 120ms after the expand spring passes 80%. Content is clipped to the sidebar during the transition; the page content resizes with it.

Glyphs used in the gallery: `settings-2`, `users`, `credit-card`, `key-round`, `user`, `bell`, `palette`, `shield-check`, `panel-left-close`, `panel-left-open`.

---

## 10. Tooltip

Why: names an unlabeled control or shows its shortcut. It is a hint, never the only place information lives, and never interactive.

```
            +-----------------------+
            | <8> Copy key   [C] <8>|   24 tall single line, radius 6
            +-----------------------+   fill INVERSE_SURFACE, no border, elevation_1
                      <6 gap>
                   [ anchor ]
```

| Property | Value |
|---|---|
| Fill / text | `INVERSE_SURFACE` / `INVERSE_FOREGROUND`. No border. Shadow `elevation_1` |
| Text | `label` role (Inter 500, 12/16) |
| Padding | 4 vertical, 8 horizontal (single line: 24 tall). Multi-line: 6 vertical, 8 horizontal |
| Radius | 6 |
| Max width | 240 including padding; text wraps, left-aligned. No truncation |
| Arrow | none |
| Shortcut chip (optional) | after the text with an 8 gap: `mono` at 11/16, `INVERSE_FOREGROUND`, on a rect of `INVERSE_FOREGROUND` at 16% alpha, 16 tall, horizontal padding 4, radius 4 |
| Placement | default top, centered on the anchor, 6 from it. If it does not fit, flip to bottom, then right, then left. After choosing a side, shift along the anchor edge to stay 8 inside the window |
| Sidebar (collapsed) items | placement right, vertically centered, 8 from the item |
| Delay in | 500ms with the pointer at rest over the anchor. Keyboard focus-visible on the anchor also opens it, after the same 500ms |
| Delay out | 100ms after the pointer leaves or focus moves. Closes immediately on press, scroll, Escape or any key press |
| Warm window | for 400ms after a tooltip closes, the next tooltip opens with no delay and no entrance animation |
| Entrance | alpha 0 to 1 over 120ms linear, plus a 4px translate from the anchor side toward its resting position on `snappy`. No scale |
| Exit | alpha 1 to 0 over 100ms linear, no movement |
| Pointer | the tooltip ignores the pointer; hovering it does not keep it open |
| Disabled anchors | still show their tooltip |
| States | none of its own |

Only one tooltip exists at a time. It is drawn in the top layer, above everything, and is not clipped by any parent.

---

## 11. Gallery page

The gallery is a believable settings screen for a fictional product, **Relay** (a deploy and release tool), in the workspace **Northwind Labs**. Every batch-1 widget appears doing a real job. A final section shows the raw state matrix. Shell dimensions come from DESIGN.md section 8.

### 11.1 Top bar (56 tall, right of the sidebar)

```
| Settings / General                          (o)(o)(o)(o)(o)(o)(o)(o)  |  [moon] |
  ^ breadcrumb                                 ^ accent picker           ^ theme toggle
```

- Left: breadcrumb. `Settings` in `body`, `TEXT_MUTED`; ` / ` in `TEXT_DISABLED` with 6px on each side; current section name in `body-strong`, `FOREGROUND`. The section name follows the scroll position.
- Right, in order: accent picker, 12 gap, a 1px x 16px `BORDER` divider, 12 gap, theme toggle. Everything is vertically centered.

**Accent picker.** Eight swatches in the order NEUTRAL, VIOLET, BLUE, GREEN, AMBER, ORANGE, CORAL, ROSE.

| Property | Value |
|---|---|
| Swatch | 20 x 20 circle filled with that accent's `ACCENT` for the current mode. No border |
| Spacing | 8 between swatches (row is 216 wide). Hit area 28 x 28 each |
| Hover | swatch grows to 22 on `bouncy`; back to 20 on `snappy` |
| Pressed | swatch shrinks to 18 on `snappy` |
| Selected | Lucide `check` at 12px centered, in that accent's `ACCENT_FOREGROUND`, plus a ring: 2px wide, 2px gap, in that swatch's own `ACCENT` (28 across). The ring animates in like a focus ring (gap 4 to 2, alpha 0 to 1, `snappy`); the check uses the checkbox wipe |
| Focus-visible | the standard ring in `FOCUS_RING`; on the selected swatch it replaces the selection ring while focused |
| Tooltip | the accent's name in title case: `Neutral`, `Violet`, ... |
| On select | all accent tokens blend to the new accent over 200ms (OKLab). Default accent: VIOLET |
| Keyboard | Left/Right move between swatches, Space or Enter selects |

**Theme toggle.** A ghost, icon-only, md button (32 x 32).

- Shows the theme you would switch to: `moon` in light mode, `sun` in dark mode. Tooltip: `Switch to dark theme` / `Switch to light theme`, with shortcut chip `T`.
- On click the icons cross-fade over 120ms while the old one moves 4px down and the new one arrives from 4px above on `snappy`. No rotation.
- The whole UI blends over 300ms per DESIGN.md section 7.2. The gallery starts in the OS theme.

### 11.2 Sidebar content

Header: avatar `N`, name `Northwind Labs`.

| Section | Items (glyph, label, badge) |
|---|---|
| Workspace | `settings-2` General (selected at launch) · `users` Members, badge `12` · `credit-card` Billing · `key-round` API keys |
| Account | `user` Profile · `bell` Notifications, badge `3` · `palette` Appearance · `shield-check` Security (disabled) |
| Footer | `panel-left-close` Collapse |

Clicking an item scrolls the page to that section on `gentle` and moves the pill. Scrolling the page updates the selected item. Items without a section on this page (Members, Billing) select but do not scroll.

### 11.3 Page content (720 column)

Page header: `display` heading **Settings**, then 8 below, `body` `TEXT_SECONDARY`: "Manage your workspace, your account, and how Relay looks on this device." 32 to the first section.

Each section is a `heading-3` title, a one-line `body` `TEXT_SECONDARY` description 4 below, then 16 to its card(s). Sections are 48 apart.

**Section 1: General** - "Basics for the Northwind Labs workspace."

Card, title "Workspace", description "This name and URL appear in invites and deploy notifications."

- Two inputs side by side, 16 apart, equal width:
  - Label `Workspace name`, value `Northwind Labs`.
  - Label `Workspace URL`, leading icon `at-sign`, value `northwind labs`, in the **error** state with message "URLs can use lowercase letters, numbers and hyphens only."
- 16 below, full-width input: label `Billing email`, leading icon `mail`, placeholder `billing@northwind.dev`, helper text "Invoices and payment receipts are sent here."
- 16 below, disabled input: label `Workspace ID`, value `ws_8f3k2m9q` (shown in `mono`), helper "Used in API requests. This cannot be changed."
- Footer: hint "Last saved 2 minutes ago." Buttons: ghost `Discard`, primary `Save changes`.

**Section 2: Profile** - "How you appear to teammates."

Card, title "Your details", header action: secondary sm button `Upload photo` with leading `upload`.

- Inputs in two columns: `Full name` = `Mara Okafor`; `Role` with placeholder `e.g. Platform engineer`.
- Full width: `Search teammates` with leading `search`, placeholder `Find someone by name or email`. This is the field to click to see focus, caret and selection.

**Section 3: Notifications** - "Choose what reaches you and where."

Card, title "Activity". Settings rows, each with a switch on the right:

| Title | Description | State |
|---|---|---|
| Deploy failures | When a production deploy fails or is rolled back. | on |
| Mentions and replies | When someone mentions you or replies to your comment. | on |
| Weekly digest | A Monday summary of releases, incidents and review time. | off |
| Audit log exports | Available on the Enterprise plan. | off, disabled |

Second card, 16 below, title "Channels", description "Where notifications are delivered."

- Parent checkbox `All channels`, **indeterminate**.
- Children, indented 24, 12 apart: `Email` checked, description "mara@northwind.dev" · `Desktop` checked · `Mobile push` unchecked, description "Install the Relay app to enable." · `SMS` unchecked, disabled, description "Add a phone number in Security first."
- Toggling children updates the parent (all checked, none, or indeterminate); toggling the parent sets all enabled children.

**Section 4: Appearance** - "Applies to this device only."

Card, title "Interface". Settings rows with the control on the right:

| Title | Description | Control |
|---|---|---|
| Text size | Scales interface text. The default is 13 px. | slider 11 to 16, step 1, value 13, readout `13 px`, width 240 |
| Sidebar density | How tightly navigation items are packed. | slider 0 to 100, continuous, value 64, readout `64%`, width 240 |
| Reduce motion | Replace movement with fades. Follows your system setting by default. | switch, off. This one is live: it sets `reduced_motion` for the whole gallery |
| Animation speed | Locked while Reduce motion is on. | slider, value 100, readout `100%`, **disabled** whenever the switch above is on |

The two sliders in the first rows are display-only; they do not restyle the gallery.

**Section 5: API keys** - "Keys let scripts and CI act on behalf of this workspace."

Card, title "Active keys", header action: secondary md button `New key` with leading `plus`.

Rows (settings-row pattern, dividers between):

| Left | Right |
|---|---|
| `Production deploys` (`body-strong`) over `rk_live_9d2f...c41a` (`mono`, `TEXT_SECONDARY`), then `caption`: "Last used 4 minutes ago" | ghost icon-only `copy` (tooltip `Copy key`, shortcut chip `C`), ghost icon-only `trash-2` (tooltip `Revoke key`) |
| `Staging CI` over `rk_test_41ab...77e0`, caption "Last used yesterday" | same two buttons |
| `Local development` over `rk_test_c09e...1b3d`, caption "Never used" | same two buttons |

Footer: hint "Keys are shown once at creation. Store them in your secret manager." Button: secondary `View audit log` with trailing `arrow-right`.

**Section 6: Danger zone** - "These actions cannot be undone."

Card, title "Delete workspace", description "Permanently removes Northwind Labs, its 12 members, and all deploy history. Billing stops at the end of the current period." Body: one input, label `Type the workspace name to confirm`, placeholder `Northwind Labs`, width 320. Footer: hint "You will be asked to confirm once more." Buttons: secondary `Export data` with leading `download`, danger `Delete workspace`. The danger button is disabled until the input equals `Northwind Labs`.

**Section 7: States** - "Every button variant at every size, for visual review."

Card, no title, padding 20. A grid: rows are Primary, Secondary, Ghost, Danger; columns are `sm`, `md`, `lg`, `md + icon`, `icon only`, `disabled`. Row labels in the first column, `label` role, `TEXT_MUTED`, 96 wide. Column headers in `caption`, `TEXT_MUTED`. Cells are left-aligned, 16 column gap, 12 row gap, each row vertically centered. Button copy by row: `Deploy`, `Preview`, `Cancel`, `Delete`. The icon column uses `rocket`, `eye`, `x`, `trash-2`. Below the grid, after a 1px `BORDER_SUBTLE` divider with 16 above and below, one line of `body` `TEXT_SECONDARY`: "Tab through this page to see focus rings. Hover any icon button for its tooltip."

### 11.4 What a reviewer should see

- At rest the page is nearly monochrome. Accent appears on: the avatar, the selected nav item, one primary button per card footer, checked boxes, switches that are on, slider fills, the focused input. Changing the accent swatch recolors exactly those and nothing else.
- No shadow anywhere until a tooltip opens.
- Light and dark have the same hierarchy: page darker than cards in dark mode, page slightly grayer than cards in light mode.
- Every interactive element can be reached and operated with the keyboard, with a visible ring.
