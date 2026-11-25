//! Virtualized rows, list_view and table at 100k rows.

use std::collections::HashMap;

use lumen_core::*;

struct H<S> {
    ui: Ui,
    t: f64,
    s: S,
    view: fn(&mut Ui, &mut S),
}

impl<S> H<S> {
    fn new(s: S, view: fn(&mut Ui, &mut S)) -> Self {
        let mut h = H { ui: Ui::new(), t: 1.0, s, view };
        h.ui.theme.reduced_motion = true;
        h.idle(4);
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
    fn wheel(&mut self, dy: f32) {
        self.frame(vec![Event::PointerMoved(vec2(400.0, 300.0)), Event::Scroll(vec2(0.0, dy))]);
        self.idle(2);
    }
    fn key(&mut self, k: Key) {
        self.frame(vec![
            Event::Key { key: k, pressed: true, repeat: false, modifiers: Modifiers::default() },
            Event::Key { key: k, pressed: false, repeat: false, modifiers: Modifiers::default() },
        ]);
        self.idle(2);
    }
    fn click(&mut self, p: Vec2) {
        self.frame(vec![
            Event::PointerMoved(p),
            Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true },
            Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false },
        ]);
        self.idle(2);
    }
}

// ------------------------------------------------------------- virtual_rows

#[derive(Default)]
struct VS {
    /// Row keys (an id per row, as from a database).
    keys: Vec<u64>,
    out: VirtualOut,
    /// Last frame's rect of each built row, by key.
    rects: HashMap<u64, Rect>,
}

fn height_of(key: u64) -> f32 {
    24.0 + (key % 5) as f32 * 12.0
}

fn v_view(ui: &mut Ui, s: &mut VS) {
    s.rects.clear();
    let keys = s.keys.clone();
    let key_of = |i: usize| keys[i];
    ui.scroll("sc").show(|ui| {
        ui.label("Header above the rows");
        let rects = &mut s.rects;
        s.out = ui.virtual_rows("rows", keys.len()).keys(&key_of).estimate(40.0).show(|ui, i| {
            let k = keys[i];
            let shown = ui.column().id(("cell", k)).fill_width().min_height(height_of(k)).show(|ui| {
                ui.label(format!("Row {k}"));
            });
            rects.insert(k, shown.rect);
        });
    });
}

fn first_visible(s: &VS) -> (u64, f32) {
    s.rects.iter().filter(|(_, r)| r.max.y > 0.5 && r.height() > 0.0).map(|(k, r)| (*k, r.min.y)).min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0))).filter(|(_, y)| *y > -1000.0).map(|(k, _)| {
        // The topmost row whose bottom is on screen.
        let mut best = (k, f32::MIN);
        for (k2, r) in &s.rects {
            if r.max.y > 0.5 && (best.1 == f32::MIN || r.min.y < best.1) {
                best = (*k2, r.min.y);
            }
        }
        best
    }).unwrap()
}

#[test]
fn hundred_thousand_rows_build_only_whats_in_view() {
    let mut h = H::new(VS { keys: (0..100_000).collect(), ..Default::default() }, v_view);
    let out = h.s.out;
    assert!(out.built > 5 && out.built < 80, "built {} rows", out.built);
    assert!(out.total_height > 100_000.0 * 20.0, "total {}", out.total_height);

    // Scroll far: the built window moves with it.
    for _ in 0..20 {
        h.wheel(5000.0);
    }
    let out = h.s.out;
    assert!(out.first > 1000, "first {}", out.first);
    assert!(out.built < 80, "built {}", out.built);
    // Rows on screen are laid out back to back with their own heights.
    let mut on: Vec<(u64, Rect)> = h.s.rects.iter().filter(|(_, r)| r.max.y > 0.0 && r.min.y < 600.0).map(|(k, r)| (*k, *r)).collect();
    on.sort_by_key(|(k, _)| *k);
    assert!(on.len() > 5);
    for w in on.windows(2) {
        assert_eq!(w[1].0, w[0].0 + 1, "consecutive rows");
        assert!((w[1].1.min.y - w[0].1.max.y).abs() < 0.5, "no gaps or overlaps: {:?}", w);
        assert!((w[0].1.height() - height_of(w[0].0)).abs() < 0.5, "measured height");
    }
}

#[test]
fn inserting_above_keeps_the_visible_row_in_place() {
    let mut h = H::new(VS { keys: (1_000..101_000).collect(), ..Default::default() }, v_view);
    for _ in 0..4 {
        h.wheel(3000.0);
    }
    let (k, y) = first_visible(&h.s);
    // 50 new rows arrive at the top (a chat or log loading history).
    let mut keys: Vec<u64> = (0..50).collect();
    keys.extend(h.s.keys.iter().copied());
    h.s.keys = keys;
    h.idle(4);
    let y2 = h.s.rects.get(&k).map(|r| r.min.y);
    assert!(y2.is_some_and(|y2| (y2 - y).abs() < 1.0), "row {k} was at {y}, now {y2:?}");

    // And removing rows above too.
    h.s.keys.drain(0..120);
    h.idle(4);
    let y3 = h.s.rects.get(&k).map(|r| r.min.y);
    assert!(y3.is_some_and(|y3| (y3 - y).abs() < 1.0), "row {k} was at {y}, now {y3:?}");
}

// ---------------------------------------------------------------- list_view

struct LS {
    rows: Vec<ListRow>,
    sel: Option<usize>,
    out: ListOut,
}

fn l_view(ui: &mut Ui, s: &mut LS) {
    ui.scroll("ls").show(|ui| {
        s.out = ui.list_view("big", &s.rows, &mut s.sel);
    });
}

fn big_list() -> LS {
    let names = ["alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel"];
    let rows = (0..100_000).map(|i| ListRow::new(format!("{} {i:05}", names[i % names.len()])).subtitle("subtitle").meta("09:12")).collect();
    LS { rows, sel: None, out: ListOut::default() }
}

#[test]
fn list_view_keyboard_reaches_any_row() {
    let mut h = H::new(big_list(), l_view);
    h.click(vec2(200.0, 20.0));
    assert_eq!(h.s.sel, Some(0));
    h.key(Key::End);
    assert_eq!(h.s.sel, Some(99_999), "End selects the last of 100k rows");
    h.key(Key::Home);
    assert_eq!(h.s.sel, Some(0));
    h.key(Key::PageDown);
    let p = h.s.sel.unwrap();
    assert!(p > 3 && p < 40, "a page is about a viewport of rows: {p}");
    h.key(Key::ArrowDown);
    assert_eq!(h.s.sel, Some(p + 1));
    // Type-ahead: "go" jumps to the next "golf".
    h.frame(vec![Event::Text("g".into())]);
    h.frame(vec![Event::Text("o".into())]);
    h.idle(2);
    let s = h.s.sel.unwrap();
    assert!(h.s.rows[s].title.starts_with("golf"), "type-ahead picked {}", h.s.rows[s].title);
    // Focus survives: Down still works after the jump.
    h.key(Key::ArrowDown);
    assert_eq!(h.s.sel, Some(s + 1));
}

// -------------------------------------------------------------------- table

#[derive(Default)]
struct TS {
    clicked: Option<usize>,
    count: usize,
}

fn t_view(ui: &mut Ui, s: &mut TS) {
    let c = ui
        .table("big")
        .columns([TableCol::new("Id").width(90.0), TableCol::new("Name"), TableCol::new("Latency").right()])
        .lazy_rows(s.count, |i| i, |i| vec![TableCell::mono(format!("#{i}")), TableCell::text(format!("service {i}")), TableCell::muted(format!("{} ms", i % 97))])
        .height(400.0)
        .show();
    if c.is_some() {
        s.clicked = c;
    }
}

#[test]
fn lazy_table_scrolls_under_its_header_and_navigates() {
    let mut h = H::new(TS { count: 100_000, ..Default::default() }, t_view);
    // Click the first body row (below the header), then End.
    h.click(vec2(300.0, 60.0));
    assert_eq!(h.s.clicked, Some(0));
    h.key(Key::End);
    h.key(Key::Enter);
    assert_eq!(h.s.clicked, Some(99_999), "End then Enter acts on the last row");
    h.key(Key::Home);
    h.key(Key::ArrowDown);
    h.key(Key::Enter);
    assert_eq!(h.s.clicked, Some(1));
}

