//! Ruler slider: a tick ruler that slides under a fixed needle, the way
//! camera and scale apps pick a value. Drag it 1:1 and throw it (it keeps
//! its momentum, then springs onto a tick), scroll a tick at a time, or use
//! the arrow keys. Ticks near the needle grow, the edges fade, and the
//! readout rolls the way the value moved.

use std::ops::RangeInclusive;

use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

use super::show_on_drop;

#[derive(Default)]
struct RulerMem {
    /// Pointer x and position (in steps) when the drag started.
    drag: Option<(f32, f32)>,
    pos: f32,
    /// Steps per second, for the throw.
    vel: f32,
    last: Option<(f32, f64)>,
    wheel: f32,
    /// The readout before the last change, when it changed, and which way.
    roll: Option<(String, f64, f32)>,
    shown_text: String,
}

pub struct RulerSlider<'u> {
    ui: &'u mut Ui,
    label: String,
    value: &'u mut f32,
    range: RangeInclusive<f32>,
    step: f32,
    major: u32,
    unit: String,
    decimals: Option<usize>,
    shown: bool,
}

impl RulerSlider<'_> {
    /// Value between ticks (default 1).
    pub fn step(mut self, s: f32) -> Self {
        self.step = s.abs().max(f32::EPSILON);
        self
    }
    /// A labelled long tick every `n` steps (default 10).
    pub fn major(mut self, n: u32) -> Self {
        self.major = n.max(1);
        self
    }
    /// Unit after the readout: `"kg"`, `"cm"`, `"°"`.
    pub fn unit(mut self, u: impl Into<String>) -> Self {
        self.unit = u.into();
        self
    }
    /// Decimals in the readout (default: enough for the step).
    pub fn decimals(mut self, d: usize) -> Self {
        self.decimals = Some(d);
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("ruler", &self.label));
        let sense = Sense { click: true, drag: true, focusable: true, scroll: true };
        let ix = ui.interact(id, sense, false);
        let mo = ui.motion();
        let reduced = ui.reduced_motion();
        let now = ui.time();
        let lk = ui.look().clone();
        let (lo, hi) = (self.range.start().min(*self.range.end()), self.range.start().max(*self.range.end()));
        let step = self.step;
        let n = ((hi - lo) / step).round().max(0.0) as i64;
        let to_i = |v: f32| (((v - lo) / step).round() as i64).clamp(0, n);
        let old_i = to_i(*self.value);
        let mut idx = old_i;
        // Pixels between ticks, from the look's control metrics.
        let gap = (lk.control.height[1] / 4.0).round().max(6.0);
        let major = self.major as i64;

        // Drag 1:1, then throw and snap.
        let p = ix.pointer.map(|p| p.x);
        let m = ui.mem::<RulerMem>(id);
        if ix.press_started {
            if let Some(x) = p {
                m.drag = Some((x, idx as f32));
                m.pos = idx as f32;
                m.vel = 0.0;
                m.last = Some((x, now));
            }
        }
        let mut snap_from = None;
        if let (true, Some((x0, p0)), Some(x)) = (ix.held, m.drag, p) {
            let pos = (p0 - (x - x0) / gap).clamp(-0.4, n as f32 + 0.4);
            if let Some((lx, lt)) = m.last {
                let dt = (now - lt) as f32;
                if dt > 0.0 {
                    let v = -(x - lx) / gap / dt;
                    m.vel = m.vel * 0.6 + v * 0.4;
                }
            }
            m.last = Some((x, now));
            m.pos = pos;
            idx = (pos.round() as i64).clamp(0, n);
        } else if m.drag.is_some() {
            let moved = (m.pos - m.drag.map(|d| d.1).unwrap_or(m.pos)).abs() > 0.3;
            if moved {
                idx = ((m.pos + m.vel * 0.3).round() as i64).clamp(0, n);
                snap_from = Some(m.pos);
            } else if let Some(x) = p {
                // A tap on a tick goes there.
                let d = ((x - ix.rect.center().x) / gap).round() as i64;
                idx = (idx + d).clamp(0, n);
            }
            m.drag = None;
            m.vel = 0.0;
        }
        let dragging = m.drag.is_some();
        let drag_pos = m.pos;

        // Wheel: horizontal or vertical, one tick per notch.
        let sc = ui.take_scroll(id);
        let wheel = if sc.x.abs() > sc.y.abs() { -sc.x } else { sc.y };
        if wheel != 0.0 {
            let m = ui.mem::<RulerMem>(id);
            m.wheel += wheel / gap;
            while m.wheel >= 0.5 {
                m.wheel -= 1.0;
                idx = (idx + 1).min(n);
            }
            while m.wheel <= -0.5 {
                m.wheel += 1.0;
                idx = (idx - 1).max(0);
            }
        }
        if ix.focused {
            for (k, mods, _) in ui.keys_for(id) {
                let big = if mods.shift { major } else { 1 };
                idx = match k {
                    Key::ArrowRight | Key::ArrowUp => idx + big,
                    Key::ArrowLeft | Key::ArrowDown => idx - big,
                    Key::PageUp => idx + major,
                    Key::PageDown => idx - major,
                    Key::Home => 0,
                    Key::End => n,
                    _ => idx,
                }
                .clamp(0, n);
            }
        }
        let (st, set) = ui.a11y_step(id);
        if let Some(v) = set {
            idx = to_i(v as f32);
        }
        idx = (idx + st as i64).clamp(0, n);

        let value = lo + idx as f32 * step;
        *self.value = value;
        let changed = idx != old_i;

        if dragging {
            ui.anims.set_spring(id, 0, drag_pos);
        } else if let Some(from) = snap_from {
            ui.anims.set_spring(id, 0, from);
        }
        let shown = if dragging {
            drag_pos
        } else if reduced {
            idx as f32
        } else {
            ui.spring(id, 0, idx as f32, mo.smooth)
        };
        let stretch = if reduced { 0.0 } else { ui.spring(id, 1, if dragging || ix.held { 1.0 } else { 0.0 }, mo.bouncy) };
        let hot = ui.fade(id, 2, ix.hovered || dragging, mo.hover_in, mo.hover_out, Ease::Linear);
        let lit = ui.fade(id, 3, dragging || ix.held, mo.hover_in, mo.hover_out, Ease::Linear);
        let (ring_a, ring_gap) = ui.focus_anim(&ix);

        // Readout, rolling up when the value grows and down when it shrinks.
        let dec = self.decimals.unwrap_or_else(|| {
            let s = format!("{step}");
            s.split('.').nth(1).map(|f| f.len()).unwrap_or(0)
        });
        let text = format!("{value:.dec$}");
        // Major labels only need the major spacing's precision (71, not 71.0).
        let dec_major = {
            let s = format!("{}", step * major as f32);
            s.split('.').nth(1).map(|f| f.len().min(dec)).unwrap_or(0)
        };
        let roll_t = mo.layer_in.max(0.12) * 1.6;
        let m = ui.mem::<RulerMem>(id);
        if m.shown_text.is_empty() {
            m.shown_text = text.clone();
        } else if m.shown_text != text {
            let dir = if idx > old_i || (idx == old_i && text > m.shown_text) { 1.0 } else { -1.0 };
            let prev = std::mem::replace(&mut m.shown_text, text.clone());
            if !reduced {
                m.roll = Some((prev, now, dir));
            }
        }
        let roll = match &m.roll {
            Some((prev, t0, dir)) if ((now - t0) as f32) < roll_t => Some((prev.clone(), ((now - t0) as f32 / roll_t).clamp(0.0, 1.0), *dir)),
            _ => None,
        };
        if roll.is_some() {
            ui.anims.keep_alive();
        } else {
            ui.mem::<RulerMem>(id).roll = None;
        }

        let fg = ui.color(Token::Foreground);
        let sec = ui.color(Token::TextSecondary);
        let tick_c = ui.color(Token::BorderStrong);
        let accent = ui.color(Token::Accent);
        let ring = ui.color(Token::FocusRing);
        let bg = ui.color(Token::SurfaceMuted).mix(ui.color(Token::Surface).over(ui.color(Token::Hover)), hot);
        let edge = ui.color(Token::BorderSubtle);
        let mut st_val = ui.text_style(TextRole::Heading1);
        st_val.tabular = true;
        let st_unit = ui.text_style(TextRole::Label);
        let mut st_tick = ui.text_style(TextRole::Caption);
        st_tick.tabular = true;
        let rad = lk.control.radius[2];
        // Tick zone from the control height, then room for the labels.
        let zone = lk.control.height[1] * 1.7;
        let label_gap = lk.control.gap[0].max(3.0);
        let label_h = st_tick.line_height.max(st_tick.size * 1.2);
        let height = (zone + label_gap * 2.0 + label_h).round();

        ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: lk.control.gap[1].max(6.0), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        // Header: label on the left, the rolling value on the right.
        ui.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::End, gap: lk.control.gap[0].max(4.0), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if !self.label.is_empty() {
            let st = ui.text_style(TextRole::Label);
            let l = ui.text_leaf(self.label.clone(), st, sec, false, true, Style { width: Size::FILL, ..Default::default() });
            ui.a11y_hide(l);
        } else {
            ui.begin(Style { width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            ui.end();
        }
        let gv = ui.galley(&text, &st_val);
        let gw = roll.as_ref().map(|(p, _, _)| ui.galley(p, &st_val).width).unwrap_or(0.0).max(gv.width);
        let val_c = fg.mix(accent, lit);
        let text_now = text.clone();
        let st_v = st_val;
        let vn = ui.leaf(
            Style::default(),
            vec2(gw, gv.height),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                cx.painter.push_clip(r.expand(2.0));
                let g = cx.galley(&text_now, &st_v);
                match &roll {
                    Some((prev, t, dir)) => {
                        let e = Ease::Out.apply(*t);
                        let dy = r.height() * 0.6 * dir;
                        let gp = cx.galley(prev, &st_v);
                        cx.painter.text(cx.text, &gp, vec2(r.max.x - gp.width, r.min.y - dy * e), val_c.multiply_alpha(1.0 - e));
                        cx.painter.text(cx.text, &g, vec2(r.max.x - g.width, r.min.y + dy * (1.0 - e)), val_c.multiply_alpha(e));
                    }
                    None => cx.painter.text(cx.text, &g, vec2(r.max.x - g.width, r.min.y), val_c),
                }
                cx.painter.pop_clip();
            })),
        );
        ui.a11y_hide(vn);
        if !self.unit.is_empty() {
            let u = ui.text_leaf(self.unit.clone(), st_unit, sec, false, false, Style { pad: crate::math::Sides { bottom: 4.0, ..Default::default() }, ..Default::default() });
            ui.a11y_hide(u);
        }
        ui.end();

        // The ruler.
        ui.leaf(
            Style { width: Size::FILL, height: Size::Fixed(height), ..Default::default() },
            vec2(160.0, height),
            Some(id),
            sense,
            if dragging { CursorIcon::Grabbing } else { CursorIcon::Grab },
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(rad), bg, 1.0, edge);
                cx.painter.push_clip(r);
                let cxp = r.center().x;
                let base = r.max.y - label_h - label_gap * 2.0;
                let half = (r.width() * 0.5 / gap).ceil() as i64 + 1;
                let first = (shown.floor() as i64 - half).max(0);
                let last = (shown.ceil() as i64 + half).min(n);
                for i in first..=last {
                    let x = cxp + (i as f32 - shown) * gap;
                    let edge_d = ((x - r.min.x).min(r.max.x - x) / (r.width() * 0.22)).clamp(0.0, 1.0);
                    let near = (1.0 - (x - cxp).abs() / (gap * 3.5)).clamp(0.0, 1.0);
                    let is_major = i % major == 0;
                    let is_mid = major % 2 == 0 && i % (major / 2) == 0;
                    let th = zone * if is_major { 0.48 } else if is_mid { 0.34 } else { 0.22 } * (1.0 + 0.35 * near);
                    let c = tick_c.mix(fg, near * 0.6).multiply_alpha(edge_d);
                    cx.painter.rect(Rect { min: vec2((x - 0.5).round(), base - th), max: vec2((x - 0.5).round() + 1.0, base) }, c);
                    if is_major {
                        let v = lo + i as f32 * step;
                        let label = format!("{v:.dec_major$}");
                        let g = cx.galley(&label, &st_tick);
                        cx.painter.text(cx.text, &g, vec2((x - g.width * 0.5).round(), base + label_gap), sec.multiply_alpha(edge_d));
                    }
                }
                cx.painter.pop_clip();
                // The needle: a rounded accent bar that stretches while held.
                let nh = zone * (0.72 + 0.16 * stretch);
                let nw = 3.0 + stretch;
                let needle = Rect { min: vec2(cxp - nw * 0.5, base - nh), max: vec2(cxp + nw * 0.5, base + 2.0) };
                cx.painter.rrect_unsnapped(needle, Corners::all(nw * 0.5), [accent; 2], 0.0, Color::TRANSPARENT);
                cx.painter.circle(vec2(cxp, base - nh), nw * (1.1 + 0.3 * stretch), accent, 0.0, Color::TRANSPARENT);
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr, rad, ring_a, ring_gap, ring)));
                }
            })),
        );
        ui.end();

        let label = if self.label.is_empty() { "Value".to_owned() } else { self.label.clone() };
        let unit = self.unit.clone();
        ui.a11y(id, || {
            let mut a = A11y::new(Role::Slider).label(label).numeric(value as f64, lo as f64, hi as f64, Some(step as f64));
            if !unit.is_empty() {
                a = a.value(format!("{} {}", text, unit));
            }
            a
        });
        Response { id, rect: ui.rect_of(id).unwrap_or(ix.rect), hovered: ix.hovered, clicked: ix.clicked, changed, focused: ix.focused }
    }
}

show_on_drop!(RulerSlider);

impl Ui {
    /// A value on a sliding tick ruler. `label` names it (`""` for none).
    pub fn ruler_slider<'u>(&'u mut self, label: impl Into<String>, value: &'u mut f32, range: RangeInclusive<f32>) -> RulerSlider<'u> {
        RulerSlider { ui: self, label: label.into(), value, range, step: 1.0, major: 10, unit: String::new(), decimals: None, shown: false }
    }
}
