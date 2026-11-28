//! The immediate-mode context.
//!
//! Each frame the user's closure calls widget functions on [`Ui`]. Widgets
//! read interaction state computed from the *previous* frame's layout (hit
//! rects) plus this frame's input, record nodes into a layout tree, and
//! attach paint closures. After the closure returns, the tree is laid out,
//! painted into [`DrawData`], and this frame's hit rects are stored for the
//! next frame.

use std::any::Any;
use std::sync::Arc;

use lumen_text::{Galley, TextAlign, TextStyle, TextSystem};

use crate::a11y::{A11y, A11yAction, A11yNode, A11yTree, Role};
use crate::anim::{AnimStore, Ease, SpringParams};
use crate::color::Color;
use crate::id::Id;
use crate::input::{CursorIcon, Event, Key, Modifiers, PointerButton, RawInput};
use crate::layout::{Align, Content, Dir, Justify, NodeId, Size, Style, Tree};
use crate::math::{vec2, Corners, Rect, Sides, Vec2};
use crate::paint::{DrawData, Painter};
use crate::look::{Fonts, Look, Motion};
use crate::theme::{durations, Accent, ControlMetrics, ControlSize, Elevation, Mode, TextRole, Theme, Token};
use crate::FastMap;

/// What a widget wants from the pointer and keyboard.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sense {
    pub click: bool,
    pub drag: bool,
    pub focusable: bool,
    pub scroll: bool,
}

impl Sense {
    pub const NONE: Sense = Sense { click: false, drag: false, focusable: false, scroll: false };
    pub const HOVER: Sense = Sense::NONE;
    pub const CLICK: Sense = Sense { click: true, drag: false, focusable: true, scroll: false };
    pub const DRAG: Sense = Sense { click: true, drag: true, focusable: true, scroll: false };
}

/// Interaction state of a widget this frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct Interact {
    pub id: Id,
    /// The widget's rect from the previous frame (zero on first frame).
    pub rect: Rect,
    pub hovered: bool,
    /// Pointer is down on this widget and still over it.
    pub pressed: bool,
    /// Pointer went down on this widget and has not been released.
    pub held: bool,
    pub clicked: bool,
    /// Pointer went down on this widget this frame.
    pub press_started: bool,
    pub released: bool,
    pub pointer: Option<Vec2>,
    pub focused: bool,
    /// Focused and the focus came from the keyboard.
    pub focus_visible: bool,
    pub disabled: bool,
    /// Click count of the latest press (2 = double click, 3 = triple).
    pub click_count: u32,
}

/// Returned by widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct Response {
    pub id: Id,
    pub rect: Rect,
    pub hovered: bool,
    pub clicked: bool,
    pub changed: bool,
    pub focused: bool,
}

impl Response {
    pub fn clicked(&self) -> bool {
        self.clicked
    }
    pub fn changed(&self) -> bool {
        self.changed
    }
    pub fn hovered(&self) -> bool {
        self.hovered
    }
    pub fn has_focus(&self) -> bool {
        self.focused
    }
}

/// Result of showing a container.
pub struct Shown<R> {
    pub inner: R,
    pub id: Id,
    /// Rect from the previous frame.
    pub rect: Rect,
}

impl<R> Shown<R> {
    /// A response for the container (id and last frame's rect), to anchor
    /// popovers, hover cards and context menus to it.
    pub fn response(&self) -> Response {
        Response { id: self.id, rect: self.rect, ..Default::default() }
    }
}

/// Painting context handed to paint closures after layout.
pub struct PaintCx<'a> {
    pub painter: &'a mut Painter,
    pub text: &'a mut TextSystem,
    pub theme: &'a Theme,
    pub anims: &'a mut AnimStore,
    /// Rects of this frame's identified nodes.
    pub rects: &'a FastMap<Id, Rect>,
    pub time: f64,
    deferred: &'a mut Vec<(Rect, PaintFn)>,
    scroll_clip: Rect,
    /// Text each node's paint closures drew (`node, log range`), recorded
    /// while accessibility is on: it names controls that paint their text.
    painted: Option<Vec<(NodeId, usize, usize)>>,
}

pub type PaintFn = Box<dyn FnOnce(&mut PaintCx<'_>, Rect)>;

impl PaintCx<'_> {
    /// The part of the screen this node can show (its scroll viewport, or
    /// the screen): tall custom-painted content paints only what's in it.
    pub fn visible_rect(&self) -> Rect {
        self.scroll_clip
    }

    pub fn color(&self, t: Token) -> Color {
        self.theme.color(t)
    }

    pub fn galley(&mut self, text: &str, style: &TextStyle) -> Arc<Galley> {
        self.text.layout(text, style, None, TextAlign::Start)
    }

    /// One line of text, shortened with an ellipsis when it's wider than
    /// `max_width`. Use it instead of clipping, which cuts glyphs in half.
    pub fn galley_fit(&mut self, text: &str, style: &TextStyle, max_width: f32) -> Arc<Galley> {
        let g = self.galley(text, style);
        if g.width <= max_width || text.is_empty() {
            g
        } else {
            truncate(self.text, text, style, max_width.max(0.0))
        }
    }

    /// Paint later, above the rest of the tree, clipped only by the nearest
    /// scroll container (focus rings, pills that must not be covered).
    pub fn defer(&mut self, rect: Rect, f: PaintFn) {
        self.deferred.push((self.scroll_clip, Box::new(move |cx, _| f(cx, rect))));
    }

    /// Draw a Lucide icon centered at `c` with em size `size`.
    pub fn icon(&mut self, name: &str, c: Vec2, size: f32, color: Color) {
        if let Some(ch) = self.text.icon_char(name) {
            let st = TextStyle { family: lumen_text::FontFamily::Icons, size, line_height: size, ..Default::default() };
            let g = self.text.layout(ch.encode_utf8(&mut [0; 4]), &st, None, TextAlign::Start);
            let pos = vec2(c.x - g.width / 2.0, c.y - size / 2.0);
            // Snap to the pixel grid so 1.5px strokes stay crisp.
            let pos = vec2(self.painter.snap(pos.x), self.painter.snap(pos.y));
            self.painter.text(self.text, &g, pos, color);
        }
    }

    /// Standard focus ring around `r` (DESIGN.md 6). `t` is the animation
    /// value (0 hidden .. 1 shown); `gap` comes from [`Ui::focus_anim`].
    /// Width follows the look.
    pub fn focus_ring(&mut self, r: Rect, radius: f32, t: f32, gap: f32, color: Color) {
        if t <= 0.0 {
            return;
        }
        let w = self.theme.look().focus.width;
        let ring = r.expand(gap + w);
        self.painter.ring(ring, Corners::all((radius + gap + w).max(w)), w, color.multiply_alpha(t));
    }

    /// Current look (blended).
    pub fn look(&self) -> &crate::look::Look {
        self.theme.look()
    }
}

struct NodeExtra {
    id: Option<Id>,
    sense: Sense,
    cursor: CursorIcon,
    clip: bool,
    /// Extra vertical hit area (px above and below).
    hit_pad: f32,
    paint: Option<PaintFn>,
    paint_after: Option<PaintFn>,
}

impl Default for NodeExtra {
    fn default() -> Self {
        NodeExtra { id: None, sense: Sense::NONE, cursor: CursorIcon::Default, clip: false, hit_pad: 0.0, paint: None, paint_after: None }
    }
}

/// A text leaf measured during layout and painted after it.
struct TextLeaf {
    text: String,
    style: TextStyle,
    color: Color,
    wrap: bool,
    truncate: bool,
    align: TextAlign,
}

#[derive(Clone, Copy)]
struct Hit {
    id: Id,
    rect: Rect,
    sense: Sense,
    cursor: CursorIcon,
}

/// Where a floating layer sits relative to its anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    Top,
    Bottom,
    Right,
    Left,
}

/// How a layer's content lines up with its anchor along the anchor's edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LayerAlign {
    #[default]
    Start,
    Center,
    End,
}

/// Where a [`Ui::layer`] goes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayerPlace {
    /// Next to a widget laid out this frame, flipping when it doesn't fit.
    Anchor { anchor: Id, placement: Placement, gap: f32, align: LayerAlign, match_width: bool },
    /// The whole window (dialogs, toasts). The root fills the screen.
    Screen,
    /// At a point (context menus): below and to the right of it, flipping
    /// when it doesn't fit.
    Point(Vec2),
}

struct Layer {
    root: NodeId,
    place: LayerPlace,
    offset: Vec2,
    alpha: f32,
}

struct Overlay {
    root: NodeId,
    anchor: Id,
    placement: Placement,
    gap: f32,
    /// Translate in from the anchor side (px), animated.
    slide: f32,
    alpha: f32,
}

/// Tooltip request registered by a widget this frame.
#[derive(Clone)]
pub struct TooltipSpec {
    pub text: String,
    pub shortcut: Option<String>,
    pub placement: Placement,
    pub gap: f32,
}

struct TipState {
    anchor: Id,
    spec: TooltipSpec,
    opened_at: f64,
    /// Time it started closing.
    closing_at: Option<f64>,
    warm: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ScrollState {
    pub(crate) offset: f32,
    content: f32,
    viewport: f32,
    last_scroll_time: f64,
    /// Programmatic target, animated with the gentle spring.
    pub(crate) target: Option<f32>,
}

/// Persistent state of a [`Ui::scroll_row`].
#[derive(Clone, Copy, Default)]
pub(crate) struct ScrollXState {
    offset: f32,
    content: f32,
    viewport: f32,
    target: Option<f32>,
}

/// What a [`Ui::scroll_row`] showed: its id (for
/// [`Ui::scroll_row_reveal`]), offset and how far it can scroll.
#[derive(Clone, Copy, Debug, Default)]
pub struct ScrollX {
    pub id: Id,
    pub offset: f32,
    /// 0 when everything fits.
    pub max: f32,
}

impl Default for ScrollState {
    fn default() -> Self {
        ScrollState { offset: 0.0, content: 0.0, viewport: 0.0, last_scroll_time: -10.0, target: None }
    }
}

/// Information returned by a scroll area.
#[derive(Clone, Copy, Debug, Default)]
pub struct ScrollInfo {
    pub offset: f32,
    pub content_height: f32,
    pub viewport_height: f32,
}

/// A part of a custom window title bar ([`Ui::window_chrome`] with
/// `.window()`). Hosts map these to the platform: a drag region moves the
/// window and maximizes it on double-click, the others are caption buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WindowRegion {
    Drag,
    Close,
    Minimize,
    Maximize,
}

/// Something the UI asks the host window to do, for example from a caption
/// button. Hosts without a native window (the web) ignore them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowRequest {
    Close,
    Minimize,
    ToggleMaximize,
}

/// Per-frame output for the app layer.
pub struct FrameOutput {
    pub draw: DrawData,
    pub cursor: CursorIcon,
    /// Seconds until the next frame is needed (`Some(0.0)` = asap).
    pub repaint_after: Option<f64>,
    /// Text the user copied; the app puts it on the system clipboard.
    pub copied_text: Option<String>,
    /// Text input is focused (show the IME / virtual keyboard).
    pub wants_text_input: bool,
    /// The focused text field's value, selection, composition and caret,
    /// for hosts that run a platform input method (IME candidate window
    /// placement, GPUI's input handler). `None` for password fields.
    pub text_field: Option<TextFieldState>,
    /// Bounds of the visible menus, popovers, context menus and tooltips
    /// (not dialogs or toasts), in logical px. With a
    /// [content rect](Ui::set_content_rect) these may lie outside it.
    pub layer_rects: Vec<Rect>,
    /// Springs, fades or the theme blend are still moving (as opposed to a
    /// timer such as a tooltip delay or caret blink).
    pub animating: bool,
    /// Window actions asked for this frame (caption buttons).
    pub window_requests: Vec<WindowRequest>,
    /// Custom title bar regions in logical px, buttons before the drag
    /// region. Hosts that need hit-test areas up front (GPUI on Windows)
    /// use these; others call [`Ui::window_region_at`] on mouse-down.
    pub window_regions: Vec<(WindowRegion, Rect)>,
    /// The accessibility tree, when [`Ui::set_accessibility`] is on.
    pub a11y: Option<A11yTree>,
}

/// The focused text field as the platform's input method sees it. Offsets
/// are bytes into `text`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextFieldState {
    /// The value, including any composition text.
    pub text: String,
    /// Selection: `cursor` is the end that moves, `anchor` the fixed one.
    pub cursor: usize,
    pub anchor: usize,
    /// The composition (marked text), if any.
    pub marked: Option<std::ops::Range<usize>>,
    /// The composition, or the caret when not composing, in logical pixels.
    pub rect: Rect,
}

/// Text typed into the focused field this frame, in order.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TextIn {
    Text(String),
    Ime(crate::input::Ime),
}

/// What the OS reports about the user's preferences, filled in by the app
/// layer (`None` = unknown).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SystemPrefs {
    pub dark: Option<bool>,
    pub accent: Option<Color>,
    pub reduced_motion: Option<bool>,
    /// Title bar buttons the desktop shows, `[close, minimize, maximize]`
    /// (GNOME's `button-layout`; stock GNOME shows only close). A window
    /// title bar in the GNOME look follows it.
    pub title_buttons: Option<[bool; 3]>,
}

/// The UI context. Persistent across frames.
pub struct Ui {
    /// Blink the text caret (on by default; screenshots turn it off).
    pub caret_blink: bool,
    /// Record text draws and report clipped and overlapping text in
    /// [`layout_issues`](Self::layout_issues) (debugging and visual tests;
    /// costs time, off by default).
    pub check_layout: bool,
    issues: Vec<String>,
    /// Fill the background with `Token::Background` (on by default). Hosts
    /// that composite lumen over their own content turn it off.
    pub paint_background: bool,
    /// Where the main UI lays out inside the screen (see
    /// [`set_content_rect`](Self::set_content_rect)).
    content: Option<Rect>,
    pub text: TextSystem,
    pub theme: Theme,
    pub anims: AnimStore,
    memory: FastMap<(Id, std::any::TypeId), (Box<dyn Any>, u64)>,
    frame: u64,
    text_focused_prev: bool,

    // Input state.
    time: f64,
    dt: f32,
    screen: Vec2,
    scale: f32,
    pointer: Option<Vec2>,
    pointer_down: bool,
    press_pos: Vec2,
    last_click: (f64, Vec2, u32),
    click_count: u32,
    window_focused: bool,
    window_maximized: bool,
    window_region_ids: Vec<(WindowRegion, Id)>,
    prev_window_regions: Vec<(WindowRegion, Rect)>,
    prev_layer_rects: Vec<Rect>,
    window_requests: Vec<WindowRequest>,
    active: Option<Id>,
    focused: Option<Id>,
    focus_visible: bool,
    hovered: Option<Id>,
    clicked: Vec<Id>,
    press_started: Option<Id>,
    released: Option<Id>,
    keys: Vec<(Key, Modifiers, bool)>,
    keys_down: Vec<Key>,
    text_input: Vec<TextIn>,
    text_field: Option<TextFieldState>,
    /// A focused widget that takes Tab itself (a code editor), set each
    /// frame by [`Ui::capture_tab`].
    tab_capture: Option<Id>,
    tab_capture_next: Option<Id>,
    scroll_delta: Vec2,
    scroll_target: Option<Id>,
    modifiers: Modifiers,
    copied: Option<String>,
    wants_text_input: bool,
    pending_focus: Option<Id>,
    any_press: bool,
    any_key: bool,

    // Previous frame.
    prev_hits: Vec<Hit>,
    prev_rects: FastMap<Id, Rect>,
    prev_focus_order: Vec<Id>,

    // This frame.
    tree: Tree,
    extras: Vec<NodeExtra>,
    leaves: Vec<TextLeaf>,
    stack: Vec<NodeId>,
    id_scope: Vec<Id>,
    seen: FastMap<Id, u32>,
    focus_order: Vec<Id>,
    overlays: Vec<Overlay>,
    layers: Vec<Layer>,
    layer_close: bool,
    press_at: Option<Vec2>,
    secondary_at: Option<Vec2>,
    pub(crate) toasts: Vec<crate::widgets::toast::ToastState>,
    pub(crate) menu: Option<crate::widgets::menu::MenuCtx>,
    tooltips: FastMap<Id, TooltipSpec>,
    tip: Option<TipState>,
    tip_last_closed: f64,
    hover_since: Option<(Id, f64)>,
    scrolls: Vec<(Id, NodeId)>,
    /// Open vertical scroll areas while the view runs: id and how far the
    /// offset moved this frame (for virtualized rows inside them).
    pub(crate) scroll_ctx: Vec<(Id, f32)>,
    /// Horizontal scroll rows this frame, and last frame's ids (sideways
    /// wheel input goes to them; vertical input passes through).
    scrolls_x: Vec<(Id, NodeId)>,
    prev_scrolls_x: Vec<Id>,
    repaint_at: Option<f64>,
    root: NodeId,
    frame_started: bool,
    pub(crate) nav: Option<crate::widgets::nav::NavCtx>,
    applied_fonts: Option<Fonts>,
    system: SystemPrefs,

    // Accessibility (see `a11y`).
    a11y_on: bool,
    a11y_by_id: FastMap<Id, A11y>,
    a11y_by_node: Vec<(NodeId, A11y)>,
    a11y_requests: Vec<(Id, A11yAction)>,
    /// Icon leaves this frame: a name for controls that show only an icon.
    a11y_icons: FastMap<NodeId, String>,
    a11y_painted: FastMap<NodeId, Vec<String>>,
    /// A field label waiting for its editor (see `field_label`).
    pub(crate) a11y_field_label: Option<(String, NodeId)>,
    /// Last frame: expanded state and scroll context per exposed widget.
    a11y_expanded: FastMap<Id, bool>,
    a11y_scroll: FastMap<Id, (Id, Rect, Rect)>,
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}

impl Ui {
    pub fn new() -> Self {
        Ui {
            caret_blink: true,
            paint_background: true,
            check_layout: false,
            issues: vec![],
            content: None,
            text: TextSystem::new(),
            theme: Theme::new(Mode::Light, Accent::Blue),
            anims: AnimStore::new(),
            memory: FastMap::default(),
            frame: 0,
            text_focused_prev: false,
            time: 0.0,
            dt: 0.0,
            screen: vec2(800.0, 600.0),
            scale: 1.0,
            pointer: None,
            pointer_down: false,
            press_pos: Vec2::ZERO,
            last_click: (-10.0, Vec2::ZERO, 0),
            click_count: 0,
            window_focused: true,
            window_maximized: false,
            window_region_ids: vec![],
            prev_window_regions: vec![],
            prev_layer_rects: vec![],
            window_requests: vec![],
            active: None,
            focused: None,
            focus_visible: false,
            hovered: None,
            clicked: vec![],
            press_started: None,
            released: None,
            keys: vec![],
            keys_down: vec![],
            text_input: vec![],
            text_field: None,
            tab_capture: None,
            tab_capture_next: None,
            scroll_delta: Vec2::ZERO,
            scroll_target: None,
            modifiers: Modifiers::default(),
            copied: None,
            wants_text_input: false,
            pending_focus: None,
            any_press: false,
            any_key: false,
            prev_hits: vec![],
            prev_rects: FastMap::default(),
            prev_focus_order: vec![],
            tree: Tree::new(),
            extras: vec![],
            leaves: vec![],
            stack: vec![],
            id_scope: vec![],
            seen: FastMap::default(),
            focus_order: vec![],
            overlays: vec![],
            layers: vec![],
            layer_close: false,
            press_at: None,
            secondary_at: None,
            toasts: vec![],
            menu: None,
            tooltips: FastMap::default(),
            tip: None,
            tip_last_closed: -10.0,
            hover_since: None,
            scrolls: vec![],
            scroll_ctx: vec![],
            scrolls_x: vec![],
            prev_scrolls_x: vec![],
            repaint_at: None,
            root: 0,
            frame_started: false,
            nav: None,
            applied_fonts: None,
            system: SystemPrefs::default(),
            a11y_on: false,
            a11y_by_id: FastMap::default(),
            a11y_by_node: vec![],
            a11y_requests: vec![],
            a11y_icons: FastMap::default(),
            a11y_painted: FastMap::default(),
            a11y_field_label: None,
            a11y_expanded: FastMap::default(),
            a11y_scroll: FastMap::default(),
        }
    }

    // ---------------------------------------------------------------- frame

    /// Start a frame: consume input and open the root container.
    pub fn begin_frame(&mut self, input: RawInput) {
        let dt = if self.frame_started || self.time > 0.0 { (input.time - self.time).max(0.0) as f32 } else { 0.0 };
        self.dt = dt;
        self.time = input.time;
        self.screen = input.screen_size;
        self.scale = input.scale;
        self.modifiers = input.modifiers;
        self.anims.reduced_motion = self.theme.reduced_motion;
        self.anims.time_scale = self.theme.motion_scale;
        self.anims.begin_frame(dt);
        self.theme.update(self.time);
        let fonts = self.theme.target_look().fonts;
        if self.applied_fonts != Some(fonts) {
            self.apply_fonts(fonts);
        }

        self.clicked.clear();
        self.press_started = None;
        self.released = None;
        self.keys.clear();
        self.text_input.clear();
        self.scroll_delta = Vec2::ZERO;
        self.scroll_target = None;
        self.copied = None;
        self.wants_text_input = false;
        self.text_field = None;
        self.tab_capture = self.tab_capture_next.take();
        self.any_press = false;
        self.any_key = false;
        self.press_at = None;
        self.secondary_at = None;
        self.a11y_requests.clear();
        self.a11y_by_id.clear();
        self.a11y_by_node.clear();
        self.a11y_icons.clear();
        self.a11y_field_label = None;
        if let Some(f) = self.pending_focus.take() {
            self.focused = Some(f);
        }

        for ev in input.events {
            self.process_event(ev);
        }
        self.hovered = self.pointer.and_then(|p| self.hit_test(p)).map(|h| h.id);
        if let Some(a) = self.active {
            if self.hovered != Some(a) {
                self.hovered = None;
            }
        }
        // Keep the active widget alive only while it still exists.
        if let Some(a) = self.active {
            if !self.prev_rects.contains_key(&a) {
                self.active = None;
            }
        }

        // Reset per-frame recording.
        self.tree.clear();
        self.extras.clear();
        self.leaves.clear();
        self.stack.clear();
        self.id_scope.clear();
        self.id_scope.push(Id::ROOT);
        self.seen.clear();
        self.focus_order.clear();
        self.overlays.clear();
        self.layers.clear();
        self.layer_close = false;
        self.tooltips.clear();
        self.scrolls.clear();
        self.scroll_ctx.clear();
        self.scrolls_x.clear();
        self.repaint_at = None;
        let root_style = Style { width: Size::FILL, height: Size::FILL, ..Default::default() };
        self.root = self.push_node(root_style, Content::None, NodeExtra::default());
        self.stack.push(self.root);
        self.frame_started = true;
    }

    fn hit_test(&self, p: Vec2) -> Option<Hit> {
        self.prev_hits.iter().rev().find(|h| h.rect.contains(p) && (h.sense.click || h.sense.drag)).copied()
    }

    fn process_event(&mut self, ev: Event) {
        match ev {
            Event::PointerMoved(p) => self.pointer = Some(p),
            Event::PointerLeft => {
                if !self.pointer_down {
                    self.pointer = None;
                }
            }
            Event::PointerButton { pos, button: PointerButton::Primary, pressed: true } => {
                self.pointer = Some(pos);
                self.pointer_down = true;
                self.press_pos = pos;
                self.any_press = true;
                self.press_at = Some(pos);
                let (t, p, n) = self.last_click;
                self.click_count = if self.time - t < 0.4 && (p - pos).length() < 6.0 { n + 1 } else { 1 };
                self.last_click = (self.time, pos, self.click_count);
                let hit = self.hit_test(pos);
                self.active = hit.map(|h| h.id);
                self.press_started = self.active;
                match hit {
                    Some(h) if h.sense.focusable => {
                        if self.focused != Some(h.id) {
                            self.focus_visible = false;
                        }
                        self.focused = Some(h.id);
                    }
                    Some(_) => {}
                    None => {
                        self.focused = None;
                        self.focus_visible = false;
                    }
                }
            }
            Event::PointerButton { pos, button: PointerButton::Primary, pressed: false } => {
                self.pointer = Some(pos);
                self.pointer_down = false;
                if let Some(a) = self.active.take() {
                    self.released = Some(a);
                    if self.hit_test(pos).map(|h| h.id) == Some(a) {
                        self.clicked.push(a);
                    }
                }
            }
            Event::PointerButton { pos, button: PointerButton::Secondary, pressed: true } => {
                self.pointer = Some(pos);
                self.any_press = true;
                self.press_at = Some(pos);
                self.secondary_at = Some(pos);
            }
            Event::PointerButton { .. } => {}
            Event::Scroll(d) => {
                self.scroll_delta += d;
                self.any_press = true;
                if let Some(p) = self.pointer {
                    // A horizontal scroll row only takes sideways input
                    // (trackpad, or Shift + wheel).
                    let sideways = d.x.abs() > d.y.abs() || self.modifiers.shift;
                    let hx = &self.prev_scrolls_x;
                    self.scroll_target =
                        self.prev_hits.iter().rev().find(|h| h.sense.scroll && h.rect.contains(p) && (sideways || !hx.contains(&h.id))).map(|h| h.id);
                }
            }
            Event::Key { key, pressed, modifiers, repeat } => {
                self.modifiers = modifiers;
                if pressed {
                    self.any_key = true;
                    if !self.keys_down.contains(&key) {
                        self.keys_down.push(key);
                    }
                    let captured = self.tab_capture.is_some() && self.tab_capture == self.focused;
                    if key == Key::Tab && !modifiers.command() && !captured {
                        self.move_focus(!modifiers.shift);
                    } else if key == Key::Escape && self.tip.is_some() {
                        // Escape closes tooltips; still delivered to widgets.
                    }
                    self.keys.push((key, modifiers, repeat));
                } else {
                    self.keys_down.retain(|k| *k != key);
                }
            }
            Event::Text(t) | Event::Paste(t) => self.text_input.push(TextIn::Text(t)),
            Event::Ime(e) => self.text_input.push(TextIn::Ime(e)),
            Event::WindowFocused(false) => {
                self.window_focused = false;
                self.keys_down.clear();
                self.pointer_down = false;
                self.active = None;
            }
            Event::WindowFocused(true) => self.window_focused = true,
            Event::WindowMaximized(m) => self.window_maximized = m,
            Event::A11y { target, action } => self.a11y_event(target, action),
        }
    }

    fn a11y_event(&mut self, target: Id, action: A11yAction) {
        let focusable = self.prev_focus_order.contains(&target);
        match &action {
            A11yAction::Click => {
                self.clicked.push(target);
                self.any_press = true;
                if focusable {
                    self.focused = Some(target);
                    self.focus_visible = true;
                }
            }
            A11yAction::Expand | A11yAction::Collapse => {
                let open = matches!(action, A11yAction::Expand);
                if self.a11y_expanded.get(&target).copied() != Some(open) {
                    self.clicked.push(target);
                    self.any_press = true;
                }
            }
            A11yAction::Focus => {
                self.focused = Some(target);
                self.focus_visible = true;
            }
            A11yAction::Blur => {
                if self.focused == Some(target) {
                    self.focused = None;
                }
            }
            A11yAction::Increment | A11yAction::Decrement | A11yAction::SetValue(_) | A11yAction::SetNumber(_) => {
                if focusable {
                    self.focused = Some(target);
                    self.focus_visible = true;
                }
            }
            A11yAction::ScrollIntoView => {
                if let Some(&(scroll, r, view)) = self.a11y_scroll.get(&target) {
                    let st = self.mem::<ScrollState>(scroll);
                    let off = st.offset;
                    let d = if r.min.y < view.min.y {
                        r.min.y - view.min.y
                    } else if r.max.y > view.max.y {
                        (r.max.y - view.max.y).min(r.min.y - view.min.y)
                    } else {
                        0.0
                    };
                    if d != 0.0 {
                        self.scroll_to(scroll, off + d);
                    }
                }
            }
        }
        self.a11y_requests.push((target, action));
    }

    fn move_focus(&mut self, forward: bool) {
        let order = &self.prev_focus_order;
        if order.is_empty() {
            return;
        }
        let idx = self.focused.and_then(|f| order.iter().position(|&o| o == f));
        let next = match (idx, forward) {
            (None, true) => 0,
            (None, false) => order.len() - 1,
            (Some(i), true) => (i + 1) % order.len(),
            (Some(i), false) => (i + order.len() - 1) % order.len(),
        };
        self.focused = Some(order[next]);
        self.focus_visible = true;
    }

    /// Finish the frame: lay out, paint, and return draw data.
    pub fn end_frame(&mut self) -> FrameOutput {
        assert!(self.frame_started, "end_frame without begin_frame");
        self.build_toasts();
        self.frame_started = false;
        self.update_tooltip();

        // Apply scroll offsets from memory before layout.
        for (id, node) in self.scrolls.clone() {
            let st = self.mem::<ScrollState>(id);
            self.tree.nodes[node].scroll_offset = st.offset;
        }
        for (id, node) in self.scrolls_x.clone() {
            self.tree.nodes[node].scroll_offset_x = self.mem::<ScrollXState>(id).offset;
        }

        // Layout. The main tree and full-screen layers use the content
        // rect; anchored layers and overlays may use the whole screen.
        let full = Rect::from_min_size(Vec2::ZERO, self.screen);
        let screen = self.content_rect();
        {
            let leaves = &self.leaves;
            let text = &mut self.text;
            let mut measure = |k: u32, max: Option<f32>| {
                let l = &leaves[k as usize];
                let wrap = if l.wrap { max } else { None };
                let g = text.layout(&l.text, &l.style, wrap, TextAlign::Start);
                vec2(g.width.ceil(), g.height)
            };
            self.tree.layout(self.root, screen, &mut measure);
            for ly in &self.layers {
                match ly.place {
                    LayerPlace::Screen => self.tree.layout(ly.root, screen.translate(ly.offset), &mut measure),
                    LayerPlace::Point(p) => {
                        let s = self.tree.measure_root(ly.root, (self.screen.x - 16.0).max(0.0), &mut measure);
                        let pos = place_layer(Rect::from_min_size(p, Vec2::ZERO), s, Placement::Bottom, 2.0, LayerAlign::Start, self.screen);
                        self.tree.layout(ly.root, Rect::from_min_size(pos + ly.offset, s), &mut measure);
                    }
                    LayerPlace::Anchor { anchor, placement, gap, align, match_width } => {
                        let a = rect_of_id(&self.tree, &self.extras, anchor).unwrap_or(Rect::ZERO);
                        if match_width {
                            let st = &mut self.tree.nodes[ly.root].style;
                            st.min.x = st.min.x.max(a.width());
                        }
                        let s = self.tree.measure_root(ly.root, self.screen.x - 16.0, &mut measure);
                        let pos = place_layer(a, s, placement, gap, align, self.screen);
                        self.tree.layout(ly.root, Rect::from_min_size(pos + ly.offset, s), &mut measure);
                    }
                }
            }
            for ov in &self.overlays {
                let s = self.tree.measure_root(ov.root, self.screen.x, &mut measure);
                let anchor = rect_of_id(&self.tree, &self.extras, ov.anchor).unwrap_or(Rect::ZERO);
                let pos = place_overlay(anchor, s, ov.placement, ov.gap, self.screen);
                self.tree.layout(ov.root, Rect::from_min_size(pos, s), &mut measure);
            }
        }

        // Update scroll memory with the measured content size.
        for (id, node) in self.scrolls.clone() {
            let n = &self.tree.nodes[node];
            let (content, viewport) = (n.content_size.y, n.rect.height());
            let st = self.mem::<ScrollState>(id);
            st.content = content;
            st.viewport = viewport;
            st.offset = st.offset.clamp(0.0, (content - viewport).max(0.0));
        }
        for (id, node) in self.scrolls_x.clone() {
            let n = &self.tree.nodes[node];
            let (content, viewport) = (n.content_w, n.rect.width());
            let st = self.mem::<ScrollXState>(id);
            st.content = content;
            st.viewport = viewport;
            st.offset = st.offset.clamp(0.0, (content - viewport).max(0.0));
        }
        self.prev_scrolls_x = self.scrolls_x.iter().map(|(id, _)| *id).collect();

        // Collect rects, hits and paint.
        let mut rects = FastMap::default();
        let mut hits = vec![];
        for (i, e) in self.extras.iter().enumerate() {
            if let Some(id) = e.id {
                rects.insert(id, self.tree.nodes[i].rect);
            }
        }
        let bg = self.theme.color(Token::Background);
        let clear = if self.content.is_some() || !self.paint_background { Color::TRANSPARENT } else { bg };
        let mut painter = Painter::new(self.screen, self.scale, clear);
        if self.content.is_some() && self.paint_background {
            painter.rect(screen, bg);
        }
        let mut layer_rects = vec![];
        if self.check_layout || self.a11y_on {
            painter.text_log = Some(vec![]);
        }
        let mut extras = std::mem::take(&mut self.extras);
        let mut deferred: Vec<(Rect, PaintFn)> = vec![];
        let mut painted;
        {
            let mut cx = PaintCx {
                painter: &mut painter,
                text: &mut self.text,
                theme: &self.theme,
                anims: &mut self.anims,
                rects: &rects,
                time: self.time,
                deferred: &mut deferred,
                scroll_clip: screen,
                painted: self.a11y_on.then(Vec::new),
            };
            paint_node(&self.tree, &mut extras, &self.leaves, self.root, screen, &mut cx, &mut hits);
            let deferred_now = std::mem::take(cx.deferred);
            for (clip, f) in deferred_now {
                cx.painter.push_clip_replace(clip);
                f(&mut cx, Rect::ZERO);
                cx.painter.pop_clip();
            }
            for ly in &self.layers {
                cx.painter.layer_index += 1;
                let start = cx.painter.data.vertices.len();
                let clip = if ly.place == LayerPlace::Screen { screen } else { full };
                if ly.place != LayerPlace::Screen && ly.alpha > 0.0 {
                    layer_rects.push(self.tree.nodes[ly.root].rect);
                }
                cx.painter.push_clip_replace(clip);
                cx.scroll_clip = clip;
                paint_node(&self.tree, &mut extras, &self.leaves, ly.root, clip, &mut cx, &mut hits);
                let deferred_now = std::mem::take(cx.deferred);
                for (clip, f) in deferred_now {
                    cx.painter.push_clip_replace(clip);
                    f(&mut cx, Rect::ZERO);
                    cx.painter.pop_clip();
                }
                cx.painter.pop_clip();
                if ly.alpha < 1.0 {
                    let a = ly.alpha.clamp(0.0, 1.0);
                    for v in &mut cx.painter.data.vertices[start..] {
                        v.color[3] = (v.color[3] as f32 * a) as u8;
                        v.border_color[3] = (v.border_color[3] as f32 * a) as u8;
                    }
                }
            }
            for ov in &self.overlays {
                cx.painter.layer_index += 1;
                let r = self.tree.nodes[ov.root].rect;
                let d = match ov.placement {
                    Placement::Top => vec2(0.0, ov.slide),
                    Placement::Bottom => vec2(0.0, -ov.slide),
                    Placement::Right => vec2(-ov.slide, 0.0),
                    Placement::Left => vec2(ov.slide, 0.0),
                };
                if ov.alpha > 0.0 {
                    layer_rects.push(r.translate(d));
                }
                cx.painter.push_clip_replace(full);
                let alpha = ov.alpha;
                paint_overlay(&self.tree, &mut extras, &self.leaves, ov.root, d, alpha, &mut cx);
                cx.painter.pop_clip();
            }
            painted = cx.painted.take();
        }
        self.extras = extras;

        // Store this frame's interaction data for the next frame.
        self.prev_hits = hits;
        self.prev_rects = rects;
        self.prev_focus_order = std::mem::take(&mut self.focus_order);
        if let Some(f) = self.focused {
            if !self.prev_rects.contains_key(&f) {
                self.focused = None;
            }
        }
        self.anims.end_frame();
        self.text.end_frame();
        self.gc_memory();
        self.frame += 1;
        self.text_focused_prev = self.wants_text_input;

        let active_cursor = self.active.and_then(|a| self.prev_hits.iter().rev().find(|h| h.id == a && h.sense.drag).map(|h| h.cursor));
        let cursor = active_cursor
            .or_else(|| {
                self.pointer
                    .and_then(|p| self.prev_hits.iter().rev().find(|h| h.rect.contains(p) && (h.sense.click || h.sense.drag)))
                    .map(|h| h.cursor)
            })
            .unwrap_or_default();
        let cursor = match (self.active, cursor) {
            (Some(_), CursorIcon::Grab) => CursorIcon::Grabbing,
            (_, c) => c,
        };

        let mut repaint = self.repaint_at.map(|t| (t - self.time).max(0.0));
        let animating = self.anims.is_active() || self.theme.is_animating();
        // A click, key or screen reader action may change state that parts
        // of the view built earlier this frame already read: draw once more
        // so the change shows without waiting for the next input.
        let acted = !self.clicked.is_empty() || self.released.is_some() || self.any_key || !self.a11y_requests.is_empty();
        if animating || acted {
            repaint = Some(0.0);
        }
        if let Some(log) = painter.text_log.take() {
            if self.check_layout {
                self.issues = text_issues(&log);
                self.issues.append(&mut painter.paint_issues);
            }
            if let Some(rec) = painted.take() {
                let mut map: FastMap<NodeId, Vec<String>> = FastMap::default();
                for (n, a, b) in rec {
                    let texts = log[a..b].iter().filter(|t| t.alpha >= 0.3 && !t.text.trim().is_empty()).map(|t| t.text.trim().to_owned());
                    map.entry(n).or_default().extend(texts);
                }
                self.a11y_painted = map;
            }
        }
        let a11y = self.a11y_on.then(|| self.build_a11y());
        let mut draw = painter.finish();
        draw.atlas = self.text.take_atlas_delta();
        let mut window_regions: Vec<(WindowRegion, Rect)> =
            self.window_region_ids.drain(..).filter_map(|(k, id)| self.prev_rects.get(&id).map(|r| (k, *r))).collect();
        window_regions.sort_by_key(|(k, _)| *k == WindowRegion::Drag);
        self.prev_window_regions = window_regions.clone();
        self.prev_layer_rects = layer_rects.clone();
        let window_requests = std::mem::take(&mut self.window_requests);
        FrameOutput {
            draw,
            cursor,
            repaint_after: repaint,
            copied_text: self.copied.take(),
            wants_text_input: self.wants_text_input,
            text_field: self.text_field.take(),
            layer_rects,
            animating,
            window_requests,
            window_regions,
            a11y,
        }
    }

    /// Run one frame with a closure (convenience for tests and apps).
    pub fn run(&mut self, input: RawInput, f: impl FnOnce(&mut Ui)) -> FrameOutput {
        self.begin_frame(input);
        f(self);
        self.end_frame()
    }

    // ------------------------------------------------------------ queries

    pub fn time(&self) -> f64 {
        self.time
    }
    pub fn dt(&self) -> f32 {
        self.dt
    }
    /// The size of the area the UI lays out in: the window, or the
    /// [content rect](Self::set_content_rect) when one is set.
    pub fn screen_size(&self) -> Vec2 {
        self.content_rect().size()
    }

    /// Lay the main UI out in `rect` (logical px) instead of the whole
    /// screen. Menus, popovers, context menus and tooltips may still use
    /// the whole screen, so a host that embeds lumen in part of its window
    /// (GPUI, a game) can let them overflow the embedded area. Dialogs and
    /// toasts stay inside `rect`. Pixels outside `rect` are transparent
    /// except for those layers.
    pub fn set_content_rect(&mut self, rect: Option<Rect>) {
        self.content = rect;
    }

    /// Problems found in the last frame when
    /// [`check_layout`](Self::check_layout) is on: text cut by a clip, and
    /// text drawn over other text.
    pub fn layout_issues(&self) -> &[String] {
        &self.issues
    }

    /// The area the main UI lays out in. It may extend past the screen
    /// (an embedded panel scrolled half out of view); drawing is clipped to
    /// the screen.
    pub fn content_rect(&self) -> Rect {
        self.content.unwrap_or(Rect::from_min_size(Vec2::ZERO, self.screen))
    }
    pub fn color(&self, t: Token) -> Color {
        self.theme.color(t)
    }
    pub fn mode(&self) -> Mode {
        self.theme.mode()
    }
    pub fn accent(&self) -> Accent {
        self.theme.accent()
    }
    /// Switch light/dark; all tokens blend over 300ms.
    pub fn set_mode(&mut self, m: Mode) {
        let a = self.theme.accent();
        self.theme.set(m, a, self.time);
    }
    /// Switch accent; accent tokens blend over 200ms.
    pub fn set_accent(&mut self, a: Accent) {
        let m = self.theme.mode();
        self.theme.set(m, a, self.time);
    }
    /// Switch to a look; colors, geometry and motion blend to it. Cheap to
    /// call every frame with the same look.
    pub fn set_look(&mut self, look: Look) {
        self.theme.set_look(look, self.time);
        if self.applied_fonts != Some(look.fonts) {
            self.apply_fonts(look.fonts);
        }
    }

    /// The look on screen (blended while a change animates).
    pub fn look(&self) -> &Look {
        self.theme.look()
    }

    /// The current motion: springs and durations widgets animate with.
    pub fn motion(&self) -> Motion {
        self.theme.look().motion
    }

    fn apply_fonts(&mut self, f: Fonts) {
        use lumen_text::FontFamily;
        self.applied_fonts = Some(f);
        for (role, c) in [(FontFamily::Sans, f.sans), (FontFamily::Display, f.display), (FontFamily::Mono, f.mono)] {
            self.text.choose_family(role, c);
        }
    }

    /// The OS accent color used by [`Accent::Native`] (set by the app layer).
    pub fn set_native_accent(&mut self, c: Option<Color>) {
        self.theme.set_native_accent(c, self.time);
    }

    /// Override one token's color in one mode (`None` restores it).
    pub fn override_color(&mut self, mode: Mode, t: Token, c: Option<Color>) {
        self.theme.set_override(mode, t, c, self.time);
    }

    /// OS preferences reported by the app layer.
    pub fn system(&self) -> SystemPrefs {
        self.system
    }
    pub fn set_system(&mut self, p: SystemPrefs) {
        self.system = p;
        self.set_native_accent(p.accent);
    }

    /// A text role's style, scaled for the current look.
    pub fn text_style(&self, role: TextRole) -> TextStyle {
        let mut st = role.style();
        let k = self.look().text_scale;
        if (k - 1.0).abs() > 1e-3 {
            st.size = (st.size * k * 4.0).round() / 4.0;
            st.line_height = (st.line_height * k).round();
        }
        st
    }

    /// Sizes for a control size class in the current look.
    pub fn control_metrics(&self, size: ControlSize) -> ControlMetrics {
        let c = self.look().control;
        let i = size as usize;
        let mut text = TextRole::BodyStrong.style();
        text.size = (c.text_size[i] * 4.0).round() / 4.0;
        text.line_height = (text.size * 1.45).round();
        let height = c.height[i];
        ControlMetrics {
            height,
            pad_x: c.pad_x[i],
            pad_icon: (c.pad_x[i] - 2.0).max(4.0),
            icon: (height * 0.5).clamp(13.0, 18.0).round(),
            gap: c.gap[i],
            radius: c.radius[i].min(height / 2.0),
            text,
            inset: self.look().button.press_inset * [1.0, 1.0, 1.5][i],
        }
    }

    pub fn set_reduced_motion(&mut self, on: bool) {
        self.theme.reduced_motion = on;
    }
    pub fn reduced_motion(&self) -> bool {
        self.theme.reduced_motion
    }
    pub fn modifiers(&self) -> Modifiers {
        self.modifiers
    }
    /// True while the primary button is held (anywhere in the window).
    pub fn pointer_down(&self) -> bool {
        self.pointer_down
    }
    pub fn pointer(&self) -> Option<Vec2> {
        self.pointer
    }
    /// Mark the identified node as a custom title bar region this frame.
    /// [`Ui::window_chrome`] calls it in window mode.
    pub fn window_region(&mut self, kind: WindowRegion, id: Id) {
        self.window_region_ids.push((kind, id));
    }
    /// Ask the host window to close, minimize or toggle maximize.
    pub fn request_window(&mut self, req: WindowRequest) {
        self.window_requests.push(req);
    }
    /// The title bar region under `pos` (last frame's layout), for hosts to
    /// call on mouse-down: start a native window move on [`WindowRegion::Drag`].
    /// Menus and popovers over the bar take precedence and return `None`.
    pub fn window_region_at(&self, pos: Vec2) -> Option<WindowRegion> {
        if self.prev_layer_rects.iter().any(|r| r.contains(pos)) {
            return None;
        }
        self.prev_window_regions.iter().find(|(_, r)| r.contains(pos)).map(|(k, _)| *k)
    }
    /// Last frame's title bar regions (buttons first, then the drag region).
    pub fn window_regions(&self) -> &[(WindowRegion, Rect)] {
        &self.prev_window_regions
    }
    /// Whether the host window has keyboard focus (true until told otherwise).
    pub fn window_focused(&self) -> bool {
        self.window_focused
    }
    /// Whether the host window is maximized, as reported by
    /// [`Event::WindowMaximized`].
    pub fn window_maximized(&self) -> bool {
        self.window_maximized
    }
    // ------------------------------------------------------ accessibility

    /// Build an accessibility tree each frame ([`FrameOutput::a11y`]).
    /// Hosts turn it on when a screen reader asks for the tree.
    pub fn set_accessibility(&mut self, on: bool) {
        self.a11y_on = on;
    }

    pub fn accessibility(&self) -> bool {
        self.a11y_on
    }

    /// Describe widget `id` (its layout node) to assistive technology. The
    /// closure only runs while accessibility is on.
    pub fn a11y(&mut self, id: Id, f: impl FnOnce() -> A11y) {
        if self.a11y_on {
            self.a11y_by_id.entry(id).or_insert_with(f);
        }
    }

    /// Change what widget `id` already said about itself this frame (a
    /// popover marks its trigger expanded).
    pub fn a11y_update(&mut self, id: Id, f: impl FnOnce(&mut A11y)) {
        if self.a11y_on {
            if let Some(a) = self.a11y_by_id.get_mut(&id) {
                f(a);
            }
        }
    }

    /// Hide a node from assistive technology: a visible field label that
    /// already names its control.
    pub fn a11y_hide(&mut self, n: NodeId) {
        if self.a11y_on {
            self.a11y_by_node.push((n, A11y::hidden()));
        }
    }

    /// Describe a layout node that has no id (a heading's text leaf).
    pub fn a11y_node(&mut self, n: NodeId, f: impl FnOnce() -> A11y) {
        if self.a11y_on {
            self.a11y_by_node.push((n, f()));
        }
    }

    /// Actions assistive technology requested for `id` this frame (clicks
    /// and focus are already applied as ordinary input).
    pub fn a11y_actions(&self, id: Id) -> Vec<A11yAction> {
        self.a11y_requests.iter().filter(|(t, _)| *t == id).map(|(_, a)| a.clone()).collect()
    }

    /// Step a value control by a screen reader's increment or decrement:
    /// `+1`, `-1` or `0`, and a set number if one was sent.
    pub fn a11y_step(&self, id: Id) -> (i32, Option<f64>) {
        let mut step = 0;
        let mut set = None;
        for (t, a) in &self.a11y_requests {
            if *t != id {
                continue;
            }
            match a {
                A11yAction::Increment => step += 1,
                A11yAction::Decrement => step -= 1,
                A11yAction::SetNumber(v) => set = Some(*v),
                A11yAction::SetValue(v) => set = v.trim().parse().ok().or(set),
                _ => {}
            }
        }
        (step, set)
    }

    fn build_a11y(&mut self) -> A11yTree {
        let mut info: FastMap<NodeId, A11y> = FastMap::default();
        for (n, a) in self.a11y_by_node.drain(..) {
            info.insert(n, a);
        }
        if !self.a11y_by_id.is_empty() {
            for (i, e) in self.extras.iter().enumerate() {
                if let Some(id) = e.id {
                    if let Some(a) = self.a11y_by_id.remove(&id) {
                        info.entry(i).or_insert(a);
                    }
                }
            }
        }
        let scroll_of: FastMap<NodeId, Id> = self.scrolls.iter().map(|(id, n)| (*n, *id)).collect();
        let screen = Rect::from_min_size(Vec2::ZERO, self.screen);
        let mut b = A11yBuild {
            tree: &self.tree,
            extras: &self.extras,
            leaves: &self.leaves,
            info,
            scroll_of,
            tips: &self.tooltips,
            icons: &self.a11y_icons,
            painted: &self.a11y_painted,
            icon_names: vec![],
            nodes: vec![],
            names: vec![],
            seen: FastMap::default(),
            expanded: FastMap::default(),
            scroll: FastMap::default(),
        };
        let mut root = A11yNode { id: Id::ROOT.0, info: A11y::new(Role::Window), rect: screen, children: vec![], actions: Default::default() };
        root.info.label = None;
        b.nodes.push(root);
        b.names.push(None);
        b.icon_names.push(None);
        b.seen.insert(Id::ROOT.0, ());
        b.walk(self.root, 0, Sink::None, screen, None);
        for ly in &self.layers {
            if ly.alpha > 0.01 {
                b.walk(ly.root, 0, Sink::None, screen, None);
            }
        }
        b.finish_names();
        let focus = self.focused.map(|f| f.0).filter(|f| b.seen.contains_key(f)).unwrap_or(Id::ROOT.0);
        self.a11y_expanded = std::mem::take(&mut b.expanded);
        self.a11y_scroll = std::mem::take(&mut b.scroll);
        A11yTree { nodes: b.nodes, root: Id::ROOT.0, focus }
    }

    /// Previous-frame rect of an identified node.
    pub fn rect_of(&self, id: Id) -> Option<Rect> {
        self.prev_rects.get(&id).copied()
    }
    pub fn focused(&self) -> Option<Id> {
        self.focused
    }
    pub fn request_focus(&mut self, id: Id) {
        self.pending_focus = Some(id);
    }
    pub fn surrender_focus(&mut self, id: Id) {
        if self.focused == Some(id) {
            self.focused = None;
        }
    }
    /// Ask for another frame within `secs`.
    pub fn request_repaint_after(&mut self, secs: f64) {
        // A NaN or infinite delay means "no timer"; never schedule it.
        if !secs.is_finite() {
            return;
        }
        let t = self.time + secs.max(0.0);
        self.repaint_at = Some(self.repaint_at.map_or(t, |r| r.min(t)));
    }
    pub fn set_copied_text(&mut self, s: String) {
        self.copied = Some(s);
    }
    pub(crate) fn set_wants_text_input(&mut self) {
        self.wants_text_input = true;
    }

    /// Key presses this frame for the focused widget `id`.
    pub fn keys_for(&self, id: Id) -> Vec<(Key, Modifiers, bool)> {
        if self.focused == Some(id) {
            self.keys.clone()
        } else {
            vec![]
        }
    }

    /// True if `key` was pressed this frame while `id` is focused.
    pub fn key_pressed_for(&self, id: Id, key: Key) -> bool {
        self.focused == Some(id) && self.keys.iter().any(|(k, m, _)| *k == key && !m.command())
    }

    /// True if `key` was pressed this frame (globally), without modifiers
    /// and while no text input is focused. For app shortcuts.
    pub fn shortcut(&self, key: Key) -> bool {
        !self.text_focused_prev && self.keys.iter().any(|(k, m, _)| *k == key && !m.command() && !m.ctrl && !m.alt)
    }

    pub fn key_down(&self, key: Key) -> bool {
        self.keys_down.contains(&key)
    }

    /// Committed text for the focused field `id` (compositions dropped).
    pub(crate) fn take_text_input(&mut self, id: Id) -> Vec<String> {
        self.take_text_ops(id)
            .into_iter()
            .filter_map(|t| match t {
                TextIn::Text(t) | TextIn::Ime(crate::input::Ime::Commit { text: t, .. }) => Some(t),
                TextIn::Ime(_) => None,
            })
            .collect()
    }

    /// Typed text and input-method events for the focused field `id`.
    pub(crate) fn take_text_ops(&mut self, id: Id) -> Vec<TextIn> {
        if self.focused == Some(id) {
            std::mem::take(&mut self.text_input)
        } else {
            vec![]
        }
    }

    /// While `id` is focused, deliver Tab and Shift+Tab to it (as keys)
    /// instead of moving focus. Call it every frame the widget wants Tab;
    /// stop calling it (on Escape, say) to let Tab leave.
    pub fn capture_tab(&mut self, id: Id) {
        self.tab_capture_next = Some(id);
    }

    /// Report the focused field to the host's input method.
    pub(crate) fn report_text_field(&mut self, f: TextFieldState) {
        self.text_field = Some(f);
    }

    /// Scroll delta for the scroll area `id` (if the pointer is over it).
    pub(crate) fn take_scroll(&mut self, id: Id) -> Vec2 {
        if self.scroll_target == Some(id) {
            std::mem::take(&mut self.scroll_delta)
        } else {
            Vec2::ZERO
        }
    }

    // ------------------------------------------------------------- memory

    /// Persistent per-widget state of type `T`.
    pub fn mem<T: Default + 'static>(&mut self, id: Id) -> &mut T {
        let key = (id, std::any::TypeId::of::<T>());
        let frame = self.frame;
        let entry = self.memory.entry(key).or_insert_with(|| (Box::new(T::default()), frame));
        entry.1 = frame;
        entry.0.downcast_mut::<T>().expect("memory type")
    }

    /// Read memory without creating it.
    pub fn mem_get<T: 'static>(&self, id: Id) -> Option<&T> {
        self.memory.get(&(id, std::any::TypeId::of::<T>())).and_then(|(b, _)| b.downcast_ref::<T>())
    }

    fn gc_memory(&mut self) {
        // State is dropped once its widget has been gone for ~10 seconds.
        let f = self.frame;
        if f % 64 == 0 {
            self.memory.retain(|_, (_, seen)| f - *seen < 600);
        }
    }

    // ------------------------------------------------------------ ids

    /// Make a stable id for a widget from a key, unique within the scope.
    /// The id the next [`make_id`](Self::make_id) with `key` will return,
    /// without claiming it: for addressing a widget before it's built, such
    /// as the scroll area of `ui.scroll(key)` for [`scroll_to`](Self::scroll_to).
    pub fn peek_id(&self, key: impl std::hash::Hash) -> Id {
        let scope = *self.id_scope.last().unwrap_or(&Id::ROOT);
        let id = scope.with(key);
        match self.seen.get(&id) {
            None | Some(0) => id,
            Some(n) => id.with(*n + 1),
        }
    }
    pub fn make_id(&mut self, key: impl std::hash::Hash) -> Id {
        let scope = *self.id_scope.last().unwrap_or(&Id::ROOT);
        let id = scope.with(key);
        let n = self.seen.entry(id).or_insert(0);
        *n += 1;
        if *n == 1 {
            id
        } else {
            id.with(*n)
        }
    }

    /// Run `f` with ids scoped under `key`.
    pub fn scope<R>(&mut self, key: impl std::hash::Hash, f: impl FnOnce(&mut Ui) -> R) -> R {
        let id = self.make_id(key);
        self.id_scope.push(id);
        let r = f(self);
        self.id_scope.pop();
        r
    }

    // ------------------------------------------------------- interaction

    /// Interaction for widget `id` against the previous frame's layout.
    pub fn interact(&mut self, id: Id, sense: Sense, disabled: bool) -> Interact {
        let rect = self.prev_rects.get(&id).copied().unwrap_or(Rect::ZERO);
        if sense.focusable && !disabled {
            self.focus_order.push(id);
        }
        if disabled {
            if self.focused == Some(id) {
                self.focused = None;
            }
            if self.active == Some(id) {
                self.active = None;
            }
            return Interact { id, rect, disabled: true, hovered: self.hovered == Some(id), ..Default::default() };
        }
        let hovered = self.hovered == Some(id);
        let held = self.active == Some(id);
        let focused = self.focused == Some(id);
        // Track hover start for tooltips.
        if hovered {
            match self.hover_since {
                Some((h, _)) if h == id => {}
                _ => self.hover_since = Some((id, self.time)),
            }
        }
        Interact {
            id,
            rect,
            hovered,
            pressed: held && hovered,
            held,
            clicked: self.clicked.contains(&id),
            press_started: self.press_started == Some(id),
            released: self.released == Some(id),
            pointer: self.pointer,
            focused,
            focus_visible: focused && self.focus_visible,
            disabled,
            click_count: self.click_count,
        }
    }

    // ---------------------------------------------------- tree building

    fn push_node(&mut self, style: Style, content: Content, extra: NodeExtra) -> NodeId {
        let n = self.tree.add(style, content);
        if let Some(&parent) = self.stack.last() {
            self.tree.nodes[parent].children.push(n);
        }
        self.extras.push(extra);
        n
    }

    /// Add a leaf node with fixed content size and a paint closure.
    pub fn leaf(&mut self, style: Style, size: Vec2, id: Option<Id>, sense: Sense, cursor: CursorIcon, paint: Option<PaintFn>) -> NodeId {
        self.push_node(style, Content::Fixed(size), NodeExtra { id, sense, cursor, clip: false, hit_pad: 0.0, paint, paint_after: None })
    }

    /// Add a text leaf.
    pub fn text_leaf(&mut self, text: impl Into<String>, style: TextStyle, color: Color, wrap: bool, truncate: bool, layout: Style) -> NodeId {
        let text = text.into();
        let mut layout = layout;
        if !wrap && !truncate && !matches!(layout.width, Size::Fixed(_)) {
            // One line that neither wraps nor shortens: its width is its
            // min-content (CSS `nowrap`). Squeezing it only draws it over
            // the next item.
            let w = self.text.layout(&text, &style, None, TextAlign::Start).width.ceil();
            layout.min.x = layout.min.x.max(w);
        }
        if truncate {
            layout.min.x = layout.min.x.max(0.0);
        }
        let k = self.leaves.len() as u32;
        self.leaves.push(TextLeaf { text, style, color, wrap, truncate, align: TextAlign::Start });
        self.push_node(layout, Content::Measured(k), NodeExtra::default())
    }

    /// Add an icon leaf (Lucide glyph in a square of `size`).
    pub fn icon_leaf(&mut self, name: &str, size: f32, color: Color) -> NodeId {
        let name = name.to_owned();
        if self.a11y_on {
            let n = self.tree.nodes.len();
            self.a11y_icons.insert(n, name.clone());
        }
        // Glyphs don't scale with their rect, so a crowded row must not
        // squeeze one (it would spill into the next item).
        self.leaf(
            Style { min: vec2(size, size), ..Default::default() },
            vec2(size, size),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| cx.icon(&name, r.center(), size, color))),
        )
    }

    /// Open a container node; children added until [`end`](Self::end) go inside.
    pub fn begin(&mut self, style: Style, id: Option<Id>, sense: Sense, cursor: CursorIcon, clip: bool, paint: Option<PaintFn>) -> NodeId {
        let n = self.push_node(style, Content::None, NodeExtra { id, sense, cursor, clip, hit_pad: 0.0, paint, paint_after: None });
        self.stack.push(n);
        n
    }

    pub fn end(&mut self) {
        debug_assert!(self.stack.len() > 1, "unbalanced end()");
        self.stack.pop();
    }

    /// Replace a node's paint closure (run before its children).
    pub fn set_paint(&mut self, n: NodeId, f: PaintFn) {
        self.extras[n].paint = Some(f);
    }

    /// Extend a node's pointer hit area vertically by `px` on each side.
    pub fn set_hit_pad(&mut self, n: NodeId, px: f32) {
        self.extras[n].hit_pad = px;
    }

    /// Set alignment of a text leaf inside its rect.
    pub fn set_text_align(&mut self, n: NodeId, a: TextAlign) {
        if let Content::Measured(k) = self.tree.nodes[n].content {
            self.leaves[k as usize].align = a;
        }
    }

    /// Paint closure run after a node's children.
    pub fn set_paint_after(&mut self, n: NodeId, f: PaintFn) {
        self.extras[n].paint_after = Some(f);
    }

    /// The most recently added node.
    pub fn last_node(&self) -> NodeId {
        self.tree.nodes.len() - 1
    }

    /// Mutable access to a recorded node's layout style.
    pub fn node_style(&mut self, n: NodeId) -> &mut Style {
        &mut self.tree.nodes[n].style
    }

    // ------------------------------------------------------- containers

    pub fn column(&mut self) -> Container<'_> {
        Container::new(self, Dir::Column)
    }
    pub fn row(&mut self) -> Container<'_> {
        Container::new(self, Dir::Row)
    }
    pub fn stack(&mut self) -> Container<'_> {
        Container::new(self, Dir::Stack)
    }

    /// Fixed-size empty space.
    pub fn space(&mut self, px: f32) {
        self.leaf(Style::default(), vec2(px, px), None, Sense::NONE, CursorIcon::Default, None);
    }

    /// Flexible space that pushes siblings apart in a row or column.
    pub fn spacer(&mut self) {
        let st = Style { width: Size::FILL, height: Size::FILL, ..Default::default() };
        self.leaf(st, Vec2::ZERO, None, Sense::NONE, CursorIcon::Default, None);
    }

    /// 1px divider spanning the container's cross axis.
    pub fn divider(&mut self) {
        let c = self.color(Token::BorderSubtle);
        let parent_dir = self.stack.last().map(|&p| self.tree.nodes[p].style.dir).unwrap_or(Dir::Column);
        let (st, size) = if parent_dir == Dir::Row {
            (Style { self_align: Some(Align::Stretch), ..Default::default() }, vec2(1.0, 0.0))
        } else {
            (Style { self_align: Some(Align::Stretch), ..Default::default() }, vec2(0.0, 1.0))
        };
        self.leaf(st, size, None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.rect(r, c))));
    }

    /// A row that scrolls sideways when its children don't fit: they keep
    /// their natural widths and the row clips them. Trackpads and Shift +
    /// wheel scroll it; plain wheel input passes through to the page.
    /// `style` is the row's style (`dir` and `scroll_x` are set for you).
    pub fn scroll_row<R>(&mut self, key: impl std::hash::Hash, style: Style, f: impl FnOnce(&mut Ui) -> R) -> (R, ScrollX) {
        let id = self.make_id(("scroll_x", key));
        let wheel = self.take_scroll(id);
        let dx = if wheel.x != 0.0 { wheel.x } else { wheel.y };
        let reduced = self.reduced_motion();
        let gentle = self.motion().gentle;
        let st = *self.mem::<ScrollXState>(id);
        let max = (st.content - st.viewport).max(0.0);
        let mut next = st;
        if dx != 0.0 {
            next.target = None;
            next.offset = (st.offset + dx).clamp(0.0, max);
        } else if let Some(target) = st.target {
            let target = target.clamp(0.0, max);
            let key = id.with("reveal");
            if self.anims.spring_state(key, 0).is_none() {
                self.anims.set_spring(key, 0, st.offset);
            }
            let v = if reduced { target } else { self.anims.spring(key, 0, target, gentle) };
            next.offset = v.clamp(0.0, max);
            if (v - target).abs() < 0.5 {
                next.offset = target;
                next.target = None;
            }
        }
        *self.mem::<ScrollXState>(id) = next;
        let style = Style { dir: Dir::Row, scroll_x: true, ..style };
        let n = self.begin(style, Some(id), Sense { scroll: max > 0.0, ..Sense::NONE }, CursorIcon::Default, true, None);
        self.scrolls_x.push((id, n));
        self.id_scope.push(id);
        let r = f(self);
        self.id_scope.pop();
        self.end();
        (r, ScrollX { id, offset: next.offset, max })
    }

    /// Scroll a [`scroll_row`](Self::scroll_row) so `rect` (last frame's
    /// screen rect of one of its children) is in view, on a gentle spring.
    pub fn scroll_row_reveal(&mut self, row: Id, rect: Rect) {
        let Some(view) = self.rect_of(row) else { return };
        let st = self.mem::<ScrollXState>(row);
        let base = st.target.unwrap_or(st.offset);
        let target = if rect.min.x < view.min.x {
            base - (view.min.x - rect.min.x) - 24.0
        } else if rect.max.x > view.max.x {
            base + (rect.max.x - view.max.x) + 24.0
        } else {
            return;
        };
        st.target = Some(target.max(0.0));
    }

    /// A vertical scroll area that fills its parent.
    pub fn scroll(&mut self, key: impl std::hash::Hash) -> ScrollArea<'_> {
        let id = self.make_id(("scroll", key));
        ScrollArea { ui: self, id, style: Style { width: Size::FILL, height: Size::FILL, scroll: true, ..Default::default() } }
    }

    /// Animate the scroll area `key` (made by [`scroll`](Self::scroll) in
    /// the current scope) to `offset` on the gentle spring.
    pub fn scroll_to(&mut self, scroll_id: Id, offset: f32) {
        self.mem::<ScrollState>(scroll_id).target = Some(offset.max(0.0));
    }

    /// Jump a scroll area immediately to `offset` (e.g. initial open).
    pub fn scroll_set(&mut self, scroll_id: Id, offset: f32) {
        let st = self.scroll_state(scroll_id);
        st.offset = offset.max(0.0);
        st.target = None;
    }

    /// Id that [`scroll`](Self::scroll)`(key)` gets in the current scope.
    pub fn scroll_id(&self, key: impl std::hash::Hash) -> Id {
        let scope = *self.id_scope.last().unwrap_or(&Id::ROOT);
        scope.with(("scroll", key))
    }

    // --------------------------------------------------------- tooltips

    /// Attach a tooltip to widget `id` (shown on hover after 500ms, or on
    /// keyboard focus).
    /// Build a top layer: content drawn above everything else and hit-tested
    /// first. `offset` and `alpha` animate it in and out. With `block`, the
    /// layer's root takes that id and swallows clicks (modal scrims).
    pub fn layer<R>(&mut self, place: LayerPlace, offset: Vec2, alpha: f32, block: Option<Id>, f: impl FnOnce(&mut Ui) -> R) -> R {
        let saved = std::mem::take(&mut self.stack);
        let saved_nav = self.nav.take();
        let style = match place {
            LayerPlace::Screen => Style { width: Size::FILL, height: Size::FILL, ..Default::default() },
            LayerPlace::Anchor { .. } | LayerPlace::Point(_) => Style::default(),
        };
        let sense = if block.is_some() { Sense::CLICK } else { Sense::NONE };
        let root = self.begin(style, block, sense, CursorIcon::Default, false, None);
        let r = f(self);
        self.stack.clear();
        self.stack = saved;
        self.nav = saved_nav;
        self.layers.push(Layer { root, place, offset, alpha });
        r
    }

    /// The root node of the layer currently being built (inside the
    /// [`layer`](Self::layer) closure), for styling it.
    pub fn layer_root_building(&self) -> Option<NodeId> {
        self.stack.first().copied()
    }

    /// True if the widget with `id` was clicked this frame.
    pub fn clicked_id(&self, id: Id) -> bool {
        self.clicked.contains(&id)
    }

    pub(crate) fn repaint_after_secs(&mut self, secs: f64) {
        self.request_repaint_after(secs);
    }

    /// Ask the enclosing popover, menu or dialog to close (e.g. after a
    /// menu item is chosen).
    pub fn close_layer(&mut self) {
        self.layer_close = true;
    }

    pub(crate) fn take_layer_close(&mut self) -> bool {
        std::mem::take(&mut self.layer_close)
    }

    /// True if the primary button went down this frame outside all `rects`.
    pub fn pressed_outside(&self, rects: &[Rect]) -> bool {
        self.press_at.is_some_and(|p| !rects.iter().any(|r| r.contains(p)))
    }

    /// Where the secondary (right) button went down this frame, if inside
    /// `rect`. Context menus open from it.
    pub fn secondary_pressed_in(&self, rect: Rect) -> Option<Vec2> {
        self.secondary_at.filter(|p| rect.contains(*p))
    }

    /// True if `key` was pressed this frame, whatever has focus.
    pub fn key_pressed(&self, key: Key) -> bool {
        self.keys.iter().any(|(k, m, _)| *k == key && !m.command())
    }

    /// True if `⌘/Ctrl + ch` (case-insensitive) was pressed this frame.
    pub fn shortcut_pressed(&self, ch: char) -> bool {
        let want = ch.to_ascii_lowercase();
        self.keys.iter().any(|(k, m, r)| !*r && m.command() && matches!(k, Key::Char(c) if c.to_ascii_lowercase() == want))
    }

    /// Ids that took keyboard focus slots since `since` (an index from
    /// [`focus_mark`](Self::focus_mark)).
    pub fn focusables_since(&self, since: usize) -> Vec<Id> {
        self.focus_order.get(since..).map(|s| s.to_vec()).unwrap_or_default()
    }
    pub fn focus_mark(&self) -> usize {
        self.focus_order.len()
    }
    /// Whether the current focus came from the keyboard.
    pub fn focus_visible(&self) -> bool {
        self.focus_visible
    }

    pub fn tooltip_for(&mut self, id: Id, spec: TooltipSpec) {
        self.tooltips.insert(id, spec);
    }

    fn update_tooltip(&mut self) {
        let now = self.time;
        let candidate = self
            .hovered
            .filter(|h| self.tooltips.contains_key(h))
            .or_else(|| self.focused.filter(|f| self.focus_visible && self.tooltips.contains_key(f)));
        // Pressing, scrolling or typing closes immediately.
        if self.any_press || self.any_key {
            if let Some(t) = self.tip.take() {
                if t.closing_at.is_none() {
                    self.tip_last_closed = now;
                }
            }
            // Don't reopen until the pointer re-enters.
            if self.any_press {
                self.hover_since = self.hovered.map(|h| (h, f64::INFINITY));
            }
        }
        let focus_since = candidate.filter(|c| self.focused == Some(*c)).map(|c| *self.mem_focus_since(c));
        match (&mut self.tip, candidate) {
            (Some(t), Some(c)) if t.anchor == c => {
                t.closing_at = None;
                t.spec = self.tooltips[&c].clone();
            }
            (tip, Some(c)) => {
                let since = match self.hover_since {
                    Some((h, s)) if h == c => s,
                    // Keyboard focus: delay counts from focus time.
                    _ if focus_since.is_some() => focus_since.unwrap_or(now),
                    _ => now,
                };
                let warm = now - self.tip_last_closed < durations::TOOLTIP_WARM || tip.as_ref().is_some_and(|t| t.closing_at.is_none());
                let open_at = if warm { since } else { since + durations::TOOLTIP_DELAY };
                if now >= open_at {
                    self.tip = Some(TipState { anchor: c, spec: self.tooltips[&c].clone(), opened_at: now, closing_at: None, warm });
                } else if open_at.is_finite() {
                    // After a click, `since` is infinite: the tip waits for
                    // the pointer to leave and come back, not for a timer.
                    self.request_repaint_after(open_at - now);
                }
            }
            (Some(t), None) => {
                if t.closing_at.is_none() {
                    t.closing_at = Some(now);
                    self.tip_last_closed = now;
                }
            }
            (None, None) => {}
        }
        // Build the overlay.
        let Some(t) = &self.tip else { return };
        let (anchor, spec, opened, closing, warm) = (t.anchor, t.spec.clone(), t.opened_at, t.closing_at, t.warm);
        let alpha = match closing {
            Some(c) => {
                let a = 1.0 - ((now - c) / durations::FADE_OUT as f64) as f32;
                if a <= 0.0 {
                    self.tip = None;
                    return;
                }
                a
            }
            None if warm => 1.0,
            None => (((now - opened) / durations::LAYER_IN as f64) as f32).clamp(0.0, 1.0),
        };
        if alpha < 1.0 {
            self.request_repaint_after(0.0);
        }
        let tip_id = Id::new("lumen-tooltip");
        let slide = if warm { 0.0 } else { self.anims.spring_from(tip_id.with(anchor), 0, 4.0, 0.0, self.motion().snappy) };
        let root = self.build_tooltip(&spec);
        self.overlays.push(Overlay { root, anchor, placement: spec.placement, gap: spec.gap, slide, alpha });
    }

    fn mem_focus_since(&mut self, id: Id) -> &mut f64 {
        #[derive(Default)]
        struct FocusSince(f64, bool);
        let now = self.time;
        let s = self.mem::<FocusSince>(id);
        if !s.1 {
            s.0 = now;
            s.1 = true;
        }
        &mut s.0
    }

    fn build_tooltip(&mut self, spec: &TooltipSpec) -> NodeId {
        let fill = self.color(Token::InverseSurface);
        let fg = self.color(Token::InverseForeground);
        let shadows = Elevation::Level1.shadows(self.theme.darkness(), self.color(Token::ShadowInk));
        let style_text = self.text_style(TextRole::Label);
        let multiline = {
            let g = self.text.layout(&spec.text, &style_text, None, TextAlign::Start);
            g.width > 240.0 - 16.0
        };
        let pad = if multiline { Sides::xy(8.0, 6.0) } else { Sides::xy(8.0, 4.0) };
        let style = Style { dir: Dir::Row, pad, gap: 8.0, align: Align::Center, max: vec2(240.0, f32::INFINITY), ..Default::default() };
        // Overlay roots are not children of the main tree.
        let saved = std::mem::take(&mut self.stack);
        let root = self.begin(
            style,
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                cx.painter.shadows(r, Corners::all(6.0), &shadows);
                cx.painter.rrect(r, Corners::all(6.0), fill, 0.0, Color::TRANSPARENT);
            })),
        );
        self.text_leaf(spec.text.clone(), style_text, fg, true, false, Style { width: Size::Hug, ..Default::default() });
        if let Some(sc) = &spec.shortcut {
            let mut mono = self.text_style(TextRole::Mono);
            mono.size = 11.0;
            mono.line_height = 16.0;
            let chip = fg.with_alpha(0.16 * fg.a);
            self.begin(
                Style { pad: Sides::xy(4.0, 0.0), height: Size::Fixed(16.0), justify: Justify::Center, ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(4.0), chip, 0.0, Color::TRANSPARENT))),
            );
            self.text_leaf(sc.clone(), mono, fg, false, false, Style::default());
            self.end();
        }
        self.stack.pop();
        self.stack = saved;
        root
    }

    // ------------------------------------------------------ text helpers

    /// Lay out a single line of text (cached).
    pub fn galley(&mut self, text: &str, style: &TextStyle) -> Arc<Galley> {
        self.text.layout(text, style, None, TextAlign::Start)
    }

    /// Convenience: spring keyed by id/slot.
    pub fn spring(&mut self, id: Id, slot: u32, target: f32, p: SpringParams) -> f32 {
        self.anims.spring(id, slot, target, p)
    }

    /// Convenience: fade keyed by id/slot.
    pub fn fade(&mut self, id: Id, slot: u32, on: bool, in_s: f32, out_s: f32, ease: Ease) -> f32 {
        self.anims.fade(id, slot, on, in_s, out_s, ease)
    }

    /// Hover/press color weights per DESIGN.md 7.2: returns (hover_t, press_t).
    pub fn state_weights(&mut self, ix: &Interact, key_held: bool) -> (f32, f32) {
        let m = self.motion();
        let h = self.fade(ix.id, 900, ix.hovered || ix.held, m.hover_in, m.hover_out, Ease::Linear);
        let p = self.fade(ix.id, 901, ix.pressed || key_held, 0.0, m.press_out, Ease::Out);
        (h, p)
    }

    /// Focus ring animation per DESIGN.md 6: returns (alpha, gap).
    pub fn focus_anim(&mut self, ix: &Interact) -> (f32, f32) {
        let on = ix.focus_visible;
        let m = self.motion();
        let f = self.look().focus;
        let a = self.fade(ix.id, 902, on, 0.0, m.fade_out, Ease::Linear);
        let a_in = self.anims.spring_from(ix.id, 903, 0.0, if on { 1.0 } else { 0.0 }, m.snappy);
        let alpha = if on { a_in.clamp(0.0, 1.0) } else { a };
        let gap = if on { f.gap_from + (f.gap - f.gap_from) * a_in.clamp(0.0, 1.0) } else { f.gap };
        (alpha, gap)
    }

    pub(crate) fn scroll_state(&mut self, id: Id) -> &mut ScrollState {
        self.mem::<ScrollState>(id)
    }
}

/// Clipped and overlapping text among a frame's text draws. Only text
/// that's at least half opaque counts (fades and exits are fine).
fn text_issues(log: &[crate::paint::TextDraw]) -> Vec<String> {
    let mut out = vec![];
    let shown: Vec<_> = log.iter().filter(|t| t.alpha >= 0.5 && !t.text.trim().is_empty()).collect();
    let short = |s: &str| -> String {
        let s: String = s.chars().take(32).collect();
        s.replace('\n', " ")
    };
    for t in &shown {
        let visible = t.ink.intersect(&t.clip);
        // Partly clipped: some ink is cut, some shows (scrolled-away rows
        // are fully outside and fine). Allow a pixel of antialiasing.
        if !visible.is_empty() && !t.clip.expand(1.0).contains_rect(&t.ink) {
            let cut_x = (t.ink.width() - visible.width()).max(0.0);
            let cut_y = (t.ink.height() - visible.height()).max(0.0);
            if cut_x > 1.5 || cut_y > 1.5 {
                out.push(format!("clipped: {:?} cut by {:.0}x{:.0} px at ({:.0}, {:.0})", short(&t.text), cut_x, cut_y, t.ink.min.x, t.ink.min.y));
            }
        }
    }
    for (i, a) in shown.iter().enumerate() {
        let va = a.ink.intersect(&a.clip);
        if va.is_empty() {
            continue;
        }
        for b in &shown[i + 1..] {
            let vb = b.ink.intersect(&b.clip);
            let o = va.intersect(&vb);
            // A layer above hides the text under it.
            if o.is_empty() || a.text == b.text || a.layer != b.layer {
                continue;
            }
            let smaller = (va.width() * va.height()).min(vb.width() * vb.height()).max(1.0);
            if o.width() > 2.0 && o.height() > 2.0 && o.width() * o.height() > 0.15 * smaller {
                out.push(format!("overlap: {:?} and {:?} at ({:.0}, {:.0})", short(&a.text), short(&b.text), o.min.x, o.min.y));
            }
        }
    }
    out
}

fn rect_of_id(tree: &Tree, extras: &[NodeExtra], id: Id) -> Option<Rect> {
    extras.iter().position(|e| e.id == Some(id)).map(|i| tree.nodes[i].rect)
}

fn place_overlay(anchor: Rect, size: Vec2, placement: Placement, gap: f32, screen: Vec2) -> Vec2 {
    let margin = 8.0;
    let fits = |p: Placement| match p {
        Placement::Top => anchor.min.y - gap - size.y >= margin,
        Placement::Bottom => anchor.max.y + gap + size.y <= screen.y - margin,
        Placement::Right => anchor.max.x + gap + size.x <= screen.x - margin,
        Placement::Left => anchor.min.x - gap - size.x >= margin,
    };
    let order: &[Placement] = match placement {
        Placement::Top => &[Placement::Top, Placement::Bottom, Placement::Right, Placement::Left],
        Placement::Bottom => &[Placement::Bottom, Placement::Top, Placement::Right, Placement::Left],
        Placement::Right => &[Placement::Right, Placement::Left, Placement::Top, Placement::Bottom],
        Placement::Left => &[Placement::Left, Placement::Right, Placement::Top, Placement::Bottom],
    };
    let p = order.iter().copied().find(|&p| fits(p)).unwrap_or(placement);
    let c = anchor.center();
    let mut pos = match p {
        Placement::Top => vec2(c.x - size.x / 2.0, anchor.min.y - gap - size.y),
        Placement::Bottom => vec2(c.x - size.x / 2.0, anchor.max.y + gap),
        Placement::Right => vec2(anchor.max.x + gap, c.y - size.y / 2.0),
        Placement::Left => vec2(anchor.min.x - gap - size.x, c.y - size.y / 2.0),
    };
    pos.x = pos.x.clamp(margin, (screen.x - margin - size.x).max(margin));
    pos.y = pos.y.clamp(margin, (screen.y - margin - size.y).max(margin));
    vec2(pos.x.round(), pos.y.round())
}

fn place_layer(anchor: Rect, size: Vec2, placement: Placement, gap: f32, align: LayerAlign, screen: Vec2) -> Vec2 {
    let margin = 8.0;
    let fits = |p: Placement| match p {
        Placement::Top => anchor.min.y - gap - size.y >= margin,
        Placement::Bottom => anchor.max.y + gap + size.y <= screen.y - margin,
        Placement::Right => anchor.max.x + gap + size.x <= screen.x - margin,
        Placement::Left => anchor.min.x - gap - size.x >= margin,
    };
    let flip = match placement {
        Placement::Top => Placement::Bottom,
        Placement::Bottom => Placement::Top,
        Placement::Right => Placement::Left,
        Placement::Left => Placement::Right,
    };
    let p = if fits(placement) || !fits(flip) { placement } else { flip };
    let along = |lo: f32, hi: f32, len: f32| match align {
        LayerAlign::Start => lo,
        LayerAlign::Center => (lo + hi - len) / 2.0,
        LayerAlign::End => hi - len,
    };
    let mut pos = match p {
        Placement::Top => vec2(along(anchor.min.x, anchor.max.x, size.x), anchor.min.y - gap - size.y),
        Placement::Bottom => vec2(along(anchor.min.x, anchor.max.x, size.x), anchor.max.y + gap),
        Placement::Right => vec2(anchor.max.x + gap, along(anchor.min.y, anchor.max.y, size.y)),
        Placement::Left => vec2(anchor.min.x - gap - size.x, along(anchor.min.y, anchor.max.y, size.y)),
    };
    pos.x = pos.x.clamp(margin, (screen.x - margin - size.x).max(margin));
    pos.y = pos.y.clamp(margin, (screen.y - margin - size.y).max(margin));
    vec2(pos.x.round(), pos.y.round())
}

fn paint_text_leaf(cx: &mut PaintCx<'_>, l: &TextLeaf, r: Rect, alpha: f32) {
    let wrap = if l.wrap { Some(r.width()) } else { None };
    let mut g = cx.text.layout(&l.text, &l.style, wrap, l.align);
    if l.truncate && !l.wrap && g.width > r.width() + 0.5 {
        g = truncate(cx.text, &l.text, &l.style, r.width());
        if g.width > r.width() + 1.0 {
            // Not even "x…" fits: draw nothing rather than spill.
            return;
        }
    }
    let mut pos = r.min;
    if !l.wrap {
        match l.align {
            TextAlign::Start => {}
            TextAlign::Center => pos.x += ((r.width() - g.width) / 2.0).max(0.0),
            TextAlign::End => pos.x += (r.width() - g.width).max(0.0),
        }
    }
    cx.painter.text(cx.text, &g, pos, l.color.multiply_alpha(alpha));
}

/// Longest prefix of `text` that fits `width` with an ellipsis appended.
fn truncate(ts: &mut TextSystem, text: &str, style: &TextStyle, width: f32) -> Arc<Galley> {
    let bounds: Vec<usize> = text.char_indices().map(|(i, _)| i).chain(std::iter::once(text.len())).collect();
    let (mut lo, mut hi) = (0usize, bounds.len() - 1);
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        let s = format!("{}…", text[..bounds[mid]].trim_end());
        if ts.layout(&s, style, None, TextAlign::Start).width <= width {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    let s = format!("{}…", text[..bounds[lo]].trim_end());
    ts.layout(&s, style, None, TextAlign::Start)
}

fn paint_node(
    tree: &Tree,
    extras: &mut [NodeExtra],
    leaves: &[TextLeaf],
    n: NodeId,
    clip: Rect,
    cx: &mut PaintCx<'_>,
    hits: &mut Vec<Hit>,
) {
    let node = &tree.nodes[n];
    let r = node.rect;
    let e = &mut extras[n];
    if let Some(id) = e.id {
        let hr = r.expand2(0.0, e.hit_pad).intersect(&clip);
        if !hr.is_empty() && (e.sense != Sense::NONE) {
            hits.push(Hit { id, rect: hr, sense: e.sense, cursor: e.cursor });
        }
    }
    // Skip subtrees fully outside the clip.
    if r.intersect(&clip).is_empty() && !node.style.scroll {
        // Still register descendants' hits? They are clipped away anyway.
        return;
    }
    if let Some(p) = e.paint.take() {
        let logged = cx.painter.text_log.as_ref().map(|l| l.len());
        p(cx, r);
        if let (Some(rec), Some(from)) = (cx.painted.as_mut(), logged) {
            let to = cx.painter.text_log.as_ref().map_or(from, |l| l.len());
            if to > from {
                rec.push((n, from, to));
            }
        }
        // A fixed-size leaf the layout squeezed below its size: charts
        // redraw smaller, but text drawn past the rect means the widget
        // needed that width as a `min`.
        if let (Some(from), Content::Fixed(want)) = (logged, node.content) {
            if r.width() < want.x - 0.5 || r.height() < want.y - 0.5 {
                let log = cx.painter.text_log.as_ref().map(|l| &l[from..]).unwrap_or(&[]);
                let spill = log.iter().filter(|t| t.alpha >= 0.5 && !t.text.trim().is_empty()).find(|t| {
                    // Only what shows counts: a clipped canvas (editor,
                    // marquee) may scroll text past its rect on purpose.
                    let seen = t.ink.intersect(&t.clip);
                    !seen.is_empty() && !r.expand(2.5).contains_rect(&seen)
                });
                if let Some(t) = spill {
                    let text: String = t.text.chars().take(32).collect();
                    let note = format!(
                        "squeezed: \"{}\" spills out of a leaf laid out at {:.0}x{:.0} (wants {:.0}x{:.0}) at ({:.0}, {:.0})",
                        text.replace('\n', " "), r.width(), r.height(), want.x, want.y, r.min.x, r.min.y
                    );
                    cx.painter.paint_issues.push(note);
                }
            }
        }
    }
    if let Content::Measured(k) = node.content {
        paint_text_leaf(cx, &leaves[k as usize], r, 1.0);
    }
    let clips = e.clip || node.style.scroll;
    let child_clip = if clips { clip.intersect(&r) } else { clip };
    let saved_scroll_clip = cx.scroll_clip;
    if clips {
        cx.painter.push_clip(r);
        if node.style.scroll {
            cx.scroll_clip = child_clip;
        }
    }
    for &c in &node.children {
        paint_node(tree, extras, leaves, c, child_clip, cx, hits);
    }
    if clips {
        cx.painter.pop_clip();
        cx.scroll_clip = saved_scroll_clip;
    }
    if let Some(p) = extras[n].paint_after.take() {
        let from = cx.painter.text_log.as_ref().map(|l| l.len());
        p(cx, r);
        if let (Some(rec), Some(from)) = (cx.painted.as_mut(), from) {
            let to = cx.painter.text_log.as_ref().map_or(from, |l| l.len());
            if to > from {
                rec.push((n, from, to));
            }
        }
    }
}

fn paint_overlay(tree: &Tree, extras: &mut [NodeExtra], leaves: &[TextLeaf], n: NodeId, d: Vec2, alpha: f32, cx: &mut PaintCx<'_>) {
    // Overlays are small: paint with an offset and alpha by painting into a
    // temporary painter state. We translate by adjusting rects.
    let node = &tree.nodes[n];
    let r = node.rect.translate(d);
    let start = cx.painter.data.vertices.len();
    if let Some(p) = extras[n].paint.take() {
        p(cx, r);
    }
    if let Content::Measured(k) = node.content {
        paint_text_leaf(cx, &leaves[k as usize], r, 1.0);
    }
    for &c in &node.children {
        paint_overlay(tree, extras, leaves, c, d, 1.0, cx);
    }
    if alpha < 1.0 {
        for v in &mut cx.painter.data.vertices[start..] {
            v.color[3] = (v.color[3] as f32 * alpha) as u8;
            v.border_color[3] = (v.border_color[3] as f32 * alpha) as u8;
        }
    }
}

// ------------------------------------------------------------- containers

/// Builder for rows, columns and stacks.
#[must_use = "call .show(|ui| ...) to add the container"]
pub struct Container<'u> {
    ui: &'u mut Ui,
    style: Style,
    id: Option<Id>,
    fill: Option<Token>,
    border: Option<Token>,
    radius: f32,
    clip: bool,
    layer: Option<(f32, Vec2)>,
}

impl<'u> Container<'u> {
    fn new(ui: &'u mut Ui, dir: Dir) -> Self {
        Container { ui, style: Style { dir, ..Default::default() }, id: None, fill: None, border: None, radius: 0.0, clip: false, layer: None }
    }
    pub fn pad(mut self, p: impl Into<Sides>) -> Self {
        self.style.pad = p.into();
        self
    }
    pub fn gap(mut self, g: impl Into<f64>) -> Self {
        self.style.gap = g.into() as f32;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.style.width = Size::Fixed(w);
        self
    }
    pub fn height(mut self, h: f32) -> Self {
        self.style.height = Size::Fixed(h);
        self
    }
    pub fn size(mut self, w: Size, h: Size) -> Self {
        self.style.width = w;
        self.style.height = h;
        self
    }
    /// Fill the parent's free space on both axes.
    pub fn fill(mut self) -> Self {
        self.style.width = Size::FILL;
        self.style.height = Size::FILL;
        self
    }
    pub fn fill_width(mut self) -> Self {
        self.style.width = Size::FILL;
        self
    }
    pub fn fill_height(mut self) -> Self {
        self.style.height = Size::FILL;
        self
    }
    pub fn min_width(mut self, w: f32) -> Self {
        self.style.min.x = w;
        self
    }
    pub fn max_width(mut self, w: f32) -> Self {
        self.style.max.x = w;
        self
    }
    pub fn min_height(mut self, h: f32) -> Self {
        self.style.min.y = h;
        self
    }
    pub fn align(mut self, a: Align) -> Self {
        self.style.align = a;
        self
    }
    pub fn center(mut self) -> Self {
        self.style.align = Align::Center;
        self.style.justify = Justify::Center;
        self
    }
    pub fn justify(mut self, j: Justify) -> Self {
        self.style.justify = j;
        self
    }
    pub fn self_align(mut self, a: Align) -> Self {
        self.style.self_align = Some(a);
        self
    }
    pub fn margin_top(mut self, m: f32) -> Self {
        self.style.margin_top = m;
        self
    }
    /// Give the container an id (so its rect can be queried next frame and
    /// its children get a stable id scope).
    pub fn id(mut self, key: impl std::hash::Hash) -> Self {
        self.id = Some(self.ui.make_id(("container", key)));
        self
    }
    pub fn bg(mut self, t: Token) -> Self {
        self.fill = Some(t);
        self
    }
    pub fn border(mut self, t: Token) -> Self {
        self.border = Some(t);
        self
    }
    pub fn radius(mut self, r: f32) -> Self {
        self.radius = r;
        self
    }
    pub fn clip(mut self) -> Self {
        self.clip = true;
        self
    }
    /// Rows: move children that don't fit to a new line.
    pub fn wrap(mut self) -> Self {
        self.style.wrap = true;
        self
    }
    pub fn style(mut self, f: impl FnOnce(&mut Style)) -> Self {
        f(&mut self.style);
        self
    }

    /// Fade (`opacity`) and move (`offset`) the container and everything
    /// in it as one group, for entrances: drive them from a spring or an
    /// eased time. Layout is unaffected.
    pub fn layer(mut self, opacity: f32, offset: Vec2) -> Self {
        self.layer = Some((opacity, offset));
        self
    }

    pub fn show<R>(self, f: impl FnOnce(&mut Ui) -> R) -> Shown<R> {
        let Container { ui, style, id, fill, border, radius, clip, layer } = self;
        let paint: Option<PaintFn> = if fill.is_some() || border.is_some() || layer.is_some() {
            let fc = fill.map(|t| ui.color(t)).unwrap_or(Color::TRANSPARENT);
            let bc = border.map(|t| ui.color(t)).unwrap_or(Color::TRANSPARENT);
            let bw = if border.is_some() { 1.0 } else { 0.0 };
            let has_frame = fill.is_some() || border.is_some();
            Some(Box::new(move |cx, r| {
                if let Some((a, o)) = layer {
                    cx.painter.push_layer(a, o);
                }
                if has_frame {
                    cx.painter.rrect(r, Corners::all(radius), fc, bw, bc);
                }
            }))
        } else {
            None
        };
        let rect = id.and_then(|i| ui.rect_of(i)).unwrap_or(Rect::ZERO);
        let node = ui.begin(style, id, Sense::NONE, CursorIcon::Default, clip, paint);
        if layer.is_some() {
            ui.set_paint_after(node, Box::new(|cx, _| cx.painter.pop_layer()));
        }
        if let Some(i) = id {
            ui.id_scope.push(i);
        }
        let inner = f(ui);
        if id.is_some() {
            ui.id_scope.pop();
        }
        ui.end();
        Shown { inner, id: id.unwrap_or_default(), rect }
    }
}

/// Builder for a vertical scroll area.
#[must_use = "call .show(|ui| ...)"]
pub struct ScrollArea<'u> {
    ui: &'u mut Ui,
    id: Id,
    style: Style,
}

impl ScrollArea<'_> {
    /// A fixed height instead of filling the parent.
    pub fn height(mut self, px: f32) -> Self {
        self.style.height = Size::Fixed(px);
        self
    }
    pub fn pad(mut self, p: impl Into<Sides>) -> Self {
        self.style.pad = p.into();
        self
    }
    pub fn gap(mut self, g: f32) -> Self {
        self.style.gap = g;
        self
    }
    pub fn align(mut self, a: Align) -> Self {
        self.style.align = a;
        self
    }

    pub fn show<R>(self, f: impl FnOnce(&mut Ui) -> R) -> (R, ScrollInfo) {
        let ScrollArea { ui, id, style } = self;
        let wheel = ui.take_scroll(id);
        let time = ui.time;
        let dt = ui.dt;
        let reduced = ui.reduced_motion();
        let st = *ui.scroll_state(id);
        let max = (st.content - st.viewport).max(0.0);
        let mut next = st;
        if wheel.y != 0.0 {
            next.target = None;
            next.offset = (st.offset + wheel.y).clamp(0.0, max);
            next.last_scroll_time = time;
        }
        if let Some(target) = st.target {
            if st.viewport <= 0.0 {
                ui.request_repaint_after(0.0);
            } else {
                let target = target.min(max);
                let key = id.with("prog");
                if ui.anims.spring_state(key, 0).is_none_or(|s| s.target != target) && ui.anims.spring_state(key, 0).is_none() {
                    ui.anims.set_spring(key, 0, st.offset);
                }
                let gentle = ui.motion().gentle;
                let v = if reduced { target } else { ui.anims.spring(key, 0, target, gentle) };
                next.offset = v.clamp(0.0, max);
                if (v - target).abs() < 0.5 {
                    next.offset = target;
                    next.target = None;
                }
                next.last_scroll_time = time;
            }
        }
        let _ = dt;
        *ui.scroll_state(id) = next;
        let info = ScrollInfo { offset: next.offset, content_height: st.content, viewport_height: st.viewport };

        // Scrollbar (DESIGN.md 8): 8 wide overlay, 2 inset, visible while
        // scrolling or hovered, fades after 1s idle over 400ms.
        let since = (time - next.last_scroll_time) as f32;
        let hovered_bar = ui.pointer.zip(ui.rect_of(id)).is_some_and(|(p, r)| r.contains(p) && p.x > r.max.x - 12.0);
        let vis_target = since < 1.0 || hovered_bar;
        let vis = ui.fade(id, 1, vis_target, 0.06, 0.4, Ease::Linear);
        if since < 1.0 {
            ui.request_repaint_after((1.0 - since) as f64);
        }
        let thumb = ui.color(Token::BorderStrong);
        let n = ui.begin(style, Some(id), Sense { scroll: true, ..Sense::NONE }, CursorIcon::Default, true, None);
        ui.scrolls.push((id, n));
        ui.scroll_ctx.push((id, next.offset - st.offset));
        ui.id_scope.push(id);
        let r = f(ui);
        ui.id_scope.pop();
        ui.scroll_ctx.pop();
        ui.end();
        if max > 0.0 && vis > 0.0 {
            let (off, content, viewport) = (next.offset, st.content, st.viewport);
            ui.set_paint_after(
                n,
                Box::new(move |cx, rect| {
                    let track = Rect::from_xywh(rect.max.x - 10.0, rect.min.y + 2.0, 8.0, rect.height() - 4.0);
                    let h = (viewport / content * track.height()).max(32.0);
                    let y = track.min.y + (off / (content - viewport).max(1.0)) * (track.height() - h);
                    let tr = Rect::from_xywh(track.min.x + 2.0, y, 4.0, h);
                    cx.painter.rrect(tr, Corners::all(2.0), thumb.multiply_alpha(vis), 0.0, Color::TRANSPARENT);
                }),
            );
        }
        (r, info)
    }
}

#[derive(Clone, Copy)]
enum Sink {
    None,
    /// Text names this node (index into `nodes`).
    Name(usize),
    /// Text belongs to a value control; drop it.
    Drop,
}

struct A11yBuild<'a> {
    tree: &'a Tree,
    extras: &'a [NodeExtra],
    leaves: &'a [TextLeaf],
    info: FastMap<NodeId, A11y>,
    scroll_of: FastMap<NodeId, Id>,
    tips: &'a FastMap<Id, TooltipSpec>,
    icons: &'a FastMap<NodeId, String>,
    painted: &'a FastMap<NodeId, Vec<String>>,
    /// Icon names seen under each node, for controls without text.
    icon_names: Vec<Option<String>>,
    nodes: Vec<A11yNode>,
    /// Text collected for each node's name.
    names: Vec<Option<String>>,
    seen: FastMap<u64, ()>,
    expanded: FastMap<Id, bool>,
    scroll: FastMap<Id, (Id, Rect, Rect)>,
}

impl A11yBuild<'_> {
    fn add(&mut self, parent: usize, want: u64, info: A11y, rect: Rect, clickable: bool, focusable: bool) -> usize {
        let mut id = want;
        let mut k = 0u32;
        while self.seen.contains_key(&id) {
            k += 1;
            id = Id(want).with(k).0;
        }
        self.seen.insert(id, ());
        let actions = A11yTree::default_actions(&info, clickable, focusable);
        let ix = self.nodes.len();
        self.nodes.push(A11yNode { id, info, rect, children: vec![], actions });
        self.names.push(None);
        self.icon_names.push(None);
        self.nodes[parent].children.push(id);
        ix
    }

    fn walk(&mut self, n: NodeId, parent: usize, sink: Sink, clip: Rect, scroll: Option<(Id, Rect)>) {
        let node = &self.tree.nodes[n];
        let e = &self.extras[n];
        let rect = node.rect;
        // Collapsed or clipped away (not scrolled away: a screen reader can
        // still reach content below the fold).
        if clip.width() <= 0.0 || clip.height() <= 0.0 {
            return;
        }
        let vis = rect.intersect(&clip);
        if rect.width() > 0.0 && rect.height() > 0.0 && (vis.width() <= 0.0 || vis.height() <= 0.0) {
            return;
        }
        let mut info = self.info.remove(&n);
        if info.as_ref().is_some_and(|i| i.hidden) {
            return;
        }
        // An unnamed group says nothing: pass through to its children.
        let explicit = info.is_some();
        if info.as_ref().is_some_and(|i| i.role == Role::Group && i.label.is_none() && i.description.is_none()) {
            info = None;
        }
        let parent_role = self.nodes[parent].info.role;
        if info.is_none() {
            if let (Some(icon), Sink::Name(s)) = (self.icons.get(&n), sink) {
                self.icon_names[s].get_or_insert_with(|| crate::widgets::button::humanize(icon));
                return;
            }
            if let Content::Measured(k) = node.content {
                let text = self.leaves[k as usize].text.trim();
                if text.is_empty() {
                    return;
                }
                match sink {
                    Sink::Name(s) => {
                        let nm = self.names[s].get_or_insert_with(String::new);
                        if !nm.is_empty() {
                            nm.push(' ');
                        }
                        nm.push_str(text);
                    }
                    Sink::Drop => {}
                    Sink::None => {
                        if !parent_role.owns_text() {
                            let want = Id(self.nodes[parent].id).with(("text", self.nodes[parent].children.len())).0;
                            let a = A11y::new(Role::Label).label(text);
                            self.add(parent, want, a, rect, false, false);
                        }
                    }
                }
                return;
            }
            // Something clickable that never described itself: a generic
            // button, named by its text.
            if !explicit && e.sense.click && e.sense.focusable && e.id.is_some() && matches!(sink, Sink::None) {
                info = Some(A11y::new(Role::Button));
            }
        }
        let clip = if e.clip && !self.scroll_of.contains_key(&n) { clip.intersect(&rect) } else { clip };
        let scroll = self.scroll_of.get(&n).map(|id| (*id, rect)).or(scroll);
        let (here, sink) = match info {
            Some(mut a) => {
                if let (Content::Measured(k), None) = (node.content, &a.label) {
                    a.label = Some(self.leaves[k as usize].text.trim().to_owned());
                }
                if let Some(id) = e.id {
                    if let Some(t) = self.tips.get(&id) {
                        if a.label.is_none() && !a.role.owns_text() {
                            a.label = Some(t.text.clone());
                        } else if a.label.as_deref() != Some(t.text.as_str()) && a.description.is_none() {
                            a.description = Some(t.text.clone());
                        }
                    }
                    if let Some(x) = a.expanded {
                        self.expanded.insert(id, x);
                    }
                    if let Some((sid, view)) = scroll {
                        self.scroll.insert(id, (sid, rect, view));
                    }
                }
                let role = a.role;
                let named = a.label.is_some();
                let want = e.id.map(|i| i.0).unwrap_or_else(|| Id(self.nodes[parent].id).with(("node", n)).0);
                let ix = self.add(parent, want, a, rect, e.sense.click, e.sense.focusable);
                let sink = if role.owns_text() {
                    Sink::Drop
                } else if role.named_by_contents() {
                    if named {
                        Sink::Drop
                    } else {
                        Sink::Name(ix)
                    }
                } else {
                    Sink::None
                };
                (ix, sink)
            }
            None => (parent, sink),
        };
        if let (Some(texts), Sink::Name(s)) = (self.painted.get(&n), sink) {
            let nm = self.names[s].get_or_insert_with(String::new);
            for t in texts {
                if !nm.is_empty() {
                    nm.push(' ');
                }
                nm.push_str(t);
            }
        }
        for &c in &self.tree.nodes[n].children {
            self.walk(c, here, sink, clip, scroll);
        }
    }

    fn finish_names(&mut self) {
        for (i, nm) in self.names.iter_mut().enumerate() {
            let node = &mut self.nodes[i];
            if let Some(nm) = nm.take() {
                if node.info.label.is_none() && !nm.is_empty() {
                    node.info.label = Some(nm);
                }
            }
            if node.info.label.is_none() {
                node.info.label = self.icon_names[i].take();
            }
        }
        // A form control with no label of its own (a switch in a settings
        // row) is named by the text just before it, the way a row's title
        // labels its control.
        let index: FastMap<u64, usize> = self.nodes.iter().enumerate().map(|(i, n)| (n.id, i)).collect();
        for p in 0..self.nodes.len() {
            let mut last: Option<String> = None;
            for k in 0..self.nodes[p].children.len() {
                let Some(&c) = index.get(&self.nodes[p].children[k]) else { continue };
                let n = &self.nodes[c];
                if n.info.role == Role::Label || n.info.role == Role::Heading {
                    last = n.info.label.clone();
                    continue;
                }
                let form = matches!(
                    n.info.role,
                    Role::CheckBox | Role::Switch | Role::Slider | Role::TextInput | Role::ComboBox | Role::SpinButton | Role::MultilineTextInput | Role::SearchInput | Role::PasswordInput
                );
                if form && n.info.label.is_none() && n.info.placeholder.is_none() {
                    if let Some(l) = &last {
                        self.nodes[c].info.label = Some(l.clone());
                    }
                }
            }
        }
    }
}
