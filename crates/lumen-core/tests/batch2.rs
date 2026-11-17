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

// ------------------------------------------------------------------ Inputs

#[derive(Default)]
struct AreaState {
    text: String,
    rect: Rect,
}

fn area_view(ui: &mut Ui, s: &mut AreaState) {
    ui.column().pad(24).show(|ui| {
        s.rect = ui.text_area(&mut s.text).label("Notes").rows(2, 5).max_chars(24).width(280.0).show().rect;
    });
}

#[test]
fn text_area_inserts_newlines_grows_and_caps_max_chars() {
    let mut h = Harness::new(AreaState::default(), area_view);
    let initial_h = h.s.rect.height();
    h.frame(key(Key::Tab));
    h.frame(vec![Event::Text("line1".into())]);
    h.frame(key(Key::Enter));
    h.frame(vec![Event::Text("line2".into())]);
    h.frame(key(Key::Enter));
    h.frame(vec![Event::Text("line3".into())]);
    h.idle(30);
    assert!(h.s.text.contains('\n'));
    assert!(h.s.rect.height() > initial_h, "height grew from {initial_h} to {}", h.s.rect.height());

    // Cap at 24 characters.
    h.frame(vec![Event::Text("01234567890123456789".into())]);
    assert_eq!(h.s.text.chars().count(), 24);
}

struct NumState {
    val: f64,
    rect: Rect,
}

fn num_view(ui: &mut Ui, s: &mut NumState) {
    ui.column().pad(24).show(|ui| {
        s.rect = ui.number_field(&mut s.val).label("Timeout").range(0.0..=1000.0).step(10.0).suffix("ms").show().rect;
        ui.button("After");
    });
}

#[test]
fn number_field_scrubs_steps_and_parses_inline_text() {
    let mut h = Harness::new(NumState { val: 100.0, rect: Rect::ZERO }, num_view);
    // Drag horizontally on the left scrub handle (20px = 5 steps * 10 = +50 -> 150).
    let scrub_p = vec2(h.s.rect.min.x + 14.0, h.s.rect.center().y);
    h.frame(vec![Event::PointerMoved(scrub_p), press(scrub_p)]);
    h.frame(vec![Event::PointerMoved(vec2(scrub_p.x + 20.0, scrub_p.y))]);
    h.frame(vec![release(vec2(scrub_p.x + 20.0, scrub_p.y))]);
    h.idle(1);
    assert_eq!(h.s.val, 150.0, "horizontal scrub updated value");

    // Focus via Tab: text is selected, ArrowUp steps +10 -> 160.
    h.frame(key(Key::Tab));
    h.frame(key(Key::ArrowUp));
    assert_eq!(h.s.val, 160.0);

    // Select all, type "2050" (clamped to 1000) and press Enter.
    let cmd_a = Modifiers { logo: true, mac: true, ..Default::default() };
    h.frame(vec![Event::Key { key: Key::Char('a'), pressed: true, repeat: false, modifiers: cmd_a }]);
    h.frame(vec![Event::Text("2050".into())]);
    h.frame(key(Key::Enter));
    assert_eq!(h.s.val, 1000.0);
}

#[derive(Default)]
struct TagState {
    tags: Vec<String>,
}

fn tag_view(ui: &mut Ui, s: &mut TagState) {
    ui.column().pad(24).show(|ui| {
        ui.tag_input(&mut s.tags).label("Tags").lowercase(true).max_tags(4).width(300.0);
    });
}

#[test]
fn tag_input_commits_on_enter_and_comma_and_deduplicates() {
    let mut h = Harness::new(TagState::default(), tag_view);
    h.frame(key(Key::Tab));
    // Comma-separated burst commits first two and leaves empty draft.
    h.frame(vec![Event::Text("Rust, WGPU,".into())]);
    h.idle(1);
    assert_eq!(h.s.tags, vec!["rust", "wgpu"]);

    // Duplicate "RUST" + Enter is ignored; new "wasm" + Enter is added.
    h.frame(vec![Event::Text("RUST".into())]);
    h.frame(key(Key::Enter));
    assert_eq!(h.s.tags, vec!["rust", "wgpu"]);
    h.frame(vec![Event::Text("wasm".into())]);
    h.frame(key(Key::Enter));
    assert_eq!(h.s.tags, vec!["rust", "wgpu", "wasm"]);

    // Two Backspaces on empty draft remove "wasm".
    h.frame(key(Key::Backspace));
    h.frame(key(Key::Backspace));
    assert_eq!(h.s.tags, vec!["rust", "wgpu"]);
}

#[derive(Default)]
struct OtpState {
    code: String,
    done: u32,
}

fn otp_view(ui: &mut Ui, s: &mut OtpState) {
    ui.column().pad(24).show(|ui| {
        if ui.otp_input("2fa", &mut s.code).completed() {
            s.done += 1;
        }
    });
}

#[test]
fn otp_input_auto_advances_pastes_with_hyphens_and_backspaces() {
    let mut h = Harness::new(OtpState::default(), otp_view);
    h.frame(key(Key::Tab));
    // Paste with hyphen fills all 6 cells and triggers completed().
    h.frame(vec![Event::Paste("482-913".into())]);
    assert_eq!(h.s.code, "482913");
    assert_eq!(h.s.done, 1);

    // Backspace clears the last digit; typing '7' completes again.
    h.frame(key(Key::Backspace));
    assert_eq!(h.s.code, "48291");
    h.frame(vec![Event::Text("7".into())]);
    assert_eq!(h.s.code, "482917");
    assert_eq!(h.s.done, 2);
}

#[derive(Default)]
struct PwState {
    pw: String,
}

fn pw_view(ui: &mut Ui, s: &mut PwState) {
    ui.column().pad(24).show(|ui| {
        ui.password_input(&mut s.pw).label("Password").strength(true).copyable(true);
    });
}

#[test]
fn password_strength_scores_and_reveal_toggle_works() {
    assert_eq!(password_strength(""), 0);
    assert_eq!(password_strength("password123"), 1);
    assert_eq!(password_strength("abcdefghij"), 2);
    assert_eq!(password_strength("Abcdefgh12"), 3);
    assert_eq!(password_strength("Correct-Horse9!"), 4);

    let mut h = Harness::new(PwState::default(), pw_view);
    h.frame(key(Key::Tab));
    h.frame(vec![Event::Text("S3cret-Phrase!".into())]);
    assert_eq!(h.s.pw, "S3cret-Phrase!");
    // While masked, Cmd+C must not copy the password.
    let cmd_c = Modifiers { logo: true, mac: true, ..Default::default() };
    let out = h.frame(vec![Event::Key { key: Key::Char('c'), pressed: true, repeat: false, modifiers: cmd_c }]);
    assert_eq!(out.copied_text, None);
}

#[derive(Default)]
struct SearchState {
    q: String,
}

fn search_view(ui: &mut Ui, s: &mut SearchState) {
    ui.column().pad(24).show(|ui| {
        ui.search_field(&mut s.q).shortcut("/").focus_key('/');
    });
}

#[test]
fn search_field_focuses_on_slash_and_clears_on_escape() {
    let mut h = Harness::new(SearchState::default(), search_view);
    // Pressing '/' outside text inputs focuses the search field.
    h.frame(key(Key::Char('/')));
    h.idle(1);
    h.frame(vec![Event::Text("canary".into())]);
    assert_eq!(h.s.q, "canary");
    // First Escape clears the query while keeping focus; typing works right away.
    h.frame(key(Key::Escape));
    assert_eq!(h.s.q, "");
    h.frame(vec![Event::Text("prod".into())]);
    assert_eq!(h.s.q, "prod");
}

// -------------------------------------------------- Feedback and Navigation

struct AlertState {
    open: bool,
    retried: u32,
}

fn alert_view(ui: &mut Ui, s: &mut AlertState) {
    ui.column().pad(24).gap(10).show(|ui| {
        if ui.alert("Quota warning").warning().action("Upgrade").dismissible(&mut s.open).clicked() {
            s.retried += 1;
        }
    });
}

#[test]
fn alert_triggers_action_and_dismisses_cleanly() {
    let mut h = Harness::new(AlertState { open: true, retried: 0 }, alert_view);
    // First Tab lands on the Upgrade action button; Enter triggers it.
    h.frame(key(Key::Tab));
    h.frame(key(Key::Enter));
    assert_eq!(h.s.retried, 1);
    assert!(h.s.open);

    // Second Tab lands on the dismiss × button; Enter closes the alert.
    h.frame(key(Key::Tab));
    h.frame(key(Key::Enter));
    assert!(!h.s.open);
}

#[test]
fn skeleton_requests_no_frames_under_reduced_motion() {
    let mut ui = Ui::new();
    ui.set_reduced_motion(true);
    let mut t = 1.0;
    for _ in 0..5 {
        t += 1.0 / 60.0;
        let out = ui.run(RawInput { time: t, ..Default::default() }, |ui| {
            ui.skeleton_circle(36.0);
            ui.skeleton_lines(3);
        });
        assert_eq!(out.repaint_after, None, "reduced-motion skeleton must be idle");
    }
}

#[derive(Default)]
struct CrumbState {
    picked: Option<usize>,
}

fn crumb_view(ui: &mut Ui, s: &mut CrumbState) {
    const ITEMS: [&str; 5] = ["Root", "Org", "Team", "Repo", "main.rs"];
    ui.column().pad(24).show(|ui| {
        if let Some(i) = ui.breadcrumb("trail", &ITEMS).max_items(4).show() {
            s.picked = Some(i);
        }
    });
}

#[test]
fn breadcrumb_navigates_ancestors_and_overflow_menu() {
    let mut h = Harness::new(CrumbState::default(), crumb_view);
    // Tab focuses Root (index 0); Enter selects it.
    h.frame(key(Key::Tab));
    h.frame(key(Key::Enter));
    assert_eq!(h.s.picked, Some(0));

    // ArrowRight moves to ellipsis button; Enter opens collapsed menu ("Org", "Team"); Enter picks "Org" (index 1).
    h.frame(key(Key::ArrowRight));
    h.idle(1);
    h.frame(key(Key::Enter));
    h.idle(2);
    h.frame(key(Key::Enter));
    h.idle(1);
    assert_eq!(h.s.picked, Some(1));
}

#[derive(Default)]
struct PageState {
    page: usize,
}

fn page_view(ui: &mut Ui, s: &mut PageState) {
    ui.column().pad(24).show(|ui| {
        ui.pagination("p", &mut s.page, 12);
    });
}

#[test]
fn pagination_steps_and_jumps_with_keyboard() {
    let mut h = Harness::new(PageState { page: 0 }, page_view);
    // At page 0, prev is disabled so first Tab focuses page 1 button (index 0).
    h.frame(key(Key::Tab));
    h.frame(key(Key::ArrowRight));
    assert_eq!(h.s.page, 1);
    h.frame(key(Key::End));
    assert_eq!(h.s.page, 11);
    h.frame(key(Key::Home));
    assert_eq!(h.s.page, 0);
}

#[derive(Default)]
struct WizState {
    step: usize,
}

fn wiz_view(ui: &mut Ui, s: &mut WizState) {
    const STEPS: [(&str, &str); 3] = [("Build", ""), ("Test", ""), ("Ship", "")];
    ui.column().pad(24).show(|ui| {
        ui.steps("w", &mut s.step, &STEPS).linear(true);
    });
}

#[test]
fn linear_steps_unlock_as_progress_advances() {
    let mut h = Harness::new(WizState { step: 1 }, wiz_view);
    // Step 0 and Step 1 are unlocked; Step 2 is locked until reached.
    h.frame(key(Key::Tab)); // focuses Step 0
    h.frame(key(Key::Enter));
    assert_eq!(h.s.step, 0, "can return to completed step 0");
    // Step 1 is still remembered in max_reached, so ArrowRight + Enter returns to Step 1.
    h.frame(key(Key::ArrowRight));
    h.idle(1);
    h.frame(key(Key::Enter));
    assert_eq!(h.s.step, 1);
}

// ----------------------------------------- Selection, Actions & Chrome (G5)

#[derive(Default)]
struct G5State {
    tier: u8,
    stars: u8,
    band: (f32, f32),
    chord: Vec<String>,
    purged: u32,
}

fn g5_view(ui: &mut Ui, s: &mut G5State) {
    ui.column().pad(24).gap(10).show(|ui| {
        ui.radio_card(&mut s.tier, 0, "Shared");
        ui.radio_card(&mut s.tier, 1, "Dedicated");
        ui.rating("r", &mut s.stars);
        ui.range_slider("Band", &mut s.band, 0.0..=100.0).step(10.0);
        ui.shortcut_recorder("rec", &mut s.chord);
        if ui.hold_button("Purge", 0.5).confirmed() {
            s.purged += 1;
        }
    });
}

#[test]
fn radio_cards_rating_range_slider_recorder_and_hold_button() {
    let mut h = Harness::new(G5State { tier: 0, stars: 3, band: (20.0, 80.0), chord: vec![], purged: 0 }, g5_view);
    // Tab 1 -> card 0; Tab 2 -> card 1, Enter selects tier 1.
    h.frame(key(Key::Tab));
    h.frame(key(Key::Tab));
    h.frame(key(Key::Enter));
    assert_eq!(h.s.tier, 1);

    // Tab 3 -> rating; ArrowRight steps 3 -> 4.
    h.frame(key(Key::Tab));
    h.frame(key(Key::ArrowRight));
    assert_eq!(h.s.stars, 4);

    // Tab 4 -> range_slider low thumb (20 -> 30); Tab 5 -> high thumb (80 -> 70).
    h.frame(key(Key::Tab));
    h.frame(key(Key::ArrowRight));
    h.frame(key(Key::Tab));
    h.frame(key(Key::ArrowLeft));
    assert_eq!(h.s.band, (30.0, 70.0));

    // Tab 6 -> shortcut recorder: Enter arms, Cmd+Shift+K records ["⇧", "⌘", "K"].
    h.frame(key(Key::Tab));
    h.frame(key(Key::Enter));
    let mods = Modifiers { logo: true, shift: true, mac: true, ..Default::default() };
    h.frame(vec![Event::Key { key: Key::Char('k'), pressed: true, repeat: false, modifiers: mods }]);
    assert_eq!(h.s.chord, vec!["⇧", "⌘", "K"]);

    // Tab 7 -> clear button of shortcut recorder, Tab 8? Note clear button has focusable: false!
    // So Tab 7 lands directly on hold_button! Hold Space for 0.6s -> fires once!
    h.frame(key(Key::Tab));
    h.frame(vec![Event::Key { key: Key::Space, pressed: true, repeat: false, modifiers: Modifiers::default() }]);
    h.t += 0.25;
    h.frame(vec![]);
    assert_eq!(h.s.purged, 0, "early hold has not fired yet");
    h.t += 0.35;
    h.frame(vec![]);
    assert_eq!(h.s.purged, 1, "holding past duration fires confirmed()");
    h.frame(vec![Event::Key { key: Key::Space, pressed: false, repeat: false, modifiers: Modifiers::default() }]);
}
