# lumen site and brand: Ethereal

Night sky. Light is the only material. The page is a dark (or pale, in light mode) canvas with a live dithered aurora behind it, and everything on top is frosted glass that catches that light. The components are the content, and every preview on the site is drawn by lumen itself.

These are the rules. Break one only on purpose.

## Tokens

| Token | Dark | Light | Job |
|---|---|---|---|
| `--bg` | `#07080C` | `#EEF0F5` | Canvas. Matches lumen's `BACKGROUND`. |
| `--raised` | `#141720` | `#FFFFFF` | Opaque layers: menus, palette, settings panel. |
| `--glass` | `rgb(22 24 32 / 0.62)` | `rgb(255 255 255 / 0.66)` | Cards and bars, always with `backdrop-filter: blur(16–22px) saturate(1.3)`. |
| `--glass-2` | white 6% | ink 6% | Pressed, selected and track fills on glass. |
| `--line` / `--line-2` | white 7% / 12% | ink 7% / 12% | Hairlines. Never a solid gray border. |
| `--text` / `--text-2` / `--text-3` | `#EEF0F6` / `#9AA0B2` / `#80869B` | `#11131C` / `#5B6072` / `#666B7C` | Three text grades, the same as lumen's tokens. |
| `--accent` | user's pick, default blue `#3B82F6` | same | Focus, selection, the lit word, the glow. |

- Radii: 10 for controls, 14 for bars and small cards, 18 to 26 for frames. Pills for chips and tags.
- Spacing on a 4 px grid. Bands are 120 px apart. Content max width is 1240.
- Type: Geist SemiBold for display (-0.05em at 112 px, -0.04em at 64 px), Inter for everything else, Geist Mono for code and kickers.

## Light

- **The sky** is a canvas behind everything: an ordered-dither (4×4 Bayer) aurora of three drifting light sources plus one that follows the pointer, tinted with the accent. It pauses when reduced motion is on and costs one small canvas at a quarter resolution.
- **Glass** is the only surface. A glass panel has a 1px top highlight (`inset 0 1px 0` white 6%) and, on cards in a grid, a radial accent glow that follows the pointer (240 px, 12%).
- **One lit word** per page: a moving shimmer of the accent through the headline. Display headlines only.
- **Frames** around live demos have a 1px accent gradient on their top edge that fades out by 40% of the height.

## Motion

- Every change is a spring or a named ease, never linear except the shimmer and the sky. `--spring` is a sampled spring with about 16% overshoot, used for things that land (thumbs, panels, menus). `--ease` (`0.2, 0, 0, 1`) for fades and colors.
- Durations scale with the motion-speed setting (`--m`), and reduced motion removes movement but keeps fades.
- Sections arrive once: 24 px rise, blur 6 → 0, opacity 0 → 1 over 700 ms.
- Small things move: dropdown rows arrive in sequence, a picked value rolls in, numbers roll the way they changed, chips grow a check, switches stretch while pressed, sparklines draw themselves in and their end dot pops.

## Controls on the site

The site's own controls copy lumen's widgets so the page and the demos feel like one thing. Never a native `<select>`: dropdowns are a button plus a listbox popover with a check, keyboard support and the spring entrance.

## Settings panel

Accent (eight plus a custom color), theme, look (Lumen, macOS, Windows, GNOME), motion speed (1×, ½×, ¼×, ⅒×), reduced motion, and light and dark side by side. Every live demo on the page follows these settings, because the iframe URLs carry them (`accent`, `look`, `dark`, `speed`, `reduced`).

## Imagery

Only what lumen draws: live WebAssembly demos, or screenshots rendered headless by `scripts/screenshots.sh`. No stock, no illustration, no icons as decoration.

## Don'ts

- No solid color blocks behind content, no hard drop shadows on content, no gray borders.
- No second filled button style on a page. The one filled CTA is the inverse of the canvas (white on dark, ink on light).
- No `transition: all`. Animate named properties.
- No vanity stats without a source. Counts come from `content.js` or the build.

## Logo

An ink tile (radius 7/22), a canvas-colored "l" with a foot, and one accent point of light with a soft glow. Files in `site/assets/`.
