# lumen site and brand

The site follows a Notion-style system: warm paper, ink, one blue, hairlines.
These are the rules; break one only on purpose.

## Tokens

- Canvas `#F6F5F4` (dark: `#191918`). Cards `#FFFFFF` (dark: `#222221`). Never a white page.
- Text is ink at four alphas: 95 / body warm gray `#615D59` / 54 / 40.
- Hairlines `rgb(0 0 0 / 0.08)`; stronger `0.14`. No shadows on content.
- Shadows exist in two places only: the sticky nav (barely) and product renders (`0 4px 12px / 0.10`).
- Radii: 4, 8, 12, pill. Nothing else.
- Spacing on a 4 px grid. Sections are 160 px apart.
- Type: Geist SemiBold for display with negative tracking (-0.048em at 96 px), Inter for everything else.

## Color jobs

- **Blue `#0075DE`** is the primary action and the current-page marker. Nothing else. Matches lumen's `Accent::Blue`.
- **Marigold, peach, sky, midnight** are decorative: they paint panels behind product renders and the one highlighted word. They are never buttons, links or text.

## The signature device

One word per page may be "lit": a pill highlight behind it (peach in light, marigold in dark). Only on display headlines.

## Imagery

Only things lumen draws: live demos (WebAssembly) or screenshots rendered headless by `scripts/screenshots.sh`. No illustration, no stock, no icons used as decoration.

## Don'ts

- No gradients, glows, blurred blobs, grid textures or noise.
- No icon-in-a-tinted-square feature grids. Explain features with real renders and code.
- No vanity stat strips, no emoji, no fake window chrome.
- No second filled button color on a page.
- No `transition: all`; animate named properties, and respect reduced motion.

## Logo

An ink tile (radius 14/64), a white "l" stroke that turns into a foot, and one marigold point of light. Inverted tile for dark backgrounds. Flat, two colors plus ink, legible at 16 px. Files in `site/assets/`.
