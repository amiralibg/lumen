//! Headless interaction tests for Batch 2 widgets.

use lumen_core::*;

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
        let input = RawInput { screen_size: vec2(960.0, 640.0), scale: 1.0, time: self.t, events, modifiers: Modifiers::default() };
        let (s, view) = (&mut self.s, self.view);
        self.ui.run(input, |ui| view(ui, s))
    }

    fn idle(&mut self, n: usize) {
        for _ in 0..n {
            self.frame(vec![]);
        }
    }

    #[allow(dead_code)]
    fn click(&mut self, p: Vec2) {
        self.frame(vec![Event::PointerMoved(p), press(p)]);
        self.frame(vec![release(p)]);
    }
}

#[allow(dead_code)]
fn press(p: Vec2) -> Event {
    Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true }
}
#[allow(dead_code)]
fn release(p: Vec2) -> Event {
    Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false }
}
fn key(k: Key) -> Vec<Event> {
    vec![
        Event::Key { key: k, pressed: true, repeat: false, modifiers: Modifiers::default() },
        Event::Key { key: k, pressed: false, repeat: false, modifiers: Modifiers::default() },
    ]
}

// ------------------------------------------------------------------ Pickers

#[derive(Default)]
struct ComboState {
    v: Option<usize>,
    rect: Rect,
    changed_count: u32,
}

const CITIES: [(usize, &str); 5] = [(0, "Frankfurt"), (1, "Virginia"), (2, "Tokyo"), (3, "Sydney"), (4, "Singapore")];

fn combo_view(ui: &mut Ui, s: &mut ComboState) {
    ui.column().pad(24).show(|ui| {
        let r = ui.combobox(&mut s.v, &CITIES).label("Region").show();
        s.rect = r.rect;
        s.changed_count += r.changed as u32;
        ui.button("Other");
    });
}

#[test]
fn combobox_filters_selects_and_clears() {
    let mut h = Harness::new(ComboState { v: Some(0), ..Default::default() }, combo_view);
    // Focus combobox via Tab, type "sy" -> filters to Sydney, Enter picks it.
    h.frame(key(Key::Tab));
    h.idle(1);
    h.frame(vec![Event::Text("sy".into())]);
    h.idle(2);
    h.frame(key(Key::Enter));
    h.idle(1);
    assert_eq!(h.s.v, Some(3), "fuzzy filter + Enter picks Sydney");
    assert_eq!(h.s.changed_count, 1);

    // Type partial junk and Escape reverts to Sydney.
    h.frame(vec![Event::Text("zzz".into())]);
    h.idle(1);
    h.frame(key(Key::Escape));
    h.idle(1);
    assert_eq!(h.s.v, Some(3));

    // Select all, Backspace to empty, Tab out -> clears to None.
    let cmd_a = Modifiers { logo: true, mac: true, ..Default::default() };
    h.frame(vec![Event::Key { key: Key::Char('a'), pressed: true, repeat: false, modifiers: cmd_a }]);
    h.frame(key(Key::Backspace));
    h.frame(key(Key::Tab));
    h.idle(2);
    assert_eq!(h.s.v, None, "leaving an empty combobox clears value to None");
}

#[derive(Default)]
struct MultiState {
    vals: Vec<usize>,
    rect: Rect,
}

fn multi_view(ui: &mut Ui, s: &mut MultiState) {
    ui.column().pad(24).show(|ui| {
        s.rect = ui.multi_select(&mut s.vals, &CITIES).label("Regions").width(320.0).show().rect;
    });
}

#[test]
fn multi_select_toggles_chips_and_backspace_removes() {
    let mut h = Harness::new(MultiState { vals: vec![0], ..Default::default() }, multi_view);
    h.frame(key(Key::Tab));
    h.idle(1);
    // Type "tok" -> Tokyo is highlighted -> Enter adds it.
    h.frame(vec![Event::Text("tok".into())]);
    h.idle(2);
    h.frame(key(Key::Enter));
    h.idle(2);
    assert_eq!(h.s.vals, vec![0, 2]);

    // Query cleared automatically; two Backspaces arm then remove the last chip (Tokyo).
    h.frame(key(Key::Backspace));
    h.idle(1);
    assert_eq!(h.s.vals, vec![0, 2], "first backspace only arms the last chip");
    h.frame(key(Key::Backspace));
    h.idle(1);
    assert_eq!(h.s.vals, vec![0], "second backspace removes it");
}

#[derive(Default)]
struct RangeState {
    span: Option<(Date, Date)>,
    rect: Rect,
}

fn range_view(ui: &mut Ui, s: &mut RangeState) {
    ui.column().pad(24).show(|ui| {
        s.rect = ui.date_range_picker("Stay", &mut s.span).rect;
    });
}

#[test]
fn date_range_picker_keyboard_picks_ordered_span() {
    let start = Date::new(2026, 11, 15);
    let mut h = Harness::new(RangeState { span: Some((start, start)), ..Default::default() }, range_view);
    // Tab to field, Space opens and focuses the calendar grid.
    h.frame(key(Key::Tab));
    h.frame(key(Key::Space));
    h.idle(2);
    // Enter sets anchor at Nov 15; move left 3 days to Nov 12; Enter completes ordered range (Nov 12, Nov 15).
    h.frame(key(Key::Enter));
    for _ in 0..3 {
        h.frame(key(Key::ArrowLeft));
    }
    h.frame(key(Key::Enter));
    h.idle(2);
    assert_eq!(h.s.span, Some((Date::new(2026, 11, 12), Date::new(2026, 11, 15))));
}

struct TimeState {
    t: Time,
    rect: Rect,
}

fn time_view(ui: &mut Ui, s: &mut TimeState) {
    ui.column().pad(24).show(|ui| {
        s.rect = ui.time_picker("Start", &mut s.t).twelve_hour(true).step(15).show().rect;
    });
}

#[test]
fn time_picker_steps_columns_and_twelve_hour() {
    let mut h = Harness::new(TimeState { t: Time::new(9, 15), rect: Rect::ZERO }, time_view);
    h.frame(key(Key::Tab));
    h.frame(key(Key::Enter));
    h.idle(2);
    // Column 0 (hour): ArrowDown -> 10:15 AM
    h.frame(key(Key::ArrowDown));
    assert_eq!(h.s.t, Time::new(10, 15));
    // Move to column 1 (minute): ArrowDown steps by 15 -> 10:30 AM
    h.frame(key(Key::ArrowRight));
    h.frame(key(Key::ArrowDown));
    assert_eq!(h.s.t, Time::new(10, 30));
    // Move to column 2 (AM/PM): ArrowDown flips to PM -> 22:30
    h.frame(key(Key::ArrowRight));
    h.frame(key(Key::ArrowDown));
    assert_eq!(h.s.t, Time::new(22, 30));
    // Enter closes the popover.
    h.frame(key(Key::Enter));
    h.idle(2);
    assert_eq!(h.s.t.format_12(), "10:30 PM");
}

struct ColorState {
    c: Color,
    rect: Rect,
}

fn color_view(ui: &mut Ui, s: &mut ColorState) {
    ui.column().pad(24).show(|ui| {
        s.rect = ui.color_picker("Accent", &mut s.c).alpha(true).show().rect;
    });
}

#[test]
fn color_picker_keyboard_and_hex_editing() {
    let mut h = Harness::new(ColorState { c: Color::hex(0x0E6EE6), rect: Rect::ZERO }, color_view);
    // Open via keyboard: focus lands on the 2D SV canvas.
    h.frame(key(Key::Tab));
    h.frame(key(Key::Space));
    h.idle(3);
    let before = h.s.c.to_hex();
    h.frame(key(Key::ArrowLeft));
    h.frame(key(Key::ArrowDown));
    assert_ne!(h.s.c.to_hex(), before, "arrow keys on SV canvas update saturation/value");

    // Tab through Hue, Alpha, and 8 swatches into the Hex editor (1 + 1 + 8 + 1 = 11 Tabs).
    for _ in 0..11 {
        h.frame(key(Key::Tab));
    }
    let cmd_a = Modifiers { logo: true, mac: true, ..Default::default() };
    h.frame(vec![Event::Key { key: Key::Char('a'), pressed: true, repeat: false, modifiers: cmd_a }]);
    h.frame(vec![Event::Text("ff550080".into())]);
    h.idle(1);
    assert_eq!(h.s.c.to_hex(), 0xFF5500);
    assert!((h.s.c.a - 128.0 / 255.0).abs() < 0.01, "8-digit hex sets alpha: {}", h.s.c.a);
}
