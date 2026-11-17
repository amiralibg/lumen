//! One-time verification code input (`OtpInput`): individual digit cells with
//! auto-advance, paste-all, backspace navigation and bouncy glyph pop.

use super::show_on_drop;
use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect};
use crate::theme::{ControlSize, TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

#[derive(Default)]
struct OtpMem {
    slot: usize,
    was_focused: bool,
    last_activity: f64,
}

/// Verification code field built by [`Ui::otp_input`]. The whole strip is one
/// Tab stop; `←` / `→` move between slots, typing auto-advances, and pasting a
/// code (even with spaces or hyphens like `"482-913"`) fills every cell at once.
pub struct OtpInput<'u> {
    ui: &'u mut Ui,
    key: String,
    value: &'u mut String,
    digits: usize,
    group: Option<usize>,
    alphanumeric: bool,
    masked: bool,
    label: Option<String>,
    helper: Option<String>,
    error: Option<String>,
    disabled: bool,
    shown: bool,
}

impl<'u> OtpInput<'u> {
    /// Number of character cells (`3..=8`, default `6`).
    pub fn digits(mut self, n: usize) -> Self {
        self.digits = n.clamp(3, 8);
        self
    }
    /// Insert a separator dash every `n` cells (`0` = none; default `3` when `digits == 6`).
    pub fn group(mut self, n: usize) -> Self {
        self.group = Some(n);
        self
    }
    /// Accept ASCII letters as well as digits (letters are uppercased).
    pub fn alphanumeric(mut self) -> Self {
        self.alphanumeric = true;
        self
    }
    /// Draw bullets instead of visible characters (for numeric PINs).
    pub fn masked(mut self, on: bool) -> Self {
        self.masked = on;
        self
    }
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn helper(mut self, h: impl Into<String>) -> Self {
        self.helper = Some(h.into());
        self
    }
    pub fn error(mut self, e: Option<impl Into<String>>) -> Self {
        self.error = e.map(Into::into);
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    /// Show the input and return `true` on the frame the final cell is filled.
    pub fn completed(self) -> bool {
        let digits = self.digits;
        let ptr: *const String = self.value;
        let r = self.show();
        // SAFETY: `ptr` came from `&mut String` which outlives `self` (`'u`).
        let len = unsafe { (*ptr).chars().count() };
        r.changed && len == digits
    }

    fn valid_char(&self, c: char) -> Option<char> {
        if c.is_ascii_digit() {
            Some(c)
        } else if self.alphanumeric && c.is_ascii_alphabetic() {
            Some(c.to_ascii_uppercase())
        } else {
            None
        }
    }

    fn show_inner(&mut self) -> Response {
        let digits = self.digits;
        let group = self.group.unwrap_or(if digits == 6 { 3 } else { 0 });
        let mut chars: Vec<char> = self.value.chars().filter_map(|c| self.valid_char(c)).take(digits).collect();
        let before: String = chars.iter().collect();

        let ui = &mut *self.ui;
        let id = ui.make_id(("otp_input", &self.key));
        let now = ui.time();
        let mo = ui.motion();
        let m = ui.control_metrics(ControlSize::Md);
        let lk = ui.look().input;

        let ix = ui.interact(id, Sense::CLICK, self.disabled);
        let mut cell_ixs = Vec::with_capacity(digits);
        for i in 0..digits {
            let cid = id.with(("cell", i));
            cell_ixs.push(ui.interact(cid, Sense { click: true, drag: false, focusable: false, scroll: false }, self.disabled));
        }

        let mut slot = ui.mem::<OtpMem>(id).slot.min(digits - 1);
        let was_focused = ui.mem::<OtpMem>(id).was_focused;
        let mut clicked_cell = None;
        for (i, cix) in cell_ixs.iter().enumerate() {
            if cix.press_started && !self.disabled {
                clicked_cell = Some(i);
            }
        }
        if let Some(i) = clicked_cell {
            slot = i.min(chars.len()).min(digits - 1);
            ui.request_focus(id);
            ui.mem::<OtpMem>(id).last_activity = now;
        } else if ix.focused && !was_focused {
            slot = chars.len().min(digits - 1);
            ui.mem::<OtpMem>(id).last_activity = now;
        }
        slot = slot.min(chars.len()).min(digits - 1);

        let mut popped: Vec<usize> = vec![];
        let mut copied = None;
        if ix.focused && !self.disabled {
            ui.set_wants_text_input();
            let inputs = ui.take_text_input(id);
            let keys = ui.keys_for(id);
            let mut activity = !inputs.is_empty() || !keys.is_empty();

            for raw in inputs {
                let valid: Vec<char> = raw
                    .chars()
                    .filter(|c| !c.is_whitespace() && *c != '-' && *c != '–')
                    .filter_map(|c| {
                        if c.is_ascii_digit() {
                            Some(c)
                        } else if self.alphanumeric && c.is_ascii_alphabetic() {
                            Some(c.to_ascii_uppercase())
                        } else {
                            None
                        }
                    })
                    .collect();
                if valid.is_empty() {
                    continue;
                }
                if valid.len() >= digits || (valid.len() > 1 && slot == 0) {
                    chars.clear();
                    for (k, ch) in valid.into_iter().take(digits).enumerate() {
                        chars.push(ch);
                        popped.push(k);
                    }
                    slot = chars.len().min(digits - 1);
                } else {
                    for ch in valid {
                        if slot < chars.len() {
                            chars[slot] = ch;
                        } else if chars.len() < digits {
                            chars.push(ch);
                        }
                        popped.push(slot);
                        slot = (slot + 1).min(digits - 1);
                    }
                }
            }

            for (k, mods, _) in keys {
                let cmd = mods.command();
                match k {
                    Key::ArrowLeft => slot = slot.saturating_sub(1),
                    Key::ArrowRight => slot = (slot + 1).min(chars.len()).min(digits - 1),
                    Key::Home => slot = 0,
                    Key::End => slot = chars.len().min(digits - 1),
                    Key::Backspace => {
                        if slot < chars.len() {
                            chars.remove(slot);
                            slot = slot.min(chars.len()).min(digits - 1);
                        } else if slot > 0 && !chars.is_empty() {
                            slot -= 1;
                            if slot < chars.len() {
                                chars.remove(slot);
                            }
                        }
                    }
                    Key::Delete => {
                        if slot < chars.len() {
                            chars.remove(slot);
                            slot = slot.min(chars.len()).min(digits - 1);
                        }
                    }
                    Key::Char('a') if cmd => slot = 0,
                    Key::Char('c') if cmd && !self.masked && !chars.is_empty() => {
                        copied = Some(chars.iter().collect::<String>());
                    }
                    Key::Escape => {
                        chars.clear();
                        slot = 0;
                    }
                    Key::Tab => activity = false,
                    _ => {}
                }
            }
            if activity {
                ui.mem::<OtpMem>(id).last_activity = now;
            }
        }
        if let Some(c) = copied {
            ui.set_copied_text(c);
        }

        for &i in &popped {
            ui.anims.set_spring(id.with(("cell", i)), 2, 0.62);
        }

        let next_str: String = chars.iter().collect();
        let changed = next_str != before || *self.value != next_str;
        let value_changed = next_str != before;
        *self.value = next_str;

        let last_act = ui.mem::<OtpMem>(id).last_activity;
        let caret_a = if self.disabled { 0.0 } else { ui.caret_alpha(ix.focused, last_act) };
        let has_error = self.error.is_some();
        let err_t = ui.fade(id, 44, has_error, 0.12, 0.12, Ease::Linear);

        let cw = (m.height + 4.0).round();
        let ch = (m.height + 8.0).round();
        let radius = m.radius;
        let mut cell_font = ui.text_style(TextRole::Mono);
        cell_font.size = (17.0 * ui.look().text_scale).round();
        cell_font.weight = 600;
        cell_font.line_height = (22.0 * ui.look().text_scale).round();

        ui.begin(Style { width: Size::Hug, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if let Some(l) = self.label.take() {
            ui.field_label(&l, self.disabled);
        }

        ui.begin(
            Style { dir: Dir::Row, gap: 8.0, align: Align::Center, ..Default::default() },
            Some(id),
            Sense::CLICK,
            if self.disabled { CursorIcon::Default } else { CursorIcon::Text },
            false,
            None,
        );
        for i in 0..digits {
            if group > 0 && i > 0 && i % group == 0 {
                let dash_c = ui.color(Token::BorderStrong);
                ui.leaf(
                    Style::default(),
                    vec2(10.0, ch),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, r| {
                        let dr = Rect::from_center_size(r.center(), vec2(8.0, 2.0));
                        cx.painter.rrect(dr, Corners::all(1.0), dash_c, 0.0, Color::TRANSPARENT);
                    })),
                );
            }
            let cid = id.with(("cell", i));
            let cix = &cell_ixs[i];
            let active = (ix.focused || clicked_cell == Some(i)) && i == slot && !self.disabled;
            let ch_opt = chars.get(i).copied();
            let h = ui.fade(cid, 0, cix.hovered && !self.disabled, 0.06, 0.16, Ease::Linear);
            let f = ui.fade(cid, 1, active, 0.06, 0.16, Ease::Linear);
            let pop = ui.spring(cid, 2, if ch_opt.is_some() { 1.0 } else { 0.85 }, mo.bouncy);
            let halo_w = ui.spring(cid, 3, if active { 3.0 * lk.halo.max(0.75) } else { 0.0 }, mo.snappy);
            let halo_a = ui.fade(cid, 4, active, 0.0, 0.10, Ease::Linear);

            let c = |t| ui.color(t);
            let (fill, border, fg) = if self.disabled {
                (c(Token::SurfaceMuted), c(Token::BorderSubtle), c(Token::TextDisabled))
            } else {
                let filled_tint = if ch_opt.is_some() { 0.35 } else { 0.0 };
                let base_b = c(Token::Border).mix(c(Token::BorderStrong), h.max(filled_tint)).mix(c(Token::FocusRing), f);
                let rest = c(Token::Surface).mix(c(Token::Control), lk.filled.max(lk.underline * (1.0 - f)));
                let fill = rest.mix(c(Token::Control).over(c(Token::Hover)), h * 0.6);
                let b = base_b.mix(c(Token::Danger), err_t);
                (fill, b, c(Token::Foreground))
            };
            let halo_c = c(Token::FocusHalo).mix(c(Token::Danger).with_alpha(c(Token::FocusHalo).a), err_t);
            let caret_c = c(Token::FocusRing);
            let glyph = ch_opt.map(|ch| {
                let s = if self.masked { "•".to_owned() } else { ch.to_string() };
                ui.galley(&s, &cell_font)
            });
            let draw_caret = active && ch_opt.is_none() && caret_a > 0.0;

            ui.leaf(
                Style::default(),
                vec2(cw, ch),
                Some(cid),
                Sense { click: true, drag: false, focusable: false, scroll: false },
                if self.disabled { CursorIcon::Default } else { CursorIcon::Text },
                Some(Box::new(move |cx, r| {
                    let r = cx.painter.snap_rect(r);
                    if halo_w > 0.01 && halo_a > 0.0 {
                        cx.painter.ring(r.expand(halo_w), Corners::all(radius + halo_w), halo_w, halo_c.multiply_alpha(halo_a));
                    }
                    cx.painter.rrect(r, Corners::all(radius), fill, 1.0, border);
                    if let Some(g) = &glyph {
                        let rise = (1.0 - pop) * 5.0;
                        let a = pop.clamp(0.0, 1.0);
                        let pos = vec2((r.center().x - g.width / 2.0).round(), (r.center().y - g.height / 2.0 + rise).round());
                        cx.painter.text(cx.text, g, pos, fg.multiply_alpha(a));
                    } else if draw_caret {
                        let cr = Rect::from_center_size(r.center(), vec2(2.0, ch * 0.44));
                        cx.painter.rrect_unsnapped(cr, Corners::all(1.0), [caret_c.multiply_alpha(caret_a); 2], 0.0, Color::TRANSPARENT);
                    }
                })),
            );
        }
        ui.end();

        ui.field_message(id, self.helper.as_deref(), self.error.as_deref(), None, self.disabled);
        ui.end();

        let st = ui.mem::<OtpMem>(id);
        st.slot = slot;
        st.was_focused = ix.focused;

        let _ = changed;
        let rect = ui.rect_of(id).unwrap_or_default();
        let any_hovered = ix.hovered || cell_ixs.iter().any(|c| c.hovered);
        Response { id, rect, hovered: any_hovered, clicked: clicked_cell.is_some() || ix.clicked, changed: value_changed, focused: ix.focused }
    }
}

show_on_drop!(OtpInput);

impl Ui {
    /// Six-digit (or `.digits(n)`) verification code input bound to `value`.
    pub fn otp_input<'u>(&'u mut self, key: impl Into<String>, value: &'u mut String) -> OtpInput<'u> {
        OtpInput {
            ui: self,
            key: key.into(),
            value,
            digits: 6,
            group: None,
            alphanumeric: false,
            masked: false,
            label: None,
            helper: None,
            error: None,
            disabled: false,
            shown: false,
        }
    }
}
