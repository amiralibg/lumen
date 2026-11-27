//! Motion set & native window-chrome widgets:
//! - [`RollingNumber`] (per-digit vertical odometer reel that springs smoothly across digits)
//! - [`Ui::dock`] (Gaussian neighbor-magnifying floating icon dock with bouncy springs)
//! - [`Waveform`] (scrubbable audio / voice waveform player with play/pause and mm:ss readout)
//! - [`WindowChrome`] (platform-aware window header bar that morphs across Lumen, macOS, Windows and GNOME)

use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::color::Color;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::look::Platform;
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{Elevation, TextRole, Token};
use crate::ui::{Placement, Response, Sense, TooltipSpec, Ui, WindowRegion, WindowRequest};

use super::{blend3, key_activate};

// =========================================================== RollingNumber

/// Per-digit vertical odometer counter. Each digit slot spins independently
/// on the smooth spring and settles to zero idle wakeups at rest.
pub struct RollingNumber<'u> {
    ui: &'u mut Ui,
    key: String,
    value: i64,
    prefix: String,
    suffix: String,
    role: TextRole,
    shown: bool,
}

impl<'u> RollingNumber<'u> {
    pub fn prefix(mut self, p: impl Into<String>) -> Self {
        self.prefix = p.into();
        self
    }
    pub fn suffix(mut self, s: impl Into<String>) -> Self {
        self.suffix = s.into();
        self
    }
    pub fn role(mut self, role: TextRole) -> Self {
        self.role = role;
        self
    }
    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("rolling_num", &self.key));
        let mo = ui.motion();

        let mut st = ui.text_style(self.role);
        st.tabular = true;
        let digit_w = ui.galley("8", &st).width;
        let line_h = st.line_height.max(st.size * 1.2).round();

        let neg = self.value < 0;
        let abs_str = self.value.unsigned_abs().to_string();
        let digits_len = abs_str.len();
        let mut formatted = String::with_capacity(digits_len + digits_len / 3 + 2);
        if neg {
            formatted.push('−');
        }
        for (i, ch) in abs_str.chars().enumerate() {
            if i > 0 && (digits_len - i).is_multiple_of(3) {
                formatted.push(',');
            }
            formatted.push(ch);
        }

        let fg_c = ui.color(Token::Foreground);
        let muted_c = ui.color(Token::TextSecondary);

        ui.begin(Style { dir: Dir::Row, align: Align::Center, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);
        if !self.prefix.is_empty() {
            ui.text_leaf(self.prefix.clone(), st, muted_c, false, false, Style::default());
        }

        // Assign place-value slots right-to-left so ones, tens, hundreds stay stable as magnitude grows!
        let total_chars = formatted.chars().count();
        let mut place_idx = digits_len;
        for (ci, ch) in formatted.chars().enumerate() {
            let _ = total_chars;
            if let Some(d) = ch.to_digit(10) {
                place_idx = place_idx.saturating_sub(1);
                let slot = place_idx as u32;
                let target = d as f32;
                let pos = ui.anims.spring_from(id, slot, target, target, mo.smooth);
                let st_cl = st;
                ui.leaf(
                    Style::default(),
                    vec2(digit_w, line_h),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, r| {
                        cx.painter.push_clip(r);
                        let base_d = pos.floor() as i32;
                        let frac = pos - base_d as f32;
                        for off in -1..=2 {
                            let dv = (base_d + off).rem_euclid(10);
                            let y = r.min.y + (off as f32 - frac) * line_h;
                            if y + line_h > r.min.y - 1.0 && y < r.max.y + 1.0 {
                                let s = dv.to_string();
                                let g = cx.galley(&s, &st_cl);
                                let dist = (off as f32 - frac).abs().min(1.0);
                                cx.painter.text(cx.text, &g, vec2(r.min.x + (r.width() - g.width) * 0.5, y), fg_c.multiply_alpha(1.0 - 0.35 * dist));
                            }
                        }
                        cx.painter.pop_clip();
                    })),
                );
            } else {
                let _ = ci;
                ui.text_leaf(ch.to_string(), st, fg_c, false, false, Style::default());
            }
        }

        if !self.suffix.is_empty() {
            ui.space(4.0);
            let st_s = ui.text_style(TextRole::Body);
            ui.text_leaf(self.suffix.clone(), st_s, muted_c, false, false, Style::default());
        }
        ui.end();

        let rect = ui.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }
}

impl Drop for RollingNumber<'_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

// ==================================================================== Dock

/// Item in a floating neighbor-magnifying [`Ui::dock`].
#[derive(Clone, Debug)]
pub struct DockItem {
    pub icon: String,
    pub label: String,
    pub active: bool,
}

impl DockItem {
    pub fn new(icon: &str, label: impl Into<String>) -> Self {
        DockItem { icon: icon.to_owned(), label: label.into(), active: false }
    }
    pub fn active(mut self, on: bool) -> Self {
        self.active = on;
        self
    }
}

// ================================================================ Waveform

/// Scrubbable audio / voice waveform player with play/pause toggle and `mm:ss` readout.
pub struct Waveform<'u, 'a> {
    ui: &'u mut Ui,
    key: String,
    samples: &'a [f32],
    progress: &'u mut f32,
    playing: &'u mut bool,
    duration_secs: f32,
    title: Option<String>,
    shown: bool,
}

impl<'u, 'a> Waveform<'u, 'a> {
    pub fn duration(mut self, secs: f32) -> Self {
        self.duration_secs = secs.max(1.0);
        self
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("waveform", &self.key));
        let scrub_id = id.with("bars");
        let mo = ui.motion();

        *self.progress = self.progress.clamp(0.0, 1.0);
        let before = *self.progress;

        // Advance playback smoothly when playing!
        if *self.playing {
            // Real elapsed time, so playback speed is the same at any frame
            // rate; capped so a stalled frame doesn't skip ahead.
            let dt = ui.dt().min(0.1);
            *self.progress += dt / self.duration_secs;
            if *self.progress >= 1.0 {
                *self.progress = 0.0;
                *self.playing = false;
            } else {
                ui.request_repaint_after(0.016);
            }
        }

        let r_card = ui.look().card.radius;
        let card_bg = ui.color(Token::Surface);
        let card_bd = ui.color(Token::BorderSubtle);

        ui.begin(
            Style {
                dir: Dir::Column,
                width: Size::FILL,
                pad: Sides::xy(14.0, 12.0),
                gap: 8.0,
                ..Default::default()
            },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), card_bg, 1.0, card_bd))),
        );

        if let Some(ref title) = self.title {
            let st_t = ui.text_style(TextRole::BodyStrong);
            let fg_c = ui.color(Token::Foreground);
            ui.text_leaf(title.clone(), st_t, fg_c, false, true, Style { width: Size::FILL, ..Default::default() });
        }

        ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 12.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);

        // Circular Play / Pause button.
        let play_id = id.with("play");
        let pix = ui.interact(play_id, Sense::CLICK, false);
        let (pk_act, pk_held) = key_activate(ui, &pix, true);
        if pix.clicked || pk_act {
            *self.playing = !*self.playing;
        }
        let (ph, pp) = ui.state_weights(&pix, pk_held);
        let p_squash = ui.spring(play_id, 0, if pix.pressed || pk_held { 0.92 } else { 1.0 }, mo.bouncy);
        let (p_ra, p_rg) = ui.focus_anim(&pix);
        let p_fill = blend3(ui.color(Token::Accent), ui.color(Token::AccentHover), ui.color(Token::AccentPressed), ph, pp);
        let p_fg = ui.color(Token::AccentForeground);
        let ring_c = ui.color(Token::FocusRing);
        let is_playing = *self.playing;

        ui.leaf(
            Style::default(),
            vec2(36.0, 36.0),
            Some(play_id),
            Sense::CLICK,
            CursorIcon::Pointer,
            Some(Box::new(move |cx, r| {
                cx.painter.circle(r.center(), 18.0 * p_squash, p_fill, 0.0, Color::TRANSPARENT);
                cx.icon(if is_playing { "pause" } else { "play" }, r.center() + vec2(if is_playing { 0.0 } else { 1.0 }, 0.0), 15.0, p_fg);
                if p_ra > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, 18.0, p_ra, p_rg, ring_c)));
                }
            })),
        );

        // Scrubbable bar spectrum.
        let sense_scrub = Sense { click: true, drag: true, focusable: true, scroll: false };
        let six = ui.interact(scrub_id, sense_scrub, false);
        let (a_st, a_set) = ui.a11y_step(scrub_id);
        if a_st != 0 || a_set.is_some() {
            *self.progress = (a_set.map(|v| v as f32 / 100.0).unwrap_or(*self.progress) + a_st as f32 * 0.05).clamp(0.0, 1.0);
        }
        let prog = *self.progress as f64 * 100.0;
        ui.a11y(scrub_id, || A11y::new(Role::Slider).label("Position").numeric(prog, 0.0, 100.0, Some(5.0)));
        if (six.held || six.clicked) && six.rect.width() > 1.0 {
            if let Some(p) = six.pointer {
                *self.progress = ((p.x - six.rect.min.x) / six.rect.width()).clamp(0.0, 1.0);
            }
        }
        if six.focused {
            for (k, _, _) in ui.keys_for(scrub_id) {
                match k {
                    Key::ArrowRight => *self.progress = (*self.progress + 0.05).min(1.0),
                    Key::ArrowLeft => *self.progress = (*self.progress - 0.05).max(0.0),
                    Key::Space => *self.playing = !*self.playing,
                    _ => {}
                }
            }
        }

        let (s_ra, s_rg) = ui.focus_anim(&six);
        let prog = *self.progress;
        let enter = ui.anims.spring_from(id, 1, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);
        let played_c = ui.color(Token::Accent);
        let unplayed_c = ui.color(Token::Track);
        let samples_vec = self.samples.to_vec();

        ui.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(180.0, 38.0),
            Some(scrub_id),
            sense_scrub,
            CursorIcon::Pointer,
            Some(Box::new(move |cx, r| {
                let gap = 2.5;
                // Fit the bars to the width: when there are more samples than
                // 2 px bars fit, pool neighbours (peak) instead of overflowing.
                let fit = (((r.width() + gap) / (2.0 + gap)).floor() as usize).max(1);
                let bars: Vec<f32> = if samples_vec.len() <= fit {
                    samples_vec.clone()
                } else {
                    (0..fit)
                        .map(|b| {
                            let a = b * samples_vec.len() / fit;
                            let z = ((b + 1) * samples_vec.len() / fit).max(a + 1);
                            samples_vec[a..z].iter().copied().fold(0.0, f32::max)
                        })
                        .collect()
                };
                let nb = bars.len().max(1);
                let bw = ((r.width() - gap * (nb.saturating_sub(1)) as f32) / nb as f32).max(2.0);
                let play_x = r.min.x + r.width() * prog;

                for (i, &amp) in bars.iter().enumerate() {
                    let x = r.min.x + i as f32 * (bw + gap);
                    let bh = (amp.clamp(0.12, 1.0) * (r.height() - 4.0) * enter).max(4.0);
                    let br = Rect::from_xywh(x, r.center().y - bh * 0.5, bw, bh);
                    let rad = Corners::all((bw * 0.5).min(2.5));
                    if x + bw <= play_x {
                        cx.painter.rrect(br, rad, played_c, 0.0, Color::TRANSPARENT);
                    } else if x >= play_x {
                        cx.painter.rrect(br, rad, unplayed_c, 0.0, Color::TRANSPARENT);
                    } else {
                        cx.painter.rrect(br, rad, unplayed_c, 0.0, Color::TRANSPARENT);
                        cx.painter.push_clip(Rect::from_xywh(x, r.min.y, play_x - x, r.height()));
                        cx.painter.rrect(br, rad, played_c, 0.0, Color::TRANSPARENT);
                        cx.painter.pop_clip();
                    }
                }
                if s_ra > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, 6.0, s_ra, s_rg, ring_c)));
                }
            })),
        );

        // Tabular mm:ss readout.
        let cur_s = (prog * self.duration_secs).round() as u32;
        let tot_s = self.duration_secs.round() as u32;
        let fmt = |s: u32| format!("{}:{:02}", s / 60, s % 60);
        let mut st_tm = ui.text_style(TextRole::Mono);
        st_tm.tabular = true;
        let tm_c = ui.color(Token::TextSecondary);
        ui.text_leaf(format!("{} / {}", fmt(cur_s), fmt(tot_s)), st_tm, tm_c, false, false, Style::default());

        ui.end();
        ui.end();

        let changed = (*self.progress - before).abs() > 1e-5;
        let rect = ui.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, clicked: pix.clicked, changed, focused: pix.focused || six.focused, hovered: six.hovered }
    }
}

impl Drop for Waveform<'_, '_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

// =========================================================== WindowChrome

/// What a [`WindowChrome`] did this frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct ChromeResponse {
    /// A caption button was clicked.
    pub close: bool,
    pub minimize: bool,
    pub zoom: bool,
    /// The pointer went down on the title bar, outside the buttons.
    pub drag_started: bool,
    /// The title bar was double-clicked.
    pub double_clicked: bool,
    /// The chrome shows the maximized state (restore glyph).
    pub maximized: bool,
    /// Last frame's rects, in logical px (zero on the first frame or when a
    /// button isn't shown).
    pub title_bar: Rect,
    pub close_rect: Rect,
    pub minimize_rect: Rect,
    pub zoom_rect: Rect,
}

/// The body's result and the chrome's [`ChromeResponse`].
pub struct ChromeShown<R> {
    pub inner: R,
    pub response: ChromeResponse,
}

/// Native-feel window frame and title bar that follows the active
/// [`Look`](crate::Look)'s [`Platform`]: traffic lights on macOS, caption
/// buttons on Windows 11, round headerbar buttons on GNOME, pills on Lumen.
///
/// By default it draws a window *preview* (a framed card): its buttons
/// report clicks and zoom toggles the preview's maximized state. With
/// [`.window()`](Self::window) it is the real window's title bar: it fills
/// its parent, registers [`WindowRegion`]s so the host can move the window
/// and maximize it on double-click, and sends [`WindowRequest`]s for the
/// buttons (see `App::custom_chrome` in lumen-app).
pub struct WindowChrome<'u> {
    ui: &'u mut Ui,
    key: String,
    title: String,
    subtitle: Option<String>,
    window: bool,
    handle: bool,
    buttons: [bool; 3],
}

impl<'u> WindowChrome<'u> {
    pub fn subtitle(mut self, sub: impl Into<String>) -> Self {
        self.subtitle = Some(sub.into());
        self
    }
    /// This is the real window's title bar (see the type docs).
    pub fn window(mut self) -> Self {
        self.window = true;
        self
    }
    /// In window mode, send [`WindowRequest`]s for button clicks (default
    /// true). Turn it off to handle [`ChromeResponse`] yourself, for example
    /// to confirm before closing, then call [`Ui::request_window`].
    pub fn handle_buttons(mut self, on: bool) -> Self {
        self.handle = on;
        self
    }
    /// Show the minimize button (default true).
    pub fn minimizable(mut self, on: bool) -> Self {
        self.buttons[1] = on;
        self
    }
    /// Show the maximize/zoom button (default true).
    pub fn maximizable(mut self, on: bool) -> Self {
        self.buttons[2] = on;
        self
    }
    /// Show the close button (default true).
    pub fn closable(mut self, on: bool) -> Self {
        self.buttons[0] = on;
        self
    }

    pub fn show<R>(self, body: impl FnOnce(&mut Ui) -> R) -> ChromeShown<R> {
        let WindowChrome { ui, key, title, subtitle, window, handle, mut buttons } = self;
        let id = ui.make_id(("win_chrome", &key));
        let bar_id = id.with("bar");
        let platform = ui.theme.target_look().platform;
        // A real GNOME window shows the buttons the desktop is set to show.
        if let (true, Platform::Gnome, Some(b)) = (window, platform, ui.system().title_buttons) {
            for (on, os) in buttons.iter_mut().zip(b) {
                *on &= os;
            }
        }
        let mo = ui.motion();
        let r_win: f32 = if window {
            0.0
        } else {
            match platform {
                Platform::Windows => 8.0,
                Platform::Mac | Platform::Gnome => 12.0,
                Platform::Lumen => ui.look().card.radius - 2.0,
            }
        };
        let top_rad = (r_win - 1.0).max(0.0);
        let maximized = if window { ui.window_maximized() } else { *ui.mem::<bool>(id.with("max")) };
        let active = ui.fade(id.with("active"), 0, !window || ui.window_focused(), mo.hover_in, mo.hover_out, Ease::Out);

        let mut resp = ChromeResponse { maximized, ..Default::default() };
        let click = Sense { click: true, drag: false, focusable: false, scroll: false };
        let bar_ix = ui.interact(bar_id, click, false);
        resp.title_bar = bar_ix.rect;
        if bar_ix.press_started {
            resp.drag_started = true;
            resp.double_clicked = bar_ix.click_count == 2;
        }
        if window {
            ui.window_region(WindowRegion::Drag, bar_id);
        }

        let win_bg = ui.color(if window { Token::Background } else { Token::Surface });
        let win_bd = ui.color(Token::Border);
        let hdr_bg = ui.color(Token::SurfaceMuted).multiply_alpha(if window { 1.0 } else { 0.65 });
        let div_c = ui.color(Token::BorderSubtle);
        let shadows = Elevation::Level2.shadows(ui.theme.darkness(), ui.color(Token::ShadowInk));

        ui.begin(
            Style { width: Size::FILL, height: if window { Size::FILL } else { Size::Hug }, dir: Dir::Column, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                if window {
                    cx.painter.rect(r, win_bg);
                } else {
                    cx.painter.shadows(r, Corners::all(r_win), &shadows);
                    cx.painter.rrect(r, Corners::all(r_win), win_bg, 1.0, win_bd);
                }
            })),
        );

        // Title bar.
        let bar_h = match platform {
            Platform::Windows => 34.0,
            Platform::Mac => 36.0,
            Platform::Gnome | Platform::Lumen => 38.0,
        };
        let inset = if window { 0.0 } else { 1.0 };
        let pad_h = if platform == Platform::Windows { 12.0 } else { 14.0 };
        ui.begin(
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                height: Size::Fixed(bar_h),
                pad: Sides { left: pad_h, right: if platform == Platform::Windows { 0.0 } else { pad_h }, top: 0.0, bottom: 0.0 },
                gap: 10.0,
                align: Align::Center,
                ..Default::default()
            },
            Some(bar_id),
            click,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                let inner = Rect::from_xywh(r.min.x + inset, r.min.y + inset, (r.width() - 2.0 * inset).max(0.0), (r.height() - inset).max(0.0));
                cx.painter.rrect(inner, Corners([top_rad, top_rad, 0.0, 0.0]), hdr_bg, 0.0, Color::TRANSPARENT);
                cx.painter.rect(Rect::from_xywh(r.min.x + inset, r.max.y - 1.0, (r.width() - 2.0 * inset).max(0.0), 1.0), div_c);
            })),
        );

        // Buttons in platform order. `slot` keeps the title centered.
        let order: [WindowRegion; 3] = match platform {
            Platform::Mac => [WindowRegion::Close, WindowRegion::Minimize, WindowRegion::Maximize],
            _ => [WindowRegion::Minimize, WindowRegion::Maximize, WindowRegion::Close],
        };
        let shown: Vec<WindowRegion> = order.into_iter().filter(|k| buttons[btn_index(*k)]).collect();
        let n = shown.len() as f32;
        let (btn_w, btn_gap) = match platform {
            Platform::Mac => (12.0, 8.0),
            Platform::Windows => (46.0, 0.0),
            Platform::Gnome => (24.0, 10.0),
            Platform::Lumen => (22.0, 6.0),
        };
        let group_w = (n * btn_w + (n - 1.0).max(0.0) * btn_gap).max(0.0);
        let slot = match platform {
            Platform::Mac => group_w.max(10.0),
            Platform::Lumen => group_w.max(10.0),
            _ => group_w,
        };

        let fg_c = ui.color(Token::TextMuted).mix(ui.color(Token::Foreground), active);
        let sub_c = ui.color(Token::TextMuted);
        let st_t = ui.text_style(TextRole::BodyStrong);
        let sub = subtitle.map(|s| (s, ui.text_style(TextRole::Caption)));
        let title_block = |ui: &mut Ui, justify: Justify| {
            ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, align: Align::Center, justify, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            ui.text_leaf(title.clone(), st_t, fg_c, false, true, Style::default());
            if let Some((s, st)) = &sub {
                ui.text_leaf(s.clone(), *st, sub_c, false, true, Style::default());
            }
            ui.end();
        };

        let clicked = match platform {
            Platform::Windows => {
                let ic_c = ui.color(Token::AccentStrong);
                ui.icon_leaf("sparkles", 14.0, ic_c);
                title_block(ui, Justify::Start);
                caption_group(ui, id, &shown, platform, bar_h, btn_w, btn_gap, maximized, active, top_rad, window, &mut resp)
            }
            Platform::Mac => {
                ui.begin(Style { dir: Dir::Row, width: Size::Fixed(slot), gap: btn_gap, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                let c = caption_group(ui, id, &shown, platform, bar_h, btn_w, btn_gap, maximized, active, top_rad, window, &mut resp);
                ui.end();
                title_block(ui, Justify::Center);
                ui.leaf(Style::default(), vec2(slot, 1.0), None, Sense::NONE, CursorIcon::Default, None);
                c
            }
            Platform::Gnome | Platform::Lumen => {
                if platform == Platform::Lumen {
                    let dot = ui.color(Token::Accent).multiply_alpha(0.4 + 0.6 * active);
                    ui.begin(Style { dir: Dir::Row, width: Size::Fixed(slot), align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                    ui.leaf(Style::default(), vec2(10.0, 10.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.circle(r.center(), 4.5, dot, 0.0, Color::TRANSPARENT))));
                    ui.end();
                } else {
                    ui.leaf(Style::default(), vec2(slot, 1.0), None, Sense::NONE, CursorIcon::Default, None);
                }
                title_block(ui, Justify::Center);
                ui.begin(Style { dir: Dir::Row, width: Size::Fixed(slot), gap: btn_gap, align: Align::Center, justify: Justify::End, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                let c = caption_group(ui, id, &shown, platform, bar_h, btn_w, btn_gap, maximized, active, top_rad, window, &mut resp);
                ui.end();
                c
            }
        };
        ui.end(); // title bar

        match clicked {
            Some(WindowRegion::Close) => resp.close = true,
            Some(WindowRegion::Minimize) => resp.minimize = true,
            Some(WindowRegion::Maximize) => resp.zoom = true,
            _ => {}
        }
        if window && handle {
            if resp.close {
                ui.request_window(WindowRequest::Close);
            }
            if resp.minimize {
                ui.request_window(WindowRequest::Minimize);
            }
            if resp.zoom {
                ui.request_window(WindowRequest::ToggleMaximize);
            }
        }
        if !window && (resp.zoom || resp.double_clicked) {
            let m = ui.mem::<bool>(id.with("max"));
            *m = !*m;
            resp.maximized = *m;
        }

        // Body.
        let body_style = if window {
            Style { width: Size::FILL, height: Size::FILL, dir: Dir::Column, ..Default::default() }
        } else {
            Style { width: Size::FILL, dir: Dir::Column, pad: Sides::all(16.0), gap: 10.0, ..Default::default() }
        };
        ui.begin(body_style, None, Sense::NONE, CursorIcon::Default, false, None);
        let inner = ui.scope("win_body", body);
        ui.end();

        ui.end();
        ChromeShown { inner, response: resp }
    }
}

fn btn_index(k: WindowRegion) -> usize {
    match k {
        WindowRegion::Close => 0,
        WindowRegion::Minimize => 1,
        _ => 2,
    }
}

/// Draws one platform's caption buttons; returns the one clicked.
#[allow(clippy::too_many_arguments)]
fn caption_group(
    ui: &mut Ui,
    id: crate::Id,
    shown: &[WindowRegion],
    platform: Platform,
    bar_h: f32,
    btn_w: f32,
    _gap: f32,
    maximized: bool,
    active: f32,
    top_rad: f32,
    window: bool,
    resp: &mut ChromeResponse,
) -> Option<WindowRegion> {
    let mo = ui.motion();
    let click = Sense { click: true, drag: false, focusable: false, scroll: false };
    // macOS shows every glyph while the pointer is over any of the lights.
    let group_hover = shown.iter().any(|k| ui.interact(id.with(("cap", btn_index(*k))), click, false).hovered);
    let glyphs = ui.fade(id.with("lights"), 0, group_hover, mo.hover_in, mo.hover_out, Ease::Out);
    let reduced = ui.reduced_motion();
    let mut clicked = None;
    for (i, &kind) in shown.iter().enumerate() {
        let bid = id.with(("cap", btn_index(kind)));
        let ix = ui.interact(bid, click, false);
        if ix.clicked {
            clicked = Some(kind);
        }
        match kind {
            WindowRegion::Close => resp.close_rect = ix.rect,
            WindowRegion::Minimize => resp.minimize_rect = ix.rect,
            _ => resp.zoom_rect = ix.rect,
        }
        if window {
            ui.window_region(kind, bid);
        }
        let hov = ui.fade(bid, 0, ix.hovered, mo.hover_in, mo.hover_out, Ease::Out);
        let press = ui.fade(bid, 1, ix.pressed, 0.0, mo.hover_out, Ease::Out);
        let squash = if reduced { 1.0 } else { ui.spring(bid, 2, if ix.pressed { 0.9 } else { 1.0 }, mo.bouncy) };
        // Maximize glyph cross-fades to restore.
        let restore = ui.fade(bid, 3, maximized, mo.hover_in, mo.hover_out, Ease::Out);
        let last = i + 1 == shown.len();
        let ink = ui.color(Token::ShadowInk);
        let paint: crate::ui::PaintFn = match platform {
            Platform::Mac => {
                let base = ui.color(match kind {
                    WindowRegion::Close => Token::Danger,
                    WindowRegion::Minimize => Token::Warning,
                    _ => Token::Success,
                });
                let idle = ui.color(Token::BorderStrong);
                let fill = idle.mix(base, active.max(glyphs)).mix(ink, press * 0.25);
                let glyph = ink.with_alpha(0.62 * glyphs);
                Box::new(move |cx, r| {
                    let c = r.center();
                    let rad = 6.0 * squash;
                    cx.painter.circle(c, rad, fill, 0.5, fill.mix(ink, 0.18));
                    if glyph.a > 0.01 {
                        let k = 3.0 * squash;
                        match kind {
                            WindowRegion::Close => {
                                cx.painter.polyline(&[c + vec2(-k, -k), c + vec2(k, k)], 1.2, glyph);
                                cx.painter.polyline(&[c + vec2(k, -k), c + vec2(-k, k)], 1.2, glyph);
                            }
                            WindowRegion::Minimize => cx.painter.polyline(&[c + vec2(-k - 0.5, 0.0), c + vec2(k + 0.5, 0.0)], 1.3, glyph),
                            _ => {
                                cx.painter.polyline(&[c + vec2(-k - 0.5, 0.0), c + vec2(k + 0.5, 0.0)], 1.3, glyph);
                                cx.painter.polyline(&[c + vec2(0.0, -k - 0.5), c + vec2(0.0, k + 0.5)], 1.3, glyph.with_alpha(glyph.a * (1.0 - restore)));
                            }
                        }
                    }
                })
            }
            Platform::Windows => {
                let danger = kind == WindowRegion::Close;
                let hover_bg = if danger { ui.color(Token::Danger) } else { ui.color(Token::Hover) };
                let press_bg = if danger { ui.color(Token::Danger).mix(ink, 0.15) } else { ui.color(Token::Pressed) };
                let bg = hover_bg.mix(press_bg, press).multiply_alpha(hov.max(press));
                let fg = ui.color(Token::TextMuted).mix(ui.color(Token::Foreground), active);
                let stroke = if danger { fg.mix(ui.color(Token::DangerForeground), hov) } else { fg };
                let cr = if danger && last && !window { Corners([0.0, top_rad, 0.0, 0.0]) } else { Corners::all(0.0) };
                Box::new(move |cx, r| {
                    if bg.a > 0.0 {
                        cx.painter.rrect(r, cr, bg, 0.0, Color::TRANSPARENT);
                    }
                    let c = r.center();
                    let s = squash;
                    match kind {
                        WindowRegion::Minimize => cx.painter.rect(Rect::from_center_size(c, vec2(10.0 * s, 1.0)), stroke),
                        WindowRegion::Close => {
                            let k = 5.0 * s;
                            cx.painter.polyline(&[c + vec2(-k, -k), c + vec2(k, k)], 1.0, stroke);
                            cx.painter.polyline(&[c + vec2(k, -k), c + vec2(-k, k)], 1.0, stroke);
                        }
                        _ => {
                            let a = 1.0 - restore;
                            if a > 0.01 {
                                cx.painter.rrect(Rect::from_center_size(c, vec2(10.0 * s, 10.0 * s)), Corners::all(1.5), Color::TRANSPARENT, 1.0, stroke.with_alpha(stroke.a * a));
                            }
                            if restore > 0.01 {
                                let st = stroke.with_alpha(stroke.a * restore);
                                cx.painter.rrect(Rect::from_center_size(c + vec2(-1.5, 1.5), vec2(8.0 * s, 8.0 * s)), Corners::all(1.5), Color::TRANSPARENT, 1.0, st);
                                cx.painter.polyline(&[c + vec2(-1.0, -4.0 * s), c + vec2(4.0 * s, -4.0 * s), c + vec2(4.0 * s, 1.0)], 1.0, st);
                            }
                        }
                    }
                })
            }
            Platform::Gnome | Platform::Lumen => {
                let gnome = platform == Platform::Gnome;
                let danger = kind == WindowRegion::Close && !gnome;
                let rest = ui.color(if gnome { Token::Hover } else { Token::Control });
                let bd = if gnome { Color::TRANSPARENT } else { ui.color(Token::BorderSubtle) };
                let hover_bg = if danger { ui.color(Token::Danger) } else { ui.color(Token::Pressed) };
                let bg = rest.mix(hover_bg, hov).mix(ink, press * 0.12);
                let fg = ui.color(Token::TextMuted).mix(ui.color(Token::Foreground), active * (if gnome { 1.0 } else { 0.6 }));
                let fg = if danger { fg.mix(ui.color(Token::DangerForeground), hov) } else { fg };
                let (ic, ic2) = match kind {
                    WindowRegion::Close => ("x", "x"),
                    WindowRegion::Minimize => ("minus", "minus"),
                    _ => (if gnome { "square" } else { "maximize-2" }, if gnome { "copy" } else { "minimize-2" }),
                };
                let size = if gnome { 13.0 } else { 12.0 };
                Box::new(move |cx, r| {
                    let c = r.center();
                    cx.painter.circle(c, r.width() * 0.5 * squash, bg, if gnome { 0.0 } else { 1.0 }, bd);
                    if ic == ic2 {
                        cx.icon(ic, c, size * squash, fg);
                    } else {
                        cx.icon(ic, c, size * squash, fg.with_alpha(fg.a * (1.0 - restore)));
                        cx.icon(ic2, c, size * squash, fg.with_alpha(fg.a * restore));
                    }
                })
            }
        };
        let h = if platform == Platform::Windows { bar_h - if window { 0.0 } else { 2.0 } } else { btn_w };
        ui.leaf(Style::default(), vec2(btn_w, h), Some(bid), click, CursorIcon::Default, Some(paint));
    }
    clicked
}

impl Ui {
    /// Per-digit rolling odometer number.
    pub fn rolling_number(&mut self, key: impl Into<String>, value: i64) -> RollingNumber<'_> {
        RollingNumber {
            ui: self,
            key: key.into(),
            value,
            prefix: String::new(),
            suffix: String::new(),
            role: TextRole::Heading1,
            shown: false,
        }
    }

    /// Floating neighbor-magnifying icon dock; returns `Some(index)` when an item is clicked.
    pub fn dock(&mut self, key: impl std::hash::Hash, items: &[DockItem]) -> Option<usize> {
        let id = self.make_id(("dock", key));
        let mo = self.motion();
        let n = items.len();
        let base_sz = 42.0f32;
        let max_grow = 18.0f32;

        let ptr = self.pointer();
        let dock_rect = self.rect_of(id);
        let ptr_near = ptr.zip(dock_rect).is_some_and(|(p, dr)| dr.expand2(24.0, 32.0).contains(p));

        let mut clicked_idx: Option<usize> = None;
        let mut sizes = Vec::with_capacity(n);
        let mut item_ixs = Vec::with_capacity(n);

        for (i, it) in items.iter().enumerate() {
            let iid = id.with(("item", i));
            let ix = self.interact(iid, Sense::CLICK, false);
            let (k_act, k_held) = key_activate(self, &ix, true);
            if ix.clicked || k_act {
                clicked_idx = Some(i);
            }
            self.tooltip_for(
                iid,
                TooltipSpec { text: it.label.clone(), shortcut: None, placement: Placement::Top, gap: 6.0 },
            );

            // Gaussian magnification based on pointer X distance from the item's center!
            let target_sz = if self.reduced_motion() {
                base_sz
            } else if let (true, Some(p), Some(ir)) = (ptr_near, ptr, self.rect_of(iid)) {
                let dx = (p.x - ir.center().x).abs();
                let g = (-dx * dx / (2.0 * 48.0 * 48.0)).exp();
                base_sz + max_grow * g
            } else if ix.focused {
                base_sz + 8.0
            } else {
                base_sz
            };
            let sz = self.anims.spring_from(iid, 0, base_sz, target_sz, mo.bouncy).clamp(base_sz - 2.0, base_sz + max_grow + 6.0);
            let press = self.spring(iid, 1, if ix.pressed || k_held { 0.92 } else { 1.0 }, mo.snappy);
            sizes.push((sz, press));
            item_ixs.push((iid, ix));
        }

        let bar_bg = self.color(Token::SurfaceRaised).multiply_alpha(0.92);
        let bar_bd = self.color(Token::Border);
        let shadows = Elevation::Level2.shadows(self.theme.darkness(), self.color(Token::ShadowInk));

        self.begin(
            Style {
                dir: Dir::Row,
                pad: Sides { left: 12.0, right: 12.0, top: 8.0, bottom: 10.0 },
                gap: 8.0,
                align: Align::End,
                ..Default::default()
            },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                // Draw the dock shelf anchored to the bottom of the tallest magnified icon.
                let shelf_h = 62.0f32.min(r.height());
                let shelf = Rect::from_xywh(r.min.x, r.max.y - shelf_h, r.width(), shelf_h);
                cx.painter.shadows(shelf, Corners::all(18.0), &shadows);
                cx.painter.rrect(shelf, Corners::all(18.0), bar_bg, 1.0, bar_bd);
            })),
        );

        let palette = [
            Token::Accent,
            Token::Series1,
            Token::Series2,
            Token::Series3,
            Token::Series4,
            Token::Series5,
        ];

        for (i, it) in items.iter().enumerate() {
            let (iid, ref ix) = item_ixs[i];
            let (sz, press) = sizes[i];
            let (ring_a, ring_gap) = self.focus_anim(ix);

            let accent_c = self.color(palette[i % palette.len()]);
            let tile_bg = self.color(Token::Control).mix(accent_c.multiply_alpha(0.18), if it.active { 1.0 } else { 0.55 });
            let tile_bd = self.color(Token::BorderSubtle).mix(accent_c.multiply_alpha(0.45), if it.active { 0.8 } else { 0.25 });
            let ic_c = if it.active { accent_c } else { self.color(Token::Foreground) };
            let dot_c = self.color(Token::Accent);
            let ring_c = self.color(Token::FocusRing);
            let active = it.active;
            let ic_name = it.icon.clone();

            self.leaf(
                Style::default(),
                vec2(sz, sz + 6.0),
                Some(iid),
                Sense::CLICK,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    let tile_sz = sz * press;
                    let tr = Rect::from_center_size(vec2(r.center().x, r.max.y - 6.0 - sz * 0.5), vec2(tile_sz, tile_sz));
                    let rad = (tile_sz * 0.28).clamp(10.0, 16.0);
                    cx.painter.rrect(tr, Corners::all(rad), tile_bg, 1.0, tile_bd);
                    cx.icon(&ic_name, tr.center(), (tile_sz * 0.46).round(), ic_c);
                    if active {
                        cx.painter.circle(vec2(r.center().x, r.max.y - 1.5), 2.2, dot_c, 0.0, Color::TRANSPARENT);
                    }
                    if ring_a > 0.0 {
                        cx.defer(tr, Box::new(move |cx, r| cx.focus_ring(r, rad, ring_a, ring_gap, ring_c)));
                    }
                })),
            );
        }

        self.end();
        clicked_idx
    }

    /// Scrubbable audio / voice note waveform player.
    pub fn waveform<'u, 'a>(
        &'u mut self,
        key: impl Into<String>,
        samples: &'a [f32],
        progress: &'u mut f32,
        playing: &'u mut bool,
    ) -> Waveform<'u, 'a> {
        Waveform {
            ui: self,
            key: key.into(),
            samples,
            progress,
            playing,
            duration_secs: 42.0,
            title: None,
            shown: false,
        }
    }

    /// Platform-aware window frame and title bar (macOS traffic lights,
    /// Windows caption controls, GNOME headerbar or Lumen floating chrome).
    pub fn window_chrome(&mut self, key: impl Into<String>, title: impl Into<String>) -> WindowChrome<'_> {
        WindowChrome { ui: self, key: key.into(), title: title.into(), subtitle: None, window: false, handle: true, buttons: [true; 3] }
    }
}
