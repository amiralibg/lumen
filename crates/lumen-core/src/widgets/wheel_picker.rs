//! Wheel picker: a column of values on a drum. Drag it (1:1, then it keeps
//! its momentum and springs onto an item), scroll it a step at a time, or
//! walk it with the arrow keys. Items curve away and fade with distance
//! from the center band. [`Ui::time_wheel`] pairs two for a time.

use crate::anim::Ease;
use crate::date::Time;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};

/// Items visible above and below the center.
const REACH: i32 = 2;

#[derive(Default)]
struct WheelMem {
    /// Pointer y and position when the drag started.
    drag: Option<(f32, f32)>,
    /// Position while dragging (items, fractional).
    pos: f32,
    /// Recent velocity (items per second) for the throw.
    vel: f32,
    last: Option<(f32, f64)>,
    wheel: f32,
}

impl Ui {
    /// A wheel over `items`; `value` is the chosen index. Drag, scroll or
    /// use ↑ ↓ Home End PageUp PageDown.
    pub fn wheel_picker(&mut self, key: impl std::hash::Hash, value: &mut usize, items: &[&str]) -> Response {
        let id = self.make_id(("wheel", key));
        let n = items.len().max(1);
        *value = (*value).min(n - 1);
        let old = *value;
        let sense = Sense { click: true, drag: true, focusable: true, scroll: true };
        let ix = self.interact(id, sense, false);
        let mo = self.motion();
        let reduced = self.reduced_motion();
        let now = self.time();
        let item_h = self.look().control.height[1].max(28.0);

        // Drag: track 1:1, measure velocity; on release throw and snap.
        let p = ix.pointer.map(|p| p.y);
        let m = self.mem::<WheelMem>(id);
        if ix.press_started {
            if let Some(y) = p {
                m.drag = Some((y, *value as f32));
                m.pos = *value as f32;
                m.vel = 0.0;
                m.last = Some((y, now));
            }
        }
        let mut snap_from = None;
        if let (true, Some((y0, p0)), Some(y)) = (ix.held, m.drag, p) {
            let pos = (p0 - (y - y0) / item_h).clamp(-0.4, n as f32 - 0.6);
            if let Some((ly, lt)) = m.last {
                let dt = (now - lt) as f32;
                if dt > 0.0 {
                    let v = -(y - ly) / item_h / dt;
                    m.vel = m.vel * 0.6 + v * 0.4;
                }
            }
            m.last = Some((y, now));
            m.pos = pos;
            *value = pos.round().clamp(0.0, (n - 1) as f32) as usize;
        } else if m.drag.is_some() {
            // Released: project the throw a quarter second ahead.
            let landing = (m.pos + m.vel * 0.25).round().clamp(0.0, (n - 1) as f32);
            *value = landing as usize;
            snap_from = Some(m.pos);
            m.drag = None;
            m.vel = 0.0;
        }
        let dragging = m.drag.is_some();
        let drag_pos = m.pos;

        // Wheel: one item per notch (pixel deltas accumulate).
        let wheel = self.take_scroll(id).y;
        if wheel != 0.0 {
            let m = self.mem::<WheelMem>(id);
            m.wheel += wheel / item_h;
            while m.wheel >= 0.5 {
                m.wheel -= 1.0;
                *value = (*value + 1).min(n - 1);
            }
            while m.wheel <= -0.5 {
                m.wheel += 1.0;
                *value = value.saturating_sub(1);
            }
        }
        if ix.focused {
            for (k, mods, _) in self.keys_for(id) {
                if mods.command() {
                    continue;
                }
                *value = match k {
                    Key::ArrowDown => (*value + 1).min(n - 1),
                    Key::ArrowUp => value.saturating_sub(1),
                    Key::PageDown => (*value + 5).min(n - 1),
                    Key::PageUp => value.saturating_sub(5),
                    Key::Home => 0,
                    Key::End => n - 1,
                    _ => *value,
                };
            }
        }
        if ix.clicked && !dragging && snap_from.is_none() {
            // A tap above or below the band steps toward it.
            if let Some(y) = p {
                let d = ((y - ix.rect.center().y) / item_h).round() as i32;
                *value = (*value as i32 + d).clamp(0, n as i32 - 1) as usize;
            }
        }

        // Position shown: the drag itself, else a spring onto the value.
        if dragging {
            self.anims.set_spring(id, 0, drag_pos);
        } else if let Some(from) = snap_from {
            self.anims.set_spring(id, 0, from);
        }
        let shown = if dragging || reduced { if dragging { drag_pos } else { *value as f32 } } else { self.spring(id, 0, *value as f32, mo.smooth) };
        let changed = *value != old;

        let h = self.fade(id, 2, ix.hovered || dragging, mo.hover_in, mo.hover_out, Ease::Linear);
        let (ring_a, ring_gap) = self.focus_anim(&ix);
        let band = self.color(Token::SurfaceMuted).mix(self.color(Token::SurfaceMuted).over(self.color(Token::Hover)), h);
        let band_border = self.color(Token::BorderSubtle);
        let fg = self.color(Token::Foreground);
        let sec = self.color(Token::TextSecondary);
        let ring = self.color(Token::FocusRing);
        let radius = self.look().control.radius[1];
        let st = self.text_style(TextRole::Body);
        let st_on = self.text_style(TextRole::BodyStrong);
        let widest = items.iter().map(|t| self.galley(t, &st_on).width).fold(0.0f32, f32::max);
        let width = (widest + 36.0).max(56.0);
        let height = item_h * (2 * REACH + 1) as f32;
        let labels: Vec<String> = items.iter().map(|s| s.to_string()).collect();

        self.leaf(
            Style { width: Size::Fixed(width), height: Size::Fixed(height), ..Default::default() },
            vec2(width, height),
            Some(id),
            sense,
            if dragging { CursorIcon::Grabbing } else { CursorIcon::Grab },
            Some(Box::new(move |cx, r| {
                let c = r.center().y;
                let band_r = Rect::from_xywh(r.min.x, c - item_h * 0.5, r.width(), item_h);
                cx.painter.rrect(band_r, Corners::all(radius), band, 1.0, band_border);
                cx.painter.push_clip(r);
                let first = (shown.floor() as i32 - REACH - 1).max(0);
                let last = (shown.ceil() as i32 + REACH + 1).min(labels.len() as i32 - 1);
                for i in first..=last {
                    let d = i as f32 - shown;
                    if d.abs() > REACH as f32 + 0.8 {
                        continue;
                    }
                    // A drum: items bunch up and fade toward the edges.
                    let a = d * 0.5;
                    let y = c + a.sin() / 0.5 * item_h;
                    let near = (1.0 - d.abs()).clamp(0.0, 1.0);
                    let alpha = (1.0 - 0.3 * d.abs()).clamp(0.0, 1.0) * a.cos().max(0.0);
                    let g = cx.galley(&labels[i as usize], if near > 0.5 { &st_on } else { &st });
                    let scale = 1.0 - 0.06 * d.abs().min(3.0);
                    let col = sec.mix(fg, near).multiply_alpha(alpha);
                    let x = r.center().x - g.width * 0.5;
                    let gy = y - g.height * 0.5 * scale;
                    cx.painter.text(cx.text, &g, vec2(x.round(), gy.round()), col);
                }
                cx.painter.pop_clip();
                if ring_a > 0.0 {
                    cx.defer(band_r, Box::new(move |cx, rr| cx.focus_ring(rr, radius, ring_a, ring_gap, ring)));
                }
            })),
        );
        let rect = self.rect_of(id).unwrap_or(ix.rect);
        Response { id, rect, hovered: ix.hovered, clicked: ix.clicked, changed, focused: ix.focused }
    }

    /// Hours and minutes on two wheels (24-hour), minutes in steps of
    /// `minute_step`.
    pub fn time_wheel(&mut self, key: impl std::hash::Hash, time: &mut Time, minute_step: u8) -> bool {
        let step = minute_step.clamp(1, 30) as usize;
        let hours: Vec<String> = (0..24).map(|h| format!("{h:02}")).collect();
        let minutes: Vec<String> = (0..60).step_by(step).map(|m| format!("{m:02}")).collect();
        let hr: Vec<&str> = hours.iter().map(|s| s.as_str()).collect();
        let mn: Vec<&str> = minutes.iter().map(|s| s.as_str()).collect();
        let mut h = time.hour as usize;
        let mut m = time.minute as usize / step;
        let id = self.make_id(("time_wheel", key));
        let mut changed = false;
        self.scope(id, |ui| {
            ui.begin(Style { dir: Dir::Row, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            changed |= ui.wheel_picker("h", &mut h, &hr).changed;
            let st = ui.text_style(TextRole::Heading3);
            let c = ui.color(Token::TextSecondary);
            ui.text_leaf(":".to_owned(), st, c, false, false, Style::default());
            changed |= ui.wheel_picker("m", &mut m, &mn).changed;
            ui.end();
        });
        time.hour = h as u8;
        time.minute = (m * step) as u8;
        changed
    }
}
