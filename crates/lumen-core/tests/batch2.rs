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

// ----------------------------------------------------------------- Overlays

#[derive(Default)]
struct SheetState {
    open: bool,
    name: String,
    trigger_focused: bool,
}

fn sheet_view(ui: &mut Ui, s: &mut SheetState) {
    ui.column().pad(24).show(|ui| {
        let r = ui.button("Open sheet").show();
        if r.clicked {
            s.open = true;
        }
        s.trigger_focused = r.focused;
        ui.sheet("inspector", &mut s.open).title("Inspector").show(|ui| {
            ui.text_input(&mut s.name).label("Name").fill_width();
        });
    });
}

#[test]
fn sheet_opens_focuses_body_and_escape_restores_trigger_focus() {
    let mut h = Harness::new(SheetState::default(), sheet_view);
    h.frame(key(Key::Tab));
    h.frame(key(Key::Enter));
    h.idle(3);
    assert!(h.s.open);
    h.frame(vec![Event::Text("arm64".into())]);
    assert_eq!(h.s.name, "arm64", "sheet focuses its first control on open");
    h.frame(key(Key::Escape));
    h.idle(2);
    assert!(!h.s.open);
    assert!(h.s.trigger_focused, "escape returns focus to trigger");
}

#[derive(Default)]
struct PalState {
    open: bool,
    picked: Option<&'static str>,
}

fn pal_view(ui: &mut Ui, s: &mut PalState) {
    const CMDS: &[Command<'static>] = &[
        Command::new("deploy", "Deploy production").group("Actions").shortcut("⌘D"),
        Command::new("logs", "Tail live logs").group("Actions").keywords("console"),
        Command::new("settings", "Workspace settings").group("Navigation"),
    ];
    ui.column().pad(24).show(|ui| {
        ui.button("Trigger");
        if let Some(id) = ui.command_palette("main", &mut s.open, CMDS).show() {
            s.picked = Some(id);
        }
    });
}

#[test]
fn command_palette_cmd_k_filters_and_runs() {
    let mut h = Harness::new(PalState::default(), pal_view);
    let cmd = Modifiers { logo: true, mac: true, ..Default::default() };
    h.frame(vec![Event::Key { key: Key::Char('k'), pressed: true, repeat: false, modifiers: cmd }]);
    h.idle(2);
    assert!(h.s.open, "⌘K opens the palette");
    // Filter by keyword ("console" -> Tail live logs) and press Enter.
    h.frame(vec![Event::Text("console".into())]);
    h.idle(1);
    h.frame(key(Key::Enter));
    h.idle(1);
    assert_eq!(h.s.picked, Some("logs"));
    assert!(!h.s.open, "running a command closes the palette");
}

#[derive(Default)]
struct HoverState {
    anchor: Rect,
    shown: bool,
}

fn hover_view(ui: &mut Ui, s: &mut HoverState) {
    ui.column().pad(40).show(|ui| {
        let r = ui.button("@jonas").ghost().show();
        s.anchor = r.rect;
        s.shown = ui.hover_card(&r).show(|ui| {
            ui.label("Jonas Berg");
        }).is_some();
    });
}

#[test]
fn hover_card_delays_bridges_gap_and_dismisses_on_escape() {
    let mut h = Harness::new(HoverState::default(), hover_view);
    let c = h.s.anchor.center();
    h.frame(vec![Event::PointerMoved(c)]);
    assert!(!h.s.shown, "waits for delay");
    h.t += 0.35;
    h.idle(2);
    assert!(h.s.shown, "opens after delay");
    // Move into the card below the anchor: stays open.
    h.frame(vec![Event::PointerMoved(vec2(c.x, h.s.anchor.max.y + 20.0))]);
    h.t += 0.30;
    h.idle(2);
    assert!(h.s.shown, "stays open while pointer is inside the card");
    h.frame(key(Key::Escape));
    h.idle(1);
    assert!(!h.s.shown, "escape dismisses");
}

#[derive(Default)]
struct CtxState {
    zone: Rect,
    copied: u32,
}

fn ctx_view(ui: &mut Ui, s: &mut CtxState) {
    let z = ui.column().pad(24).id("target").show(|ui| {
        ui.label("Release v2.4.1");
    });
    s.zone = z.rect;
    ui.context_menu(&z.response(), |ui| {
        if ui.menu_item("copy", "Copy SHA").clicked() {
            s.copied += 1;
        }
    });
}

#[test]
fn context_menu_opens_on_secondary_click_and_activates_item() {
    let mut h = Harness::new(CtxState::default(), ctx_view);
    let p = h.s.zone.center();
    h.frame(vec![
        Event::PointerMoved(p),
        Event::PointerButton { pos: p, button: PointerButton::Secondary, pressed: true },
        Event::PointerButton { pos: p, button: PointerButton::Secondary, pressed: false },
    ]);
    h.idle(3);
    // Click the first item just below `p`.
    h.click(vec2(p.x + 30.0, p.y + 20.0));
    h.idle(1);
    assert_eq!(h.s.copied, 1);
}

#[derive(Default)]
struct ConfirmState {
    btn: Rect,
    confirmed: u32,
}

fn confirm_view(ui: &mut Ui, s: &mut ConfirmState) {
    ui.column().pad(24).show(|ui| {
        let r = ui.button("Revoke").danger().show();
        s.btn = r.rect;
        if ui.confirm_popover(&r, "Revoke key?").confirm_label("Revoke").confirmed() {
            s.confirmed += 1;
        }
    });
}

#[test]
fn confirm_popover_cancels_on_escape_and_confirms_via_keyboard() {
    let mut h = Harness::new(ConfirmState::default(), confirm_view);
    // Open via keyboard: focus lands on Cancel. Escape cancels.
    h.frame(key(Key::Tab));
    h.frame(key(Key::Enter));
    h.idle(2);
    h.frame(key(Key::Escape));
    h.idle(2);
    assert_eq!(h.s.confirmed, 0);

    // Reopen, Tab from Cancel to Confirm, press Enter -> confirmed!
    h.frame(key(Key::Enter));
    h.idle(2);
    h.frame(key(Key::Tab));
    h.frame(key(Key::Enter));
    h.idle(2);
    assert_eq!(h.s.confirmed, 1);
}
