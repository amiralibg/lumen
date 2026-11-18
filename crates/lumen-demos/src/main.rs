//! One live demo per lumen component.
//!
//! - Native: `cargo run -p lumen-demos -- chart-line [--dark] [--look mac]`
//! - Web: `demo.html?demo=chart-line&dark=1&look=mac&accent=blue`
//! - Screenshot: `cargo run -p lumen-demos -- chart-line --screenshot out.png`

use std::collections::BTreeSet;

use lumen::{
    vec2, Accent, Align, BarListItem, Candle, CarouselSlide, ChatMessage, ChecklistStep, CitationSource, Color,
    Command, CommitRow, Currency, Date, DockItem, EditorTab, FilterFacet, FlameSpan, GanttTask, GraphNode, GraphWire,
    GridColumn, HistogramMarker, ImageStatus, Justify, KanbanCard, KanbanCol, KpiMetric, KvPair, LogEntry, LogLevel,
    Look, MentionCandidate, Mode, NavBarItem, NotificationItem, Placement, Platform, ReorderItem, RingSpec, SankeyLink,
    ScatterPoint, ScatterSeries, ScheduleEvent, Series, Sides, SortDir, TableCell, TableCol, TermLine, TestCase,
    TextRole, Time, TimelineItem, Toast, TocItem, Tone, ToolCall, ToolStatus, TraceSpan, TreeNode, TreemapItem, Ui,
    UptimeStatus,
};

/// Every demo: url name, title.
pub const DEMOS: &[(&str, &str)] = &[
    ("bento", "Overview bento"),
    ("hero", "Dashboard"),
    ("button", "Button"),
    ("split-button", "Split button"),
    ("hold-button", "Hold and copy buttons"),
    ("switch", "Switch"),
    ("checkbox", "Checkbox"),
    ("radio", "Radio"),
    ("radio-cards", "Radio cards"),
    ("rating", "Rating"),
    ("slider", "Slider"),
    ("range-slider", "Range slider"),
    ("shortcut-recorder", "Shortcut recorder"),
    ("dropzone", "Dropzone"),
    ("text-input", "Text input"),
    ("text-area", "Text area"),
    ("number-field", "Number field"),
    ("tag-input", "Tag input"),
    ("otp-input", "One-time code"),
    ("password-input", "Password input"),
    ("search-field", "Search field"),
    ("select", "Select"),
    ("segmented", "Segmented control"),
    ("tabs", "Tabs"),
    ("breadcrumb", "Breadcrumb"),
    ("pagination", "Pagination"),
    ("steps", "Steps"),
    ("toolbar", "Toolbar"),
    ("status-bar", "Status bar"),
    ("menu", "Menu"),
    ("popover", "Popover"),
    ("dialog", "Dialog"),
    ("sheet", "Sheet"),
    ("command-palette", "Command palette"),
    ("hover-card", "Hover card"),
    ("context-menu", "Context menu"),
    ("confirm-popover", "Confirm popover"),
    ("toast", "Toast"),
    ("tooltip", "Tooltip"),
    ("chips", "Filter chips"),
    ("stepper", "Stepper"),
    ("toggle-group", "Toggle group"),
    ("avatar", "Avatar"),
    ("card", "Card and settings"),
    ("badge", "Badge and status"),
    ("progress", "Progress and spinner"),
    ("meter", "Segmented meter"),
    ("alert", "Alert"),
    ("skeleton", "Skeleton"),
    ("empty-state", "Empty state"),
    ("kbd", "Keycap"),
    ("accordion", "Accordion"),
    ("table", "Data table"),
    ("tree", "Tree view"),
    ("timeline", "Timeline"),
    ("code-block", "Code block"),
    ("json-viewer", "JSON viewer"),
    ("key-value", "Key-value editor"),
    ("log-viewer", "Log viewer"),
    ("calendar", "Calendar"),
    ("date-picker", "Date picker"),
    ("combobox", "Combobox"),
    ("multi-select", "Multi-select"),
    ("date-range-picker", "Date range picker"),
    ("time-picker", "Time picker"),
    ("color-picker", "Color picker"),
    ("chart-line", "Line chart"),
    ("chart-area", "Area chart"),
    ("chart-bar", "Bar chart"),
    ("donut", "Donut chart"),
    ("gauge", "Gauge"),
    ("progress-ring", "Progress and activity rings"),
    ("bar-list", "Bar list"),
    ("heatmap", "Heatmap"),
    ("radar-chart", "Radar chart"),
    ("scatter-chart", "Scatter chart"),
    ("funnel-chart", "Funnel chart"),
    ("candlestick", "Candlestick chart"),
    ("kanban", "Kanban board"),
    ("ai-reasoning", "AI reasoning and tools"),
    ("ai-composer", "AI prompt composer"),
    ("ai-chat", "AI chat thread"),
    ("diff-viewer", "Diff viewer"),
    ("waterfall", "Trace waterfall"),
    ("uptime-bars", "Uptime status"),
    ("rolling-number", "Rolling number"),
    ("dock", "Magnifying dock"),
    ("waveform", "Audio waveform"),
    ("window-chrome", "Window chrome"),
    ("split-pane", "Split pane"),
    ("menu-bar", "Menu bar"),
    ("terminal", "Terminal"),
    ("gantt", "Gantt chart"),
    ("treemap", "Treemap"),
    ("markdown", "Markdown view"),
    ("property-grid", "Property inspector"),
    ("flame-graph", "Flame graph"),
    ("notification-list", "Notification inbox"),
    ("spotlight-card", "Spotlight card"),
    ("editable-text", "Editable text"),
    ("money-field", "Money field"),
    ("masked-input", "Masked input"),
    ("mention-input", "Mention input"),
    ("step-slider", "Step slider"),
    ("transfer-list", "Transfer list"),
    ("reorderable-list", "Reorderable list"),
    ("editor-tabs", "Editor tabs"),
    ("table-of-contents", "Table of contents"),
    ("nav-bar", "Navigation bar"),
    ("carousel", "Carousel"),
    ("callout", "Callout"),
    ("undo-toast", "Undo snackbar"),
    ("image-frame", "Image frame"),
    ("sankey-chart", "Sankey chart"),
    ("contribution-graph", "Contribution graph"),
    ("histogram", "Histogram"),
    ("kpi-strip", "KPI strip"),
    ("day-schedule", "Week schedule"),
    ("node-editor", "Node editor"),
    ("commit-graph", "Commit graph"),
    ("test-explorer", "Test explorer"),
    ("data-grid", "Data grid"),
    ("filter-bar", "Filter bar"),
    ("ai-citations", "AI citations"),
    ("text-effects", "Text effects"),
    ("rotary-knob", "Rotary knob"),
    ("compare-slider", "Comparison slider"),
    ("countdown", "Countdown timer"),
    ("checklist-card", "Checklist card"),
    ("sparkline", "Sparkline and stats"),
    ("looks", "Looks"),
    ("vault", "Vault (password manager preview)"),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Region {
    Frankfurt,
    Virginia,
    Tokyo,
    Sydney,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Range {
    Day,
    Week,
    Month,
    Year,
}

struct State {
    demo: String,
    embed: bool,
    /// Screenshots: show overlays (menus, dialogs) open.
    force_open: bool,
    on: [bool; 4],
    checks: [bool; 4],
    plan: u8,
    vol: f32,
    price: f32,
    name: String,
    email: String,
    region: Region,
    range: Range,
    tab: u8,
    open: bool,
    date: Date,
    span: Option<(Date, Date)>,
    look: Platform,
    progress: f32,
    t: u32,
    search: String,
    os: u8,
    runner: String,
    labels: [bool; 5],
    jobs: i64,
    slot: u8,
    reduced: bool,
    tz: Option<usize>,
    reviewers: Vec<usize>,
    deploy_time: Time,
    quiet_end: Time,
    brand_color: Color,
    glow_color: Color,
    sheet_side: Placement,
    pinned: bool,
    revoked: bool,
    hover_shot: bool,
    release_notes: String,
    timeout_ms: f64,
    mem_gb: f64,
    budget_usd: f64,
    tags: Vec<String>,
    otp_code: String,
    new_pw: String,
    confirm_pw: String,
    deploy_query: String,
    banner_open: bool,
    skeleton_loading: bool,
    crumb_depth: usize,
    page: usize,
    wizard_step: usize,
    tier: u8,
    stars: u8,
    price_band: (f32, f32),
    deploy_chord: Vec<String>,
    uploads: Vec<(String, String)>,
    fmt_flags: [bool; 4],
    busy_deploy: bool,
    acc_open: Option<usize>,
    table_sort: Option<(usize, SortDir)>,
    table_sel: Vec<bool>,
    tree_open: BTreeSet<String>,
    tree_sel: String,
    env_vars: Vec<KvPair>,
    kanban_cards: Vec<KanbanCard>,
    cot_open: bool,
    ai_prompt: String,
    ai_model: usize,
    ai_files: Vec<String>,
    ai_streaming: bool,
    mrr_val: i64,
    dock_active: usize,
    audio_prog: f32,
    audio_playing: bool,
    win_look: Platform,
    split_ratio: f32,
    term_lines: Vec<TermLine>,
    term_in: String,
    prop_pos: (f64, f64),
    prop_radius: f64,
    prop_opacity: f64,
    prop_clip: bool,
    notifs: Vec<NotificationItem>,
    edit_heading: String,
    edit_summary: String,
    money_budget: f64,
    money_cur: Currency,
    money_alert: f64,
    phone_mask: String,
    license_mask: String,
    mention_msg: String,
    mem_stop: usize,
    ret_stop: usize,
    avail_scopes: Vec<String>,
    assigned_scopes: Vec<String>,
    reorder_items: Vec<ReorderItem>,
    ed_tabs: Vec<EditorTab>,
    ed_active: usize,
    toc_active: usize,
    nav_active: usize,
    carousel_idx: usize,
    undo_visible: bool,
    undo_rem: f32,
    img_status: ImageStatus,
    graph_nodes: Vec<GraphNode>,
    commit_sel: usize,
    test_cases: Vec<TestCase>,
    grid_rows: Vec<Vec<String>>,
    facets: Vec<FilterFacet>,
    knobs: [f32; 3],
    cmp_ratio: f32,
    cd_secs: u32,
    cd_running: bool,
    check_steps: Vec<ChecklistStep>,
}

impl State {
    fn new(demo: String) -> State {
        let today = Date::today();
        State {
            demo,
            embed: false,
            force_open: false,
            on: [true, false, true, false],
            checks: [true, false, true, false],
            plan: 1,
            vol: 0.64,
            price: 40.0,
            name: "Ada Lovelace".into(),
            email: String::new(),
            region: Region::Frankfurt,
            range: Range::Week,
            tab: 0,
            open: false,
            date: today,
            span: Some((today.add_days(-3), today.add_days(4))),
            look: Platform::Lumen,
            progress: 0.42,
            t: 0,
            search: String::new(),
            os: 0,
            runner: "linux-arm64-02".into(),
            labels: [true, true, false, false, false],
            jobs: 4,
            slot: 1,
            reduced: false,
            tz: Some(2),
            reviewers: vec![0, 3],
            deploy_time: Time::new(14, 30),
            quiet_end: Time::new(8, 0),
            brand_color: Color::hex(0x0E6EE6),
            glow_color: Color::hex(0x7D55EC).with_alpha(0.72),
            sheet_side: Placement::Right,
            pinned: true,
            revoked: false,
            hover_shot: false,
            release_notes: "Added zero-copy glyph uploads and spring-driven sheet drawers.\nReduced idle frame wakeups to zero across all four looks.".into(),
            timeout_ms: 2500.0,
            mem_gb: 8.0,
            budget_usd: 240.0,
            tags: vec!["rust".into(), "wgpu".into(), "wasm".into(), "arm64".into(), "metal".into(), "linux".into()],
            otp_code: "482".into(),
            new_pw: "Northwind42".into(),
            confirm_pw: String::new(),
            deploy_query: String::new(),
            banner_open: true,
            skeleton_loading: true,
            crumb_depth: 4,
            page: 4,
            wizard_step: 1,
            tier: 1,
            stars: 4,
            price_band: (20.0, 75.0),
            deploy_chord: vec!["⌘".into(), "⇧".into(), "D".into()],
            uploads: vec![("lumen-demos.wasm".into(), "7.4 MB".into()), ("lucide.ttf".into(), "218 KB".into())],
            fmt_flags: [true, false, false, true],
            busy_deploy: false,
            acc_open: Some(0),
            table_sort: Some((2, SortDir::Asc)),
            table_sel: vec![false, true, false, true],
            tree_open: ["crates", "lumen-core", "src"].into_iter().map(String::from).collect(),
            tree_sel: "theme.rs".into(),
            env_vars: vec![
                KvPair::new("DATABASE_URL", "postgres://app:s3cr3t@db.internal:5432/prod").secret(),
                KvPair::new("RUST_LOG", "info,lumen_wgpu=warn"),
                KvPair::new("EDGE_REGION", "eu-central-1"),
            ],
            kanban_cards: vec![
                KanbanCard::new("104", 0, "Subpixel atlas defragmentation").tag("GPU", Tone::Accent).meta("LUM-104").assignee("JB"),
                KanbanCard::new("108", 0, "IME composition underline").tag("Text", Tone::Neutral).meta("LUM-108").assignee("PN"),
                KanbanCard::new("112", 1, "Spring-driven data table").tag("Widgets", Tone::Accent).meta("LUM-112").assignee("LF"),
                KanbanCard::new("115", 1, "High-DPI scissor clip rounding").tag("Fix", Tone::Warning).meta("LUM-115").assignee("TR"),
                KanbanCard::new("119", 2, "Zero idle wakeups at rest").tag("Perf", Tone::Success).meta("LUM-119").assignee("AK"),
            ],
            cot_open: true,
            ai_prompt: "Audit crates/lumen-core/src/layout.rs for rounding drift on 1.5x displays.".into(),
            ai_model: 0,
            ai_files: vec!["layout.rs".into(), "paint.rs".into()],
            ai_streaming: false,
            mrr_val: 128_490,
            dock_active: 1,
            audio_prog: 0.38,
            audio_playing: false,
            win_look: Platform::Mac,
            split_ratio: 0.38,
            term_lines: vec![
                TermLine::cmd("cargo test -p lumen-core --quiet"),
                TermLine::ok("test result: ok. 119 passed; 0 failed; finished in 0.04s"),
                TermLine::cmd("lumen build --target wasm32 --release"),
                TermLine::out("Optimized lumen-demos_bg.wasm (100 live components ready)"),
            ],
            term_in: String::new(),
            prop_pos: (128.0, 64.0),
            prop_radius: 16.0,
            prop_opacity: 94.0,
            prop_clip: true,
            notifs: vec![
                NotificationItem::new("n1", "LF", "Merged PR #142 into ethereal", "Added GPU arc-band activity rings and radar meshes.", "3m ago", true),
                NotificationItem::new("n2", "JB", "Canary rollout completed", "Zero error-budget burn across eu-central and us-east.", "18m ago", true),
                NotificationItem::new("n3", "PN", "Weekly benchmark digest", "Median frame tessellation held at 0.19 ms on M3.", "2h ago", false),
            ],
            edit_heading: "Production Edge Gateway".into(),
            edit_summary: "Zero-downtime TLS termination and routing across 14 regions.".into(),
            money_budget: 4250.0,
            money_cur: Currency::Usd,
            money_alert: 950.0,
            phone_mask: "+1 (415) 890".into(),
            license_mask: "LUMN-2026-CORE".into(),
            mention_msg: "Ready to cut v0.5 — cc @lena".into(),
            mem_stop: 2,
            ret_stop: 3,
            avail_scopes: vec![
                "audit:export".into(),
                "billing:write".into(),
                "runners:delete".into(),
                "secrets:rotate".into(),
            ],
            assigned_scopes: vec![
                "deploys:read".into(),
                "deploys:trigger".into(),
                "metrics:read".into(),
            ],
            reorder_items: vec![
                ReorderItem::new("lint", "Typecheck & clippy").subtitle("cargo clippy --workspace --deny warnings").badge("18s"),
                ReorderItem::new("test", "Headless core suite").subtitle("120 deterministic layout & spring assertions").badge("0.4s"),
                ReorderItem::new("wasm", "Build WebGPU bundle").subtitle("wasm-bindgen --target web --release").badge("24s"),
                ReorderItem::new("edge", "Promote canary traffic").subtitle("Shift 10% → 100% on zero 5xx burn").badge("Gate"),
            ],
            ed_tabs: vec![
                EditorTab::new("shader", "shader.wgsl").icon("sparkles"),
                EditorTab::new("paint", "paint.rs").icon("file-code").dirty(true),
                EditorTab::new("theme", "theme.rs").icon("palette"),
                EditorTab::new("cargo", "Cargo.toml").icon("package"),
            ],
            ed_active: 1,
            toc_active: 1,
            nav_active: 0,
            carousel_idx: 0,
            undo_visible: true,
            undo_rem: 0.68,
            img_status: ImageStatus::Loaded,
            graph_nodes: vec![
                GraphNode::new("UV Coordinates", vec2(18.0, 56.0), &[], &["uv", "dpi"], Tone::Info),
                GraphNode::new("SDF RoundRect", vec2(218.0, 34.0), &["uv", "radii"], &["dist"], Tone::Accent),
                GraphNode::new("OKLab Shade", vec2(422.0, 56.0), &["dist", "dpi"], &["rgba"], Tone::Success),
            ],
            commit_sel: 0,
            test_cases: vec![
                TestCase::passed("anim::spring", "retarget_preserves_velocity", 0.08),
                TestCase::passed("paint::sdf", "quads_batch_by_scissor_clip", 0.14),
                TestCase::failed(
                    "layout::flex",
                    "subpixel_baseline_snap_2x",
                    0.31,
                    "assertion `left == right` failed: expected y=42.5, got y=42.0 (missing scale=2.0 snap)",
                ),
                TestCase::passed("theme::oklab", "derived_accents_meet_contrast", 0.11),
            ],
            grid_rows: vec![
                vec!["eu-central-1".into(), "4".into(), "48,000".into(), "14.2 ms".into()],
                vec!["us-east-1".into(), "6".into(), "72,000".into(), "18.6 ms".into()],
                vec!["ap-northeast-1".into(), "3".into(), "36,000".into(), "21.0 ms".into()],
                vec!["eu-west-2".into(), "3".into(), "32,000".into(), "15.8 ms".into()],
            ],
            facets: vec![
                FilterFacet::new("Status", "is", "Healthy"),
                FilterFacet::new("Backend", "is", "WebGPU"),
            ],
            knobs: [64.0, 420.0, 38.0],
            cmp_ratio: 0.48,
            cd_secs: 702,
            cd_running: false,
            check_steps: vec![
                ChecklistStep::new("Verify zero idle wakeups", "All 130 components return repaint_after == None at rest.", "Core", true),
                ChecklistStep::new("Audit dark & light OKLab tokens", "Zero hardcoded hex colors inside widget renderers.", "Theme", true),
                ChecklistStep::new("Compile WebGPU demo bundle", "wasm-bindgen release target under site/demo.", "Web", true),
                ChecklistStep::new("Tag v0.5.0 release branch", "Sign release notes and publish crates.", "Ship", false),
            ],
        }
    }
}

const ZONES: [&str; 14] = [
    "Pacific/Honolulu", "America/Los_Angeles", "America/Denver", "America/Chicago", "America/New_York", "America/Sao_Paulo", "Europe/London",
    "Europe/Berlin", "Europe/Istanbul", "Asia/Dubai", "Asia/Kolkata", "Asia/Singapore", "Asia/Tokyo", "Australia/Sydney",
];
const PEOPLE: [&str; 8] = ["Jonas Berg", "Priya Nair", "Tomas Rivera", "Lena Fischer", "Sam Wu", "Dmitri Markov", "Aiko Kato", "Noor Haddad"];

const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

fn revenue() -> Vec<f32> {
    vec![42.0, 48.0, 45.0, 61.0, 58.0, 72.0, 69.0, 84.0, 80.0, 96.0, 104.0, 118.0]
}
fn costs() -> Vec<f32> {
    vec![30.0, 33.0, 35.0, 38.0, 41.0, 40.0, 46.0, 49.0, 52.0, 55.0, 58.0, 61.0]
}

fn frame(ui: &mut Ui, f: impl FnOnce(&mut Ui)) {
    frame_at(ui, Justify::Center, f)
}

/// Demos whose overlays open downward start near the top.
fn frame_at(ui: &mut Ui, justify: Justify, f: impl FnOnce(&mut Ui)) {
    ui.column().fill().pad(40).align(Align::Center).justify(justify).show(|ui| {
        ui.column().fill_width().max_width(760.0).gap(16).align(Align::Center).show(f);
    });
}

fn view(ui: &mut Ui, s: &mut State) {
    if s.reduced {
        ui.set_reduced_motion(true);
    }
    let demo = s.demo.clone();
    // Embedded component demos sit on the host page's white card.
    if s.embed && !matches!(demo.as_str(), "hero" | "vault" | "bento") {
        for m in [Mode::Light, Mode::Dark] {
            let surf = ui.theme.target_look().neutrals(m).surface;
            ui.override_color(m, lumen::Token::Background, Some(surf));
        }
    }
    match demo.as_str() {
        "hero" => hero(ui, s),
        "bento" => bento(ui, s),
        "looks" => looks(ui, s),
        "vault" => vault(ui, s),
        "menu"
        | "popover"
        | "select"
        | "date-picker"
        | "stepper"
        | "combobox"
        | "multi-select"
        | "date-range-picker"
        | "time-picker"
        | "color-picker"
        | "hover-card"
        | "context-menu"
        | "confirm-popover"
        | "split-button"
        | "menu-bar"
        | "mention-input" => frame_at(ui, Justify::Start, |ui| component(ui, s, &demo)),
        _ => frame(ui, |ui| component(ui, s, &demo)),
    }
}

fn component(ui: &mut Ui, s: &mut State, demo: &str) {
    match demo {
        "button" => {
            ui.row().gap(10).align(Align::Center).show(|ui| {
                ui.button("Deploy").primary().icon("rocket");
                ui.button("Preview").icon("eye");
                ui.button("Cancel").ghost();
                ui.button("Delete").danger().icon("trash-2");
                ui.icon_button("copy").tooltip_shortcut("Copy", "⌘C");
            });
            ui.row().gap(10).align(Align::Center).show(|ui| {
                ui.button("Small").small();
                ui.button("Medium");
                ui.button("Large").large().primary();
                ui.button("Disabled").disabled(true);
            });
        }
        "switch" => {
            ui.column().gap(14).show(|ui| {
                ui.switch("Wi-Fi", &mut s.on[0]);
                ui.switch("Bluetooth", &mut s.on[1]);
                ui.switch("AirDrop", &mut s.on[2]);
                ui.switch("Hotspot (managed)", &mut s.on[3]).disabled(true);
            });
        }
        "checkbox" => {
            ui.column().gap(12).show(|ui| {
                let all = s.checks.iter().all(|c| *c);
                let none = s.checks.iter().all(|c| !*c);
                let mut a = all;
                if ui.checkbox("Select all", &mut a).indeterminate(!all && !none).changed() {
                    let v = !all;
                    s.checks = [v; 4];
                }
                ui.column().gap(10).pad(Sides { left: 24.0, ..Default::default() }).show(|ui| {
                    ui.checkbox("Email", &mut s.checks[0]).description("Digests and receipts");
                    ui.checkbox("Push", &mut s.checks[1]);
                    ui.checkbox("SMS", &mut s.checks[2]);
                    ui.checkbox("Pager", &mut s.checks[3]);
                });
            });
        }
        "radio" => {
            ui.column().gap(12).show(|ui| {
                ui.radio(&mut s.plan, 0, "Hobby — free");
                ui.radio(&mut s.plan, 1, "Pro — $20 / month");
                ui.radio(&mut s.plan, 2, "Team — $80 / month");
            });
        }
        "slider" => {
            ui.column().gap(20).show(|ui| {
                let v = format!("{:.0}%", s.vol * 100.0);
                ui.slider("Volume", &mut s.vol, 0.0..=1.0).readout(v).width(320.0);
                let p = format!("${:.0}", s.price);
                ui.slider("Max price", &mut s.price, 0.0..=100.0).step(5.0).readout(p).width(320.0);
            });
        }
        "text-input" => {
            ui.column().gap(16).fill_width().max_width(360.0).show(|ui| {
                ui.text_input(&mut s.name).label("Full name").fill_width();
                let err = (!s.email.is_empty() && !s.email.contains('@')).then_some("Enter a valid email address.");
                ui.text_input(&mut s.email).label("Email").icon("mail").placeholder("you@example.com").error(err).helper("We never share it.").fill_width();
                ui.text_input(&mut s.search).icon("search").placeholder("Search…").fill_width();
            });
        }
        "text-area" => {
            ui.column().gap(12).fill_width().max_width(420.0).show(|ui| {
                ui.text_area(&mut s.release_notes)
                    .label("Release notes")
                    .placeholder("Summarize what shipped in this tag…")
                    .rows(3, 6)
                    .max_chars(240)
                    .helper("Shown on the public changelog.")
                    .fill_width();
                ui.row().fill_width().gap(8).align(Align::Center).show(|ui| {
                    ui.spacer();
                    if ui.button("Clear").ghost().small().disabled(s.release_notes.is_empty()).clicked() {
                        s.release_notes.clear();
                    }
                    if ui.button("Publish notes").primary().small().disabled(s.release_notes.trim().is_empty()).clicked() {
                        ui.toast(Toast::new("Release notes published").success());
                    }
                });
            });
        }
        "number-field" => {
            ui.column().gap(14).fill_width().max_width(420.0).show(|ui| {
                ui.row().fill_width().gap(12).show(|ui| {
                    ui.number_field(&mut s.timeout_ms).label("Timeout").range(100.0..=30000.0).step(50.0).suffix("ms").fill_width();
                    ui.number_field(&mut s.mem_gb).label("Memory").range(0.5..=64.0).step(0.5).decimals(1).suffix("GB").fill_width();
                });
                ui.number_field(&mut s.budget_usd)
                    .label("Monthly spend cap")
                    .range(0.0..=5000.0)
                    .step(10.0)
                    .prefix("$")
                    .helper("Drag any label or handle left/right to scrub · Shift 10× · Alt 0.1×")
                    .fill_width();
            });
        }
        "tag-input" => {
            ui.column().gap(12).fill_width().max_width(400.0).show(|ui| {
                ui.tag_input(&mut s.tags)
                    .label("Runner labels")
                    .placeholder("Add label…")
                    .lowercase(true)
                    .max_tags(8)
                    .helper("Press Enter or comma to add · Backspace twice removes the last tag.")
                    .fill_width();
            });
        }
        "otp-input" => {
            ui.column().gap(12).align(Align::Center).show(|ui| {
                ui.column().gap(2).align(Align::Center).show(|ui| {
                    ui.label("Verify hardware key").strong();
                    ui.label("Enter the 6-digit code from your authenticator.").small().secondary();
                });
                let bad = (s.otp_code == "000000").then_some("Invalid verification code. Try any other 6 digits.");
                if ui.otp_input("2fa", &mut s.otp_code).error(bad).completed() && s.otp_code != "000000" {
                    ui.toast(Toast::new("Session verified").success());
                }
                if s.otp_code.len() == 6 && s.otp_code != "000000" {
                    ui.badge("Verified", Tone::Success);
                } else {
                    ui.label("Tip: paste “482-913” to fill every cell at once.").small().muted();
                }
            });
        }
        "password-input" => {
            ui.column().gap(14).fill_width().max_width(380.0).show(|ui| {
                ui.password_input(&mut s.new_pw)
                    .label("Master password")
                    .icon("lock")
                    .placeholder("Create a strong passphrase")
                    .strength(true)
                    .copyable(true)
                    .fill_width();
                let mismatch = (!s.confirm_pw.is_empty() && s.confirm_pw != s.new_pw).then_some("Passwords do not match.");
                ui.password_input(&mut s.confirm_pw)
                    .label("Confirm password")
                    .placeholder("Re-enter master password")
                    .error(mismatch)
                    .fill_width();
            });
        }
        "search-field" => {
            const DEPLOYS: [(&str, &str, Tone); 5] = [
                ("v2.4.1 · Frankfurt", "2m ago", Tone::Success),
                ("v2.4.0 · Virginia", "1h ago", Tone::Success),
                ("v2.4.0-rc2 · Tokyo", "4h ago", Tone::Info),
                ("canary-882 · Sydney", "6h ago", Tone::Warning),
                ("v2.3.9 · London", "Yesterday", Tone::Neutral),
            ];
            let q = s.deploy_query.trim().to_lowercase();
            let hits: Vec<_> = DEPLOYS.iter().copied().filter(|(name, _, _)| q.is_empty() || name.to_lowercase().contains(&q)).collect();
            ui.column().gap(10).fill_width().max_width(400.0).show(|ui| {
                ui.search_field(&mut s.deploy_query)
                    .placeholder("Filter deployments…")
                    .shortcut("/")
                    .focus_key('/')
                    .results(Some(hits.len()))
                    .fill_width();
                ui.card().pad(6.0).show(|ui| {
                    if hits.is_empty() {
                        ui.row().fill_width().pad(10.0).justify(Justify::Center).show(|ui| {
                            ui.label(format!("No deployments matching “{}”.", s.deploy_query.trim())).small().muted();
                        });
                    } else {
                        ui.column().fill_width().gap(2).show(|ui| {
                            for (name, when, tone) in hits.into_iter().take(3) {
                                ui.row().fill_width().pad(Sides::xy(10.0, 6.0)).gap(10).align(Align::Center).show(|ui| {
                                    ui.status(name, tone);
                                    ui.spacer();
                                    ui.label(when).small().muted();
                                });
                            }
                        });
                    }
                });
            });
        }
        "select" => {
            ui.column().gap(16).fill_width().max_width(280.0).show(|ui| {
                ui.select(&mut s.region, &[(Region::Frankfurt, "Frankfurt (eu-central)"), (Region::Virginia, "Virginia (us-east)"), (Region::Tokyo, "Tokyo (ap-northeast)"), (Region::Sydney, "Sydney (ap-southeast)")])
                    .label("Region")
                    .fill_width();
            });
        }
        "segmented" => {
            ui.segmented(&mut s.range, &[(Range::Day, "Day"), (Range::Week, "Week"), (Range::Month, "Month"), (Range::Year, "Year")]);
        }
        "tabs" => {
            ui.column().fill_width().max_width(480.0).gap(16).show(|ui| {
                ui.tabs(&mut s.tab, &[(0, "Overview"), (1, "Activity"), (2, "Settings"), (3, "Billing")]);
                let text = ["Everything at a glance.", "What changed, and who did it.", "Preferences for this project.", "Invoices and payment methods."][s.tab as usize];
                ui.label(text).secondary();
            });
        }
        "menu" => {
            let r = ui.button("Actions").trailing_icon("chevron-down").show();
            if std::mem::take(&mut s.force_open) {
                ui.set_popover_open(r.id, true);
            }
            ui.menu(&r, |ui| {
                ui.menu_label("Project");
                ui.menu_item("pencil", "Rename").shortcut("⌘R");
                ui.menu_item("copy", "Duplicate").shortcut("⌘D");
                ui.menu_item("share-2", "Share…");
                ui.menu_separator();
                ui.menu_item("archive", "Archive");
                ui.menu_item("trash-2", "Delete").danger().shortcut("⌫");
            });
            ui.label("Click the button, or Tab to it and press Enter.").muted();
        }
        "popover" => {
            let r = ui.button("Share").icon("share-2").show();
            ui.popover(&r).pad(16.0).width(300.0).show(|ui| {
                ui.column().gap(10).fill_width().show(|ui| {
                    ui.label("Share this board").strong();
                    ui.label("Anyone with the link can view.").secondary();
                    ui.row().gap(8).fill_width().show(|ui| {
                        let mut link = "lumen.app/b/8f3k".to_string();
                        ui.text_input(&mut link).fill_width().id("link");
                        if ui.button("Copy").primary().clicked() {
                            ui.close_layer();
                        }
                    });
                });
            });
        }
        "dialog" => {
            if ui.button("Delete project").danger().icon("trash-2").clicked() || std::mem::take(&mut s.force_open) {
                s.open = true;
            }
            ui.dialog("confirm", &mut s.open)
                .title("Delete “Northwind”?")
                .description("This removes the project, its 14 deploys and all logs. You can't undo this.")
                .footer(|ui| {
                    if ui.button("Cancel").ghost().clicked() {
                        ui.close_layer();
                    }
                    if ui.button("Delete").danger().clicked() {
                        ui.close_layer();
                    }
                })
                .show(|_| {});
        }
        "sheet" => {
            if std::mem::take(&mut s.force_open) {
                s.open = true;
            }
            ui.row().gap(12).align(Align::Center).show(|ui| {
                ui.segmented(&mut s.sheet_side, &[(Placement::Right, "Right"), (Placement::Left, "Left"), (Placement::Bottom, "Bottom")]);
                if ui.button("Inspect runner").primary().icon("panel-right").clicked() {
                    s.open = true;
                }
            });
            ui.sheet("runner", &mut s.open)
                .side(s.sheet_side)
                .title("linux-arm64-02")
                .description("Frankfurt · online · 4 concurrent jobs")
                .footer(|ui| {
                    if ui.button("Cancel").ghost().clicked() {
                        ui.close_layer();
                    }
                    if ui.button("Save changes").primary().clicked() {
                        ui.toast(Toast::new("Runner updated").success());
                        ui.close_layer();
                    }
                })
                .show(|ui| {
                    ui.text_input(&mut s.runner).label("Hostname").fill_width();
                    ui.select(&mut s.region, &[(Region::Frankfurt, "Frankfurt (eu-central)"), (Region::Virginia, "Virginia (us-east)"), (Region::Tokyo, "Tokyo (ap-northeast)"), (Region::Sydney, "Sydney (ap-southeast)")])
                        .label("Region")
                        .fill_width();
                    ui.settings(|rows| {
                        rows.row("Auto-drain before reboot", "Finish running jobs before restarting.", |ui| {
                            ui.switch("", &mut s.on[0]);
                        });
                        rows.row("Spot fallback", "Allow burst capacity on spot instances.", |ui| {
                            ui.switch("", &mut s.on[2]);
                        });
                    });
                });
        }
        "command-palette" => {
            const CMDS: &[Command<'static>] = &[
                Command::new("deploy", "Deploy production").group("Actions").icon("rocket").shortcut("⌘D").keywords("ship release"),
                Command::new("logs", "Tail live logs").group("Actions").icon("terminal").shortcut("⌘L").keywords("console output"),
                Command::new("branch", "Switch branch…").group("Actions").icon("git-branch").shortcut("⌘B"),
                Command::new("runners", "Go to Runners").group("Navigation").icon("users").shortcut("G R"),
                Command::new("settings", "Workspace settings").group("Navigation").icon("settings").shortcut("⌘,"),
                Command::new("theme", "Toggle dark mode").group("Appearance").icon("moon").shortcut("⌘T"),
            ];
            if std::mem::take(&mut s.force_open) {
                s.open = true;
            }
            ui.column().gap(10).align(Align::Center).show(|ui| {
                if ui.button("Open command palette").icon("search").clicked() {
                    s.open = true;
                }
                ui.label("Or press ⌘K / Ctrl+K anywhere in this demo.").muted();
            });
            if let Some(cmd) = ui.command_palette("main", &mut s.open, CMDS).show() {
                ui.toast(Toast::new(format!("Triggered “{cmd}”")).tone(Tone::Accent));
            }
        }
        "hover-card" => {
            if std::mem::take(&mut s.force_open) {
                s.hover_shot = true;
            }
            let people = [
                ("@jonas", "JB", "Jonas Berg", "Staff Systems · Frankfurt", "42 deploys this month", Some(true)),
                ("@priya", "PN", "Priya Nair", "Security Lead · London", "19 reviews this week", Some(false)),
                ("@lena", "LF", "Lena Fischer", "Runtime · Berlin", "Maintains lumen-wgpu", Some(true)),
            ];
            ui.row().gap(8).align(Align::Center).show(|ui| {
                ui.label("Approved by").secondary();
                for (i, (handle, ini, name, role, note, pres)) in people.iter().enumerate() {
                    let r = ui.button(*handle).ghost().small().show();
                    ui.hover_card(&r).width(276.0).open(s.hover_shot && i == 0).show(|ui| {
                        ui.row().gap(12).align(Align::Center).show(|ui| {
                            ui.avatar(ini, i, 38.0, *pres);
                            ui.column().gap(2).fill_width().show(|ui| {
                                ui.label(*name).strong();
                                ui.label(*role).small().secondary();
                            });
                        });
                        ui.divider();
                        ui.label(*note).small().muted();
                    });
                }
            });
        }
        "context-menu" => {
            let zone = ui.column().fill_width().max_width(440.0).id("release-row").show(|ui| {
                ui.card().pad(16.0).show(|ui| {
                    ui.row().fill_width().gap(12).align(Align::Center).show(|ui| {
                        ui.column().gap(2).fill_width().show(|ui| {
                            ui.label("v2.4.1 — Production").strong();
                            ui.label("Right-click this card (or focus and press Shift+Enter).").small().muted();
                        });
                        if s.pinned {
                            ui.badge("Pinned", Tone::Accent);
                        }
                        ui.badge("Live", Tone::Success);
                    });
                });
            });
            let resp = zone.response();
            if std::mem::take(&mut s.force_open) {
                ui.open_context_menu(resp.id, lumen::vec2(260.0, 126.0));
            }
            ui.context_menu(&resp, |ui| {
                ui.menu_label("Release v2.4.1");
                ui.menu_item("file-text", "View build log").shortcut("⌘L");
                ui.menu_item("copy", "Copy commit SHA").shortcut("⌘C");
                if ui.menu_item("pin", "Pin to overview").checked(s.pinned).clicked() {
                    s.pinned = !s.pinned;
                }
                ui.menu_separator();
                if ui.menu_item("rotate-ccw", "Roll back release…").danger().clicked() {
                    ui.toast(Toast::new("Rollback queued").tone(Tone::Warning));
                }
            });
        }
        "confirm-popover" => {
            ui.column().fill_width().max_width(440.0).show(|ui| {
                ui.card().pad(16.0).show(|ui| {
                    ui.row().fill_width().gap(12).align(Align::Center).show(|ui| {
                        ui.column().gap(2).fill_width().show(|ui| {
                            ui.label("Production API key").strong();
                            ui.label(if s.revoked { "Revoked · create a new key to restore access" } else { "rk_live_9d2f…8a1c · last used 2m ago" }).small().muted();
                        });
                        let r = ui.button(if s.revoked { "Revoked" } else { "Revoke" }).danger().small().disabled(s.revoked).show();
                        if std::mem::take(&mut s.force_open) {
                            ui.set_popover_open(r.id, true);
                        }
                        if ui
                            .confirm_popover(&r, "Revoke this API key?")
                            .description("Clients authenticating with rk_live_9d2f will be rejected immediately.")
                            .confirm_label("Revoke key")
                            .confirmed()
                        {
                            s.revoked = true;
                            ui.toast(Toast::new("API key revoked").danger());
                        }
                    });
                });
            });
        }
        "toast" => {
            ui.row().gap(10).show(|ui| {
                if ui.button("Saved").icon("check").clicked() {
                    ui.toast(Toast::new("Changes saved").description("Your profile is up to date.").success());
                }
                if ui.button("Failed").icon("x").clicked() {
                    ui.toast(Toast::new("Deploy failed").description("Build step exited with code 1.").danger());
                }
                if ui.button("Info").icon("info").clicked() {
                    ui.toast(Toast::new("New version available").tone(Tone::Info));
                }
            });
        }
        "chips" => {
            ui.column().gap(12).align(Align::Center).show(|ui| {
                ui.row().gap(8).show(|ui| {
                    for (i, l) in ["arm64", "gpu", "docker", "nightly", "macos-14"].iter().enumerate() {
                        ui.chip(l, &mut s.labels[i]);
                    }
                });
                let n = s.labels.iter().filter(|x| **x).count();
                ui.label(format!("{n} of 5 labels: runners must match all of them.")).muted();
            });
        }
        "stepper" => {
            ui.column().gap(16).fill_width().max_width(320.0).show(|ui| {
                ui.row().fill_width().align(Align::Center).show(|ui| {
                    ui.label("Concurrent jobs");
                    ui.spacer();
                    ui.stepper("jobs", &mut s.jobs, 1, 16);
                });
                ui.label("Click − and +, or focus a button and use the arrow keys. The number rolls the way it moved.").muted();
            });
        }
        "toggle-group" => {
            ui.column().gap(12).fill_width().max_width(360.0).show(|ui| {
                ui.label("Release window").strong();
                ui.toggle_group("slots", &mut s.slot, &[(0, "09:00"), (1, "11:30"), (2, "14:00"), (3, "16:30")]);
            });
        }
        "avatar" => {
            ui.column().gap(18).align(Align::Center).show(|ui| {
                ui.row().gap(12).align(Align::Center).show(|ui| {
                    ui.avatar("JB", 0, 40.0, Some(true));
                    ui.avatar("PN", 1, 40.0, Some(false));
                    ui.avatar("TR", 2, 40.0, None);
                    ui.avatar("LF", 3, 32.0, None);
                    ui.avatar("SW", 4, 24.0, None);
                });
                ui.avatar_group(&["JB", "PN", "TR", "LF", "SW", "DM", "AK"], 4, 32.0);
            });
        }
        "tooltip" => {
            ui.row().gap(8).show(|ui| {
                ui.icon_button("bold").ghost().tooltip_shortcut("Bold", "⌘B");
                ui.icon_button("italic").ghost().tooltip_shortcut("Italic", "⌘I");
                ui.icon_button("underline").ghost().tooltip_shortcut("Underline", "⌘U");
                ui.icon_button("link").ghost().tooltip("Insert link");
            });
            ui.label("Hover an icon. Tooltips open after a delay, then instantly while you move along.").muted();
        }
        "card" => {
            ui.column().fill_width().max_width(560.0).show(|ui| {
                ui.card().title("Notifications").description("Choose what reaches you.").show(|ui| {
                    ui.settings(|rows| {
                        rows.row("Deploy failures", "When a production deploy fails.", |ui| {
                            ui.switch("", &mut s.on[0]);
                        });
                        rows.row("Weekly digest", "A Monday summary.", |ui| {
                            ui.switch("", &mut s.on[1]);
                        });
                    });
                });
            });
        }
        "badge" => {
            ui.row().gap(8).show(|ui| {
                ui.badge("Neutral", Tone::Neutral);
                ui.badge("Accent", Tone::Accent);
                ui.badge("Live", Tone::Success);
                ui.badge("Degraded", Tone::Warning);
                ui.badge("Down", Tone::Danger);
                ui.badge("Beta", Tone::Info);
            });
            ui.row().gap(20).show(|ui| {
                ui.status("Operational", Tone::Success);
                ui.status("Partial outage", Tone::Warning);
                ui.status("Major outage", Tone::Danger);
            });
        }
        "progress" => {
            ui.column().fill_width().max_width(360.0).gap(18).show(|ui| {
                s.t += 1;
                if s.t % 90 == 0 {
                    s.progress = if s.progress > 0.9 { 0.1 } else { s.progress + 0.2 };
                }
                ui.request_repaint_after(0.05);
                ui.label(format!("Uploading… {:.0}%", s.progress * 100.0)).small().secondary();
                ui.progress(Some(s.progress));
                ui.label("Indeterminate").small().secondary();
                ui.scope("indet", |ui| ui.progress(None));
                ui.row().gap(16).align(Align::Center).show(|ui| {
                    ui.spinner(20.0);
                    ui.label("Syncing").secondary();
                });
            });
        }
        "alert" => {
            ui.column().fill_width().max_width(480.0).gap(12).show(|ui| {
                ui.alert("Scheduled maintenance")
                    .description("eu-central upgrades at 02:00 UTC.")
                    .warning()
                    .banner()
                    .dismissible(&mut s.banner_open);
                if ui
                    .alert("Certificate renewal delayed")
                    .description("DNS propagation timed out for acme-challenge.northwind.io.")
                    .danger()
                    .action("Retry")
                    .clicked()
                {
                    ui.toast(Toast::new("Renewal queued").tone(Tone::Info));
                }
                if !s.banner_open {
                    if ui.button("Restore maintenance banner").ghost().small().clicked() {
                        s.banner_open = true;
                    }
                }
            });
        }
        "skeleton" => {
            ui.column().fill_width().max_width(420.0).gap(12).show(|ui| {
                ui.row().fill_width().align(Align::Center).show(|ui| {
                    ui.label("Loading state").strong();
                    ui.spacer();
                    ui.switch("", &mut s.skeleton_loading);
                });
                ui.card().pad(16.0).show(|ui| {
                    if s.skeleton_loading {
                        ui.row().fill_width().gap(12).align(Align::Center).show(|ui| {
                            ui.skeleton_circle(40.0);
                            ui.column().fill_width().gap(6).show(|ui| {
                                ui.skeleton(140.0, 14.0);
                                ui.skeleton(200.0, 11.0);
                            });
                            ui.skeleton(56.0, 20.0).radius(10.0);
                        });
                        ui.divider();
                        ui.skeleton_lines(3);
                    } else {
                        ui.row().fill_width().gap(12).align(Align::Center).show(|ui| {
                            ui.avatar("LF", 3, 40.0, Some(true));
                            ui.column().fill_width().gap(2).show(|ui| {
                                ui.label("Lena Fischer").strong();
                                ui.label("Runtime · Berlin · 18 commits this week").small().secondary();
                            });
                            ui.badge("Active", Tone::Success);
                        });
                        ui.divider();
                        ui.label("Shipped per-corner SDF gradients and reduced idle wakeups across WebGPU surfaces.").secondary();
                    }
                });
            });
        }
        "empty-state" => {
            ui.column().fill_width().max_width(440.0).show(|ui| {
                ui.empty_state("inbox", "No self-hosted runners yet")
                    .description("Install the lightweight daemon on any Linux or macOS machine to start picking up builds.")
                    .action(|ui| {
                        if ui.button("Copy install script").primary().icon("terminal").clicked() {
                            ui.set_copied_text("curl -fsSL https://lumen.dev/agent.sh | sh".into());
                            ui.toast(Toast::new("Install command copied").success());
                        }
                        ui.button("Read docs").ghost();
                    });
            });
        }
        "breadcrumb" => {
            const TRAIL: [&str; 4] = ["Northwind", "Infrastructure", "Frankfurt", "linux-arm64-02"];
            const DEEP: [&str; 6] = ["Workspace", "Platform", "Networking", "Ingress", "Certificates", "wildcard-prod"];
            ui.column().fill_width().max_width(480.0).gap(16).align(Align::Start).show(|ui| {
                let depth = s.crumb_depth.clamp(1, TRAIL.len());
                if let Some(i) = ui.breadcrumb("main", &TRAIL[..depth]).root_icon("layers").show() {
                    s.crumb_depth = i + 1;
                }
                if depth < TRAIL.len() {
                    if ui.button(format!("Open “{}” →", TRAIL[depth])).ghost().small().clicked() {
                        s.crumb_depth += 1;
                    }
                }
                ui.divider();
                ui.column().gap(4).align(Align::Start).show(|ui| {
                    ui.label("Collapsed middle crumbs (click …)").small().muted();
                    if let Some(i) = ui.breadcrumb("deep", &DEEP).max_items(4).show() {
                        ui.toast(Toast::new(format!("Navigated to {}", DEEP[i])));
                    }
                });
            });
        }
        "pagination" => {
            ui.column().fill_width().max_width(500.0).gap(16).show(|ui| {
                ui.card().pad(14.0).show(|ui| {
                    ui.column().fill_width().gap(8).show(|ui| {
                        let base = s.page * 10;
                        for k in 0..2 {
                            let idx = base + k + 1;
                            if idx <= 174 {
                                ui.row().fill_width().align(Align::Center).show(|ui| {
                                    ui.label(format!("Build #{:04}", 1200 + idx)).strong();
                                    ui.spacer();
                                    ui.badge("Passed", Tone::Success);
                                });
                            }
                        }
                    });
                });
                ui.pagination("builds", &mut s.page, 18).summary(174, 10);
            });
        }
        "steps" => {
            const WIZ: [(&str, &str); 3] = [("Build", "Image signed"), ("Canary", "10% traffic"), ("Promote", "All regions")];
            ui.column().fill_width().max_width(500.0).gap(16).show(|ui| {
                ui.steps("rollout", &mut s.wizard_step, &WIZ);
                let desc = [
                    "Container digest sha256:9f4c…81a2 verified and pushed to registry.",
                    "Canary error rate is 0.01% across 2,400 requests in eu-central.",
                    "Ready to shift 100% of production traffic across all 4 regions.",
                ][s.wizard_step];
                ui.card().pad(14.0).show(|ui| {
                    ui.row().fill_width().gap(12).align(Align::Center).show(|ui| {
                        ui.column().fill_width().show(|ui| {
                            ui.label(desc).secondary();
                        });
                        if s.wizard_step > 0 && ui.button("Back").ghost().small().clicked() {
                            s.wizard_step -= 1;
                        }
                        if s.wizard_step + 1 < WIZ.len() {
                            if ui.button("Continue").primary().small().clicked() {
                                s.wizard_step += 1;
                            }
                        } else if ui.button("Finish").primary().small().clicked() {
                            ui.toast(Toast::new("Rollout complete").success());
                        }
                    });
                });
            });
        }
        "kbd" => {
            ui.column().fill_width().max_width(420.0).gap(14).show(|ui| {
                ui.card().pad(14.0).show(|ui| {
                    ui.column().fill_width().gap(10).show(|ui| {
                        for (label, chord) in [
                            ("Command palette", &["⌘", "K"][..]),
                            ("Deploy to production", &["⌘", "Shift", "D"][..]),
                            ("Filter current view", &["/"][..]),
                        ] {
                            ui.row().fill_width().align(Align::Center).show(|ui| {
                                ui.label(label);
                                ui.spacer();
                                ui.shortcut_keys(chord);
                            });
                        }
                    });
                });
                ui.row().fill_width().gap(8).align(Align::Center).justify(Justify::Center).show(|ui| {
                    ui.label("Try holding arrow keys:").small().muted();
                    ui.keycap("←").large().live_key(lumen::Key::ArrowLeft);
                    ui.keycap("↑").large().live_key(lumen::Key::ArrowUp);
                    ui.keycap("↓").large().live_key(lumen::Key::ArrowDown);
                    ui.keycap("→").large().live_key(lumen::Key::ArrowRight);
                });
            });
        }
        "radio-cards" => {
            ui.column().fill_width().max_width(440.0).gap(10).show(|ui| {
                ui.radio_card(&mut s.tier, 0, "Shared CPU")
                    .icon("cpu")
                    .badge("$8 / mo")
                    .description("2 vCPU · 2 GB RAM · bursts up to 100% for hobby workloads.");
                ui.radio_card(&mut s.tier, 1, "Performance ARM64")
                    .icon("zap")
                    .badge("$32 / mo")
                    .description("8 dedicated Ampere cores · 16 GB ECC · NVMe local scratch.");
                ui.radio_card(&mut s.tier, 2, "Dedicated Bare Metal")
                    .icon("server")
                    .badge("$180 / mo")
                    .description("32 cores · 128 GB RAM · isolated single-tenant hardware.");
            });
        }
        "rating" => {
            ui.column().gap(14).align(Align::Center).show(|ui| {
                ui.card().pad(18.0).show(|ui| {
                    ui.column().gap(10).align(Align::Center).show(|ui| {
                        ui.label("How smooth did the release feel?").strong();
                        ui.rating("release", &mut s.stars).size(22.0);
                        let note = ["No rating yet", "Rough", "Needs work", "Solid", "Smooth", "Effortless"][s.stars as usize];
                        ui.label(format!("{} / 5 · {note}", s.stars)).small().muted();
                    });
                });
            });
        }
        "range-slider" => {
            ui.column().fill_width().max_width(380.0).gap(18).show(|ui| {
                let txt = format!("${:.0} – ${:.0}", s.price_band.0, s.price_band.1);
                ui.range_slider("Instance budget", &mut s.price_band, 0.0..=120.0).step(5.0).readout(txt).fill_width();
                let mut lat = (12.0f32, 68.0f32);
                let ltxt = format!("{:.0}–{:.0} ms", lat.0, lat.1);
                ui.range_slider("Target p95 window", &mut lat, 0.0..=100.0).readout(ltxt).fill_width();
            });
        }
        "shortcut-recorder" => {
            ui.column().fill_width().max_width(360.0).gap(12).show(|ui| {
                ui.shortcut_recorder("deploy-key", &mut s.deploy_chord)
                    .label("Deploy production shortcut")
                    .fill_width();
                ui.label("Click the field and press any modifier + key chord (Backspace clears).").small().muted();
            });
        }
        "split-button" => {
            ui.row().gap(12).align(Align::Center).show(|ui| {
                if ui
                    .split_button("Deploy production")
                    .primary()
                    .icon("rocket")
                    .show(|ui| {
                        ui.menu_label("Target");
                        if ui.menu_item("zap", "Deploy canary (10%)").shortcut("⌘⇧C").clicked() {
                            ui.toast(Toast::new("Canary rollout started").tone(Tone::Accent));
                        }
                        if ui.menu_item("play", "Dry-run validation").clicked() {
                            ui.toast(Toast::new("Validation passed").success());
                        }
                        ui.menu_separator();
                        ui.menu_item("globe", "Promote to all regions");
                    })
                    .clicked()
                {
                    ui.toast(Toast::new("Production deploy triggered").success());
                }
                ui.button("Deploying").primary().loading(true);
            });
        }
        "hold-button" => {
            ui.column().gap(14).align(Align::Center).show(|ui| {
                ui.row().gap(12).align(Align::Center).show(|ui| {
                    if ui.hold_button("Hold to purge cache", 0.9).icon("trash-2").confirmed() {
                        ui.toast(Toast::new("Edge cache purged").danger());
                    }
                    ui.copy_button("Copy token", "rk_live_9d2f4b8c1e7a");
                    let label = if s.busy_deploy { "Syncing…" } else { "Sync state" };
                    if ui.button(label).secondary().icon("zap").loading(s.busy_deploy).clicked() {
                        s.busy_deploy = true;
                    }
                });
                ui.label("Press and hold the red button for 0.9 s — releasing early springs back safely.").small().muted();
            });
        }
        "dropzone" => {
            ui.column().fill_width().max_width(420.0).show(|ui| {
                if ui
                    .dropzone("artifacts", "Drop build artifacts or click to add")
                    .hint("WASM, dSYM or tar.zst up to 50 MB")
                    .files(&mut s.uploads)
                    .clicked()
                {
                    let n = s.uploads.len() + 1;
                    s.uploads.push((format!("bundle-{n}.tar.zst"), "1.8 MB".into()));
                }
            });
        }
        "toolbar" => {
            ui.column().gap(14).align(Align::Center).show(|ui| {
                ui.toolbar("editor", |tb| {
                    tb.toggle("bold", "Bold (⌘B)", &mut s.fmt_flags[0]);
                    tb.toggle("italic", "Italic (⌘I)", &mut s.fmt_flags[1]);
                    tb.toggle("underline", "Underline (⌘U)", &mut s.fmt_flags[2]);
                    tb.toggle("code", "Inline code", &mut s.fmt_flags[3]);
                    tb.separator();
                    tb.action("align-left", "Align left");
                    tb.action("align-center", "Center");
                    tb.separator();
                    if tb.primary("Share") {
                        tb.ui.toast(Toast::new("Link copied").success());
                    }
                });
                ui.label("Floating raised toolbar with spring-pressed icon toggles and primary CTA.").small().muted();
            });
        }
        "status-bar" => {
            ui.column().fill_width().max_width(520.0).gap(12).show(|ui| {
                ui.status_bar(|bar| {
                    bar.status("Connected", Tone::Success);
                    if bar.item("git-branch", "ethereal*") {
                        bar.ui.toast(Toast::new("Branch: ethereal"));
                    }
                    bar.item("wifi", "14 ms");
                    bar.spacer();
                    bar.text("Ln 142, Col 18");
                    bar.text("UTF-8");
                    bar.text("Rust");
                });
            });
        }
        "meter" => {
            ui.column().fill_width().max_width(440.0).show(|ui| {
                ui.card().pad(16.0).show(|ui| {
                    ui.meter(
                        "storage",
                        100.0,
                        &[("Artifacts", 42.0), ("Build cache", 21.5), ("Logs", 12.0), ("Snapshots", 6.5)],
                    )
                    .label("Workspace storage")
                    .unit("GB");
                });
            });
        }
        "calendar" => {
            ui.row().gap(32).align(Align::Start).show(|ui| {
                ui.calendar(&mut s.date);
                ui.calendar_range(&mut s.span);
            });
        }
        "date-picker" => {
            ui.row().gap(16).align(Align::Start).show(|ui| {
                let r = ui.date_picker("Due date", &mut s.date);
                if std::mem::take(&mut s.force_open) {
                    ui.set_popover_open(r.id, true);
                }
                ui.date_range_picker("Stay", &mut s.span);
            });
        }
        "combobox" => {
            ui.column().gap(12).fill_width().max_width(320.0).show(|ui| {
                let opts: Vec<(usize, &str)> = ZONES.iter().copied().enumerate().collect();
                ui.combobox(&mut s.tz, &opts).label("Time zone").placeholder("Search time zones").fill_width();
                let now = s.tz.map(|i| ZONES[i]).unwrap_or("none");
                ui.label(format!("Reports run at 09:00 in {now}.")).muted();
            });
        }
        "multi-select" => {
            ui.column().gap(12).fill_width().max_width(380.0).show(|ui| {
                let opts: Vec<(usize, &str)> = PEOPLE.iter().copied().enumerate().collect();
                ui.multi_select(&mut s.reviewers, &opts).label("Reviewers").placeholder("Add reviewers").fill_width();
                ui.label(format!("{} of 8 required to approve.", s.reviewers.len().min(2))).muted();
            });
        }
        "date-range-picker" => {
            ui.column().gap(12).align(Align::Start).show(|ui| {
                let r = ui.date_range_picker("Maintenance window", &mut s.span);
                if std::mem::take(&mut s.force_open) {
                    ui.set_popover_open(r.id, true);
                }
                let msg = match s.span {
                    Some((a, b)) if a != b => format!("{} nights scheduled · status page updates automatically.", b.to_days() - a.to_days()),
                    Some((a, _)) => format!("Single day: {}.", a.short()),
                    None => "No window scheduled.".into(),
                };
                ui.label(msg).muted();
            });
        }
        "time-picker" => {
            ui.row().gap(16).align(Align::Start).show(|ui| {
                let r = ui.time_picker("Deploy at (UTC)", &mut s.deploy_time).show();
                if std::mem::take(&mut s.force_open) {
                    ui.set_popover_open(r.id, true);
                }
                ui.time_picker("Quiet hours end", &mut s.quiet_end).twelve_hour(true).step(15);
            });
        }
        "color-picker" => {
            ui.row().gap(16).align(Align::Start).show(|ui| {
                let r = ui.color_picker("Brand color", &mut s.brand_color).show();
                if std::mem::take(&mut s.force_open) {
                    ui.set_popover_open(r.id, true);
                }
                ui.color_picker("Spotlight glow", &mut s.glow_color).alpha(true);
            });
        }
        "chart-line" => {
            ui.line_chart("rev", &MONTHS, vec![Series::new("Revenue", revenue()), Series::new("Costs", costs())]).smooth(true).height(260.0).format(|v| if v == 0.0 { "$0".into() } else { format!("${}k", lumen::widgets::chart::compact(v)) });
        }
        "chart-area" => {
            let v: Vec<f32> = (0..30).map(|i| 40.0 + 18.0 * ((i as f32) * 0.45).sin() + i as f32 * 1.6).collect();
            let labels: Vec<String> = (1..=30).map(|d| format!("Nov {d}")).collect();
            let l: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
            ui.area_chart("visitors", &l, vec![Series::new("Visitors", v)]).smooth(true).height(260.0);
        }
        "chart-bar" => {
            ui.bar_chart(
                "week",
                &DAYS,
                vec![Series::new("Web", vec![12.0, 18.0, 15.0, 22.0, 26.0, 9.0, 7.0]), Series::new("iOS", vec![8.0, 10.0, 9.0, 14.0, 15.0, 11.0, 10.0]), Series::new("Android", vec![5.0, 7.0, 8.0, 9.0, 11.0, 6.0, 5.0])],
            )
            .stacked()
            .height(260.0);
        }
        "donut" => {
            ui.donut("traffic", &[("Search", 4200.0), ("Direct", 2600.0), ("Social", 1500.0), ("Referral", 900.0), ("Email", 600.0)]);
        }
        "gauge" => {
            ui.row().gap(24).show(|ui| {
                ui.gauge("cpu", 0.38, "CPU");
                ui.gauge("mem", 0.71, "Memory");
                ui.gauge("disk", 0.92, "Disk");
            });
        }
        "sparkline" => {
            ui.row().fill_width().gap(16).show(|ui| {
                for (label, value, delta, trend) in [
                    ("Revenue", "$118k", 12.4, revenue()),
                    ("Active users", "8,214", 4.1, vec![60.0, 62.0, 61.0, 66.0, 70.0, 69.0, 74.0, 78.0]),
                    ("Churn", "2.1%", -0.6, vec![3.0, 2.8, 2.9, 2.6, 2.5, 2.3, 2.2, 2.1]),
                ] {
                    ui.card().show(|ui| {
                        ui.stat(label, value, Some(delta), Some(&trend));
                    });
                }
            });
        }
        "accordion" => {
            ui.column().fill_width().max_width(500.0).show(|ui| {
                ui.accordion("faq", &mut s.acc_open)
                    .item_with("Zero-copy glyph atlas", Some("sparkles"), Some("GPU"), |ui| {
                        ui.label("Shaped glyphs stream straight into a single R8 coverage texture — no DOM, CSS or OS native controls in the loop.").secondary();
                    })
                    .item_with("Spring-driven micro-motion", Some("activity"), Some("120 Hz"), |ui| {
                        ui.label("Every disclosure height, selection pill and press squash settles analytically via critically and under-damped springs.").secondary();
                    })
                    .item_with("Four runtime looks", Some("layers"), Some("Live"), |ui| {
                        ui.label("Switch between Lumen, macOS, Windows 11 and GNOME Adwaita at runtime; tokens blend in perceptual OKLab.").secondary();
                    })
                    .show();
            });
        }
        "table" => {
            let mut rows = vec![
                (0usize, "edge-proxy", "eu-central", 4.2f32, "Healthy", Tone::Success),
                (1usize, "auth-gateway", "us-east", 11.8f32, "Healthy", Tone::Success),
                (2usize, "search-indexer", "eu-central", 7.9f32, "Healthy", Tone::Success),
                (3usize, "billing-worker", "ap-northeast", 48.6f32, "Degraded", Tone::Warning),
            ];
            if let Some((col, dir)) = s.table_sort {
                rows.sort_by(|a, b| {
                    let ord = match col {
                        0 => a.1.cmp(b.1),
                        1 => a.2.cmp(b.2),
                        2 => a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal),
                        _ => std::cmp::Ordering::Equal,
                    };
                    if dir == SortDir::Desc { ord.reverse() } else { ord }
                });
            }
            ui.column().fill_width().max_width(580.0).gap(8).show(|ui| {
                let mut tbl = ui
                    .table("services")
                    .columns([
                        TableCol::new("Service").fill(1.45).sortable(),
                        TableCol::new("Region").fill(1.05).sortable(),
                        TableCol::new("p95 latency").width(104.0).sortable().right(),
                        TableCol::new("Status").width(96.0).right(),
                    ])
                    .sort_by(&mut s.table_sort)
                    .selection(&mut s.table_sel);
                for (rid, svc, reg, ms, st, tone) in rows {
                    tbl = tbl.row(rid, [
                        TableCell::strong(svc),
                        TableCell::muted(reg),
                        TableCell::mono(format!("{ms:.1} ms")),
                        TableCell::badge(st, tone),
                    ]);
                }
                tbl.show();
            });
        }
        "tree" => {
            let tree = [
                TreeNode::branch("crates", "crates", [
                    TreeNode::branch("lumen-core", "lumen-core", [
                        TreeNode::branch("src", "src", [
                            TreeNode::leaf("lib.rs", "lib.rs").icon("file-code").meta("1.4 KB"),
                            TreeNode::leaf("theme.rs", "theme.rs").icon("file-code").meta("24.8 KB"),
                            TreeNode::leaf("paint.rs", "paint.rs").icon("file-code").meta("18.2 KB"),
                        ]),
                        TreeNode::leaf("core-toml", "Cargo.toml").icon("file-text").meta("620 B"),
                    ]),
                    TreeNode::branch("lumen-wgpu", "lumen-wgpu", [
                        TreeNode::leaf("shader.wgsl", "shader.wgsl").icon("file-code").meta("6.1 KB"),
                    ]),
                ]),
                TreeNode::leaf("readme", "README.md").icon("file-text").meta("4.9 KB"),
            ];
            ui.column().fill_width().max_width(420.0).show(|ui| {
                ui.tree("workspace", &tree, &mut s.tree_open, &mut s.tree_sel).show();
            });
        }
        "timeline" => {
            let events = [
                TimelineItem::new("Promoted v0.4.2 to production", "2m ago")
                    .icon("rocket")
                    .tone(Tone::Success)
                    .tag("9f3c1a2")
                    .description("All 14 health checks green across Frankfurt and Virginia."),
                TimelineItem::new("Canary traffic shifted to 25%", "14m ago")
                    .icon("git-branch")
                    .tone(Tone::Accent)
                    .tag("canary")
                    .description("Error budget nominal · p99 latency held steady at 8.4 ms."),
                TimelineItem::new("Rebalanced shard worker-03", "41m ago")
                    .icon("triangle-alert")
                    .tone(Tone::Warning)
                    .description("Memory pressure cleared automatically before rollout."),
            ];
            ui.column().fill_width().max_width(500.0).show(|ui| {
                ui.card().pad(18.0).show(|ui| {
                    ui.timeline("deploys", &events);
                });
            });
        }
        "code-block" => {
            let snippet = r#"pub fn deploy(ui: &mut Ui, s: &mut State) {
    ui.heading("Production rollout");
    ui.slider("Canary", &mut s.traffic, 0.0..=100.0);
    if ui.button("Promote build").primary().icon("rocket").clicked() {
        ui.toast(Toast::new("v0.4.2 live").success());
    }
}"#;
            ui.column().fill_width().max_width(580.0).show(|ui| {
                ui.code_block("rollout_rs", snippet)
                    .title("examples/rollout.rs")
                    .language("rust")
                    .highlight_lines(&[4, 5])
                    .show();
            });
        }
        "json-viewer" => {
            let payload = r#"{
  "service": "edge-proxy",
  "version": "0.4.2",
  "healthy": true,
  "replicas": 6,
  "regions": ["eu-central-1", "us-east-1"],
  "limits": { "cpu": "2000m", "memory_mb": 4096 }
}"#;
            ui.column().fill_width().max_width(520.0).show(|ui| {
                ui.json_viewer("manifest", payload).title("release-manifest.json").default_depth(2).show();
            });
        }
        "key-value" => {
            ui.column().fill_width().max_width(580.0).show(|ui| {
                ui.key_value_editor("env_editor", &mut s.env_vars).show();
            });
        }
        "log-viewer" => {
            let logs = [
                LogEntry::new("02:14:01.10", LogLevel::Info, "wgpu::hal", "Adapter Apple M3 Max (Metal) acquired"),
                LogEntry::new("02:14:01.12", LogLevel::Info, "lumen::atlas", "Glyph cache warmed: 312 glyphs in 1.2 ms"),
                LogEntry::new("02:14:02.44", LogLevel::Warn, "edge::tls", "OCSP staple expires in 18m; refreshing"),
                LogEntry::new("02:14:03.01", LogLevel::Info, "edge::router", "Matched /v1/inference -> eu-central (4.1 ms)"),
                LogEntry::new("02:14:04.88", LogLevel::Error, "worker::shard", "Replica 03 heartbeat timed out (1500 ms)"),
                LogEntry::new("02:14:05.12", LogLevel::Info, "worker::shard", "Replica 03 rejoined cluster cleanly"),
            ];
            ui.column().fill_width().max_width(640.0).show(|ui| {
                ui.log_viewer("console", &logs).height(166.0).show();
            });
        }
        "progress-ring" => {
            let rings = [
                RingSpec::new("Compute", 0.84, "67.2 / 80 vCPU"),
                RingSpec::new("Memory", 0.62, "99.2 / 160 GB"),
                RingSpec::new("SLO budget", 0.94, "99.96% uptime"),
            ];
            ui.row().gap(36).align(Align::Center).show(|ui| {
                ui.activity_rings("cluster_rings", &rings, 132.0);
                ui.divider();
                ui.progress_ring("cache_hit", 0.92).size(104.0).thickness(9.0).label("Cache hit").show();
            });
        }
        "bar-list" => {
            let routes = [
                BarListItem::new("/v1/chat/completions", 48200.0),
                BarListItem::new("/v1/embeddings", 29400.0),
                BarListItem::new("/v1/models", 16800.0),
                BarListItem::new("/v1/files/upload", 9100.0),
                BarListItem::new("/v1/billing/usage", 4300.0),
            ];
            ui.column().fill_width().max_width(500.0).show(|ui| {
                ui.card().title("Top edge endpoints").pad(16.0).show(|ui| {
                    ui.bar_list("routes", &routes);
                });
            });
        }
        "heatmap" => {
            let rows_lbl = ["Mon", "Tue", "Wed", "Thu", "Fri"];
            let cols_lbl = ["00h", "02h", "04h", "06h", "08h", "10h", "12h", "14h", "16h", "18h", "20h", "22h"];
            let mut vals = Vec::with_capacity(5 * 12);
            for r in 0..5 {
                for c in 0..12 {
                    let peak = 1.0 - ((c as f32 - 6.5).abs() / 6.5).min(1.0);
                    let day_boost = [0.85, 1.0, 1.08, 0.96, 0.78][r];
                    vals.push((120.0 + 840.0 * peak * peak * day_boost).round());
                }
            }
            ui.column().fill_width().max_width(560.0).show(|ui| {
                ui.card().title("Request density (req/s)").pad(16.0).show(|ui| {
                    ui.heatmap("req_heat", 5, 12, &vals)
                        .row_labels(&rows_lbl)
                        .col_labels(&cols_lbl)
                        .unit("req/s")
                        .show();
                });
            });
        }
        "radar-chart" => {
            let axes = ["Throughput", "Latency", "Memory", "Startup", "Binary size", "Battery"];
            let s1 = Series::new("lumen 0.4", vec![94.0, 92.0, 88.0, 95.0, 86.0, 90.0]);
            let s2 = Series::new("WebView", vec![58.0, 52.0, 44.0, 49.0, 62.0, 55.0]);
            ui.column().fill_width().max_width(520.0).show(|ui| {
                ui.radar_chart("bench_radar", &axes, vec![s1, s2]).height(260.0).show();
            });
        }
        "scatter-chart" => {
            let metal = ScatterSeries::new("Metal ARM64", vec![
                ScatterPoint::bubble(1.8, 142.0, 7.5, "M3 Max"),
                ScatterPoint::bubble(2.4, 118.0, 6.5, "M2 Pro"),
                ScatterPoint::bubble(3.1, 96.0, 5.5, "M1 Air"),
                ScatterPoint::bubble(1.5, 160.0, 8.5, "M4 Pro"),
            ]);
            let vulkan = ScatterSeries::new("Vulkan Linux", vec![
                ScatterPoint::bubble(2.2, 134.0, 7.0, "RX 7800"),
                ScatterPoint::bubble(3.6, 88.0, 6.0, "Arc A770"),
                ScatterPoint::bubble(4.4, 72.0, 5.0, "Xe iGPU"),
                ScatterPoint::bubble(1.9, 148.0, 8.0, "RTX 4080"),
            ]);
            ui.column().fill_width().max_width(580.0).show(|ui| {
                ui.scatter_chart("gpu_scatter", vec![metal, vulkan]).height(240.0).show();
            });
        }
        "funnel-chart" => {
            let stages = [
                ("Visited docs", 18400.0),
                ("Ran quickstart", 11800.0),
                ("Built first view", 6400.0),
                ("Shipped release", 2950.0),
            ];
            ui.column().fill_width().max_width(560.0).show(|ui| {
                ui.card().title("Developer activation funnel").pad(16.0).show(|ui| {
                    ui.funnel_chart("activation", &stages).height(205.0).show();
                });
            });
        }
        "candlestick" => {
            let candles = [
                Candle::new("Nov 1", 142.0, 148.5, 139.0, 146.2),
                Candle::new("Nov 2", 146.2, 151.0, 144.5, 149.8),
                Candle::new("Nov 3", 149.8, 150.4, 143.0, 144.6),
                Candle::new("Nov 4", 144.6, 147.0, 140.2, 141.5),
                Candle::new("Nov 5", 141.5, 149.2, 141.0, 148.4),
                Candle::new("Nov 6", 148.4, 155.0, 147.2, 153.6),
                Candle::new("Nov 7", 153.6, 158.4, 151.8, 156.9),
                Candle::new("Nov 8", 156.9, 157.5, 150.2, 151.8),
                Candle::new("Nov 9", 151.8, 156.0, 149.5, 155.1),
                Candle::new("Nov 10", 155.1, 162.4, 154.0, 160.8),
                Candle::new("Nov 11", 160.8, 163.5, 157.0, 158.4),
                Candle::new("Nov 12", 158.4, 166.2, 158.0, 165.0),
            ];
            ui.column().fill_width().max_width(600.0).show(|ui| {
                ui.candlestick_chart("ohlc", &candles).height(250.0).show();
            });
        }
        "kanban" => {
            let cols = [
                KanbanCol::new("Backlog", Tone::Neutral),
                KanbanCol::new("In progress", Tone::Accent),
                KanbanCol::new("Shipped", Tone::Success),
            ];
            ui.column().fill_width().max_width(660.0).show(|ui| {
                ui.kanban("sprint_board", &cols, &mut s.kanban_cards);
            });
        }
        "ai-reasoning" => {
            ui.column().fill_width().max_width(560.0).show(|ui| {
                ui.card().pad(16.0).show(|ui| {
                    ui.reasoning_block("cot_1", "Thought for 2.4s", Some("3 steps"), &mut s.cot_open, |ui| {
                        ui.label("Checked physical pixel snapping in Painter::snap_rect; verifying whether Fill children accumulate subpixel remainders across columns.").secondary();
                        ui.tool_call("tc1", &ToolCall::new("read_file", "crates/lumen-core/src/layout.rs:240-310", ToolStatus::Done).duration("18 ms"));
                        ui.tool_call("tc2", &ToolCall::new("cargo_test", "-p lumen-core layout::tests", ToolStatus::Done).duration("310 ms"));
                    });
                    ui.divider();
                    ui.label("All 14 flex layout tests pass; coordinates snap cleanly to physical pixels at 1.5× and 2.0× scale.").strong();
                });
            });
        }
        "ai-composer" => {
            ui.column().fill_width().max_width(560.0).gap(10).show(|ui| {
                let out = ui
                    .prompt_composer("main_composer", &mut s.ai_prompt, &mut s.ai_model)
                    .models(&["Claude 3.7 Sonnet", "Gemini 2.5 Pro", "o3-mini"])
                    .attachments(&mut s.ai_files)
                    .context_used(0.34)
                    .streaming(s.ai_streaming)
                    .show();
                if out.submitted {
                    s.ai_streaming = true;
                } else if out.stopped {
                    s.ai_streaming = false;
                } else if out.attach_clicked && s.ai_files.len() < 4 {
                    s.ai_files.push(format!("trace-{}.json", s.ai_files.len() + 1));
                }
            });
        }
        "ai-chat" => {
            let msgs = [
                ChatMessage::user("Why does lumen draw zero frames when idle?"),
                ChatMessage::assistant(
                    "lumen assistant",
                    "Springs evaluate in closed form and settle once displacement and velocity fall below 0.0005. With no active springs or timers, FrameOutput::repaint_after returns None and winit parks the thread.",
                )
                .time("Just now")
                .citations(&["anim.rs:84", "ui.rs:643"]),
            ];
            ui.column().fill_width().max_width(560.0).show(|ui| {
                ui.card().pad(18.0).show(|ui| {
                    ui.chat_thread("demo_chat", &msgs);
                });
            });
        }
        "diff-viewer" => {
            let diff = r#"@@ -268,7 +268,9 @@
 pub(crate) struct ScrollState {
     offset: f32,
     content: f32,
-    last_scroll_time: 0.0,
+    viewport: f32,
+    last_scroll_time: -10.0,
     target: Option<f32>,
 }"#;
            ui.column().fill_width().max_width(580.0).show(|ui| {
                ui.diff_viewer("scroll_fix", "crates/lumen-core/src/ui.rs", diff).show();
            });
        }
        "waterfall" => {
            let spans = [
                TraceSpan::new("POST /v1/chat/completions", "edge-proxy", 0.0, 148.0, 0),
                TraceSpan::new("verify_session_jwt", "auth-svc", 3.5, 14.2, 1),
                TraceSpan::new("pg.fetch_context", "postgres-primary", 18.0, 26.5, 1),
                TraceSpan::new("kv_cache.lookup", "redis-eu", 21.0, 6.8, 2),
                TraceSpan::new("model.forward_stream", "gpu-pool-a100", 46.0, 98.4, 1),
            ];
            ui.column().fill_width().max_width(620.0).show(|ui| {
                ui.waterfall("req_trace", &spans);
            });
        }
        "uptime-bars" => {
            let mut api = vec![UptimeStatus::Operational; 44];
            api[14] = UptimeStatus::Degraded;
            let mut cdn = vec![UptimeStatus::Operational; 44];
            cdn[29] = UptimeStatus::Degraded;
            cdn[30] = UptimeStatus::Outage;
            ui.column().fill_width().max_width(560.0).show(|ui| {
                ui.card().pad(18.0).show(|ui| {
                    ui.uptime_bars("u_api", "Inference API · eu-central", &api);
                    ui.divider();
                    ui.uptime_bars("u_cdn", "Artifact CDN · global", &cdn);
                });
            });
        }
        "rolling-number" => {
            ui.column().align(Align::Center).gap(16).show(|ui| {
                ui.card().pad(24.0).show(|ui| {
                    ui.label("Annual recurring revenue").small().secondary();
                    ui.rolling_number("mrr", s.mrr_val).prefix("$").suffix("USD").role(TextRole::Display).show();
                });
                ui.row().gap(8).show(|ui| {
                    if ui.button("− $350").clicked() {
                        s.mrr_val = (s.mrr_val - 350).max(0);
                    }
                    if ui.button("+ $1,250").primary().icon("plus").clicked() {
                        s.mrr_val += 1_250;
                    }
                    if ui.button("+ $10,000").clicked() {
                        s.mrr_val += 10_000;
                    }
                });
            });
        }
        "dock" => {
            let items: Vec<DockItem> = [
                ("compass", "Explorer"),
                ("terminal", "Terminal"),
                ("code", "Editor"),
                ("message-square", "Threads"),
                ("music", "Studio"),
                ("settings", "Settings"),
            ]
            .iter()
            .enumerate()
            .map(|(i, (ic, lbl))| DockItem::new(ic, *lbl).active(i == s.dock_active || i == 2))
            .collect();
            ui.column().align(Align::Center).gap(14).show(|ui| {
                if let Some(chosen) = ui.dock("app_dock", &items) {
                    s.dock_active = chosen;
                }
                ui.label("Move the pointer across the dock — neighboring icons magnify with a bouncy spring.").small().secondary();
            });
        }
        "waveform" => {
            let samples: Vec<f32> = (0..44)
                .map(|i| {
                    let x = i as f32 * 0.37;
                    (0.24 + 0.46 * x.sin().abs() + 0.28 * (x * 2.3 + 1.1).cos().abs()).clamp(0.15, 0.98)
                })
                .collect();
            ui.column().fill_width().max_width(540.0).show(|ui| {
                ui.waveform("voice_note", &samples, &mut s.audio_prog, &mut s.audio_playing)
                    .title("Architecture walkthrough · voice note")
                    .duration(42.0)
                    .show();
            });
        }
        "window-chrome" => {
            ui.set_look(s.win_look.look());
            if s.embed {
                for m in [Mode::Light, Mode::Dark] {
                    ui.override_color(m, lumen::Token::Background, Some(Look::lumen().neutrals(m).surface));
                }
            }
            ui.column().fill_width().max_width(520.0).align(Align::Center).gap(14).show(|ui| {
                let opts: Vec<(Platform, &str)> = Platform::ALL.iter().map(|p| (*p, p.name())).collect();
                ui.segmented(&mut s.win_look, &opts);
                ui.window_chrome("preview_win", "lumen — workspace")
                    .subtitle("ethereal")
                    .show(|ui| {
                        ui.row().fill_width().align(Align::Center).show(|ui| {
                            ui.label("Native window chrome").strong();
                            ui.spacer();
                            ui.badge(s.win_look.name(), Tone::Accent);
                        });
                        ui.label("Traffic lights on macOS, caption buttons on Windows 11, headerbar pill on GNOME, and frosted chrome on Lumen.").secondary();
                    });
            });
        }
        "split-pane" => {
            ui.column().fill_width().max_width(580.0).gap(8).show(|ui| {
                let cur_r = s.split_ratio;
                ui.split_pane("ide_split", &mut s.split_ratio).height(220.0).show(
                    |ui| {
                        ui.label("Explorer").strong();
                        ui.divider();
                        for name in ["src/lib.rs", "src/ui.rs", "src/paint.rs", "src/theme.rs"] {
                            ui.row().gap(6).align(Align::Center).show(|ui| {
                                let ic_c = ui.color(lumen::Token::AccentStrong);
                                ui.icon_leaf("file-code", 14.0, ic_c);
                                ui.label(name);
                            });
                        }
                    },
                    |ui| {
                        ui.row().fill_width().align(Align::Center).show(|ui| {
                            ui.label("Inspector").strong();
                            ui.spacer();
                            ui.badge(format!("{:.0}% / {:.0}%", cur_r * 100.0, (1.0 - cur_r) * 100.0), Tone::Neutral);
                        });
                        ui.label("Drag the center sash or double-click it to spring back to 50 / 50.").secondary();
                    },
                );
            });
        }
        "menu-bar" => {
            ui.column().fill_width().max_width(540.0).show(|ui| {
                ui.menu_bar("app_menubar", |bar| {
                    bar.menu("File", |ui| {
                        ui.menu_item("plus", "New workspace").shortcut("⌘N");
                        ui.menu_item("folder-open", "Open project…").shortcut("⌘O");
                        ui.menu_separator();
                        ui.menu_item("rocket", "Export release bundle").shortcut("⌘⇧E");
                    });
                    bar.menu("Edit", |ui| {
                        ui.menu_item("", "Undo").shortcut("⌘Z");
                        ui.menu_item("", "Redo").shortcut("⌘⇧Z");
                        ui.menu_separator();
                        ui.menu_item("copy", "Copy selection").shortcut("⌘C");
                    });
                    bar.menu("View", |ui| {
                        ui.menu_item("", "Toggle sidebar").shortcut("⌘B");
                        ui.menu_item("", "Command palette").shortcut("⌘K");
                    });
                    bar.menu("Build", |ui| {
                        ui.menu_item("play", "Run target").shortcut("⌘R");
                        ui.menu_item("flame", "Profile frame");
                    });
                });
            });
        }
        "terminal" => {
            ui.column().fill_width().max_width(600.0).show(|ui| {
                let lines_snapshot = s.term_lines.clone();
                if let Some(cmd) = ui.terminal("demo_term", &lines_snapshot, &mut s.term_in).title("zsh — lumen/crates").height(196.0).show() {
                    s.term_lines.push(TermLine::cmd(&cmd));
                    match cmd.as_str() {
                        "clear" => s.term_lines.clear(),
                        "help" => s.term_lines.push(TermLine::out("Built-ins: help, status, test, clear")),
                        "status" => s.term_lines.push(TermLine::ok("branch ethereal · clean · 0 idle wakeups")),
                        "test" => s.term_lines.push(TermLine::ok("119 passed in 0.04s")),
                        other => s.term_lines.push(TermLine::out(format!("executed: {other}"))),
                    }
                }
            });
        }
        "gantt" => {
            let weeks = ["W1", "W2", "W3", "W4", "W5", "W6"];
            let tasks = [
                GanttTask::new("Token & OKLab engine", 0.0, 1.8, 1.0).owner("JB").tone(Tone::Success),
                GanttTask::new("Single-shader wgpu pipeline", 1.0, 2.2, 1.0).owner("LF").tone(Tone::Success),
                GanttTask::new("Batch 2–4 component suite", 2.4, 2.4, 0.82).owner("PN").tone(Tone::Accent),
                GanttTask::new("WebGPU docs & posters", 4.0, 1.6, 0.55).owner("AK").tone(Tone::Info),
                GanttTask::milestone("v0.5 RC freeze", 5.6),
            ];
            ui.column().fill_width().max_width(620.0).show(|ui| {
                ui.gantt_chart("q4_roadmap", &weeks, &tasks);
            });
        }
        "treemap" => {
            let crates = [
                TreemapItem::new("lumen-core", 342.0).detail("Widgets, layout & springs"),
                TreemapItem::new("cosmic-text", 218.0).detail("HarfBuzz + shaping"),
                TreemapItem::new("wgpu-hal", 184.0).detail("Metal / Vulkan / WebGPU"),
                TreemapItem::new("lucide-ttf", 96.0).detail("Bundled vector icons"),
                TreemapItem::new("lumen-app", 64.0).detail("winit loop & clipboard"),
                TreemapItem::new("lumen-text", 48.0).detail("R8 glyph atlas"),
            ];
            ui.column().fill_width().max_width(580.0).show(|ui| {
                ui.card().title("Release WASM binary breakdown (KB)").pad(14.0).show(|ui| {
                    ui.treemap("wasm_size", &crates).unit("KB").height(215.0).show();
                });
            });
        }
        "markdown" => {
            let doc = r#"# Immediate-mode architecture
Every frame records a compact layout tree and emits `DrawData` triangles for **one unified shader**.
> Springs integrate in closed form, so motion stays exact at 60 Hz, 120 Hz or 240 Hz.
- Colors resolve strictly through `Token` in perceptual OKLab
- Zero idle wakeups when animations settle (`repaint_after == None`)
```rust
ui.button("Ship release").primary().icon("rocket");
```"#;
            ui.column().fill_width().max_width(560.0).show(|ui| {
                ui.card().pad(18.0).show(|ui| {
                    ui.markdown("arch_md", doc);
                });
            });
        }
        "property-grid" => {
            ui.column().fill_width().max_width(380.0).show(|ui| {
                ui.property_inspector("layer_props", |insp| {
                    insp.section("Transform");
                    let (mut px, mut py) = s.prop_pos;
                    insp.vec2_row("Position", &mut px, &mut py);
                    s.prop_pos = (px, py);
                    insp.number("Corner radius", &mut s.prop_radius, 0.0..=48.0, "px");
                    insp.section("Appearance");
                    insp.number("Opacity", &mut s.prop_opacity, 0.0..=100.0, "%");
                    insp.toggle("Clip sublayers", &mut s.prop_clip);
                });
            });
        }
        "flame-graph" => {
            let frames = [
                FlameSpan::new("Ui::run_frame", 0, 0.0, 4.80),
                FlameSpan::new("layout::solve", 1, 0.12, 1.35),
                FlameSpan::new("measure_text", 2, 0.24, 0.78),
                FlameSpan::new("place_flex", 2, 1.06, 0.38),
                FlameSpan::new("paint::tessellate", 1, 1.55, 1.92),
                FlameSpan::new("Painter::rrect", 2, 1.60, 0.84),
                FlameSpan::new("Painter::text", 2, 2.48, 0.94),
                FlameSpan::new("wgpu::submit", 1, 3.55, 1.18),
            ];
            ui.column().fill_width().max_width(620.0).gap(6).show(|ui| {
                ui.flame_graph("cpu_flame", &frames);
            });
        }
        "notification-list" => {
            ui.column().fill_width().max_width(540.0).show(|ui| {
                ui.notification_list("inbox", &mut s.notifs);
            });
        }
        "spotlight-card" => {
            ui.row().fill_width().max_width(600.0).gap(14).show(|ui| {
                ui.spotlight_card("sp_gpu").icon("zap").title("Single-pass SDF").show(|ui| {
                    ui.label("Rounded rects, borders, Gaussian shadows and atlas glyphs batch into one draw call.").secondary();
                });
                ui.spotlight_card("sp_oklab").icon("sparkles").title("Perceptual OKLab").show(|ui| {
                    ui.label("Switching themes or accents interpolates every token smoothly without muddy midpoints.").secondary();
                });
            });
        }
        "editable-text" => {
            ui.column().fill_width().max_width(480.0).gap(10).show(|ui| {
                ui.card().pad(16.0).show(|ui| {
                    ui.row().fill_width().align(Align::Center).show(|ui| {
                        ui.badge("Click any line to edit inline", Tone::Accent);
                        ui.spacer();
                        ui.status("Live", Tone::Success);
                    });
                    ui.editable_text("ed_h", &mut s.edit_heading).heading();
                    ui.editable_text("ed_s", &mut s.edit_summary).role(TextRole::Body);
                });
            });
        }
        "money-field" => {
            ui.column().fill_width().max_width(380.0).gap(14).show(|ui| {
                ui.money_field(&mut s.money_budget)
                    .label("Monthly cluster budget")
                    .switchable_currency(&mut s.money_cur)
                    .step(100.0)
                    .helper("Click the currency pill to cycle USD · EUR · GBP · JPY.")
                    .fill_width();
                ui.money_field(&mut s.money_alert)
                    .label("Soft spend alert")
                    .currency(Currency::Eur)
                    .step(50.0)
                    .fill_width();
            });
        }
        "masked-input" => {
            ui.column().fill_width().max_width(380.0).gap(14).show(|ui| {
                ui.masked_input(&mut s.phone_mask, "+1 (###) ###-####")
                    .label("On-call pager number")
                    .icon("phone")
                    .helper("Remaining template slots stay visible as you type.")
                    .fill_width();
                ui.masked_input(&mut s.license_mask, "AAAA-####-AAAA")
                    .label("Enterprise license key")
                    .icon("key-round")
                    .fill_width();
            });
        }
        "mention-input" => {
            let team = [
                MentionCandidate::new("lena", "Lena Fischer", "Graphics"),
                MentionCandidate::new("jonas", "Jonas Berg", "Runtime"),
                MentionCandidate::new("priya", "Priya Nair", "Design Systems"),
                MentionCandidate::new("aiko", "Aiko Kato", "WebGPU"),
            ];
            ui.column().fill_width().max_width(460.0).gap(8).show(|ui| {
                ui.mention_input(&mut s.mention_msg, &team)
                    .label("Release review comment")
                    .placeholder("Write a note — type @ to mention a teammate…");
            });
        }
        "step-slider" => {
            const MEM_STOPS: [&str; 5] = ["512 MB", "2 GB", "8 GB", "32 GB", "128 GB"];
            const RET_STOPS: [&str; 5] = ["7d", "14d", "30d", "90d", "1y"];
            ui.column().fill_width().max_width(440.0).show(|ui| {
                ui.card().pad(18.0).show(|ui| {
                    ui.step_slider("Worker memory tier", &mut s.mem_stop, &MEM_STOPS);
                    ui.divider();
                    ui.step_slider("Trace retention window", &mut s.ret_stop, &RET_STOPS);
                });
            });
        }
        "transfer-list" => {
            ui.column().fill_width().max_width(580.0).show(|ui| {
                ui.transfer_list("rbac_shuttle", &mut s.avail_scopes, &mut s.assigned_scopes)
                    .titles("Available scopes", "Assigned to role")
                    .height(152.0);
            });
        }
        "reorderable-list" => {
            ui.column().fill_width().max_width(500.0).gap(6).show(|ui| {
                ui.reorderable_list("pipeline_order", &mut s.reorder_items);
            });
        }
        "editor-tabs" => {
            ui.column().fill_width().max_width(580.0).gap(10).show(|ui| {
                let out = ui.editor_tabs("ide_tabs", &s.ed_tabs, &mut s.ed_active);
                if let Some(ci) = out.closed {
                    if s.ed_tabs.len() > 1 {
                        s.ed_tabs.remove(ci);
                        s.ed_active = s.ed_active.min(s.ed_tabs.len() - 1);
                    }
                }
                if out.add_clicked {
                    let n = s.ed_tabs.len() + 1;
                    s.ed_tabs.push(EditorTab::new(format!("u{n}"), format!("untitled-{n}.rs")).icon("file-code").dirty(true));
                    s.ed_active = s.ed_tabs.len() - 1;
                }
                let active_name = s.ed_tabs.get(s.ed_active).map(|t| t.label.as_str()).unwrap_or("main.rs");
                let snippet = match active_name {
                    "shader.wgsl" => "@fragment fn fs_main(in: Varyings) -> @location(0) vec4<f32> {\n    return sd_round_rect(in.local, in.half, in.radii);\n}",
                    "paint.rs" => "pub fn rrect(&mut self, r: Rect, rad: Corners, fill: Color) {\n    self.push_sdf_quad(r, rad, fill);\n}",
                    "theme.rs" => "pub fn blend(a: Color, b: Color, t: f32) -> Color {\n    a.mix_oklab(b, t.clamp(0.0, 1.0))\n}",
                    _ => "[package]\nname = \"lumen-core\"\nedition = \"2021\"",
                };
                ui.code_block("tab_preview", snippet).language(if active_name.ends_with(".wgsl") { "wgsl" } else { "rust" });
            });
        }
        "table-of-contents" => {
            let items = [
                TocItem::new("Architecture", 0),
                TocItem::new("Immediate layout tree", 1),
                TocItem::new("Unified WGSL shader", 1),
                TocItem::new("Spring physics", 0),
                TocItem::new("Perceptual tokens", 0),
            ];
            let descriptions = [
                ("Architecture", "Five focused crates keep layout, text shaping and springs completely independent of the GPU backend."),
                ("Immediate layout tree", "Widgets record lightweight flex nodes per frame and hit-test against previous-frame rectangles."),
                ("Unified WGSL shader", "Every rounded rect, border, Gaussian shadow and glyph quad renders in a single Uber-shader."),
                ("Spring physics", "Harmonic oscillators integrate analytically in closed form so motion is identical at 60 Hz or 240 Hz."),
                ("Perceptual tokens", "All semantic surface and accent colors interpolate cleanly in OKLab during theme transitions."),
            ];
            ui.row().fill_width().max_width(580.0).gap(18).align(Align::Start).show(|ui| {
                let (sec_t, sec_b) = descriptions[s.toc_active.min(descriptions.len() - 1)];
                ui.column().fill_width().show(|ui| {
                    ui.card().title(sec_t).pad(18.0).show(|ui| {
                        ui.label(sec_b).secondary();
                        ui.row().gap(8).show(|ui| {
                            if ui.button("Previous").ghost().small().disabled(s.toc_active == 0).clicked() {
                                s.toc_active = s.toc_active.saturating_sub(1);
                            }
                            if ui.button("Next section").secondary().small().disabled(s.toc_active + 1 >= items.len()).clicked() {
                                s.toc_active += 1;
                            }
                        });
                    });
                });
                ui.column().width(210.0).show(|ui| {
                    ui.table_of_contents("guide_toc", &items, &mut s.toc_active);
                });
            });
        }
        "nav-bar" => {
            let items = [
                NavBarItem::new("Overview").icon("layout-dashboard"),
                NavBarItem::new("Deployments").icon("rocket").badge("3"),
                NavBarItem::new("Metrics").icon("activity"),
                NavBarItem::new("Security").icon("shield-check"),
            ];
            let summaries = [
                ("Cluster overview", "14 edge regions healthy · median frame time 0.19 ms · 0 active alerts."),
                ("Active deployments", "3 progressive rollouts advancing across eu-central-1 and us-east-1."),
                ("Real-time telemetry", "p99 TLS handshake 18.4 ms · 99.98% cache hit ratio."),
                ("Zero-trust posture", "Hardware mTLS certificates rotated automatically every 24 hours."),
            ];
            ui.column().fill_width().max_width(560.0).gap(12).show(|ui| {
                ui.nav_bar("app_nav", &items, &mut s.nav_active);
                let (t, desc) = summaries[s.nav_active.min(summaries.len() - 1)];
                ui.card().title(t).pad(16.0).show(|ui| {
                    ui.label(desc).secondary();
                });
            });
        }
        "carousel" => {
            let slides = [
                CarouselSlide::new(
                    "Renderer",
                    "One shader for everything",
                    "Rounded rectangles, hairline borders, analytical Gaussian shadows and subpixel glyphs batch into a single GPU pipeline.",
                    "layers",
                    Tone::Accent,
                ),
                CarouselSlide::new(
                    "Motion",
                    "Closed-form spring physics",
                    "Springs preserve momentum across mid-flight retargets and settle to exact zero idle CPU/GPU wakeups.",
                    "sparkles",
                    Tone::Success,
                ),
                CarouselSlide::new(
                    "Looks",
                    "Four native design languages",
                    "Morph live between Ethereal Lumen, macOS, Windows 11 and GNOME Adwaita with perceptual OKLab color blending.",
                    "palette",
                    Tone::Warning,
                ),
            ];
            ui.column().fill_width().max_width(520.0).show(|ui| {
                ui.carousel("feature_deck", &mut s.carousel_idx, &slides).height(182.0);
            });
        }
        "callout" => {
            ui.column().fill_width().max_width(520.0).gap(12).show(|ui| {
                ui.callout(Tone::Info, "Zero idle frame wakeups").show(|ui| {
                    ui.label("Once springs and fades settle within epsilon, FrameOutput::repaint_after returns None so the process sleeps.").secondary();
                });
                ui.callout(Tone::Warning, "Physical pixel snapping").show(|ui| {
                    ui.label("Hairline strokes snap to physical device pixels using the active DPI scale factor to stay crisp on Retina panels.").secondary();
                });
            });
        }
        "undo-toast" => {
            ui.column().fill_width().max_width(480.0).gap(14).show(|ui| {
                ui.card().pad(14.0).show(|ui| {
                    ui.row().fill_width().align(Align::Center).gap(10).show(|ui| {
                        ui.avatar("EU", 2, 28.0, Some(true));
                        ui.column().fill_width().gap(2).show(|ui| {
                            ui.label("eu-central-canary-04").strong();
                            ui.label(if s.undo_visible { "Pending archival…" } else { "Active · taking jobs" }).small().muted();
                        });
                        if !s.undo_visible && ui.button("Archive runner").secondary().small().clicked() {
                            s.undo_visible = true;
                            s.undo_rem = 0.72;
                        }
                    });
                });
                if s.undo_visible {
                    let out = ui.undo_bar("runner_undo", "Archived runner “eu-central-canary-04”", s.undo_rem).show();
                    if out.undo_clicked {
                        s.undo_visible = false;
                        ui.toast(Toast::new("Runner restored").success());
                    } else if out.dismissed {
                        s.undo_visible = false;
                    }
                }
            });
        }
        "image-frame" => {
            ui.column().fill_width().max_width(460.0).gap(10).align(Align::Center).show(|ui| {
                ui.segmented(
                    &mut s.img_status,
                    &[
                        (ImageStatus::Loaded, "Loaded"),
                        (ImageStatus::Loading, "Loading"),
                        (ImageStatus::Fallback, "Fallback"),
                    ],
                );
                ui.image_frame("aurora_preview", "Ethereal Aurora Pass")
                    .caption("Click frame to zoom · 2× DPI")
                    .status(s.img_status)
                    .height(190.0);
            });
        }
        "sankey-chart" => {
            let nodes = ["Ingress", "Edge Cache", "Origin API", "2xx Cached", "2xx Dynamic", "429 Throttled"];
            let links = [
                SankeyLink::new(0, 1, 84.0),
                SankeyLink::new(0, 2, 44.0),
                SankeyLink::new(1, 3, 78.0),
                SankeyLink::new(1, 2, 6.0),
                SankeyLink::new(2, 4, 46.0),
                SankeyLink::new(2, 5, 4.0),
            ];
            ui.column().fill_width().max_width(600.0).show(|ui| {
                ui.card().title("Edge request flow (k req/s)").pad(16.0).show(|ui| {
                    ui.sankey_chart("edge_sankey", &nodes, &links, 210.0);
                });
            });
        }
        "contribution-graph" => {
            let mut weeks = [[0u8; 7]; 26];
            for (wi, wk) in weeks.iter_mut().enumerate() {
                for (di, cell) in wk.iter_mut().enumerate() {
                    let h = ((wi * 17 + di * 31 + 11) % 19) as u8;
                    *cell = if di == 0 || di == 6 {
                        h.saturating_sub(13)
                    } else if h < 5 {
                        0
                    } else {
                        (h - 4).min(12)
                    };
                }
            }
            let months = ["Jun", "Jul", "Aug", "Sep", "Oct", "Nov"];
            ui.column().fill_width().max_width(580.0).show(|ui| {
                ui.contribution_graph("git_contrib", &weeks, &months);
            });
        }
        "histogram" => {
            let buckets = ["0.05", "0.08", "0.11", "0.14", "0.17", "0.20", "0.23", "0.26", "0.29", "0.32", "0.35", "0.38", "0.41", "0.45ms"];
            let counts = [18.0, 54.0, 128.0, 210.0, 245.0, 182.0, 114.0, 68.0, 39.0, 24.0, 15.0, 9.0, 5.0, 3.0];
            let markers = [
                HistogramMarker::new(0.32, "p50 · 0.17ms", Tone::Success),
                HistogramMarker::new(0.68, "p95 · 0.29ms", Tone::Warning),
                HistogramMarker::new(0.88, "p99 · 0.39ms", Tone::Danger),
            ];
            ui.column().fill_width().max_width(580.0).show(|ui| {
                ui.card().title("Frame tessellation latency distribution").pad(16.0).show(|ui| {
                    ui.histogram("lat_hist", &buckets, &counts, &markers, 196.0);
                });
            });
        }
        "kpi-strip" => {
            let metrics = [
                KpiMetric::new("Throughput", "142.8k/s", 14.2, vec![94.0, 102.0, 99.0, 112.0, 118.0, 126.0, 134.0, 142.8]),
                KpiMetric::new("Median frame", "0.18 ms", 8.4, vec![0.26, 0.24, 0.23, 0.21, 0.20, 0.19, 0.19, 0.18]),
                KpiMetric::new("5xx error burn", "0.02%", -0.4, vec![0.06, 0.05, 0.05, 0.04, 0.03, 0.03, 0.02, 0.02]),
            ];
            ui.column().fill_width().max_width(620.0).show(|ui| {
                ui.kpi_strip("exec_kpis", &metrics);
            });
        }
        "day-schedule" => {
            let days = ["Mon 16", "Tue 17", "Wed 18", "Thu 19"];
            let events = [
                ScheduleEvent::new(0, 9.5, 2.0, "WGSL shader audit", Tone::Accent).room("Lab A"),
                ScheduleEvent::new(1, 11.0, 2.2, "OKLab contrast", Tone::Success).room("Design"),
                ScheduleEvent::new(2, 10.0, 1.8, "WebGPU bundle", Tone::Info),
                ScheduleEvent::new(2, 14.0, 2.2, "Canary rollout", Tone::Warning).room("Release"),
                ScheduleEvent::new(3, 12.5, 2.0, "v0.5 RC sign-off", Tone::Accent),
            ];
            ui.column().fill_width().max_width(600.0).show(|ui| {
                ui.day_schedule("sprint_cal", &days, 9, 17, Some((2, 13.3)), &events, 250.0);
            });
        }
        "node-editor" => {
            let wires = [
                GraphWire::new(0, 0, 1, 0),
                GraphWire::new(0, 1, 2, 1),
                GraphWire::new(1, 0, 2, 0),
            ];
            ui.column().fill_width().max_width(600.0).show(|ui| {
                ui.node_editor("shader_graph", &mut s.graph_nodes, &wires, 224.0);
            });
        }
        "commit-graph" => {
            let commits = [
                CommitRow::new("f3d0731", "widgets: add pro inputs, shuttle & editor tabs", "AL", "4m ago", 0, vec![0, 1]).badge("HEAD → ethereal"),
                CommitRow::new("825366a", "widgets: smooth Gaussian spotlight & flame ellipsis", "LF", "22m ago", 1, vec![0]),
                CommitRow::new("6ca5a39", "widgets: AI interfaces, waterfall & window chrome", "JB", "1h ago", 0, vec![0]).badge("v0.5-rc1"),
                CommitRow::new("f58d8e2", "widgets: activity rings, radar, funnel & kanban", "PN", "3h ago", 0, vec![0]),
                CommitRow::new("26ce625", "widgets: sortable table, tree & syntax code block", "AK", "5h ago", 0, vec![]),
            ];
            ui.column().fill_width().max_width(620.0).show(|ui| {
                ui.commit_graph("repo_log", &commits, &mut s.commit_sel);
            });
        }
        "test-explorer" => {
            ui.column().fill_width().max_width(580.0).show(|ui| {
                if ui.test_explorer("core_suite", &s.test_cases) {
                    for tc in &mut s.test_cases {
                        tc.state = lumen::TestState::Passed;
                        tc.failure_msg = None;
                    }
                    ui.toast(Toast::new("All 4 tests passed (0.58 ms)").success());
                }
            });
        }
        "data-grid" => {
            let cols = [
                GridColumn::new("Region"),
                GridColumn::new("PoPs").mono(),
                GridColumn::new("RPS Cap").mono(),
                GridColumn::new("P99 Latency").mono(),
            ];
            ui.column().fill_width().max_width(580.0).show(|ui| {
                ui.data_grid("cap_grid", &cols, &mut s.grid_rows);
            });
        }
        "filter-bar" => {
            const PRESETS: [(&str, &str, &str); 4] = [
                ("Status", "is", "Healthy"),
                ("Backend", "is", "WebGPU"),
                ("Region", "in", "EU, US"),
                ("Scale", "≥", "2× Retina"),
            ];
            const NODES: [(&str, &str, &str); 3] = [
                ("fra-gpu-01", "Healthy", "WebGPU · EU"),
                ("iad-gpu-04", "Healthy", "WebGPU · US"),
                ("nrt-gl2-02", "Degraded", "WebGL2 · APAC"),
            ];
            ui.column().fill_width().max_width(580.0).gap(10).show(|ui| {
                ui.filter_bar("cluster_flt", &mut s.facets, &PRESETS);
                ui.card().pad(10.0).show(|ui| {
                    for (name, status, meta) in NODES {
                        let ok_status = !s.facets.iter().any(|f| f.field == "Status" && f.value != status);
                        let ok_back = !s.facets.iter().any(|f| f.field == "Backend" && !meta.contains(&f.value));
                        if ok_status && ok_back {
                            ui.row().fill_width().pad(6.0).align(Align::Center).gap(10).show(|ui| {
                                ui.status(name, if status == "Healthy" { Tone::Success } else { Tone::Warning });
                                ui.spacer();
                                ui.label(meta).small().muted();
                            });
                        }
                    }
                });
            });
        }
        "ai-citations" => {
            let segs = [
                ("lumen batches rounded rectangles, borders, shadows and glyphs into one unified WGSL shader,", Some(1)),
                ("interpolating every semantic token in perceptual OKLab", Some(2)),
                ("while closed-form springs settle to exact zero idle wakeups.", Some(3)),
            ];
            let sources = [
                CitationSource::new(1, "paint.rs · Uber-shader", "lumen-core/src", "64-byte vertices carry mode & SDF radii."),
                CitationSource::new(2, "color.rs · OKLab mix", "lumen-core/src", "Uniform lightness across theme blends."),
                CitationSource::new(3, "anim.rs · Closed form", "lumen-core/src", "Analytical harmonic integration."),
            ];
            ui.column().fill_width().max_width(580.0).show(|ui| {
                ui.cited_answer("arch_cite", &segs, &sources);
            });
        }
        "text-effects" => {
            ui.column().fill_width().max_width(560.0).show(|ui| {
                ui.text_effects_card(
                    "kinetic_hero",
                    "Kinetic Typography",
                    "Closed-form springs meet perceptual OKLab.",
                    "Each word rises on its own harmonic oscillator and settles to zero idle frame wakeups.",
                );
            });
        }
        "rotary-knob" => {
            ui.column().fill_width().max_width(440.0).show(|ui| {
                ui.card().title("Synth filter stage").pad(18.0).show(|ui| {
                    ui.row().fill_width().justify(Justify::SpaceBetween).align(Align::Center).show(|ui| {
                        ui.rotary_knob("Drive", &mut s.knobs[0], 0.0..=100.0).unit("%");
                        ui.rotary_knob("Cutoff", &mut s.knobs[1], 80.0..=2400.0).unit(" Hz");
                        ui.rotary_knob("Resonance", &mut s.knobs[2], 0.0..=100.0).unit("%");
                    });
                });
            });
        }
        "compare-slider" => {
            ui.column().fill_width().max_width(520.0).show(|ui| {
                ui.compare_split("sdf_split", &mut s.cmp_ratio, "Wireframe mesh", "SDF shaded").height(212.0);
            });
        }
        "countdown" => {
            ui.column().fill_width().max_width(480.0).show(|ui| {
                ui.countdown_card("canary_bake", "Canary bake window · eu-central-1", &mut s.cd_secs, 900, &mut s.cd_running);
            });
        }
        "checklist-card" => {
            ui.column().fill_width().max_width(520.0).show(|ui| {
                ui.checklist_card("rc_steps", "v0.5 Release readiness", &mut s.check_steps);
            });
        }
        _ => {
            ui.label(format!("Unknown demo “{demo}”.")).secondary();
        }
    }
}

/// The overview bento on the site: one scene of everyday components, all of
/// them live. Mirrors the Ethereal concept in site/concepts.
fn bento(ui: &mut Ui, s: &mut State) {
    ui.row().fill().pad(24).gap(16).align(Align::Start).show(|ui| {
        ui.column().fill_width().gap(16).show(|ui| {
            ui.card().title("Add a runner").description("Runners pick up jobs for the platforms you choose.").show(|ui| {
                ui.segmented(&mut s.os, &[(0, "Linux"), (1, "macOS"), (2, "Windows")]);
                ui.text_input(&mut s.runner).label("Name").fill_width();
                ui.select(&mut s.region, &[(Region::Frankfurt, "Frankfurt"), (Region::Virginia, "Virginia"), (Region::Tokyo, "Tokyo"), (Region::Sydney, "Sydney")]).label("Region").fill_width();
                ui.column().gap(6).show(|ui| {
                    ui.label("Labels").strong();
                    ui.row().gap(6).show(|ui| {
                        for (i, l) in ["arm64", "gpu", "docker", "nightly"].iter().enumerate() {
                            ui.chip(l, &mut s.labels[i]);
                        }
                    });
                });
                ui.row().fill_width().align(Align::Center).show(|ui| {
                    ui.label("Concurrent jobs").strong();
                    ui.spacer();
                    ui.stepper("jobs", &mut s.jobs, 1, 16);
                });
                ui.row().fill_width().gap(8).show(|ui| {
                    ui.spacer();
                    ui.button("Cancel").ghost();
                    if ui.button("Add runner").primary().clicked() {
                        let d = format!("{} is online and taking jobs.", s.runner);
                        ui.toast(Toast::new("Runner added").description(d).success());
                    }
                });
            });
            ui.card().title("Traffic").show(|ui| {
                ui.donut_sized("bento-traffic", &[("Search", 42.0), ("Direct", 26.0), ("Social", 17.0), ("Other", 15.0)], 120.0);
            });
        });
        ui.column().fill_width().gap(16).show(|ui| {
            ui.card().show(|ui| ui.stat("Deploys", "1,284", Some(12.4), Some(&[9.0, 8.6, 9.4, 10.8, 10.0, 11.1, 11.3, 12.4, 12.6, 13.8, 15.1])));
            ui.card().title("Quick settings").show(|ui| {
                ui.settings(|rows| {
                    rows.row("Wi-Fi", "", |ui| {
                        ui.switch("", &mut s.on[0]);
                    });
                    rows.row("Bluetooth", "", |ui| {
                        ui.switch("", &mut s.on[1]);
                    });
                    rows.row("Focus", "", |ui| {
                        ui.switch("", &mut s.on[2]);
                    });
                });
                let v = format!("{:.0}%", s.vol * 100.0);
                ui.slider("Volume", &mut s.vol, 0.0..=1.0).readout(v).width(f32::INFINITY);
            });
            ui.card().title("Reviewers").show(|ui| {
                for (i, (ini, name, mail, req)) in [("JB", "Jonas Berg", "jonas@northlight.io", "Required"), ("PN", "Priya Nair", "priya@harbor.dev", "Required"), ("TR", "Tomas Rivera", "tomas@kiln.app", "Optional")].iter().enumerate() {
                    ui.row().fill_width().gap(12).align(Align::Center).show(|ui| {
                        ui.avatar(ini, i, 34.0, [Some(true), Some(false), None][i]);
                        ui.column().gap(1).fill_width().show(|ui| {
                            ui.label(*name).strong();
                            ui.label(*mail).muted();
                        });
                        ui.badge(*req, if i < 2 { Tone::Accent } else { Tone::Neutral });
                    });
                }
                ui.progress(Some(0.66));
            });
        });
        ui.column().fill_width().gap(16).show(|ui| {
            ui.card().show(|ui| ui.stat("Active users", "8,214", Some(4.1), Some(&[3.0, 3.4, 3.2, 4.1, 4.0, 4.8, 5.2, 5.0, 6.1, 6.4, 7.2])));
            ui.card().show(|ui| {
                ui.calendar_range(&mut s.span);
                ui.divider();
                ui.column().gap(2).show(|ui| {
                    ui.label("Release 2.4").strong();
                    ui.label("Pick a window while the freeze holds.").muted();
                });
                ui.toggle_group("slots", &mut s.slot, &[(0, "09:00"), (1, "11:30"), (2, "14:00"), (3, "16:30")]);
            });
        });
    });
}

/// The landing-page hero: a compact analytics dashboard.
fn hero(ui: &mut Ui, s: &mut State) {
    ui.row().fill().show(|ui| {
        ui.sidebar().width(208.0).show(|ui| {
            ui.nav_header("N", "Northwind");
            ui.nav_section("Workspace");
            ui.nav_item("layout-dashboard", "Overview").selected(true);
            ui.nav_item("chart-line", "Reports");
            ui.nav_item("users", "Customers").badge(4);
            ui.nav_item("settings", "Settings");
        });
        ui.column().fill().pad(24).gap(16).show(|ui| {
            ui.row().fill_width().align(Align::Center).gap(12).show(|ui| {
                ui.heading("Overview").level(2);
                ui.spacer();
                ui.segmented(&mut s.range, &[(Range::Day, "Day"), (Range::Week, "Week"), (Range::Month, "Month")]);
                ui.date_picker("", &mut s.date);
            });
            ui.row().fill_width().gap(16).show(|ui| {
                for (label, value, delta, trend) in [
                    ("Revenue", "$118.4k", 12.4, revenue()),
                    ("Active users", "8,214", 4.1, vec![60.0, 62.0, 61.0, 66.0, 70.0, 69.0, 74.0, 78.0]),
                    ("Churn", "2.1%", -0.6, vec![3.0, 2.8, 2.9, 2.6, 2.5, 2.3, 2.2, 2.1]),
                ] {
                    ui.card().pad(16.0).show(|ui| ui.stat(label, value, Some(delta), Some(&trend)));
                }
            });
            ui.row().fill_width().gap(16).align(Align::Start).show(|ui| {
                ui.column().fill_width().show(|ui| {
                    ui.card().title("Revenue").pad(16.0).show(|ui| {
                        ui.line_chart("hero-rev", &MONTHS, vec![Series::new("Revenue", revenue()), Series::new("Costs", costs())]).smooth(true).area().height(200.0);
                    });
                });
                ui.column().width(330.0).show(|ui| {
                    ui.card().title("Traffic").pad(16.0).show(|ui| {
                        ui.donut_sized("hero-traffic", &[("Search", 42.0), ("Direct", 26.0), ("Social", 15.0), ("Other", 17.0)], 120.0);
                    });
                });
            });
        });
    });
}

/// The same controls in every look.
fn looks(ui: &mut Ui, s: &mut State) {
    ui.set_look(s.look.look());
    frame(ui, |ui| {
        let opts: Vec<(Platform, &str)> = Platform::ALL.iter().map(|p| (*p, p.name())).collect();
        ui.segmented(&mut s.look, &opts);
        ui.column().fill_width().max_width(520.0).show(|ui| {
            ui.card().title("Sync").description("Everything blends when you switch looks.").show(|ui| {
                ui.settings(|rows| {
                    rows.row("iCloud Drive", "Keep files up to date.", |ui| {
                        ui.switch("", &mut s.on[0]);
                    });
                    let v = format!("{:.0}%", s.vol * 100.0);
                    rows.row("Quality", "", |ui| {
                        ui.slider("", &mut s.vol, 0.0..=1.0).readout(v).width(180.0);
                    });
                });
                ui.row().gap(12).show(|ui| {
                    ui.checkbox("Remember me", &mut s.checks[0]);
                    ui.radio(&mut s.plan, 1, "Fast");
                    ui.radio(&mut s.plan, 2, "Small");
                });
                ui.row().gap(8).show(|ui| {
                    ui.text_input(&mut s.name).fill_width();
                    ui.button("Cancel");
                    ui.button("Save").primary();
                });
            });
        });
    });
}

/// A preview of the password manager we plan to build with lumen.
fn vault(ui: &mut Ui, s: &mut State) {
    ui.row().fill().show(|ui| {
        ui.sidebar().width(200.0).show(|ui| {
            ui.nav_header("V", "Vault");
            ui.nav_section("Library");
            ui.nav_item("key-round", "All items").badge(128).selected(true);
            ui.nav_item("star", "Favorites");
            ui.nav_item("credit-card", "Cards");
            ui.nav_item("file-lock", "Secure notes");
            ui.nav_section("Health");
            ui.nav_item("shield-alert", "Watchtower").badge(3);
        });
        ui.column().fill().pad(24).gap(16).show(|ui| {
            ui.row().fill_width().gap(10).align(Align::Center).show(|ui| {
                ui.text_input(&mut s.search).icon("search").placeholder("Search 128 items").fill_width();
                ui.button("New item").primary().icon("plus");
            });
            ui.card().pad(0.0).show(|ui| {
                ui.column().fill_width().pad(16).gap(14).show(|ui| {
                    ui.row().fill_width().gap(12).align(Align::Center).show(|ui| {
                        ui.column().gap(2).fill_width().show(|ui| {
                            ui.label("GitHub").strong();
                            ui.label("ada@analytical.engine").secondary();
                        });
                        ui.badge("Strong", Tone::Success);
                    });
                    ui.divider();
                    ui.row().fill_width().gap(10).align(Align::Center).show(|ui| {
                        let mut pw = "••••••••••••••••".to_string();
                        ui.text_input(&mut pw).label("Password").mono().fill_width().id("pw");
                        ui.column().pad(Sides { top: 22.0, ..Default::default() }).show(|ui| {
                            ui.row().gap(4).show(|ui| {
                                ui.icon_button("eye").tooltip("Reveal");
                                ui.icon_button("copy").tooltip_shortcut("Copy password", "⌘C");
                            });
                        });
                    });
                    ui.label("One-time code").small().secondary();
                    ui.row().gap(12).align(Align::Center).show(|ui| {
                        ui.label("482 913").role(lumen::TextRole::Heading2);
                        ui.scope("otp", |ui| ui.progress(Some(0.6)));
                    });
                });
            });
            ui.card().title("Password health").pad(16.0).show(|ui| {
                ui.row().gap(24).align(Align::Center).show(|ui| {
                    let good = ui.color(lumen::Token::Success);
                    ui.gauge_colored("health", 0.82, "Score", good);
                    ui.column().gap(8).show(|ui| {
                        ui.status("3 reused passwords", Tone::Warning);
                        ui.status("1 in a known breach", Tone::Danger);
                        ui.status("124 strong", Tone::Success);
                    });
                });
            });
        });
    });
}

/// Input that opens a demo's overlay for its screenshot (`--open`).
#[cfg(not(target_arch = "wasm32"))]
fn open_steps(demo: &str) -> Vec<lumen::ShotStep> {
    use lumen::{Modifiers, ShotStep::*};
    let k = |key| Key(key, Modifiers::default());
    match demo {
        "combobox" => vec![k(lumen::Key::Tab), Type("an".into()), Wait(0.6)],
        "multi-select" => vec![k(lumen::Key::Tab), k(lumen::Key::ArrowDown), k(lumen::Key::ArrowDown), Wait(0.6)],
        "otp-input" => vec![k(lumen::Key::Tab), Wait(0.4)],
        "split-button" => vec![k(lumen::Key::Tab), k(lumen::Key::Tab), k(lumen::Key::Enter), Wait(0.5)],
        "dock" => vec![Hover(lumen::vec2(342.0, 212.0)), Wait(0.6)],
        "menu-bar" => vec![k(lumen::Key::Tab), k(lumen::Key::Enter), Wait(0.5)],
        "spotlight-card" => vec![Hover(lumen::vec2(235.0, 215.0)), Wait(0.4)],
        "mention-input" => vec![k(lumen::Key::Tab), k(lumen::Key::End), Type(" @".into()), Wait(0.5)],
        _ => vec![],
    }
}

fn parse_look(s: &str) -> Platform {
    match s {
        "mac" | "macos" => Platform::Mac,
        "windows" | "win" => Platform::Windows,
        "gnome" | "linux" => Platform::Gnome,
        _ => Platform::Lumen,
    }
}

fn parse_accent(s: &str) -> Accent {
    if let Some(hex) = s.strip_prefix('#') {
        if let Ok(v) = u32::from_str_radix(hex, 16) {
            return Accent::Custom(v);
        }
    }
    Accent::ALL.iter().copied().find(|a| a.name().eq_ignore_ascii_case(s)).unwrap_or(Accent::Blue)
}

struct Params {
    demo: String,
    dark: bool,
    look: Platform,
    accent: Accent,
    screenshot: Option<String>,
    size: (u32, u32),
    scale: f32,
    embed: bool,
    open: bool,
    speed: f32,
    reduced: bool,
}

fn params() -> Params {
    let mut p = Params { demo: "hero".into(), dark: false, look: Platform::Lumen, accent: Accent::Blue, screenshot: None, size: (960, 600), scale: 2.0, embed: false, open: false, speed: 1.0, reduced: false };
    #[cfg(target_arch = "wasm32")]
    {
        let q = web_sys::window().and_then(|w| w.location().search().ok()).unwrap_or_default();
        for kv in q.trim_start_matches('?').split('&') {
            let (k, v) = kv.split_once('=').unwrap_or((kv, ""));
            let v = v.replace("%23", "#");
            match k {
                "demo" => p.demo = v.to_string(),
                "dark" => p.dark = v == "1" || v == "true",
                "look" => p.look = parse_look(&v),
                "accent" => p.accent = parse_accent(&v),
                "embed" => p.embed = v == "1",
                "speed" => p.speed = v.parse().unwrap_or(1.0),
                "reduced" => p.reduced = v == "1",
                _ => {}
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let mut i = 0;
        while i < args.len() {
            let next = args.get(i + 1).cloned().unwrap_or_default();
            match args[i].as_str() {
                "--dark" => p.dark = true,
                "--look" => {
                    p.look = parse_look(&next);
                    i += 1;
                }
                "--accent" => {
                    p.accent = parse_accent(&next);
                    i += 1;
                }
                "--screenshot" => {
                    p.screenshot = Some(next);
                    i += 1;
                }
                "--size" => {
                    let (w, h) = next.split_once('x').unwrap_or(("960", "600"));
                    p.size = (w.parse().unwrap_or(960), h.parse().unwrap_or(600));
                    i += 1;
                }
                "--embed" => p.embed = true,
                "--open" => p.open = true,
                "--scale" => {
                    p.scale = next.parse().unwrap_or(2.0);
                    i += 1;
                }
                "--list" => {
                    for (k, t) in DEMOS {
                        println!("{k}\t{t}");
                    }
                    std::process::exit(0);
                }
                d if !d.starts_with('-') => p.demo = d.to_string(),
                _ => {}
            }
            i += 1;
        }
    }
    p
}

fn main() {
    let mut p = params();
    if p.look != Platform::Lumen && p.accent == Accent::Blue {
        p.accent = Accent::Native;
    }
    let mut state = State::new(p.demo.clone());
    state.embed = p.embed;
    state.force_open = p.open;
    state.look = p.look;
    let mode = if p.dark { Mode::Dark } else { Mode::Light };
    let mut look: Look = p.look.look();
    if p.speed > 0.0 && p.speed != 1.0 {
        look.motion = look.motion.slower(1.0 / p.speed);
    }
    if p.reduced {
        state.reduced = true;
    }
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(out) = &p.screenshot {
        let steps = if p.open { open_steps(&p.demo) } else { vec![] };
        let shot = lumen::Shot { size: p.size, scale: p.scale, mode, accent: p.accent, look, steps, ..Default::default() };
        match lumen::screenshot(out, &shot, &mut state, view) {
            Ok(()) => println!("wrote {out}"),
            Err(e) => {
                eprintln!("screenshot failed: {e}");
                std::process::exit(1);
            }
        }
        return;
    }
    let title = DEMOS.iter().find(|d| d.0 == p.demo).map(|d| d.1).unwrap_or("lumen");
    lumen::App::new(format!("lumen — {title}")).size(p.size.0, p.size.1).mode(mode).accent(p.accent).look(look).canvas("lumen").run(state, view);
}
