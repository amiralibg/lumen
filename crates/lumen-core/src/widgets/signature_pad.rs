//! Signature pad: draw with a mouse, pen or finger. Strokes are smoothed,
//! thin out as the pen speeds up, and land in accent ink that dries to the
//! foreground. Undo and Clear fade strokes away; the hint fades out once
//! there's a signature and a check pops in.

use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Vec2};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

use super::show_on_drop;

/// One sampled point: position in pad units (0..1 on both axes, so a
/// signature survives a resize) and a width factor (0..1, slower is wider).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SigPoint {
    pub x: f32,
    pub y: f32,
    pub w: f32,
}

/// The strokes of a signature.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Signature {
    pub strokes: Vec<Vec<SigPoint>>,
}

impl Signature {
    pub fn is_empty(&self) -> bool {
        self.strokes.iter().all(|s| s.is_empty())
    }
    pub fn clear(&mut self) {
        self.strokes.clear();
    }
    /// Remove the last stroke; false if there was none.
    pub fn undo(&mut self) -> bool {
        self.strokes.pop().is_some()
    }
}

#[derive(Default)]
struct PadMem {
    drawing: bool,
    /// Last sample: position (px) and time, for speed.
    last: Option<(Vec2, f64)>,
    /// Smoothed width factor.
    w: f32,
    /// When the last stroke was lifted (its ink dries from accent).
    lifted: f64,
    /// Strokes on their way out after Undo or Clear, with the time they left.
    ghosts: Vec<(Vec<Vec<SigPoint>>, f64)>,
}

pub struct SignaturePad<'u> {
    ui: &'u mut Ui,
    label: String,
    sig: &'u mut Signature,
    height: Option<f32>,
    hint: String,
    shown: bool,
}

impl SignaturePad<'_> {
    /// Height of the drawing area.
    pub fn height(mut self, h: f32) -> Self {
        self.height = Some(h);
        self
    }
    /// Text under the line while the pad is empty.
    pub fn hint(mut self, s: impl Into<String>) -> Self {
        self.hint = s.into();
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("signature", &self.label));
        let sense = Sense { click: true, drag: true, focusable: true, scroll: false };
        let ix = ui.interact(id, sense, false);
        let mo = ui.motion();
        let reduced = ui.reduced_motion();
        let now = ui.time();
        let lk = ui.look().clone();
        let pen = (lk.control.height[1] / 12.0).max(lk.focus.width);
        let rad = lk.control.radius[2].max(lk.card.radius * 0.6);
        let dry = mo.layer_in * 3.0 + 0.2;
        let leave = mo.fade_out * 2.0 + 0.1;
        let r = ix.rect;
        let mut changed = false;

        // Drawing: sample while held, wider when slow.
        let to_unit = |p: Vec2| vec2(((p.x - r.min.x) / r.width().max(1.0)).clamp(0.0, 1.0), ((p.y - r.min.y) / r.height().max(1.0)).clamp(0.0, 1.0));
        let m = ui.mem::<PadMem>(id);
        if ix.press_started && r.width() > 0.0 {
            if let Some(p) = ix.pointer {
                m.drawing = true;
                m.last = Some((p, now));
                m.w = 0.8;
                let u = to_unit(p);
                self.sig.strokes.push(vec![SigPoint { x: u.x, y: u.y, w: m.w }]);
            }
        } else if m.drawing && ix.held {
            if let (Some(p), Some((lp, lt))) = (ix.pointer, m.last) {
                let d = (p - lp).length();
                if d >= 1.5 {
                    let dt = ((now - lt) as f32).max(1.0 / 240.0);
                    let speed = d / dt;
                    let target = (1.0 - speed / 1800.0).clamp(0.35, 1.0);
                    m.w = m.w * 0.65 + target * 0.35;
                    m.last = Some((p, now));
                    let u = to_unit(p);
                    if let Some(s) = self.sig.strokes.last_mut() {
                        s.push(SigPoint { x: u.x, y: u.y, w: m.w });
                    }
                }
            }
        } else if m.drawing {
            m.drawing = false;
            m.lifted = now;
            changed = true;
        }
        let drawing = m.drawing;

        // Undo / clear from the keyboard: Backspace drops the last stroke,
        // Cmd/Ctrl+Backspace clears.
        let mut undo = false;
        let mut clear = false;
        if ix.focused {
            for (k, mods, _) in ui.keys_for(id) {
                match k {
                    Key::Backspace | Key::Delete if mods.command() => clear = true,
                    Key::Backspace | Key::Delete => undo = true,
                    Key::Char('z') if mods.command() => undo = true,
                    _ => {}
                }
            }
        }

        let empty = self.sig.is_empty();
        let hint_a = 1.0 - ui.fade(id, 0, !empty, mo.hover_in, mo.fade_out, Ease::Out);
        let ink_on = ui.fade(id, 1, drawing || ix.focused, mo.hover_in, mo.hover_out, Ease::Linear);
        let (ring_a, ring_gap) = ui.focus_anim(&ix);

        let fg = ui.color(Token::Foreground);
        let accent = ui.color(Token::Accent);
        let line_c = ui.color(Token::BorderStrong).mix(accent, ink_on);
        let muted = ui.color(Token::TextMuted);
        let bg = ui.color(Token::SurfaceMuted).mix(ui.color(Token::Surface), 0.5);
        let border = ui.color(Token::Border).mix(accent, ink_on * 0.6);
        let ring = ui.color(Token::FocusRing);
        let st_hint = ui.text_style(TextRole::Caption);
        let st_x = ui.text_style(TextRole::Heading3);
        let hint = self.hint.clone();
        let height = self.height.unwrap_or(lk.control.height[1] * 5.0);

        let col = ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: lk.control.gap[1].max(6.0), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let _ = col;
        if !self.label.is_empty() {
            let st = ui.text_style(TextRole::Label);
            let n = ui.text_leaf(self.label.clone(), st, fg, false, true, Style::default());
            ui.a11y_hide(n);
        }

        // The drawing itself: strokes, ghosts on their way out, wet ink.
        let strokes = self.sig.strokes.clone();
        let ghosts: Vec<(Vec<Vec<SigPoint>>, f32)> = {
            let m = ui.mem::<PadMem>(id);
            m.ghosts.retain(|(_, t)| now - t < leave as f64);
            m.ghosts.iter().map(|(g, t)| (g.clone(), ((now - t) as f32 / leave).clamp(0.0, 1.0))).collect()
        };
        let lifted = ui.mem::<PadMem>(id).lifted;
        let wet = if reduced || drawing { if drawing { 0.0 } else { 1.0 } } else { ((now - lifted) as f32 / dry).clamp(0.0, 1.0) };
        if !ghosts.is_empty() || (wet < 1.0 && !drawing) {
            ui.request_repaint_after(0.0);
        }
        let last_ix = strokes.len().saturating_sub(1);
        let drop = if reduced { 0.0 } else { pen * 3.0 };
        ui.leaf(
            Style { width: Size::FILL, height: Size::Fixed(height), ..Default::default() },
            vec2(200.0, height),
            Some(id),
            sense,
            CursorIcon::Crosshair,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(rad), bg, 1.0, border);
                // The line to sign on, with an x at its start.
                let base_y = r.min.y + r.height() * 0.72;
                let pad = rad.max(pen * 6.0);
                let gx = cx.galley("×", &st_x);
                let x0 = r.min.x + pad;
                cx.painter.text(cx.text, &gx, vec2(x0, base_y - gx.height - pen), line_c);
                cx.painter.rect(Rect { min: vec2(x0, base_y), max: vec2(r.max.x - pad, base_y + 1.0) }, line_c);
                if hint_a > 0.01 && !hint.is_empty() {
                    let g = cx.galley(&hint, &st_hint);
                    cx.painter.text(cx.text, &g, vec2(r.center().x - g.width * 0.5, base_y + pen * 2.0), muted.multiply_alpha(hint_a));
                }
                cx.painter.push_clip(r);
                let map = |p: &SigPoint, dy: f32| vec2(r.min.x + p.x * r.width(), r.min.y + p.y * r.height() + dy);
                for (g, t) in &ghosts {
                    let a = 1.0 - t;
                    for s in g {
                        draw_stroke(cx, s, &map, t * t * drop, pen, fg.multiply_alpha(a * a));
                    }
                }
                for (i, s) in strokes.iter().enumerate() {
                    let c = if i == last_ix { accent.mix(fg, wet) } else { fg };
                    draw_stroke(cx, s, &map, 0.0, pen, c);
                }
                cx.painter.pop_clip();
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr, rad, ring_a, ring_gap, ring)));
                }
            })),
        );

        // Footer: status on the left, Undo and Clear on the right.
        ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: lk.control.gap[0].max(4.0), align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let signed = ui.spring(id, 2, if empty { 0.0 } else { 1.0 }, mo.bouncy);
        let signed = if reduced { if empty { 0.0 } else { 1.0 } } else { signed };
        let ok = ui.color(Token::Success);
        let isz = ui.text_style(TextRole::Caption).size + 2.0;
        ui.leaf(
            Style::default(),
            vec2(isz, isz),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let s = signed.clamp(0.0, 1.3);
                if s > 0.01 {
                    cx.icon("circle-check", r.center(), isz * s, ok.multiply_alpha(signed.clamp(0.0, 1.0)));
                } else {
                    cx.icon("pen-line", r.center(), isz, muted);
                }
            })),
        );
        let status = if empty { "Not signed".to_owned() } else { "Signed".to_owned() };
        let st = ui.text_style(TextRole::Caption);
        let n = ui.text_leaf(status, st, if empty { muted } else { fg }, false, true, Style { width: Size::FILL, ..Default::default() });
        ui.a11y_hide(n);
        let can = !empty;
        undo |= ui.scope(id.with("undo"), |ui| ui.icon_button("undo-2").ghost().small().tooltip("Undo stroke").disabled(!can).show().clicked);
        clear |= ui.scope(id.with("clear"), |ui| ui.icon_button("eraser").ghost().small().tooltip("Clear").disabled(!can).show().clicked);
        ui.end();
        ui.end();

        if clear && !self.sig.is_empty() {
            let gone = std::mem::take(&mut self.sig.strokes);
            ui.mem::<PadMem>(id).ghosts.push((gone, now));
            changed = true;
        } else if undo {
            if let Some(s) = self.sig.strokes.pop() {
                ui.mem::<PadMem>(id).ghosts.push((vec![s], now));
                changed = true;
            }
        }

        let n = self.sig.strokes.len();
        let value = if n == 0 { "Empty".to_owned() } else { format!("Signed, {n} stroke{}", if n == 1 { "" } else { "s" }) };
        ui.a11y(id, || {
            A11y::new(Role::Canvas)
                .label(if self.label.is_empty() { "Signature".to_owned() } else { self.label.clone() })
                .value(value)
                .description("Draw to sign. Backspace undoes a stroke, Command or Control Backspace clears.")
        });
        Response { id, rect: ui.rect_of(id).unwrap_or(r), hovered: ix.hovered, clicked: false, changed, focused: ix.focused }
    }
}

/// A smoothed stroke: quadratic curves through the midpoints, each
/// piece as wide as its pen speed allowed, with round joins.
fn draw_stroke(cx: &mut crate::ui::PaintCx, s: &[SigPoint], map: &impl Fn(&SigPoint, f32) -> Vec2, dy: f32, pen: f32, c: Color) {
    if c.a <= 0.0 || s.is_empty() {
        return;
    }
    let width = |w: f32| pen * (0.55 + 0.9 * w);
    if s.len() == 1 {
        cx.painter.circle(map(&s[0], dy), width(s[0].w) * 0.5, c, 0.0, Color::TRANSPARENT);
        return;
    }
    let pts: Vec<Vec2> = s.iter().map(|p| map(p, dy)).collect();
    let mid = |a: Vec2, b: Vec2| (a + b) * 0.5;
    let mut prev = pts[0];
    let mut prev_w = s[0].w;
    for i in 1..pts.len() {
        let end = if i + 1 < pts.len() { mid(pts[i], pts[i + 1]) } else { pts[i] };
        let ctrl = pts[i];
        let start = prev;
        let steps = (((end - start).length() / 3.0).ceil() as usize).clamp(1, 8);
        for k in 1..=steps {
            let t = k as f32 / steps as f32;
            let a = start + (ctrl - start) * t;
            let b = ctrl + (end - ctrl) * t;
            let q = a + (b - a) * t;
            let w = prev_w + (s[i].w - prev_w) * t;
            let wpx = width(w);
            cx.painter.polyline(&[prev, q], wpx, c);
            cx.painter.circle(q, wpx * 0.5, c, 0.0, Color::TRANSPARENT);
            prev = q;
        }
        prev_w = s[i].w;
    }
}

show_on_drop!(SignaturePad);

impl Ui {
    /// A pad to sign on. `label` names it (pass `""` for none).
    pub fn signature_pad<'u>(&'u mut self, label: impl Into<String>, sig: &'u mut Signature) -> SignaturePad<'u> {
        SignaturePad { ui: self, label: label.into(), sig, height: None, hint: "Sign above the line".into(), shown: false }
    }
}
