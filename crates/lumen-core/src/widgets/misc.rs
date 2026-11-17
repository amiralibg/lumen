//! Radio buttons, progress, spinner and badges.

use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

use super::toast::Tone;

impl Ui {
    /// One radio button: selects `option` when clicked. Group several that
    /// share `value`.
    pub fn radio<T: Copy + PartialEq>(&mut self, value: &mut T, option: T, label: &str) -> Response {
        let id = self.make_id(("radio", label));
        let ix = self.interact(id, Sense::CLICK, false);
        let act = ix.clicked || (ix.focused && (self.key_pressed_for(id, Key::Space) || self.key_pressed_for(id, Key::Enter)));
        let changed = act && *value != option;
        if act {
            *value = option;
        }
        let on = *value == option;
        let mo = self.motion();
        let lk = self.look().checkbox;
        let (h, p) = self.state_weights(&ix, false);
        let sel = self.fade(id, 3, on, mo.selected, mo.selected, Ease::Out);
        let dot = self.spring(id, 4, if on { 1.0 } else { 0.0 }, mo.bouncy);
        let (ring_a, ring_gap) = self.focus_anim(&ix);
        let c = |t| self.color(t);
        let surf = c(Token::Control);
        let un_border = c(Token::BorderStrong).mix(c(Token::TextMuted), h.max(p));
        let on_fill = c(Token::Accent).mix(c(Token::AccentHover), h).mix(c(Token::AccentPressed), p);
        let fill = surf.mix(on_fill, sel);
        let border = un_border.mix(Color::TRANSPARENT, sel);
        let mark = c(Token::AccentForeground);
        let ring = c(Token::FocusRing);
        let body = self.text_style(TextRole::Body);
        let fg = c(Token::Foreground);
        let size = lk.size;
        let bw = lk.border;
        let row = self.begin(Style { dir: Dir::Row, gap: 8.0, align: Align::Center, ..Default::default() }, Some(id), Sense::CLICK, CursorIcon::Pointer, false, None);
        self.set_hit_pad(row, 4.0);
        self.leaf(
            Style::default(),
            vec2(size, size),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let c = r.center();
                cx.painter.circle(c, size / 2.0, fill, if border.a > 0.0 { bw } else { 0.0 }, border);
                if dot > 0.01 {
                    cx.painter.circle(c, (size * 0.19 * dot).max(0.0), mark, 0.0, Color::TRANSPARENT);
                }
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, size / 2.0, ring_a, ring_gap, ring)));
                }
            })),
        );
        if !label.is_empty() {
            self.text_leaf(label, body, fg, false, true, Style::default());
        }
        self.end();
        Response { id, rect: ix.rect, hovered: ix.hovered, clicked: act, changed, focused: ix.focused }
    }

    /// A progress bar. `None` shows an indeterminate sweep.
    pub fn progress(&mut self, value: Option<f32>) -> Response {
        let id = self.make_id("progress");
        let mo = self.motion();
        let v = value.map(|v| self.spring(id, 0, v.clamp(0.0, 1.0), mo.smooth));
        let track = self.color(Token::Track).multiply_alpha(0.7);
        let fill = self.color(Token::Accent);
        let time = self.time() as f32;
        if v.is_none() {
            self.request_repaint_after(0.0);
        }
        let reduced = self.reduced_motion();
        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(120.0, 6.0),
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(3.0), track, 0.0, Color::TRANSPARENT);
                let w = r.width();
                let (x0, x1) = match v {
                    Some(v) => (0.0, w * v),
                    None if reduced => (0.0, w * 0.4),
                    None => {
                        let t = (time * 0.8).fract();
                        let e = |x: f32| x * x * (3.0 - 2.0 * x);
                        (w * (e(t) * 1.3 - 0.3).max(0.0), w * (e(t) * 1.3).min(1.0))
                    }
                };
                if x1 > x0 {
                    let fr = Rect::from_xywh(r.min.x + x0, r.min.y, x1 - x0, r.height());
                    cx.painter.push_clip(r);
                    cx.painter.rrect_unsnapped(fr, Corners::all(3.0), [fill; 2], 0.0, Color::TRANSPARENT);
                    cx.painter.pop_clip();
                }
            })),
        );
        Response { id, ..Default::default() }
    }

    /// A spinning activity indicator.
    pub fn spinner(&mut self, size: f32) {
        let c = self.color(Token::Accent);
        let track = self.color(Token::Track).multiply_alpha(0.6);
        let t = self.time() as f32;
        self.request_repaint_after(0.0);
        self.leaf(
            Style::default(),
            vec2(size, size),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let w = (size / 8.0).max(2.0);
                let (ro, ri) = (size / 2.0, size / 2.0 - w);
                cx.painter.arc_band(r.center(), ri, ro, 0.0, std::f32::consts::TAU, track);
                let a = t * 5.0;
                let len = 1.2 + 0.9 * (t * 2.3).sin();
                cx.painter.arc_band(r.center(), ri, ro, a, a + len, c);
            })),
        );
    }

    /// A small pill label.
    pub fn badge(&mut self, text: impl Into<String>, tone: Tone) {
        let (bg, fg, _) = tone.tokens();
        let bg = self.color(bg);
        let fg = self.color(fg);
        let mut st = self.text_style(TextRole::Caption);
        st.weight = 500;
        self.begin(
            Style { dir: Dir::Row, height: Size::Fixed(20.0), pad: Sides::xy(7.0, 0.0), align: Align::Center, justify: Justify::Center, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(10.0), bg, 0.0, Color::TRANSPARENT))),
        );
        self.text_leaf(text, st, fg, false, false, Style::default());
        self.end();
    }

    /// An inline status dot with a label.
    pub fn status(&mut self, text: impl Into<String>, tone: Tone) {
        let (_, fg, solid) = tone.tokens();
        let dot = self.color(solid);
        let st = self.text_style(TextRole::Body);
        let c = self.color(Token::Foreground);
        let _ = fg;
        self.begin(Style { dir: Dir::Row, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.leaf(Style::default(), vec2(8.0, 8.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| {
            cx.painter.circle(r.center(), 4.0, dot.with_alpha(0.25), 0.0, Color::TRANSPARENT);
            cx.painter.circle(r.center(), 2.75, dot, 0.0, Color::TRANSPARENT);
        })));
        self.text_leaf(text, st, c, false, true, Style::default());
        self.end();
    }
}
