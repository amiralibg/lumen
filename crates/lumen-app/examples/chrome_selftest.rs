//! Real-window check of `App::custom_chrome` on macOS.
//!
//! `cargo run -p lumen-app --example chrome_selftest`
//!
//! Opens a window with lumen's title bar and drives it by posting `NSEvent`s
//! into its own event queue (no global input, no focus change): the zoom
//! button, a title bar double-click, a title bar press followed by a click in
//! the body, minimize, then close. It checks the `NSWindow` after each step
//! and prints PASS or FAIL lines.

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("chrome_selftest runs on macOS only");
}

#[cfg(target_os = "macos")]
fn main() {
    mac::run();
}

#[cfg(target_os = "macos")]
mod mac {
    use std::sync::{Arc, Mutex};

    use lumen_app::App;
    use lumen_core::{Look, Rect, Ui, Vec2};
    use objc2::rc::Retained;
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSEvent, NSEventModifierFlags, NSEventType, NSWindow};
    use objc2_foundation::{NSObjectNSDelayedPerforming, NSPoint, NSRect, NSSize};
    use std::cell::Cell;

    thread_local! {
        /// Delay for the next posted event, so a step's events stay in order.
        static DELAY: Cell<f64> = const { Cell::new(0.0) };
    }

    #[derive(Default)]
    struct St {
        frame: u32,
        step: usize,
        wait: u32,
        body_clicks: u32,
        bar: Rect,
        zoom: Rect,
        min: Rect,
        close: Rect,
        button: Rect,
        zoomed_before: bool,
        small_w: f64,
        managed: bool,
        saw_zoom: u32,
        saw_double: u32,
        log: Arc<Mutex<Vec<String>>>,
    }

    fn window(mtm: MainThreadMarker) -> Option<Retained<NSWindow>> {
        NSApplication::sharedApplication(mtm).windows().iter().find(|w| w.isVisible() || w.isMiniaturized())
    }

    fn post(mtm: MainThreadMarker, kind: NSEventType, p: Vec2, clicks: isize) {
        let Some(w) = window(mtm) else { return };
        let h = w.contentView().map(|v| v.frame().size.height).unwrap_or(0.0);
        let loc = NSPoint::new(p.x as f64, h - p.y as f64);
        let e = NSEvent::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure(
            kind,
            loc,
            NSEventModifierFlags::empty(),
            0.0,
            w.windowNumber(),
            None,
            0,
            clicks,
            1.0,
        );
        if let Some(e) = e {
            // Deliver from the run loop, outside winit's callback (sending
            // it from inside would re-enter winit and be dropped).
            let d = DELAY.with(|c| {
                let d = c.get();
                c.set(d + 0.02);
                d
            });
            unsafe { w.performSelector_withObject_afterDelay(objc2::sel!(sendEvent:), Some(&e), d) };
        }
    }

    fn click(mtm: MainThreadMarker, p: Vec2, clicks: isize) {
        post(mtm, NSEventType::MouseMoved, p, 0);
        post(mtm, NSEventType::LeftMouseDown, p, clicks);
        post(mtm, NSEventType::LeftMouseUp, p, clicks);
    }

    fn check(s: &St, ok: bool, what: &str) {
        let line = format!("{} {what}", if ok { "PASS" } else { "FAIL" });
        println!("{line}");
        s.log.lock().unwrap().push(line);
    }

    fn view(ui: &mut Ui, s: &mut St) {
        let shown = ui.window_chrome("chrome", "Chrome self-test").window().show(|ui| {
            ui.column().pad(24).gap(12).show(|ui| {
                ui.label("lumen draws this title bar; the OS's is hidden.");
                let b = ui.button("Count a click").show();
                if b.clicked {
                    s.body_clicks += 1;
                }
                s.button = b.rect;
                ui.label(format!("Body clicks: {}", s.body_clicks));
            });
        });
        let r = shown.response;
        s.saw_zoom += r.zoom as u32;
        s.saw_double += r.double_clicked as u32;
        (s.bar, s.zoom, s.min, s.close) = (r.title_bar, r.zoom_rect, r.minimize_rect, r.close_rect);
        ui.request_repaint_after(0.03);
        s.frame += 1;
        if s.wait > 0 {
            s.wait -= 1;
            return;
        }
        let mtm = MainThreadMarker::new().expect("main thread");
        DELAY.with(|c| c.set(0.0));
        let Some(w) = window(mtm) else { return };
        let title = Vec2 { x: s.bar.center().x + 120.0, y: s.bar.center().y };
        if std::env::var_os("PROBE").is_some() {
            if s.frame == 20 {
                println!("probe: button at {:?}", s.button);
                click(mtm, s.button.center(), 1);
            }
            if s.frame == 40 {
                println!("probe: body clicks = {}", s.body_clicks);
                std::process::exit(0);
            }
            return;
        }
        match s.step {
            0 if s.frame > 20 && s.zoom.width() > 0.0 => {
                check(s, ui.window_regions().len() == 4, "window mode registers 3 buttons and a drag region");
                // Start small so zooming visibly grows the window (a tiling
                // window manager may have sized it to the whole screen).
                let f = w.frame();
                w.setFrame_display(NSRect::new(f.origin, NSSize::new(640.0, 400.0)), true);
                s.wait = 20;
                s.step = 10;
            }
            10 => {
                s.small_w = w.frame().size.width;
                s.managed = (s.small_w - 640.0).abs() > 1.0;
                if s.managed {
                    println!("SKIP a window manager keeps this window at {} px wide, so zoom sizes can't be measured; checking that the gestures reach the app instead", s.small_w);
                }
                s.zoomed_before = w.isZoomed();
                click(mtm, s.zoom.center(), 1);
                s.wait = 30;
                s.step = 1;
            }
            1 => {
                println!("  zoom: width {} -> {}, isZoomed {} -> {}", s.small_w, w.frame().size.width, s.zoomed_before, w.isZoomed());
                if s.managed {
                    check(s, s.saw_zoom == 1, "zoom button click reaches the app (ChromeResponse::zoom, ToggleMaximize sent)");
                } else {
                    check(s, w.frame().size.width > s.small_w + 1.0, "zoom button zooms the window");
                }
                click(mtm, s.zoom.center(), 1);
                s.wait = 30;
                s.step = 2;
            }
            2 => {
                println!("  restore: width {}", w.frame().size.width);
                if s.managed {
                    check(s, s.saw_zoom == 2, "second zoom click reaches the app");
                } else {
                    check(s, (w.frame().size.width - s.small_w).abs() < 1.0, "zoom button again restores");
                }
                click(mtm, title, 1);
                click(mtm, title, 2);
                s.wait = 30;
                s.step = 3;
            }
            3 => {
                // AppleActionOnDoubleClick decides; the default zooms.
                let action = lumen_app::system::title_double_click();
                let ok = match action {
                    lumen_app::system::TitleAction::Maximize if s.managed => s.saw_double == 1,
                    lumen_app::system::TitleAction::Maximize => w.frame().size.width > s.small_w + 1.0,
                    lumen_app::system::TitleAction::Minimize => w.isMiniaturized(),
                    lumen_app::system::TitleAction::None => w.isZoomed() == s.zoomed_before,
                };
                check(s, ok, &format!("title bar double-click follows the OS setting ({action:?}), toggled once"));
                if w.isMiniaturized() {
                    w.deminiaturize(None);
                } else if w.frame().size.width > s.small_w + 1.0 {
                    w.zoom(None);
                }
                s.wait = 30;
                s.step = 4;
            }
            4 => {
                // A press on the title bar starts a native move; the UI must
                // not be left thinking the button is still held.
                post(mtm, NSEventType::LeftMouseDown, title, 1);
                s.wait = 10;
                s.step = 5;
            }
            5 => {
                click(mtm, s.button.center(), 1);
                s.wait = 10;
                s.step = 6;
            }
            6 => {
                check(s, s.body_clicks == 1, "after a title bar press the body still takes clicks");
                click(mtm, s.min.center(), 1);
                s.wait = 40;
                s.step = 7;
            }
            7 => {
                check(s, w.isMiniaturized(), "minimize button miniaturizes the window");
                w.deminiaturize(None);
                s.wait = 60;
                s.step = 8;
            }
            8 => {
                click(mtm, s.close.center(), 1);
                s.step = 9;
            }
            _ => {}
        }
    }

    pub fn run() {
        let log = Arc::new(Mutex::new(vec![]));
        let st = St { log: log.clone(), ..Default::default() };
        // A watchdog in case frames stop (e.g. a minimized window).
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_secs(40));
            println!("FAIL timed out");
            std::process::exit(2);
        });
        let app = App::new("lumen chrome self-test").size(640, 400).look(Look::mac());
        let app = if std::env::var_os("PLAIN").is_some() { app } else { app.custom_chrome() };
        app.run(st, view);
        let log = log.lock().unwrap();
        let fails = log.iter().filter(|l| l.starts_with("FAIL")).count();
        println!("PASS close button closes the window (event loop exited)");
        println!("{} checks, {fails} failed", log.len() + 1);
        std::process::exit(if fails == 0 && log.len() == 6 { 0 } else { 1 });
    }
}
