//! Rich selection and input controls: [`RadioCard`], star [`Rating`],
//! dual-thumb [`RangeSlider`], and [`ShortcutRecorder`].

use std::ops::RangeInclusive;

use lumen_text::TextAlign;

use super::{key_activate, show_on_drop};
use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{ControlSize, Elevation, TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

// =============================================================== RadioCard

/// Selectable card option bound to `value` == `option`: shows an optional
/// leading icon, title, right-aligned badge/price, secondary description, and a
/// spring-animated radio indicator.
pub struct RadioCard<'u, T: Copy + PartialEq> {
    ui: &'u mut Ui,
    value: &'u mut T,
    option: T,
    title: String,
    description: Option<String>,
    badge: Option<String>,
    icon: Option<String>,
    disabled: bool,
    shown: bool,
}

impl<'u, T: Copy + PartialEq> RadioCard<'u, T> {
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    /// Trailing pill or price label (e.g. `"$20 / mo"` or `"Recommended"`).
    pub fn badge(mut self, b: impl Into<String>) -> Self {
        self.badge = Some(b.into());
        self
    }
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = Some(name.to_owned());
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
        let id = ui.make_id(("radio_card", &self.title));
        let ix = ui.interact(id, Sense::CLICK, self.disabled);
        let (k_act, k_held) = key_activate(ui, &ix, true);
        let act = (ix.clicked || k_act) && !self.disabled;
        let changed = act && *self.value != self.option;
        if act {
            *self.value = self.option;
        }
        let on = *self.value == self.option;

        let mo = ui.motion();
        let lk_cb = ui.look().checkbox;
        let sel_t = ui.fade(id, 0, on, mo.selected, mo.selected, Ease::Out);
        let dot_t = ui.spring(id, 1, if on { 1.0 } else { 0.0 }, mo.bouncy);
        let hv = ui.fade(id, 2, ix.hovered && !self.disabled, mo.hover_in, mo.hover_out, Ease::Linear);
        let press = ui.spring(id, 3, if (ix.pressed || k_held) && !self.disabled { 1.0 } else { 0.0 }, mo.snappy);
        let (ring_a, ring_gap) = ui.focus_anim(&ix);

        let radius = (ui.look().card.radius * 0.78).max(8.0);
        let (fill, border, tile_bg, ic_c, tc, bc, dc) = {
            let c = |t| ui.color(t);
            let (fill, border) = if self.disabled {
                (c(Token::SurfaceMuted), c(Token::BorderSubtle))
            } else {
                let base = c(Token::Surface).mix(c(Token::Surface).over(c(Token::Hover)), hv * (1.0 - sel_t * 0.6));
                let f = base.mix(c(Token::Surface).mix(c(Token::AccentSubtle), 0.52), sel_t);
                let b = c(Token::Border).mix(c(Token::BorderStrong), hv).mix(c(Token::Accent), sel_t);
                (f, b)
            };
            let tile_bg = if self.disabled { c(Token::SurfaceMuted) } else { c(Token::SurfaceMuted).mix(c(Token::AccentSubtle), sel_t) };
            let ic_c = if self.disabled { c(Token::TextDisabled) } else { c(Token::TextSecondary).mix(c(Token::AccentStrong), sel_t) };
            let tc = if self.disabled { c(Token::TextDisabled) } else { c(Token::Foreground) };
            let bc = if self.disabled { c(Token::TextDisabled) } else { c(Token::TextSecondary).mix(c(Token::AccentStrong), sel_t) };
            let dc = if self.disabled { c(Token::TextDisabled) } else { c(Token::TextSecondary) };
            (fill, border, tile_bg, ic_c, tc, bc, dc)
        };
        let border_w = 1.0 + 0.5 * sel_t;
        let ring = ui.color(Token::FocusRing);
        let r_sz = lk_cb.size;
        let r_fill = ui.color(Token::Control).mix(ui.color(Token::Accent), sel_t);
        let r_bd = ui.color(Token::BorderStrong).mix(Color::TRANSPARENT, sel_t);
        let r_mark = ui.color(Token::AccentForeground);
        let r_bw = lk_cb.border;

        ui.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::all(14.0), gap: 12.0, align: Align::Start, ..Default::default() },
            Some(id),
            Sense::CLICK,
            if self.disabled { CursorIcon::Default } else { CursorIcon::Pointer },
            false,
            Some(Box::new(move |cx, r| {
                let sr = Rect::from_center_size(r.center(), r.size() * (1.0 - 0.02 * press));
                cx.painter.rrect_unsnapped(sr, Corners::all(radius), [fill; 2], border_w, border);
                if ring_a > 0.0 {
                    cx.defer(sr, Box::new(move |cx, r| cx.focus_ring(r, radius, ring_a, ring_gap, ring)));
                }
            })),
        );

        if let Some(ic_name) = &self.icon {
            let name = ic_name.clone();
            ui.leaf(
                Style::default(),
                vec2(34.0, 34.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    cx.painter.rrect(r, Corners::all(8.0), tile_bg, 0.0, Color::TRANSPARENT);
                    cx.icon(&name, r.center(), 17.0, ic_c);
                })),
            );
        }

        ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 3.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let st_t = ui.text_style(TextRole::BodyStrong);
        ui.text_leaf(self.title.clone(), st_t, tc, false, true, Style { width: Size::FILL, ..Default::default() });
        if let Some(b) = &self.badge {
            let mut st_b = ui.text_style(TextRole::Caption);
            st_b.weight = 600;
            ui.text_leaf(b.clone(), st_b, bc, false, false, Style::default());
        }
        ui.end();

        if let Some(d) = &self.description {
            let st_d = ui.text_style(TextRole::Body);
            ui.text_leaf(d.clone(), st_d, dc, true, false, Style { width: Size::FILL, ..Default::default() });
        }
        ui.end();

        // Trailing radio indicator circle.
        ui.begin(Style { pad: Sides { top: 2.0, ..Default::default() }, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        ui.leaf(
            Style::default(),
            vec2(r_sz, r_sz),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let cen = r.center();
                cx.painter.circle(cen, r_sz / 2.0, r_fill, if r_bd.a > 0.0 { r_bw } else { 0.0 }, r_bd);
                if dot_t > 0.01 {
                    cx.painter.circle(cen, (r_sz * 0.2 * dot_t).max(0.0), r_mark, 0.0, Color::TRANSPARENT);
                }
            })),
        );
        ui.end();

        ui.end();
        let rect = ui.rect_of(id).unwrap_or(ix.rect);
        Response { id, rect, hovered: ix.hovered, clicked: act, changed, focused: ix.focused }
    }
}

impl<T: Copy + PartialEq> Drop for RadioCard<'_, T> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

// ================================================================== Rating

/// Interactive star rating (`0..=max`). Hovering previews the new score; stars
/// pop on the bouncy spring when chosen.
pub struct Rating<'u> {
    ui: &'u mut Ui,
    key: String,
    value: &'u mut u8,
    max: u8,
    icon: String,
    size: f32,
    disabled: bool,
    shown: bool,
}

impl<'u> Rating<'u> {
    pub fn max(mut self, m: u8) -> Self {
        self.max = m.clamp(3, 10);
        self
    }
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = name.to_owned();
        self
    }
    pub fn size(mut self, px: f32) -> Self {
        self.size = px.clamp(14.0, 36.0);
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }

    fn show_inner(&mut self) -> Response {
        let max = self.max;
        *self.value = (*self.value).min(max);
        let before = *self.value;

        let ui = &mut *self.ui;
        let id = ui.make_id(("rating", &self.key));
        let ix = ui.interact(id, Sense::CLICK, self.disabled);
        let mo = ui.motion();
        let (ring_a, ring_gap) = ui.focus_anim(&ix);

        let mut star_ixs = Vec::with_capacity(max as usize);
        let mut hovered_star: Option<u8> = None;
        for i in 1..=max {
            let sid = id.with(("star", i));
            let six = ui.interact(sid, Sense { click: true, drag: false, focusable: false, scroll: false }, self.disabled);
            if six.hovered && !self.disabled {
                hovered_star = Some(i);
            }
            if six.clicked && !self.disabled {
                *self.value = if *self.value == i { i - 1 } else { i };
            }
            star_ixs.push((i, six));
        }

        if ix.focused && !self.disabled {
            for (k, _, _) in ui.keys_for(id) {
                match k {
                    Key::ArrowRight | Key::ArrowUp => *self.value = (*self.value + 1).min(max),
                    Key::ArrowLeft | Key::ArrowDown => *self.value = self.value.saturating_sub(1),
                    Key::Home => *self.value = 0,
                    Key::End => *self.value = max,
                    _ => {}
                }
            }
        }

        let changed = *self.value != before;
        if changed {
            let target_idx = (*self.value).max(1);
            ui.anims.set_spring(id.with(("star", target_idx)), 2, 1.32);
        }

        let preview = hovered_star.unwrap_or(*self.value);
        let cell = self.size + 8.0;
        let sz = self.size;
        let ring = ui.color(Token::FocusRing);
        let rad = ui.look().control.radius[0].max(6.0);

        ui.begin(
            Style { dir: Dir::Row, gap: 2.0, align: Align::Center, ..Default::default() },
            Some(id),
            Sense::CLICK,
            if self.disabled { CursorIcon::Default } else { CursorIcon::Pointer },
            false,
            Some(Box::new(move |cx, r| {
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, rad, ring_a, ring_gap, ring)));
                }
            })),
        );

        let c_off = if self.disabled { ui.color(Token::TextDisabled) } else { ui.color(Token::BorderStrong) };
        let c_on = if self.disabled { ui.color(Token::TextDisabled) } else { ui.color(Token::Warning) };
        let c_prev = ui.color(Token::Warning).multiply_alpha(0.65);
        let bg_on = ui.color(Token::WarningSubtle);

        for (i, six) in star_ixs {
            let sid = six.id;
            let lit = i <= *self.value;
            let prev_lit = hovered_star.is_some() && i <= preview && !lit;
            let lit_t = ui.spring(sid, 0, if lit { 1.0 } else { 0.0 }, mo.snappy).clamp(0.0, 1.0);
            let hv_t = ui.fade(sid, 1, six.hovered && !self.disabled, mo.hover_in, mo.hover_out, Ease::Linear);
            let pop = ui.spring(sid, 2, if six.pressed { 0.86 } else if six.hovered { 1.10 } else { 1.0 }, mo.bouncy);
            let ic = c_off.mix(c_prev, if prev_lit { 1.0 } else { 0.0 }).mix(c_on, lit_t);
            let pill = bg_on.multiply_alpha((lit_t * 0.55 + hv_t * 0.35).clamp(0.0, 0.8));
            let icon_name = self.icon.clone();
            ui.leaf(
                Style::default(),
                vec2(cell, cell),
                Some(sid),
                Sense { click: true, drag: false, focusable: false, scroll: false },
                if self.disabled { CursorIcon::Default } else { CursorIcon::Pointer },
                Some(Box::new(move |cx, r| {
                    let sr = Rect::from_center_size(r.center(), r.size() * (0.92 + 0.08 * pop.min(1.1)));
                    if pill.a > 0.01 {
                        cx.painter.rrect_unsnapped(sr, Corners::all(6.0), [pill; 2], 0.0, Color::TRANSPARENT);
                    }
                    cx.icon(&icon_name, r.center(), sz * pop.max(0.5), ic);
                })),
            );
        }
        ui.end();

        let rect = ui.rect_of(id).unwrap_or(ix.rect);
        Response { id, rect, hovered: ix.hovered || hovered_star.is_some(), clicked: changed, changed, focused: ix.focused }
    }
}

show_on_drop!(Rating);

// ============================================================= RangeSlider

#[derive(Default)]
struct RangeDrag {
    which: u8, // 0 = lo, 1 = hi
    tracking: bool,
}

/// Dual-thumb interval slider over `range` (`values.0 <= values.1`). Both
/// thumbs are keyboard-focusable (`Tab` moves low → high) and follow the
/// current look's thumb, halo and inner-dot styling.
pub struct RangeSlider<'u> {
    ui: &'u mut Ui,
    label: String,
    values: &'u mut (f32, f32),
    range: RangeInclusive<f32>,
    step: Option<f32>,
    readout: Option<String>,
    width: Option<f32>,
    disabled: bool,
    shown: bool,
}

impl<'u> RangeSlider<'u> {
    pub fn step(mut self, s: f32) -> Self {
        self.step = Some(s);
        self
    }
    pub fn readout(mut self, s: impl Into<String>) -> Self {
        self.readout = Some(s.into());
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w.max(140.0));
        self
    }
    pub fn fill_width(mut self) -> Self {
        self.width = Some(f32::INFINITY);
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }

    fn show_inner(&mut self) -> Response {
        let (r_min, r_max) = (*self.range.start(), *self.range.end());
        let step_opt = self.step;
        let quantize = |v: f32| -> f32 {
            let v = v.clamp(r_min.min(r_max), r_min.max(r_max));
            match step_opt {
                Some(s) if s > 0.0 => (r_min + ((v - r_min) / s).round() * s).clamp(r_min.min(r_max), r_min.max(r_max)),
                _ => v,
            }
        };
        let span = (r_max - r_min).abs().max(f32::EPSILON);
        let mut v_lo = quantize(self.values.0.min(self.values.1));
        let mut v_hi = quantize(self.values.1.max(self.values.0));
        let before = (v_lo, v_hi);

        let ui = &mut *self.ui;
        let id = ui.make_id(("range_slider", &self.label));
        let lo_id = id.with("lo");
        let hi_id = id.with("hi");
        let lk = ui.look().slider;
        let mo = ui.motion();
        let half = lk.thumb / 2.0;

        // Track drag + two focusable thumb ids.
        let track_ix = ui.interact(id, Sense { click: true, drag: true, focusable: false, scroll: false }, self.disabled);
        let lo_ix = ui.interact(lo_id, Sense::CLICK, self.disabled);
        let hi_ix = ui.interact(hi_id, Sense::CLICK, self.disabled);

        let mut jump = false;
        if !self.disabled {
            let r = track_ix.rect;
            let (x0, x1) = (r.min.x + half, r.max.x - half);
            let from_x = |x: f32| r_min + ((x - x0) / (x1 - x0).max(1.0)).clamp(0.0, 1.0) * (r_max - r_min);
            if track_ix.press_started {
                if let Some(p) = track_ix.pointer {
                    let x_lo = x0 + ((v_lo - r_min) / span) * (x1 - x0);
                    let x_hi = x0 + ((v_hi - r_min) / span) * (x1 - x0);
                    let pick_hi = (p.x - x_hi).abs() < (p.x - x_lo).abs() || ((p.x - x_hi).abs() == (p.x - x_lo).abs() && p.x > x_hi);
                    let which = if pick_hi { 1 } else { 0 };
                    let thumb_x = if pick_hi { x_hi } else { x_lo };
                    jump = (p.x - thumb_x).abs() > half;
                    let st = ui.mem::<RangeDrag>(id);
                    st.which = which;
                    st.tracking = !jump;
                    let nv = from_x(p.x);
                    if which == 0 {
                        v_lo = nv.min(v_hi);
                        ui.request_focus(lo_id);
                    } else {
                        v_hi = nv.max(v_lo);
                        ui.request_focus(hi_id);
                    }
                }
            } else if track_ix.held {
                if let Some(p) = track_ix.pointer {
                    let which = ui.mem::<RangeDrag>(id).which;
                    ui.mem::<RangeDrag>(id).tracking = true;
                    let nv = from_x(p.x);
                    if which == 0 {
                        v_lo = nv.min(v_hi);
                    } else {
                        v_hi = nv.max(v_lo);
                    }
                }
            }

            let step = self.step.unwrap_or(span / 100.0);
            for (tid, focused, is_hi) in [(lo_id, lo_ix.focused, false), (hi_id, hi_ix.focused, true)] {
                if !focused {
                    continue;
                }
                for (k, m, _) in ui.keys_for(tid) {
                    let mult = if m.shift { 10.0 } else { 1.0 };
                    let (lo_bound, hi_bound) = if is_hi { (v_lo, r_max) } else { (r_min, v_hi) };
                    let cur = if is_hi { &mut v_hi } else { &mut v_lo };
                    match k {
                        Key::ArrowLeft | Key::ArrowDown => *cur -= step * mult,
                        Key::ArrowRight | Key::ArrowUp => *cur += step * mult,
                        Key::Home => *cur = lo_bound,
                        Key::End => *cur = hi_bound,
                        _ => continue,
                    }
                    jump = true;
                }
                if is_hi {
                    v_hi = v_hi.max(v_lo);
                } else {
                    v_lo = v_lo.min(v_hi);
                }
            }
        }

        v_lo = quantize(v_lo).min(quantize(v_hi));
        v_hi = quantize(v_hi).max(v_lo);
        *self.values = (v_lo, v_hi);
        let changed = *self.values != before;

        let n_lo = ((v_lo - r_min) / span).clamp(0.0, 1.0);
        let n_hi = ((v_hi - r_min) / span).clamp(0.0, 1.0);
        let active_which = ui.mem::<RangeDrag>(id).which;
        let tracking = track_ix.held && ui.mem::<RangeDrag>(id).tracking && !jump && self.step.is_none();

        let p_lo = if tracking && active_which == 0 {
            ui.anims.set_spring(id, 0, n_lo);
            n_lo
        } else {
            ui.spring(id, 0, n_lo, mo.snappy)
        };
        let p_hi = if tracking && active_which == 1 {
            ui.anims.set_spring(id, 1, n_hi);
            n_hi
        } else {
            ui.spring(id, 1, n_hi, mo.snappy)
        };

        let ptr = ui.pointer();
        let prev_r = track_ix.rect;
        let (px0, px1) = (prev_r.min.x + half, prev_r.max.x - half);
        let hot_lo = !self.disabled && ((track_ix.held && active_which == 0) || (track_ix.hovered && ptr.is_some_and(|p| (p.x - (px0 + p_lo * (px1 - px0))).abs() <= half + 6.0)));
        let hot_hi = !self.disabled && ((track_ix.held && active_which == 1) || (track_ix.hovered && ptr.is_some_and(|p| (p.x - (px0 + p_hi * (px1 - px0))).abs() <= half + 6.0)));

        let sz_lo = ui.spring(lo_id, 1, if hot_lo { lk.thumb + lk.hover_grow } else { lk.thumb }, if hot_lo { mo.bouncy } else { mo.snappy });
        let sz_hi = ui.spring(hi_id, 1, if hot_hi { lk.thumb + lk.hover_grow } else { lk.thumb }, if hot_hi { mo.bouncy } else { mo.snappy });
        let halo_lo = ui.fade(lo_id, 2, track_ix.held && active_which == 0 && !self.disabled, 0.06, 0.16, Ease::Linear) * lk.halo;
        let halo_hi = ui.fade(hi_id, 2, track_ix.held && active_which == 1 && !self.disabled, 0.06, 0.16, Ease::Linear) * lk.halo;
        let (ring_lo_a, ring_lo_gap) = ui.focus_anim(&lo_ix);
        let (ring_hi_a, ring_hi_gap) = ui.focus_anim(&hi_ix);

        let c = |t| ui.color(t);
        let (rail_c, fill_c, knob, knob_border, readout_c) = if self.disabled {
            (c(Token::Border), c(Token::TextDisabled), c(Token::SurfaceMuted), c(Token::BorderStrong), c(Token::TextDisabled))
        } else {
            let rc = if track_ix.held { c(Token::Foreground) } else { c(Token::TextSecondary) };
            let kb = c(Token::BorderStrong).multiply_alpha(0.6 * lk.thumb_border);
            let k = c(Token::Knob).mix(c(Token::Control), lk.inner_dot * ui.theme.darkness());
            (c(Token::Track), c(Token::Accent), k, kb, rc)
        };
        let dot_c = if self.disabled { c(Token::TextDisabled) } else { c(Token::Accent) };
        let inner_dot = lk.inner_dot;
        let track_h = lk.track;
        let shadows = if self.disabled { vec![] } else { Elevation::Knob.shadows(ui.theme.darkness(), c(Token::ShadowInk)) };
        let halo_c = c(Token::FocusHalo);
        let ring_c = c(Token::FocusRing);

        let width = self.width.unwrap_or(260.0);
        let fill = !width.is_finite();
        let fw = if fill { Size::FILL } else { Size::Hug };

        ui.begin(Style { gap: 6.0, width: fw, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if !self.label.is_empty() || self.readout.is_some() {
            ui.begin(Style { dir: Dir::Row, width: fw, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            if !self.label.is_empty() {
                let lc = if self.disabled { ui.color(Token::TextDisabled) } else { ui.color(Token::Foreground) };
                let st = ui.text_style(TextRole::Label);
                ui.text_leaf(self.label.clone(), st, lc, false, true, Style::default());
            }
            ui.spacer();
            if let Some(ro) = self.readout.take() {
                let mut st = ui.text_style(TextRole::Label);
                st.tabular = true;
                let n = ui.text_leaf(ro, st, readout_c, false, false, Style::default());
                ui.set_text_align(n, TextAlign::End);
            }
            ui.end();
        }

        // Hidden zero-hit focus anchors keep `lo_id` and `hi_id` alive in `prev_rects` for Tab navigation.
        let h_row = (lk.thumb + 6.0).max(22.0);
        ui.begin(Style { dir: Dir::Stack, width: if fill { Size::FILL } else { Size::Fixed(width) }, height: Size::Fixed(h_row), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        ui.leaf(Style { width: Size::Fixed(1.0), height: Size::Fixed(1.0), ..Default::default() }, vec2(1.0, 1.0), Some(lo_id), Sense { click: false, drag: false, focusable: true, scroll: false }, CursorIcon::Default, None);
        ui.leaf(Style { width: Size::Fixed(1.0), height: Size::Fixed(1.0), ..Default::default() }, vec2(1.0, 1.0), Some(hi_id), Sense { click: false, drag: false, focusable: true, scroll: false }, CursorIcon::Default, None);
        ui.leaf(
            Style { width: Size::FILL, height: Size::FILL, ..Default::default() },
            vec2(if fill { 140.0 } else { width }, h_row),
            Some(id),
            Sense { click: true, drag: true, focusable: false, scroll: false },
            if self.disabled { CursorIcon::Default } else { CursorIcon::Pointer },
            Some(Box::new(move |cx, r| {
                let tr = track_h / 2.0;
                let rail_r = cx.painter.snap_rect(Rect::from_xywh(r.min.x, r.center().y - tr, r.width(), track_h));
                cx.painter.rrect(rail_r, Corners::all(tr), rail_c, 0.0, Color::TRANSPARENT);
                let (x0, x1) = (r.min.x + half, r.max.x - half);
                let cx_lo = x0 + p_lo * (x1 - x0);
                let cx_hi = x0 + p_hi * (x1 - x0);
                let band = Rect::from_xywh(cx_lo.min(cx_hi), rail_r.min.y, (cx_hi - cx_lo).abs().max(2.0), rail_r.height());
                cx.painter.rrect_unsnapped(band, Corners::all(tr), [fill_c; 2], 0.0, Color::TRANSPARENT);

                for (cxp, sz, hl, ra, rg) in [(cx_lo, sz_lo, halo_lo, ring_lo_a, ring_lo_gap), (cx_hi, sz_hi, halo_hi, ring_hi_a, ring_hi_gap)] {
                    let center = vec2(cxp, r.center().y);
                    if hl > 0.0 {
                        cx.painter.circle(center, half + 5.0, halo_c.multiply_alpha(hl), 0.0, Color::TRANSPARENT);
                    }
                    let tr_k = Rect::from_center_size(center, vec2(sz, sz));
                    cx.painter.shadows(tr_k, Corners::all(sz / 2.0), &shadows);
                    let bw = if knob_border.a > 0.0 { 1.0 } else { 0.0 };
                    cx.painter.circle(center, sz / 2.0, knob, bw, knob_border);
                    if inner_dot > 0.0 {
                        cx.painter.circle(center, sz * 0.3, dot_c.multiply_alpha(inner_dot), 0.0, Color::TRANSPARENT);
                    }
                    if ra > 0.0 {
                        cx.defer(tr_k, Box::new(move |cx, r| cx.focus_ring(r, r.width() / 2.0, ra, rg, ring_c)));
                    }
                }
            })),
        );
        ui.end();
        ui.end();

        let rect = ui.rect_of(id).unwrap_or(track_ix.rect);
        Response { id, rect, hovered: track_ix.hovered, clicked: false, changed, focused: lo_ix.focused || hi_ix.focused }
    }
}

show_on_drop!(RangeSlider);

// ======================================================= ShortcutRecorder

#[derive(Default)]
struct RecMem {
    recording: bool,
}

fn format_key(k: Key) -> Option<String> {
    match k {
        Key::Char(c) => Some(c.to_ascii_uppercase().to_string()),
        Key::Enter => Some("↵".into()),
        Key::Space => Some("Space".into()),
        Key::ArrowUp => Some("↑".into()),
        Key::ArrowDown => Some("↓".into()),
        Key::ArrowLeft => Some("←".into()),
        Key::ArrowRight => Some("→".into()),
        Key::Home => Some("Home".into()),
        Key::End => Some("End".into()),
        Key::PageUp => Some("PgUp".into()),
        Key::PageDown => Some("PgDn".into()),
        Key::Delete => Some("⌦".into()),
        Key::Tab | Key::Escape | Key::Backspace => None,
    }
}

/// Interactive keybinding recorder: click or press Enter to arm recording,
/// press any chord (e.g. `⌘ + Shift + K`) to capture it as sculpted keycaps,
/// `Backspace` to clear, or `Escape` to cancel.
pub struct ShortcutRecorder<'u> {
    ui: &'u mut Ui,
    key: String,
    chord: &'u mut Vec<String>,
    label: Option<String>,
    disabled: bool,
    width: Size,
    shown: bool,
}

impl<'u> ShortcutRecorder<'u> {
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Size::Fixed(w.max(160.0));
        self
    }
    pub fn fill_width(mut self) -> Self {
        self.width = Size::FILL;
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("shortcut_rec", &self.key));
        let clear_id = id.with("clear");
        let ix = ui.interact(id, Sense::CLICK, self.disabled);
        let mo = ui.motion();
        let m = ui.control_metrics(ControlSize::Md);

        let mut recording = ui.mem::<RecMem>(id).recording && ix.focused && !self.disabled;
        let mut changed = false;

        if ix.clicked && !self.disabled {
            recording = !recording;
        }
        if ix.focused && !self.disabled {
            for (k, mods, repeat) in ui.keys_for(id) {
                if repeat {
                    continue;
                }
                if !recording {
                    if matches!(k, Key::Enter | Key::Space) {
                        recording = true;
                    } else if k == Key::Backspace && !self.chord.is_empty() {
                        self.chord.clear();
                        changed = true;
                    }
                    continue;
                }
                match k {
                    Key::Escape => recording = false,
                    Key::Backspace => {
                        if !self.chord.is_empty() {
                            self.chord.clear();
                            changed = true;
                        }
                        recording = false;
                    }
                    _ => {
                        if let Some(main_k) = format_key(k) {
                            let mut next = Vec::new();
                            if mods.ctrl && mods.mac {
                                next.push("⌃".into());
                            }
                            if mods.alt {
                                next.push(if mods.mac { "⌥".into() } else { "Alt".into() });
                            }
                            if mods.shift {
                                next.push(if mods.mac { "⇧".into() } else { "Shift".into() });
                            }
                            if mods.command() {
                                next.push(if mods.mac { "⌘".into() } else { "Ctrl".into() });
                            }
                            next.push(main_k);
                            if *self.chord != next {
                                *self.chord = next;
                                changed = true;
                            }
                            recording = false;
                        }
                    }
                }
            }
        }
        ui.mem::<RecMem>(id).recording = recording;

        let hv = ui.fade(id, 0, ix.hovered && !self.disabled, mo.hover_in, mo.hover_out, Ease::Linear);
        let rec_t = ui.fade(id, 1, recording, 0.08, 0.14, Ease::Linear);
        let halo_w = ui.spring(id, 2, if recording { 3.0 } else if ix.focus_visible { 2.0 } else { 0.0 }, mo.snappy);
        let (fill, border, halo_c, ic_c) = {
            let c = |t| ui.color(t);
            let fill = if self.disabled {
                c(Token::SurfaceMuted)
            } else {
                c(Token::Surface).mix(c(Token::Control).over(c(Token::Hover)), hv).mix(c(Token::AccentSubtle), rec_t * 0.35)
            };
            let border = if self.disabled {
                c(Token::BorderSubtle)
            } else {
                c(Token::Border).mix(c(Token::BorderStrong), hv).mix(c(Token::Accent), rec_t.max(if ix.focused { 0.7 } else { 0.0 }))
            };
            let ic_c = if self.disabled { c(Token::TextDisabled) } else { c(Token::TextMuted).mix(c(Token::AccentStrong), rec_t) };
            (fill, border, c(Token::FocusHalo), ic_c)
        };
        let radius = m.radius;

        let fill_w = matches!(self.width, Size::Fill(_));
        let fw = match self.width {
            Size::Hug => Size::Fixed(240.0),
            w => w,
        };

        ui.begin(Style { width: if fill_w { Size::FILL } else { Size::Hug }, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if let Some(l) = self.label.take() {
            ui.field_label(&l, self.disabled);
        }

        ui.begin(
            Style { dir: Dir::Row, width: fw, height: Size::Fixed(m.height), pad: Sides { left: 10.0, right: 6.0, top: 0.0, bottom: 0.0 }, gap: 8.0, align: Align::Center, ..Default::default() },
            Some(id),
            Sense::CLICK,
            if self.disabled { CursorIcon::Default } else { CursorIcon::Pointer },
            false,
            Some(Box::new(move |cx, r| {
                let r = cx.painter.snap_rect(r);
                if halo_w > 0.01 {
                    cx.painter.ring(r.expand(halo_w), Corners::all(radius + halo_w), halo_w, halo_c);
                }
                cx.painter.rrect(r, Corners::all(radius), fill, 1.0, border);
            })),
        );

        ui.icon_leaf("keyboard", 15.0, ic_c);

        if recording {
            let mut st = ui.text_style(TextRole::Body);
            st.weight = 500;
            let ac = ui.color(Token::AccentStrong);
            ui.text_leaf("Press keys…", st, ac, false, true, Style { width: Size::FILL, ..Default::default() });
        } else if self.chord.is_empty() {
            let st = ui.text_style(TextRole::Body);
            let mc = ui.color(Token::TextMuted);
            ui.text_leaf("Click to record", st, mc, false, true, Style { width: Size::FILL, ..Default::default() });
        } else {
            ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 4.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            for (i, part) in self.chord.iter().enumerate() {
                ui.scope(("part", i), |ui| {
                    ui.kbd(part);
                });
            }
            ui.end();
        }

        let show_clear = ui.spring(id, 4, if !self.chord.is_empty() && !recording && !self.disabled { 1.0 } else { 0.0 }, mo.bouncy);
        if show_clear > 0.02 {
            if ui.field_icon_button(clear_id, "x", show_clear, false) {
                self.chord.clear();
                changed = true;
            }
        }
        ui.end();
        ui.end();

        let rect = ui.rect_of(id).unwrap_or(ix.rect);
        Response { id, rect, hovered: ix.hovered, clicked: ix.clicked, changed, focused: ix.focused }
    }
}

show_on_drop!(ShortcutRecorder);

impl Ui {
    /// Selectable option card with icon, badge, description and radio indicator.
    pub fn radio_card<'u, T: Copy + PartialEq>(&'u mut self, value: &'u mut T, option: T, title: impl Into<String>) -> RadioCard<'u, T> {
        RadioCard { ui: self, value, option, title: title.into(), description: None, badge: None, icon: None, disabled: false, shown: false }
    }

    /// Interactive star rating (`0..=5` by default).
    pub fn rating<'u>(&'u mut self, key: impl Into<String>, value: &'u mut u8) -> Rating<'u> {
        Rating { ui: self, key: key.into(), value, max: 5, icon: "star".into(), size: 20.0, disabled: false, shown: false }
    }

    /// Dual-thumb range slider over `range`.
    pub fn range_slider<'u>(&'u mut self, label: impl Into<String>, values: &'u mut (f32, f32), range: RangeInclusive<f32>) -> RangeSlider<'u> {
        RangeSlider { ui: self, label: label.into(), values, range, step: None, readout: None, width: None, disabled: false, shown: false }
    }

    /// Keybinding recorder that captures modifier + key chords as keycaps.
    pub fn shortcut_recorder<'u>(&'u mut self, key: impl Into<String>, chord: &'u mut Vec<String>) -> ShortcutRecorder<'u> {
        ShortcutRecorder { ui: self, key: key.into(), chord, label: None, disabled: false, width: Size::Hug, shown: false }
    }
}
