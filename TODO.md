# Phase 1 todo

- [x] 1. Repo, licenses, workspace skeleton, native + wasm32 check
- [x] 2. Fable: DESIGN.md tokens + widget specs (batch 1)
- [x] 3. lumen-text: fonts, shaping, atlas, tests
- [x] 4. lumen-core: Id, input, layout, painter, tokens, springs, tests
- [x] 5. lumen-wgpu: shader, atlas, offscreen PNG capture
- [x] 6. lumen-app + facade: hello world native + browser
- [x] 7. Widgets batch 1 + input tests
- [x] 8. Gallery: all widgets, theme toggle, accent picker, native + wasm
- [x] 9. Screenshot review → fixes → approval (final sweep 2026-11-25: 1,918 renders, every demo × 4 looks × 2 modes plus card and app widths; kanban, masonry, KPI strip, action bar, stat tiles and badges fixed at card width)
- [ ] 10. README.md + final DESIGN.md, report

# Looks and customization

- [x] `Look`: neutrals, control metrics, per-widget shapes, fonts, motion; blends on change
- [x] Presets: Lumen, macOS, Windows 11, GNOME; `Look::native()`
- [x] `Accent::Custom` / `Accent::Native` with contrast-checked derived shades; token overrides
- [x] All batch-1 widgets read the look; new `segmented` widget
- [x] lumen-app: live OS dark mode, OS accent, reduce motion; `App::native()` / `App::look()`
- [x] Gallery Appearance: look, system fonts, accent hue, roundness, density, motion style
- [x] Screenshot review of each look in both modes (`--check` sweep of every demo × 4 looks × 2 modes, 2026-11-21), then per-platform polish
- [ ] Windows/Linux accent and reduce-motion queries are untested on real machines
- [x] Layout: rows narrower than their content shrank hugging children below their descendants' `min` widths; min-content widths now propagate (fixed widths and `min` only; text and fixed-size content still shrink)
