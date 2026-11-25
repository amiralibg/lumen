//! lumen-core: the immediate-mode heart of lumen.
//!
//! Owns ids, input, layout, theme tokens and springs, and produces
//! [`paint::DrawData`] (triangles plus clip commands). It never touches a
//! window or GPU.

pub mod anim;
pub mod color;
pub mod date;
pub mod id;
pub mod input;
pub mod layout;
pub mod look;
pub mod math;
pub mod paint;
pub mod theme;

pub use color::Color;
pub use date::{Date, Time};
pub use id::Id;
pub use math::{vec2, Rect, Sides, Vec2};
pub use theme::{Accent, Mode, Token};
pub use look::{Density, Fonts, Look, Motion, Neutrals, Platform};

/// Hash map with a fast, deterministic hasher (no runtime RNG, works on wasm).
pub type FastMap<K, V> = std::collections::HashMap<K, V, std::hash::BuildHasherDefault<ahash::AHasher>>;

pub mod ui;
pub mod widgets;

pub use anim::{Ease, SpringParams};
pub use input::{CursorIcon, Event, Ime, Key, Modifiers, PointerButton, RawInput};
pub use layout::{Align, Justify, Size};
pub use theme::{springs, ControlSize, TextRole};
pub use ui::{FrameOutput, LayerAlign, LayerPlace, Placement, Response, ScrollInfo, ScrollX, Shown, SystemPrefs, TextFieldState, Ui, WindowRegion, WindowRequest};
pub use widgets::{
    password_strength, ActionBarOut, AiModelSpec, ArenaSlab, AttachmentItem, BarListItem, BulletMetric, Candle,
    CarouselSlide, ChartKind, ChromeResponse, ChromeShown, ChatMessage, ChatRole, ChecklistStep, CitationSource, Command, CommentEntry, CommitRow,
    ComposerOut, ContextSlice, Currency, DateStripDay, DescPair, DimensionUnit, DivergingBar, DockItem, EditorTab,
    EditorTabsOut, FilterFacet, FlameSpan, GanttTask, GraphNode, GraphWire, GridColumn, HistogramMarker, HttpMethod,
    ImageStatus, KanbanCard, KanbanCol, KpiMetric, KvPair, ListOut, ListRow, TourStep, FileEntry, FileOut, LogEntry, LogLevel, MasonryCard, MentionCandidate,
    MeterSegment, MetricRow, NavBarItem, NotificationItem, PlanSpec, PresencePeer, PromptTemplate, ReactionChip,
    ReorderItem, RingSpec, SankeyLink, ScatterPoint, ScatterSeries, ScheduleEvent, ScheduleRule, SchemaField, Series,
    ShortcutHint, SortDir, SpeedDialItem, SuggestionDecision, TableCell, TableCol, TermLine, TermTone, TestCase,
    TestState, TimelineItem, Toast, TocItem, Tone, ToolCall, ToolStatus, TraceSpan, TreeNode, TreemapItem, UndoBarOut,
    UptimeStatus, Variant, ViewMode, WaffleSlice, WorldClock,
};
