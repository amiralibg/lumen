# lumen-gpui

Host lumen widgets inside a [GPUI](https://gpui.rs) (Zed) application.

A `LumenView<S>` is an ordinary GPUI view over your state `S`. Put it anywhere in a GPUI element tree and it draws lumen's widgets, looks and motion in the space it's given. Mouse, scroll, keyboard, input methods, clipboard, cursors and the window's light or dark appearance are forwarded.

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
cargo run --example selftest    # scripted real-window check (macOS), see below
```

## Menus and popovers

Menus, select lists, popovers, context menus and tooltips may extend past the view's bounds, over neighbouring GPUI elements, like a native popup. While one sticks out, the view paints its frame in a GPUI `deferred` layer on top of the window, unclipped by scrolling parents, and takes the mouse only over the menu itself. A click anywhere else in the window closes it and still reaches the GPUI element under the pointer. Dialogs, sheets and toasts stay inside the view.

Call `.contain_overlays()` on the view to keep everything inside its bounds instead; menus then flip and shift to fit, the way a macOS pop-up button covers itself.

`has_open_overlay()` and `overlay_bounds()` tell your GPUI code where lumen's popups are, for example to avoid opening your own on top of them.

## Custom title bar

`ui.window_chrome("main", "Title").window()` inside a `LumenView` draws lumen's title bar for the current look and makes it work:

- A press on the bar moves the window (`start_window_move`; on macOS AppKit moves it natively, so give the window a transparent title bar with `TitlebarOptions { appears_transparent: true, .. }`). A double-click follows the OS setting (`titlebar_double_click` on macOS, zoom elsewhere).
- The buttons minimize, zoom and close the window. With `.handle_buttons(false)` they only report clicks in `ChromeResponse`, so you can confirm before closing.
- The view adds a GPUI `WindowControlArea` for the drag region and each button, which is what Windows hit-tests for snapping and caption behaviour. `window_control_regions()` returns the same rects if your own GPUI elements need to stay out of them.
- The bar dims when the window is inactive and shows restore while it's maximized.

On macOS GPUI keeps its own traffic lights; there, use `.closable(false).minimizable(false).maximizable(false)` to show only the title, or position the native lights over lumen's with `traffic_light_position`.

## Text input and IME

When a lumen text field is focused, the view registers a GPUI input handler, so input methods work: CJK composition shows in the field underlined (the segment being converted thicker) and commits on confirm, macOS press-and-hold accents and dead keys work, and the candidate window sits under the caret. Scripts Geist lacks (CJK, Arabic, Hebrew, emoji...) load the system fonts as fallbacks the first time they appear. Password fields take committed text only.

## How it works

GPUI has no hook for another renderer, so lumen renders offscreen with wgpu (one device shared by all views), reads the frame back as BGRA and paints it as a GPUI image. Views at rest produce no frames.

The readback is pipelined: GPUI shows the newest finished frame while the GPU renders the next, so the main thread never waits for the GPU. Only the first frame and frames after a resize wait, so you never see a stretched image. On a 1326×1352 px panel animating on Apple silicon (release build):

| | main thread per frame | p95 | submit to screen |
|---|---|---|---|
| blocking readback (`LUMEN_GPUI_SYNC=1`) | 3.1–3.6 ms | 4.4–5.1 ms | ~3 ms |
| pipelined (default) | 1.7–1.8 ms | 2.4–2.7 ms | one frame (~18 ms at 60 Hz) |

Set `LUMEN_GPUI_TRACE=1` to print build, submit and readback times per frame.

## gpui-ce

The community fork [gpui-ce](https://github.com/gpui-ce/gpui-ce) 0.3 is supported behind a feature (it needs Rust 1.95):

```toml
gpui = { package = "gpui-ce", version = "0.3" }
lumen-gpui = { git = "https://github.com/amiralibg/lumen", default-features = false, features = ["ce"] }
```

Start the app with `lumen_gpui::application()`, which picks the right constructor for either GPUI. The `ce` build compiles and passes the unit tests. It isn't verified in a real window yet: on macOS 27, gpui-ce 0.3.0 deadlocks inside its own `open_window` (its README hello world too), before any lumen code runs.

## Accessibility (gpui-ce)

With the `ce` feature, a lumen view's accessibility tree joins the window's: lumen's nodes become synthetic children of the view's element, nested as in lumen, and screen reader actions on them (press, focus, step, set value) reach lumen as input. Since GPUI gathers its tree while laying out, before lumen runs, lumen's part shows one frame later. `cargo +1.95 test --no-default-features --features ce-test --test a11y_ce` checks it headlessly.

## The self-test

`cargo run --example selftest [-- out_dir]` opens a real window and drives it through the real macOS paths: clicks are `NSEvent`s sent to the window, typing and IME composition go through `insertText:` and `setMarkedText:`. It checks typing, composition and commit, a select menu that overflows a 120 pt panel over GPUI content, picking an option outside the panel, and closing on an outside click, saves window screenshots, then animates for timing. `SELFTEST_CONTAIN=1` runs it with `contain_overlays()` (give it a panel taller than the menu, e.g. `SELFTEST_PANEL_H=400`); `SELFTEST_PANEL_H=676` uses the tall panel from the table above. It also checks a lumen title bar at the top of the window: its control regions, a zoom click, and that its minimize button really miniaturizes the window.

GPUI on macOS only draws a window that isn't key around input, so the script keeps moving the pointer while it waits, like a resting hand would.

## Limits

- Zed's GPUI 0.2 has no accessibility tree, so lumen views are invisible to screen readers there. gpui-ce bridges it (see below).
- While a menu overflows the view, frames are read back for the union of the view and the menu, and need a straight-alpha conversion on the CPU (about 1 ms more at Retina sizes).

This crate keeps its own `Cargo.lock` and is excluded from the main workspace, because GPUI needs Xcode's Metal toolchain on macOS and Wayland/X11 headers on Linux.
