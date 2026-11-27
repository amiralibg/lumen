//! One live demo per lumen component.
//!
//! - Native: `cargo run -p lumen-demos -- chart-line [--dark] [--look mac]`
//! - Web: `demo.html?demo=chart-line&dark=1&look=mac&accent=blue`
//! - Screenshot: `cargo run -p lumen-demos -- chart-line --screenshot out.png`

use std::collections::BTreeSet;

mod apps;
mod scenes;

use lumen::{
    vec2, Accent, AiModelSpec, Align, ArenaSlab, AttachmentItem, BarListItem, BulletMetric, Candle, CarouselSlide,
    ChatMessage, ChecklistStep, CitationSource, Color, Command, CommentEntry, CommitRow, ContextSlice, Currency, Date,
    DateStripDay, DescPair, DimensionUnit, DivergingBar, DockItem, EditorTab, FilterFacet, FlameSpan, GanttTask,
    GraphNode, GraphWire, GridColumn, HistogramMarker, HttpMethod, ImageStatus, Justify, KanbanCard, KanbanCol,
    KpiMetric, KvPair, LogEntry, LogLevel, Look, MasonryCard, MentionCandidate, MeterSegment, MetricRow, Mode,
    NavBarItem, NotificationItem, Placement, PlanSpec, Platform, PresencePeer, PromptTemplate, ReactionChip,
    ReorderItem, RingSpec, SankeyLink, ScatterPoint, ScatterSeries, ScheduleEvent, ScheduleRule, SchemaField, Series,
    ShortcutHint, Sides, SortDir, SpeedDialItem, SuggestionDecision, TableCell, TableCol, TermLine, TestCase, TextRole,
    Time, TimelineItem, Toast, TocItem, Tone, ToolCall, ToolStatus, TraceSpan, TreeNode, TreemapItem, Ui, UptimeStatus,
    ViewMode, WaffleSlice, WorldClock,
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
    ("list-view", "List view"),
    ("virtual-list", "Virtualized list"),
    ("accessibility", "Accessibility"),
    ("code-editor", "Code editor"),
    ("emoji-picker", "Emoji picker"),
    ("wheel-picker", "Wheel and time picker"),
    ("calendar-heatmap", "Calendar heatmap"),
    ("file-browser", "File browser"),
    ("tour", "Guided tour"),
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
    ("action-bar", "Floating action bar"),
    ("speed-dial", "Speed dial"),
    ("numeric-pad", "Numeric keypad"),
    ("dimension-input", "Dimension input"),
    ("stacked-meter", "Stacked meter"),
    ("announcement-bar", "Announcement bar"),
    ("quantity-picker", "Quantity picker"),
    ("swatch-picker", "Swatch picker"),
    ("date-strip", "Date strip"),
    ("world-clocks", "World clocks"),
    ("bullet-chart", "Bullet chart"),
    ("metric-table", "Metric watchlist"),
    ("ai-model-picker", "AI model picker"),
    ("context-meter", "Context meter"),
    ("api-inspector", "API inspector"),
    ("permission-matrix", "Permission matrix"),
    ("description-list", "Description list"),
    ("masonry-grid", "Masonry grid"),
    ("collapsible-card", "Collapsible card"),
    ("marquee", "Marquee ticker"),
    ("presence-pile", "Presence pile"),
    ("comment-thread", "Comment thread"),
    ("reaction-bar", "Reaction bar"),
    ("attachment-list", "Attachment list"),
    ("progress-steps", "Progress steps"),
    ("shortcut-bar", "Shortcut bar"),
    ("view-switcher", "View switcher"),
    ("gradient-slider", "Gradient slider"),
    ("schedule-picker", "Schedule picker"),
    ("quarter-picker", "Quarter picker"),
    ("win-loss-chart", "Win-loss chart"),
    ("waffle-chart", "Waffle chart"),
    ("schema-viewer", "Schema viewer"),
    ("env-switcher", "Environment switcher"),
    ("memory-profiler", "Memory profiler"),
    ("ai-suggestion", "AI code suggestion"),
    ("prompt-templates", "Prompt templates"),
    ("pricing-cards", "Pricing cards"),
    ("ab-comparison", "A/B comparison"),
    ("equalizer", "Graphic equalizer"),
    ("sparkline", "Sparkline and stats"),
    ("looks", "Looks"),
    ("vault", "Vault (password manager preview)"),
    ("scene-ide", "AI coding workspace"),
    ("scene-observability", "Observability & profiling console"),
    ("scene-shader", "Visual shader studio"),
    ("scene-release", "Release control plane"),
    ("scene-finance", "Financial & analytics terminal"),
    ("scene-collab", "Collaborative spec & RFC studio"),
    ("scene-governance", "Cloud governance & FinOps"),
    ("scene-schedule", "Global on-call & launch room"),
    ("app-mail", "Postbox (mail)"),
    ("app-music", "Cadence (music)"),
    ("app-tracker", "Relay (issue tracker)"),
    ("app-notes", "Margin (notes)"),
    ("app-assistant", "Lumi (AI assistant)"),
    ("app-finance", "Tally (finance)"),
    ("app-todo", "Today (to-do)"),
    ("app-settings", "Preferences (settings)"),
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
    win_event: Option<(&'static str, &'static str)>,
    win_events: u32,
    /// virtual-list: 100k rows, made on first use.
    vl_rows: Vec<lumen::ListRow>,
    vl_sel: Option<usize>,
    vl_mode: u8,
    vl_next_key: u64,
    vl_built: usize,
    vl_off: f32,
    /// accessibility: a form a screen reader can drive.
    ax_name: String,
    ax_wifi: bool,
    ax_vol: f32,
    ax_theme: u8,
    ax_dialog: bool,
    ax_log: String,
    ax_sel: Option<usize>,
    ax_rows: Vec<lumen::ListRow>,
    split_ratio: f32,
    term_lines: Vec<TermLine>,
    term_in: String,
    prop_pos: (f64, f64),
    prop_radius: f64,
    prop_opacity: f64,
    prop_clip: bool,
    notifs: Vec<NotificationItem>,
    list_sel: Option<usize>,
    code: String,
    emoji: String,
    wheel_size: usize,
    wheel_time: lumen::Time,
    heat_sel: Option<Date>,
    fb_path: Vec<String>,
    fb_sel: Option<usize>,
    tour_step: Option<usize>,
    list_rows: Vec<lumen::ListRow>,
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
    batch_sel: [bool; 3],
    sd_open: bool,
    pin_val: String,
    pin_masked: bool,
    dim_w: f32,
    dim_w_unit: DimensionUnit,
    dim_side: f32,
    dim_side_unit: DimensionUnit,
    ann_open: bool,
    replica_qty: i64,
    swatch_idx: usize,
    strip_day: usize,
    llm_idx: usize,
    http_verb: HttpMethod,
    api_url: String,
    rbac_grid: Vec<Vec<bool>>,
    cc_open: [bool; 2],
    marquee_play: bool,
    follow_peer: bool,
    comments: Vec<CommentEntry>,
    thread_resolved: bool,
    reactions: Vec<ReactionChip>,
    attachments: Vec<AttachmentItem>,
    story_step: usize,
    view_mode: usize,
    llm_temp: f32,
    color_warmth: f32,
    sched_rule: ScheduleRule,
    fy_year: i32,
    fy_quarter: u8,
    env_idx: usize,
    env_locked: bool,
    patch_decision: SuggestionDecision,
    annual_billing: bool,
    selected_tier_plan: usize,
    eq_bands: [f32; 8],
    eq_preset: usize,
    scenes: scenes::Scenes,
    apps: apps::Apps,
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
            win_event: None,
            win_events: 0,
            vl_rows: vec![],
            vl_sel: None,
            vl_mode: 0,
            vl_next_key: 0,
            vl_built: 0,
            vl_off: 0.0,
            ax_name: String::new(),
            ax_wifi: false,
            ax_vol: 4.0,
            ax_theme: 0,
            ax_dialog: false,
            ax_log: "Nothing yet".into(),
            ax_sel: None,
            ax_rows: vec![],
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
            list_sel: Some(1),
            code: CODE_SAMPLE.into(),
            emoji: "Ship it".into(),
            wheel_size: 3,
            wheel_time: lumen::Time::new(9, 30),
            heat_sel: None,
            fb_path: vec!["lumen".into(), "crates".into()],
            fb_sel: Some(1),
            tour_step: None,
            list_rows: vec![
                lumen::ListRow::new("Q4 planning.key").subtitle("Keynote · 18.4 MB").icon("presentation").meta("09:12").unread(true),
                lumen::ListRow::new("Brand guidelines.pdf").subtitle("PDF · 6.2 MB").icon("file-text").meta("Yesterday").starred(true).badge("Shared", Tone::Accent),
                lumen::ListRow::new("Hero render.png").subtitle("Image · 3.1 MB").icon("image").meta("Mon"),
                lumen::ListRow::new("Budget 2027.numbers").subtitle("Spreadsheet · 412 KB").icon("sheet").meta("Oct 30").unread(true),
                lumen::ListRow::new("Interview audio.m4a").subtitle("Audio · 48 MB").icon("audio-lines").meta("Oct 28"),
            ],
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
            batch_sel: [true, true, false],
            sd_open: true,
            pin_val: "482".into(),
            pin_masked: true,
            dim_w: 760.0,
            dim_w_unit: DimensionUnit::Px,
            dim_side: 24.0,
            dim_side_unit: DimensionUnit::Percent,
            ann_open: true,
            replica_qty: 8,
            swatch_idx: 1,
            strip_day: 2,
            llm_idx: 0,
            http_verb: HttpMethod::Post,
            api_url: "https://edge.lumen.dev/v1/deploys/promote".into(),
            rbac_grid: vec![
                vec![true, true, false, false],
                vec![true, false, false, false],
                vec![true, true, true, false],
                vec![true, true, true, true],
            ],
            cc_open: [true, false],
            marquee_play: true,
            follow_peer: true,
            comments: vec![
                CommentEntry::new(
                    "Lena Fischer",
                    "LF",
                    "4m ago",
                    "Should we clamp the Gaussian blur radius to card.radius * 0.85 so outer scissor corners stay razor-sharp?",
                )
                .with_likes(4, true),
                CommentEntry::new(
                    "Jonas Berg",
                    "JB",
                    "2m ago",
                    "Done in paint.rs — added a post-clip 1px ring pass and verified both light & dark posters.",
                )
                .reply()
                .with_likes(2, false),
            ],
            thread_resolved: false,
            reactions: vec![
                ReactionChip::new("rocket", "Ship it", 12, true),
                ReactionChip::new("sparkles", "Crisp", 8, true),
                ReactionChip::new("heart", "Love", 5, false),
                ReactionChip::new("flame", "Fast", 3, false),
            ],
            attachments: vec![
                AttachmentItem::uploaded("lumen-demos_bg.wasm", "WASM", "8.2 MB"),
                AttachmentItem::uploaded("sdf_uber.wgsl", "WGSL", "14.6 KB"),
                AttachmentItem::uploading("poster-atlas-2x.png", "PNG", "1.9 MB", 0.68),
            ],
            story_step: 1,
            view_mode: 1,
            llm_temp: 0.42,
            color_warmth: 5600.0,
            sched_rule: ScheduleRule::default(),
            fy_year: 2026,
            fy_quarter: 2,
            env_idx: 2,
            env_locked: true,
            patch_decision: SuggestionDecision::Pending,
            annual_billing: true,
            selected_tier_plan: 1,
            eq_bands: [-2.5, -1.0, 1.5, 4.5, 5.2, 3.0, 1.0, -0.5],
            eq_preset: 1,
            scenes: scenes::Scenes::default(),
            apps: apps::Apps::default(),
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
    if s.embed
        && !matches!(demo.as_str(), "hero" | "vault" | "bento")
        && !demo.starts_with("scene-")
        && !demo.starts_with("app-")
    {
        for m in [Mode::Light, Mode::Dark] {
            let surf = ui.theme.target_look().neutrals(m).surface;
            ui.override_color(m, lumen::Token::Background, Some(surf));
        }
    }
    match demo.as_str() {
        "hero" => hero(ui, s),
        "bento" => scenes::bento::view(ui, &mut s.scenes.bento),
        "looks" => looks(ui, s),
        "vault" => vault(ui, s),
        "scene-ide" => scenes::ide::view(ui, &mut s.scenes.ide),
        "scene-observability" => scenes::observability::view(ui, &mut s.scenes.observability),
        "scene-shader" => scenes::shader::view(ui, &mut s.scenes.shader),
        "scene-release" => scenes::release::view(ui, &mut s.scenes.release),
        "scene-finance" => scenes::finance::view(ui, &mut s.scenes.finance),
        "scene-collab" => scenes::collab::view(ui, &mut s.scenes.collab),
        "scene-governance" => scenes::governance::view(ui, &mut s.scenes.governance),
        "scene-schedule" => scenes::schedule::view(ui, &mut s.scenes.schedule),
        "app-mail" => apps::mail::view(ui, &mut s.apps.mail),
        "app-music" => apps::music::view(ui, &mut s.apps.music),
        "app-tracker" => apps::tracker::view(ui, &mut s.apps.tracker),
        "app-notes" => apps::notes::view(ui, &mut s.apps.notes),
        "app-assistant" => apps::assistant::view(ui, &mut s.apps.assistant),
        "app-finance" => apps::finance::view(ui, &mut s.apps.finance),
        "app-todo" => apps::todo::view(ui, &mut s.apps.todo),
        "app-settings" => apps::settings::view(ui, &mut s.apps.settings),
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
            ui.row().gap(10).wrap().justify(Justify::Center).align(Align::Center).show(|ui| {
                ui.button("Deploy").primary().icon("rocket");
                ui.button("Preview").icon("eye");
                ui.button("Cancel").ghost();
                ui.button("Delete").danger().icon("trash-2");
                ui.icon_button("copy").tooltip_shortcut("Copy", "⌘C");
            });
            ui.row().gap(10).wrap().justify(Justify::Center).align(Align::Center).show(|ui| {
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
                ui.row().gap(8).wrap().justify(Justify::Center).show(|ui| {
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
            ui.row().fill_width().gap(8).wrap().justify(Justify::Center).show(|ui| {
                ui.badge("Neutral", Tone::Neutral);
                ui.badge("Accent", Tone::Accent);
                ui.badge("Live", Tone::Success);
                ui.badge("Degraded", Tone::Warning);
                ui.badge("Down", Tone::Danger);
                ui.badge("Beta", Tone::Info);
            });
            ui.row().fill_width().gap(20).wrap().justify(Justify::Center).show(|ui| {
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
            // Side by side when there's room, stacked on narrow embeds.
            ui.row().fill_width().wrap().gap(32).align(Align::Start).show(|ui| {
                ui.calendar(&mut s.date);
                ui.calendar_range(&mut s.span);
            });
        }
        "date-picker" => {
            ui.row().gap(16).wrap().align(Align::Start).show(|ui| {
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
            ui.row().fill_width().gap(16).wrap().show(|ui| {
                for (label, value, delta, trend) in [
                    ("Revenue", "$118k", 12.4, revenue()),
                    ("Active users", "8,214", 4.1, vec![60.0, 62.0, 61.0, 66.0, 70.0, 69.0, 74.0, 78.0]),
                    ("Churn", "2.1%", -0.6, vec![3.0, 2.8, 2.9, 2.6, 2.5, 2.3, 2.2, 2.1]),
                ] {
                    ui.column().fill_width().min_width(130.0).show(|ui| {
                        ui.card().show(|ui| {
                            ui.stat(label, value, Some(delta), Some(&trend));
                        });
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
            ui.row().gap(36).wrap().justify(Justify::Center).align(Align::Center).show(|ui| {
                ui.activity_rings("cluster_rings", &rings, 132.0);
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
                let ev = s.win_event;
                let n = s.win_events;
                let shown = ui
                    .window_chrome("preview_win", "lumen — workspace")
                    .subtitle("ethereal")
                    .show(|ui| {
                        ui.row().fill_width().align(Align::Center).show(|ui| {
                            ui.label("Native window chrome").strong();
                            ui.spacer();
                            ui.badge(s.win_look.name(), Tone::Accent);
                        });
                        match ev {
                            Some((icon, what)) => {
                                // Each new event rises in and fades up.
                                let eid = ui.make_id(("win_ev", n));
                                let t = ui.fade(eid, 0, true, 0.22, 0.1, lumen::Ease::Out);
                                let c = ui.color(lumen::Token::AccentStrong);
                                ui.row().gap(8).align(Align::Center).layer(t, lumen::vec2(0.0, (1.0 - t) * 6.0)).show(|ui| {
                                    ui.icon_leaf(icon, 14.0, c);
                                    ui.label(what).secondary();
                                });
                            }
                            None => {
                                ui.label("Try the buttons, drag the title bar or double-click it. In a real window (App::custom_chrome) they move, minimize, zoom and close it.").secondary();
                            }
                        }
                    });
                let r = shown.response;
                let e = if r.close {
                    Some(("x", "Close clicked: a real window would close (or ask first)."))
                } else if r.minimize {
                    Some(("minus", "Minimize clicked."))
                } else if r.zoom || r.double_clicked {
                    Some(if r.maximized { ("maximize-2", "Zoomed: the button now shows restore.") } else { ("minimize-2", "Restored to the previous size.") })
                } else if r.drag_started {
                    Some(("move", "Title bar pressed: a real window starts moving here."))
                } else {
                    None
                };
                if e.is_some() {
                    s.win_event = e;
                    s.win_events += 1;
                }
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
        "emoji-picker" => {
            // Side by side when there's room; on a narrow card the message
            // goes on top and the picker below.
            let narrow = ui.screen_size().x < 600.0;
            let picker = |ui: &mut Ui, msg: &mut String| {
                ui.card().pad(12.0).show(|ui| {
                    if let Some(e) = ui.emoji_picker("demo") {
                        msg.push(' ');
                        msg.push_str(e);
                    }
                });
            };
            let field = |ui: &mut Ui, msg: &mut String, w: Option<f32>| {
                let col = ui.column().gap(6);
                let col = match w {
                    Some(w) => col.width(w),
                    None => col.fill_width(),
                };
                col.show(|ui| {
                    ui.label("Message").strong();
                    ui.text_input(msg).id("emoji-msg").fill_width();
                    ui.label("Pick to append. ↑ ↓ ← → and Enter work on the grid.").muted().small();
                });
            };
            if narrow {
                ui.column().gap(14).fill_width().show(|ui| {
                    field(ui, &mut s.emoji, None);
                    picker(ui, &mut s.emoji);
                });
            } else {
                ui.row().gap(24).align(Align::Start).show(|ui| {
                    picker(ui, &mut s.emoji);
                    field(ui, &mut s.emoji, Some(200.0));
                });
            }
        }
        "wheel-picker" => {
            ui.row().gap(40).align(Align::Center).show(|ui| {
                ui.column().gap(8).align(Align::Center).show(|ui| {
                    ui.label("Shirt size").strong();
                    ui.wheel_picker("size", &mut s.wheel_size, &["XXS", "XS", "S", "M", "L", "XL", "XXL"]);
                });
                ui.column().gap(8).align(Align::Center).show(|ui| {
                    ui.label("Reminder").strong();
                    ui.time_wheel("alarm", &mut s.wheel_time, 5);
                    ui.label(format!("Rings at {}", s.wheel_time.format_12())).muted().small();
                });
            });
        }
        "calendar-heatmap" => {
            let first = Date::new(2026, 10, 1);
            let vals: Vec<(Date, f32)> = (0..61)
                .map(|k| {
                    let d = first.add_days(k);
                    let wk = d.weekday();
                    let base = if wk >= 5 { 0.0 } else { ((k as f32 * 0.37).sin() * 3.0 + 4.0).max(0.0) };
                    (d, (base + ((k * 7919) % 5) as f32 * 0.6).round())
                })
                .collect();
            ui.column().fill_width().gap(8).show(|ui| {
                ui.calendar_heatmap("deploys", first, 2, &vals, "deploys", &mut s.heat_sel);
            });
        }
        "file-browser" => {
            let entries = demo_files(&s.fb_path);
            ui.column().fill_width().max_width(680.0).show(|ui| {
                let path: Vec<&str> = s.fb_path.iter().map(|p| p.as_str()).collect();
                let out = ui.file_browser("files", &path, &entries, &mut s.fb_sel);
                if let Some(k) = out.navigate {
                    s.fb_path.truncate(k + 1);
                    s.fb_sel = None;
                }
                if out.up && s.fb_path.len() > 1 {
                    s.fb_path.pop();
                    s.fb_sel = None;
                }
                if let Some(i) = out.opened {
                    if entries[i].folder {
                        s.fb_path.push(entries[i].name.clone());
                        s.fb_sel = None;
                    } else {
                        ui.toast(Toast::new(format!("Opened {}", entries[i].name)));
                    }
                }
            });
        }
        "tour" => {
            if std::mem::take(&mut s.force_open) {
                s.tour_step = Some(1);
            }
            let mut ids = vec![];
            ui.column().fill_width().max_width(560.0).gap(14).show(|ui| {
                ui.row().fill_width().gap(10).align(Align::Center).show(|ui| {
                    ids.push(ui.search_field(&mut s.search).placeholder("Search projects").fill_width().show().id);
                    ids.push(ui.button("New project").primary().icon("plus").show().id);
                });
                ui.card().title("Northwind").description("3 environments · deployed 4m ago").show(|ui| {
                    ids.push(ui.segmented(&mut s.range, &[(Range::Day, "Day"), (Range::Week, "Week"), (Range::Month, "Month")]).id);
                    ui.progress(Some(0.66));
                });
                if ui.button("Start the tour").icon("sparkles").show().clicked() {
                    s.tour_step = Some(0);
                }
            });
            let steps = [
                lumen::TourStep::new(ids[0], "Find anything", "Search projects, people and settings. Press / from anywhere."),
                lumen::TourStep::new(ids[1], "Start a project", "Pick a template or import a repository. You can change everything later."),
                lumen::TourStep::new(ids[2], "Pick a time range", "Every chart on the page follows this range."),
            ];
            ui.tour("onboarding", &steps, &mut s.tour_step);
        }
        "code-editor" => {
            ui.column().fill_width().max_width(640.0).gap(8).show(|ui| {
                if ui.code_editor("demo", &mut s.code).lang("rust").rows(11).changed() {
                    s.t += 1;
                }
                ui.label("Tab indents, ⌘/ comments, Esc then Tab leaves.").muted().small();
            });
        }
        "list-view" => {
            ui.column().fill_width().max_width(520.0).gap(10).show(|ui| {
                let out = ui.list_view("files", &s.list_rows, &mut s.list_sel);
                if let Some(i) = out.starred {
                    s.list_rows[i].starred = !s.list_rows[i].starred;
                }
                if out.changed {
                    if let Some(i) = s.list_sel {
                        s.list_rows[i].unread = false;
                    }
                }
                if let Some(i) = out.activated {
                    let name = s.list_rows[i].title.clone();
                    ui.toast(Toast::new(format!("Opened {name}")));
                }
                let pick = s.list_sel.map(|i| s.list_rows[i].title.clone()).unwrap_or_else(|| "nothing".into());
                ui.label(format!("Selected: {pick}. Use ↑ ↓ to move, Enter to open.")).muted().small();
            });
        }
        "accessibility" => {
            if s.ax_rows.is_empty() {
                s.ax_rows = (0..1000u64).map(|k| lumen::ListRow::new(format!("Message {}", k + 1)).subtitle(["Ada", "Grace", "Linus", "Barbara"][(k % 4) as usize])).collect();
            }
            ui.row().fill_width().gap(24).align(Align::Start).wrap().show(|ui| {
                ui.column().width(320.0).gap(12).show(|ui| {
                    ui.heading("Profile").level(2);
                    ui.text_input(&mut s.ax_name).label("Name").placeholder("Your name").show();
                    ui.switch("Wi-Fi", &mut s.ax_wifi).show();
                    ui.slider("Volume", &mut s.ax_vol, 0.0..=10.0).step(1.0).show();
                    ui.select(&mut s.ax_theme, &[(0, "System"), (1, "Light"), (2, "Dark")]).label("Theme").show();
                    ui.row().gap(8).show(|ui| {
                        if ui.button("Save").primary().show().clicked() {
                            s.ax_log = format!("Saved {}", if s.ax_name.is_empty() { "(no name)" } else { s.ax_name.as_str() });
                        }
                        let more = ui.button("More").icon("chevron-down").show();
                        ui.menu(&more, |ui| {
                            if ui.menu_item("copy", "Duplicate").shortcut("⌘D").show().clicked() {
                                s.ax_log = "Duplicated".into();
                            }
                            if ui.menu_item("trash", "Delete…").danger().show().clicked() {
                                s.ax_dialog = true;
                            }
                        });
                    });
                    ui.label(format!("Last action: {}", s.ax_log)).muted();
                });
                ui.column().fill_width().min_width(260.0).gap(8).show(|ui| {
                    ui.heading("Inbox").level(2);
                    ui.scroll("ax-inbox").height(300.0).show(|ui| {
                        ui.list_view("ax-inbox", &s.ax_rows, &mut s.ax_sel);
                    });
                });
            });
            let mut deleted = false;
            ui.dialog("ax-confirm", &mut s.ax_dialog).title("Delete profile?").description("This can't be undone.").show(|ui| {
                ui.row().gap(8).show(|ui| {
                    if ui.button("Cancel").show().clicked() {
                        ui.close_layer();
                    }
                    if ui.button("Delete").variant(lumen::Variant::Danger).show().clicked() {
                        deleted = true;
                        ui.close_layer();
                    }
                });
            });
            if deleted {
                s.ax_log = "Deleted".into();
            }
        }
        "virtual-list" => {
            if s.vl_rows.is_empty() {
                s.vl_rows = (0..100_000u64).map(virtual_row).collect();
                s.vl_next_key = 100_000;
                if std::env::var_os("LUMEN_VL_TABLE").is_some() {
                    s.vl_mode = 1;
                }
            }
            let n = s.vl_rows.len();
            let avail = (ui.screen_size().y - if s.embed { 150.0 } else { 190.0 }).max(200.0);
            ui.column().fill_width().max_width(720.0).gap(12).show(|ui| {
                ui.row().fill_width().gap(10).align(Align::Center).show(|ui| {
                    ui.segmented(&mut s.vl_mode, &[(0u8, "List"), (1u8, "Table")]);
                    ui.spacer();
                    // History arriving above: what you're reading stays put.
                    if ui.button("Load 25 earlier").icon("history").show().clicked {
                        let base = s.vl_next_key;
                        s.vl_next_key += 25;
                        let mut older: Vec<lumen::ListRow> = (0..25).map(|k| virtual_row(base + k)).collect();
                        older.append(&mut s.vl_rows);
                        s.vl_rows = older;
                        s.vl_sel = s.vl_sel.map(|i| i + 25);
                    }
                });
                let sid = if s.vl_mode == 0 { ui.peek_id(("scroll", "vl")) } else { ui.peek_id(("scroll", ("tbody", "vl_table"))) };
                if std::env::var_os("LUMEN_AUTOSCROLL").is_some() {
                    // Benchmark: scroll continuously through all rows.
                    s.vl_off += 37.0;
                    ui.scroll_set(sid, s.vl_off);
                    ui.request_repaint_after(0.0);
                }
                if s.vl_mode == 0 {
                    let rad = ui.look().card.radius;
                    ui.column().fill_width().bg(lumen::Token::Surface).border(lumen::Token::BorderSubtle).radius(rad).pad(6).show(|ui| {
                        ui.scroll("vl").height(avail).show(|ui| {
                            let out = ui.list_view("vl_list", &s.vl_rows, &mut s.vl_sel);
                            s.vl_built = out.built;
                            if let Some(i) = out.starred {
                                s.vl_rows[i].starred = !s.vl_rows[i].starred;
                            }
                        });
                    });
                } else {
                    let rows = &s.vl_rows;
                    let clicked = ui
                        .table("vl_table")
                        .columns([TableCol::new("Event").fill(2.0), TableCol::new("Service"), TableCol::new("Time").width(64.0).right()])
                        .lazy_rows(
                            n,
                            |i| rows[i].key.unwrap_or(i as u64) as usize,
                            |i| {
                                let r = &rows[i];
                                vec![TableCell::strong(r.title.clone()), TableCell::muted(r.subtitle.clone().unwrap_or_default()), TableCell::mono(r.meta.clone().unwrap_or_default())]
                            },
                        )
                        .height(avail)
                        .show();
                    if let Some(c) = clicked {
                        s.vl_sel = rows.iter().position(|r| r.key == Some(c as u64));
                    }
                    s.vl_built = 0;
                }
                let shown = if s.vl_mode == 0 { format!(" · {} rows built", s.vl_built) } else { String::new() };
                ui.label(format!("{} rows{shown}. ↑ ↓ Page Up/Down Home End move; type to jump.", group_digits(n))).muted().small();
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
        "action-bar" => {
            const RUNNERS: [(&str, &str); 3] = [
                ("fra-arm64-01", "Healthy · 14 jobs"),
                ("iad-x64-04", "Healthy · 9 jobs"),
                ("nrt-arm64-02", "Idle · 0 jobs"),
            ];
            ui.column().fill_width().max_width(520.0).gap(14).align(Align::Center).show(|ui| {
                ui.card().pad(10.0).show(|ui| {
                    for (i, (name, meta)) in RUNNERS.iter().enumerate() {
                        ui.row().fill_width().pad(6.0).gap(10).align(Align::Center).show(|ui| {
                            ui.scope(("r_cb", i), |ui| {
                                ui.checkbox("", &mut s.batch_sel[i]);
                            });
                            ui.label(*name).strong();
                            ui.spacer();
                            ui.label(*meta).small().muted();
                        });
                    }
                });
                let cnt = s.batch_sel.iter().filter(|&&b| b).count();
                let out = ui.floating_action_bar(
                    "runner_batch",
                    cnt,
                    &[("download", "Export", false), ("refresh-cw", "Drain", false), ("trash-2", "Terminate", true)],
                );
                if out.cleared {
                    s.batch_sel = [false; 3];
                } else if let Some(act) = out.clicked_action {
                    let msg = ["Exported selected runners", "Draining selected runners", "Terminated selected runners"][act];
                    ui.toast(Toast::new(msg).tone(Tone::Info));
                }
            });
        }
        "speed-dial" => {
            let items = [
                SpeedDialItem::new("rocket", "Promote canary").shortcut("⌘P"),
                SpeedDialItem::new("server", "Provision runner").shortcut("⌘N"),
                SpeedDialItem::new("terminal", "Open shell").shortcut("⌘J"),
            ];
            ui.row().fill_width().max_width(420.0).justify(Justify::End).pad(12.0).show(|ui| {
                if let Some(idx) = ui.speed_dial("quick_fab", &mut s.sd_open, &items) {
                    ui.toast(Toast::new(format!("Triggered “{}”", items[idx].label)).success());
                }
            });
        }
        "numeric-pad" => {
            ui.column().gap(10).align(Align::Center).show(|ui| {
                if ui.numeric_pad("vault_pad", &mut s.pin_val, 6, s.pin_masked) {
                    ui.toast(Toast::new("Hardware vault unlocked").success());
                }
                ui.switch("Mask digits", &mut s.pin_masked);
            });
        }
        "dimension-input" => {
            ui.column().fill_width().max_width(400.0).gap(14).show(|ui| {
                ui.dimension_input("max_w", "Max container width", &mut s.dim_w, &mut s.dim_w_unit);
                ui.dimension_input("side_w", "Sidebar basis", &mut s.dim_side, &mut s.dim_side_unit);
            });
        }
        "stacked-meter" => {
            let segs = [
                MeterSegment::new("Rust", 68.0, "68% · 965 KB"),
                MeterSegment::new("WGSL", 14.0, "14% · 198 KB"),
                MeterSegment::new("JavaScript", 11.0, "11% · 156 KB"),
                MeterSegment::new("Fonts", 7.0, "7% · 101 KB"),
            ];
            ui.column().fill_width().max_width(540.0).show(|ui| {
                ui.stacked_meter("bundle_breakdown", "Artifact composition", "1.42 MB total", &segs);
            });
        }
        "announcement-bar" => {
            ui.column().fill_width().max_width(600.0).gap(12).align(Align::Center).show(|ui| {
                if s.ann_open {
                    let (cta, dismissed) = ui.announcement_bar(
                        "v05_ribbon",
                        "v0.5",
                        "150 GPU-rendered Rust components across four OS looks",
                        "Explore",
                    );
                    if cta {
                        ui.toast(Toast::new("Opened v0.5 release highlights").tone(Tone::Info));
                    }
                    if dismissed {
                        s.ann_open = false;
                    }
                } else if ui.button("Restore announcement").secondary().small().clicked() {
                    s.ann_open = true;
                }
            });
        }
        "quantity-picker" => {
            ui.column().fill_width().max_width(440.0).show(|ui| {
                ui.quantity_picker("gpu_workers", "Dedicated GPU worker replicas", &mut s.replica_qty, 1, 32, &[1, 4, 8, 16]);
            });
        }
        "swatch-picker" => {
            let swatches = [
                ("Electric Blue", Color::hex(0x3B82F6)),
                ("Ethereal Violet", Color::hex(0x7F58EE)),
                ("Emerald", Color::hex(0x22B07D)),
                ("Amber", Color::hex(0xF5AF20)),
                ("Coral", Color::hex(0xF46C5F)),
                ("Rose", Color::hex(0xE0336F)),
                ("Ice Cyan", Color::hex(0x38BDF8)),
            ];
            ui.column().fill_width().max_width(440.0).show(|ui| {
                ui.swatch_picker("accent_sw", "Brand accent token", &mut s.swatch_idx, &swatches);
            });
        }
        "date-strip" => {
            let days = [
                DateStripDay::new("Mon", 16, 1),
                DateStripDay::new("Tue", 17, 2),
                DateStripDay::new("Wed", 18, 3),
                DateStripDay::new("Thu", 19, 1),
                DateStripDay::new("Fri", 20, 2),
                DateStripDay::new("Sat", 21, 0),
                DateStripDay::new("Sun", 22, 0),
            ];
            let agendas = [
                "10:00 WGSL shader audit",
                "11:00 OKLab token contrast review · 15:00 Triage",
                "10:00 WebGPU bundle trim · 14:00 Canary rollout · 16:30 Retro",
                "12:30 v0.5 RC sign-off",
                "09:30 Release freeze & crate publish",
                "No scheduled events — weekend quiet window",
                "No scheduled events — weekend quiet window",
            ];
            ui.column().fill_width().max_width(540.0).gap(12).show(|ui| {
                ui.date_strip("wk_strip", &days, &mut s.strip_day);
                let d = &days[s.strip_day.min(days.len() - 1)];
                ui.card().title(format!("{} Nov {}", d.weekday, d.day_num)).pad(14.0).show(|ui| {
                    ui.label(agendas[s.strip_day.min(agendas.len() - 1)]).secondary();
                });
            });
        }
        "world-clocks" => {
            let clocks = [
                WorldClock::new("San Francisco", "SFO", "08:30", "UTC-8", true),
                WorldClock::new("London", "LHR", "16:30", "UTC+0", true),
                WorldClock::new("Berlin", "BER", "17:30", "UTC+1", true),
                WorldClock::new("Tokyo", "HND", "01:30", "UTC+9", false),
            ];
            ui.column().fill_width().max_width(620.0).show(|ui| {
                ui.world_clocks("global_tz", &clocks);
            });
        }
        "bullet-chart" => {
            let metrics = [
                BulletMetric::new("Edge cache hit ratio", 94.0, 90.0, 100.0, "%"),
                BulletMetric::new("Sustained throughput", 142.0, 125.0, 180.0, "k/s"),
                BulletMetric::new("Cold WASM init budget", 38.0, 45.0, 80.0, " ms"),
            ];
            ui.column().fill_width().max_width(540.0).show(|ui| {
                ui.bullet_chart("slo_targets", &metrics);
            });
        }
        "metric-table" => {
            let rows = [
                MetricRow::new("edge-gateway", "eu-central-1 · TLS", vec![82.0, 86.0, 85.0, 91.0, 96.0, 104.0, 112.0], "112.4k/s", 14.8),
                MetricRow::new("glyph-atlas", "subpixel rasterizer", vec![0.24, 0.22, 0.21, 0.19, 0.19, 0.18, 0.17], "0.17 ms", 9.2),
                MetricRow::new("wasm-hydration", "client cold boot", vec![46.0, 44.0, 45.0, 42.0, 40.0, 39.0, 38.0], "38.2 ms", -4.1),
            ];
            ui.column().fill_width().max_width(580.0).show(|ui| {
                ui.metric_table("svc_watchlist", &rows);
            });
        }
        "ai-model-picker" => {
            let models = [
                AiModelSpec::new("Gemini 3.1 Pro", "Google DeepMind", "1M ctx", "$1.25 / 1M", 4, &["Reasoning", "Vision"]),
                AiModelSpec::new("Claude Opus 4.6", "Anthropic", "200k ctx", "$5.00 / 1M", 3, &["Code", "Tools"]),
                AiModelSpec::new("Gemini 3 Flash", "Google DeepMind", "1M ctx", "$0.15 / 1M", 5, &["Low-latency"]),
            ];
            ui.column().fill_width().max_width(540.0).show(|ui| {
                ui.ai_model_picker("model_cards", &models, &mut s.llm_idx);
            });
        }
        "context-meter" => {
            let slices = [
                ContextSlice::new("System instructions & tools", 8.4),
                ContextSlice::new("Indexed Rust workspace AST", 54.2),
                ContextSlice::new("Conversation trajectory", 23.4),
            ];
            ui.column().fill_width().max_width(480.0).show(|ui| {
                ui.context_meter("ctx_budget", &slices, 200.0);
            });
        }
        "api-inspector" => {
            let resp = "{\n  \"deployment_id\": \"dep_01J9ETHEREAL\",\n  \"traffic_weight\": 100,\n  \"healthy_pops\": 14\n}";
            ui.column().fill_width().max_width(560.0).show(|ui| {
                if ui.api_inspector("edge_rpc", &mut s.http_verb, &mut s.api_url, "200 OK", 14.8, resp) {
                    ui.toast(Toast::new("200 OK · 14.8 ms").success());
                }
            });
        }
        "permission-matrix" => {
            let roles = ["Admin", "Maintainer", "Dev", "Auditor"];
            let caps = ["deploys:promote", "secrets:rotate", "runners:scale", "audit:export"];
            ui.column().fill_width().max_width(580.0).show(|ui| {
                ui.permission_matrix("rbac_matrix", &roles, &caps, &mut s.rbac_grid);
            });
        }
        "description-list" => {
            let pairs = [
                DescPair::new("Cluster ID", "clu_01J9LUMEN8842").mono().copyable(),
                DescPair::new("Primary region", "eu-central-1 (Frankfurt)"),
                DescPair::new("GPU pipeline", "wgpu · Metal / WebGPU single-pass SDF"),
                DescPair::new("Ingress endpoint", "https://edge.lumen.dev").mono().copyable(),
            ];
            ui.column().fill_width().max_width(540.0).show(|ui| {
                ui.description_list("clu_details", "Cluster specification", &pairs);
            });
        }
        "masonry-grid" => {
            let cards = [
                MasonryCard::new("Shader", "64-byte vertex format", "Every rounded rect, border, Gaussian shadow and glyph shares one vertex layout.", Tone::Accent).metric("1 draw"),
                MasonryCard::new("Motion", "Closed-form springs", "Exact analytical position & velocity at 60 Hz, 120 Hz or 240 Hz.", Tone::Success),
                MasonryCard::new("Color", "Perceptual OKLab", "Theme transitions never pass through muddy sRGB midpoints.", Tone::Info),
                MasonryCard::new("Idle", "Zero background wakeups", "Once springs settle within epsilon, the event loop sleeps completely.", Tone::Success).metric("0% CPU"),
                MasonryCard::new("Targets", "Native + WebAssembly", "Metal, Vulkan, DX12 and WebGPU from one codebase.", Tone::Warning),
            ];
            ui.column().fill_width().max_width(640.0).show(|ui| {
                ui.masonry_grid("arch_masonry", 3, &cards);
            });
        }
        "collapsible-card" => {
            ui.column().fill_width().max_width(500.0).gap(10).show(|ui| {
                ui.collapsible_card("cc_tls", "Zero-trust mTLS certificates", Some("Active"), &mut s.cc_open[0], |ui| {
                    ui.label("Short-lived X.509 workload identities rotate every 24 hours across all edge PoPs.").secondary();
                    ui.switch("Enforce strict SNI verification", &mut s.on[0]);
                });
                ui.collapsible_card("cc_wh", "Webhook delivery retries", Some("3 endpoints"), &mut s.cc_open[1], |ui| {
                    ui.label("Exponential backoff over 5 attempts with HMAC-SHA256 request signing.").secondary();
                });
            });
        }
        "marquee" => {
            let items = [
                ("layers", "Single-pass WGSL"),
                ("sparkles", "Closed-form springs"),
                ("palette", "Perceptual OKLab"),
                ("cpu", "Zero idle wakeups"),
                ("globe", "Native + WebGPU"),
            ];
            ui.column().fill_width().max_width(580.0).gap(10).align(Align::Center).show(|ui| {
                ui.marquee_strip("tech_ticker", &items, s.marquee_play);
                ui.switch("Animate ticker", &mut s.marquee_play);
            });
        }
        "presence-pile" => {
            let peers = [
                PresencePeer::new("LF", "Lena Fischer", "Editing paint.rs · line 233", 0).editing(true),
                PresencePeer::new("JB", "Jonas Berg", "Inspecting flame graph", 1).editing(true),
                PresencePeer::new("PN", "Priya Nair", "Reviewing RBAC matrix", 2),
                PresencePeer::new("TR", "Tomas Rivera", "Viewing dark theme", 3),
                PresencePeer::new("AK", "Aiko Kato", "Idle · Tokyo", 0),
                PresencePeer::new("SW", "Sam Wu", "Idle · San Francisco", 1),
            ];
            ui.column().fill_width().max_width(500.0).show(|ui| {
                if let Some(idx) = ui.presence_pile("collab_bar", &peers, 4, &mut s.follow_peer) {
                    ui.toast(Toast::new(format!("Jumped to {}’s viewport", peers[idx].name)).tone(Tone::Info));
                }
            });
        }
        "comment-thread" => {
            ui.column().fill_width().max_width(540.0).show(|ui| {
                ui.comment_thread("pr_148_thread", "PR #148 · crates/lumen-core/src/paint.rs", &mut s.comments, &mut s.thread_resolved);
            });
        }
        "reaction-bar" => {
            ui.column().fill_width().max_width(520.0).gap(12).show(|ui| {
                ui.card().pad(16.0).show(|ui| {
                    ui.column().fill_width().gap(12).show(|ui| {
                        ui.label("v0.5 shipped 170 GPU-rendered components with zero idle CPU wakeups.").strong();
                        ui.reaction_bar("release_rx", &mut s.reactions);
                    });
                });
            });
        }
        "attachment-list" => {
            ui.column().fill_width().max_width(500.0).gap(10).show(|ui| {
                ui.attachment_list("build_artifacts", &mut s.attachments);
                if s.attachments.len() < 3 && ui.button("Restore sample files").ghost().small().clicked() {
                    s.attachments = vec![
                        AttachmentItem::uploaded("lumen-demos_bg.wasm", "WASM", "8.2 MB"),
                        AttachmentItem::uploaded("sdf_uber.wgsl", "WGSL", "14.6 KB"),
                        AttachmentItem::uploading("poster-atlas-2x.png", "PNG", "1.9 MB", 0.68),
                    ];
                }
            });
        }
        "progress-steps" => {
            let steps = [
                ("Connect repository", "Grant read-only GitHub webhook access to build branches automatically."),
                ("Configure WebGPU cache", "Persistent pipeline state objects trim cold startup below 12 ms."),
                ("Verify regional PoPs", "Synthetic health probes run across Frankfurt, Virginia and Tokyo."),
                ("Promote production traffic", "Shift live requests with instant automatic rollback on 5xx burn."),
            ];
            ui.column().fill_width().max_width(520.0).show(|ui| {
                ui.story_bars("onboard_story", &steps, &mut s.story_step);
            });
        }
        "shortcut-bar" => {
            let hints = [
                ShortcutHint::new(&["⌘", "Enter"], "Deploy"),
                ShortcutHint::new(&["⌘", "K"], "Actions"),
                ShortcutHint::new(&["Esc"], "Clear"),
            ];
            ui.column().fill_width().max_width(580.0).show(|ui| {
                if let Some(hi) = ui.shortcut_bar("cmd_footer", "3 runners selected", &hints) {
                    ui.toast(Toast::new(format!("Triggered “{}”", hints[hi].action)).tone(Tone::Accent));
                }
            });
        }
        "view-switcher" => {
            let modes = [
                ViewMode::new("list", "List"),
                ViewMode::new("kanban", "Board"),
                ViewMode::new("calendar", "Timeline"),
                ViewMode::new("git-Fork", "Graph"),
            ];
            let modes_clean = [
                ViewMode::new("list", "List"),
                ViewMode::new("layout-grid", "Board"),
                ViewMode::new("calendar", "Calendar"),
                ViewMode::new("git-branch", "Graph"),
            ];
            let _ = modes;
            ui.column().gap(12).align(Align::Center).show(|ui| {
                ui.view_switcher("ws_views", &modes_clean, &mut s.view_mode);
                let desc = [
                    "Compact tabular rows sorted by latency",
                    "Grouped by deployment stage across 3 columns",
                    "Scheduled rollout windows for November",
                    "Dependency DAG across 5 workspace crates",
                ][s.view_mode.min(3)];
                ui.label(desc).small().muted();
            });
        }
        "gradient-slider" => {
            let temp_stops = [
                ui.color(lumen::Token::Series0),
                ui.color(lumen::Token::Series2),
                ui.color(lumen::Token::Warning),
                ui.color(lumen::Token::Danger),
            ];
            let kelvin_stops = [
                ui.color(lumen::Token::Warning),
                ui.color(lumen::Token::Knob),
                ui.color(lumen::Token::Info),
            ];
            ui.column().fill_width().max_width(440.0).gap(16).show(|ui| {
                ui.card().pad(16.0).show(|ui| {
                    ui.column().fill_width().gap(16).show(|ui| {
                        ui.gradient_slider("sampling_temp", "Sampling temperature", &mut s.llm_temp, 0.0..=1.5, &temp_stops, "");
                        ui.gradient_slider("white_point", "Display white point", &mut s.color_warmth, 2700.0..=7500.0, &kelvin_stops, " K");
                    });
                });
            });
        }
        "schedule-picker" => {
            ui.column().fill_width().max_width(500.0).show(|ui| {
                ui.schedule_picker("backup_cron", &mut s.sched_rule);
            });
        }
        "quarter-picker" => {
            ui.column().fill_width().max_width(480.0).show(|ui| {
                ui.quarter_picker("roadmap_fy", &mut s.fy_year, &mut s.fy_quarter);
            });
        }
        "win-loss-chart" => {
            let deltas = [
                DivergingBar::new("sdf_batch_pass", 28.4, "%"),
                DivergingBar::new("glyph_subpixel", 16.2, "%"),
                DivergingBar::new("oklab_lut_mix", 9.5, "%"),
                DivergingBar::new("cold_font_scan", -6.8, "%"),
                DivergingBar::new("wasm_init_parse", -11.4, "%"),
            ];
            ui.column().fill_width().max_width(540.0).show(|ui| {
                ui.diverging_bars("bench_deltas", "v0.5 Frame Benchmark vs Baseline", &deltas, 32.0);
            });
        }
        "waffle-chart" => {
            let slices = [
                WaffleSlice::new("WebGPU / Metal", 58, Tone::Accent),
                WaffleSlice::new("Vulkan Linux", 24, Tone::Success),
                WaffleSlice::new("DirectX 12", 12, Tone::Info),
                WaffleSlice::new("Software fallback", 6, Tone::Warning),
            ];
            ui.column().fill_width().max_width(500.0).show(|ui| {
                ui.waffle_chart("gpu_fleet", "Active Renderer Backends", &slices);
            });
        }
        "schema-viewer" => {
            let cols = [
                SchemaField::new("id", "uuid").pk(),
                SchemaField::new("org_id", "uuid").fk(),
                SchemaField::new("region_code", "varchar(16)").indexed(),
                SchemaField::new("artifact_sha256", "bytea"),
                SchemaField::new("promoted_at", "timestamptz").nullable(),
            ];
            ui.column().fill_width().max_width(500.0).show(|ui| {
                ui.schema_card("deployments_tbl", "public.deployments", &cols);
            });
        }
        "env-switcher" => {
            ui.column().fill_width().max_width(560.0).show(|ui| {
                ui.env_banner("target_env", &mut s.env_idx, "eu-central-1", &mut s.env_locked);
            });
        }
        "memory-profiler" => {
            let slabs = [
                ArenaSlab::new("Vertex VBO", 14.2, 32.0),
                ArenaSlab::new("Glyph Atlas", 8.6, 16.0),
                ArenaSlab::new("Clip & Uniforms", 3.1, 8.0),
                ArenaSlab::new("Staging Ring", 4.8, 16.0),
            ];
            ui.column().fill_width().max_width(520.0).show(|ui| {
                ui.memory_arena("wgpu_arena", &slabs, 48.5);
            });
        }
        "ai-suggestion" => {
            let removed = ["let alpha = 1.0 - (dist / blur);"];
            let added = [
                "let sigma = (blur * 0.5).max(0.5);",
                "let alpha = 0.5 * erfc(dist / (sigma * SQRT_2));",
            ];
            ui.column().fill_width().max_width(560.0).show(|ui| {
                ui.code_suggestion(
                    "sdf_patch",
                    "crates/lumen-wgpu/src/shader.wgsl",
                    "Replace linear shadow falloff with analytical Gaussian error-function integration to eliminate Mach banding.",
                    &removed,
                    &added,
                    &mut s.patch_decision,
                );
            });
        }
        "prompt-templates" => {
            let templates = [
                PromptTemplate::new("cpu", "Audit frame allocations", "Perf", "Find per-frame heap allocations inside {{crate_name}} and propose stack buffers."),
                PromptTemplate::new("palette", "Check OKLab contrast", "Theme", "Verify APCA contrast of {{token_pair}} across both Light and Dark modes."),
                PromptTemplate::new("shield", "Fuzz input parser", "Security", "Generate edge-case UTF-8 grapheme clusters for {{widget_fn}}."),
                PromptTemplate::new("sparkles", "Spring motion tune", "Motion", "Tune damping & response for {{interaction}} with reduced-motion fallback."),
            ];
            ui.column().fill_width().max_width(600.0).show(|ui| {
                if let Some(ti) = ui.prompt_templates("starter_prompts", &templates) {
                    ui.toast(Toast::new(format!("Inserted “{}”", templates[ti].title)).tone(Tone::Accent));
                }
            });
        }
        "pricing-cards" => {
            let plans = [
                PlanSpec::new("Hobby", 0, 0, "For solo hackers & prototypes.", &["2 edge regions", "100k requests / mo", "Community support"]),
                PlanSpec::new("Pro", 30, 24, "Production apps & fast teams.", &["14 global regions", "10M requests / mo", "Zero-downtime rollouts"]).popular(),
                PlanSpec::new("Scale", 120, 96, "Dedicated runners & compliance.", &["Custom bare-metal", "SAML SSO & audit log", "99.99% SLA"]),
            ];
            ui.column().fill_width().max_width(660.0).show(|ui| {
                ui.pricing_cards("cloud_tiers", &plans, &mut s.annual_billing, &mut s.selected_tier_plan);
            });
        }
        "ab-comparison" => {
            ui.column().fill_width().max_width(500.0).show(|ui| {
                ui.ab_comparison(
                    "checkout_exp",
                    "Sub-millisecond WASM hydration experiment",
                    "Control (v0.4 lazy init)",
                    61.4,
                    "Variant B (streaming compile)",
                    74.8,
                    99.2,
                );
            });
        }
        "equalizer" => {
            ui.column().fill_width().max_width(560.0).show(|ui| {
                ui.graphic_eq("master_eq", &mut s.eq_bands, &mut s.eq_preset);
            });
        }
        _ => {
            ui.label(format!("Unknown demo “{demo}”.")).secondary();
        }
    }
}

fn demo_files(path: &[String]) -> Vec<lumen::FileEntry> {
    use lumen::FileEntry as F;
    match path.last().map(|s| s.as_str()) {
        Some("crates") => vec![
            F::folder("lumen-core", "Today, 09:12", 9),
            F::folder("lumen-text", "Yesterday", 8),
            F::folder("lumen-wgpu", "Yesterday", 7),
            F::folder("lumen-gpui", "Nov 20", 6),
            F::file("README.md", 4_812, "Nov 18", 5),
            F::file("Cargo.toml", 812, "Nov 17", 4),
        ],
        Some("lumen-core") => vec![
            F::folder("src", "Today, 09:12", 9),
            F::folder("tests", "Today, 08:40", 8),
            F::file("Cargo.toml", 1_204, "Nov 19", 5),
        ],
        Some("src") => vec![
            F::file("ui.rs", 96_400, "Today, 09:12", 9),
            F::file("layout.rs", 31_800, "Today, 08:57", 8),
            F::file("paint.rs", 24_100, "Yesterday", 7),
            F::file("look.rs", 41_900, "Nov 19", 6),
        ],
        _ => vec![
            F::folder("crates", "Today, 09:12", 9),
            F::folder("site", "Yesterday", 8),
            F::folder("docs", "Nov 20", 7),
            F::file("hero.png", 3_100_000, "Nov 18", 6),
            F::file("demo.m4a", 48_000_000, "Nov 12", 5),
            F::file("LICENSE-MIT", 1_070, "Nov 2", 4),
        ],
    }
}

const CODE_SAMPLE: &str = "use lumen::prelude::*;

/// A settings panel with live motion.
fn panel(ui: &mut Ui, s: &mut State) {
    ui.card().title(\"Rendering\").show(|ui| {
        // Springs settle; values never overshoot.
        ui.switch(\"Reduce motion\", &mut s.reduced);
        let fps = s.frames as f32 / s.elapsed.max(1e-3);
        ui.slider(\"Scale\", &mut s.scale, 0.5..=2.0);
        if ui.button(\"Apply\").primary().clicked() {
            s.apply(fps);
        }
    });
}
";

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
    /// Print clipped and overlapping text (with --screenshot).
    check: bool,
    /// `--click X,Y` (repeatable): clicks before the screenshot.
    clicks: Vec<lumen::Vec2>,
    chrome: bool,
    a11y: bool,
}

fn params() -> Params {
    let mut p = Params { demo: "hero".into(), dark: false, look: Platform::Lumen, accent: Accent::Blue, screenshot: None, size: (960, 600), scale: 2.0, embed: false, open: false, speed: 1.0, reduced: false, check: false, clicks: vec![], chrome: false, a11y: false };
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
                "--check" => p.check = true,
                "--chrome" => p.chrome = true,
                "--a11y" => p.a11y = true,
                "--click" => {
                    i += 1;
                    if let Some((x, y)) = args.get(i).and_then(|v| v.split_once(',')) {
                        if let (Ok(x), Ok(y)) = (x.parse(), y.parse()) {
                            p.clicks.push(lumen::vec2(x, y));
                        }
                    }
                }
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
        let mut steps = if p.open { open_steps(&p.demo) } else { vec![] };
        for &c in &p.clicks {
            steps.push(lumen::ShotStep::Click(c));
            steps.push(lumen::ShotStep::Wait(0.6));
        }
        let shot = lumen::Shot { size: p.size, scale: p.scale, mode, accent: p.accent, look, steps, check_layout: p.check, a11y: p.a11y, ..Default::default() };
        let res = if p.chrome {
            let t = DEMOS.iter().find(|d| d.0 == p.demo).map(|d| d.1).unwrap_or("lumen").to_string();
            lumen::screenshot(out, &shot, &mut state, move |ui, s| chrome_view(ui, s, &t))
        } else {
            lumen::screenshot(out, &shot, &mut state, view)
        };
        match res {
            Ok(()) => println!("wrote {out}"),
            Err(e) => {
                eprintln!("screenshot failed: {e}");
                std::process::exit(1);
            }
        }
        return;
    }
    let title = DEMOS.iter().find(|d| d.0 == p.demo).map(|d| d.1).unwrap_or("lumen");
    let app = lumen::App::new(format!("lumen — {title}")).size(p.size.0, p.size.1).mode(mode).accent(p.accent).look(look).canvas("lumen");
    if p.chrome {
        app.custom_chrome().run(state, move |ui, s| chrome_view(ui, s, title));
    } else {
        app.run(state, view);
    }
}

/// One row of the virtual-list demo: a deploy event, some with a preview
/// line, so rows have different heights.
fn virtual_row(k: u64) -> lumen::ListRow {
    const SERVICES: [&str; 6] = ["edge-proxy", "renderer", "glyph-atlas", "auth", "billing", "search"];
    const REGIONS: [&str; 4] = ["eu-central", "us-east", "ap-south", "sa-east"];
    const VERBS: [&str; 5] = ["Deployed", "Rolled back", "Scaled", "Restarted", "Promoted"];
    let h = k.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32;
    let svc = SERVICES[(h % 6) as usize];
    let mut r = lumen::ListRow::new(format!("{} {svc} #{k}", VERBS[(h / 7 % 5) as usize]))
        .key(k)
        .subtitle(REGIONS[(h / 3 % 4) as usize])
        .meta(format!("{:02}:{:02}", (h / 11) % 24, (h / 13) % 60))
        .icon(["rocket", "rotate-ccw", "layers", "refresh-cw", "arrow-up-circle"][(h / 7 % 5) as usize]);
    if h % 3 == 0 {
        r = r.preview(format!("{} replicas healthy · p95 {} ms · build {:x}", 2 + h % 9, 40 + h % 160, h & 0xFFFF));
    }
    if h % 11 == 0 {
        r = r.badge("canary", Tone::Warning);
    }
    r.unread(h % 5 == 0)
}

fn group_digits(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// `--chrome`: the demo inside lumen's own title bar instead of the OS's.
fn chrome_view(ui: &mut Ui, s: &mut State, title: &str) {
    ui.window_chrome("app_chrome", title).window().show(|ui| view(ui, s));
}
