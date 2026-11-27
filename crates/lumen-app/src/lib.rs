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
use lumen_core::{vec2, Accent, Look, Mode, Ui, Vec2, WindowRegion, WindowRequest};
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
    custom_chrome: bool,
}

impl App {
    pub fn new(title: impl Into<String>) -> Self {
        App {
            title: title.into(),
            size: (960, 640),
            min_size: None,
            mode: None,
            accent: Accent::Blue,
            look: Look::lumen(),
            follow_reduced_motion: true,
            canvas_id: None,
            custom_chrome: false,
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
    /// Draw the title bar yourself with `ui.window_chrome(..).window()`.
    /// The OS title bar goes away (on macOS the traffic lights are hidden
    /// and content runs under a transparent title bar, so the window keeps
    /// its rounded corners and shadow). Dragging lumen's title bar moves the
    /// window, double-clicking it maximizes, its buttons minimize, zoom and
    /// close, and on Windows and Linux the window edges resize it.
    pub fn custom_chrome(mut self) -> Self {
        self.custom_chrome = true;
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
        let event_loop = EventLoop::<UserEvent>::with_user_event().build().expect("event loop");
        let proxy = event_loop.create_proxy();
        let mut ui = Ui::new();
        // CJK, Arabic, emoji and other scripts fall back to system fonts.
        ui.text.set_system_fallback(true);
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
            ime_rect: None,
            clipboard: Clipboard::new(),
            cursor: CursorIcon::Default,
            followed_system_theme: false,
            exit: false,
            maximized: false,
            last_title_press: None,
            resize_dir: None,
            pointer: Vec2::ZERO,
            #[cfg(not(target_arch = "wasm32"))]
            a11y: None,
            #[cfg(not(target_arch = "wasm32"))]
            a11y_sent: None,
            trace: std::env::var_os("LUMEN_TRACE_FRAMES").map(|_| FrameTrace { since: Instant::now(), last_present: None, cpu: vec![], submit: vec![], interval: vec![] }),
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
/// Events the runner sends itself: the GPU once it's ready, and screen
/// reader requests.
pub enum UserEvent {
    Gpu(Box<Gpu>),
    #[cfg(not(target_arch = "wasm32"))]
    A11y(accesskit_winit::Event),
}

#[cfg(not(target_arch = "wasm32"))]
impl From<accesskit_winit::Event> for UserEvent {
    fn from(e: accesskit_winit::Event) -> Self {
        UserEvent::A11y(e)
    }
}

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
    proxy: Option<EventLoopProxy<UserEvent>>,
    /// Screen reader bridge (AccessKit), created with the window.
    #[cfg(not(target_arch = "wasm32"))]
    a11y: Option<accesskit_winit::Adapter>,
    /// The tree last sent, so updates carry only what changed.
    #[cfg(not(target_arch = "wasm32"))]
    a11y_sent: Option<lumen_core::A11yTree>,
    start: Instant,
    events: Vec<Event>,
    modifiers: Modifiers,
    scale: f32,
    next_frame: Option<Instant>,
    ime_rect: Option<lumen_core::Rect>,
    clipboard: Clipboard,
    cursor: CursorIcon,
    followed_system_theme: bool,
    /// A close request from the UI, acted on in `about_to_wait`.
    exit: bool,
    maximized: bool,
    /// Time and place of the last press on the title bar (double-click).
    last_title_press: Option<(Instant, Vec2)>,
    /// Pointer is on a resizable window edge (custom chrome, not macOS).
    resize_dir: Option<winit::window::ResizeDirection>,
    /// Latest pointer position from winit (the UI's lags by a frame).
    pointer: Vec2,
    /// `LUMEN_TRACE_FRAMES=1`: frame times, printed every two seconds.
    trace: Option<FrameTrace>,
}

/// Frame statistics for `LUMEN_TRACE_FRAMES=1`.
struct FrameTrace {
    since: Instant,
    last_present: Option<Instant>,
    /// Per frame: the UI (view, layout, tessellation), encoding and
    /// submitting the draw (not waiting for a drawable), and the interval
    /// between presents.
    cpu: Vec<f64>,
    submit: Vec<f64>,
    interval: Vec<f64>,
}

impl FrameTrace {
    fn report(&mut self) {
        if self.since.elapsed().as_secs_f64() < 2.0 || self.cpu.is_empty() {
            return;
        }
        let pct = |v: &mut Vec<f64>, p: f64| {
            v.sort_by(|a, b| a.total_cmp(b));
            v[((v.len() - 1) as f64 * p) as usize]
        };
        let secs = self.since.elapsed().as_secs_f64();
        let fps = self.interval.len() as f64 / secs;
        let (c50, c95, cmax) = (pct(&mut self.cpu, 0.5), pct(&mut self.cpu, 0.95), pct(&mut self.cpu, 1.0));
        let (s50, s95) = (pct(&mut self.submit, 0.5), pct(&mut self.submit, 0.95));
        let (i95, imax) = if self.interval.is_empty() { (0.0, 0.0) } else { (pct(&mut self.interval, 0.95), pct(&mut self.interval, 1.0)) };
        eprintln!(
            "lumen: {fps:.1} fps | ui p50 {c50:.2} ms p95 {c95:.2} ms max {cmax:.2} ms | draw submit p50 {s50:.2} ms p95 {s95:.2} ms | present interval p95 {i95:.1} ms max {imax:.1} ms | {} frames",
            self.cpu.len()
        );
        self.cpu.clear();
        self.submit.clear();
        self.interval.clear();
        self.since = Instant::now();
    }
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

    /// A primary press with custom chrome: resize from an edge, maximize on
    /// a title bar double-click, or start moving the window. Window moves
    /// must start inside the mouse-down event (macOS reads the current
    /// event). Returns true when the press was consumed.
    fn chrome_press(&mut self, pos: Vec2) -> bool {
        let Some(window) = self.window.clone() else { return false };
        if let Some(dir) = self.resize_dir {
            return window.drag_resize_window(dir).is_ok();
        }
        if self.ui.window_region_at(pos) != Some(WindowRegion::Drag) {
            self.last_title_press = None;
            return false;
        }
        let now = Instant::now();
        let double = self.last_title_press.is_some_and(|(t, p)| now.duration_since(t).as_secs_f32() < 0.4 && (p - pos).length() < 6.0);
        // The UI still sees the press (ChromeResponse::drag_started and
        // double_clicked); the move eats the release, so send it too.
        self.events.push(Event::PointerButton { pos, button: PointerButton::Primary, pressed: true });
        if double {
            self.last_title_press = None;
            match system::title_double_click() {
                system::TitleAction::Maximize => window.set_maximized(!window.is_maximized()),
                system::TitleAction::Minimize => window.set_minimized(true),
                system::TitleAction::None => {}
            }
        } else {
            self.last_title_press = Some((now, pos));
            let _ = window.drag_window();
        }
        self.events.push(Event::PointerButton { pos, button: PointerButton::Primary, pressed: false });
        true
    }

    /// Custom chrome on Windows and Linux: the outer 5 px resize the window.
    fn update_resize_edge(&mut self, pos: Vec2) {
        use winit::window::ResizeDirection as R;
        if !self.app.custom_chrome || cfg!(target_os = "macos") || cfg!(target_arch = "wasm32") {
            return;
        }
        let Some(window) = self.window.clone() else { return };
        let size = self.ui.screen_size();
        let e = 5.0;
        let dir = if window.is_maximized() || !window.is_resizable() {
            None
        } else {
            let (l, r, t, b) = (pos.x < e, pos.x > size.x - e, pos.y < e, pos.y > size.y - e);
            match (l, r, t, b) {
                (true, _, true, _) => Some(R::NorthWest),
                (_, true, true, _) => Some(R::NorthEast),
                (true, _, _, true) => Some(R::SouthWest),
                (_, true, _, true) => Some(R::SouthEast),
                (true, ..) => Some(R::West),
                (_, true, ..) => Some(R::East),
                (_, _, true, _) => Some(R::North),
                (_, _, _, true) => Some(R::South),
                _ => None,
            }
        };
        if dir != self.resize_dir {
            self.resize_dir = dir;
            use winit::window::CursorIcon as C;
            window.set_cursor(match dir {
                Some(R::West | R::East) => C::EwResize,
                Some(R::North | R::South) => C::NsResize,
                Some(R::NorthWest | R::SouthEast) => C::NwseResize,
                Some(R::NorthEast | R::SouthWest) => C::NeswResize,
                None => {
                    // Back to whatever the UI asked for.
                    self.cursor = CursorIcon::Default;
                    C::Default
                }
            });
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
        let t_frame = Instant::now();
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
        #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
        let mut out = self.ui.end_frame();
        let ui_ms = t_frame.elapsed().as_secs_f64() * 1e3;
        #[cfg(not(target_arch = "wasm32"))]
        if let (Some(tree), Some(adapter)) = (out.a11y.take(), self.a11y.as_mut()) {
            let prev = self.a11y_sent.take();
            let title = &self.app.title;
            adapter.update_if_active(|| tree.update_since(prev.as_ref(), scale, title));
            self.a11y_sent = Some(tree);
        }

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
        let t_submit = Instant::now();
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor { format: Some(gpu.view_format), ..Default::default() });
        gpu.renderer.draw_to(&gpu.device, &gpu.queue, &view, &out.draw);
        window.pre_present_notify();
        gpu.queue.present(frame);
        if let Some(t) = self.trace.as_mut() {
            let now = Instant::now();
            t.cpu.push(ui_ms);
            t.submit.push(now.duration_since(t_submit).as_secs_f64() * 1e3);
            if let Some(p) = t.last_present {
                t.interval.push(now.duration_since(p).as_secs_f64() * 1e3);
            }
            t.last_present = Some(now);
            t.report();
        }

        if let Some(t) = out.copied_text {
            self.clipboard.set(t);
        }
        for req in &out.window_requests {
            match req {
                WindowRequest::Close => self.exit = true,
                WindowRequest::Minimize => window.set_minimized(true),
                WindowRequest::ToggleMaximize => window.set_maximized(!window.is_maximized()),
            }
        }
        if out.cursor != self.cursor && self.resize_dir.is_none() {
            self.cursor = out.cursor;
            window.set_cursor(match out.cursor {
                CursorIcon::Default => winit::window::CursorIcon::Default,
                CursorIcon::Pointer => winit::window::CursorIcon::Pointer,
                CursorIcon::Text => winit::window::CursorIcon::Text,
                CursorIcon::Grab => winit::window::CursorIcon::Grab,
                CursorIcon::Grabbing => winit::window::CursorIcon::Grabbing,
                CursorIcon::ResizeHorizontal => winit::window::CursorIcon::EwResize,
            });
        }
        window.set_ime_allowed(out.wants_text_input);
        if let Some(f) = &out.text_field {
            // Put the IME candidate window under the caret or composition.
            if self.ime_rect != Some(f.rect) {
                self.ime_rect = Some(f.rect);
                let r = f.rect;
                window.set_ime_cursor_area(winit::dpi::LogicalPosition::new(r.min.x, r.min.y), winit::dpi::LogicalSize::new(r.width(), r.height()));
            }
        } else {
            self.ime_rect = None;
        }
        self.next_frame = out.repaint_after.and_then(|s| std::time::Duration::try_from_secs_f64(s).ok()).map(|d| Instant::now() + d);
        if out.repaint_after == Some(0.0) {
            self.request_redraw();
        }
    }
}

impl<S> ApplicationHandler<UserEvent> for Runner<S> {
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
            if self.app.custom_chrome {
                #[cfg(target_os = "macos")]
                {
                    use winit::platform::macos::WindowAttributesExtMacOS;
                    attrs = attrs
                        .with_titlebar_transparent(true)
                        .with_fullsize_content_view(true)
                        .with_title_hidden(true)
                        .with_titlebar_buttons_hidden(true)
                        .with_movable_by_window_background(false);
                }
                #[cfg(not(target_os = "macos"))]
                {
                    attrs = attrs.with_decorations(false);
                }
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
        #[cfg(not(target_arch = "wasm32"))]
        {
            // AccessKit must hook the window before it's first shown.
            attrs = attrs.with_visible(false);
        }
        let window = Arc::new(el.create_window(attrs).expect("window"));
        #[cfg(target_os = "macos")]
        if self.app.custom_chrome {
            mac_custom_chrome(&window);
        }
        self.scale = window.scale_factor() as f32;
        self.window = Some(window.clone());
        let size = window.inner_size();
        let proxy = self.proxy.take().unwrap();
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.a11y = Some(accesskit_winit::Adapter::with_event_loop_proxy(el, &window, proxy.clone()));
            window.set_visible(true);
            let gpu = pollster::block_on(create_gpu(window, (size.width, size.height))).expect("GPU init");
            let _ = proxy.send_event(UserEvent::Gpu(Box::new(gpu)));
        }
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                match create_gpu(window, (size.width, size.height)).await {
                    Ok(gpu) => {
                        let _ = proxy.send_event(UserEvent::Gpu(Box::new(gpu)));
                    }
                    Err(e) => log::error!("lumen: GPU init failed: {e}"),
                }
            });
        }
    }

    fn user_event(&mut self, _el: &ActiveEventLoop, ev: UserEvent) {
        match ev {
            UserEvent::Gpu(gpu) => {
                self.gpu = Some(*gpu);
                self.request_redraw();
            }
            #[cfg(not(target_arch = "wasm32"))]
            UserEvent::A11y(e) => {
                use accesskit_winit::WindowEvent as A;
                match e.window_event {
                    A::InitialTreeRequested => {
                        self.ui.set_accessibility(true);
                        self.a11y_sent = None;
                    }
                    A::ActionRequested(req) => {
                        if std::env::var_os("LUMEN_TRACE_A11Y").is_some() {
                            eprintln!("a11y request: {:?} on {:?} {:?}", req.action, req.target_node, req.data);
                        }
                        if let Some((target, action)) = lumen_core::A11yAction::from_accesskit(&req) {
                            self.events.push(Event::A11y { target, action });
                        }
                    }
                    A::AccessibilityDeactivated => {
                        self.ui.set_accessibility(false);
                        self.a11y_sent = None;
                    }
                }
                self.request_redraw();
            }
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let scale = self.scale;
        #[cfg(not(target_arch = "wasm32"))]
        if let (Some(a), Some(w)) = (self.a11y.as_mut(), self.window.as_ref()) {
            a.process_event(w, &event);
        }
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::RedrawRequested => self.frame(),
            WindowEvent::Resized(_) => {
                if let Some(w) = &self.window {
                    let m = w.is_maximized();
                    if m != self.maximized {
                        self.maximized = m;
                        self.events.push(Event::WindowMaximized(m));
                    }
                }
                self.request_redraw();
            }
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
                let pos = vec2(position.x as f32 / scale, position.y as f32 / scale);
                self.pointer = pos;
                self.update_resize_edge(pos);
                self.events.push(Event::PointerMoved(pos));
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
                let pos = self.pointer;
                let pressed = state == ElementState::Pressed;
                if pressed && button == PointerButton::Primary && self.app.custom_chrome && self.chrome_press(pos) {
                    self.request_redraw();
                    return;
                }
                self.events.push(Event::PointerButton { pos, button, pressed });
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
            WindowEvent::Ime(ime) => {
                use lumen_core::input::Ime;
                let e = match ime {
                    winit::event::Ime::Preedit(text, cursor) => Ime::Preedit { text, cursor, replace: None },
                    winit::event::Ime::Commit(text) => Ime::Commit { text, replace: None },
                    winit::event::Ime::Disabled => Ime::Preedit { text: String::new(), cursor: None, replace: None },
                    winit::event::Ime::Enabled => return,
                };
                self.events.push(Event::Ime(e));
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
        if self.exit {
            el.exit();
            return;
        }
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

/// AppKit starts its own window move for any mouse-down in the (hidden,
/// transparent) title bar strip, which swallows the mouse-up and would steal
/// clicks from lumen's caption buttons. Turn that off; lumen starts moves
/// itself with `drag_window` (`performWindowDragWithEvent:`), which still works.
#[cfg(target_os = "macos")]
fn mac_custom_chrome(window: &Window) {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let Ok(h) = window.window_handle() else { return };
    let RawWindowHandle::AppKit(h) = h.as_raw() else { return };
    // SAFETY: winit hands out a valid NSView pointer for the window's lifetime.
    let view: &objc2_app_kit::NSView = unsafe { h.ns_view.cast().as_ref() };
    if let Some(w) = view.window() {
        w.setMovable(false);
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
