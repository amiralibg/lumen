//! Color picker: a lumen-drawn 2D saturation–value field, hue and optional
//! alpha sliders, preset swatches, and a live hex editor in a popover.

use std::f32::consts::PI;

use crate::id::Id;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides, Vec2};
use crate::theme::{Elevation, TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

use super::calendar::Lead;
use super::field::{Chrome, EditOpts};

const DEFAULT_SWATCHES: [u32; 8] = [0x0E6EE6, 0x7D55EC, 0x02844B, 0x0090A8, 0xF5AF20, 0xF77C10, 0xDD2729, 0xDB1E69];

#[derive(Clone)]
struct ColorMem {
    h: f32,
    s: f32,
    v: f32,
    last_bytes: [u8; 4],
    hex: String,
    copied_until: f64,
    init: bool,
}

impl Default for ColorMem {
    fn default() -> Self {
        ColorMem { h: 210.0, s: 0.8, v: 0.9, last_bytes: [0; 4], hex: String::new(), copied_until: -10.0, init: false }
    }
}

/// Builder for [`Ui::color_picker`].
pub struct ColorPicker<'u, 's> {
    ui: &'u mut Ui,
    label: String,
    color: &'u mut Color,
    alpha: bool,
    swatches: Option<&'s [Color]>,
    width: Size,
    disabled: bool,
    shown: bool,
}

impl<'u, 's> ColorPicker<'u, 's> {
    /// Include an opacity slider and 8-digit `#RRGGBBAA` hex editing.
    pub fn alpha(mut self, on: bool) -> Self {
        self.alpha = on;
        self
    }

    /// Custom preset swatches shown under the sliders.
    pub fn swatches(mut self, swatches: &'s [Color]) -> Self {
        self.swatches = Some(swatches);
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = Size::Fixed(w);
        self
    }

    pub fn fill_width(mut self) -> Self {
        self.width = Size::FILL;
        self
    }

    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }

    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }

    pub fn changed(self) -> bool {
        self.show().changed
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("color_picker", &self.label));
        let sv_id = id.with("panel").with("sv");
        let text = format_hex(*self.color, self.alpha);
        let r = ui.picker_field_sized(id, &self.label, Lead::Swatch(*self.color), &text, false, Some(sv_id), self.width, self.disabled);
        let mut changed = false;
        let alpha = self.alpha;
        let swatches = self.swatches;
        let color = &mut *self.color;
        ui.popover(&r).pad(12.0).show(|ui| {
            ui.scope(id, |ui| {
                changed |= ui.color_panel_impl(id.with("panel"), color, alpha, swatches);
            });
        });
        Response { changed, ..r }
    }
}

impl Drop for ColorPicker<'_, '_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

impl Ui {
    /// A color field with a swatch and hex readout that opens a 2D color
    /// canvas, hue slider, swatches and hex input in a popover.
    pub fn color_picker<'u>(&'u mut self, label: impl Into<String>, color: &'u mut Color) -> ColorPicker<'u, 'static> {
        ColorPicker { ui: self, label: label.into(), color, alpha: false, swatches: None, width: Size::Hug, disabled: false, shown: false }
    }

    /// Inline color editor (used inside [`color_picker`](Self::color_picker)
    /// and usable directly in cards or sheets).
    pub fn color_editor(&mut self, key: &str, color: &mut Color, alpha: bool) -> Response {
        let id = self.make_id(("color_editor", key));
        let changed = self.color_panel_impl(id, color, alpha, None);
        Response { id, rect: self.rect_of(id).unwrap_or_default(), changed, ..Default::default() }
    }

    fn color_panel_impl(&mut self, id: Id, color: &mut Color, alpha: bool, custom_swatches: Option<&[Color]>) -> bool {
        let before = color.to_u8();
        let hex_id = id.with("hex");
        let editing_hex = self.focused() == Some(hex_id);
        {
            let m = self.mem::<ColorMem>(id);
            if !m.init || m.last_bytes != before {
                let (h, s, v) = rgb_to_hsv(*color);
                if !m.init || s > 0.002 {
                    m.h = h;
                }
                if !m.init || v > 0.002 {
                    m.s = s;
                }
                m.v = v;
                m.last_bytes = before;
                if !editing_hex {
                    m.hex = format_hex_digits(*color, alpha);
                }
                m.init = true;
            }
        }

        let mut mem = self.mem::<ColorMem>(id).clone();
        let mut a_val = color.a.clamp(0.0, 1.0);
        let mut hsv_dirty = false;
        let mo = self.motion();
        let panel_w = 232.0;

        self.begin(Style { width: Size::Fixed(panel_w), gap: 10.0, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);

        // 1. 2D Saturation–Value field.
        let sv_id = id.with("sv");
        let sv_ix = self.interact(sv_id, Sense::DRAG, false);
        if sv_ix.press_started || sv_ix.held {
            if let Some(p) = sv_ix.pointer {
                let r = sv_ix.rect;
                if r.width() > 1.0 && r.height() > 1.0 {
                    mem.s = ((p.x - r.min.x) / r.width()).clamp(0.0, 1.0);
                    mem.v = (1.0 - (p.y - r.min.y) / r.height()).clamp(0.0, 1.0);
                    hsv_dirty = true;
                }
            }
        }
        if sv_ix.focused {
            for (k, m, _) in self.keys_for(sv_id) {
                let d = if m.shift { 0.10 } else { 0.02 };
                match k {
                    Key::ArrowLeft => mem.s = (mem.s - d).clamp(0.0, 1.0),
                    Key::ArrowRight => mem.s = (mem.s + d).clamp(0.0, 1.0),
                    Key::ArrowUp => mem.v = (mem.v + d).clamp(0.0, 1.0),
                    Key::ArrowDown => mem.v = (mem.v - d).clamp(0.0, 1.0),
                    _ => continue,
                }
                hsv_dirty = true;
            }
        }
        let sv_hot = sv_ix.hovered || sv_ix.held;
        let sv_knob = self.spring(sv_id, 0, if sv_hot { 16.0 } else { 14.0 }, if sv_hot { mo.bouncy } else { mo.snappy });
        let (sv_ra, sv_rg) = self.focus_anim(&sv_ix);
        let hue = mem.h;
        let (sat, val) = (mem.s, mem.v);
        let solid_rgb = hsv_to_rgb(hue, sat, val, 1.0);
        let sv_rad = (self.look().card.radius * 0.55).clamp(6.0, 10.0);
        let edge = self.color(Token::BorderStrong).multiply_alpha(0.55);
        let knob_c = self.color(Token::Knob);
        let shadows = Elevation::Knob.shadows(self.theme.darkness(), self.color(Token::ShadowInk));
        let ring = self.color(Token::FocusRing);

        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(panel_w, 140.0),
            Some(sv_id),
            Sense::DRAG,
            CursorIcon::Pointer,
            Some(Box::new(move |cx, r| {
                let r = cx.painter.snap_rect(r);
                paint_sv_mesh(&mut cx.painter, r, sv_rad, hue);
                cx.painter.ring(r, Corners::all(sv_rad), 1.0, edge);
                let p = vec2(r.min.x + sat * r.width(), r.min.y + (1.0 - val) * r.height());
                let kr = Rect::from_center_size(p, vec2(sv_knob, sv_knob));
                cx.painter.shadows(kr, Corners::all(sv_knob / 2.0), &shadows);
                cx.painter.circle(p, sv_knob / 2.0, knob_c, 0.0, Color::TRANSPARENT);
                cx.painter.circle(p, (sv_knob / 2.0 - 2.5).max(2.0), solid_rgb, 0.0, Color::TRANSPARENT);
                if sv_ra > 0.0 {
                    cx.defer(kr, Box::new(move |cx, r| cx.focus_ring(r, r.width() / 2.0, sv_ra, sv_rg, ring)));
                }
            })),
        );

        // 2. Hue slider.
        let hue_id = id.with("hue");
        let hue_ix = self.interact(hue_id, Sense::DRAG, false);
        if hue_ix.press_started || hue_ix.held {
            if let Some(p) = hue_ix.pointer {
                let (x0, x1) = (hue_ix.rect.min.x + 7.0, hue_ix.rect.max.x - 7.0);
                if x1 > x0 {
                    mem.h = ((p.x - x0) / (x1 - x0)).clamp(0.0, 1.0) * 360.0;
                    hsv_dirty = true;
                }
            }
        }
        if hue_ix.focused {
            for (k, m, _) in self.keys_for(hue_id) {
                let d = if m.shift { 15.0 } else { 3.0 };
                match k {
                    Key::ArrowLeft | Key::ArrowDown => mem.h = (mem.h - d).rem_euclid(360.0),
                    Key::ArrowRight | Key::ArrowUp => mem.h = (mem.h + d).rem_euclid(360.0),
                    Key::Home => mem.h = 0.0,
                    Key::End => mem.h = 359.0,
                    _ => continue,
                }
                hsv_dirty = true;
            }
        }
        let h_hot = hue_ix.hovered || hue_ix.held;
        let h_knob = self.spring(hue_id, 0, if h_hot { 16.0 } else { 14.0 }, if h_hot { mo.bouncy } else { mo.snappy });
        let (h_ra, h_rg) = self.focus_anim(&hue_ix);
        let h_norm = (mem.h / 360.0).clamp(0.0, 1.0);
        let pure_hue = hsv_to_rgb(mem.h, 1.0, 1.0, 1.0);
        let shadows_h = Elevation::Knob.shadows(self.theme.darkness(), self.color(Token::ShadowInk));
        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(panel_w, 16.0),
            Some(hue_id),
            Sense::DRAG,
            CursorIcon::Pointer,
            Some(Box::new(move |cx, r| {
                let bar = cx.painter.snap_rect(Rect::from_xywh(r.min.x, r.center().y - 6.0, r.width(), 12.0));
                paint_hue_mesh(&mut cx.painter, bar, 6.0);
                cx.painter.ring(bar, Corners::all(6.0), 1.0, edge);
                let x = bar.min.x + 6.0 + h_norm * (bar.width() - 12.0);
                let c = vec2(x, bar.center().y);
                let kr = Rect::from_center_size(c, vec2(h_knob, h_knob));
                cx.painter.shadows(kr, Corners::all(h_knob / 2.0), &shadows_h);
                cx.painter.circle(c, h_knob / 2.0, knob_c, 0.0, Color::TRANSPARENT);
                cx.painter.circle(c, (h_knob / 2.0 - 2.5).max(2.0), pure_hue, 0.0, Color::TRANSPARENT);
                if h_ra > 0.0 {
                    cx.defer(kr, Box::new(move |cx, r| cx.focus_ring(r, r.width() / 2.0, h_ra, h_rg, ring)));
                }
            })),
        );

        // 3. Optional Alpha slider.
        if alpha {
            let a_id = id.with("alpha");
            let a_ix = self.interact(a_id, Sense::DRAG, false);
            if a_ix.press_started || a_ix.held {
                if let Some(p) = a_ix.pointer {
                    let (x0, x1) = (a_ix.rect.min.x + 7.0, a_ix.rect.max.x - 7.0);
                    if x1 > x0 {
                        a_val = ((p.x - x0) / (x1 - x0)).clamp(0.0, 1.0);
                        hsv_dirty = true;
                    }
                }
            }
            if a_ix.focused {
                for (k, m, _) in self.keys_for(a_id) {
                    let d = if m.shift { 0.10 } else { 0.02 };
                    match k {
                        Key::ArrowLeft | Key::ArrowDown => a_val = (a_val - d).clamp(0.0, 1.0),
                        Key::ArrowRight | Key::ArrowUp => a_val = (a_val + d).clamp(0.0, 1.0),
                        Key::Home => a_val = 0.0,
                        Key::End => a_val = 1.0,
                        _ => continue,
                    }
                    hsv_dirty = true;
                }
            }
            let a_hot = a_ix.hovered || a_ix.held;
            let a_knob = self.spring(a_id, 0, if a_hot { 16.0 } else { 14.0 }, if a_hot { mo.bouncy } else { mo.snappy });
            let (a_ra, a_rg) = self.focus_anim(&a_ix);
            let rgb_opaque = hsv_to_rgb(mem.h, mem.s, mem.v, 1.0);
            let shadows_a = Elevation::Knob.shadows(self.theme.darkness(), self.color(Token::ShadowInk));
            let chk_a = self.color(Token::Surface);
            let chk_b = self.color(Token::Track);
            self.leaf(
                Style { width: Size::FILL, ..Default::default() },
                vec2(panel_w, 16.0),
                Some(a_id),
                Sense::DRAG,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    let bar = cx.painter.snap_rect(Rect::from_xywh(r.min.x, r.center().y - 6.0, r.width(), 12.0));
                    cx.painter.rrect(bar, Corners::all(6.0), chk_a, 0.0, Color::TRANSPARENT);
                    paint_alpha_mesh(&mut cx.painter, bar, 6.0, chk_a, chk_b, rgb_opaque);
                    cx.painter.ring(bar, Corners::all(6.0), 1.0, edge);
                    let x = bar.min.x + 6.0 + a_val * (bar.width() - 12.0);
                    let c = vec2(x, bar.center().y);
                    let kr = Rect::from_center_size(c, vec2(a_knob, a_knob));
                    cx.painter.shadows(kr, Corners::all(a_knob / 2.0), &shadows_a);
                    cx.painter.circle(c, a_knob / 2.0, knob_c, 0.0, Color::TRANSPARENT);
                    cx.painter.circle(c, (a_knob / 2.0 - 2.5).max(2.0), chk_a.over(rgb_opaque.with_alpha(a_val)), 0.0, Color::TRANSPARENT);
                    if a_ra > 0.0 {
                        cx.defer(kr, Box::new(move |cx, r| cx.focus_ring(r, r.width() / 2.0, a_ra, a_rg, ring)));
                    }
                })),
            );
        }

        if hsv_dirty {
            *color = hsv_to_rgb(mem.h, mem.s, mem.v, a_val);
            mem.hex = format_hex_digits(*color, alpha);
        }

        // 4. Preset swatches.
        let default_cols: [Color; 8] = DEFAULT_SWATCHES.map(Color::hex);
        let swatches = custom_swatches.unwrap_or(&default_cols);
        if !swatches.is_empty() {
            let sw_ids: Vec<Id> = (0..swatches.len()).map(|i| id.with(("sw", i))).collect();
            self.begin(Style { dir: Dir::Row, width: Size::FILL, justify: Justify::SpaceBetween, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            for (i, &sw) in swatches.iter().take(8).enumerate() {
                let sid = sw_ids[i];
                let six = self.interact(sid, Sense::CLICK, false);
                if six.focused {
                    self.arrow_focus(sid, &sw_ids);
                }
                let (act, held) = super::key_activate(self, &six, true);
                if six.clicked || act {
                    let keep_a = if alpha && sw.a >= 0.999 { color.a } else { sw.a };
                    *color = sw.with_alpha(keep_a);
                    let (h, s, v) = rgb_to_hsv(*color);
                    mem.h = h;
                    mem.s = s;
                    mem.v = v;
                    mem.hex = format_hex_digits(*color, alpha);
                }
                let selected = color.to_hex() == sw.to_hex();
                let pressed = six.pressed || held;
                let sz = self.spring(sid, 0, if pressed { 18.0 } else if six.hovered { 22.0 } else { 20.0 }, if six.hovered && !pressed { mo.bouncy } else { mo.snappy });
                let tick = self.spring(sid, 1, if selected { 1.0 } else { 0.0 }, mo.bouncy).max(0.0);
                let (s_ra, s_rg) = self.focus_anim(&six);
                let check_c = if sw.luminance() > 0.42 { Color::hex(0x11131C) } else { Color::WHITE };
                self.leaf(
                    Style::default(),
                    vec2(26.0, 26.0),
                    Some(sid),
                    Sense::CLICK,
                    CursorIcon::Pointer,
                    Some(Box::new(move |cx, r| {
                        let c = r.center();
                        cx.painter.circle(c, sz / 2.0, sw, 1.0, edge);
                        if tick > 0.01 {
                            cx.icon("check", c, 12.0 * tick, check_c.multiply_alpha(tick.min(1.0)));
                        }
                        if s_ra > 0.0 {
                            let rr = Rect::from_center_size(c, vec2(20.0, 20.0));
                            cx.defer(rr, Box::new(move |cx, r| cx.focus_ring(r, 10.0, s_ra, s_rg, ring)));
                        }
                    })),
                );
            }
            self.end();
        }

        // 5. Hex input + copy button.
        let frame_id = id.with("hex-frame");
        let hovered = self.pointer().is_some_and(|p| self.rect_of(frame_id).is_some_and(|r| r.contains(p)));
        let ch = (self.look().control.height[0] + 2.0).max(30.0);
        self.field_begin(
            frame_id,
            Chrome { hovered, focused: editing_hex, error: false, disabled: false },
            Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(ch), pad: Sides { left: 10.0, right: 4.0, top: 0.0, bottom: 0.0 }, gap: 4.0, align: Align::Center, ..Default::default() },
        );
        let hash_st = self.text_style(TextRole::Mono);
        let muted = self.color(Token::TextMuted);
        self.text_leaf("#", hash_st, muted, false, false, Style::default());
        let mut o = EditOpts::new(self);
        o.style = self.text_style(TextRole::Mono);
        o.style.line_height = (18.0 * self.look().text_scale).round();
        o.accept = Some(|c| c.is_ascii_hexdigit());
        o.max_chars = Some(if alpha { 8 } else { 6 });
        let ed = self.inline_edit(hex_id, &mut mem.hex, o);
        mem.hex = mem.hex.to_ascii_uppercase();
        if ed.changed {
            if let Some(parsed) = parse_hex_digits(&mem.hex, alpha, color.a) {
                *color = parsed;
                let (h, s, v) = rgb_to_hsv(*color);
                if s > 0.002 {
                    mem.h = h;
                }
                if v > 0.002 {
                    mem.s = s;
                }
                mem.v = v;
            }
        }
        if ed.key(Key::Enter) {
            mem.hex = format_hex_digits(*color, alpha);
            self.close_layer();
        }
        let now = self.time();
        let copied = now < mem.copied_until;
        if copied {
            self.repaint_after_secs(mem.copied_until - now);
        }
        let icon = if copied { "check" } else { "copy" };
        if self.field_icon_button(id.with("copy"), icon, 1.0, true) {
            self.set_copied_text(format_hex(*color, alpha));
            mem.copied_until = now + 1.2;
        }
        self.end();
        if self.clicked_id(frame_id) {
            self.request_focus(hex_id);
        }
        if !editing_hex && self.focused() != Some(hex_id) {
            mem.hex = format_hex_digits(*color, alpha);
        }

        self.end();

        let after = color.to_u8();
        mem.last_bytes = after;
        *self.mem::<ColorMem>(id) = mem;
        after != before
    }
}

fn format_hex_digits(c: Color, alpha: bool) -> String {
    let [r, g, b, a] = c.to_u8();
    if alpha && a < 255 {
        format!("{r:02X}{g:02X}{b:02X}{a:02X}")
    } else {
        format!("{r:02X}{g:02X}{b:02X}")
    }
}

fn format_hex(c: Color, alpha: bool) -> String {
    format!("#{}", format_hex_digits(c, alpha))
}

fn parse_hex_digits(s: &str, alpha: bool, fallback_a: f32) -> Option<Color> {
    match s.len() {
        6 => {
            let v = u32::from_str_radix(s, 16).ok()?;
            Some(Color::hex(v).with_alpha(fallback_a))
        }
        8 if alpha => {
            let v = u32::from_str_radix(s, 16).ok()?;
            let a = (v & 0xff) as f32 / 255.0;
            Some(Color::hex(v >> 8).with_alpha(a))
        }
        _ => None,
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32, a: f32) -> Color {
    let h = h.rem_euclid(360.0) / 60.0;
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let c = v * s;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Color::rgba(r + m, g + m, b + m, a)
}

fn rgb_to_hsv(c: Color) -> (f32, f32, f32) {
    let (r, g, b) = (c.r.clamp(0.0, 1.0), c.g.clamp(0.0, 1.0), c.b.clamp(0.0, 1.0));
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d < 1e-5 {
        0.0
    } else if (max - r).abs() < 1e-5 {
        60.0 * (((g - b) / d).rem_euclid(6.0))
    } else if (max - g).abs() < 1e-5 {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    let s = if max < 1e-5 { 0.0 } else { d / max };
    (h, s, max)
}

/// Project a point inside `r` onto the rounded rectangle of radius `rad` so
/// mesh corners never poke outside an SDF border ring.
fn clamp_to_rrect(p: Vec2, r: Rect, rad: f32) -> Vec2 {
    let c = r.center();
    let half = r.size() * 0.5;
    let rad = rad.min(half.x).min(half.y);
    let inner = vec2((half.x - rad).max(0.0), (half.y - rad).max(0.0));
    let d = p - c;
    let q = vec2(d.x.abs(), d.y.abs()) - inner;
    if q.x > 0.0 && q.y > 0.0 {
        let l = q.length();
        if l > rad && l > 1e-5 {
            let k = inner + q * (rad / l);
            return c + vec2(k.x.copysign(d.x), k.y.copysign(d.y));
        }
    }
    p
}

fn paint_sv_mesh(painter: &mut crate::paint::Painter, r: Rect, rad: f32, hue: f32) {
    const NX: usize = 20;
    const NY: usize = 14;
    let mut verts = Vec::with_capacity((NX + 1) * (NY + 1));
    for iy in 0..=NY {
        let ty = 0.5 * (1.0 - (PI * iy as f32 / NY as f32).cos());
        for ix in 0..=NX {
            let tx = 0.5 * (1.0 - (PI * ix as f32 / NX as f32).cos());
            let p = clamp_to_rrect(vec2(r.min.x + tx * r.width(), r.min.y + ty * r.height()), r, rad);
            let col = hsv_to_rgb(hue, tx, 1.0 - ty, 1.0);
            verts.push((p, col));
        }
    }
    let mut indices = Vec::with_capacity(NX * NY * 6);
    let stride = (NX + 1) as u32;
    for iy in 0..NY as u32 {
        for ix in 0..NX as u32 {
            let a = iy * stride + ix;
            let b = a + 1;
            let c = a + stride + 1;
            let d = a + stride;
            indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }
    painter.mesh(&verts, &indices);
}

fn paint_hue_mesh(painter: &mut crate::paint::Painter, r: Rect, rad: f32) {
    const NX: usize = 36;
    const NY: usize = 6;
    let mut verts = Vec::with_capacity((NX + 1) * (NY + 1));
    for iy in 0..=NY {
        let ty = iy as f32 / NY as f32;
        for ix in 0..=NX {
            let tx = ix as f32 / NX as f32;
            let p = clamp_to_rrect(vec2(r.min.x + tx * r.width(), r.min.y + ty * r.height()), r, rad);
            let col = hsv_to_rgb(tx * 360.0, 1.0, 1.0, 1.0);
            verts.push((p, col));
        }
    }
    let mut indices = Vec::with_capacity(NX * NY * 6);
    let stride = (NX + 1) as u32;
    for iy in 0..NY as u32 {
        for ix in 0..NX as u32 {
            let a = iy * stride + ix;
            let b = a + 1;
            let c = a + stride + 1;
            let d = a + stride;
            indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }
    painter.mesh(&verts, &indices);
}

fn paint_alpha_mesh(painter: &mut crate::paint::Painter, r: Rect, rad: f32, chk_a: Color, chk_b: Color, rgb: Color) {
    // Subtle checkerboard squares along the center, then clamped alpha gradient.
    let cell = 6.0;
    let cols = (r.width() / cell).ceil() as usize;
    painter.push_clip(r.expand(-1.0));
    for i in 0..cols {
        for j in 0..2 {
            if (i + j) % 2 == 1 {
                let x0 = (r.min.x + i as f32 * cell).max(r.min.x + 2.0);
                let x1 = (x0 + cell).min(r.max.x - 2.0);
                if x1 > x0 {
                    let y0 = r.min.y + j as f32 * cell;
                    painter.rect(Rect::from_xywh(x0, y0, x1 - x0, cell), chk_b);
                }
            }
        }
    }
    painter.pop_clip();
    let _ = chk_a;

    const NX: usize = 20;
    const NY: usize = 6;
    let mut verts = Vec::with_capacity((NX + 1) * (NY + 1));
    for iy in 0..=NY {
        let ty = iy as f32 / NY as f32;
        for ix in 0..=NX {
            let tx = 0.5 * (1.0 - (PI * ix as f32 / NX as f32).cos());
            let p = clamp_to_rrect(vec2(r.min.x + tx * r.width(), r.min.y + ty * r.height()), r, rad);
            verts.push((p, rgb.with_alpha(tx)));
        }
    }
    let mut indices = Vec::with_capacity(NX * NY * 6);
    let stride = (NX + 1) as u32;
    for iy in 0..NY as u32 {
        for ix in 0..NX as u32 {
            let a = iy * stride + ix;
            let b = a + 1;
            let c = a + stride + 1;
            let d = a + stride;
            indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }
    painter.mesh(&verts, &indices);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_roundtrip() {
        for hex in DEFAULT_SWATCHES {
            let c = Color::hex(hex);
            let (h, s, v) = rgb_to_hsv(c);
            let back = hsv_to_rgb(h, s, v, 1.0);
            assert_eq!(back.to_hex(), hex);
        }
    }
}
