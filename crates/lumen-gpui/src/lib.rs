//! # lumen-gpui
//!
//! Host lumen widgets inside a [GPUI](https://gpui.rs) application (the
//! framework behind the Zed editor).
//!
//! A [`LumenView`] is an ordinary GPUI view. Put it anywhere in your element
//! tree, next to native GPUI elements, and it fills the space it's given
//! with a lumen UI: same widgets, looks, springs and text as a standalone
//! lumen app.
//!
//! ```no_run
//! use gpui::{prelude::*, div, px, size, App, Bounds, Entity, Window, WindowBounds, WindowOptions};
//! use lumen_gpui::LumenView;
//!
//! #[derive(Default)]
//! struct Settings { notify: bool, volume: f32 }
//!
//! struct Shell { panel: Entity<LumenView<Settings>> }
//!
//! impl Render for Shell {
//!     fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
//!         div().size_full().flex().flex_col()
//!             .child(div().p_2().child("A GPUI header"))
//!             .child(div().flex_1().child(self.panel.clone()))
//!     }
//! }
//!
//! lumen_gpui::run(lumen_gpui::application(), |cx: &mut App| {
//!     let bounds = Bounds::centered(None, size(px(720.), px(480.)), cx);
//!     cx.open_window(WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() }, |window, cx| {
//!         let panel = LumenView::build(Settings::default(), |ui, s| {
//!             ui.column().pad(24).gap(14).show(|ui| {
//!                 ui.heading("General");
//!                 ui.switch("Notifications", &mut s.notify);
//!                 ui.slider("Volume", &mut s.volume, 0.0..=1.0);
//!             });
//!         }, window, cx);
//!         cx.new(|_| Shell { panel })
//!     }).unwrap();
//! });
//! ```
//!
//! ## How it works
//!
//! GPUI doesn't let other renderers draw into its frame, so lumen renders
//! offscreen with wgpu on a device shared by every `LumenView`, reads the
//! pixels back and hands them to GPUI as an image. The readback is
//! pipelined: while the GPU works on a frame, GPUI shows the previous one,
//! so the main thread never waits for the GPU (the first frame and frames
//! after a resize do wait, so you never see a stretched image).
//!
//! A view at rest renders nothing: frames are only produced after input, a
//! state change you make through [`LumenView::update`], a theme change, a
//! resize, or while lumen is animating.
//!
//! ## Overlays
//!
//! Menus, select lists, popovers, context menus and tooltips may extend past
//! the view's bounds, over neighbouring GPUI elements, the way a native
//! popup would. They're drawn in a GPUI deferred layer on top of the window
//! and take the mouse only where they are. Dialogs, sheets and toasts stay
//! inside the view. Call [`LumenView::contain_overlays`] to keep everything
//! inside the bounds instead.
//!
//! ## Text input
//!
//! When a lumen text field is focused, the view registers a GPUI input
//! handler, so input methods work: CJK composition shows underlined in the
//! field and commits on confirm, macOS press-and-hold accents and dead keys
//! work, and the candidate window sits under the caret.

#[cfg(all(feature = "zed", feature = "ce"))]
compile_error!("enable one of the `zed` (gpui) and `ce` (gpui-ce) features, not both");
#[cfg(not(any(feature = "zed", feature = "ce")))]
compile_error!("enable the `zed` (gpui) or the `ce` (gpui-ce) feature");
// gpui-ce's library is also named `gpui`, so code is the same for both.

use std::collections::VecDeque;
use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    canvas, deferred, div, point, prelude::*, size, App, Bounds, ClipboardItem, Context, Corners, CursorStyle,
    DispatchPhase, ElementInputHandler, Entity, EntityInputHandler, FocusHandle, Global, Hitbox, HitboxBehavior,
    KeyDownEvent, KeyUpEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, RenderImage,
    ScrollDelta, ScrollWheelEvent, Subscription, Task, UTF16Selection, Window, WindowAppearance, WindowControlArea,
};
use lumen_core::input::{CursorIcon, Event, Ime, Key, Modifiers, PointerButton, RawInput};
use lumen_core::theme::Theme;
use lumen_core::{vec2, Accent, Look, Mode, Rect, TextFieldState, Ui, WindowRegion, WindowRequest};
use lumen_wgpu::offscreen::{self, AsyncReadback};
use lumen_wgpu::Renderer;
use web_time::Instant;

/// lumen itself, re-exported so you can depend on `lumen-gpui` alone.
pub use lumen_core;

/// One wgpu device shared by every [`LumenView`] in the process.
struct SharedGpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

struct GpuGlobal(Option<Arc<SharedGpu>>);
impl Global for GpuGlobal {}

fn shared_gpu(cx: &mut App) -> Option<Arc<SharedGpu>> {
    if !cx.has_global::<GpuGlobal>() {
        let gpu = offscreen::headless_device().map(|(device, queue)| Arc::new(SharedGpu { device, queue }));
        if gpu.is_none() {
            eprintln!("lumen-gpui: no wgpu adapter available; lumen views will stay blank");
        }
        cx.set_global(GpuGlobal(gpu));
    }
    cx.global::<GpuGlobal>().0.clone()
}

type ViewFn<S> = Box<dyn FnMut(&mut Ui, &mut S)>;

/// Shadows reach this far past a menu's rect.
const SHADOW_REACH: f32 = 40.0;

struct Target {
    gpu: Arc<SharedGpu>,
    renderer: Renderer,
    readback: AsyncReadback,
}

/// What a submitted frame covers, kept until its pixels come back.
#[derive(Clone)]
struct FrameMeta {
    seq: u64,
    /// Window coordinates (logical px) of the image.
    region: Rect,
    /// Part of it lies outside the view (an overflowing menu).
    overflow: bool,
    /// Menus and popovers in the image, window coordinates.
    layers: Vec<Rect>,
    straight: bool,
    t_submit: Instant,
}

/// The frame GPUI currently paints.
struct Shown {
    image: Arc<RenderImage>,
    region: Rect,
    overflow: bool,
    layers: Vec<Rect>,
}

/// A GPUI view that draws a lumen UI over your state `S`.
///
/// Create one with [`LumenView::build`] (or `cx.new(|cx| LumenView::new(..))`)
/// and use the resulting `Entity<LumenView<S>>` as a child element.
pub struct LumenView<S: 'static> {
    ui: Ui,
    state: S,
    view: ViewFn<S>,
    focus: FocusHandle,

    events: Vec<Event>,
    modifiers: Modifiers,
    start: Instant,
    hovered: bool,
    pressed: u8,
    /// Window position of lumen's origin: zero when overlays may escape
    /// (lumen's screen is the window), the view's origin otherwise.
    origin: Point<Pixels>,
    bounds: Bounds<Pixels>,

    target: Option<Target>,
    pending: VecDeque<FrameMeta>,
    shown: Option<Shown>,
    retired: Vec<Arc<RenderImage>>,
    overlay_hitboxes: Vec<Hitbox>,
    /// Menus and popovers in lumen's latest frame (window coordinates),
    /// which may be ahead of the image on screen.
    open_layers: Vec<Rect>,
    size_key: ([u32; 2], u32),
    dirty: bool,
    cursor: CursorIcon,
    timer: Option<Task<()>>,
    /// The focused text field as the input method sees it: lumen's last
    /// report, updated right away by input-handler calls.
    text_field: Option<TextFieldState>,

    mode: Option<Mode>,
    transparent: bool,
    escape: bool,
    started: bool,
    _appearance: Subscription,
    _activation: Subscription,
    _bounds: Subscription,
    /// Custom title bar regions from `window_chrome(..).window()`, in window
    /// coordinates.
    window_regions: Vec<(WindowRegion, Rect)>,
    window_active: bool,
    window_maximized: bool,
    /// gpui-ce: lumen's accessibility tree from the last frame, with the
    /// window position of lumen's origin and the scale it was built at.
    #[cfg(feature = "ce")]
    a11y: Option<(lumen_core::A11yTree, lumen_core::Vec2, f32)>,
}

impl<S: 'static> LumenView<S> {
    /// Create the view. Call inside `cx.new(|cx| ...)`.
    pub fn new(state: S, view: impl FnMut(&mut Ui, &mut S) + 'static, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut ui = Ui::new();
        // Input methods produce CJK and other scripts Geist lacks.
        ui.text.set_system_fallback(true);
        ui.theme = Theme::with_look(Mode::Light, Accent::Blue, Look::lumen());
        let appearance = cx.observe_window_appearance(window, |this, _, cx| {
            this.dirty = true;
            cx.notify();
        });
        // A custom title bar dims when the window is inactive and shows
        // restore when it's maximized.
        let activation = cx.observe_window_activation(window, |this, _, cx| {
            this.dirty = true;
            cx.notify();
        });
        let bounds = cx.observe_window_bounds(window, |this, _, cx| {
            this.dirty = true;
            cx.notify();
        });
        LumenView {
            ui,
            state,
            view: Box::new(view),
            focus: cx.focus_handle(),
            events: vec![],
            modifiers: Modifiers { mac: cfg!(target_os = "macos"), ..Default::default() },
            start: Instant::now(),
            hovered: false,
            pressed: 0,
            origin: Point::default(),
            bounds: Bounds::default(),
            target: None,
            pending: VecDeque::new(),
            shown: None,
            retired: vec![],
            overlay_hitboxes: vec![],
            open_layers: vec![],
            size_key: ([0, 0], 0),
            dirty: true,
            cursor: CursorIcon::Default,
            timer: None,
            text_field: None,
            mode: None,
            transparent: false,
            escape: true,
            started: false,
            _appearance: appearance,
            _activation: activation,
            _bounds: bounds,
            window_regions: vec![],
            window_active: true,
            window_maximized: false,
            #[cfg(feature = "ce")]
            a11y: None,
        }
    }

    /// Create the view as an entity in one call.
    pub fn build(state: S, view: impl FnMut(&mut Ui, &mut S) + 'static, window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(state, view, window, cx))
    }

    /// Use a look (Lumen, macOS, Windows 11, GNOME or your own).
    pub fn look(mut self, look: Look) -> Self {
        let (m, a) = (self.ui.mode(), self.ui.accent());
        self.ui.theme = Theme::with_look(m, a, look);
        self
    }

    /// Use an accent color.
    pub fn accent(mut self, accent: Accent) -> Self {
        let look = *self.ui.theme.target_look();
        self.ui.theme = Theme::with_look(self.ui.mode(), accent, look);
        self
    }

    /// Pin light or dark mode. By default the view follows the GPUI
    /// window's appearance.
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Don't paint lumen's window background, so GPUI content behind the
    /// view shows through between widgets.
    pub fn transparent(mut self) -> Self {
        self.transparent = true;
        self
    }

    /// Keep menus, popovers and tooltips inside the view's bounds (they
    /// flip and shift to fit) instead of letting them overflow it.
    pub fn contain_overlays(mut self) -> Self {
        self.escape = false;
        self
    }

    /// Your state.
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Change your state from GPUI code and redraw.
    pub fn update(&mut self, cx: &mut Context<Self>, f: impl FnOnce(&mut S)) {
        f(&mut self.state);
        self.refresh(cx);
    }

    /// The lumen context, for theme changes such as
    /// `view.ui().set_look(Look::mac())`. Call [`refresh`](Self::refresh)
    /// afterwards.
    pub fn ui(&mut self) -> &mut Ui {
        &mut self.ui
    }

    /// Redraw on the next frame (after changing data the view reads).
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.dirty = true;
        cx.notify();
    }

    /// The focus handle lumen uses for keyboard input.
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus
    }

    /// True while a lumen menu, popover or tooltip is open.
    pub fn has_open_overlay(&self) -> bool {
        !self.open_layers.is_empty()
    }

    /// Window bounds of the open lumen menus, popovers and tooltips.
    pub fn overlay_bounds(&self) -> Vec<Bounds<Pixels>> {
        self.open_layers.iter().map(|r| bounds_of(*r)).collect()
    }

    /// True if the last frame shown draws past the view's bounds.
    pub fn is_overflowing(&self) -> bool {
        self.shown.as_ref().is_some_and(|s| s.overflow)
    }

    /// Title bar regions of a `ui.window_chrome(..).window()` in this view,
    /// in window coordinates: buttons first, then the drag region. The view
    /// already maps them to GPUI's [`WindowControlArea`]s (used for hit
    /// testing on Windows) and handles presses itself: dragging moves the
    /// window, a double-click follows the OS setting, and the buttons
    /// minimize, zoom and close. Use these to keep your own GPUI elements
    /// out of them.
    pub fn window_control_regions(&self) -> Vec<(WindowRegion, Bounds<Pixels>)> {
        self.window_regions.iter().map(|(k, r)| (*k, bounds_of(*r))).collect()
    }

    fn push(&mut self, e: Event, cx: &mut Context<Self>) {
        self.events.push(e);
        self.dirty = true;
        cx.notify();
    }

    fn local(&self, p: Point<Pixels>) -> lumen_core::Vec2 {
        vec2(f32::from(p.x - self.origin.x), f32::from(p.y - self.origin.y))
    }

    fn on_move(&mut self, e: &MouseMoveEvent, hovered: bool, cx: &mut Context<Self>) {
        self.modifiers = map_mods(&e.modifiers, self.modifiers.mac);
        if hovered || self.pressed != 0 {
            self.hovered = hovered;
            let p = self.local(e.position);
            self.push(Event::PointerMoved(p), cx);
        } else if self.hovered {
            self.hovered = false;
            self.push(Event::PointerLeft, cx);
        }
    }

    /// `inside`: over the view or one of its overlays. Presses elsewhere are
    /// forwarded only while an overlay is open, so it can close, and don't
    /// take focus.
    fn on_down(&mut self, e: &MouseDownEvent, inside: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(button) = map_button(e.button) else { return };
        if !inside && !self.has_open_overlay() {
            return;
        }
        self.modifiers = map_mods(&e.modifiers, self.modifiers.mac);
        if inside {
            let focus = self.focus.clone();
            focus_view(window, &focus, cx);
        }
        let pos = self.local(e.position);
        if inside && button == PointerButton::Primary && self.ui.window_region_at(pos) == Some(WindowRegion::Drag) {
            // A custom title bar: the platform moves the window (on macOS
            // AppKit does it natively from a transparent title bar). The
            // move eats the release, so lumen gets the whole click now.
            if e.click_count == 2 {
                window.titlebar_double_click();
                #[cfg(not(target_os = "macos"))]
                window.zoom_window();
            } else {
                window.start_window_move();
            }
            self.events.push(Event::PointerMoved(pos));
            self.events.push(Event::PointerButton { pos, button, pressed: true });
            self.push(Event::PointerButton { pos, button, pressed: false }, cx);
            return;
        }
        self.pressed |= bit(button);
        self.events.push(Event::PointerMoved(pos));
        self.push(Event::PointerButton { pos, button, pressed: true }, cx);
    }

    fn on_up(&mut self, e: &MouseUpEvent, cx: &mut Context<Self>) {
        let Some(button) = map_button(e.button) else { return };
        if self.pressed & bit(button) == 0 {
            return;
        }
        self.pressed &= !bit(button);
        let pos = self.local(e.position);
        self.push(Event::PointerButton { pos, button, pressed: false }, cx);
        if !self.hovered {
            self.events.push(Event::PointerLeft);
        }
    }

    fn on_scroll(&mut self, e: &ScrollWheelEvent, cx: &mut Context<Self>) {
        let d = match e.delta {
            ScrollDelta::Pixels(p) => vec2(-f32::from(p.x), -f32::from(p.y)),
            ScrollDelta::Lines(l) => vec2(-l.x * 40.0, -l.y * 40.0),
        };
        self.push(Event::Scroll(d), cx);
    }

    fn on_key_down(&mut self, e: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.modifiers = map_mods(&e.keystroke.modifiers, self.modifiers.mac);
        let m = self.modifiers;
        if let Some(key) = map_key(&e.keystroke.key) {
            if m.command() && key == Key::Char('v') {
                if let Some(t) = cx.read_from_clipboard().and_then(|c| c.text()) {
                    self.events.push(Event::Paste(t));
                }
            }
            self.events.push(Event::Key { key, pressed: true, repeat: e.is_held, modifiers: m });
        }
        // With a text field focused, typed text comes through the input
        // handler instead (see `EntityInputHandler`); taking it here too
        // would insert it twice.
        if self.text_field.is_none() && !m.command() && !m.ctrl {
            if let Some(t) = &e.keystroke.key_char {
                if t.chars().any(|c| !c.is_control()) {
                    self.events.push(Event::Text(t.clone()));
                }
            }
        }
        self.refresh(cx);
    }

    fn on_key_up(&mut self, e: &KeyUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.modifiers = map_mods(&e.keystroke.modifiers, self.modifiers.mac);
        if let Some(key) = map_key(&e.keystroke.key) {
            let modifiers = self.modifiers;
            self.push(Event::Key { key, pressed: false, repeat: false, modifiers }, cx);
        }
    }

    /// Run a lumen frame if anything changed, start its readback, and pick
    /// up any frame that finished. Called from paint.
    fn frame(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        let scale = window.scale_factor();
        let viewport = window.viewport_size();
        let size_px = [(f32::from(bounds.size.width) * scale).round() as u32, (f32::from(bounds.size.height) * scale).round() as u32];
        if size_px[0] == 0 || size_px[1] == 0 {
            return;
        }
        let key = (size_px, scale.to_bits());
        let resized = key != self.size_key;
        if resized || bounds.origin != self.bounds.origin {
            self.size_key = key;
            self.dirty = true;
        }
        self.bounds = bounds;
        self.origin = if self.escape { Point::default() } else { bounds.origin };
        let mode = self.mode.unwrap_or(match window.appearance() {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Mode::Dark,
            WindowAppearance::Light | WindowAppearance::VibrantLight => Mode::Light,
        });
        if !self.started {
            // First frame: start in the right mode instead of blending to it.
            let (accent, look) = (self.ui.accent(), *self.ui.theme.target_look());
            self.ui.theme = Theme::with_look(mode, accent, look);
            self.ui.theme.settle();
            self.started = true;
        } else if mode != self.ui.mode() {
            self.ui.set_mode(mode);
            self.dirty = true;
        }
        if self.target.is_none() {
            let Some(gpu) = shared_gpu(cx) else { return };
            // GPUI sprites are BGRA, so render BGRA and skip a CPU swizzle.
            let format = wgpu::TextureFormat::Bgra8Unorm;
            let renderer = Renderer::new(&gpu.device, format);
            self.target = Some(Target { gpu, renderer, readback: AsyncReadback::new(format, 3) });
        }
        let trace = std::env::var_os("LUMEN_GPUI_TRACE").is_some();
        let sync = std::env::var_os("LUMEN_GPUI_SYNC").is_some();

        if self.dirty {
            let t0 = Instant::now();
            let view_rect = rect_of(bounds);
            let (screen, content) = if self.escape {
                (vec2(f32::from(viewport.width), f32::from(viewport.height)), Some(view_rect))
            } else {
                (view_rect.size(), None)
            };
            self.ui.set_content_rect(content);
            self.ui.paint_background = !self.transparent;
            let (active, maximized) = (window.is_window_active(), window.is_maximized());
            if active != self.window_active {
                self.window_active = active;
                self.events.push(Event::WindowFocused(active));
            }
            if maximized != self.window_maximized {
                self.window_maximized = maximized;
                self.events.push(Event::WindowMaximized(maximized));
            }
            let input = RawInput {
                screen_size: screen,
                scale,
                time: self.start.elapsed().as_secs_f64(),
                events: std::mem::take(&mut self.events),
                modifiers: self.modifiers,
            };
            #[cfg(feature = "ce")]
            self.ui.set_accessibility(window.is_a11y_active());
            self.ui.begin_frame(input);
            (self.view)(&mut self.ui, &mut self.state);
            let mut out = self.ui.end_frame();
            #[cfg(feature = "ce")]
            {
                let o = vec2(f32::from(self.origin.x), f32::from(self.origin.y));
                let next = out.a11y.take().map(|t| (t, o, scale));
                if next != self.a11y {
                    // GPUI gathers the tree while it lays out, before lumen
                    // runs: show the new one on the next frame.
                    self.a11y = next;
                    cx.notify();
                }
            }

            // The image covers the view, plus any menu that sticks out of it.
            let window_rect = Rect::from_min_size(lumen_core::Vec2::ZERO, screen);
            let base = if self.escape { view_rect } else { window_rect };
            let mut region = base;
            let mut overflow = false;
            for l in out.layer_rects.iter().filter(|_| self.escape) {
                let reach = l.expand(SHADOW_REACH);
                if !base.contains_rect(&reach) {
                    overflow = true;
                    region = region.union(&reach);
                }
            }
            let region = region.intersect(&window_rect);
            if region.width() < 1.0 || region.height() < 1.0 {
                // Scrolled out of the window: nothing to draw.
                self.dirty = false;
                return;
            }
            out.draw.crop(region);
            let origin = vec2(f32::from(self.origin.x), f32::from(self.origin.y));
            let meta_layers: Vec<Rect> = out.layer_rects.iter().map(|r| r.translate(origin)).collect();
            self.open_layers = meta_layers.clone();

            let t = self.target.as_mut().unwrap();
            let t_build = t0.elapsed();
            if let Some(seq) = t.readback.submit(&t.gpu.device, &t.gpu.queue, &mut t.renderer, &out.draw) {
                self.dirty = false;
                self.pending.push_back(FrameMeta {
                    seq,
                    region: region.translate(origin),
                    overflow,
                    layers: meta_layers,
                    straight: overflow || self.transparent,
                    t_submit: Instant::now(),
                });
            } else {
                // Every readback slot is busy: keep the events and retry.
                self.dirty = true;
            }
            if trace {
                eprintln!(
                    "lumen-gpui: {}x{} px frame built in {:.2} ms, submitted in {:.2} ms",
                    out.draw.size_px[0],
                    out.draw.size_px[1],
                    t_build.as_secs_f64() * 1e3,
                    (t0.elapsed() - t_build).as_secs_f64() * 1e3
                );
            }

            if let Some(text) = out.copied_text {
                cx.write_to_clipboard(ClipboardItem::new_string(text));
            }
            self.window_regions = out.window_regions.iter().map(|(k, r)| (*k, r.translate(origin))).collect();
            for req in &out.window_requests {
                match req {
                    WindowRequest::Close => window.remove_window(),
                    WindowRequest::Minimize => window.minimize_window(),
                    WindowRequest::ToggleMaximize => window.zoom_window(),
                }
            }
            self.cursor = out.cursor;
            self.text_field = out.text_field;
            self.timer = None;
            match out.repaint_after {
                Some(s) if s <= 0.0 => self.dirty = true,
                Some(s) => {
                    if let Ok(delay) = Duration::try_from_secs_f64(s) {
                        self.timer = Some(cx.spawn(async move |this, cx| {
                            cx.background_executor().timer(delay).await;
                            let _ = this.update(cx, |this, cx| this.refresh(cx));
                        }));
                    }
                }
                None => {}
            }
        }

        // Pick up the newest finished frame. Block only when there's
        // nothing sensible to show: the first frame, or the old frame is
        // the wrong size (it would stretch).
        // An older frame is drawn at its own region, never stretched, so
        // only a resized view (it would leave a gap) waits.
        let wait = !self.pending.is_empty() && (sync || resized || self.shown.is_none());
        let t1 = Instant::now();
        let Some(t) = self.target.as_mut() else { return };
        let frame = if wait { t.readback.wait(&t.gpu.device) } else { t.readback.try_take(&t.gpu.device) };
        if let Some(f) = frame {
            while self.pending.front().is_some_and(|m| m.seq < f.seq) {
                self.pending.pop_front();
            }
            if let Some(meta) = self.pending.pop_front() {
                let [w, h] = f.size;
                let mut px = f.pixels.to_vec();
                if meta.straight {
                    unpremultiply(&mut px);
                }
                if let Some(img) = image::RgbaImage::from_raw(w, h, px) {
                    let image = Arc::new(RenderImage::new(vec![image::Frame::new(img)]));
                    if trace {
                        eprintln!(
                            "lumen-gpui: frame {} shown {:.2} ms after submit ({:.2} ms on the main thread{})",
                            meta.seq,
                            meta.t_submit.elapsed().as_secs_f64() * 1e3,
                            t1.elapsed().as_secs_f64() * 1e3,
                            if wait { ", waited" } else { "" }
                        );
                    }
                    // GPUI caches images in its sprite atlas until told
                    // otherwise. Keep the previous image for one more
                    // frame, then release it.
                    let old = self.shown.replace(Shown { image, region: meta.region, overflow: meta.overflow, layers: meta.layers });
                    if let Some(old) = old {
                        self.retired.push(old.image);
                    }
                    while self.retired.len() > 1 {
                        let old = self.retired.remove(0);
                        cx.drop_image(old, Some(window));
                    }
                }
            }
        }

        // Keep painting while lumen animates or a readback is in flight.
        if self.dirty || !self.pending.is_empty() {
            let id = cx.entity_id();
            window.on_next_frame(move |_, cx| cx.notify(id));
        }
    }

    // --- input method ----------------------------------------------------

    fn send_ime(&mut self, e: Ime, cx: &mut Context<Self>) {
        self.push(Event::Ime(e), cx);
    }
}

/// The GPUI application, from whichever GPUI this crate was built with:
/// `Application::new()` for gpui, the platform crate's
/// `application()` for gpui-ce.
pub fn application() -> gpui::Application {
    #[cfg(feature = "ce")]
    return gpui_ce_platform::application();
    #[cfg(not(feature = "ce"))]
    gpui::Application::new()
}

/// Runs `app` and calls `launch` once it has finished launching, where you
/// open your windows. Use this instead of `app.run(launch)`: with gpui-ce
/// 0.3.0 on macOS, the launch callback runs while the platform lock is
/// held, so an `open_window` inside it deadlocks (gpui-ce discussion #433,
/// fixed on their main branch). Here `launch` runs from a foreground task
/// on the next turn of the main loop, after the lock is released. With
/// Zed's GPUI it's called directly.
///
/// ```no_run
/// lumen_gpui::run(lumen_gpui::application(), |_cx| {
///     // cx.open_window(...)
/// });
/// ```
pub fn run(app: gpui::Application, launch: impl FnOnce(&mut App) + 'static) {
    app.run(move |cx: &mut App| {
        #[cfg(feature = "ce")]
        cx.spawn(async move |cx| {
            let _ = cx.update(launch);
        })
        .detach();
        #[cfg(not(feature = "ce"))]
        launch(cx);
    });
}

// The few calls that differ between gpui 0.2 and gpui-ce 0.3.

fn paint_frame(window: &mut Window, bounds: Bounds<Pixels>, image: Arc<RenderImage>) {
    #[cfg(feature = "ce")]
    let _ = window.paint_image(bounds, bounds, Corners::default(), 0.0, image, 0, false);
    #[cfg(not(feature = "ce"))]
    let _ = window.paint_image(bounds, Corners::default(), image, 0, false);
}

fn focus_view(window: &mut Window, focus: &FocusHandle, _cx: &mut App) {
    #[cfg(feature = "ce")]
    window.focus(focus, _cx);
    #[cfg(not(feature = "ce"))]
    window.focus(focus);
}

fn on_top(d: gpui::Deferred) -> gpui::Deferred {
    #[cfg(feature = "ce")]
    return d.priority(1usize);
    #[cfg(not(feature = "ce"))]
    d.with_priority(1)
}

fn rect_of(b: Bounds<Pixels>) -> Rect {
    Rect::from_xywh(f32::from(b.origin.x), f32::from(b.origin.y), f32::from(b.size.width), f32::from(b.size.height))
}

fn bounds_of(r: Rect) -> Bounds<Pixels> {
    Bounds { origin: point(gpui::px(r.min.x), gpui::px(r.min.y)), size: size(gpui::px(r.width()), gpui::px(r.height())) }
}

/// lumen blends premultiplied; GPUI sprites want straight alpha.
fn unpremultiply(px: &mut [u8]) {
    for p in px.chunks_exact_mut(4) {
        let a = p[3];
        if a != 0 && a != 255 {
            let k = 255.0 / a as f32;
            for c in &mut p[..3] {
                *c = (*c as f32 * k).round().min(255.0) as u8;
            }
        }
    }
}

/// Byte offset in `s` for UTF-16 offset `u`, clamped.
fn byte_at(s: &str, u: usize) -> usize {
    let mut n = 0;
    for (i, c) in s.char_indices() {
        if n >= u {
            return i;
        }
        n += c.len_utf16();
    }
    s.len()
}

/// UTF-16 offset for byte offset `b` of `s`.
fn utf16_at(s: &str, b: usize) -> usize {
    s[..b.min(s.len())].encode_utf16().count()
}

fn bytes(s: &str, r: &Range<usize>) -> Range<usize> {
    let a = byte_at(s, r.start);
    a..byte_at(s, r.end).max(a)
}

fn utf16(s: &str, r: &Range<usize>) -> Range<usize> {
    utf16_at(s, r.start)..utf16_at(s, r.end)
}

impl<S: 'static> EntityInputHandler for LumenView<S> {
    fn text_for_range(&mut self, range: Range<usize>, adjusted: &mut Option<Range<usize>>, _: &mut Window, _: &mut Context<Self>) -> Option<String> {
        let f = self.text_field.as_ref()?;
        let r = bytes(&f.text, &range);
        *adjusted = Some(utf16(&f.text, &r));
        Some(f.text[r].to_owned())
    }

    fn selected_text_range(&mut self, _: bool, _: &mut Window, _: &mut Context<Self>) -> Option<UTF16Selection> {
        let f = self.text_field.as_ref()?;
        let (a, b) = (f.anchor.min(f.cursor), f.anchor.max(f.cursor));
        Some(UTF16Selection { range: utf16(&f.text, &(a..b)), reversed: f.cursor < f.anchor })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        let f = self.text_field.as_ref()?;
        f.marked.as_ref().map(|m| utf16(&f.text, m))
    }

    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        // Keep the composition as typed.
        let Some(f) = self.text_field.as_mut() else { return };
        let Some(m) = f.marked.take() else { return };
        let text = f.text[m].to_owned();
        self.send_ime(Ime::Commit { text, replace: None }, cx);
    }

    fn replace_text_in_range(&mut self, range: Option<Range<usize>>, text: &str, _: &mut Window, cx: &mut Context<Self>) {
        let Some(f) = self.text_field.as_mut() else { return };
        let replace = range.map(|r| bytes(&f.text, &r));
        let r = f.marked.clone().or(replace.clone()).unwrap_or(f.anchor.min(f.cursor)..f.anchor.max(f.cursor));
        let text: String = text.chars().filter(|c| !c.is_control()).collect();
        f.text.replace_range(r.clone(), &text);
        f.cursor = r.start + text.len();
        f.anchor = f.cursor;
        let composing = f.marked.take().is_some();
        self.send_ime(Ime::Commit { text, replace: if composing { None } else { replace } }, cx);
    }

    fn replace_and_mark_text_in_range(&mut self, range: Option<Range<usize>>, new_text: &str, new_selected: Option<Range<usize>>, _: &mut Window, cx: &mut Context<Self>) {
        let Some(f) = self.text_field.as_mut() else { return };
        let replace = range.map(|r| bytes(&f.text, &r));
        let r = f.marked.clone().or(replace.clone()).unwrap_or(f.anchor.min(f.cursor)..f.anchor.max(f.cursor));
        let composing = f.marked.is_some();
        let text = new_text.to_owned();
        let cursor = new_selected.map(|s| (byte_at(&text, s.start), byte_at(&text, s.end)));
        f.text.replace_range(r.clone(), &text);
        let (a, b) = cursor.unwrap_or((text.len(), text.len()));
        f.anchor = r.start + a;
        f.cursor = r.start + b;
        f.marked = (!text.is_empty()).then(|| r.start..r.start + text.len());
        self.send_ime(Ime::Preedit { text, cursor, replace: if composing { None } else { replace } }, cx);
    }

    fn bounds_for_range(&mut self, _: Range<usize>, _: Bounds<Pixels>, _: &mut Window, _: &mut Context<Self>) -> Option<Bounds<Pixels>> {
        let f = self.text_field.as_ref()?;
        let o = vec2(f32::from(self.origin.x), f32::from(self.origin.y));
        Some(bounds_of(f.rect.translate(o)))
    }

    fn character_index_for_point(&mut self, _: Point<Pixels>, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        None
    }
}

impl<S: 'static> Render for LumenView<S> {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let overlay = entity.clone();
        // Custom title bar areas, for platforms that hit-test them (Windows).
        let origin = self.bounds.origin;
        let controls = self.window_regions.iter().map(move |(k, r)| {
            let area = match k {
                WindowRegion::Drag => WindowControlArea::Drag,
                WindowRegion::Close => WindowControlArea::Close,
                WindowRegion::Minimize => WindowControlArea::Min,
                WindowRegion::Maximize => WindowControlArea::Max,
            };
            let b = bounds_of(*r);
            div().absolute().left(b.origin.x - origin.x).top(b.origin.y - origin.y).w(b.size.width).h(b.size.height).window_control_area(area)
        });
        let root = div().id("lumen-view");
        #[cfg(feature = "ce")]
        let root = {
            let tree = self.a11y.clone();
            let view = cx.entity_id().as_u64();
            root.role(gpui::Role::Group).a11y_synthetic_children(move |b| {
                if let Some((t, o, scale)) = &tree {
                    a11y_children(b, t, view, *o, *scale);
                }
            })
        };
        root
            .size_full()
            .relative()
            .children(controls)
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::on_key_down))
            .on_key_up(cx.listener(Self::on_key_up))
            .child(
                canvas(
                    |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
                    move |bounds, hitbox, window, cx| paint(entity, bounds, hitbox, window, cx),
                )
                .size_full(),
            )
            // Overflowing menus: painted after the rest of the window, on
            // top, and unclipped by scrolling parents.
            .child(on_top(
                deferred(
                    canvas(
                        move |_, window, cx| {
                            let rects = overlay.read(cx).shown.as_ref().filter(|s| s.overflow).map(|s| s.layers.clone()).unwrap_or_default();
                            let hbs: Vec<Hitbox> = rects.into_iter().map(|r| window.insert_hitbox(bounds_of(r), HitboxBehavior::BlockMouse)).collect();
                            overlay.update(cx, |this, _| this.overlay_hitboxes = hbs.clone());
                            (overlay.clone(), hbs)
                        },
                        |_, (overlay, hbs), window, cx| {
                            let this = overlay.read(cx);
                            if let Some(s) = this.shown.as_ref().filter(|s| s.overflow) {
                                paint_frame(window, bounds_of(s.region), s.image.clone());
                            }
                            let cursor = map_cursor(this.cursor);
                            for hb in &hbs {
                                window.set_cursor_style(cursor, hb);
                            }
                        },
                    )
                    .size_0(),
                )
            ))
    }
}

fn paint<S: 'static>(entity: Entity<LumenView<S>>, bounds: Bounds<Pixels>, hitbox: Hitbox, window: &mut Window, cx: &mut App) {
    entity.update(cx, |this, cx| this.frame(bounds, window, cx));
    #[cfg(feature = "ce")]
    a11y_listeners(&entity, window, cx);
    let this = entity.read(cx);
    if let Some(s) = this.shown.as_ref().filter(|s| !s.overflow) {
        paint_frame(window, bounds_of(s.region), s.image.clone());
    }
    window.set_cursor_style(map_cursor(this.cursor), &hitbox);
    if this.text_field.is_some() {
        let focus = this.focus.clone();
        window.handle_input(&focus, ElementInputHandler::new(bounds, entity.clone()), cx);
    }

    // Over the view, or over one of its overflowing menus.
    fn inside<S: 'static>(e: &Entity<LumenView<S>>, hb: &Hitbox, window: &Window, cx: &App) -> bool {
        hb.is_hovered(window) || e.read(cx).overlay_hitboxes.iter().any(|h| h.is_hovered(window))
    }
    let (e, hb) = (entity.clone(), hitbox.clone());
    window.on_mouse_event(move |ev: &MouseMoveEvent, phase, window, cx| {
        if phase == DispatchPhase::Bubble {
            let hovered = inside(&e, &hb, window, cx);
            e.update(cx, |this, cx| this.on_move(ev, hovered, cx));
        }
    });
    let (e, hb) = (entity.clone(), hitbox.clone());
    window.on_mouse_event(move |ev: &MouseDownEvent, phase, window, cx| {
        if phase == DispatchPhase::Bubble {
            let hovered = inside(&e, &hb, window, cx);
            e.update(cx, |this, cx| this.on_down(ev, hovered, window, cx));
        }
    });
    let e = entity.clone();
    window.on_mouse_event(move |ev: &MouseUpEvent, phase, _, cx| {
        if phase == DispatchPhase::Bubble {
            e.update(cx, |this, cx| this.on_up(ev, cx));
        }
    });
    let (e, hb) = (entity, hitbox);
    window.on_mouse_event(move |ev: &ScrollWheelEvent, phase, window, cx| {
        let over_menu = e.read(cx).overlay_hitboxes.iter().any(|h| h.is_hovered(window));
        if phase == DispatchPhase::Bubble && (hb.should_handle_scroll(window) || over_menu) {
            e.update(cx, |this, cx| this.on_scroll(ev, cx));
        }
    });
}

fn bit(b: PointerButton) -> u8 {
    match b {
        PointerButton::Primary => 1,
        PointerButton::Secondary => 2,
        PointerButton::Middle => 4,
    }
}

fn map_button(b: MouseButton) -> Option<PointerButton> {
    Some(match b {
        MouseButton::Left => PointerButton::Primary,
        MouseButton::Right => PointerButton::Secondary,
        MouseButton::Middle => PointerButton::Middle,
        _ => return None,
    })
}

fn map_mods(m: &gpui::Modifiers, mac: bool) -> Modifiers {
    Modifiers { shift: m.shift, ctrl: m.control, alt: m.alt, logo: m.platform, mac }
}

fn map_key(k: &str) -> Option<Key> {
    Some(match k {
        "tab" => Key::Tab,
        "enter" => Key::Enter,
        "space" => Key::Space,
        "escape" => Key::Escape,
        "backspace" => Key::Backspace,
        "delete" => Key::Delete,
        "left" => Key::ArrowLeft,
        "right" => Key::ArrowRight,
        "up" => Key::ArrowUp,
        "down" => Key::ArrowDown,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
        _ => {
            let mut chars = k.chars();
            let c = chars.next()?;
            if chars.next().is_some() {
                return None;
            }
            Key::Char(c.to_lowercase().next()?)
        }
    })
}

fn map_cursor(c: CursorIcon) -> CursorStyle {
    match c {
        CursorIcon::Default => CursorStyle::Arrow,
        CursorIcon::Pointer => CursorStyle::PointingHand,
        CursorIcon::Text => CursorStyle::IBeam,
        CursorIcon::Grab => CursorStyle::OpenHand,
        CursorIcon::Grabbing => CursorStyle::ClosedHand,
        CursorIcon::ResizeHorizontal => CursorStyle::ResizeLeftRight,
        CursorIcon::Crosshair => CursorStyle::Crosshair,
        CursorIcon::ResizeVertical => CursorStyle::ResizeUpDown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_map_like_the_winit_runner() {
        assert_eq!(map_key("left"), Some(Key::ArrowLeft));
        assert_eq!(map_key("A"), Some(Key::Char('a')));
        assert_eq!(map_key("space"), Some(Key::Space));
        assert_eq!(map_key("f13"), None);
    }

    #[test]
    fn utf16_offsets_round_trip() {
        let s = "a日😀b";
        // a=1, 日=1, 😀=2 UTF-16 units; bytes 1, 3, 4.
        assert_eq!(byte_at(s, 1), 1);
        assert_eq!(byte_at(s, 2), 4);
        assert_eq!(byte_at(s, 4), 8);
        assert_eq!(byte_at(s, 99), s.len());
        assert_eq!(utf16_at(s, 8), 4);
        assert_eq!(utf16(s, &(1..8)), 1..4);
        assert_eq!(bytes(s, &(2..4)), 4..8);
    }
}

/// gpui-ce: lumen's nodes as synthetic children of the view's element.
/// Every node is pushed under the view (that's how the builder adds
/// them), then the view's children are cut back to lumen's top level; the
/// nested nodes keep their own child lists, so the tree keeps its shape.
#[cfg(feature = "ce")]
fn a11y_children(b: &mut gpui::A11ySubtreeBuilder, t: &lumen_core::A11yTree, view: u64, origin: lumen_core::Vec2, scale: f32) {
    use gpui::accesskit::Rect as AkRect;
    let Some(root) = t.get(t.root) else { return };
    let s = scale as f64;
    let ids = |c: &[u64]| c.iter().map(|c| ak_id(view, *c)).collect::<Vec<_>>();
    for n in t.nodes.iter().filter(|n| n.id != t.root) {
        let mut node = n.to_accesskit(scale);
        if !n.children.is_empty() {
            node.set_children(ids(&n.children));
        }
        let r = n.rect.translate(origin);
        node.set_bounds(AkRect { x0: r.min.x as f64 * s, y0: r.min.y as f64 * s, x1: r.max.x as f64 * s, y1: r.max.y as f64 * s });
        b.push_child(ak_id(view, n.id), node);
    }
    b.parent_node().set_children(ids(&root.children));
}

/// lumen's node ids are only unique within one view (they derive from the
/// same root id), so each view's nodes are scoped by its entity id in the
/// window's tree.
#[cfg(feature = "ce")]
fn ak_id(view: u64, id: u64) -> gpui::accesskit::NodeId {
    gpui::accesskit::NodeId(lumen_core::Id(view).with(id).0)
}

/// gpui-ce: route screen reader actions on lumen's nodes back into lumen.
#[cfg(feature = "ce")]
fn a11y_listeners<S: 'static>(entity: &Entity<LumenView<S>>, window: &mut Window, cx: &mut App) {
    use gpui::accesskit::{Action, ActionData, ActionRequest, NodeId, TreeId};
    let Some((tree, _, _)) = entity.read(cx).a11y.clone() else { return };
    let view = entity.entity_id().as_u64();
    for n in &tree.nodes {
        let a = n.actions;
        let mut kinds = vec![Action::ScrollIntoView];
        if a.click {
            kinds.push(Action::Click);
        }
        if a.focus {
            kinds.push(Action::Focus);
        }
        if a.increment {
            kinds.extend([Action::Increment, Action::Decrement]);
        }
        if a.set_value {
            kinds.push(Action::SetValue);
        }
        if a.expand {
            kinds.extend([Action::Expand, Action::Collapse]);
        }
        for action in kinds {
            let e = entity.clone();
            let id = n.id;
            window.on_a11y_action(ak_id(view, id), action, move |data: Option<&ActionData>, _, cx| {
                let req = ActionRequest { action, target_tree: TreeId::ROOT, target_node: NodeId(id), data: data.cloned() };
                if let Some((target, act)) = lumen_core::A11yAction::from_accesskit(&req) {
                    e.update(cx, |this, cx| {
                        this.events.push(Event::A11y { target, action: act });
                        this.refresh(cx);
                    });
                }
            });
        }
    }
}
