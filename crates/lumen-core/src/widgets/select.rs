//! Select: pick one value from a list in a popover.

use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Sides};
use crate::theme::{ControlSize, TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

use super::popover::PopState;

/// Builder for [`Ui::select`]. Shown on drop or with `.show()`.
pub struct Select<'u, 'o, T: Copy + PartialEq> {
    ui: &'u mut Ui,
    value: &'u mut T,
    options: &'o [(T, &'o str)],
    label: Option<String>,
    placeholder: String,
    width: Size,
    disabled: bool,
    shown: bool,
}

impl<T: Copy + PartialEq> Select<'_, '_, T> {
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
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
        let key: Vec<&str> = self.options.iter().map(|o| o.1).collect();
        let id = ui.make_id(("select", self.label.as_deref().unwrap_or(""), key));
        let ix = ui.interact(id, Sense::CLICK, self.disabled);
        let open = ui.mem::<PopState>(id).open;
        let key_open = ix.focused && !open && !self.disabled && (ui.key_pressed_for(id, Key::Space) || ui.key_pressed_for(id, Key::Enter) || ui.key_pressed_for(id, Key::ArrowDown));
        let m = ui.control_metrics(ControlSize::Md);
        let mo = ui.motion();
        let h = ui.fade(id, 1, ix.hovered && !self.disabled, mo.hover_in, mo.hover_out, Ease::Linear);
        let f = ui.fade(id, 2, open || ix.focused, mo.hover_in, mo.hover_out, Ease::Linear);
        let chev = ui.spring(id, 3, if open { 1.0 } else { 0.0 }, mo.snappy);
        let (ring_a, ring_gap) = ui.focus_anim(&ix);
        let lk = ui.look().input;
        let c = |t| ui.color(t);
        let (fill, border, fg) = if self.disabled {
            (c(Token::SurfaceMuted), c(Token::BorderSubtle), c(Token::TextDisabled))
        } else {
            let fill = c(Token::Control).mix(c(Token::Control).over(c(Token::Hover)), h);
            let cb = c(Token::ControlBorder);
            let b = if cb.a > 0.0 { cb.mix(c(Token::BorderStrong), h).mix(c(Token::FocusRing), f * lk.halo) } else { cb };
            (fill, b, c(Token::Foreground))
        };
        let muted = c(Token::TextMuted);
        let current = self.options.iter().find(|o| o.0 == *self.value).map(|o| o.1.to_owned());
        let (text, text_c) = match current {
            Some(t) => (t, fg),
            None => (self.placeholder.clone(), muted),
        };
        let icon_c = c(Token::TextMuted);
        let ring = c(Token::FocusRing);

        let radius = m.radius;
        let width = self.width;

        ui.begin(Style { width: if matches!(width, Size::Fill(_)) { Size::FILL } else { Size::Hug }, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if let Some(l) = self.label.take() {
            let st = ui.text_style(TextRole::Label);
            let lc = if self.disabled { ui.color(Token::TextDisabled) } else { ui.color(Token::Foreground) };
            ui.text_leaf(l, st, lc, false, true, Style::default());
        }
        let bw = if border.a > 0.0 { 1.0 } else { 0.0 };
        ui.begin(
            Style {
                dir: Dir::Row,
                width: match width {
                    Size::Hug => Size::Fixed(200.0),
                    w => w,
                },
                height: Size::Fixed(m.height),
                pad: Sides { left: m.pad_x, right: m.pad_x - 4.0, top: 0.0, bottom: 0.0 },
                gap: 8.0,
                align: Align::Center,
                ..Default::default()
            },
            Some(id),
            Sense::CLICK,
            if self.disabled { CursorIcon::Default } else { CursorIcon::Pointer },
            false,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(radius), fill, bw, border);
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, radius, ring_a, ring_gap, ring)));
                }
            })),
        );
        let st = ui.text_style(TextRole::Body);
        ui.text_leaf(text, st, text_c, false, true, Style { width: Size::FILL, ..Default::default() });
        let icon_sz = m.icon;
        ui.leaf(
            Style::default(),
            vec2(icon_sz, icon_sz),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                // The chevron flips as the list opens: fade between down and up.
                let c0 = icon_c.multiply_alpha(1.0 - chev.clamp(0.0, 1.0));
                let c1 = icon_c.multiply_alpha(chev.clamp(0.0, 1.0));
                cx.icon("chevron-down", r.center() + vec2(0.0, 2.0 * chev), icon_sz, c0);
                cx.icon("chevron-up", r.center() + vec2(0.0, 2.0 * (chev - 1.0)), icon_sz, c1);
            })),
        );
        ui.end();
        let resp = Response { id, rect: ix.rect, hovered: ix.hovered, clicked: (ix.clicked || key_open) && !self.disabled, changed: false, focused: ix.focused };
        if key_open {
            let st = ui.mem::<PopState>(id);
            st.open = true;
        }
        let mut chosen = None;
        let value = *self.value;
        let options = self.options;
        let kb = key_open;
        ui.popover(&Response { clicked: resp.clicked && !key_open, ..resp }).match_width().pad(4.0).show(|ui| {
            let items = ui.mem_get::<PopState>(id).map(|s| s.items.clone()).unwrap_or_default();
            let body = ui.text_style(TextRole::Body);
            let ch = (m.height - 4.0).max(24.0);
            let mut sel_id = None;
            for (i, &(v, label)) in options.iter().enumerate() {
                let oid = id.with(("opt", i));
                let oix = ui.interact(oid, Sense::CLICK, false);
                if oix.focused {
                    ui.arrow_focus(oid, &items);
                }
                let act = oix.clicked || (oix.focused && (ui.key_pressed_for(oid, Key::Enter) || ui.key_pressed_for(oid, Key::Space)));
                if act {
                    chosen = Some(v);
                    ui.close_layer();
                }
                let selected = v == value;
                if selected {
                    sel_id = Some(oid);
                }
                let hot = oix.hovered || oix.focused;
                let hh = ui.fade(oid, 1, hot, 0.0, mo.hover_out, Ease::Linear);
                let hl = ui.color(Token::Hover).over(ui.color(Token::Hover)).multiply_alpha(hh);
                let fg = ui.color(Token::Foreground);
                let check = ui.color(Token::AccentStrong);
                let r_opt = (radius - 2.0).max(0.0);
                ui.begin(
                    Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(ch), pad: Sides::xy(8.0, 0.0), gap: 8.0, align: Align::Center, ..Default::default() },
                    Some(oid),
                    Sense::CLICK,
                    CursorIcon::Pointer,
                    false,
                    Some(Box::new(move |cx, r| {
                        if hl.a > 0.0 {
                            cx.painter.rrect(r, Corners::all(r_opt), hl, 0.0, Color::TRANSPARENT);
                        }
                    })),
                );
                ui.text_leaf(label, body, fg, false, true, Style { width: Size::FILL, ..Default::default() });
                ui.leaf(
                    Style::default(),
                    vec2(16.0, 16.0),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, r| {
                        if selected {
                            cx.icon("check", r.center(), 15.0, check);
                        }
                    })),
                );
                ui.end();
            }
            // Keyboard opening lands on the selected option.
            if kb {
                if let Some(s) = sel_id {
                    ui.request_focus(s);
                }
            }
        });
        ui.end();
        let changed = chosen.is_some_and(|v| v != *self.value);
        if let Some(v) = chosen {
            *self.value = v;
        }
        Response { changed, ..resp }
    }
}

impl<T: Copy + PartialEq> Drop for Select<'_, '_, T> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

impl Ui {
    /// A dropdown select bound to `value`.
    pub fn select<'u, 'o, T: Copy + PartialEq>(&'u mut self, value: &'u mut T, options: &'o [(T, &'o str)]) -> Select<'u, 'o, T> {
        Select { ui: self, value, options, label: None, placeholder: "Select…".into(), width: Size::Hug, disabled: false, shown: false }
    }
}
