//! Window chrome: caption button clicks, title bar gestures, regions for
//! hosts, and window requests.

use lumen_core::*;

#[derive(Default)]
struct S {
    window: bool,
    handle: bool,
    platform: Option<Platform>,
    last: ChromeResponse,
    menu_open: bool,
}

fn view(ui: &mut Ui, s: &mut S) {
    if let Some(p) = s.platform {
        ui.set_look(p.look());
    }
    let mut c = ui.window_chrome("w", "Title");
    if s.window {
        c = c.window();
    }
    let shown = c.handle_buttons(s.handle).show(|ui| {
        let b = ui.button("Open").show();
        if s.menu_open {
            ui.set_popover_open(b.id, true);
            s.menu_open = false;
        }
        ui.popover(&b).at(vec2(300.0, 12.0)).show(|ui| {
            ui.label("A popover that covers the title bar");
        });
    });
    s.last = shown.response;
}

struct H {
    ui: Ui,
    t: f64,
    s: S,
}

impl H {
    fn new(s: S) -> Self {
        let mut h = H { ui: Ui::new(), t: 1.0, s };
        h.ui.theme.reduced_motion = true;
        for _ in 0..3 {
            h.frame(vec![]);
        }
        h
    }
    fn frame(&mut self, events: Vec<Event>) -> FrameOutput {
        self.t += 1.0 / 60.0;
        let input = RawInput { screen_size: vec2(800.0, 500.0), scale: 1.0, time: self.t, events, modifiers: Modifiers::default() };
        let s = &mut self.s;
        self.ui.run(input, |ui| view(ui, s))
    }
    fn click(&mut self, p: Vec2) -> FrameOutput {
        self.frame(vec![
            Event::PointerMoved(p),
            Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true },
            Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false },
        ])
    }
}

#[test]
fn window_mode_reports_regions_and_sends_requests() {
    for p in Platform::ALL {
        let mut h = H::new(S { window: true, handle: true, platform: Some(p), ..Default::default() });
        let out = h.frame(vec![]);
        let kinds: Vec<WindowRegion> = out.window_regions.iter().map(|(k, _)| *k).collect();
        assert_eq!(kinds.len(), 4, "{p:?}: three buttons and the drag region");
        assert_eq!(*kinds.last().unwrap(), WindowRegion::Drag, "drag region comes last");
        let bar = h.s.last.title_bar;
        assert!(bar.width() > 700.0, "{p:?}: title bar spans the window");

        // Hit testing agrees with the response rects.
        let close = h.s.last.close_rect;
        assert_eq!(h.ui.window_region_at(close.center()), Some(WindowRegion::Close), "{p:?}");
        assert_eq!(h.ui.window_region_at(h.s.last.minimize_rect.center()), Some(WindowRegion::Minimize));
        assert_eq!(h.ui.window_region_at(h.s.last.zoom_rect.center()), Some(WindowRegion::Maximize));
        assert_eq!(h.ui.window_region_at(bar.center()), Some(WindowRegion::Drag));
        assert_eq!(h.ui.window_region_at(vec2(400.0, 300.0)), None, "the body isn't a region");

        let out = h.click(close.center());
        assert!(h.s.last.close, "{p:?}");
        assert_eq!(out.window_requests, vec![WindowRequest::Close]);
        let out = h.click(h.s.last.zoom_rect.center());
        assert_eq!(out.window_requests, vec![WindowRequest::ToggleMaximize]);
        let out = h.click(h.s.last.minimize_rect.center());
        assert_eq!(out.window_requests, vec![WindowRequest::Minimize]);
    }
}

#[test]
fn unhandled_buttons_report_without_requests() {
    let mut h = H::new(S { window: true, handle: false, ..Default::default() });
    let c = h.s.last.close_rect.center();
    let out = h.click(c);
    assert!(h.s.last.close);
    assert!(out.window_requests.is_empty(), "the app decides (confirm before closing)");
}

#[test]
fn title_bar_press_and_double_click() {
    let mut h = H::new(S { window: true, handle: true, ..Default::default() });
    let p = h.s.last.title_bar.center();
    let out = h.click(p);
    assert!(h.s.last.drag_started && !h.s.last.double_clicked);
    assert!(out.window_requests.is_empty(), "hosts move and maximize natively on mouse-down");
    h.click(p);
    assert!(h.s.last.double_clicked);
}

#[test]
fn maximized_state_follows_the_host() {
    let mut h = H::new(S { window: true, handle: true, ..Default::default() });
    assert!(!h.s.last.maximized);
    h.frame(vec![Event::WindowMaximized(true)]);
    h.frame(vec![]);
    assert!(h.s.last.maximized && h.ui.window_maximized());
}

#[test]
fn preview_toggles_its_own_zoom_and_registers_no_regions() {
    let mut h = H::new(S::default());
    let out = h.frame(vec![]);
    assert!(out.window_regions.is_empty(), "a preview card isn't the window");
    let z = h.s.last.zoom_rect.center();
    let out = h.click(z);
    assert!(h.s.last.zoom && h.s.last.maximized);
    assert!(out.window_requests.is_empty());
    h.click(z);
    assert!(!h.s.last.maximized);
}

#[test]
fn popups_over_the_title_bar_win_hit_tests() {
    let mut h = H::new(S { window: true, handle: true, menu_open: true, ..Default::default() });
    h.frame(vec![]);
    let out = h.frame(vec![]);
    let r = *out.layer_rects.first().expect("popover open");
    let bar = h.s.last.title_bar;
    let over = vec2(r.center().x, (r.max.y.min(bar.max.y) + r.min.y.max(bar.min.y)) * 0.5);
    assert!(bar.contains(over) && r.contains(over), "the popover overlaps the bar: {r:?} {bar:?}");
    assert_eq!(h.ui.window_region_at(over), None);
}
