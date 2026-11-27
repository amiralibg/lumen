//! Slider (spec section 6).

use std::ops::RangeInclusive;

use lumen_text::TextAlign;

use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect};
use crate::theme::{Elevation, TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

use super::show_on_drop;

#[derive(Default)]
struct DragState {
    /// Pointer moved since press: track 1:1.
    tracking: bool,
}

pub struct Slider<'u> {
    ui: &'u mut Ui,
    label: String,
    value: &'u mut f32,
    range: RangeInclusive<f32>,
    step: Option<f32>,
    readout: Option<String>,
    width: Option<f32>,
    disabled: bool,
    shown: bool,
}

impl<'u> Slider<'u> {
    /// Snap to multiples of `step` from the range start.
    pub fn step(mut self, s: f32) -> Self {
        self.step = Some(s);
        self
    }
    /// Text shown to the right (format it yourself: `"64%"`, `"13 px"`).
    pub fn readout(mut self, s: impl Into<String>) -> Self {
        self.readout = Some(s.into());
        self
    }
    /// Show the value with up to two decimals.
    pub fn show_value(mut self) -> Self {
        let v = *self.value;
        self.readout = Some(if v.fract() == 0.0 { format!("{v:.0}") } else { format!("{v:.2}") });
        self
    }
    /// Track width; `f32::INFINITY` fills the container.
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w.max(120.0));
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }

    fn show_inner(&mut self) -> Response {
        let range = self.range.clone();
        let step = self.step;
        let quantize = move |v: f32| quantize(&range, step, v);
        self.show_impl(quantize)
    }

    fn show_impl(&mut self, quantize: impl Fn(f32) -> f32) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("slider", &self.label));
        let ix = ui.interact(id, Sense::DRAG, self.disabled);
        let (lo, hi) = (*self.range.start(), *self.range.end());
        let span = (hi - lo).abs().max(f32::EPSILON);
        let old = *self.value;
        let lk = ui.look().slider;
        let mo = ui.motion();
        let half = lk.thumb / 2.0;
        let rail = |r: Rect| (r.min.x + half, r.max.x - half);
        let mut new = old;
        let mut jump = false;
        if !self.disabled {
            let r = ix.rect;
            let (x0, x1) = rail(r);
            let from_x = |x: f32| lo + ((x - x0) / (x1 - x0).max(1.0)).clamp(0.0, 1.0) * (hi - lo);
            if ix.press_started {
                if let Some(p) = ix.pointer {
                    new = from_x(p.x);
                    // Click away from the thumb animates there.
                    let thumb_x = x0 + ((old - lo) / span) * (x1 - x0);
                    jump = (p.x - thumb_x).abs() > half;
                    ui.mem::<DragState>(id).tracking = !jump;
                }
            } else if ix.held {
                if let Some(p) = ix.pointer {
                    let v = from_x(p.x);
                    if (quantize(v) - quantize(old)).abs() > 0.0 || ui.mem::<DragState>(id).tracking {
                        ui.mem::<DragState>(id).tracking = true;
                    }
                    new = v;
                }
            }
            if ix.focused {
                let step = self.step.unwrap_or(span / 100.0);
                for (k, m, _) in ui.keys_for(id) {
                    let n = if m.shift { 10.0 } else { 1.0 };
                    match k {
                        Key::ArrowLeft | Key::ArrowDown => new -= step * n,
                        Key::ArrowRight | Key::ArrowUp => new += step * n,
                        Key::Home => new = lo,
                        Key::End => new = hi,
                        _ => continue,
                    }
                    jump = true;
                }
            }
            let (st, set) = ui.a11y_step(id);
            if st != 0 || set.is_some() {
                let step = self.step.unwrap_or(span / 100.0);
                new = set.map(|v| v as f32).unwrap_or(new) + st as f32 * step;
                new = new.clamp(lo.min(hi), hi.max(lo));
                jump = true;
            }
        }
        let new = quantize(new);
        let changed = new != old;
        *self.value = new;
        let disabled = self.disabled;
        let a_step = self.step.map(|s| s as f64);
        ui.a11y(id, || A11y::new(Role::Slider).label(self.label.as_str()).numeric(new as f64, lo as f64, hi as f64, a_step).disabled(disabled));
        let norm = ((new - lo) / span).clamp(0.0, 1.0);
        let tracking = ix.held && ui.mem::<DragState>(id).tracking && !jump;
        let pos = if tracking && self.step.is_none() {
            ui.anims.set_spring(id, 0, norm);
            norm
        } else {
            ui.spring(id, 0, norm, mo.snappy)
        };
        let hot = (ix.hovered || ix.held) && !self.disabled;
        let size = ui.spring(id, 1, if hot { lk.thumb + lk.hover_grow } else { lk.thumb }, if hot { mo.bouncy } else { mo.snappy });
        // Windows: the accent dot inside the thumb grows on hover and
        // shrinks while dragging.
        let dot_target = if ix.held && !self.disabled { 0.5 } else if hot { 0.7 } else { 0.6 };
        let dot = ui.spring(id, 3, dot_target, if hot { mo.bouncy } else { mo.snappy });
        let halo = ui.fade(id, 2, ix.held && !self.disabled, 0.06, 0.16, Ease::Linear) * lk.halo;
        let (ring_a, ring_gap) = ui.focus_anim(&ix);
        let c = |t| ui.color(t);
        let (rail_c, fill_c, knob, knob_border, readout_c) = if self.disabled {
            (c(Token::Border), c(Token::TextDisabled), c(Token::SurfaceMuted), c(Token::BorderStrong), c(Token::TextDisabled))
        } else {
            let rc = if ix.held { c(Token::Foreground) } else { c(Token::TextSecondary) };
            let kb = c(Token::BorderStrong).multiply_alpha(0.6 * lk.thumb_border);
            let knob = c(Token::Knob).mix(c(Token::Control), lk.inner_dot * ui.theme.darkness());
            (c(Token::Track), c(Token::Accent), knob, kb, rc)
        };
        let dot_c = if self.disabled { c(Token::TextDisabled) } else { c(Token::Accent).mix(c(Token::AccentHover), if hot { 1.0 } else { 0.0 }) };
        let inner_dot = lk.inner_dot;
        let track_h = lk.track;
        let shadows = if self.disabled { vec![] } else { Elevation::Knob.shadows(ui.theme.darkness(), c(Token::ShadowInk)) };
        let halo_c = c(Token::FocusHalo);
        let ring = c(Token::FocusRing);
        let width = self.width.unwrap_or(240.0);
        let fill = !width.is_finite();
        let fw = if fill { Size::FILL } else { Size::Hug };

        let col = ui.begin(Style { gap: 6.0, width: fw, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let _ = col;
        if !self.label.is_empty() {
            let lc = if self.disabled { ui.color(Token::TextDisabled) } else { ui.color(Token::Foreground) };
            let st = ui.text_style(TextRole::Label);
            let n = ui.text_leaf(self.label.clone(), st, lc, false, true, Style::default());
            ui.a11y_hide(n);
        }
        ui.begin(Style { dir: Dir::Row, gap: 12.0, width: fw, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let cursor = if self.disabled { CursorIcon::Default } else { CursorIcon::Pointer };
        ui.leaf(
            Style { width: if fill { Size::FILL } else { Size::Fixed(width) }, ..Default::default() },
            vec2(if fill { 120.0 } else { width }, (lk.thumb + 4.0).max(20.0)),
            Some(id),
            Sense::DRAG,
            cursor,
            Some(Box::new(move |cx, r| {
                let tr = track_h / 2.0;
                let rail_r = cx.painter.snap_rect(Rect::from_xywh(r.min.x, r.center().y - tr, r.width(), track_h));
                cx.painter.rrect(rail_r, Corners::all(tr), rail_c, 0.0, Color::TRANSPARENT);
                let (x0, x1) = (r.min.x + half, r.max.x - half);
                let cxp = x0 + pos * (x1 - x0);
                let fill = Rect { min: rail_r.min, max: vec2(cxp, rail_r.max.y) };
                cx.painter.rrect_unsnapped(fill, Corners([tr, 0.0, 0.0, tr]), [fill_c; 2], 0.0, Color::TRANSPARENT);
                let center = vec2(cxp, r.center().y);
                if halo > 0.0 {
                    cx.painter.circle(center, half + 5.0, halo_c.multiply_alpha(halo), 0.0, Color::TRANSPARENT);
                }
                let tr = Rect::from_center_size(center, vec2(size, size));
                cx.painter.shadows(tr, Corners::all(size / 2.0), &shadows);
                let bw = if knob_border.a > 0.0 { 1.0 } else { 0.0 };
                cx.painter.circle(center, size / 2.0, knob, bw, knob_border);
                if inner_dot > 0.0 {
                    let d = size * dot;
                    cx.painter.circle(center, d / 2.0, dot_c.multiply_alpha(inner_dot), 0.0, Color::TRANSPARENT);
                }
                if ring_a > 0.0 {
                    let thumb = Rect::from_center_size(center, vec2(size, size));
                    cx.defer(thumb, Box::new(move |cx, r| cx.focus_ring(r, r.width() / 2.0, ring_a, ring_gap, ring)));
                }
            })),
        );
        if let Some(text) = self.readout.take() {
            let mut st = ui.text_style(TextRole::Label);
            st.tabular = true;
            let n = ui.text_leaf(text, st, readout_c, false, false, Style { width: Size::Fixed(44.0), ..Default::default() });
            ui.a11y_hide(n);
            ui.set_text_align(n, TextAlign::End);
        }
        ui.end();
        ui.end();
        Response { id, rect: ix.rect, hovered: ix.hovered, clicked: false, changed, focused: ix.focused }
    }
}

fn quantize(range: &RangeInclusive<f32>, step: Option<f32>, v: f32) -> f32 {
    let (lo, hi) = (*range.start(), *range.end());
    let v = v.clamp(lo.min(hi), hi.max(lo));
    match step {
        Some(s) if s > 0.0 => (lo + ((v - lo) / s).round() * s).clamp(lo.min(hi), hi.max(lo)),
        _ => v,
    }
}

show_on_drop!(Slider);

impl Ui {
    /// A slider over `range`. Pass `""` for no field label.
    pub fn slider<'u>(&'u mut self, label: impl Into<String>, value: &'u mut f32, range: RangeInclusive<f32>) -> Slider<'u> {
        Slider { ui: self, label: label.into(), value, range, step: None, readout: None, width: None, disabled: false, shown: false }
    }
}
