//! Circular time dial: a clock face with a hand you drag (or tap) to a
//! number. Pick the hour, and the face turns into minutes; the header's
//! hour and minute tiles switch back and forth, and AM/PM sits beside them.
//! The hand springs the short way round, numerals crossfade between hours
//! and minutes, and the chosen number sits inside the accent knob.

use std::f32::consts::TAU;

use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::date::Time;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Style};
use crate::math::{vec2, Corners, Rect, Sides, Vec2};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

use super::show_on_drop;

#[derive(Default)]
struct DialMem {
    minutes: bool,
    /// Hand angle target in turns, unwrapped so the spring takes the
    /// short way round.
    turns: f32,
    init: bool,
    dragging: bool,
}

pub struct TimeDial<'u> {
    ui: &'u mut Ui,
    key: String,
    time: &'u mut Time,
    step: u8,
    diameter: Option<f32>,
    shown: bool,
}

impl TimeDial<'_> {
    /// Minutes snap to this step (1 to 30, default 5).
    pub fn step(mut self, minutes: u8) -> Self {
        self.step = minutes;
        self
    }
    /// Diameter of the face (default: seven control heights).
    pub fn diameter(mut self, d: f32) -> Self {
        self.diameter = Some(d);
        self
    }
    fn show_inner(&mut self) -> Response {
        self.ui.time_dial_impl(&self.key, self.time, self.step, self.diameter)
    }
}

show_on_drop!(TimeDial);

impl Ui {
    /// A clock face for `time` (12-hour with AM/PM). `changed` is true when
    /// the time changed.
    pub fn time_dial<'u>(&'u mut self, key: impl Into<String>, time: &'u mut Time) -> TimeDial<'u> {
        TimeDial { ui: self, key: key.into(), time, step: 5, diameter: None, shown: false }
    }

    fn time_dial_impl(&mut self, key: &str, time: &mut Time, minute_step: u8, diameter: Option<f32>) -> Response {
        let id = self.make_id(("time_dial", key));
        let step = minute_step.clamp(1, 30) as i32;
        let old = *time;
        let mo = self.motion();
        let reduced = self.reduced_motion();
        let lk = self.look().clone();
        let dial_id = id.with("dial");
        let sense = Sense { click: true, drag: true, focusable: true, scroll: true };
        let ix = self.interact(dial_id, sense, false);
        let mut minutes = self.mem::<DialMem>(id).minutes;
        let (mut h, mut m) = (time.hour as i32, time.minute as i32);
        let pm = h >= 12;

        // Pointer: the angle from the center picks a number.
        let pick = |p: Vec2, r: Rect| -> f32 {
            let d = p - r.center();
            let a = d.x.atan2(-d.y);
            (if a < 0.0 { a + TAU } else { a }) / TAU
        };
        let mut released_on_hours = false;
        if let (true, Some(p)) = (ix.held || ix.press_started, ix.pointer) {
            if ix.rect.width() > 0.0 {
                let t = pick(p, ix.rect);
                if minutes {
                    m = (((t * 60.0) / step as f32).round() as i32 * step).rem_euclid(60);
                } else {
                    let k = (t * 12.0).round() as i32 % 12;
                    h = k + if pm { 12 } else { 0 };
                }
                self.mem::<DialMem>(id).dragging = true;
            }
        } else if self.mem::<DialMem>(id).dragging {
            self.mem::<DialMem>(id).dragging = false;
            released_on_hours = !minutes;
        }
        let dragging = self.mem::<DialMem>(id).dragging;

        if ix.focused {
            for (k, mods, _) in self.keys_for(dial_id) {
                if mods.command() {
                    continue;
                }
                let d = match k {
                    Key::ArrowUp | Key::ArrowRight => 1,
                    Key::ArrowDown | Key::ArrowLeft => -1,
                    Key::PageUp => 5,
                    Key::PageDown => -5,
                    Key::Enter | Key::Space => {
                        minutes = !minutes;
                        0
                    }
                    _ => 0,
                };
                if d != 0 {
                    if minutes {
                        m = (m + d * step).rem_euclid(60);
                    } else {
                        h = (h + d).rem_euclid(24);
                    }
                }
            }
        }
        let (st, set) = self.a11y_step(dial_id);
        if minutes {
            if let Some(v) = set {
                m = (v.round() as i32).clamp(0, 59);
            }
            m = (m + st * step).rem_euclid(60);
        } else {
            if let Some(v) = set {
                h = (v.round() as i32).clamp(0, 23);
            }
            h = (h + st).rem_euclid(24);
        }
        m = (m / step) * step;
        if released_on_hours {
            minutes = true;
        }

        // Header: hour and minute tiles choose what the dial edits.
        let mut st_big = self.text_style(TextRole::Heading1);
        st_big.tabular = true;
        let st_ap = self.text_style(TextRole::Label);
        let gap = lk.control.gap[1].max(6.0);
        let tile_r = lk.control.radius[2];
        let accent = self.color(Token::Accent);
        let accent_fg = self.color(Token::AccentForeground);
        let acc_sub = self.color(Token::AccentSubtle);
        let muted_bg = self.color(Token::SurfaceMuted);
        let fg = self.color(Token::Foreground);
        let sec = self.color(Token::TextSecondary);
        let ring = self.color(Token::FocusRing);
        let face = self.color(Token::SurfaceMuted);
        let edge = self.color(Token::BorderSubtle);

        self.begin(Style { dir: Dir::Column, gap: gap * 2.0, align: Align::Center, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);
        self.begin(Style { dir: Dir::Row, gap, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let h12 = if h % 12 == 0 { 12 } else { h % 12 };
        let tiles = [(false, format!("{h12:02}"), "Hour", h12), (true, format!("{m:02}"), "Minute", m)];
        for (i, (is_min, text, name, v)) in tiles.into_iter().enumerate() {
            if i == 1 {
                let c = self.text_leaf(":".to_owned(), st_big, sec, false, false, Style::default());
                self.a11y_hide(c);
            }
            let tid = id.with(("tile", i));
            let tix = self.interact(tid, Sense::CLICK, false);
            let (act, held) = super::key_activate(self, &tix, true);
            if tix.clicked || act {
                minutes = is_min;
            }
            let on = self.fade(tid, 0, minutes == is_min, mo.hover_in, mo.hover_out, Ease::Linear);
            let (hov, press) = self.state_weights(&tix, held);
            let squash = if reduced { 1.0 } else { self.spring(tid, 1, if tix.pressed || held { 0.94 } else { 1.0 }, mo.bouncy) };
            let (ra, rg) = self.focus_anim(&tix);
            let hover_c = self.color(Token::Hover);
            let bg = muted_bg.mix(acc_sub, on).over(hover_c.multiply_alpha(hov * (1.0 - on) + press * 0.5));
            let col = fg.mix(accent, on);
            let g = self.galley(&text, &st_big);
            let pad = vec2(lk.control.pad_x[1], lk.control.pad_x[0] * 0.5);
            let size = vec2(g.width + pad.x * 2.0, g.height + pad.y * 2.0);
            self.leaf(
                Style::default(),
                size,
                Some(tid),
                Sense::CLICK,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    let r2 = Rect::from_center_size(r.center(), r.size() * squash);
                    cx.painter.rrect(r2, Corners::all(tile_r), bg, 0.0, Color::TRANSPARENT);
                    let gg = cx.galley(&text, &st_big);
                    cx.painter.text(cx.text, &gg, vec2((r.center().x - gg.width * 0.5).round(), (r.center().y - gg.height * 0.5).round()), col);
                    if ra > 0.0 {
                        cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr, tile_r, ra, rg, ring)));
                    }
                })),
            );
            let sel = minutes == is_min;
            self.a11y(tid, || A11y::new(Role::Button).label(format!("{name}, {v}")).selected(sel));
        }

        // AM / PM: two stacked radio pills.
        self.begin(Style { dir: Dir::Column, gap: 2.0, pad: Sides { left: gap * 0.5, ..Default::default() }, ..Default::default() }, Some(id.with("ampm")), Sense::NONE, CursorIcon::Default, false, None);
        for (i, label) in ["AM", "PM"].into_iter().enumerate() {
            let aid = id.with(("ap", i));
            let aix = self.interact(aid, Sense::CLICK, false);
            let (act, held) = super::key_activate(self, &aix, true);
            let want_pm = i == 1;
            if (aix.clicked || act) && (h >= 12) != want_pm {
                h = (h + 12) % 24;
            }
            let on = self.fade(aid, 0, (h >= 12) == want_pm, mo.hover_in, mo.hover_out, Ease::Linear);
            let (hov, press) = self.state_weights(&aix, held);
            let (ra, rg) = self.focus_anim(&aix);
            let hover_c = self.color(Token::Hover);
            let bg = Color::TRANSPARENT.mix(acc_sub, on).over(hover_c.multiply_alpha(hov * (1.0 - on) + press * 0.5));
            let col = sec.mix(accent, on);
            let g = self.galley(label, &st_ap);
            let size = vec2(g.width + lk.control.pad_x[0] * 1.5, g.height + lk.control.pad_x[0] * 0.6);
            let r_small = lk.control.radius[0];
            self.leaf(
                Style::default(),
                size,
                Some(aid),
                Sense::CLICK,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    cx.painter.rrect(r, Corners::all(r_small), bg, 0.0, Color::TRANSPARENT);
                    let gg = cx.galley(label, &st_ap);
                    cx.painter.text(cx.text, &gg, vec2((r.center().x - gg.width * 0.5).round(), (r.center().y - gg.height * 0.5).round()), col);
                    if ra > 0.0 {
                        cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr, r_small, ra, rg, ring)));
                    }
                })),
            );
            let sel = (h >= 12) == want_pm;
            self.a11y(aid, || A11y::new(Role::RadioButton).label(label).selected(sel).toggled(sel).position(i, 2));
        }
        self.end();
        let pm_now = h >= 12;
        self.a11y(id.with("ampm"), || A11y::new(Role::RadioGroup).label("AM or PM").value(if pm_now { "PM" } else { "AM" }));
        self.end();

        // The dial.
        self.mem::<DialMem>(id).minutes = minutes;
        let raw = if minutes { m as f32 / 60.0 } else { (h % 12) as f32 / 12.0 };
        let dm = self.mem::<DialMem>(id);
        if !dm.init {
            dm.turns = raw;
            dm.init = true;
        } else {
            let cur = dm.turns;
            let k = (cur - raw).round();
            dm.turns = raw + k;
        }
        let turns = dm.turns;
        if reduced {
            self.anims.set_spring(dial_id, 0, turns);
        }
        let hand = if dragging || reduced { turns } else { self.spring(dial_id, 0, turns, mo.smooth) };
        if dragging {
            self.anims.set_spring(dial_id, 0, turns);
        }
        let mode = if reduced { if minutes { 1.0 } else { 0.0 } } else { self.spring(dial_id, 1, if minutes { 1.0 } else { 0.0 }, mo.smooth) };
        let knob_pop = if reduced { 1.0 } else { self.spring(dial_id, 2, if dragging { 1.12 } else { 1.0 }, mo.bouncy) };
        let (ra, rg) = self.focus_anim(&ix);
        let mut st_num = self.text_style(TextRole::Body);
        st_num.tabular = true;
        let d = diameter.unwrap_or(lk.control.height[1] * 7.0).round();
        let knob_r = (lk.control.height[0] * 0.62).max(13.0);
        let on_minute_mark = m % 5 == 0;
        let hand_w = lk.focus.width.max(1.5);
        self.leaf(
            Style::default(),
            vec2(d, d),
            Some(dial_id),
            sense,
            CursorIcon::Pointer,
            Some(Box::new(move |cx, r| {
                let c = r.center();
                let rad = r.width() * 0.5;
                cx.painter.circle(c, rad, face, 1.0, edge);
                let lr = rad - knob_r * 1.3;
                let at = |t: f32, rr: f32| c + vec2((t * TAU).sin(), -(t * TAU).cos()) * rr;
                // Hand and knob.
                let k = at(hand, lr);
                cx.painter.polyline(&[c, k], hand_w, accent);
                cx.painter.circle(c, knob_r * 0.25, accent, 0.0, Color::TRANSPARENT);
                let kr = knob_r * knob_pop * if mode > 0.5 && !on_minute_mark { 0.6 } else { 1.0 };
                cx.painter.circle(k, kr, accent, 0.0, Color::TRANSPARENT);
                // Numerals: hours fade out and spread as minutes fade in.
                for (set, alpha, scale) in [(false, 1.0 - mode, 1.0 + 0.12 * mode), (true, mode, 0.88 + 0.12 * mode)] {
                    if alpha <= 0.01 {
                        continue;
                    }
                    for i in 0..12 {
                        let t = i as f32 / 12.0;
                        let label = if set { format!("{:02}", i * 5) } else { format!("{}", if i == 0 { 12 } else { i }) };
                        let p = at(t, lr * scale.min(1.0) + lr * (scale - 1.0).max(0.0));
                        let g = cx.galley(&label, &st_num);
                        let inside = (p - k).length() < kr;
                        let col = if inside { accent_fg } else { fg }.multiply_alpha(alpha);
                        cx.painter.text(cx.text, &g, vec2((p.x - g.width * 0.5).round(), (p.y - g.height * 0.5).round()), col);
                    }
                    if set {
                        // Minute marks between the numerals.
                        for i in 0..60 {
                            if i % 5 != 0 {
                                let p = at(i as f32 / 60.0, rad - knob_r * 0.35);
                                cx.painter.circle(p, hand_w * 0.5, sec.multiply_alpha(alpha * 0.7), 0.0, Color::TRANSPARENT);
                            }
                        }
                    }
                }
                if !on_minute_mark && mode > 0.5 {
                    cx.painter.circle(k, hand_w, accent_fg, 0.0, Color::TRANSPARENT);
                }
                if ra > 0.0 {
                    cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr, rr.width() * 0.5, ra, rg, ring)));
                }
            })),
        );
        let (name, v, max, stp) = if minutes { ("Minute", m, 59.0, step as f64) } else { ("Hour", h, 23.0, 1.0) };
        let shown_time = Time::new(h as u8, m as u8).format_12();
        self.a11y(dial_id, || A11y::new(Role::Slider).label(name).numeric(v as f64, 0.0, max, Some(stp)).description(shown_time));
        self.end();

        time.hour = h as u8;
        time.minute = m as u8;
        let changed = *time != old;
        Response { id, rect: self.rect_of(id).unwrap_or_default(), hovered: ix.hovered, clicked: false, changed, focused: ix.focused }
    }
}
