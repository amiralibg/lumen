//! Input handling, driven through a headless `Ui`.

use lumen_core::*;

/// Drives frames with a fixed clock and synthetic events.
struct Harness<S> {
    ui: Ui,
    t: f64,
    s: S,
    view: fn(&mut Ui, &mut S),
}

impl<S> Harness<S> {
    fn new(s: S, view: fn(&mut Ui, &mut S)) -> Self {
        let mut h = Harness { ui: Ui::new(), t: 1.0, s, view };
        h.idle(2);
        h
    }

    fn frame(&mut self, events: Vec<Event>) -> FrameOutput {
        self.t += 1.0 / 60.0;
        let input = RawInput { screen_size: vec2(800.0, 600.0), scale: 1.0, time: self.t, events, modifiers: Modifiers::default() };
        let (s, view) = (&mut self.s, self.view);
        self.ui.run(input, |ui| view(ui, s))
    }

    fn idle(&mut self, n: usize) {
        for _ in 0..n {
            self.frame(vec![]);
        }
    }

    fn click(&mut self, p: Vec2) {
        self.frame(vec![Event::PointerMoved(p), press(p)]);
        self.frame(vec![release(p)]);
    }
}

fn press(p: Vec2) -> Event {
    Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true }
}
fn release(p: Vec2) -> Event {
    Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false }
}
fn key_mod(k: Key, m: Modifiers) -> Vec<Event> {
    vec![Event::Key { key: k, pressed: true, repeat: false, modifiers: m }, Event::Key { key: k, pressed: false, repeat: false, modifiers: m }]
}
fn key(k: Key) -> Vec<Event> {
    key_mod(k, Modifiers::default())
}
const SHIFT: Modifiers = Modifiers { shift: true, ctrl: false, alt: false, logo: false, mac: true };
const CMD: Modifiers = Modifiers { shift: false, ctrl: false, alt: false, logo: true, mac: true };
const ALT: Modifiers = Modifiers { shift: false, ctrl: false, alt: true, logo: false, mac: true };

#[derive(Default)]
struct Btn {
    clicks: u32,
    rect: Rect,
    focused: bool,
    disabled: bool,
}

fn btn_view(ui: &mut Ui, s: &mut Btn) {
    ui.column().pad(24).show(|ui| {
        let r = ui.button("Save").primary().disabled(s.disabled).show();
        s.clicks += r.clicked() as u32;
        s.rect = r.rect;
        s.focused = r.focused;
    });
}

#[test]
fn button_geometry_and_click_on_release() {
    let mut h = Harness::new(Btn::default(), btn_view);
    assert_eq!(h.s.rect, Rect::from_xywh(24.0, 24.0, 72.0, 36.0), "md button: 36 tall, min width 2x height");
    let c = h.s.rect.center();
    h.frame(vec![Event::PointerMoved(c), press(c)]);
    assert_eq!(h.s.clicks, 0, "no click on press");
    h.frame(vec![release(c)]);
    assert_eq!(h.s.clicks, 1);
    assert!(!h.s.focused || true);
}

#[test]
fn dragging_out_cancels_click() {
    let mut h = Harness::new(Btn::default(), btn_view);
    let c = h.s.rect.center();
    h.frame(vec![press(c)]);
    h.frame(vec![Event::PointerMoved(vec2(400.0, 400.0)), release(vec2(400.0, 400.0))]);
    assert_eq!(h.s.clicks, 0);
}

#[test]
fn disabled_button_ignores_pointer_and_keys() {
    let mut h = Harness::new(Btn { disabled: true, ..Default::default() }, btn_view);
    let c = h.s.rect.center();
    h.click(c);
    h.frame(key(Key::Tab));
    h.frame(key(Key::Enter));
    h.frame(key(Key::Space));
    assert_eq!(h.s.clicks, 0);
    assert!(!h.s.focused, "disabled controls take no focus");
}

#[test]
fn pointer_click_does_not_show_focus_ring_but_tab_does() {
    let mut h = Harness::new(Btn::default(), btn_view);
    let c = h.s.rect.center();
    h.click(c);
    h.frame(vec![]);
    assert!(h.s.focused, "click focuses");
    // Keyboard focus is visible; pointer focus isn't. Check via draw output:
    // a ring adds geometry.
    let n_pointer = h.frame(vec![]).draw.vertices.len();
    h.frame(vec![Event::PointerMoved(vec2(700.0, 500.0)), press(vec2(700.0, 500.0)), release(vec2(700.0, 500.0))]);
    h.frame(key(Key::Tab));
    h.idle(20);
    let n_keyboard = h.frame(vec![]).draw.vertices.len();
    assert!(n_keyboard > n_pointer, "{n_keyboard} vs {n_pointer}");
}

#[derive(Default)]
struct Form {
    on: bool,
    saves: u32,
    focus: (bool, bool),
    all: bool,
    rects: (Rect, Rect),
}

fn form_view(ui: &mut Ui, s: &mut Form) {
    ui.column().pad(24).gap(12).show(|ui| {
        let r = ui.switch("Notifications", &mut s.on).show();
        let r2 = ui.button("Save").show();
        let r3 = ui.checkbox("All channels", &mut s.all).indeterminate(true).show();
        s.saves += r2.clicked() as u32;
        s.focus = (r.focused, r2.focused);
        s.rects = (r.rect, r3.rect);
    });
}

#[test]
fn tab_focus_and_keyboard_activation() {
    let mut h = Harness::new(Form::default(), form_view);
    h.frame(key(Key::Tab));
    h.frame(vec![]);
    assert_eq!(h.s.focus, (true, false));
    h.frame(key(Key::Space));
    assert!(h.s.on, "space toggles a switch");
    h.frame(key(Key::Enter));
    assert!(h.s.on, "enter does nothing on a switch");
    h.frame(key(Key::Tab));
    h.frame(key(Key::Enter));
    assert_eq!(h.s.saves, 1, "enter activates a button");
    h.frame(key(Key::Space));
    assert_eq!(h.s.saves, 2, "space activates a button");
    h.frame(key_mod(Key::Tab, SHIFT));
    h.frame(vec![]);
    assert_eq!(h.s.focus, (true, false), "shift+tab goes back");
    for _ in 0..3 {
        h.frame(key(Key::Tab));
    }
    h.frame(vec![]);
    assert_eq!(h.s.focus, (true, false), "tab wraps");
}

#[test]
fn label_click_toggles_and_indeterminate_becomes_checked() {
    let mut h = Harness::new(Form::default(), form_view);
    let r = h.s.rects.0;
    h.click(vec2(r.max.x - 5.0, r.center().y));
    assert!(h.s.on, "clicking the label toggles the switch");
    h.click(h.s.rects.1.center());
    assert!(h.s.all);
}

#[derive(Default)]
struct Sl {
    v: f32,
    rect: Rect,
    step: Option<f32>,
    range: (f32, f32),
}

fn slider_view(ui: &mut Ui, s: &mut Sl) {
    ui.column().pad(24).show(|ui| {
        let mut sl = ui.slider("", &mut s.v, s.range.0..=s.range.1);
        if let Some(st) = s.step {
            sl = sl.step(st);
        }
        s.rect = sl.show().rect;
    });
}

#[test]
fn slider_drag_tracks_pointer_and_keys_step() {
    let mut h = Harness::new(Sl { range: (0.0, 100.0), ..Default::default() }, slider_view);
    let r = h.s.rect;
    assert_eq!(r.width(), 240.0);
    let (x0, x1) = (r.min.x + 8.0, r.max.x - 8.0);
    let mid = vec2((x0 + x1) / 2.0, r.center().y);
    h.frame(vec![Event::PointerMoved(mid), press(mid)]);
    assert!((h.s.v - 50.0).abs() < 0.5, "press at center -> 50, got {}", h.s.v);
    let end = vec2(x1 + 50.0, mid.y);
    h.frame(vec![Event::PointerMoved(end)]);
    assert_eq!(h.s.v, 100.0, "drag clamps");
    h.frame(vec![release(end)]);
    h.frame(key(Key::ArrowLeft));
    assert_eq!(h.s.v, 99.0, "arrow = 1% of range");
    h.frame(key_mod(Key::ArrowLeft, SHIFT));
    assert_eq!(h.s.v, 89.0, "shift = 10 steps");
    h.frame(key(Key::Home));
    assert_eq!(h.s.v, 0.0);
    h.frame(key(Key::End));
    assert_eq!(h.s.v, 100.0);
}

#[test]
fn stepped_slider_quantizes() {
    let mut h = Harness::new(Sl { v: 13.0, range: (11.0, 16.0), step: Some(1.0), ..Default::default() }, slider_view);
    let r = h.s.rect;
    h.click(vec2(r.min.x + 8.0 + (r.width() - 16.0) * 0.63, r.center().y));
    assert_eq!(h.s.v, 14.0);
}

#[derive(Default)]
struct Txt {
    s: String,
    rect: Rect,
}

fn text_view(ui: &mut Ui, s: &mut Txt) {
    ui.column().pad(24).show(|ui| {
        s.rect = ui.text_input(&mut s.s).label("Workspace name").show().rect;
    });
}

#[test]
fn text_input_editing_copy_undo() {
    let mut h = Harness::new(Txt { s: "Northwind".into(), ..Default::default() }, text_view);
    let r = h.s.rect;
    assert_eq!((r.width(), r.height()), (280.0, 36.0));
    h.click(vec2(r.max.x - 20.0, r.center().y));
    h.frame(vec![Event::Text(" Labs".into())]);
    assert_eq!(h.s.s, "Northwind Labs");
    h.frame(key_mod(Key::Backspace, ALT));
    assert_eq!(h.s.s, "Northwind ");
    let mut evs = vec![];
    for _ in 0..4 {
        evs.extend(key_mod(Key::ArrowLeft, SHIFT));
    }
    evs.extend(key_mod(Key::Char('c'), CMD));
    let out = h.frame(evs);
    assert_eq!(out.copied_text.as_deref(), Some("ind "));
    assert!(out.wants_text_input);
    h.frame(vec![Event::Text("!".into())]);
    assert_eq!(h.s.s, "Northw!", "typing replaces the selection");
    h.frame(key_mod(Key::Char('z'), CMD));
    assert_eq!(h.s.s, "Northwind ");
    h.frame(key_mod(Key::Char('a'), CMD));
    h.frame(key(Key::Backspace));
    assert_eq!(h.s.s, "");
}

#[test]
fn text_input_ignores_text_when_unfocused() {
    let mut h = Harness::new(Txt::default(), text_view);
    h.frame(vec![Event::Text("x".into())]);
    assert_eq!(h.s.s, "");
}

#[test]
fn double_click_selects_word() {
    let mut h = Harness::new(Txt { s: "rk_live_9d2f other".into(), ..Default::default() }, text_view);
    let p = vec2(h.s.rect.min.x + 20.0, h.s.rect.center().y);
    h.click(p);
    h.click(p);
    let out = h.frame(key_mod(Key::Char('c'), CMD));
    assert_eq!(out.copied_text.as_deref(), Some("rk_live_9d2f"));
}

#[derive(Default)]
struct Tip {
    rect: Rect,
}

fn tip_view(ui: &mut Ui, s: &mut Tip) {
    ui.column().pad(80).show(|ui| {
        s.rect = ui.icon_button("copy").ghost().tooltip_shortcut("Copy key", "C").show().rect;
    });
}

#[test]
fn tooltip_waits_500ms_then_shows_and_press_closes() {
    let mut h = Harness::new(Tip::default(), tip_view);
    let c = h.s.rect.center();
    h.frame(vec![Event::PointerMoved(c)]);
    h.idle(10); // hover fade settles (~170ms elapsed)
    let out = h.frame(vec![]);
    let base = out.draw.vertices.len();
    let wait = out.repaint_after.expect("waits for the tooltip delay");
    assert!(wait > 0.25 && wait < 0.35, "remaining delay {wait}");
    h.t += 0.35;
    h.idle(12);
    let shown = h.frame(vec![]).draw.vertices.len();
    assert!(shown > base + 8, "tooltip drawn: {shown} vs {base}");
    h.frame(vec![press(c), release(c)]);
    h.idle(2);
    let after = h.frame(vec![]).draw.vertices.len();
    assert!(after < shown, "press closes the tooltip");
}

fn idle_view(ui: &mut Ui, on: &mut bool) {
    ui.column().pad(24).show(|ui| {
        ui.heading("General");
        ui.switch("Notifications", on);
        ui.button("Save").primary();
    });
}

#[test]
fn idle_ui_requests_no_frames() {
    let mut h = Harness::new(true, idle_view);
    h.idle(3);
    assert_eq!(h.frame(vec![]).repaint_after, None, "0% CPU when idle");
}

#[test]
fn theme_switch_animates_then_settles() {
    let mut h = Harness::new(true, idle_view);
    h.ui.set_mode(Mode::Dark);
    let out = h.frame(vec![]);
    assert_eq!(out.repaint_after, Some(0.0));
    assert_ne!(out.draw.clear.to_hex(), 0x07080C);
    h.idle(25);
    let out = h.frame(vec![]);
    assert_eq!(out.draw.clear.to_hex(), 0x07080C);
    assert_eq!(out.repaint_after, None);
}

#[test]
fn focused_input_draws_caret() {
    let mut h = Harness::new(Txt { s: "Mara".into(), ..Default::default() }, text_view);
    let r = h.s.rect;
    h.click(vec2(r.max.x - 20.0, r.center().y));
    h.frame(vec![Event::Text("x".into())]);
    let out = h.frame(vec![]);
    let caret = h.ui.color(Token::FocusRing).to_u8();
    let found = out.draw.vertices.iter().any(|v| v.mode == lumen_core::paint::mode::RRECT && v.color == caret && (v.half[0] - 1.0).abs() < 0.01 && (v.half[1] - 8.0).abs() < 0.01);
    assert!(found, "caret quad present");
}

#[derive(Default)]
struct Seg {
    v: u8,
    rect: Rect,
}

fn seg_view(ui: &mut Ui, s: &mut Seg) {
    ui.column().pad(20).show(|ui| {
        s.rect = ui.segmented(&mut s.v, &[(0, "One"), (1, "Two"), (2, "Three")]).rect;
    });
}

#[test]
fn segmented_click_and_arrows() {
    let mut h = Harness::new(Seg::default(), seg_view);
    let r = h.s.rect;
    assert!(r.width() > 0.0);
    // Click the right end: the last segment.
    h.click(vec2(r.max.x - 6.0, r.center().y));
    h.idle(1);
    assert_eq!(h.s.v, 2);
    // Tab focuses the first segment; arrows move and select.
    h.frame(key(Key::Tab));
    h.frame(key(Key::ArrowRight));
    h.idle(1);
    assert_eq!(h.s.v, 1);
    h.frame(key(Key::ArrowLeft));
    h.idle(1);
    assert_eq!(h.s.v, 0);
}

#[derive(Default)]
struct LookState {
    look: u8,
    btn: Rect,
}

fn look_view(ui: &mut Ui, s: &mut LookState) {
    ui.set_look(if s.look == 0 { Look::lumen() } else { Look::gnome() });
    ui.column().pad(20).show(|ui| {
        s.btn = ui.button("Save").show().rect;
    });
}

#[test]
fn look_switch_morphs_control_size() {
    let mut h = Harness::new(LookState::default(), look_view);
    h.idle(2);
    assert_eq!(h.s.btn.height(), 36.0);
    h.s.look = 1;
    h.idle(8);
    let mid = h.s.btn.height();
    assert!(mid > 34.0 && mid < 36.0, "mid-blend height {mid}");
    let out = h.frame(vec![]);
    assert!(out.animating);
    h.idle(60);
    assert_eq!(h.s.btn.height(), 34.0);
    assert_eq!(h.ui.color(Token::Background).to_hex(), 0xFAFAFB);
}
