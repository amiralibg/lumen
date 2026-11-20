# lumen-gpui

Host lumen widgets inside a [GPUI](https://gpui.rs) (Zed) application.

A `LumenView<S>` is an ordinary GPUI view over your state `S`. Put it anywhere in a GPUI element tree and it draws lumen's widgets, looks and motion in the space it's given. Mouse, scroll, keyboard, clipboard, cursors and the window's light or dark appearance are forwarded.

```toml
[dependencies]
gpui = "0.2.2"
lumen-gpui = { git = "https://github.com/amiralibg/lumen" }
```

```rust
let panel = lumen_gpui::LumenView::build(state, |ui, s| {
    ui.switch("Notifications", &mut s.notify);
}, window, cx);
```

```sh
cargo run --example dashboard   # a GPUI sidebar driving a lumen dashboard
```

## How it works

GPUI has no hook for another renderer, so lumen renders offscreen with wgpu (one device shared by all views), reads the frame back as BGRA and paints it as a GPUI image. Views at rest produce no frames. While animating, a 1760×1352-pixel panel took 3 to 9 ms per frame on Apple silicon in our measurements; set `LUMEN_GPUI_TRACE=1` to print frame times.

## Limits

- IME composition isn't forwarded yet; typed characters arrive as committed text.
- lumen overlays (menus, popovers, toasts) stay inside the view's bounds.
- No accessibility bridge to GPUI.

This crate keeps its own `Cargo.lock` and is excluded from the main workspace, because GPUI needs Xcode's Metal toolchain on macOS and Wayland/X11 headers on Linux.
