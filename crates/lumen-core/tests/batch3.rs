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

// ----------------------------------------- AI, DevTools, Motion & Chrome

struct AiMotionState {
    cot_open: bool,
    prompt: String,
    model: usize,
    files: Vec<String>,
    prog: f32,
    playing: bool,
    dock_clicked: Option<usize>,
}

fn ai_motion_view(ui: &mut Ui, s: &mut AiMotionState) {
    ui.column().width(640.0).gap(10).show(|ui| {
        ui.reasoning_block("cot", "Thought for 1.2s", Some("2 steps"), &mut s.cot_open, |ui| {
            ui.tool_call("t1", &ToolCall::new("grep", "Token::Accent", ToolStatus::Done));
        });
        ui.prompt_composer("comp", &mut s.prompt, &mut s.model)
            .attachments(&mut s.files)
            .show();
        ui.chat_thread("chat", &[
            ChatMessage::user("Hi"),
            ChatMessage::assistant("lumen", "Hello").citations(&["lib.rs:1"]),
        ]);
        ui.diff_viewer("dv", "src/lib.rs", "@@ -1,2 +1,2 @@\n-old\n+new").show();
        ui.waterfall("wf", &[TraceSpan::new("GET /", "edge", 0.0, 12.0, 0)]);
        ui.uptime_bars("up", "API", &[UptimeStatus::Operational; 14]);
        ui.rolling_number("rn", 42_500).prefix("$").show();
        s.dock_clicked = ui.dock("dk", &[DockItem::new("terminal", "Term").active(true), DockItem::new("code", "Code")]);
        ui.waveform("wv", &[0.3, 0.7, 0.5, 0.9], &mut s.prog, &mut s.playing).show();
        ui.window_chrome("wc", "lumen").show(|ui| {
            ui.label("Inside window");
        });
    });
}

#[test]
fn ai_devtools_motion_and_window_chrome_interact_and_settle_cleanly() {
    let mut h = Harness::new(
        AiMotionState {
            cot_open: true,
            prompt: "Ship Batch 5".into(),
            model: 0,
            files: vec!["main.rs".into()],
            prog: 0.25,
            playing: false,
            dock_clicked: None,
        },
        ai_motion_view,
    );

    let settled = h.idle(120);
    assert_eq!(settled.repaint_after, None, "all Group 8 AI, devtools, motion & chrome widgets must settle to zero wakeups");

    // First Tab focuses the reasoning_block header; Space collapses it!
    h.frame(key(Key::Tab));
    h.frame(key(Key::Space));
    assert!(!h.s.cot_open);

    let settled2 = h.idle(90);
    assert_eq!(settled2.repaint_after, None);
}

// --------------------------------- Workspace, Inspector, Flame & Rich Views

struct WorkspaceState {
    ratio: f32,
    term_lines: Vec<TermLine>,
    term_in: String,
    px: f64,
    py: f64,
    clip: bool,
    notifs: Vec<NotificationItem>,
}

fn workspace_view(ui: &mut Ui, s: &mut WorkspaceState) {
    ui.column().width(660.0).gap(10).show(|ui| {
        ui.split_pane("sp", &mut s.ratio).height(120.0).show(|ui| { ui.label("L"); }, |ui| { ui.label("R"); });
        ui.menu_bar("mb", |bar| {
            bar.menu("File", |ui| { ui.menu_item("", "New"); });
            bar.menu("Edit", |ui| { ui.menu_item("", "Copy"); });
        });
        ui.terminal("tm", &s.term_lines, &mut s.term_in).height(120.0).show();
        ui.gantt_chart("gt", &["W1", "W2"], &[GanttTask::new("Core", 0.0, 1.5, 0.8), GanttTask::milestone("RC", 1.8)]);
        ui.treemap("tr", &[TreemapItem::new("core", 120.0), TreemapItem::new("wgpu", 80.0)]).height(140.0).show();
        ui.markdown("md", "# Title\n- item with `code` and **bold**\n> quote");
        ui.property_inspector("pi", |insp| {
            insp.section("Layout");
            insp.vec2_row("Pos", &mut s.px, &mut s.py);
            insp.toggle("Clip", &mut s.clip);
        });
        ui.flame_graph("fg", &[FlameSpan::new("root", 0, 0.0, 4.0), FlameSpan::new("child", 1, 0.5, 1.5)]);
        ui.notification_list("nl", &mut s.notifs);
        ui.spotlight_card("sc").icon("zap").title("Fast").show(|ui| { ui.label("Body"); });
    });
}

#[test]
fn split_pane_menubar_terminal_flame_and_rich_views_interact_and_settle() {
    let mut h = Harness::new(
        WorkspaceState {
            ratio: 0.4,
            term_lines: vec![TermLine::cmd("status")],
            term_in: String::new(),
            px: 10.0,
            py: 20.0,
            clip: true,
            notifs: vec![NotificationItem::new("1", "JB", "Ping", "Body", "1m", true)],
        },
        workspace_view,
    );

    let settled = h.idle(120);
    assert_eq!(settled.repaint_after, None, "all Group 9 workspace & rich widgets must settle to zero wakeups");

    // First Tab focuses the SplitPane sash; ArrowRight steps ratio up by 0.03; Home resets to 0.5!
    h.frame(key(Key::Tab));
    h.frame(key(Key::ArrowRight));
    assert!((h.s.ratio - 0.43).abs() < 1e-4);
    h.frame(key(Key::Home));
    assert!((h.s.ratio - 0.50).abs() < 1e-4);
}

// ------------------------- Group 10: Pro Inputs, Lists, Nav & Surfaces

struct Group10State {
    step_idx: usize,
    heading: String,
    money: f64,
    cur: lumen_core::Currency,
    masked: String,
    mention: String,
    avail: Vec<String>,
    assigned: Vec<String>,
    reorder: Vec<lumen_core::ReorderItem>,
    tabs: Vec<lumen_core::EditorTab>,
    tab_act: usize,
    toc_act: usize,
    nav_act: usize,
    slide_act: usize,
}

fn group10_view(ui: &mut Ui, s: &mut Group10State) {
    let team = [lumen_core::MentionCandidate::new("lena", "Lena Fischer", "GPU")];
    let toc = [lumen_core::TocItem::new("Intro", 0), lumen_core::TocItem::new("Shader", 1)];
    let nav = [lumen_core::NavBarItem::new("Home").icon("home"), lumen_core::NavBarItem::new("Deploy").badge("2")];
    let slides = [
        lumen_core::CarouselSlide::new("GPU", "Single shader", "SDF + glyphs", "layers", Tone::Accent),
        lumen_core::CarouselSlide::new("Motion", "Springs", "Closed form", "sparkles", Tone::Success),
    ];
    ui.column().width(660.0).gap(8).show(|ui| {
        ui.step_slider("Tier", &mut s.step_idx, &["1 GB", "4 GB", "16 GB"]);
        ui.editable_text("et", &mut s.heading).heading();
        ui.money_field(&mut s.money).switchable_currency(&mut s.cur);
        ui.masked_input(&mut s.masked, "###-###");
        ui.mention_input(&mut s.mention, &team);
        ui.transfer_list("tf", &mut s.avail, &mut s.assigned).height(120.0);
        ui.reorderable_list("ro", &mut s.reorder);
        ui.editor_tabs("etabs", &s.tabs, &mut s.tab_act);
        ui.table_of_contents("toc", &toc, &mut s.toc_act);
        ui.nav_bar("nb", &nav, &mut s.nav_act);
        ui.carousel("car", &mut s.slide_act, &slides).height(140.0);
        ui.callout(Tone::Info, "Tip").show(|ui| { ui.label("Body"); });
        ui.undo_bar("ub", "Archived item", 0.65).show();
        ui.image_frame("imf", "Preview").status(lumen_core::ImageStatus::Loaded).height(120.0);
    });
}

#[test]
fn group10_inputs_lists_nav_and_surfaces_interact_and_settle() {
    let mut h = Harness::new(
        Group10State {
            step_idx: 0,
            heading: "Gateway".into(),
            money: 1250.0,
            cur: lumen_core::Currency::Usd,
            masked: "415890".into(),
            mention: "Hi @lena".into(),
            avail: vec!["read".into()],
            assigned: vec!["write".into()],
            reorder: vec![lumen_core::ReorderItem::new("a", "Lint"), lumen_core::ReorderItem::new("b", "Test")],
            tabs: vec![lumen_core::EditorTab::new("1", "main.rs").dirty(true), lumen_core::EditorTab::new("2", "lib.rs")],
            tab_act: 0,
            toc_act: 0,
            nav_act: 0,
            slide_act: 0,
        },
        group10_view,
    );

    let settled = h.idle(120);
    assert_eq!(settled.repaint_after, None, "all 14 Group 10 widgets must settle to zero idle wakeups");
    assert_eq!(h.s.masked, "415-890", "MaskedInput should format digits automatically");

    // First Tab focuses StepSlider; ArrowRight advances stop_idx from 0 -> 1; End jumps to 2!
    h.frame(key(Key::Tab));
    h.frame(key(Key::ArrowRight));
    assert_eq!(h.s.step_idx, 1);
    h.frame(key(Key::End));
    assert_eq!(h.s.step_idx, 2);

    let settled2 = h.idle(90);
    assert_eq!(settled2.repaint_after, None);
}




