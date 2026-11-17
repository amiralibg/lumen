//! Headless interaction tests for Batch 3+ widgets (Data, Charts, Planning, AI & Motion).

use std::collections::BTreeSet;

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

    fn idle(&mut self, n: usize) -> FrameOutput {
        let mut last = self.frame(vec![]);
        for _ in 1..n {
            last = self.frame(vec![]);
        }
        last
    }
}

fn key(k: Key) -> Vec<Event> {
    vec![
        Event::Key { key: k, pressed: true, repeat: false, modifiers: Modifiers::default() },
        Event::Key { key: k, pressed: false, repeat: false, modifiers: Modifiers::default() },
    ]
}

// ---------------------------------------------------------------- Accordion

#[derive(Default)]
struct AccState {
    open: Option<usize>,
    single: bool,
}

fn acc_view(ui: &mut Ui, s: &mut AccState) {
    ui.column().width(480.0).gap(12).show(|ui| {
        ui.accordion("faq", &mut s.open)
            .item("First", |ui| {
                ui.label("First body");
            })
            .item("Second", |ui| {
                ui.label("Second body");
            })
            .show();
        ui.collapsible("extra", "Extra details", &mut s.single).show(|ui| {
            ui.label("Collapsible content");
        });
    });
}

#[test]
fn accordion_and_collapsible_keyboard_and_idle() {
    let mut h = Harness::new(AccState { open: Some(0), single: false }, acc_view);
    let rest = h.idle(3);
    assert_eq!(rest.repaint_after, None, "settled accordion must not request idle repaints");

    // Tab focuses header 0; ArrowDown moves focus to header 1; Enter opens section 1.
    h.frame(key(Key::Tab));
    h.frame(key(Key::ArrowDown));
    h.idle(1);
    h.frame(key(Key::Enter));
    assert_eq!(h.s.open, Some(1));

    // Settle spring and verify zero idle wakeups.
    let settled = h.idle(90);
    assert_eq!(settled.repaint_after, None);
}

// -------------------------------------------------------------------- Table

#[derive(Default)]
struct TblState {
    sort: Option<(usize, SortDir)>,
    sel: Vec<bool>,
}

fn tbl_view(ui: &mut Ui, s: &mut TblState) {
    ui.column().width(560.0).show(|ui| {
        ui.table("svc")
            .columns([
                TableCol::new("Service").sortable(),
                TableCol::new("p95").width(100.0).sortable().right(),
            ])
            .sort_by(&mut s.sort)
            .selection(&mut s.sel)
            .row(0, [TableCell::strong("edge"), TableCell::mono("4.2 ms")])
            .row(1, [TableCell::strong("auth"), TableCell::mono("11.8 ms")])
            .show();
    });
}

#[test]
fn table_sort_and_indeterminate_header_selection() {
    let mut h = Harness::new(TblState { sort: None, sel: vec![true, false] }, tbl_view);
    // Focus order: header checkbox -> col 0 header -> col 1 header -> row 0 -> row 0 chk -> ...
    // Tab 1 focuses header checkbox (currently indeterminate because sel = [true, false]); Space checks all!
    h.frame(key(Key::Tab));
    h.frame(key(Key::Space));
    assert_eq!(h.s.sel, vec![true, true]);

    // Space again unchecks all!
    h.frame(key(Key::Space));
    assert_eq!(h.s.sel, vec![false, false]);

    // Tab to first sortable column header and cycle None -> Asc -> Desc -> None.
    h.frame(key(Key::Tab));
    h.frame(key(Key::Enter));
    assert_eq!(h.s.sort, Some((0, SortDir::Asc)));
    h.frame(key(Key::Enter));
    assert_eq!(h.s.sort, Some((0, SortDir::Desc)));
    h.frame(key(Key::Enter));
    assert_eq!(h.s.sort, None);
}

// --------------------------------------------------------------------- Tree

struct TreeState {
    expanded: BTreeSet<String>,
    selected: String,
}

fn tree_view(ui: &mut Ui, s: &mut TreeState) {
    let nodes = [
        TreeNode::branch("src", "src", [
            TreeNode::leaf("lib.rs", "lib.rs"),
            TreeNode::leaf("ui.rs", "ui.rs"),
        ]),
        TreeNode::leaf("Cargo.toml", "Cargo.toml"),
    ];
    ui.column().width(360.0).show(|ui| {
        ui.tree("files", &nodes, &mut s.expanded, &mut s.selected).show();
    });
}

#[test]
fn tree_keyboard_expand_collapse_and_navigation() {
    let mut h = Harness::new(
        TreeState { expanded: BTreeSet::new(), selected: "src".into() },
        tree_view,
    );
    assert_eq!(h.idle(2).repaint_after, None);

    // Tab focuses tree; ArrowRight expands "src"; second ArrowRight moves to first child "lib.rs".
    h.frame(key(Key::Tab));
    h.frame(key(Key::ArrowRight));
    assert!(h.s.expanded.contains("src"));
    h.frame(key(Key::ArrowRight));
    assert_eq!(h.s.selected, "lib.rs");

    // ArrowDown moves to "ui.rs"; ArrowLeft jumps back to parent "src"; second ArrowLeft collapses "src".
    h.frame(key(Key::ArrowDown));
    assert_eq!(h.s.selected, "ui.rs");
    h.frame(key(Key::ArrowLeft));
    assert_eq!(h.s.selected, "src");
    h.frame(key(Key::ArrowLeft));
    assert!(!h.s.expanded.contains("src"));
}

// ----------------------------------------------- Code, JSON, KV & LogViewer

struct DevDataState {
    pairs: Vec<KvPair>,
}

fn dev_data_view(ui: &mut Ui, s: &mut DevDataState) {
    let logs = [
        LogEntry::new("01:00:00", LogLevel::Info, "core", "ready"),
        LogEntry::new("01:00:01", LogLevel::Error, "net", "timeout"),
    ];
    let events = [TimelineItem::new("Deployed", "1m ago").tone(Tone::Success)];
    ui.column().width(600.0).gap(10).show(|ui| {
        ui.code_block("cb", "fn main() {}\n").language("rust").show();
        ui.json_viewer("jv", r#"{"ok": true, "items": [1, 2]}"#).show();
        ui.key_value_editor("kv", &mut s.pairs).show();
        ui.log_viewer("lv", &logs).height(140.0).show();
        ui.timeline("tl", &events);
    });
}

#[test]
fn code_block_json_viewer_kv_editor_and_log_viewer_work_and_settle() {
    let mut h = Harness::new(
        DevDataState { pairs: vec![KvPair::new("PORT", "8080")] },
        dev_data_view,
    );
    let rest = h.idle(5);
    assert_eq!(rest.repaint_after, None, "all Group 6 widgets must settle with zero idle wakeups");

    // First Tab focuses the Copy button on CodeBlock; Space copies code!
    h.frame(key(Key::Tab));
    let out = h.frame(key(Key::Space));
    assert_eq!(out.copied_text.as_deref(), Some("fn main() {}\n"));
}

// --------------------------------------------------- Charts, Rings & Kanban

struct PlanChartState {
    cards: Vec<KanbanCard>,
}

fn plan_chart_view(ui: &mut Ui, s: &mut PlanChartState) {
    let cols = [
        KanbanCol::new("Todo", Tone::Neutral),
        KanbanCol::new("Doing", Tone::Accent),
        KanbanCol::new("Done", Tone::Success),
    ];
    ui.column().width(640.0).gap(12).show(|ui| {
        ui.kanban("board", &cols, &mut s.cards);
        ui.progress_ring("pr", 0.75).show();
        ui.activity_rings("ar", &[RingSpec::new("CPU", 0.8, "64 vCPU")], 96.0);
        ui.bar_list("bl", &[BarListItem::new("/v1/chat", 1200.0)]);
        ui.heatmap("hm", 2, 3, &[1.0, 4.0, 9.0, 2.0, 5.0, 8.0]).show();
        ui.radar_chart("rc", &["A", "B", "C", "D"], vec![Series::new("S", vec![80.0, 70.0, 90.0, 85.0])]).show();
        ui.scatter_chart("sc", vec![ScatterSeries::new("G", vec![ScatterPoint::new(1.0, 2.0)])]).show();
        ui.funnel_chart("fc", &[("Visit", 100.0), ("Sign up", 40.0)]).show();
        ui.candlestick_chart("cc", &[Candle::new("D1", 10.0, 14.0, 9.0, 12.0)]).show();
    });
}

#[test]
fn kanban_moves_cards_and_all_charts_settle_to_zero_wakeups() {
    let mut h = Harness::new(
        PlanChartState {
            cards: vec![KanbanCard::new("1", 0, "Ship charts").tag("UI", Tone::Accent)],
        },
        plan_chart_view,
    );
    // Let initial entrance springs settle.
    let settled = h.idle(120);
    assert_eq!(settled.repaint_after, None, "all Group 7 charts + kanban must settle to zero idle wakeups");

    // Tab focuses the first Kanban card; ArrowRight moves it from col 0 -> col 1 -> col 2.
    h.frame(key(Key::Tab));
    h.frame(key(Key::ArrowRight));
    assert_eq!(h.s.cards[0].col, 1);
    h.frame(key(Key::ArrowRight));
    assert_eq!(h.s.cards[0].col, 2);
    // Clamped at last column.
    h.frame(key(Key::ArrowRight));
    assert_eq!(h.s.cards[0].col, 2);
    // ArrowLeft moves back to col 1.
    h.frame(key(Key::ArrowLeft));
    assert_eq!(h.s.cards[0].col, 1);
}

