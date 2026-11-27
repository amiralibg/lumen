#![allow(unexpected_cfgs)] // objc 0.2 macros
//! A scripted session in a real GPUI window (macOS): clicks, typing and IME
//! composition through the real NSTextInputClient path, a select whose menu
//! overflows the lumen panel over GPUI content, and an animation to time.
//!
//! cargo run --example selftest [-- out_dir]
//!
//! Prints PASS/FAIL per check, saves window screenshots to `out_dir`
//! (default: target/selftest) and exits non-zero on failure. Run with
//! LUMEN_GPUI_TRACE=1 to log frame times, and LUMEN_GPUI_SYNC=1 to compare
//! against blocking readback.


#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("selftest drives the macOS text input client; skipping on this platform");
}

#[cfg(target_os = "macos")]
fn main() {
    mac::run();
}

#[cfg(target_os = "macos")]
mod mac {
    use std::cell::RefCell;
    use std::ffi::CString;
    use std::path::PathBuf;
    use std::rc::Rc;
    use std::time::Duration;

    use gpui::{
        div, point, prelude::*, px, rgb, size, App, AsyncWindowContext, Bounds, Context, Entity, Pixels,
        Point, Window, WindowBounds, WindowOptions,
    };
    use lumen_gpui::lumen_core::{Align, Rect, Ui, WindowRegion};
    use lumen_gpui::LumenView;
    use objc::runtime::Object;
    use objc::{class, msg_send, sel, sel_impl, Encode, Encoding};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    const WINDOW_H: f64 = 520.0;
    const REGIONS: [&str; 4] = ["Frankfurt", "Virginia", "Tokyo", "Sydney"];

    #[derive(Default)]
    struct Panel {
        text: String,
        region: u8,
        spin: bool,
        input: Rect,
        select: Rect,
    }

    fn panel(ui: &mut Ui, s: &mut Panel) {
        ui.column().fill().pad(16).gap(12).show(|ui| {
            s.input = ui.text_input(&mut s.text).placeholder("Type here").id("name").fill_width().show().rect;
            ui.row().fill_width().gap(12).align(Align::Center).show(|ui| {
                s.select = ui.select(&mut s.region, &[(0, REGIONS[0]), (1, REGIONS[1]), (2, REGIONS[2]), (3, REGIONS[3])]).label("Region").show().rect;
                if s.spin {
                    ui.spinner(20.0);
                    ui.progress(None);
                }
            });
        });
    }

    /// lumen's own title bar, in a view of its own above everything.
    const BAR_H: f32 = 38.0;

    #[derive(Default)]
    struct Bar {
        zooms: u32,
        minimizes: u32,
    }

    fn bar(ui: &mut Ui, s: &mut Bar) {
        // Report the clicks but let the test decide what to do with them,
        // except minimize, which the view performs.
        let r = ui.window_chrome("bar", "lumen-gpui self-test").window().handle_buttons(false).show(|_| {}).response;
        s.zooms += r.zoom as u32;
        s.minimizes += r.minimize as u32;
        if r.minimize {
            ui.request_window(lumen_gpui::lumen_core::WindowRequest::Minimize);
        }
    }

    /// SELFTEST_PANEL_H=676 matches the 1352 px-tall panel used for timing.
    fn panel_height() -> f32 {
        std::env::var("SELFTEST_PANEL_H").ok().and_then(|v| v.parse().ok()).unwrap_or(120.0)
    }

    /// lumen reports window coordinates, except with contain_overlays,
    /// where they're relative to the panel (right of the 180 pt sidebar).
    fn panel_origin() -> lumen_gpui::lumen_core::Vec2 {
        if std::env::var_os("SELFTEST_CONTAIN").is_some() {
            lumen_gpui::lumen_core::vec2(180.0, BAR_H)
        } else {
            lumen_gpui::lumen_core::Vec2::ZERO
        }
    }

    struct Shell {
        bar: Entity<LumenView<Bar>>,
        panel: Entity<LumenView<Panel>>,
    }

    impl Render for Shell {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let row = div()
                .flex_1()
                .flex()
                .child(div().w(px(180.)).h_full().p_3().bg(rgb(0x12151c)).child("GPUI sidebar"))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        // A short lumen panel: its select menu has to overflow it.
                        .child(div().h(px(panel_height())).child(self.panel.clone()))
                        .child(div().flex_1().p_4().bg(rgb(0x1a2030)).child("GPUI content below the lumen panel")),
                );
            div()
                .size_full()
                .flex()
                .flex_col()
                .bg(rgb(0x0b0d12))
                .text_color(rgb(0xe6e9ef))
                .child(div().h(px(BAR_H)).child(self.bar.clone()))
                .child(row)
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct NSRange {
        location: usize,
        length: usize,
    }
    unsafe impl Encode for NSRange {
        fn encode() -> Encoding {
            unsafe { Encoding::from_str("{_NSRange=QQ}") }
        }
    }
    const NOT_FOUND: NSRange = NSRange { location: isize::MAX as usize, length: 0 };

    fn ns_string(s: &str) -> *mut Object {
        let c = CString::new(s).unwrap();
        unsafe {
            let o: *mut Object = msg_send![class!(NSString), alloc];
            msg_send![o, initWithUTF8String: c.as_ptr()]
        }
    }

    /// What macOS calls on the view while an input method composes.
    fn set_marked(view: usize, text: &str, sel: NSRange) {
        let s = ns_string(text);
        unsafe {
            let _: () = msg_send![view as *mut Object, setMarkedText: s selectedRange: sel replacementRange: NOT_FOUND];
        }
    }

    fn insert(view: usize, text: &str) {
        let s = ns_string(text);
        unsafe {
            let _: () = msg_send![view as *mut Object, insertText: s replacementRange: NOT_FOUND];
        }
    }

    fn window_number(view: usize) -> i64 {
        unsafe {
            let w: *mut Object = msg_send![view as *mut Object, window];
            msg_send![w, windowNumber]
        }
    }

    fn pt(x: f32, y: f32) -> Point<Pixels> {
        point(px(x), px(y))
    }

    struct Script {
        cx: AsyncWindowContext,
        panel: Entity<LumenView<Panel>>,
        bar: Entity<LumenView<Bar>>,
        view: usize,
        out: PathBuf,
        results: Rc<RefCell<Vec<(String, bool)>>>,
    }

    impl Script {
        async fn wait(&self, ms: u64) {
            self.cx.background_executor().timer(Duration::from_millis(ms)).await;
        }

        /// A real NSEvent sent to the window, so GPUI's macOS event
        /// handling runs exactly as for a hardware click.
        fn send_mouse(&self, kind: u64, p: Point<Pixels>) {
            #[repr(C)]
            #[derive(Clone, Copy)]
            struct NSPoint {
                x: f64,
                y: f64,
            }
            unsafe impl Encode for NSPoint {
                fn encode() -> Encoding {
                    unsafe { Encoding::from_str("{CGPoint=dd}") }
                }
            }
            // Content height in points, to flip y for AppKit. Read per event:
            // the window can still be settling its size when the script starts.
            let height = self.cx.clone().update(|w, _| f32::from(w.viewport_size().height) as f64).unwrap();
            unsafe {
                let view = self.view as *mut Object;
                let window: *mut Object = msg_send![view, window];
                let num: i64 = msg_send![window, windowNumber];
                // Window coordinates have their origin at the bottom left.
                let loc = NSPoint { x: f32::from(p.x) as f64, y: height - f32::from(p.y) as f64 };
                let e: *mut Object = msg_send![class!(NSEvent), mouseEventWithType: kind location: loc modifierFlags: 0u64 timestamp: 0.0f64 windowNumber: num context: std::ptr::null_mut::<Object>() eventNumber: 0i64 clickCount: 1i64 pressure: 1.0f32];
                let _: () = msg_send![window, sendEvent: e];
            }
        }

        async fn click(&mut self, p: Point<Pixels>) {
            const MOVED: u64 = 5;
            const DOWN: u64 = 1;
            const UP: u64 = 2;
            self.send_mouse(MOVED, p);
            self.wait(80).await;
            self.send_mouse(DOWN, p);
            self.wait(40).await;
            self.send_mouse(UP, p);
            self.wait(120).await;
        }

        fn read<R>(&self, f: impl FnOnce(&LumenView<Panel>) -> R) -> R {
            let (mut cx, panel) = (self.cx.clone(), self.panel.clone());
            cx.update(|_, cx| f(panel.read(cx))).unwrap()
        }

        fn check(&self, name: &str, ok: bool, detail: String) {
            println!("{} {name}{}", if ok { "PASS" } else { "FAIL" }, if ok { String::new() } else { format!(": {detail}") });
            self.results.borrow_mut().push((name.into(), ok));
        }

        fn shot(&self, name: &str) {
            let path = self.out.join(format!("{name}.png"));
            let n = window_number(self.view);
            let ok = std::process::Command::new("screencapture").args(["-x", "-o", &format!("-l{n}")]).arg(&path).status().is_ok_and(|s| s.success());
            println!("     screenshot {} {}", path.display(), if ok { "" } else { "(failed)" });
        }

        async fn run(mut self) {
            self.wait(900).await;
            // GPUI stops drawing an occluded window (its display link
            // pauses), so screenshots of a hidden window go stale.
            let visible = unsafe {
                let w: *mut Object = msg_send![self.view as *mut Object, window];
                let state: u64 = msg_send![w, occlusionState];
                state & (1 << 1) != 0
            };
            let key = unsafe {
                let w: *mut Object = msg_send![self.view as *mut Object, window];
                let k: objc::runtime::BOOL = msg_send![w, isKeyWindow];
                k == objc::runtime::YES
            };
            if !visible || !key {
                println!("note: the window is {}; GPUI draws it only around input, so the script moves the pointer while it waits", if visible { "not key" } else { "occluded" });
            }
            // A cold first launch (new GPU device, uncompiled pipelines, font
            // loading) can take longer than the wait above; click only once
            // lumen has laid out the panel.
            for _ in 0..100 {
                if self.read(|p| p.state().input.width() > 0.0) {
                    break;
                }
                self.send_mouse(5, pt(450.0, 400.0));
                self.wait(100).await;
            }
            let input = self.read(|p| p.state().input).translate(panel_origin());
            self.click(pt(input.center().x, input.center().y)).await;

            // Plain typing arrives through insertText.
            insert(self.view, "Tok");
            self.wait(120).await;
            let t = self.read(|p| p.state().text.clone());
            self.check("insertText types into the focused field", t == "Tok", format!("{t:?}"));

            // Composition: setMarkedText twice, then commit.
            set_marked(self.view, "nih", NSRange { location: 3, length: 0 });
            self.wait(60).await;
            set_marked(self.view, "にほん", NSRange { location: 3, length: 0 });
            self.wait(250).await;
            let t = self.read(|p| p.state().text.clone());
            self.check("setMarkedText shows the composition in the field", t == "Tokにほん", format!("{t:?}"));
            self.shot("ime-composing");
            insert(self.view, "日本");
            self.wait(250).await;
            let t = self.read(|p| p.state().text.clone());
            self.check("insertText commits the composition", t == "Tok日本", format!("{t:?}"));
            self.shot("ime-committed");

            // A select at the bottom of a 120 px panel: its menu overflows
            // into the GPUI area below and must take clicks there.
            let sel = self.read(|p| p.state().select).translate(panel_origin());
            self.click(pt(sel.center().x, sel.center().y)).await;
            self.wait(400).await;
            let (open, over, menus) = self.read(|p| (p.has_open_overlay(), p.is_overflowing(), p.overlay_bounds()));
            let menu = menus.first().copied();
            let below = menu.is_some_and(|m| f32::from(m.bottom()) > panel_height());
            if std::env::var_os("SELFTEST_CONTAIN").is_some() {
                let inside = menu.is_some_and(|m| f32::from(m.bottom()) <= panel_height() + 1.0);
                self.check("contained select menu opens inside the panel", open && !over && inside, format!("open {open}, overflow {over}, menus {menus:?}"));
            } else if panel_height() < 200.0 {
                self.check("select menu opens and overflows the panel", open && over && below, format!("open {open}, overflow {over}, menus {menus:?}"));
            } else {
                self.check("select menu opens", open, format!("open {open}, menus {menus:?}"));
            }
            self.shot("overflow-menu");
            if let Some(m) = menu {
                // The last option sits well below the panel.
                self.click(pt(f32::from(m.center().x), f32::from(m.bottom()) - 22.0)).await;
                let mut waited = 0;
                while waited < 1500 && self.read(|p| p.has_open_overlay()) {
                    // A resting pointer still produces moves; GPUI needs
                    // input to keep drawing a window that isn't key.
                    self.send_mouse(5, pt(f32::from(m.center().x) + 300.0, 600.0));
                    self.wait(50).await;
                    waited += 50;
                }
                println!("     menu closed {waited} ms after the click");
                let (region, open) = self.read(|p| (p.state().region, p.has_open_overlay()));
                self.check("clicking an option outside the panel selects it", region == 3 && !open, format!("region {region}, open {open}"));
            }
            self.click(pt(sel.center().x, sel.center().y)).await;
            self.wait(400).await;
            self.click(pt(600.0, 420.0)).await;
            self.wait(400).await;
            let open = self.read(|p| p.has_open_overlay());
            self.check("clicking GPUI content closes the menu", !open, format!("open {open}"));
            self.shot("closed");

            // Animate for a while (frame times go to stderr with
            // LUMEN_GPUI_TRACE=1).
            let panel = self.panel.clone();
            let _ = self.cx.update(|_, cx| panel.update(cx, |p, cx| p.update(cx, |s| s.spin = true)));
            self.wait(3000).await;
            let _ = self.cx.update(|_, cx| panel.update(cx, |p, cx| p.update(cx, |s| s.spin = false)));
            self.wait(200).await;

            // lumen's title bar: regions for GPUI, and its buttons.
            let bar = self.bar.clone();
            let (mut cx, b) = (self.cx.clone(), bar.clone());
            let regions = cx.update(|_, cx| b.read(cx).window_control_regions()).unwrap();
            let kinds: Vec<WindowRegion> = regions.iter().map(|r| r.0).collect();
            self.check(
                "window chrome exposes three buttons and a drag region",
                kinds.len() == 4 && kinds[3] == WindowRegion::Drag && f32::from(regions[3].1.size.width) > 800.0,
                format!("{regions:?}"),
            );
            let at = |k: WindowRegion| regions.iter().find(|r| r.0 == k).map(|r| r.1.center());
            if let Some(z) = at(WindowRegion::Maximize) {
                self.click(z).await;
                self.wait(300).await;
                let (mut cx, b) = (self.cx.clone(), bar.clone());
                let zooms = cx.update(|_, cx| b.read(cx).state().zooms).unwrap();
                self.check("zoom button click reaches the app", zooms == 1, format!("zooms {zooms}"));
            }
            if let Some(m) = at(WindowRegion::Minimize) {
                self.click(m).await;
                self.wait(900).await;
                let mini = unsafe {
                    let w: *mut Object = msg_send![self.view as *mut Object, window];
                    let m: objc::runtime::BOOL = msg_send![w, isMiniaturized];
                    let _: () = msg_send![w, deminiaturize: std::ptr::null_mut::<Object>()];
                    m == objc::runtime::YES
                };
                self.check("minimize button miniaturizes the GPUI window", mini, "still on screen".into());
                self.wait(900).await;
            }

            let failed = self.results.borrow().iter().filter(|r| !r.1).count();
            println!("{} checks, {failed} failed", self.results.borrow().len());
            std::process::exit(if failed == 0 { 0 } else { 1 });
        }
    }

    pub fn run() {
        let out = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "target/selftest".into()));
        std::fs::create_dir_all(&out).unwrap();
        lumen_gpui::run(lumen_gpui::application(), move |cx: &mut App| {
            let bounds = Bounds::centered(None, size(px(900.), px(WINDOW_H as f32)), cx);
            // SELFTEST_BACKGROUND=1 leaves the app inactive and the window
            // unfocused, so it doesn't take over the screen while you work.
            let background = std::env::var_os("SELFTEST_BACKGROUND").is_some();
            let options = WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), focus: !background, ..Default::default() };
            let window = cx
                .open_window(options, |window, cx| {
                    // SELFTEST_CONTAIN=1 keeps menus inside the panel.
                    let contain = std::env::var_os("SELFTEST_CONTAIN").is_some();
                    let panel = cx.new(|cx| {
                        let v = LumenView::new(Panel::default(), panel, window, cx);
                        if contain { v.contain_overlays() } else { v }
                    });
                    let bar = cx.new(|cx| LumenView::new(Bar::default(), bar, window, cx).contain_overlays());
                    cx.new(|_| Shell { bar, panel })
                })
                .unwrap();
            if !background {
                cx.activate(true);
            }
            window
                .update(cx, |shell, window, cx| {
                    let view = match window.window_handle().unwrap().as_raw() {
                        RawWindowHandle::AppKit(h) => h.ns_view.as_ptr() as usize,
                        _ => unreachable!(),
                    };
                    let panel = shell.panel.clone();
                    let bar = shell.bar.clone();
                    window
                        .spawn(cx, async move |cx| {
                            Script { cx: cx.clone(), panel, bar, view, out, results: Rc::default() }.run().await;
                        })
                        .detach();
                })
                .unwrap();
        });
    }
}
