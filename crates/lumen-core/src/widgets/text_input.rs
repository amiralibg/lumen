//! Single-line text input (spec section 7).

use lumen_text::{TextAlign, TextStyle};

use super::edit::TextEdit;
use super::show_on_drop;
use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect};
use crate::theme::{ControlSize, TextRole, Token};
use crate::ui::{Response, Sense, TextFieldState, TextIn, Ui};
use crate::Color;

/// Persistent editing state of a text input.
#[derive(Default)]
pub struct EditState {
    pub edit: TextEdit,
    scroll: f32,
    last_activity: f64,
    last_error: String,
    dragging: bool,
}

pub struct TextInput<'u> {
    ui: &'u mut Ui,
    value: &'u mut String,
    label: Option<String>,
    placeholder: String,
    icon: Option<String>,
    helper: Option<String>,
    error: Option<String>,
    disabled: bool,
    width: Size,
    size: ControlSize,
    mono: bool,
    id_key: Option<String>,
    shown: bool,
}

impl<'u> TextInput<'u> {
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }
    /// Leading Lucide icon.
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = Some(name.to_owned());
        self
    }
    /// Helper text under the field (replaced by the error when present).
    pub fn helper(mut self, h: impl Into<String>) -> Self {
        self.helper = Some(h.into());
        self
    }
    /// Error message; `None` clears it.
    pub fn error(mut self, e: Option<impl Into<String>>) -> Self {
        self.error = e.map(Into::into);
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Size::Fixed(w.max(120.0));
        self
    }
    pub fn fill_width(mut self) -> Self {
        self.width = Size::FILL;
        self
    }
    pub fn size(mut self, s: ControlSize) -> Self {
        self.size = s;
        self
    }
    /// Show the value in Geist Mono.
    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }
    /// Explicit identity (defaults to the label or placeholder).
    pub fn id(mut self, key: impl Into<String>) -> Self {
        self.id_key = Some(key.into());
        self
    }

    fn text_style(&self, ui: &Ui) -> TextStyle {
        let mut st = ui.text_style(if self.mono { TextRole::Mono } else { TextRole::Body });
        let k = ui.look().text_scale;
        if self.size == ControlSize::Lg && !self.mono {
            st.size = (14.0 * k * 4.0).round() / 4.0;
        }
        st.line_height = (20.0 * k).round();
        st
    }

    fn show_inner(&mut self) -> Response {
        let key = self.id_key.clone().or_else(|| self.label.clone()).unwrap_or_else(|| self.placeholder.clone());
        let style = self.text_style(self.ui);
        let ui = &mut *self.ui;
        let id = ui.make_id(("text_input", key));
        let mut ix = ui.interact(id, Sense { click: true, drag: true, focusable: true, scroll: false }, self.disabled);
        let m = ui.control_metrics(self.size);
        let lk = ui.look().input;
        let mo = ui.motion();
        let now = ui.time();
        let text_left = if self.icon.is_some() { 34.0 } else { 12.0 };
        let old = self.value.clone();

        // Pointer and keyboard editing.
        let mut copied = None;
        let shift = ui.modifiers().shift;
        if !self.disabled {
            let galley = ui.galley(self.value, &style);
            let st = ui.mem::<EditState>(id);
            st.edit.clamp(self.value);
            let origin_x = ix.rect.min.x + text_left - st.scroll;
            let idx_at = |x: f32| galley.index_for_x(x - origin_x);
            if ix.press_started {
                if let Some(p) = ix.pointer {
                    let i = idx_at(p.x);
                    match ix.click_count {
                        2 => st.edit.select_word(self.value, i),
                        n if n >= 3 => st.edit.select_all(self.value),
                        _ => st.edit.click(i, shift),
                    }
                    st.dragging = ix.click_count == 1;
                    st.last_activity = now;
                }
            } else if ix.held && st.dragging {
                if let Some(p) = ix.pointer {
                    st.edit.click(idx_at(p.x), true);
                }
            } else {
                st.dragging = false;
            }
        }
        if ix.focused && !self.disabled {
            ui.set_wants_text_input();
            let keys = ui.keys_for(id);
            let inputs = ui.take_text_ops(id);
            let st = ui.mem::<EditState>(id);
            let mut activity = !keys.is_empty() || !inputs.is_empty();
            for t in inputs {
                match t {
                    TextIn::Text(t) => st.edit.insert(self.value, &t),
                    TextIn::Ime(e) => {
                        st.edit.ime(self.value, &e);
                    }
                }
            }
            for (k, mods, _) in keys {
                let ext = mods.shift;
                let word = mods.word();
                let cmd = mods.command();
                match k {
                    Key::Backspace => st.edit.backspace(self.value, word, cmd),
                    Key::Delete => st.edit.delete(self.value, word),
                    Key::ArrowLeft if cmd => st.edit.home(ext),
                    Key::ArrowRight if cmd => st.edit.end(self.value, ext),
                    Key::ArrowLeft => st.edit.left(self.value, ext, word),
                    Key::ArrowRight => st.edit.right(self.value, ext, word),
                    Key::ArrowUp | Key::Home => st.edit.home(ext),
                    Key::ArrowDown | Key::End => st.edit.end(self.value, ext),
                    Key::Char('a') if cmd => st.edit.select_all(self.value),
                    Key::Char('c') if cmd => {
                        if st.edit.has_selection() {
                            copied = Some(st.edit.selected(self.value).to_owned());
                        }
                    }
                    Key::Char('x') if cmd => {
                        if st.edit.has_selection() {
                            copied = Some(st.edit.selected(self.value).to_owned());
                            st.edit.backspace(self.value, false, false);
                        }
                    }
                    Key::Char('z') if cmd && ext => st.edit.redo(self.value),
                    Key::Char('z') if cmd => st.edit.undo(self.value),
                    Key::Char('y') if mods.ctrl && !mods.mac => st.edit.redo(self.value),
                    _ => activity = activity && !matches!(k, Key::Tab),
                }
            }
            if activity {
                st.last_activity = now;
            }
        }
        if let Some(c) = copied {
            ui.set_copied_text(c);
        }
        if !self.disabled {
            for a in ui.a11y_actions(id) {
                if let crate::a11y::A11yAction::SetValue(v) = a {
                    let st = ui.mem::<EditState>(id);
                    st.edit.select_all(self.value);
                    st.edit.insert(self.value, &v);
                }
            }
        }
        ui.a11y(id, || {
            let label = self.label.clone().unwrap_or_else(|| self.placeholder.clone());
            A11y::new(Role::TextInput)
                .label(label)
                .value(self.value.as_str())
                .placeholder(self.placeholder.as_str())
                .disabled(self.disabled)
                .invalid(self.error.is_some())
                .description(self.error.clone().or_else(|| self.helper.clone()).unwrap_or_default())
        });
        if !ix.focused {
            // Collapse the selection when focus leaves.
            if let Some(st) = ui.mem_get::<EditState>(id) {
                if st.edit.has_selection() || st.edit.marked.is_some() {
                    let st = ui.mem::<EditState>(id);
                    st.edit.anchor = st.edit.cursor;
                    st.edit.marked = None;
                }
            }
        }
        let changed = *self.value != old;

        // Visual state.
        let hovered = ix.hovered && !self.disabled;
        let h = ui.fade(id, 1, hovered, 0.06, 0.16, Ease::Linear);
        let f = ui.fade(id, 2, ix.focused, 0.06, 0.16, Ease::Linear);
        let halo_w = ui.spring(id, 3, if ix.focused { 3.0 * lk.halo } else { 0.0 }, mo.snappy);
        let halo_a = ui.fade(id, 4, ix.focused, 0.0, 0.1, Ease::Linear);
        let has_error = self.error.is_some();
        let c = |t| ui.color(t);
        let (fill, border, text_c, icon_c, ph_c) = if self.disabled {
            (c(Token::SurfaceMuted), c(Token::BorderSubtle), c(Token::TextDisabled), c(Token::TextDisabled), c(Token::TextDisabled))
        } else {
            let b = if has_error {
                c(Token::Danger)
            } else {
                c(Token::Border).mix(c(Token::BorderStrong), h).mix(c(Token::FocusRing), f)
            };
            // GNOME fills the field and drops the outline; Windows fills it
            // with the control color until focused.
            let rest = c(Token::Surface).mix(c(Token::Control), lk.filled.max(lk.underline * (1.0 - f)));
            let fill = rest.mix(c(Token::Control).mix(c(Token::Control).over(c(Token::Hover)), h), lk.filled * (1.0 - f));
            let b = if has_error { b } else { b.mix(c(Token::FocusRing).multiply_alpha(f), lk.filled) };
            (fill, b, c(Token::Foreground), c(Token::TextMuted).mix(c(Token::TextSecondary), f), c(Token::TextMuted))
        };
        let border_w = 1.0 + lk.filled * f;
        let under_c = if has_error { c(Token::Danger) } else { c(Token::BorderStrong).mix(c(Token::Accent), f) };
        let under_h = (1.0 + f) * lk.underline;
        let halo_c = if has_error { c(Token::Danger).with_alpha(c(Token::FocusHalo).a) } else { c(Token::FocusHalo) };
        let sel_c = c(Token::Selection);
        let caret_c = c(Token::FocusRing);
        let radius = m.radius;

        // Caret blink (solid 500ms after activity, then 500 on / 500 off).
        let since = (now - ui.mem::<EditState>(id).last_activity) as f32;
        let caret_a = if !ix.focused || self.disabled {
            0.0
        } else if since < 0.5 || ui.reduced_motion() || !ui.caret_blink {
            if !ui.reduced_motion() && ui.caret_blink {
                ui.request_repaint_after((0.5 - since) as f64);
            }
            1.0
        } else {
            let t = (since - 0.5) % 1.0;
            let (a, next) = if t < 0.08 {
                (1.0 - t / 0.08, 0.0)
            } else if t < 0.5 {
                (0.0, 0.5 - t)
            } else if t < 0.58 {
                ((t - 0.5) / 0.08, 0.0)
            } else {
                (1.0, 1.0 - t)
            };
            ui.request_repaint_after(next as f64);
            a
        };

        // Keep the caret visible (horizontal scroll).
        let galley = ui.galley(self.value, &style);
        let field_w_prev = ix.rect.width();
        let visible_w = (field_w_prev - text_left - 12.0).max(1.0);
        let (cursor, sel) = {
            let st = ui.mem::<EditState>(id);
            st.edit.clamp(self.value);
            let caret_x = galley.x_for_index(st.edit.cursor);
            if galley.width <= visible_w {
                st.scroll = 0.0;
            } else {
                if caret_x - st.scroll > visible_w {
                    st.scroll = caret_x - visible_w;
                }
                if caret_x - st.scroll < 0.0 {
                    st.scroll = caret_x;
                }
                st.scroll = st.scroll.clamp(0.0, (galley.width - visible_w + 2.0).max(0.0));
            }
            (st.edit.cursor, st.edit.selection())
        };
        let scroll = ui.mem::<EditState>(id).scroll;
        let marked = ui.mem::<EditState>(id).edit.marked.clone();
        if ix.focused && !self.disabled {
            // Tell the host's input method where the caret or composition is.
            let x0 = ix.rect.min.x + text_left - scroll;
            let top = ix.rect.center().y - style.line_height / 2.0;
            let span = marked.clone().unwrap_or(cursor..cursor);
            let (a, b) = (galley.x_for_index(span.start), galley.x_for_index(span.end));
            let anchor = ui.mem::<EditState>(id).edit.anchor;
            ui.report_text_field(TextFieldState {
                text: self.value.clone(),
                cursor,
                anchor,
                marked: marked.clone(),
                rect: Rect::from_xywh(x0 + a, top, (b - a).max(1.0), style.line_height),
            });
        }
        let empty = self.value.is_empty();
        let ph_galley = if empty && !self.placeholder.is_empty() { Some(ui.galley(&self.placeholder, &style)) } else { None };
        let focused = ix.focused;
        let icon = self.icon.clone();

        // Layout: column [label, field, message slot].
        let fill_w = matches!(self.width, Size::Fill(_));
        let field_w = match self.width {
            Size::Fixed(w) => Size::Fixed(w),
            Size::Fill(_) => Size::FILL,
            Size::Hug => Size::Fixed(280.0),
        };
        ui.begin(
            Style { width: if fill_w { Size::FILL } else { Size::Hug }, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );
        if let Some(l) = self.label.take() {
            let lc = if self.disabled { ui.color(Token::TextDisabled) } else { ui.color(Token::Foreground) };
            let st = ui.text_style(TextRole::Label);
            let n = ui.text_leaf(l, st, lc, false, true, Style::default());
            ui.a11y_hide(n);
        }
        let cursor_icon = if self.disabled { CursorIcon::Default } else { CursorIcon::Text };
        let galley2 = galley.clone();
        let line_h = style.line_height;
        ui.leaf(
            Style { width: field_w, height: Size::Fixed(m.height), margin_top: 6.0, min: vec2(120.0, 0.0), ..Default::default() },
            vec2(280.0, m.height),
            Some(id),
            Sense { click: true, drag: true, focusable: true, scroll: false },
            cursor_icon,
            Some(Box::new(move |cx, r| {
                let r = cx.painter.snap_rect(r);
                if halo_w > 0.01 && halo_a > 0.0 {
                    cx.painter.ring(r.expand(halo_w), Corners::all(radius + halo_w), halo_w, halo_c.multiply_alpha(halo_a));
                }
                cx.painter.rrect(r, Corners::all(radius), fill, if border.a > 0.0 { border_w } else { 0.0 }, border);
                if under_h > 0.01 {
                    // Windows: the bottom edge, thickening to an accent line.
                    cx.painter.push_clip(Rect::from_xywh(r.min.x, r.max.y - under_h, r.width(), under_h));
                    cx.painter.rrect(r, Corners::all(radius), Color::TRANSPARENT, under_h, under_c);
                    cx.painter.pop_clip();
                }
                if let Some(i) = &icon {
                    cx.icon(i, vec2(r.min.x + 10.0 + 8.0, r.center().y), 16.0, icon_c);
                }
                let text_rect = Rect { min: vec2(r.min.x + text_left, r.min.y + 1.0), max: vec2(r.max.x - 12.0, r.max.y - 1.0) };
                let top = r.center().y - line_h / 2.0;
                let origin = vec2(text_rect.min.x - scroll, cx.painter.snap(top));
                cx.painter.push_clip(text_rect.expand2(2.0, 0.0));
                if focused && sel.start != sel.end && marked.is_none() {
                    let x0 = origin.x + galley2.x_for_index(sel.start);
                    let x1 = origin.x + galley2.x_for_index(sel.end);
                    let sr = Rect::from_xywh(x0.min(x1), origin.y, (x1 - x0).abs(), line_h);
                    cx.painter.rrect_unsnapped(sr, Corners::all(3.0), [sel_c; 2], 0.0, Color::TRANSPARENT);
                }
                if let Some(ph) = &ph_galley {
                    cx.painter.text(cx.text, ph, vec2(text_rect.min.x, origin.y), ph_c);
                } else {
                    cx.painter.text(cx.text, &galley2, origin, text_c);
                }
                if let Some(m) = &marked {
                    // Composition: underline it, thicker under the segment
                    // the input method is converting.
                    let base = origin.y + line_h - 3.0;
                    let x0 = origin.x + galley2.x_for_index(m.start);
                    let x1 = origin.x + galley2.x_for_index(m.end);
                    cx.painter.rect(cx.painter.snap_rect(Rect::from_xywh(x0, base, x1 - x0, 1.0)), text_c);
                    if sel.start != sel.end {
                        let (s0, s1) = (origin.x + galley2.x_for_index(sel.start), origin.x + galley2.x_for_index(sel.end));
                        cx.painter.rect(cx.painter.snap_rect(Rect::from_xywh(s0, base - 0.5, s1 - s0, 2.0)), text_c);
                    }
                }
                if caret_a > 0.0 {
                    let x = cx.painter.snap(origin.x + galley2.x_for_index(cursor) - 1.0);
                    let cr = Rect::from_xywh(x, origin.y + 2.0, 2.0, line_h - 4.0);
                    cx.painter.rrect_unsnapped(cr, Corners::all(1.0), [caret_c.multiply_alpha(caret_a); 2], 0.0, Color::TRANSPARENT);
                }
                cx.painter.pop_clip();
            })),
        );

        // Message slot: error (animated) or helper.
        if let Some(e) = &self.error {
            ui.mem::<EditState>(id).last_error = e.clone();
        }
        // Height of the message (6 gap + wrapped text), measured at the
        // field's previous width so the reveal animates to the right size.
        let msg = ui.mem::<EditState>(id).last_error.clone();
        let mut msg_style = ui.text_style(TextRole::Label);
        msg_style.weight = 400;
        let wrap_w = (field_w_prev - 20.0).max(100.0);
        let msg_h = 6.0 + ui.text.layout(&msg, &msg_style, Some(wrap_w), TextAlign::Start).height;
        let reveal = ui.anims.spring_from(id, 5, 0.0, if has_error { 1.0 } else { 0.0 }, if has_error { mo.smooth } else { mo.snappy });
        let err_a = ui.fade(id, 6, has_error, 0.12, 0.10, Ease::Linear);
        if reveal > 0.001 || err_a > 0.0 {
            let dc = ui.color(Token::DangerStrong).multiply_alpha(err_a);
            ui.begin(Style { width: Size::FILL, height: Size::Fixed((reveal * msg_h).max(0.0)), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, true, None);
            ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, align: Align::Start, pad: crate::math::Sides { top: 6.0, ..Default::default() }, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            // 14px icon centered on the first 16px line.
            ui.begin(Style { pad: crate::math::Sides { top: 1.0, ..Default::default() }, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            ui.icon_leaf("circle-alert", 14.0, dc);
            ui.end();
            ui.text_leaf(msg, msg_style, dc, true, false, Style { width: Size::FILL, ..Default::default() });
            ui.end();
            ui.end();
        } else if let Some(hp) = self.helper.take() {
            let mut st = ui.text_style(TextRole::Label);
            st.weight = 400;
            let hc = if self.disabled { ui.color(Token::TextDisabled) } else { ui.color(Token::TextMuted) };
            let n = ui.text_leaf(hp, st, hc, true, false, Style { margin_top: 6.0, ..Default::default() });
            ui.set_text_align(n, TextAlign::Start);
        }
        ui.end();

        ix.rect = ui.rect_of(id).unwrap_or(ix.rect);
        Response { id, rect: ix.rect, hovered: ix.hovered, clicked: ix.clicked, changed, focused: ix.focused }
    }
}

show_on_drop!(TextInput);

impl Ui {
    /// Single-line text input bound to `value`.
    pub fn text_input<'u>(&'u mut self, value: &'u mut String) -> TextInput<'u> {
        TextInput {
            ui: self,
            value,
            label: None,
            placeholder: String::new(),
            icon: None,
            helper: None,
            error: None,
            disabled: false,
            width: Size::Hug,
            size: ControlSize::Md,
            mono: false,
            id_key: None,
            shown: false,
        }
    }
}
