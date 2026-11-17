//! Window, event loop and GPU setup for lumen, on native and the web.
//!
//! ```no_run
//! lumen_app::App::new("Hello").size(480, 320).run(0u32, |ui, n| {
//!     ui.column().pad(24).gap(12).show(|ui| {
//!         if ui.button("Click me").primary().clicked() { *n += 1; }
//!         ui.label(format!("Clicked {n} times"));
//!     });
//! });
//! ```

use std::sync::Arc;

use lumen_core::input::{CursorIcon, Event, Key, Modifiers, PointerButton, RawInput};
use lumen_core::{vec2, Accent, Look, Mode, Ui};
use lumen_wgpu::Renderer;
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{Key as WKey, NamedKey};
use winit::window::{Window, WindowId};

pub mod system;

#[cfg(not(target_arch = "wasm32"))]
mod screenshot;
#[cfg(not(target_arch = "wasm32"))]
pub use screenshot::{screenshot, Shot, ShotStep};

/// Application builder.
pub struct App {
    title: String,
    size: (u32, u32),
    min_size: Option<(u32, u32)>,
    mode: Option<Mode>,
    accent: Accent,
    look: Look,
    follow_reduced_motion: bool,
    canvas_id: Option<String>,
}

impl App {
    pub fn new(title: impl Into<String>) -> Self {
        App {
            title: title.into(),
            size: (960, 640),
            min_size: None,
            mode: None,
            accent: Accent::Violet,
            look: Look::lumen(),
            follow_reduced_motion: true,
            canvas_id: None,
        }
    }
    /// Initial window size in logical pixels.
    pub fn size(mut self, w: u32, h: u32) -> Self {
        self.size = (w, h);
        self
    }
    pub fn min_size(mut self, w: u32, h: u32) -> Self {
        self.min_size = Some((w, h));
        self
    }
    /// Start in this mode (default: follow the OS).
    pub fn mode(mut self, m: Mode) -> Self {
        self.mode = Some(m);
        self
    }
    pub fn accent(mut self, a: Accent) -> Self {
        self.accent = a;
        self
    }
    /// The visual language (default: lumen's own). See [`Look`].
    pub fn look(mut self, l: Look) -> Self {
        self.look = l;
        self
    }
    /// Look like the OS: its look, its accent color, its fonts.
    pub fn native(self) -> Self {
        self.look(Look::native()).accent(Accent::Native)
    }
    /// Follow the OS reduce-motion setting (default on).
    pub fn follow_reduced_motion(mut self, on: bool) -> Self {
        self.follow_reduced_motion = on;
        self
    }
    /// Web only: render into an existing `<canvas id=...>` instead of appending one.
    pub fn canvas(mut self, id: impl Into<String>) -> Self {
        self.canvas_id = Some(id.into());
        self
    }

    /// Run the app. `state` is your plain struct; `view` is called every
    /// frame with the UI and `&mut state`.
    pub fn run<S: 'static>(self, state: S, view: impl FnMut(&mut Ui, &mut S) + 'static) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).try_init();
        }
        #[cfg(target_arch = "wasm32")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            let _ = console_log::init_with_level(log::Level::Warn);
        }
        let event_loop = EventLoop::<Gpu>::with_user_event().build().expect("event loop");
        let proxy = event_loop.create_proxy();
        let mut ui = Ui::new();
        ui.theme = lumen_core::theme::Theme::with_look(self.mode.unwrap_or(Mode::Light), self.accent, self.look);
        let runner = Runner {
            app: self,
            ui,
            state,
            view: Box::new(view),
            window: None,
            gpu: None,
            proxy: Some(proxy),
            start: Instant::now(),
            events: vec![],
            modifiers: Modifiers { mac: cfg!(target_os = "macos") || is_mac_web(), ..Default::default() },
            scale: 1.0,
            next_frame: None,
            clipboard: Clipboard::new(),
            cursor: CursorIcon::Default,
            followed_system_theme: false,
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut runner = runner;
            event_loop.run_app(&mut runner).expect("run");
        }
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::EventLoopExtWebSys;
            event_loop.spawn_app(runner);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn is_mac_web() -> bool {
    web_sys::window().and_then(|w| w.navigator().platform().ok()).is_some_and(|p| p.to_lowercase().contains("mac"))
}
#[cfg(not(target_arch = "wasm32"))]
fn is_mac_web() -> bool {
    false
}

/// GPU objects, created asynchronously on the web.
pub struct Gpu {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    view_format: wgpu::TextureFormat,
    renderer: Renderer,
}

async fn create_gpu(window: Arc<Window>, size: (u32, u32)) -> Result<Gpu, String> {
    let desc = wgpu::InstanceDescriptor::new_without_display_handle_from_env();
    #[cfg(target_arch = "wasm32")]
    let instance = wgpu::util::new_instance_with_webgpu_detection(desc).await;
    #[cfg(not(target_arch = "wasm32"))]
    let instance = wgpu::Instance::new(desc);
    let surface = instance.create_surface(window.clone()).map_err(|e| e.to_string())?;
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            ..Default::default()
        })
        .await
        .map_err(|e| e.to_string())?;
    let limits = if cfg!(target_arch = "wasm32") { wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()) } else { wgpu::Limits::default() };
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor { label: Some("lumen"), required_limits: limits, ..Default::default() })
        .await
        .map_err(|e| e.to_string())?;
    let (w, h) = (size.0.max(1), size.1.max(1));
    let mut config = surface.get_default_config(&adapter, w, h).ok_or("surface unsupported")?;
    let caps = surface.get_capabilities(&adapter);
    // lumen blends in gamma space: prefer a non-sRGB surface; otherwise
    // render through a non-sRGB view of the sRGB surface.
    let format = caps.formats.iter().copied().find(|f| !f.is_srgb()).unwrap_or(config.format);
    config.format = format;
    let view_format = format.remove_srgb_suffix();
    if view_format != format {
        config.view_formats = vec![view_format];
    }
    config.present_mode = wgpu::PresentMode::AutoVsync;
    surface.configure(&device, &config);
    let renderer = Renderer::new(&device, view_format);
    Ok(Gpu { surface, device, queue, config, view_format, renderer })
}

type ViewFn<S> = Box<dyn FnMut(&mut Ui, &mut S)>;

struct Runner<S> {
    app: App,
    ui: Ui,
    state: S,
    view: ViewFn<S>,
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    proxy: Option<EventLoopProxy<Gpu>>,
    start: Instant,
    events: Vec<Event>,
    modifiers: Modifiers,
    scale: f32,
    next_frame: Option<Instant>,
    clipboard: Clipboard,
    cursor: CursorIcon,
    followed_system_theme: bool,
}

impl<S> Runner<S> {
    fn refresh_system_prefs(&mut self) {
        let mut p = system::query();
        p.dark = self.window.as_ref().and_then(|w| w.theme()).map(|t| t == winit::window::Theme::Dark);
        if p != self.ui.system() {
            self.ui.set_system(p);
            if self.app.follow_reduced_motion {
                if let Some(r) = p.reduced_motion {
                    self.ui.set_reduced_motion(r);
                }
            }
        }
    }

    fn request_redraw(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn frame(&mut self) {
        let (Some(window), Some(gpu)) = (self.window.clone(), self.gpu.as_mut()) else { return };
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }
        if gpu.config.width != size.width || gpu.config.height != size.height {
            gpu.config.width = size.width;
            gpu.config.height = size.height;
            gpu.surface.configure(&gpu.device, &gpu.config);
        }
        if !self.followed_system_theme {
            if self.app.mode.is_none() {
                if let Some(t) = window.theme() {
                    let mode = if t == winit::window::Theme::Dark { Mode::Dark } else { Mode::Light };
                    let (accent, look) = (self.ui.accent(), *self.ui.theme.target_look());
                    self.ui.theme = lumen_core::theme::Theme::with_look(mode, accent, look);
                }
            }
            self.followed_system_theme = true;
            self.refresh_system_prefs();
            // The first frame shows the OS accent without a blend.
            let (mode, accent, look) = (self.ui.mode(), self.ui.accent(), *self.ui.theme.target_look());
            let native = self.ui.system().accent;
            let reduced = self.ui.theme.reduced_motion;
            self.ui.theme = lumen_core::theme::Theme::with_look(mode, accent, look);
            self.ui.theme.reduced_motion = reduced;
            self.ui.set_native_accent(native);
            self.ui.theme.settle();
        }
        let scale = self.scale;
        let input = RawInput {
            screen_size: vec2(size.width as f32 / scale, size.height as f32 / scale),
            scale,
            time: self.start.elapsed().as_secs_f64(),
            events: std::mem::take(&mut self.events),
            modifiers: self.modifiers,
        };
        self.ui.begin_frame(input);
        (self.view)(&mut self.ui, &mut self.state);
        let out = self.ui.end_frame();

        let gpu = self.gpu.as_mut().unwrap();
        let frame = match gpu.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f) | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                gpu.surface.configure(&gpu.device, &gpu.config);
                self.request_redraw();
                return;
            }
            _ => {
                self.request_redraw();
                return;
            }
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor { format: Some(gpu.view_format), ..Default::default() });
        gpu.renderer.draw_to(&gpu.device, &gpu.queue, &view, &out.draw);
        window.pre_present_notify();
        gpu.queue.present(frame);

        if let Some(t) = out.copied_text {
            self.clipboard.set(t);
        }
        if out.cursor != self.cursor {
            self.cursor = out.cursor;
            window.set_cursor(match out.cursor {
                CursorIcon::Default => winit::window::CursorIcon::Default,
                CursorIcon::Pointer => winit::window::CursorIcon::Pointer,
                CursorIcon::Text => winit::window::CursorIcon::Text,
                CursorIcon::Grab => winit::window::CursorIcon::Grab,
                CursorIcon::Grabbing => winit::window::CursorIcon::Grabbing,
            });
        }
        window.set_ime_allowed(out.wants_text_input);
        self.next_frame = out.repaint_after.map(|s| Instant::now() + std::time::Duration::from_secs_f64(s));
        if out.repaint_after == Some(0.0) {
            self.request_redraw();
        }
    }
}

impl<S> ApplicationHandler<Gpu> for Runner<S> {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let mut attrs = Window::default_attributes().with_title(self.app.title.clone());
        // On the web the canvas is sized by CSS; window sizes apply natively.
        #[cfg(not(target_arch = "wasm32"))]
        {
            attrs = attrs.with_inner_size(winit::dpi::LogicalSize::new(self.app.size.0, self.app.size.1));
            if let Some((w, h)) = self.app.min_size {
                attrs = attrs.with_min_inner_size(winit::dpi::LogicalSize::new(w, h));
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;
            let canvas = self.app.canvas_id.as_ref().and_then(|id| {
                web_sys::window()?.document()?.get_element_by_id(id)?.dyn_into::<web_sys::HtmlCanvasElement>().ok()
            });
            attrs = if canvas.is_some() { attrs.with_canvas(canvas) } else { attrs.with_append(true) };
            attrs = attrs.with_prevent_default(true).with_focusable(true);
        }
        let window = Arc::new(el.create_window(attrs).expect("window"));
        self.scale = window.scale_factor() as f32;
        self.window = Some(window.clone());
        let size = window.inner_size();
        let proxy = self.proxy.take().unwrap();
        #[cfg(not(target_arch = "wasm32"))]
        {
            let gpu = pollster::block_on(create_gpu(window, (size.width, size.height))).expect("GPU init");
            let _ = proxy.send_event(gpu);
        }
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                match create_gpu(window, (size.width, size.height)).await {
                    Ok(gpu) => {
                        let _ = proxy.send_event(gpu);
                    }
                    Err(e) => log::error!("lumen: GPU init failed: {e}"),
                }
            });
        }
    }

    fn user_event(&mut self, _el: &ActiveEventLoop, gpu: Gpu) {
        self.gpu = Some(gpu);
        self.request_redraw();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let scale = self.scale;
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::RedrawRequested => self.frame(),
            WindowEvent::Resized(_) => self.request_redraw(),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale = scale_factor as f32;
                self.request_redraw();
            }
            WindowEvent::ThemeChanged(t) => {
                // Follow the OS live, with the usual blend.
                if self.app.mode.is_none() {
                    self.ui.set_mode(if t == winit::window::Theme::Dark { Mode::Dark } else { Mode::Light });
                }
                self.refresh_system_prefs();
                self.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.events.push(Event::PointerMoved(vec2(position.x as f32 / scale, position.y as f32 / scale)));
                self.request_redraw();
            }
            WindowEvent::CursorLeft { .. } => {
                self.events.push(Event::PointerLeft);
                self.request_redraw();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let button = match button {
                    MouseButton::Left => PointerButton::Primary,
                    MouseButton::Right => PointerButton::Secondary,
                    MouseButton::Middle => PointerButton::Middle,
                    _ => return,
                };
                let pos = self.ui.pointer().unwrap_or_default();
                self.events.push(Event::PointerButton { pos, button, pressed: state == ElementState::Pressed });
                self.request_redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(x, y) => vec2(-x * 40.0, -y * 40.0),
                    MouseScrollDelta::PixelDelta(p) => vec2(-p.x as f32 / scale, -p.y as f32 / scale),
                };
                self.events.push(Event::Scroll(d));
                self.request_redraw();
            }
            WindowEvent::ModifiersChanged(m) => {
                let s = m.state();
                self.modifiers = Modifiers { shift: s.shift_key(), ctrl: s.control_key(), alt: s.alt_key(), logo: s.super_key(), mac: self.modifiers.mac };
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;
                if let Some(key) = map_key(&event.logical_key) {
                    let m = self.modifiers;
                    if pressed && m.command() && key == Key::Char('v') {
                        if let Some(t) = self.clipboard.get() {
                            self.events.push(Event::Paste(t));
                        }
                    }
                    self.events.push(Event::Key { key, pressed, repeat: event.repeat, modifiers: m });
                }
                if pressed && !self.modifiers.command() && !self.modifiers.ctrl {
                    if let Some(t) = event.text.as_ref().map(|t| t.to_string()) {
                        if t.chars().any(|c| !c.is_control()) {
                            self.events.push(Event::Text(t));
                        }
                    }
                }
                self.request_redraw();
            }
            WindowEvent::Ime(winit::event::Ime::Commit(t)) => {
                self.events.push(Event::Text(t));
                self.request_redraw();
            }
            WindowEvent::Focused(f) => {
                if f {
                    // Accent and motion settings may have changed meanwhile.
                    self.refresh_system_prefs();
                }
                self.events.push(Event::WindowFocused(f));
                self.request_redraw();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        match self.next_frame {
            Some(t) if t <= Instant::now() => {
                self.next_frame = None;
                self.request_redraw();
                el.set_control_flow(ControlFlow::Wait);
            }
            Some(t) => el.set_control_flow(ControlFlow::WaitUntil(t)),
            None => el.set_control_flow(ControlFlow::Wait),
        }
    }
}

fn map_key(k: &WKey) -> Option<Key> {
    Some(match k {
        WKey::Named(n) => match n {
            NamedKey::Tab => Key::Tab,
            NamedKey::Enter => Key::Enter,
            NamedKey::Space => Key::Space,
            NamedKey::Escape => Key::Escape,
            NamedKey::Backspace => Key::Backspace,
            NamedKey::Delete => Key::Delete,
            NamedKey::ArrowLeft => Key::ArrowLeft,
            NamedKey::ArrowRight => Key::ArrowRight,
            NamedKey::ArrowUp => Key::ArrowUp,
            NamedKey::ArrowDown => Key::ArrowDown,
            NamedKey::Home => Key::Home,
            NamedKey::End => Key::End,
            NamedKey::PageUp => Key::PageUp,
            NamedKey::PageDown => Key::PageDown,
            _ => return None,
        },
        WKey::Character(c) => {
            let ch = c.chars().next()?.to_lowercase().next()?;
            if ch == ' ' {
                Key::Space
            } else {
                Key::Char(ch)
            }
        }
        _ => return None,
    })
}

/// System clipboard on native, an in-process one on the web.
struct Clipboard {
    #[cfg(not(target_arch = "wasm32"))]
    sys: Option<arboard::Clipboard>,
    local: Option<String>,
}

impl Clipboard {
    fn new() -> Self {
        Clipboard {
            #[cfg(not(target_arch = "wasm32"))]
            sys: arboard::Clipboard::new().ok(),
            local: None,
        }
    }
    fn set(&mut self, t: String) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(c) = self.sys.as_mut() {
            let _ = c.set_text(t.clone());
        }
        self.local = Some(t);
    }
    fn get(&mut self) -> Option<String> {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(c) = self.sys.as_mut() {
            if let Ok(t) = c.get_text() {
                return Some(t);
            }
        }
        self.local.clone()
    }
}
