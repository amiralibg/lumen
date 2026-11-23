//! Feedback and chrome widgets: [`Alert`] (inline callouts and banners),
//! [`Skeleton`] shimmer placeholders, [`EmptyState`] panels, and [`Keycap`] /
//! [`Ui::kbd`] shortcut chips.

use lumen_text::TextAlign;

use super::toast::Tone;
use super::{key_activate, show_on_drop};
use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{ControlSize, TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

// ===================================================================== Alert

/// Inline callout or full-width banner in any [`Tone`], with an optional
/// action button and animated dismissal. Built with [`Ui::alert`].
pub struct Alert<'u> {
    ui: &'u mut Ui,
    title: String,
    description: Option<String>,
    tone: Tone,
    icon: Option<String>,
    action: Option<String>,
    open: Option<&'u mut bool>,
    banner: bool,
    shown: bool,
}

impl<'u> Alert<'u> {
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    pub fn tone(mut self, t: Tone) -> Self {
        self.tone = t;
        self
    }
    pub fn info(self) -> Self {
        self.tone(Tone::Info)
    }
    pub fn success(self) -> Self {
        self.tone(Tone::Success)
    }
    pub fn warning(self) -> Self {
        self.tone(Tone::Warning)
    }
    pub fn danger(self) -> Self {
        self.tone(Tone::Danger)
    }
    pub fn accent(self) -> Self {
        self.tone(Tone::Accent)
    }
    /// Override the tone's default Lucide icon.
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = Some(name.to_owned());
        self
    }
    /// Trailing action button label; [`Response::clicked`] is `true` when pressed.
    pub fn action(mut self, label: impl Into<String>) -> Self {
        self.action = Some(label.into());
        self
    }
    /// Show a close button bound to `open`; dismissing collapses the alert on a spring.
    pub fn dismissible(mut self, open: &'u mut bool) -> Self {
        self.open = Some(open);
        self
    }
    /// Compact horizontal banner style with a leading status accent bar.
    pub fn banner(mut self) -> Self {
        self.banner = true;
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("alert", &self.title));
        let inner_id = id.with("inner");
        let mo = ui.motion();
        let is_open = self.open.as_deref().copied().unwrap_or(true);

        let vis = ui.fade(id, 0, is_open, mo.layer_in * 1.2, mo.fade_out * 1.4, Ease::Out);
        let collapse = ui.anims.spring_from(id, 1, if is_open { 1.0 } else { 0.0 }, if is_open { 1.0 } else { 0.0 }, mo.snappy).clamp(0.0, 1.0);
        if !is_open && vis <= 0.001 && collapse <= 0.001 {
            return Response { id, ..Default::default() };
        }
        let enter = ui.anims.spring_from(id, 2, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);
        let alpha = (vis * enter).clamp(0.0, 1.0);

        let (subtle, strong, solid) = self.tone.tokens();
        let fill = if self.tone == Tone::Neutral {
            ui.color(Token::SurfaceMuted)
        } else {
            ui.color(Token::Surface).mix(ui.color(subtle), 0.58)
        }
        .multiply_alpha(alpha);
        let border = if self.tone == Tone::Neutral {
            ui.color(Token::Border)
        } else {
            ui.color(Token::Border).mix(ui.color(solid), 0.36)
        }
        .multiply_alpha(alpha);
        let bar_c = ui.color(solid).multiply_alpha(alpha);
        let icon_c = ui.color(strong).multiply_alpha(alpha);
        let title_c = ui.color(Token::Foreground).multiply_alpha(alpha);
        let desc_c = ui.color(Token::TextSecondary).multiply_alpha(alpha);
        let radius = (ui.look().card.radius * 0.75).max(6.0);
        let banner = self.banner;
        let icon_name = self.icon.clone().unwrap_or_else(|| self.tone.icon().to_owned());

        let measured_h = ui.rect_of(inner_id).map(|r| r.height());
        let collapsing = !is_open || collapse < 0.999;
        if collapsing {
            let h = measured_h.unwrap_or(56.0) * collapse;
            ui.begin(Style { width: Size::FILL, height: Size::Fixed(h), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, true, None);
        }

        let pad = if banner {
            Sides { left: 16.0, right: 12.0, top: 10.0, bottom: 10.0 }
        } else {
            Sides { left: 14.0, right: 12.0, top: 12.0, bottom: 12.0 }
        };
        ui.begin(
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                pad,
                gap: 12.0,
                align: if banner || self.description.is_none() { Align::Center } else { Align::Start },
                ..Default::default()
            },
            Some(inner_id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(radius), fill, 1.0, border);
                if banner {
                    let stripe = Rect::from_xywh(r.min.x, r.min.y, 3.5, r.height());
                    cx.painter.push_clip(stripe);
                    cx.painter.rrect(r, Corners::all(radius), bar_c, 0.0, Color::TRANSPARENT);
                    cx.painter.pop_clip();
                }
            })),
        );

        // Leading status icon.
        let top_nudge = if !banner && self.description.is_some() { 2.0 } else { 0.0 };
        ui.begin(Style { pad: Sides { top: top_nudge, ..Default::default() }, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        ui.icon_leaf(&icon_name, 16.0, icon_c);
        ui.end();

        // Title and description.
        if banner {
            ui.begin(Style { dir: Dir::Row, wrap: true, width: Size::FILL, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let st_t = ui.text_style(TextRole::BodyStrong);
            ui.text_leaf(self.title.clone(), st_t, title_c, false, false, Style::default());
            if let Some(d) = &self.description {
                let st_d = ui.text_style(TextRole::Body);
                ui.text_leaf(d.clone(), st_d, desc_c, true, false, Style::default());
            }
            ui.end();
        } else {
            ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 2.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let st_t = ui.text_style(TextRole::BodyStrong);
            ui.text_leaf(self.title.clone(), st_t, title_c, true, false, Style { width: Size::FILL, ..Default::default() });
            if let Some(d) = &self.description {
                let st_d = ui.text_style(TextRole::Body);
                ui.text_leaf(d.clone(), st_d, desc_c, true, false, Style { width: Size::FILL, ..Default::default() });
            }
            ui.end();
        }

        let mut action_clicked = false;
        if let Some(act_label) = &self.action {
            let aid = id.with("action");
            let aix = ui.interact(aid, Sense::CLICK, !is_open);
            let (k_act, k_held) = key_activate(ui, &aix, true);
            if (aix.clicked || k_act) && is_open {
                action_clicked = true;
            }
            let h = ui.fade(aid, 1, aix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
            let press = ui.spring(aid, 2, if aix.pressed || k_held { 1.0 } else { 0.0 }, mo.snappy);
            let (ring_a, ring_gap) = ui.focus_anim(&aix);
            let m = ui.control_metrics(ControlSize::Sm);
            let mut st = ui.text_style(TextRole::Label);
            st.weight = 500;
            let g = ui.galley(act_label, &st);
            let bg = ui.color(Token::SurfaceRaised).mix(ui.color(Token::SurfaceRaised).over(ui.color(Token::Hover)), h).multiply_alpha(alpha);
            let bd = ui.color(Token::BorderStrong).multiply_alpha(alpha);
            let fg = ui.color(Token::Foreground).multiply_alpha(alpha);
            let ring = ui.color(Token::FocusRing);
            let rad = m.radius;
            let bw = g.width + 20.0;
            ui.leaf(
                Style::default(),
                vec2(bw, m.height),
                Some(aid),
                Sense::CLICK,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    let sr = Rect::from_center_size(r.center(), r.size() * (1.0 - 0.04 * press));
                    cx.painter.rrect_unsnapped(sr, Corners::all(rad), [bg; 2], 1.0, bd);
                    cx.painter.text(cx.text, &g, vec2((sr.center().x - g.width / 2.0).round(), (sr.center().y - g.height / 2.0).round()), fg);
                    if ring_a > 0.0 {
                        cx.defer(sr, Box::new(move |cx, r| cx.focus_ring(r, rad, ring_a, ring_gap, ring)));
                    }
                })),
            );
        }

        let mut dismissed = false;
        if self.open.is_some() {
            let cid = id.with("close");
            let btn_s = if is_open { 1.0 } else { alpha };
            if ui.field_icon_button(cid, "x", btn_s, is_open) && is_open {
                dismissed = true;
            }
        }
        ui.end();

        if collapsing {
            ui.end();
        }
        if dismissed {
            if let Some(open) = &mut self.open {
                **open = false;
            }
        }

        let rect = ui.rect_of(inner_id).unwrap_or_default();
        Response { id, rect, hovered: false, clicked: action_clicked, changed: dismissed, focused: false }
    }
}

show_on_drop!(Alert);

// ================================================================== Skeleton

/// Placeholder shimmer block while content loads. Respects reduced motion by
/// rendering a calm static surface with zero frame wakeups.
pub struct Skeleton<'u> {
    ui: &'u mut Ui,
    width: Size,
    pref_w: f32,
    height: f32,
    radius: Option<f32>,
    circle: bool,
    shown: bool,
}

impl<'u> Skeleton<'u> {
    pub fn fill_width(mut self) -> Self {
        self.width = Size::FILL;
        self
    }
    pub fn radius(mut self, r: f32) -> Self {
        self.radius = Some(r.max(0.0));
        self
    }
    pub fn circle(mut self) -> Self {
        self.circle = true;
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id("skeleton");
        let base = ui.color(Token::SurfaceMuted).mix(ui.color(Token::Track), 0.35);
        let sheen = ui.color(Token::Surface).mix(ui.color(Token::SurfaceMuted), 0.5).over(ui.color(Token::Hover));
        let reduced = ui.reduced_motion();
        let t = ui.time() as f32;
        if !reduced {
            ui.request_repaint_after(0.0);
        }
        let rad = if self.circle {
            self.height / 2.0
        } else {
            self.radius.unwrap_or_else(|| (self.height * 0.35).clamp(4.0, ui.look().control.radius[1]))
        };
        let screen_w = ui.screen_size().x.max(400.0);
        let h = self.height;
        let w_style = self.width;
        let pref_w = self.pref_w;

        ui.leaf(
            Style { width: w_style, height: Size::Fixed(h), ..Default::default() },
            vec2(pref_w, h),
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                if reduced {
                    cx.painter.rrect(r, Corners::all(rad), base, 0.0, Color::TRANSPARENT);
                    return;
                }
                let phase = (t * 0.65).fract();
                let smooth = phase * phase * (3.0 - 2.0 * phase);
                let wave_x = -180.0 + smooth * (screen_w + 360.0);
                let g = |x: f32| {
                    let d = (x - wave_x) / 150.0;
                    (-d * d).exp()
                };
                let c_left = base.mix(sheen, g(r.min.x));
                let c_right = base.mix(sheen, g(r.max.x));
                cx.painter.rrect_horiz(r, Corners::all(rad), c_left, c_right);
            })),
        );
        Response { id, rect: ui.rect_of(id).unwrap_or_default(), ..Default::default() }
    }
}

show_on_drop!(Skeleton);

// ================================================================ EmptyState

/// Centered empty-view placeholder with a 20px icon tile (`DESIGN.md` §3.3),
/// title, wrapped description, and optional action slot.
pub struct EmptyState<'u> {
    ui: &'u mut Ui,
    icon: String,
    title: String,
    description: Option<String>,
    bordered: bool,
    compact: bool,
    shown: bool,
}

impl<'u> EmptyState<'u> {
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    /// Draw a subtle surface border around the empty state (default `true`).
    pub fn bordered(mut self, on: bool) -> Self {
        self.bordered = on;
        self
    }
    /// Tighter padding for embedded panels and popovers.
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }
    /// Render the empty state with interactive action controls underneath.
    pub fn action<R>(mut self, f: impl FnOnce(&mut Ui) -> R) -> R {
        self.shown = true;
        let mut f = Some(f);
        let mut out = None;
        self.render(Some(&mut |ui| {
            if let Some(cb) = f.take() {
                out = Some(cb(ui));
            }
        }));
        out.expect("action called")
    }
    pub fn show(mut self) {
        self.shown = true;
        self.render(None);
    }

    fn render(&mut self, act: Option<&mut dyn FnMut(&mut Ui)>) {
        let ui = &mut *self.ui;
        let id = ui.make_id(("empty_state", &self.title));
        let mo = ui.motion();
        let enter = ui.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);

        let pad = if self.compact { Sides::xy(20.0, 22.0) } else { Sides::xy(28.0, 32.0) };
        let radius = ui.look().card.radius;
        let bordered = self.bordered;
        let fill = if bordered { ui.color(Token::Surface).multiply_alpha(enter) } else { Color::TRANSPARENT };
        let bd = if bordered { ui.color(Token::BorderSubtle).multiply_alpha(enter) } else { Color::TRANSPARENT };

        ui.begin(
            Style { dir: Dir::Column, width: Size::FILL, pad, gap: 12.0, align: Align::Center, justify: Justify::Center, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                if bordered {
                    cx.painter.rrect(r, Corners::all(radius), fill, 1.0, bd);
                }
            })),
        );

        let tile_bg = ui.color(Token::SurfaceMuted).multiply_alpha(enter);
        let tile_bd = ui.color(Token::Border).multiply_alpha(enter);
        let ic_col = ui.color(Token::TextSecondary).multiply_alpha(enter);
        let ic_name = self.icon.clone();
        let tile_sz = if self.compact { 40.0 } else { 44.0 };
        let tile_r = (ui.look().control.radius[2] + 2.0).min(14.0);
        ui.leaf(
            Style::default(),
            vec2(tile_sz, tile_sz),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let s = 0.88 + 0.12 * enter;
                let tr = Rect::from_center_size(r.center(), r.size() * s);
                cx.painter.rrect_unsnapped(tr, Corners::all(tile_r), [tile_bg; 2], 1.0, tile_bd);
                cx.icon(&ic_name, tr.center(), 20.0 * s, ic_col);
            })),
        );

        ui.begin(Style { dir: Dir::Column, width: Size::FILL, max: vec2(320.0, f32::INFINITY), gap: 4.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let st_t = ui.text_style(TextRole::Heading4);
        let tc = ui.color(Token::Foreground).multiply_alpha(enter);
        let nt = ui.text_leaf(self.title.clone(), st_t, tc, true, false, Style { width: Size::FILL, ..Default::default() });
        ui.set_text_align(nt, TextAlign::Center);
        if let Some(d) = &self.description {
            let st_d = ui.text_style(TextRole::Body);
            let dc = ui.color(Token::TextSecondary).multiply_alpha(enter);
            let nd = ui.text_leaf(d.clone(), st_d, dc, true, false, Style { width: Size::FILL, ..Default::default() });
            ui.set_text_align(nd, TextAlign::Center);
        }
        ui.end();

        if let Some(f) = act {
            ui.begin(Style { dir: Dir::Row, gap: 8.0, pad: Sides { top: 4.0, ..Default::default() }, align: Align::Center, justify: Justify::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            f(ui);
            ui.end();
        }
        ui.end();
    }
}

impl Drop for EmptyState<'_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.render(None);
        }
    }
}

// =================================================================== Keycap

/// Physical keyboard keycap (`radius_xs` = 4px, Geist Mono, sculpted bottom
/// edge). Supports live keyboard press depression via [`.live_key`](Keycap::live_key).
pub struct Keycap<'u> {
    ui: &'u mut Ui,
    label: String,
    large: bool,
    pressed: bool,
    live: Option<Key>,
    interactive: bool,
    shown: bool,
}

impl<'u> Keycap<'u> {
    /// Taller keycap for shortcut callouts or interactive keypads.
    pub fn large(mut self) -> Self {
        self.large = true;
        self
    }
    /// Force the depressed state.
    pub fn pressed(mut self, on: bool) -> Self {
        self.pressed = on;
        self
    }
    /// Automatically depress while `key` is held on the keyboard.
    pub fn live_key(mut self, key: Key) -> Self {
        self.live = Some(key);
        self
    }
    /// Allow clicking or focusing this keycap like a button.
    pub fn interactive(mut self) -> Self {
        self.interactive = true;
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("keycap", &self.label));
        let sense = if self.interactive { Sense::CLICK } else { Sense::NONE };
        let ix = ui.interact(id, sense, false);
        let (k_act, k_held) = if self.interactive { key_activate(ui, &ix, true) } else { (false, false) };
        let key_live = self.live.is_some_and(|k| ui.key_down(k));
        let down = self.pressed || key_live || ix.pressed || k_held;
        let mo = ui.motion();
        let dep = ui.spring(id, 0, if down { 1.0 } else { 0.0 }, mo.snappy).clamp(0.0, 1.0);
        let h = ui.fade(id, 1, ix.hovered && self.interactive, mo.hover_in, mo.hover_out, Ease::Linear);
        let (ring_a, ring_gap) = ui.focus_anim(&ix);

        let mut mono = ui.text_style(TextRole::Mono);
        let scale = ui.look().text_scale;
        mono.size = (if self.large { 13.0 } else { 11.0 } * scale).round();
        mono.weight = 500;
        mono.line_height = (if self.large { 18.0 } else { 16.0 } * scale).round();
        let g = ui.galley(&self.label, &mono);

        let h_px = if self.large { 32.0 } else { 22.0 };
        let min_w = if self.large { 32.0 } else { 22.0 };
        let pad_x = if self.large { 9.0 } else { 6.0 };
        let w_px = (g.width.ceil() + pad_x * 2.0).max(min_w);
        let rad = if self.large { 6.0 } else { 4.0 } * (ui.look().control.radius[1] / 8.0).clamp(0.6, 1.4);

        let c = |t| ui.color(t);
        let top_face = c(Token::SurfaceRaised)
            .mix(c(Token::SurfaceRaised).over(c(Token::Hover)), h)
            .mix(c(Token::AccentSubtle), dep * 0.85);
        let skirt = c(Token::BorderStrong).mix(c(Token::Accent), dep * 0.7);
        let border = c(Token::Border).mix(c(Token::Accent).with_alpha(0.65), dep);
        let fg = c(Token::TextSecondary).mix(c(Token::Foreground), h).mix(c(Token::AccentStrong), dep);
        let ring = c(Token::FocusRing);
        let skirt_h = if self.large { 2.5 } else { 1.75 };

        ui.leaf(
            Style { min: vec2(w_px, h_px), ..Default::default() },
            vec2(w_px, h_px),
            Some(id),
            sense,
            if self.interactive { CursorIcon::Pointer } else { CursorIcon::Default },
            Some(Box::new(move |cx, r| {
                let travel = skirt_h * dep;
                // Physical key base / skirt.
                let base_r = Rect::from_xywh(r.min.x, r.min.y + travel, r.width(), r.height() - travel);
                cx.painter.rrect_unsnapped(base_r, Corners::all(rad), [skirt; 2], 0.0, Color::TRANSPARENT);
                // Sculpted top face sitting above the skirt.
                let face_h = (r.height() - skirt_h).max(12.0);
                let face_r = Rect::from_xywh(r.min.x, r.min.y + travel, r.width(), face_h);
                cx.painter.rrect_unsnapped(face_r, Corners::all(rad), [top_face; 2], 1.0, border);
                cx.painter.text(cx.text, &g, vec2((face_r.center().x - g.width / 2.0).round(), (face_r.center().y - g.height / 2.0).round()), fg);
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, rad, ring_a, ring_gap, ring)));
                }
            })),
        );

        let clicked = (ix.clicked || k_act) && self.interactive;
        Response { id, rect: ix.rect, hovered: ix.hovered, clicked, changed: clicked, focused: ix.focused }
    }
}

show_on_drop!(Keycap);

impl Ui {
    /// Inline callout or banner with a tone, optional description, action and
    /// animated dismiss button.
    pub fn alert(&mut self, title: impl Into<String>) -> Alert<'_> {
        Alert { ui: self, title: title.into(), description: None, tone: Tone::Info, icon: None, action: None, open: None, banner: false, shown: false }
    }

    /// Shimmering placeholder block of `width × height` logical pixels.
    pub fn skeleton(&mut self, width: f32, height: f32) -> Skeleton<'_> {
        Skeleton { ui: self, width: Size::Fixed(width), pref_w: width, height, radius: None, circle: false, shown: false }
    }

    /// Circular avatar placeholder shimmer of `size` diameter.
    pub fn skeleton_circle(&mut self, size: f32) -> Response {
        self.skeleton(size, size).circle().show()
    }

    /// Multi-line paragraph skeleton whose final line tapers naturally.
    pub fn skeleton_lines(&mut self, lines: usize) {
        let n = lines.max(1);
        self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 8.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for i in 0..n {
            if i + 1 == n && n > 1 {
                self.begin(Style { dir: Dir::Row, width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                self.scope(("sk-line", i), |ui| {
                    let mut sk = ui.skeleton(120.0, 12.0).radius(4.0);
                    sk.width = Size::Fill(0.62);
                    sk.show();
                });
                self.leaf(Style { width: Size::Fill(0.38), ..Default::default() }, vec2(0.0, 12.0), None, Sense::NONE, CursorIcon::Default, None);
                self.end();
            } else {
                self.scope(("sk-line", i), |ui| {
                    ui.skeleton(160.0, 12.0).fill_width().radius(4.0).show();
                });
            }
        }
        self.end();
    }

    /// Centered empty-view panel with a 20px icon tile, title, description and
    /// optional `.action(|ui| ..)` slot.
    pub fn empty_state(&mut self, icon: &str, title: impl Into<String>) -> EmptyState<'_> {
        EmptyState { ui: self, icon: icon.to_owned(), title: title.into(), description: None, bordered: true, compact: false, shown: false }
    }

    /// Inline keyboard keycap (`ui.kbd("⌘K")`).
    pub fn kbd(&mut self, label: &str) -> Response {
        self.keycap(label).show()
    }

    /// Configurable keyboard keycap builder (`.large()`, `.live_key(..)`, `.interactive()`).
    pub fn keycap(&mut self, label: impl Into<String>) -> Keycap<'_> {
        Keycap { ui: self, label: label.into(), large: false, pressed: false, live: None, interactive: false, shown: false }
    }

    /// Sequence of keycaps for a multi-key chord (e.g. `&["⌘", "Shift", "P"]`).
    pub fn shortcut_keys(&mut self, keys: &[&str]) {
        self.begin(Style { dir: Dir::Row, gap: 4.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for (i, k) in keys.iter().enumerate() {
            self.scope(("kbd", i), |ui| {
                ui.kbd(k);
            });
        }
        self.end();
    }
}
