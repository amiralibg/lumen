//! Signature pad, ruler slider, time dial, aspect-ratio picker and avatar
//! upload, driven through a headless `Ui`. Controls are found through the
//! accessibility tree, which also checks that each one describes itself.

use lumen_core::*;

struct H<S> {
    ui: Ui,
    t: f64,
    s: S,
    view: fn(&mut Ui, &mut S),
    tree: Option<A11yTree>,
}

impl<S> H<S> {
    fn new(s: S, view: fn(&mut Ui, &mut S)) -> Self {
        let mut ui = Ui::new();
        ui.set_accessibility(true);
        let mut h = H { ui, t: 1.0, s, view, tree: None };
        h.idle(3);
        h
    }
    fn frame(&mut self, events: Vec<Event>) {
        self.t += 1.0 / 60.0;
        let input = RawInput { screen_size: vec2(800.0, 1400.0), scale: 1.0, time: self.t, events, modifiers: Modifiers::default() };
        let (s, view) = (&mut self.s, self.view);
        let out = self.ui.run(input, |ui| view(ui, s));
        self.tree = out.a11y;
    }
    fn idle(&mut self, n: usize) {
        for _ in 0..n {
            self.frame(vec![]);
        }
    }
    fn node(&self, role: Role, label: &str) -> A11yNode {
        let t = self.tree.as_ref().unwrap();
        t.find(role, label).cloned().unwrap_or_else(|| panic!("no {role:?} {label:?}\n{}", t.dump()))
    }
    fn click(&mut self, p: Vec2) {
        self.frame(vec![Event::PointerMoved(p), press(p)]);
        self.frame(vec![release(p)]);
    }
    fn drag(&mut self, from: Vec2, to: Vec2, steps: usize) {
        self.frame(vec![Event::PointerMoved(from), press(from)]);
        for k in 1..=steps {
            let p = from + (to - from) * (k as f32 / steps as f32);
            self.frame(vec![Event::PointerMoved(p)]);
        }
        self.frame(vec![release(to)]);
    }
    fn keys(&mut self, k: Key, m: Modifiers) {
        self.frame(vec![Event::Key { key: k, pressed: true, repeat: false, modifiers: m }, Event::Key { key: k, pressed: false, repeat: false, modifiers: m }]);
    }
    fn act(&mut self, role: Role, label: &str, action: A11yAction) {
        let id = self.node(role, label).id;
        self.frame(vec![Event::A11y { target: Id(id), action }]);
        self.idle(1);
    }
}

fn press(p: Vec2) -> Event {
    Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true }
}
fn release(p: Vec2) -> Event {
    Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false }
}
const NONE: Modifiers = Modifiers { shift: false, ctrl: false, alt: false, logo: false, mac: true };
const CMD: Modifiers = Modifiers { shift: false, ctrl: false, alt: false, logo: true, mac: true };
const SHIFT: Modifiers = Modifiers { shift: true, ctrl: false, alt: false, logo: false, mac: true };

// ------------------------------------------------------------ signature pad

#[derive(Default)]
struct Sig {
    sig: Signature,
    changes: u32,
}
fn sig_view(ui: &mut Ui, s: &mut Sig) {
    ui.column().pad(24).fill_width().show(|ui| {
        s.changes += ui.signature_pad("Signature", &mut s.sig).show().changed as u32;
    });
}

#[test]
fn signature_draws_strokes_and_undoes_them() {
    let mut h = H::new(Sig::default(), sig_view);
    let pad = h.node(Role::Canvas, "Signature");
    assert_eq!(pad.info.value.as_deref(), Some("Empty"));
    let r = pad.rect;
    h.drag(r.min + vec2(40.0, 60.0), r.min + vec2(240.0, 90.0), 12);
    assert_eq!(h.s.sig.strokes.len(), 1);
    assert!(h.s.sig.strokes[0].len() > 6, "sampled along the drag");
    assert!(h.s.sig.strokes[0].iter().all(|p| (0.0..=1.0).contains(&p.x) && (0.0..=1.0).contains(&p.y)), "pad units");
    assert_eq!(h.s.changes, 1, "a lifted stroke is a change");
    h.drag(r.min + vec2(60.0, 40.0), r.min + vec2(120.0, 120.0), 6);
    h.idle(1);
    assert_eq!(h.node(Role::Canvas, "Signature").info.value.as_deref(), Some("Signed, 2 strokes"));

    // Backspace undoes a stroke, Cmd+Backspace clears.
    h.click(r.min + vec2(5.0, 5.0));
    let n = h.s.sig.strokes.len();
    h.s.sig.strokes.truncate(n - 1);
    h.keys(Key::Backspace, NONE);
    assert_eq!(h.s.sig.strokes.len(), 1);
    h.drag(r.min + vec2(80.0, 50.0), r.min + vec2(200.0, 70.0), 6);
    h.keys(Key::Backspace, CMD);
    assert!(h.s.sig.is_empty());
    h.idle(1);
    assert_eq!(h.node(Role::Canvas, "Signature").info.value.as_deref(), Some("Empty"));
}

// ------------------------------------------------------------ ruler slider

struct Ruler {
    v: f32,
}
fn ruler_view(ui: &mut Ui, s: &mut Ruler) {
    ui.column().pad(24).width(400.0).show(|ui| {
        ui.ruler_slider("Weight", &mut s.v, 30.0..=200.0).step(0.5).major(10).unit("kg").show();
    });
}

#[test]
fn ruler_drags_snaps_and_steps() {
    let mut h = H::new(Ruler { v: 70.0 }, ruler_view);
    let n = h.node(Role::Slider, "Weight");
    assert_eq!(n.info.numeric.map(|m| m.value), Some(70.0));
    let c = n.rect.center();
    // Dragging the ruler left moves to higher values 1:1 (held still before
    // letting go, so there's no throw), onto a tick.
    h.frame(vec![Event::PointerMoved(c), press(c)]);
    for k in 1..=10 {
        h.frame(vec![Event::PointerMoved(c - vec2(8.0 * k as f32, 0.0))]);
    }
    h.idle(12);
    h.frame(vec![release(c - vec2(80.0, 0.0))]);
    h.idle(30);
    let held = h.s.v;
    assert!(held > 72.0 && held < 76.0, "{held}");
    assert_eq!((held * 2.0).fract(), 0.0, "snapped to the 0.5 step");
    // The same drag let go while moving throws further.
    h.drag(c, c - vec2(80.0, 0.0), 10);
    h.idle(30);
    assert!(h.s.v > held + 6.0, "thrown: {} after {held}", h.s.v);

    let before = h.s.v;
    h.click(c);
    h.keys(Key::ArrowRight, NONE);
    assert_eq!(h.s.v, before + 0.5);
    h.keys(Key::ArrowLeft, SHIFT);
    assert_eq!(h.s.v, before + 0.5 - 5.0, "Shift steps a major tick");
    h.keys(Key::End, NONE);
    assert_eq!(h.s.v, 200.0);
    h.act(Role::Slider, "Weight", A11yAction::Decrement);
    assert_eq!(h.s.v, 199.5);
}

// ------------------------------------------------------------ time dial

struct Dial {
    t: Time,
}
fn dial_view(ui: &mut Ui, s: &mut Dial) {
    ui.column().pad(24).show(|ui| {
        ui.time_dial("alarm", &mut s.t).step(5).show();
    });
}

#[test]
fn dial_picks_hour_then_minutes() {
    let mut h = H::new(Dial { t: Time::new(7, 30) }, dial_view);
    let dial = h.node(Role::Slider, "Hour").rect;
    let c = dial.center();
    let r = dial.width() * 0.35;
    // Tap at 3 o'clock: hour 3 (still AM), and the face turns to minutes.
    h.click(c + vec2(r, 0.0));
    assert_eq!(h.s.t, Time::new(3, 30));
    let dial = h.node(Role::Slider, "Minute").rect;
    // Tap at 6 o'clock: 30 minutes; at 9 o'clock: 45.
    h.click(dial.center() + vec2(-r, 0.0));
    assert_eq!(h.s.t, Time::new(3, 45));
    // PM moves the hour into the afternoon.
    let pm = h.node(Role::RadioButton, "PM").rect.center();
    h.click(pm);
    assert_eq!(h.s.t, Time::new(15, 45));
    // Keys step by the minute step in minute mode.
    h.click(h.node(Role::Slider, "Minute").rect.center() + vec2(0.0, -r));
    assert_eq!(h.s.t.minute, 0);
    h.keys(Key::ArrowUp, NONE);
    assert_eq!(h.s.t.minute, 5);
    // The hour tile switches back.
    let tile = h.node(Role::Button, "Hour, 3").rect.center();
    h.click(tile);
    h.act(Role::Slider, "Hour", A11yAction::Increment);
    assert_eq!(h.s.t.hour, 16);
}

// ------------------------------------------------------------ aspect ratio

struct Aspect {
    sel: usize,
    portrait: bool,
}
fn aspect_view(ui: &mut Ui, s: &mut Aspect) {
    ui.column().pad(24).width(560.0).show(|ui| {
        ui.aspect_ratio_picker("crop", &mut s.sel, &mut s.portrait, &[("Free", 0.0), ("1:1", 1.0), ("3:2", 1.5), ("16:9", 16.0 / 9.0)]);
    });
}

#[test]
fn aspect_chips_select_and_flip() {
    let mut h = H::new(Aspect { sel: 0, portrait: false }, aspect_view);
    assert!(h.node(Role::RadioButton, "Free").info.selected == Some(true));
    h.click(h.node(Role::RadioButton, "3:2").rect.center());
    assert_eq!(h.s.sel, 2);
    h.keys(Key::ArrowRight, NONE);
    assert_eq!(h.s.sel, 3, "arrow keys move along the group");
    h.idle(1);
    let flip = h.node(Role::Button, "Portrait").rect.center();
    h.click(flip);
    assert!(h.s.portrait);
    h.idle(1);
    let img = h.tree.as_ref().unwrap().nodes.iter().find(|n| n.info.role == Role::Image).unwrap().info.label.clone();
    assert_eq!(img.as_deref(), Some("Crop preview, 16:9 portrait"));
}

// ------------------------------------------------------------ avatar upload

struct Avatar {
    state: UploadState,
    choose: u32,
    retry: u32,
    remove: u32,
}
fn avatar_view(ui: &mut Ui, s: &mut Avatar) {
    ui.column().pad(24).width(480.0).show(|ui| {
        let st = s.state.clone();
        let o = ui.avatar_upload("photo", &st).initials("AL").title("Profile photo").show();
        s.choose += o.choose as u32;
        s.retry += o.retry as u32;
        s.remove += o.remove as u32;
    });
}

#[test]
fn avatar_reports_actions_per_state() {
    let mut h = H::new(Avatar { state: UploadState::Empty, choose: 0, retry: 0, remove: 0 }, avatar_view);
    h.click(h.node(Role::Button, "Upload photo").rect.center());
    assert_eq!(h.s.choose, 1, "the empty slot opens the picker");
    h.s.state = UploadState::Uploading(0.5);
    h.idle(2);
    let p = h.node(Role::ProgressIndicator, "Photo");
    assert_eq!(p.info.value.as_deref(), Some("Uploading, 50%"));
    h.click(p.rect.center());
    assert_eq!(h.s.choose, 1, "the photo does nothing mid-upload");
    h.click(h.node(Role::Button, "Cancel").rect.center());
    assert_eq!(h.s.remove, 1);
    h.s.state = UploadState::Failed("Too large".into());
    h.idle(2);
    h.act(Role::Button, "Retry upload", A11yAction::Click);
    assert_eq!(h.s.retry, 1, "a screen reader press retries");
    h.s.state = UploadState::Done;
    h.idle(2);
    h.click(h.node(Role::Button, "Change photo").rect.center());
    assert_eq!(h.s.choose, 2);
}

// ------------------------------------------------------------ idle at rest

struct All {
    sig: Signature,
    v: f32,
    t: Time,
    sel: usize,
    portrait: bool,
    state: UploadState,
}
fn all_view(ui: &mut Ui, s: &mut All) {
    ui.column().pad(24).width(560.0).gap(16).show(|ui| {
        ui.signature_pad("Signature", &mut s.sig).show();
        ui.ruler_slider("Weight", &mut s.v, 30.0..=200.0).step(0.5).show();
        ui.time_dial("alarm", &mut s.t).step(5).show();
        ui.aspect_ratio_picker("crop", &mut s.sel, &mut s.portrait, &[("Free", 0.0), ("1:1", 1.0), ("3:2", 1.5)]);
        let st = s.state.clone();
        ui.avatar_upload("photo", &st).initials("AL").show();
    });
}

#[test]
fn settled_widgets_request_no_frames() {
    let mut h = H::new(All { sig: Signature::default(), v: 70.0, t: Time::new(7, 30), sel: 0, portrait: false, state: UploadState::Done }, all_view);
    // Use each one, then let everything settle.
    let pad = h.node(Role::Canvas, "Signature").rect;
    h.drag(pad.min + vec2(30.0, 40.0), pad.min + vec2(200.0, 80.0), 8);
    h.keys(Key::Backspace, NONE);
    let ruler = h.node(Role::Slider, "Weight").rect.center();
    h.drag(ruler, ruler - vec2(60.0, 0.0), 6);
    h.click(h.node(Role::Slider, "Hour").rect.center() + vec2(40.0, 0.0));
    h.click(h.node(Role::RadioButton, "3:2").rect.center());
    h.s.state = UploadState::Uploading(0.4);
    h.idle(10);
    h.s.state = UploadState::Done;
    h.idle(240);
    h.t += 1.0 / 60.0;
    let input = RawInput { screen_size: vec2(800.0, 1400.0), scale: 1.0, time: h.t, events: vec![], modifiers: Modifiers::default() };
    let (s, view) = (&mut h.s, h.view);
    let out = h.ui.run(input, |ui| view(ui, s));
    assert!(!out.animating, "still animating after settling");
    assert_eq!(out.repaint_after, None, "a settled widget asked for a frame");
}
