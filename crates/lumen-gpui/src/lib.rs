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
//! use gpui::{prelude::*, div, px, size, App, Application, Bounds, Entity, Window, WindowBounds, WindowOptions};
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
//! Application::new().run(|cx: &mut App| {
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
//! pixels back and hands them to GPUI as an image. GPUI mouse and keyboard
//! events are translated into lumen input. A view at rest renders nothing:
//! frames are only produced after input, a state change you make through
//! [`LumenView::update`], a theme change, a resize, or while lumen is
//! animating.
//!
//! The readback costs about a millisecond per frame for a typical panel on
//! Apple silicon, so this is the right tool for panels, inspectors, charts
//! and dialogs, not for a full-window lumen app (use `lumen::App` for that).
//! IME composition is not forwarded yet; typed text arrives as committed
//! characters.

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    canvas, div, prelude::*, App, Bounds, ClipboardItem, Context, Corners, CursorStyle, DispatchPhase, Entity,
    FocusHandle, Global, Hitbox, HitboxBehavior, KeyDownEvent, KeyUpEvent, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Point, RenderImage, ScrollDelta, ScrollWheelEvent, Subscription, Task,
    Window, WindowAppearance,
};
use lumen_core::input::{CursorIcon, Event, Key, Modifiers, PointerButton, RawInput};
use lumen_core::theme::Theme;
use lumen_core::{vec2, Accent, Color, Look, Mode, Ui};
use lumen_wgpu::offscreen::{self, Readback};
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

struct Target {
    gpu: Arc<SharedGpu>,
    renderer: Renderer,
    readback: Readback,
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
    origin: Point<Pixels>,
    hovered: bool,
    pressed: u8,

    target: Option<Target>,
    image: Option<Arc<RenderImage>>,
    retired: Vec<Arc<RenderImage>>,
    size_px: [u32; 2],
    scale: f32,
    dirty: bool,
    cursor: CursorIcon,
    timer: Option<Task<()>>,

    mode: Option<Mode>,
    transparent: bool,
    started: bool,
    _appearance: Subscription,
}

impl<S: 'static> LumenView<S> {
    /// Create the view. Call inside `cx.new(|cx| ...)`.
    pub fn new(state: S, view: impl FnMut(&mut Ui, &mut S) + 'static, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut ui = Ui::new();
        ui.theme = Theme::with_look(Mode::Light, Accent::Blue, Look::lumen());
        let appearance = cx.observe_window_appearance(window, |this, _, cx| {
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
            origin: Point::default(),
            hovered: false,
            pressed: 0,
            target: None,
            image: None,
            retired: vec![],
            size_px: [0, 0],
            scale: 1.0,
            dirty: true,
            cursor: CursorIcon::Default,
            timer: None,
            mode: None,
            transparent: false,
            started: false,
            _appearance: appearance,
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

    fn on_down(&mut self, e: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Some(button) = map_button(e.button) else { return };
        self.modifiers = map_mods(&e.modifiers, self.modifiers.mac);
        window.focus(&self.focus);
        self.pressed |= bit(button);
        let pos = self.local(e.position);
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
        if !m.command() && !m.ctrl {
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

    /// Run a lumen frame if anything changed and return the image to paint.
    fn frame(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut Context<Self>) -> Option<Arc<RenderImage>> {
        self.origin = bounds.origin;
        let scale = window.scale_factor();
        let size_px = [(f32::from(bounds.size.width) * scale).round() as u32, (f32::from(bounds.size.height) * scale).round() as u32];
        if size_px[0] == 0 || size_px[1] == 0 {
            return None;
        }
        if size_px != self.size_px || scale != self.scale {
            self.size_px = size_px;
            self.scale = scale;
            self.dirty = true;
        }
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
        if !self.dirty {
            return self.image.clone();
        }
        self.dirty = false;

        if self.target.is_none() {
            let gpu = shared_gpu(cx)?;
            // GPUI sprites are BGRA, so render BGRA and skip a CPU swizzle.
            let format = wgpu::TextureFormat::Bgra8Unorm;
            let renderer = Renderer::new(&gpu.device, format);
            self.target = Some(Target { gpu, renderer, readback: Readback::with_format(format) });
        }

        let input = RawInput {
            screen_size: vec2(size_px[0] as f32 / scale, size_px[1] as f32 / scale),
            scale,
            time: self.start.elapsed().as_secs_f64(),
            events: std::mem::take(&mut self.events),
            modifiers: self.modifiers,
        };
        self.ui.begin_frame(input);
        (self.view)(&mut self.ui, &mut self.state);
        let mut out = self.ui.end_frame();
        if self.transparent {
            out.draw.clear = Color::TRANSPARENT;
        }

        let t = self.target.as_mut().unwrap();
        let t0 = Instant::now();
        let [w, h] = [out.draw.size_px[0].max(1), out.draw.size_px[1].max(1)];
        let mut px = t.readback.render(&t.gpu.device, &t.gpu.queue, &mut t.renderer, &out.draw).to_vec();
        // lumen blends premultiplied; GPUI sprites want straight alpha. Only
        // a transparent view has partially covered pixels to fix up.
        if self.transparent {
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
        let image = Arc::new(RenderImage::new(vec![image::Frame::new(image::RgbaImage::from_raw(w, h, px)?)]));
        if std::env::var_os("LUMEN_GPUI_TRACE").is_some() {
            eprintln!("lumen-gpui: {w}x{h} px frame rendered and read back in {:.2} ms", t0.elapsed().as_secs_f64() * 1e3);
        }

        // GPUI caches images in its sprite atlas until told otherwise. Keep
        // the previous frame's image for one more frame, then release it.
        if let Some(old) = self.image.replace(image.clone()) {
            self.retired.push(old);
        }
        while self.retired.len() > 1 {
            let old = self.retired.remove(0);
            cx.drop_image(old, Some(window));
        }

        if let Some(text) = out.copied_text {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
        self.cursor = out.cursor;
        self.timer = None;
        match out.repaint_after {
            Some(s) if s <= 0.0 => {
                self.dirty = true;
                let id = cx.entity_id();
                window.on_next_frame(move |_, cx| cx.notify(id));
            }
            Some(s) => {
                let Ok(delay) = Duration::try_from_secs_f64(s) else { return Some(image) };
                self.timer = Some(cx.spawn(async move |this, cx| {
                    cx.background_executor().timer(delay).await;
                    let _ = this.update(cx, |this, cx| this.refresh(cx));
                }));
            }
            None => {}
        }
        Some(image)
    }
}

impl<S: 'static> Render for LumenView<S> {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        div()
            .size_full()
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
    }
}

fn paint<S: 'static>(entity: Entity<LumenView<S>>, bounds: Bounds<Pixels>, hitbox: Hitbox, window: &mut Window, cx: &mut App) {
    if let Some(image) = entity.update(cx, |this, cx| this.frame(bounds, window, cx)) {
        let _ = window.paint_image(bounds, Corners::default(), image, 0, false);
    }
    let cursor = entity.read(cx).cursor;
    window.set_cursor_style(map_cursor(cursor), &hitbox);

    let (e, hb) = (entity.clone(), hitbox.clone());
    window.on_mouse_event(move |ev: &MouseMoveEvent, phase, window, cx| {
        if phase == DispatchPhase::Bubble {
            let hovered = hb.is_hovered(window);
            e.update(cx, |this, cx| this.on_move(ev, hovered, cx));
        }
    });
    let (e, hb) = (entity.clone(), hitbox.clone());
    window.on_mouse_event(move |ev: &MouseDownEvent, phase, window, cx| {
        if phase == DispatchPhase::Bubble && hb.is_hovered(window) {
            e.update(cx, |this, cx| this.on_down(ev, window, cx));
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
        if phase == DispatchPhase::Bubble && hb.should_handle_scroll(window) {
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
}
