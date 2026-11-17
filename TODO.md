# Phase 1 todo

- [x] 1. Repo, licenses, workspace skeleton, native + wasm32 check
- [x] 2. Fable: DESIGN.md tokens + widget specs (batch 1)
- [x] 3. lumen-text: fonts, shaping, atlas, tests
- [x] 4. lumen-core: Id, input, layout, painter, tokens, springs, tests
- [x] 5. lumen-wgpu: shader, atlas, offscreen PNG capture
- [x] 6. lumen-app + facade: hello world native + browser
- [x] 7. Widgets batch 1 + input tests
- [x] 8. Gallery: all widgets, theme toggle, accent picker, native + wasm
- [~] 9. Fable screenshot review → fixes → approval  ← round 1 in review
- [ ] 10. README.md + final DESIGN.md, report

# Looks and customization

- [x] `Look`: neutrals, control metrics, per-widget shapes, fonts, motion; blends on change
- [x] Presets: Lumen, macOS, Windows 11, GNOME; `Look::native()`
- [x] `Accent::Custom` / `Accent::Native` with contrast-checked derived shades; token overrides
- [x] All batch-1 widgets read the look; new `segmented` widget
- [x] lumen-app: live OS dark mode, OS accent, reduce motion; `App::native()` / `App::look()`
- [x] Gallery Appearance: look, system fonts, accent hue, roundness, density, motion style
- [ ] Screenshot review of each look in both modes, then per-platform polish
- [ ] Windows/Linux accent and reduce-motion queries are untested on real machines
- [ ] Layout: rows narrower than their content shrink hugging children below their descendants' `min` widths (no min-content propagation), so siblings can overlap at very narrow widths
