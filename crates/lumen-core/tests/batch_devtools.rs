//! Hex viewer, tree table, problems panel, stack trace, blame and coverage,
//! driven through a headless `Ui` and found through the accessibility tree.

use std::collections::BTreeSet;

use lumen_core::*;

struct H<S> {
    ui: Ui,
    t: f64,
    s: S,
    view: fn(&mut Ui, &mut S),
    tree: Option<A11yTree>,
    out: Option<FrameOutput>,
}

impl<S> H<S> {
    fn new(s: S, view: fn(&mut Ui, &mut S)) -> Self {
        let mut ui = Ui::new();
        ui.set_accessibility(true);
        let mut h = H { ui, t: 1.0, s, view, tree: None, out: None };
        h.idle(3);
        h
    }
    fn frame_m(&mut self, events: Vec<Event>, m: Modifiers) {
        self.t += 1.0 / 60.0;
        let input = RawInput { screen_size: vec2(900.0, 1200.0), scale: 1.0, time: self.t, events, modifiers: m };
        let (s, view) = (&mut self.s, self.view);
        let mut out = self.ui.run(input, |ui| view(ui, s));
        self.tree = out.a11y.take();
        self.out = Some(out);
    }
    fn frame(&mut self, events: Vec<Event>) {
        self.frame_m(events, Modifiers::default())
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
    fn has(&self, role: Role, label: &str) -> bool {
        self.tree.as_ref().unwrap().find(role, label).is_some()
    }
    fn click(&mut self, p: Vec2) {
        self.frame(vec![Event::PointerMoved(p), press(p)]);
        self.frame(vec![release(p)]);
    }
    fn key(&mut self, k: Key, m: Modifiers) {
        self.frame_m(vec![Event::Key { key: k, pressed: true, repeat: false, modifiers: m }, Event::Key { key: k, pressed: false, repeat: false, modifiers: m }], m);
    }
}

fn press(p: Vec2) -> Event {
    Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true }
}
fn release(p: Vec2) -> Event {
    Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false }
}
const NONE: Modifiers = Modifiers { shift: false, ctrl: false, alt: false, logo: false, mac: true };
const SHIFT: Modifiers = Modifiers { shift: true, ctrl: false, alt: false, logo: false, mac: true };

// ------------------------------------------------------------ hex viewer

struct Hex {
    bytes: Vec<u8>,
    out: HexOut,
}
fn hex_view(ui: &mut Ui, s: &mut Hex) {
    ui.column().pad(24).width(800.0).show(|ui| {
        s.out = ui.hex_viewer("data", &s.bytes).height(200.0).show();
    });
}

#[test]
fn hex_clicks_and_extends_with_shift() {
    let bytes: Vec<u8> = b"Hello, lumen!\n".iter().copied().cycle().take(100).collect();
    let mut h = H::new(Hex { bytes, out: HexOut::default() }, hex_view);
    let grid = h.node(Role::Grid, "data bytes");
    assert!(grid.info.value.as_deref().unwrap().starts_with("0x00000000 · 0x48 · 72 · 'H'"), "{:?}", grid.info.value);
    // Click the second row's first ASCII cell: byte 16.
    let r = grid.rect;
    let row_h = r.height() / (100f32 / 16.0).ceil();
    // Past the offset column, on the first hex cell of the row.
    h.click(vec2(r.min.x + 100.0, r.min.y + row_h * 1.5));
    let first = h.s.out.cursor;
    assert_eq!(first / 16, 1, "second row");
    h.key(Key::ArrowRight, SHIFT);
    h.key(Key::ArrowRight, SHIFT);
    assert_eq!(h.s.out.selection.len(), 3);
    assert_eq!(h.s.out.cursor, first + 2);
    h.key(Key::ArrowDown, NONE);
    assert_eq!(h.s.out.cursor, first + 18);
    assert_eq!(h.s.out.selection.len(), 1, "a plain move collapses the selection");
}

// ------------------------------------------------------------ tree table

struct Tt {
    open: BTreeSet<String>,
    sel: String,
}
fn tt_view(ui: &mut Ui, s: &mut Tt) {
    let rows = vec![TreeTableRow::new("src", &["src", "12 KB"]).children([TreeTableRow::new("main", &["main.rs", "4 KB"]), TreeTableRow::new("lib", &["lib.rs", "8 KB"])]), TreeTableRow::new("readme", &["README.md", "2 KB"])];
    ui.column().pad(24).width(500.0).show(|ui| {
        ui.tree_table("files", &[("Name", 0.0), ("Size", 80.0)], &rows, &mut s.open, &mut s.sel);
    });
}

#[test]
fn tree_table_expands_and_walks() {
    let mut h = H::new(Tt { open: BTreeSet::new(), sel: String::new() }, tt_view);
    assert!(!h.has(Role::TreeItem, "main.rs"), "collapsed at first");
    let src = h.node(Role::TreeItem, "src");
    assert_eq!(src.info.expanded, Some(false));
    assert_eq!(src.info.value.as_deref(), Some("Size 12 KB"));
    h.click(src.rect.center());
    h.idle(2);
    assert!(h.s.open.contains("src"));
    assert!(h.has(Role::TreeItem, "main.rs"));
    h.key(Key::ArrowDown, NONE);
    assert_eq!(h.s.sel, "main");
    h.key(Key::ArrowLeft, NONE);
    assert_eq!(h.s.sel, "src", "left goes to the parent");
    h.key(Key::ArrowLeft, NONE);
    assert!(!h.s.open.contains("src"), "then collapses it");
}

// ------------------------------------------------------------ problems

fn problems() -> Vec<Problem> {
    let p = |severity, file: &str, line, message: &str| Problem { severity, file: file.into(), line, col: 1, message: message.into(), source: "rustc".into() };
    vec![p(Severity::Error, "a.rs", 3, "boom"), p(Severity::Warning, "a.rs", 9, "careful"), p(Severity::Info, "b.rs", 1, "fyi")]
}
#[derive(Default)]
struct Pr {
    opened: Vec<usize>,
}
fn pr_view(ui: &mut Ui, s: &mut Pr) {
    let ps = problems();
    ui.column().pad(24).width(600.0).show(|ui| {
        if let Some(i) = ui.problems_panel("build", &ps) {
            s.opened.push(i);
        }
    });
}

#[test]
fn problems_filter_group_and_open() {
    let mut h = H::new(Pr::default(), pr_view);
    h.idle(30);
    assert!(h.has(Role::ListItem, "boom"));
    h.click(h.node(Role::Button, "Errors, 1").rect.center());
    h.idle(2);
    assert!(!h.has(Role::ListItem, "boom"), "errors filtered out");
    assert!(h.has(Role::ListItem, "careful"));
    h.click(h.node(Role::Button, "a.rs, 1 problems").rect.center());
    h.idle(2);
    assert!(!h.has(Role::ListItem, "careful"), "file collapsed");
    h.click(h.node(Role::ListItem, "fyi").rect.center());
    assert_eq!(h.s.opened, vec![2]);
    // Keyboard: Up from b.rs's only visible row stays, Enter opens it.
    h.key(Key::Enter, NONE);
    assert_eq!(h.s.opened, vec![2, 2]);
}

// ------------------------------------------------------------ stack trace

fn frames() -> Vec<StackFrame> {
    let f = |function: &str, in_app: bool| StackFrame { function: function.into(), file: "x.rs".into(), line: 2, in_app, context: if in_app { vec![(1, "fn a() {".into()), (2, "    boom()".into())] } else { vec![] } };
    vec![f("core::panic", false), f("core::fmt", false), f("app::run", true), f("app::main", true), f("std::rt", false)]
}
#[derive(Default)]
struct St {
    clicked: Vec<usize>,
}
fn st_view(ui: &mut Ui, s: &mut St) {
    let fr = frames();
    ui.column().pad(24).width(600.0).show(|ui| {
        if let Some(i) = ui.stack_trace("t", "Panic", "boom", &fr, "rust") {
            s.clicked.push(i);
        }
    });
}

#[test]
fn stack_folds_library_frames_and_opens_yours() {
    let mut h = H::new(St::default(), st_view);
    h.idle(30);
    assert!(!h.has(Role::Button, "core::panic at x.rs:2"), "library frames folded");
    let run = h.node(Role::Button, "app::run at x.rs:2");
    assert_eq!(run.info.expanded, Some(true), "the first app frame starts open");
    h.click(h.node(Role::Button, "2 library frames").rect.center());
    h.idle(2);
    assert!(h.has(Role::Button, "core::panic at x.rs:2"));
    h.click(h.node(Role::Button, "app::main at x.rs:2").rect.center());
    h.idle(2);
    assert_eq!(h.s.clicked, vec![3]);
    assert_eq!(h.node(Role::Button, "app::main at x.rs:2").info.expanded, Some(true));
    assert_eq!(h.node(Role::Button, "app::run at x.rs:2").info.expanded, Some(false), "one frame open at a time");
}

// ------------------------------------------------------------ blame

#[derive(Default)]
struct Bl {
    picked: Vec<usize>,
}
fn bl_view(ui: &mut Ui, s: &mut Bl) {
    let lines: Vec<BlameLine> = (0..40).map(|i| BlameLine { commit: if i < 20 { "aaa".into() } else { "bbb".into() }, author: if i < 20 { "Ada".into() } else { "Bo".into() }, age_days: if i < 20 { 300 } else { 2 }, summary: "change".into(), text: format!("let x{i} = {i};") }).collect();
    ui.column().pad(24).width(800.0).show(|ui| {
        if let Some(i) = ui.blame_view("f.rs", &lines).height(200.0).show() {
            s.picked.push(i);
        }
    });
}

#[test]
fn blame_picks_lines_by_pointer_and_keys() {
    let mut h = H::new(Bl::default(), bl_view);
    let g = h.node(Role::Grid, "f.rs blame");
    let row_h = g.rect.height() / 40.0;
    h.click(vec2(g.rect.center().x, g.rect.min.y + row_h * 2.5));
    assert_eq!(h.s.picked, vec![2]);
    for _ in 0..19 {
        h.key(Key::ArrowDown, NONE);
    }
    h.key(Key::Enter, NONE);
    assert_eq!(h.s.picked, vec![2, 21]);
    h.idle(2);
    assert_eq!(h.node(Role::Grid, "f.rs blame").info.value.as_deref(), Some("Line 22, Bo, 2d: change"));
}

// ------------------------------------------------------------ coverage

struct Cv;
fn cv_view(ui: &mut Ui, _: &mut Cv) {
    let lines: Vec<CoverLine> = (0..30)
        .map(|i| CoverLine { text: format!("step({i});"), cover: match i { 4 | 20 => Coverage::Miss, 5 => Coverage::Partial(3), 0 => Coverage::Skip, _ => Coverage::Hit(7) } })
        .collect();
    ui.column().pad(24).width(800.0).show(|ui| {
        ui.coverage_view("cov", "src/x.rs", &lines).height(200.0).show();
    });
}

#[test]
fn coverage_summarizes_and_jumps_to_misses() {
    let mut h = H::new(Cv, cv_view);
    h.idle(60);
    let t = h.tree.as_ref().unwrap();
    let meter = t.nodes.iter().find(|n| n.info.role == Role::Meter).unwrap();
    assert_eq!(meter.info.value.as_deref(), Some("27 of 29 lines"));
    h.click(h.node(Role::Button, "2 uncovered").rect.center());
    h.idle(2);
    assert_eq!(h.node(Role::Grid, "src/x.rs coverage").info.value.as_deref(), Some("Line 5: not covered"));
    // N from the code jumps on to the next miss.
    let g = h.node(Role::Grid, "src/x.rs coverage").rect;
    h.click(vec2(g.center().x, g.min.y + 5.0));
    h.key(Key::Char('n'), NONE);
    h.idle(2);
    assert_eq!(h.node(Role::Grid, "src/x.rs coverage").info.value.as_deref(), Some("Line 5: not covered"));
    h.key(Key::Char('n'), NONE);
    h.idle(2);
    assert_eq!(h.node(Role::Grid, "src/x.rs coverage").info.value.as_deref(), Some("Line 21: not covered"));
    let mut settled_at = None;
    for k in 0..600 {
        h.idle(1);
        if !h.out.as_ref().unwrap().animating {
            settled_at = Some(k);
            break;
        }
    }
    assert!(settled_at.is_some_and(|k| k < 180), "settles after the pulse: {settled_at:?}");
}
