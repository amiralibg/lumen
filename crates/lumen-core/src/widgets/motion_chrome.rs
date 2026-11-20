//! Motion set & native window-chrome widgets:
//! - [`RollingNumber`] (per-digit vertical odometer reel that springs smoothly across digits)
//! - [`Ui::dock`] (Gaussian neighbor-magnifying floating icon dock with bouncy springs)
//! - [`Waveform`] (scrubbable audio / voice waveform player with play/pause and mm:ss readout)
//! - [`WindowChrome`] (platform-aware window header bar that morphs across Lumen, macOS, Windows and GNOME)

use crate::anim::Ease;
use crate::color::Color;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::look::Platform;
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{Elevation, TextRole, Token};
use crate::ui::{Placement, Response, Sense, TooltipSpec, Ui};

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

/// Native-feel window frame + title bar that adapts its traffic lights or
/// caption controls automatically to the active [`Look`]'s [`Platform`].
pub struct WindowChrome<'u> {
    ui: &'u mut Ui,
    key: String,
    title: String,
    subtitle: Option<String>,
}

impl<'u> WindowChrome<'u> {
    pub fn subtitle(mut self, sub: impl Into<String>) -> Self {
        self.subtitle = Some(sub.into());
        self
    }

    pub fn show<R>(self, body: impl FnOnce(&mut Ui) -> R) -> R {
        let WindowChrome { ui, key, title, subtitle } = self;
        let id = ui.make_id(("win_chrome", &key));
        let platform = ui.theme.target_look().platform;
        let r_win: f32 = match platform {
            Platform::Windows => 8.0,
            Platform::Mac | Platform::Gnome => 12.0,
            Platform::Lumen => 14.0,
        };
        let top_rad = (r_win - 1.0).max(0.0);

        let win_bg = ui.color(Token::Surface);
        let win_bd = ui.color(Token::Border);
        let hdr_bg = ui.color(Token::SurfaceMuted).multiply_alpha(0.65);
        let div_c = ui.color(Token::BorderSubtle);
        let shadows = Elevation::Level2.shadows(ui.theme.darkness(), ui.color(Token::ShadowInk));

        ui.begin(
            Style { width: Size::FILL, dir: Dir::Column, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                cx.painter.shadows(r, Corners::all(r_win), &shadows);
                cx.painter.rrect(r, Corners::all(r_win), win_bg, 1.0, win_bd);
            })),
        );

        // Titlebar row.
        let bar_h = match platform {
            Platform::Windows => 34.0,
            Platform::Mac => 36.0,
            Platform::Gnome | Platform::Lumen => 38.0,
        };
        let pad_h = if platform == Platform::Windows { 12.0 } else { 14.0 };
        ui.begin(
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                height: Size::Fixed(bar_h),
                pad: Sides { left: pad_h, right: if platform == Platform::Windows { 0.0 } else { pad_h }, top: 0.0, bottom: 0.0 },
                gap: 10.0,
                align: Align::Center,
                justify: Justify::SpaceBetween,
                ..Default::default()
            },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                let inner = Rect::from_xywh(r.min.x + 1.0, r.min.y + 1.0, (r.width() - 2.0).max(0.0), (r.height() - 1.0).max(0.0));
                cx.painter.rrect(inner, Corners([top_rad, top_rad, 0.0, 0.0]), hdr_bg, 0.0, Color::TRANSPARENT);
                cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.max.y - 1.0, (r.width() - 2.0).max(0.0), 1.0), div_c);
            })),
        );

        // Left slot.
        ui.begin(Style { dir: Dir::Row, width: Size::Fixed(72.0), gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        match platform {
            Platform::Mac => {
                let c_close = ui.color(Token::Danger);
                let c_min = ui.color(Token::Warning);
                let c_max = ui.color(Token::Success);
                for col in [c_close, c_min, c_max] {
                    ui.leaf(
                        Style::default(),
                        vec2(12.0, 12.0),
                        None,
                        Sense::NONE,
                        CursorIcon::Default,
                        Some(Box::new(move |cx, r| cx.painter.circle(r.center(), 6.0, col, 0.5, col.multiply_alpha(0.6)))),
                    );
                }
            }
            Platform::Windows => {
                let ic_c = ui.color(Token::AccentStrong);
                ui.icon_leaf("sparkles", 14.0, ic_c);
            }
            Platform::Lumen => {
                let dot = ui.color(Token::Accent);
                ui.leaf(
                    Style::default(),
                    vec2(10.0, 10.0),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, r| cx.painter.circle(r.center(), 4.5, dot, 0.0, Color::TRANSPARENT))),
                );
            }
            Platform::Gnome => {}
        }
        ui.end();

        // Center title + optional subtitle.
        let fg_c = ui.color(Token::Foreground);
        let sub_c = ui.color(Token::TextMuted);
        ui.begin(Style { dir: Dir::Row, gap: 6.0, align: Align::Center, justify: Justify::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let st_t = ui.text_style(TextRole::BodyStrong);
        ui.text_leaf(title, st_t, fg_c, false, true, Style::default());
        if let Some(sub) = subtitle {
            let st_s = ui.text_style(TextRole::Caption);
            ui.text_leaf(sub, st_s, sub_c, false, false, Style::default());
        }
        ui.end();

        // Right caption buttons slot.
        ui.begin(
            Style { dir: Dir::Row, width: Size::Fixed(if platform == Platform::Windows { 126.0 } else { 72.0 }), gap: if platform == Platform::Windows { 0.0 } else { 6.0 }, align: Align::Center, justify: Justify::End, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );
        match platform {
            Platform::Windows => {
                let fg = ui.color(Token::TextSecondary);
                for kind in [0u8, 1, 2] {
                    let bid = id.with(("wcap", kind));
                    let bix = ui.interact(bid, Sense { click: true, drag: false, focusable: false, scroll: false }, false);
                    let hov = ui.fade(bid, 0, bix.hovered, 0.06, 0.12, Ease::Linear);
                    let bg = if kind == 2 {
                        ui.color(Token::Danger).multiply_alpha(hov)
                    } else {
                        ui.color(Token::Hover).multiply_alpha(hov)
                    };
                    let stroke = if kind == 2 && hov > 0.4 { ui.color(Token::DangerForeground) } else { fg };
                    let cr = if kind == 2 { Corners([0.0, top_rad, 0.0, 0.0]) } else { Corners::all(0.0) };
                    ui.leaf(
                        Style::default(),
                        vec2(42.0, bar_h - 2.0),
                        Some(bid),
                        Sense { click: true, drag: false, focusable: false, scroll: false },
                        CursorIcon::Pointer,
                        Some(Box::new(move |cx, r| {
                            if bg.a > 0.0 {
                                cx.painter.rrect(r, cr, bg, 0.0, Color::TRANSPARENT);
                            }
                            let c = r.center();
                            match kind {
                                0 => cx.painter.rect(Rect::from_center_size(c, vec2(10.0, 1.2)), stroke),
                                1 => cx.painter.rrect(Rect::from_center_size(c, vec2(9.5, 9.5)), Corners::all(1.0), Color::TRANSPARENT, 1.2, stroke),
                                _ => {
                                    cx.painter.polyline(&[c + vec2(-4.5, -4.5), c + vec2(4.5, 4.5)], 1.3, stroke);
                                    cx.painter.polyline(&[c + vec2(4.5, -4.5), c + vec2(-4.5, 4.5)], 1.3, stroke);
                                }
                            }
                        })),
                    );
                }
            }
            Platform::Gnome => {
                let pill_bg = ui.color(Token::Control);
                let fg = ui.color(Token::TextSecondary);
                ui.leaf(
                    Style::default(),
                    vec2(24.0, 24.0),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, r| {
                        cx.painter.circle(r.center(), 12.0, pill_bg, 0.0, Color::TRANSPARENT);
                        cx.icon("x", r.center(), 13.0, fg);
                    })),
                );
            }
            Platform::Lumen => {
                let pill_bg = ui.color(Token::Control);
                let pill_bd = ui.color(Token::BorderSubtle);
                let fg = ui.color(Token::TextMuted);
                for ic in ["minus", "x"] {
                    let ic_s = ic.to_owned();
                    ui.leaf(
                        Style::default(),
                        vec2(22.0, 22.0),
                        None,
                        Sense::NONE,
                        CursorIcon::Default,
                        Some(Box::new(move |cx, r| {
                            cx.painter.circle(r.center(), 11.0, pill_bg, 1.0, pill_bd);
                            cx.icon(&ic_s, r.center(), 12.0, fg);
                        })),
                    );
                }
            }
            Platform::Mac => {}
        }
        ui.end();
        ui.end(); // titlebar

        // Window content body.
        ui.begin(Style { width: Size::FILL, dir: Dir::Column, pad: Sides::all(16.0), gap: 10.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let out = ui.scope("win_body", body);
        ui.end();

        ui.end();
        out
    }
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
        WindowChrome { ui: self, key: key.into(), title: title.into(), subtitle: None }
    }
}
