//! Small choice widgets: filter chips, a number stepper, toggle groups and
//! avatars. Geometry comes from the look, colors from tokens, and every state
//! change is a spring (site/DESIGN.md, "micro-motion").

use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect};
use crate::theme::{ControlSize, TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

use super::key_activate;

/// Scale a rect about its center (press squash).
fn squash(r: Rect, s: f32) -> Rect {
    Rect::from_center_size(r.center(), r.size() * s)
}

impl Ui {
    /// A pill that toggles `on`. When on, it tints with the accent and a check
    /// slides in ahead of the label. Use a row of them as filter chips.
    pub fn chip(&mut self, label: &str, on: &mut bool) -> Response {
        let id = self.make_id(("chip", label));
        let ix = self.interact(id, Sense::CLICK, false);
        let (key_act, key_held) = key_activate(self, &ix, true);
        let mut changed = false;
        if ix.clicked || key_act {
            *on = !*on;
            changed = true;
        }
        let mo = self.motion();
        let t = self.spring(id, 0, if *on { 1.0 } else { 0.0 }, mo.bouncy);
        let (h, p) = self.state_weights(&ix, key_held);
        let press = self.spring(id, 1, if ix.pressed || key_held { 1.0 } else { 0.0 }, mo.snappy);
        let (ring_a, ring_gap) = self.focus_anim(&ix);
        let c = |tk| self.color(tk);
        let accent = c(Token::Accent);
        let fill = c(Token::Hover).multiply_alpha(h * (1.0 - t)).over(accent.with_alpha(0.16 * t.clamp(0.0, 1.0)));
        let border = c(Token::BorderStrong).mix(accent.with_alpha(0.6), t.clamp(0.0, 1.0));
        let fg = c(Token::TextSecondary).mix(c(Token::Foreground), (t + h * 0.5).clamp(0.0, 1.0));
        let check = c(Token::AccentStrong);
        let ring = c(Token::FocusRing);
        let _ = p;
        let st = self.text_style(TextRole::Body);
        let g = self.galley(label, &st);
        let check_w = 14.0 + 4.0;
        let h_px = 28.0;
        let w = 11.0 * 2.0 + g.width + check_w * t.clamp(0.0, 1.0);
        // The label can't scale, so the width is a floor: a narrow row must
        // wrap or overflow, never squeeze the pill under its text.
        let node = self.leaf(
            Style { min: vec2(w, h_px), ..Default::default() },
            vec2(w, h_px),
            Some(id),
            Sense::CLICK,
            CursorIcon::Pointer,
            Some(Box::new(move |cx, r| {
                let r = squash(r, 1.0 - 0.05 * press);
                let rad = r.height() / 2.0;
                cx.painter.rrect_unsnapped(r, Corners::all(rad), [fill; 2], 1.0, border);
                let tc = t.clamp(0.0, 1.0);
                if tc > 0.01 {
                    let cc = vec2(r.min.x + 11.0 + 7.0, r.center().y);
                    cx.icon("check", cc, 14.0 * t.max(0.0), check.multiply_alpha(tc));
                }
                let x = r.min.x + 11.0 + check_w * tc;
                cx.painter.text(cx.text, &g, vec2(x.round(), (r.center().y - g.height / 2.0).round()), fg);
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, rad, ring_a, ring_gap, ring)));
                }
            })),
        );
        self.set_hit_pad(node, 2.0);
        Response { id, rect: ix.rect, hovered: ix.hovered, clicked: changed, changed, focused: ix.focused }
    }

    /// A number field with − and + buttons. The value rolls up or down as it
    /// changes. Arrow keys step when focused.
    pub fn stepper(&mut self, key: &str, value: &mut i64, min: i64, max: i64) -> Response {
        let id = self.make_id(("stepper", key));
        let m = self.control_metrics(ControlSize::Md);
        let mo = self.motion();
        let lo_id = id.with("minus");
        let hi_id = id.with("plus");
        let lo = self.interact(lo_id, Sense::CLICK, *value <= min);
        let hi = self.interact(hi_id, Sense::CLICK, *value >= max);
        let focused = lo.focused || hi.focused;
        let mut delta = 0i64;
        if lo.clicked || (focused && (self.key_pressed_for(lo_id, Key::ArrowDown) || self.key_pressed_for(hi_id, Key::ArrowDown))) {
            delta = -1;
        }
        if hi.clicked || (focused && (self.key_pressed_for(lo_id, Key::ArrowUp) || self.key_pressed_for(hi_id, Key::ArrowUp))) {
            delta = 1;
        }
        let before = *value;
        *value = (*value + delta).clamp(min, max);
        let changed = *value != before;
        // Direction of the roll: +1 rolls up, -1 rolls down.
        if changed {
            self.anims.set_spring(id, 0, -(*value - before).signum() as f32);
        }
        let roll = self.anims.spring_from(id, 0, 0.0, 0.0, mo.bouncy);
        let field = self.color(Token::Control);
        let border = self.color(Token::ControlBorder);
        let fg = self.color(Token::Foreground);
        let ring = self.color(Token::FocusRing);
        let (hover, tsec, tdis) = (self.color(Token::Hover), self.color(Token::TextSecondary), self.color(Token::TextDisabled));
        let mut st = self.text_style(TextRole::Body);
        st.weight = 600;
        let text = value.to_string();
        let g = self.galley(&text, &st);
        let radius = m.radius;
        let hgt = m.height;
        self.begin(
            Style { dir: Dir::Row, height: Size::Fixed(hgt), align: Align::Center, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(radius), field, 1.0, border))),
        );
        for (bid, ix, icon, left) in [(lo_id, lo, "minus", true), (hi_id, hi, "plus", false)] {
            let (h, _) = self.state_weights(&ix, false);
            let press = self.spring(bid, 5, if ix.pressed { 1.0 } else { 0.0 }, mo.snappy);
            let (ring_a, ring_gap) = self.focus_anim(&ix);
            let hl = hover.multiply_alpha(h);
            let ic = if ix.disabled { tdis } else { tsec.mix(fg, h) };
            let corners = if left { Corners([radius, 0.0, 0.0, radius]) } else { Corners([0.0, radius, radius, 0.0]) };
            if !left {
                self.leaf(
                    Style::default(),
                    vec2(36.0, hgt),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new({
                        let g = g.clone();
                        move |cx, r| {
                            cx.painter.push_clip(r);
                            let y = r.center().y - g.height / 2.0 + roll * 10.0;
                            let a = (1.0 - roll.abs()).clamp(0.0, 1.0);
                            cx.painter.text(cx.text, &g, vec2((r.center().x - g.width / 2.0).round(), y), fg.multiply_alpha(a));
                            cx.painter.pop_clip();
                        }
                    })),
                );
            }
            self.leaf(
                Style::default(),
                vec2(34.0, hgt),
                Some(bid),
                Sense::CLICK,
                if ix.disabled { CursorIcon::Default } else { CursorIcon::Pointer },
                Some(Box::new(move |cx, r| {
                    let r = r.expand(-1.0);
                    if hl.a > 0.0 {
                        cx.painter.rrect(r, corners, hl, 0.0, Color::TRANSPARENT);
                    }
                    cx.icon(icon, r.center(), 15.0 * (1.0 - 0.12 * press), ic);
                    if ring_a > 0.0 {
                        cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, radius - 1.0, ring_a, ring_gap, ring)));
                    }
                })),
            );
        }
        self.end();
        Response { id, rect: self.rect_of(id).unwrap_or_default(), hovered: lo.hovered || hi.hovered, clicked: changed, changed, focused }
    }

    /// A row of equal buttons where one is selected, like time slots. The
    /// selection fills with the accent and the pressed cell squashes.
    pub fn toggle_group<T: Copy + PartialEq>(&mut self, key: &str, value: &mut T, options: &[(T, &str)]) -> Response {
        let id = self.make_id(("toggle_group", key));
        let mo = self.motion();
        let m = self.control_metrics(ControlSize::Sm);
        let mut changed = false;
        let mut hovered = false;
        let mut focused = false;
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);
        let ids: Vec<_> = (0..options.len()).map(|i| id.with(i)).collect();
        for (i, &(v, label)) in options.iter().enumerate() {
            let oid = ids[i];
            let ix = self.interact(oid, Sense::CLICK, false);
            if ix.focused {
                self.arrow_focus(oid, &ids);
            }
            let (key_act, key_held) = key_activate(self, &ix, true);
            if (ix.clicked || key_act) && *value != v {
                *value = v;
                changed = true;
            }
            hovered |= ix.hovered;
            focused |= ix.focused;
            let sel = *value == v;
            let t = self.spring(oid, 0, if sel { 1.0 } else { 0.0 }, mo.snappy);
            let h = self.fade(oid, 1, ix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
            let press = self.spring(oid, 2, if ix.pressed || key_held { 1.0 } else { 0.0 }, mo.snappy);
            let (ring_a, ring_gap) = self.focus_anim(&ix);
            let c = |tk| self.color(tk);
            let fill = c(Token::Hover).multiply_alpha(h).mix(c(Token::Accent), t);
            let border = c(Token::BorderStrong).mix(c(Token::Accent), t);
            let fg = c(Token::Foreground).mix(c(Token::AccentForeground), t);
            let ring = c(Token::FocusRing);
            let mut st = self.text_style(TextRole::Body);
            st.size = m.text.size;
            let g = self.galley(label, &st);
            let radius = m.radius;
            self.leaf(
                Style { width: Size::FILL, min: vec2(g.width + 16.0, 0.0), ..Default::default() },
                vec2(g.width + 16.0, m.height),
                Some(oid),
                Sense::CLICK,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    let r = squash(r, 1.0 - 0.05 * press);
                    cx.painter.rrect_unsnapped(r, Corners::all(radius), [fill; 2], 1.0, border);
                    let x = r.center().x - g.width / 2.0;
                    let y = r.center().y - g.height / 2.0;
                    cx.painter.text(cx.text, &g, vec2(x.round(), y.round()), fg);
                    if ring_a > 0.0 {
                        cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, radius, ring_a, ring_gap, ring)));
                    }
                })),
            );
        }
        self.end();
        Response { id, rect: self.rect_of(id).unwrap_or_default(), hovered, clicked: changed, changed, focused }
    }

    /// A round avatar with initials, tinted from the chart series by `tone`.
    /// `presence` draws a status dot (green online, amber away).
    pub fn avatar(&mut self, initials: &str, tone: usize, size: f32, presence: Option<bool>) {
        let tk = [Token::Series0, Token::Series1, Token::Series2, Token::Series3, Token::Series4, Token::Series5][tone % 6];
        let bg = self.color(tk);
        let fg = Color::WHITE;
        let ring = self.color(Token::Surface);
        let online = self.color(Token::Success);
        let away = self.color(Token::Warning);
        let mut st = self.text_style(TextRole::Caption);
        st.weight = 600;
        st.size = (size * 0.36).round();
        let g = self.galley(initials, &st);
        self.leaf(
            Style::default(),
            vec2(size, size),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let c = r.center();
                cx.painter.circle(c, size / 2.0, bg, 0.0, Color::TRANSPARENT);
                cx.painter.text(cx.text, &g, vec2((c.x - g.width / 2.0).round(), (c.y - g.height / 2.0).round()), fg);
                if let Some(on) = presence {
                    let d = (size * 0.28).max(8.0);
                    let p = vec2(r.max.x - d / 2.0, r.max.y - d / 2.0);
                    cx.painter.circle(p, d / 2.0, if on { online } else { away }, 2.0, ring);
                }
            })),
        );
    }

    /// Overlapping avatars, each ringed in the surface color, with a "+n" chip
    /// when there are more than `max`.
    pub fn avatar_group(&mut self, people: &[&str], max: usize, size: f32) {
        let ring = self.color(Token::Surface);
        let muted = self.color(Token::SurfaceMuted);
        let fg = self.color(Token::TextSecondary);
        self.begin(Style { dir: Dir::Row, align: Align::Center, justify: Justify::Start, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let shown = people.len().min(max);
        for (i, who) in people.iter().take(shown).enumerate() {
            let tk = [Token::Series0, Token::Series1, Token::Series2, Token::Series3, Token::Series4, Token::Series5][i % 6];
            let bg = self.color(tk);
            let mut st = self.text_style(TextRole::Caption);
            st.weight = 600;
            st.size = (size * 0.36).round();
            let g = self.galley(who, &st);
            let w = if i == 0 { size } else { size * 0.82 };
            self.leaf(
                Style::default(),
                vec2(w, size),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    let c = vec2(r.max.x - size / 2.0, r.center().y);
                    cx.painter.circle(c, size / 2.0 + 2.0, ring, 0.0, Color::TRANSPARENT);
                    cx.painter.circle(c, size / 2.0, bg, 0.0, Color::TRANSPARENT);
                    cx.painter.text(cx.text, &g, vec2((c.x - g.width / 2.0).round(), (c.y - g.height / 2.0).round()), Color::WHITE);
                })),
            );
        }
        if people.len() > shown {
            let mut st = self.text_style(TextRole::Caption);
            st.weight = 600;
            let g = self.galley(&format!("+{}", people.len() - shown), &st);
            self.leaf(
                Style::default(),
                vec2(size * 0.82, size),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    let c = vec2(r.max.x - size / 2.0, r.center().y);
                    cx.painter.circle(c, size / 2.0 + 2.0, ring, 0.0, Color::TRANSPARENT);
                    cx.painter.circle(c, size / 2.0, muted, 0.0, Color::TRANSPARENT);
                    cx.painter.text(cx.text, &g, vec2((c.x - g.width / 2.0).round(), (c.y - g.height / 2.0).round()), fg);
                })),
            );
        }
        self.end();
    }
}
