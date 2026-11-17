# lumen

**A polished-by-default immediate-mode UI kit for Rust.** As easy as egui, but it looks like a top-tier product out of the box: spring motion, Inter and Geist type, Lucide icons, named theme tokens, and animated light/dark with eight accents.

```rust
#[derive(Default)]
struct State { notify: bool, volume: f32 }

fn main() {
    lumen::App::new("Settings").size(900, 600).run(State::default(), |ui, s| {
        ui.column().pad(24).gap(14).show(|ui| {
            ui.heading("General");
            ui.switch("Notifications", &mut s.notify);
            ui.slider("Volume", &mut s.volume, 0.0..=1.0);
            if ui.button("Save").primary().clicked() {
                println!("saved");
            }
        });
    });
}
```

Your state is a plain struct passed as `&mut`. Widgets are builders: every option has a good default, chained methods change it, and the widget is added when the builder is dropped, or when you call `.clicked()`, `.changed()` or `.show()`.

## Try it

```sh
cargo run -p lumen-gallery                 # native
./scripts/build-web.sh                     # browser build into web/dist
python3 -m http.server -d web/dist 8080    # then open http://localhost:8080
```

The web build needs `rustup target add wasm32-unknown-unknown` and `cargo install wasm-bindgen-cli --version 0.2.130`. That version must match the `wasm-bindgen` pinned in `Cargo.toml`.

Headless screenshots, used for design review:

```sh
cargo run -p lumen-gallery -- --screenshot out.png --dark --accent blue --scroll 700
# more flags: --size WxH --scale S --hover x,y --press x,y --click x,y --tab N --type TEXT --wait SECS --collapsed
```

## What's in phase 1

| Widget | API |
|---|---|
| Label | `ui.label(t)` with `.secondary()` `.muted()` `.strong()` `.caption()` `.mono()` `.truncate()` |
| Heading | `ui.heading(t)` with `.level(1..=4)` `.display()` |
| Button | `ui.button(t)` with `.primary()` `.secondary()` `.ghost()` `.danger()` `.small()` `.large()` `.icon("plus")` `.trailing_icon(..)` `.disabled(b)` `.full_width()` `.tooltip(..)`; also `ui.icon_button("copy")` |
| Switch | `ui.switch(label, &mut bool)` |
| Checkbox | `ui.checkbox(label, &mut bool)` with `.indeterminate(b)` `.description(..)` |
| Slider | `ui.slider(label, &mut f32, range)` with `.step(s)` `.readout("64%")` `.width(w)` |
| Text input | `ui.text_input(&mut String)` with `.label` `.placeholder` `.icon` `.helper` `.error(Option)` `.mono()` `.fill_width()` |
| Card | `ui.card().title(..).description(..).action(|ui| ..).footer(hint, |ui| ..).show(|ui| ..)` plus `ui.settings(|rows| rows.row(title, desc, |ui| control))` |
| Sidebar nav | `ui.sidebar().collapsed(b).show(|ui| { ui.nav_header(..); ui.nav_section(..); ui.nav_item(icon, label).badge(3).selected(b) })` |
| Tooltip | `.tooltip(..)` / `.tooltip_shortcut(..)` on buttons, or `ui.tooltip_for(id, spec)` |
| Theme | `ui.set_mode(Mode::Dark)`, `ui.set_accent(Accent::Blue)`, `ui.accent_picker()`, `ui.color(Token::Surface)` |

Layout uses `ui.row()`, `ui.column()` and `ui.stack()`, with `.pad` `.gap` `.align` `.justify` `.fill()` `.width` `.max_width`. Also `ui.scroll(key)`, `ui.spacer()`, `ui.space(px)` and `ui.divider()`.

Text inputs support caret, selection, double- and triple-click, word jumps, Cmd/Ctrl+A/C/X/V/Z, and undo. Everything is keyboard reachable: Tab and Shift+Tab move focus, Space and Enter activate, arrows drive sliders and the nav, and the focus ring shows only for keyboard focus.

## Design

All visuals come from [`DESIGN.md`](DESIGN.md) (tokens) and [`docs/widgets-batch1.md`](docs/widgets-batch1.md) (per-widget specs). Widgets read named tokens only. Color transitions blend in OKLab. Springs are solved in closed form, so they're frame-rate independent and keep their velocity when retargeted. An idle UI requests no frames.

## Architecture

```
lumen-text   cosmic-text shaping, bundled fonts, glyph atlas (no GPU)
   ↓
lumen-core   ids, input, layout, theme, springs, widgets → DrawData (triangles + clip cmds)
   ↓
lumen-wgpu   one pipeline, one WGSL shader (SDF rounded rects, blurred shadows, glyphs)
   ↓
lumen-app    winit event loop (native + wasm32), clipboard, headless screenshots
   ↓
lumen        facade crate
```

`lumen-core` never touches a window or a GPU. Each vertex is 64 bytes and carries its shape data, so a backend is just one shader. A tiny-skia software renderer is planned.

Each frame the user closure records a layout tree. Widgets resolve interaction against the previous frame's hit rects. After the closure returns, the tree is laid out (flex-style: hug, fixed or fill sizing, min/max, padding, gap, align, justify), then painted.

## Roadmap

1. **Overlays and pickers:** command palette, select, combobox, date picker, tabs, dialog, toast, popover, menu.
2. **Data:** table, then a virtualized data grid, tree.
3. **Charts:** line, bar, area, donut, sparkline.
4. **Code editor.**
5. **Docking panels.**
6. **Cross-cutting work:** AccessKit accessibility, a tiny-skia software renderer, font subsetting to shrink wasm, and rich text.

## License

MIT OR Apache-2.0. Bundled fonts: Inter and Geist (SIL OFL 1.1), Lucide (ISC). See [THIRD_PARTY.md](THIRD_PARTY.md).
