//! Rich text inputs: multi-line [`TextArea`], [`TagInput`], [`PasswordInput`]
//! with a four-bar strength meter, and [`SearchField`].

use super::combo::chip_with_remove;
use super::field::{Chrome, EditOpts};
use super::show_on_drop;
use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{ControlSize, TextRole, Token};
use crate::ui::{Placement, Response, Sense, TooltipSpec, Ui};
use crate::Color;

// ================================================================== TextArea

/// Multi-line text field that springs in height between `rows` and scrolls
/// once full. Built with [`Ui::text_area`].
pub struct TextArea<'u> {
    ui: &'u mut Ui,
    value: &'u mut String,
    label: Option<String>,
    placeholder: String,
    helper: Option<String>,
    error: Option<String>,
    rows: (u32, u32),
    max_chars: Option<usize>,
    mono: bool,
    disabled: bool,
    width: Size,
    id_key: Option<String>,
    shown: bool,
}

impl<'u> TextArea<'u> {
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
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
    /// Minimum and maximum visible line rows before vertical scrolling (default `3..=8`).
    pub fn rows(mut self, min: u32, max: u32) -> Self {
        let min = min.max(2);
        self.rows = (min, max.max(min));
        self
    }
    /// Cap character count and show a live `n / max` counter.
    pub fn max_chars(mut self, max: usize) -> Self {
        self.max_chars = Some(max.max(1));
        self
    }
    /// Render text in Geist Mono (for snippets, keys, logs or commit bodies).
    pub fn mono(mut self) -> Self {
        self.mono = true;
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
    pub fn id(mut self, key: impl Into<String>) -> Self {
        self.id_key = Some(key.into());
        self
    }

    fn show_inner(&mut self) -> Response {
        let key = self.id_key.clone().or_else(|| self.label.clone()).unwrap_or_else(|| self.placeholder.clone());
        let ui = &mut *self.ui;
        let id = ui.make_id(("text_area", key));
        let edit_id = id.with("edit");
        let focused = ui.focused() == Some(edit_id);
        let hovered = ui.pointer().is_some_and(|p| ui.rect_of(id).is_some_and(|r| r.contains(p)));
        let fill_w = matches!(self.width, Size::Fill(_));
        let fw = match self.width {
            Size::Hug => Size::Fixed(340.0),
            w => w,
        };

        ui.begin(Style { width: if fill_w { Size::FILL } else { Size::Hug }, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if let Some(l) = self.label.take() {
            ui.field_label(&l, self.disabled);
        }
        ui.field_begin(
            id,
            Chrome { hovered, focused, error: self.error.is_some(), disabled: self.disabled },
            Style { dir: Dir::Column, width: fw, pad: Sides::xy(12.0, 8.0), ..Default::default() },
        );
        let mut o = EditOpts::new(ui);
        if self.mono {
            o.style = ui.text_style(TextRole::Mono);
            o.style.line_height = (20.0 * ui.look().text_scale).round();
        }
        o.placeholder = self.placeholder.clone();
        o.multiline = true;
        o.rows = self.rows;
        o.max_chars = self.max_chars;
        o.disabled = self.disabled;
        let ed = ui.inline_edit(edit_id, self.value, o);
        ui.end();

        if ui.clicked_id(id) && !self.disabled {
            ui.request_focus(edit_id);
        }
        let counter = self.max_chars.map(|max| {
            let n = self.value.chars().count();
            let col = if self.disabled {
                ui.color(Token::TextDisabled)
            } else if n >= max {
                ui.color(Token::DangerStrong)
            } else if n * 100 >= max * 85 {
                ui.color(Token::WarningStrong)
            } else {
                ui.color(Token::TextMuted)
            };
            (format!("{n} / {max}"), col)
        });
        ui.field_message(id, self.helper.as_deref(), self.error.as_deref(), counter, self.disabled);
        ui.end();

        let rect = ui.rect_of(id).unwrap_or_default();
        Response { id, rect, hovered, clicked: ed.ix.clicked || ui.clicked_id(id), changed: ed.changed, focused }
    }
}

show_on_drop!(TextArea);

// ================================================================== TagInput

#[derive(Default)]
struct TagMem {
    draft: String,
    armed: bool,
    was_focused: bool,
    flash: Option<(String, f64)>,
}

/// Free-form tag field: committed tags become removable chips that wrap inside
/// the frame as its height springs open. Enter or `,` commits a tag; Backspace
/// twice on an empty query arms then removes the last tag.
pub struct TagInput<'u> {
    ui: &'u mut Ui,
    tags: &'u mut Vec<String>,
    label: Option<String>,
    placeholder: String,
    helper: Option<String>,
    error: Option<String>,
    max_tags: Option<usize>,
    lowercase: bool,
    disabled: bool,
    width: Size,
    id_key: Option<String>,
    shown: bool,
}

impl<'u> TagInput<'u> {
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
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
    /// Maximum number of tags allowed.
    pub fn max_tags(mut self, max: usize) -> Self {
        self.max_tags = Some(max.max(1));
        self
    }
    /// Normalize committed tags to lowercase (e.g. runner labels or topics).
    pub fn lowercase(mut self, on: bool) -> Self {
        self.lowercase = on;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Size::Fixed(w.max(180.0));
        self
    }
    pub fn fill_width(mut self) -> Self {
        self.width = Size::FILL;
        self
    }
    pub fn id(mut self, key: impl Into<String>) -> Self {
        self.id_key = Some(key.into());
        self
    }

    fn show_inner(&mut self) -> Response {
        let key = self.id_key.clone().or_else(|| self.label.clone()).unwrap_or_else(|| self.placeholder.clone());
        let ui = &mut *self.ui;
        let id = ui.make_id(("tag_input", key));
        let edit_id = id.with("edit");
        let wrap_id = id.with("wrap");
        let now = ui.time();
        let focused = ui.focused() == Some(edit_id);
        let hovered = ui.pointer().is_some_and(|p| ui.rect_of(id).is_some_and(|r| r.contains(p)));
        let mo = ui.motion();
        let m = ui.control_metrics(ControlSize::Md);

        let mut draft = std::mem::take(&mut ui.mem::<TagMem>(id).draft);
        let armed = ui.mem::<TagMem>(id).armed;
        let flash_tag = ui.mem::<TagMem>(id).flash.clone().and_then(|(t, until)| (now < until).then_some((t, until)));
        if let Some((_, until)) = flash_tag {
            ui.request_repaint_after(until - now);
        } else if ui.mem_get::<TagMem>(id).is_some_and(|s| s.flash.is_some()) {
            ui.mem::<TagMem>(id).flash = None;
        }

        let fill_w = matches!(self.width, Size::Fill(_));
        let fw = match self.width {
            Size::Hug => Size::Fixed(340.0),
            w => w,
        };
        let full = self.max_tags.is_some_and(|max| self.tags.len() >= max);
        let mut remove_idx: Option<usize> = None;

        ui.begin(Style { width: if fill_w { Size::FILL } else { Size::Hug }, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if let Some(l) = self.label.take() {
            ui.field_label(&l, self.disabled);
        }
        let inner_h = ui.rect_of(wrap_id).map(|r| r.height()).unwrap_or(m.height - 8.0);
        let h = ui.spring(id, 5, (inner_h + 8.0).max(m.height), mo.smooth);
        ui.field_begin(
            id,
            Chrome { hovered, focused, error: self.error.is_some(), disabled: self.disabled },
            Style { dir: Dir::Row, width: fw, height: Size::Fixed(h), pad: Sides::all(4.0), gap: 4.0, align: Align::Start, ..Default::default() },
        );
        ui.begin(Style { width: Size::FILL, height: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, true, None);
        ui.begin(Style { dir: Dir::Row, wrap: true, width: Size::FILL, gap: 4.0, align: Align::Center, ..Default::default() }, Some(wrap_id), Sense::NONE, CursorIcon::Default, false, None);

        let n_tags = self.tags.len();
        for (i, tag) in self.tags.iter().enumerate() {
            let is_armed = (armed && i + 1 == n_tags) || flash_tag.as_ref().is_some_and(|(ft, _)| ft.eq_ignore_ascii_case(tag));
            if ui.scope(("tag", tag.as_str()), |ui| chip_with_remove(ui, tag, is_armed, self.disabled)) {
                remove_idx = Some(i);
            }
        }

        let mut o = EditOpts::new(ui);
        o.placeholder = if self.tags.is_empty() && !full { self.placeholder.clone() } else { String::new() };
        o.disabled = self.disabled || full;
        o.min_w = 72.0;
        ui.begin(
            Style { width: Size::FILL, min: vec2(72.0, 0.0), height: Size::Fixed(m.height - 8.0), pad: Sides::xy(8.0, 0.0), align: Align::Center, justify: Justify::Center, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );
        let ed = ui.inline_edit(edit_id, &mut draft, o);
        ui.end();
        ui.end();
        ui.end();
        ui.end();

        if ui.clicked_id(id) && !self.disabled && !full {
            ui.request_focus(edit_id);
        }

        let mut changed = false;
        let mut new_flash: Option<(String, f64)> = None;
        let lowercase = self.lowercase;
        let max_tags = self.max_tags;
        let mut commit_one = |raw: &str, tags: &mut Vec<String>| {
            let mut t = raw.trim().to_owned();
            if lowercase {
                t = t.to_lowercase();
            }
            if t.is_empty() {
                return;
            }
            if let Some(existing) = tags.iter().find(|e| e.eq_ignore_ascii_case(&t)).cloned() {
                new_flash = Some((existing, now + 0.36));
                return;
            }
            if max_tags.is_none_or(|max| tags.len() < max) {
                tags.push(t);
                changed = true;
            }
        };

        let mut armed_now = armed && ed.keys.iter().all(|(k, _)| *k == Key::Backspace) && !ed.changed && !ed.ix.press_started;

        // Commas split and commit tags immediately (including pasted comma lists).
        if draft.contains(',') {
            let parts: Vec<String> = draft.split(',').map(str::to_owned).collect();
            for p in &parts[..parts.len() - 1] {
                commit_one(p, self.tags);
            }
            draft = parts.last().cloned().unwrap_or_default().trim_start().to_owned();
            ui.inline_edit_caret_end(edit_id, &draft);
            armed_now = false;
        }

        let still_focused = ui.focused() == Some(edit_id);
        let blurred = !still_focused && ui.mem::<TagMem>(id).was_focused;
        if ed.key(Key::Enter) || blurred {
            if !draft.trim().is_empty() {
                let pending = std::mem::take(&mut draft);
                commit_one(&pending, self.tags);
                ui.inline_edit_caret_end(edit_id, &draft);
                armed_now = false;
            }
        }
        if ed.key(Key::Escape) && !draft.is_empty() {
            draft.clear();
            ui.inline_edit_caret_end(edit_id, &draft);
            armed_now = false;
        }
        if ed.key(Key::Backspace) && draft.is_empty() && !self.tags.is_empty() {
            if armed {
                remove_idx = Some(self.tags.len() - 1);
                armed_now = false;
            } else {
                armed_now = true;
            }
        }
        if let Some(i) = remove_idx {
            if i < self.tags.len() {
                self.tags.remove(i);
                changed = true;
            }
        }

        // Bounce any duplicate chip so the user sees it is already in the list.
        if let Some((ref dup, _)) = new_flash {
            let chip_id = ui.scope(("tag", dup.as_str()), |ui| ui.make_id("chip"));
            ui.anims.set_spring(chip_id, 0, 0.80);
            ui.request_repaint_after(0.36);
        }

        let counter = self.max_tags.map(|max| {
            let n = self.tags.len();
            let col = if self.disabled {
                ui.color(Token::TextDisabled)
            } else if n >= max {
                ui.color(Token::WarningStrong)
            } else {
                ui.color(Token::TextMuted)
            };
            (format!("{n} / {max}"), col)
        });
        ui.field_message(id, self.helper.as_deref(), self.error.as_deref(), counter, self.disabled);
        ui.end();

        let st = ui.mem::<TagMem>(id);
        st.draft = draft;
        st.armed = armed_now;
        st.was_focused = still_focused;
        if new_flash.is_some() {
            st.flash = new_flash;
        }
        let rect = ui.rect_of(id).unwrap_or_default();
        Response { id, rect, hovered, clicked: ed.ix.clicked || ui.clicked_id(id), changed, focused }
    }
}

show_on_drop!(TagInput);

// ============================================================= PasswordInput

/// Estimate password strength on a `0..=4` scale (`0` = empty, `1` = weak,
/// `2` = fair, `3` = good, `4` = strong).
pub fn password_strength(pw: &str) -> u8 {
    if pw.is_empty() {
        return 0;
    }
    let len = pw.chars().count();
    let lower = pw.to_ascii_lowercase();
    const COMMON: [&str; 8] = ["password", "123456", "qwerty", "letmein", "welcome", "admin", "iloveyou", "monkey"];
    if len < 8 || COMMON.iter().any(|w| lower.contains(w)) {
        return 1;
    }
    let has_lo = pw.chars().any(|c| c.is_lowercase());
    let has_hi = pw.chars().any(|c| c.is_uppercase());
    let has_num = pw.chars().any(|c| c.is_numeric());
    let has_sym = pw.chars().any(|c| !c.is_alphanumeric() && !c.is_whitespace());
    let classes = has_lo as u8 + has_hi as u8 + has_num as u8 + has_sym as u8;
    let mut uniq: Vec<char> = pw.chars().collect();
    uniq.sort_unstable();
    uniq.dedup();
    if uniq.len() < 4 {
        return 1;
    }
    if (len >= 12 && classes >= 3 && uniq.len() >= 7) || (len >= 16 && classes >= 2 && uniq.len() >= 8) {
        4
    } else if (len >= 10 && classes >= 3) || (len >= 14 && classes >= 2) {
        3
    } else {
        2
    }
}

#[derive(Default)]
struct PwMem {
    revealed: bool,
    copied_until: f64,
}

/// Password input with bullet masking, a show/hide button, optional copy
/// action, and an optional four-bar animated strength meter.
pub struct PasswordInput<'u> {
    ui: &'u mut Ui,
    value: &'u mut String,
    label: Option<String>,
    placeholder: String,
    icon: Option<String>,
    helper: Option<String>,
    error: Option<String>,
    strength: bool,
    copyable: bool,
    disabled: bool,
    width: Size,
    id_key: Option<String>,
    shown: bool,
}

impl<'u> PasswordInput<'u> {
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = Some(name.to_owned());
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
    /// Show the four-segment password strength meter and guidance below the field.
    pub fn strength(mut self, on: bool) -> Self {
        self.strength = on;
        self
    }
    /// Add a trailing copy button (useful in credential views).
    pub fn copyable(mut self, on: bool) -> Self {
        self.copyable = on;
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
    pub fn id(mut self, key: impl Into<String>) -> Self {
        self.id_key = Some(key.into());
        self
    }

    fn show_inner(&mut self) -> Response {
        let key = self.id_key.clone().or_else(|| self.label.clone()).unwrap_or_else(|| self.placeholder.clone());
        let ui = &mut *self.ui;
        let id = ui.make_id(("password", key));
        let edit_id = id.with("edit");
        let rev_id = id.with("reveal");
        let copy_id = id.with("copy");
        let now = ui.time();
        let mo = ui.motion();
        let m = ui.control_metrics(ControlSize::Md);
        let focused = ui.focused() == Some(edit_id);
        let hovered = ui.pointer().is_some_and(|p| ui.rect_of(id).is_some_and(|r| r.contains(p)));
        let revealed = ui.mem::<PwMem>(id).revealed;
        let copied_until = ui.mem::<PwMem>(id).copied_until;
        let just_copied = now < copied_until;
        if just_copied {
            ui.request_repaint_after(copied_until - now);
        }

        let fill_w = matches!(self.width, Size::Fill(_));
        let fw = match self.width {
            Size::Hug => Size::Fixed(300.0),
            w => w,
        };

        ui.begin(Style { width: if fill_w { Size::FILL } else { Size::Hug }, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if let Some(l) = self.label.take() {
            ui.field_label(&l, self.disabled);
        }
        ui.field_begin(
            id,
            Chrome { hovered, focused, error: self.error.is_some(), disabled: self.disabled },
            Style { dir: Dir::Row, width: fw, height: Size::Fixed(m.height), pad: Sides { left: 12.0, right: 6.0, top: 0.0, bottom: 0.0 }, gap: 8.0, align: Align::Center, ..Default::default() },
        );
        if let Some(ic_name) = &self.icon {
            let f = ui.fade(id, 2, focused, mo.hover_in, mo.hover_out, Ease::Linear);
            let ic = if self.disabled {
                ui.color(Token::TextDisabled)
            } else {
                ui.color(Token::TextMuted).mix(ui.color(Token::TextSecondary), f)
            };
            ui.icon_leaf(ic_name, 16.0, ic);
        }
        let mut o = EditOpts::new(ui);
        if revealed && !self.value.is_empty() {
            o.style = ui.text_style(TextRole::Mono);
            o.style.line_height = (20.0 * ui.look().text_scale).round();
        }
        o.placeholder = self.placeholder.clone();
        o.mask = !revealed;
        o.disabled = self.disabled;
        let ed = ui.inline_edit(edit_id, self.value, o);

        if self.copyable && !self.disabled {
            let show_copy = ui.spring(id, 3, if !self.value.is_empty() { 1.0 } else { 0.0 }, mo.bouncy);
            if show_copy > 0.02 {
                let ic = if just_copied { "check" } else { "copy" };
                if ui.field_icon_button(copy_id, ic, show_copy, true) {
                    ui.set_copied_text(self.value.clone());
                    ui.mem::<PwMem>(id).copied_until = now + 1.2;
                    ui.request_repaint_after(1.2);
                }
                ui.tooltip_for(
                    copy_id,
                    TooltipSpec { text: if just_copied { "Copied".into() } else { "Copy password".into() }, shortcut: None, placement: Placement::Top, gap: 6.0 },
                );
            }
        }
        if !self.disabled {
            let ic = if revealed { "eye-off" } else { "eye" };
            if ui.field_icon_button(rev_id, ic, 1.0, true) {
                let st = ui.mem::<PwMem>(id);
                st.revealed = !st.revealed;
            }
            ui.tooltip_for(
                rev_id,
                TooltipSpec { text: if revealed { "Hide password".into() } else { "Show password".into() }, shortcut: None, placement: Placement::Top, gap: 6.0 },
            );
        }
        ui.end();

        if ui.clicked_id(id) && !self.disabled {
            ui.request_focus(edit_id);
        }

        if self.strength {
            let score = password_strength(self.value);
            let s_val = ui.spring(id, 14, score as f32, mo.smooth).clamp(0.0, 4.0);
            let c_danger = ui.color(Token::Danger);
            let c_warn = ui.color(Token::Warning);
            let c_info = ui.color(Token::Info);
            let c_ok = ui.color(Token::Success);
            let bar_fill = if s_val <= 1.0 {
                c_danger
            } else if s_val <= 2.0 {
                c_danger.mix(c_warn, s_val - 1.0)
            } else if s_val <= 3.0 {
                c_warn.mix(c_info, s_val - 2.0)
            } else {
                c_info.mix(c_ok, s_val - 3.0)
            };
            let track = ui.color(Token::Track).multiply_alpha(0.75);

            ui.begin(Style { dir: Dir::Row, width: fw, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            for i in 0..4u32 {
                let fill_t = ui.spring(id, 10 + i, if score > i as u8 { 1.0 } else { 0.0 }, mo.smooth).clamp(0.0, 1.0);
                ui.leaf(
                    Style { width: Size::FILL, ..Default::default() },
                    vec2(24.0, 4.0),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, r| {
                        cx.painter.rrect(r, Corners::all(2.0), track, 0.0, Color::TRANSPARENT);
                        if fill_t > 0.01 {
                            let fr = Rect::from_xywh(r.min.x, r.min.y, r.width() * fill_t, r.height());
                            cx.painter.push_clip(r);
                            cx.painter.rrect_unsnapped(fr, Corners::all(2.0), [bar_fill; 2], 0.0, Color::TRANSPARENT);
                            cx.painter.pop_clip();
                        }
                    })),
                );
            }
            ui.end();

            let (grade, hint, grade_tok) = match score {
                0 => ("Strength", "Use 12+ characters with mixed types", Token::TextMuted),
                1 => ("Weak", "Avoid common words and use 8+ characters", Token::DangerStrong),
                2 => ("Fair", "Add uppercase, numbers or a symbol", Token::WarningStrong),
                3 => ("Good", "A few more characters makes it strong", Token::InfoStrong),
                _ => ("Strong", "Ready to save", Token::SuccessStrong),
            };
            let mut st_grade = ui.text_style(TextRole::Caption);
            st_grade.weight = 500;
            let st_hint = ui.text_style(TextRole::Caption);
            let gc = if self.disabled { ui.color(Token::TextDisabled) } else { ui.color(grade_tok) };
            let hc = if self.disabled { ui.color(Token::TextDisabled) } else { ui.color(Token::TextMuted) };
            ui.begin(Style { dir: Dir::Row, width: fw, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            ui.text_leaf(grade, st_grade, gc, false, false, Style::default());
            ui.spacer();
            ui.text_leaf(hint, st_hint, hc, false, true, Style::default());
            ui.end();
        }

        ui.field_message(id, self.helper.as_deref(), self.error.as_deref(), None, self.disabled);
        ui.end();

        let rect = ui.rect_of(id).unwrap_or_default();
        Response { id, rect, hovered, clicked: ed.ix.clicked || ui.clicked_id(id), changed: ed.changed, focused }
    }
}

show_on_drop!(PasswordInput);

// =============================================================== SearchField

/// Search input with a leading magnifier (or inline spinner), optional
/// shortcut hint badge, optional match count, and a clear button that springs
/// in when non-empty.
pub struct SearchField<'u> {
    ui: &'u mut Ui,
    value: &'u mut String,
    label: Option<String>,
    placeholder: String,
    shortcut: Option<String>,
    focus_key: Option<char>,
    cmd_key: Option<char>,
    loading: bool,
    results: Option<usize>,
    disabled: bool,
    width: Size,
    id_key: Option<String>,
    shown: bool,
}

impl<'u> SearchField<'u> {
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }
    /// Trailing shortcut badge shown while empty and unfocused (e.g. `"/"` or `"⌘F"`).
    pub fn shortcut(mut self, s: impl Into<String>) -> Self {
        self.shortcut = Some(s.into());
        self
    }
    /// Focus the search field when `ch` (e.g. `'/'`) is pressed outside text inputs.
    pub fn focus_key(mut self, ch: char) -> Self {
        self.focus_key = Some(ch);
        self
    }
    /// Focus the search field on `⌘/Ctrl + ch` (e.g. `'f'`).
    pub fn cmd_key(mut self, ch: char) -> Self {
        self.cmd_key = Some(ch);
        self
    }
    /// Replace the leading magnifier with an activity spinner while results load.
    pub fn loading(mut self, on: bool) -> Self {
        self.loading = on;
        self
    }
    /// Show a match count inside the field while the query is non-empty.
    pub fn results(mut self, count: Option<usize>) -> Self {
        self.results = count;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Size::Fixed(w.max(150.0));
        self
    }
    pub fn fill_width(mut self) -> Self {
        self.width = Size::FILL;
        self
    }
    pub fn id(mut self, key: impl Into<String>) -> Self {
        self.id_key = Some(key.into());
        self
    }

    fn show_inner(&mut self) -> Response {
        let key = self.id_key.clone().or_else(|| self.label.clone()).unwrap_or_else(|| self.placeholder.clone());
        let ui = &mut *self.ui;
        let id = ui.make_id(("search_field", key));
        let edit_id = id.with("edit");
        let clear_id = id.with("clear");
        let mo = ui.motion();
        let m = ui.control_metrics(ControlSize::Md);

        if !self.disabled {
            let trig_plain = self.focus_key.is_some_and(|c| ui.shortcut(Key::Char(c)));
            let trig_cmd = self.cmd_key.is_some_and(|c| ui.shortcut_pressed(c));
            if trig_plain || trig_cmd {
                ui.request_focus(edit_id);
                ui.inline_edit_select_all(edit_id, self.value);
            }
        }

        let focused = ui.focused() == Some(edit_id);
        let hovered = ui.pointer().is_some_and(|p| ui.rect_of(id).is_some_and(|r| r.contains(p)));
        let f = ui.fade(id, 2, focused, mo.hover_in, mo.hover_out, Ease::Linear);
        let fill_w = matches!(self.width, Size::Fill(_));
        let fw = match self.width {
            Size::Hug => Size::Fixed(280.0),
            w => w,
        };

        ui.begin(Style { width: if fill_w { Size::FILL } else { Size::Hug }, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if let Some(l) = self.label.take() {
            ui.field_label(&l, self.disabled);
        }
        ui.field_begin(
            id,
            Chrome { hovered, focused, error: false, disabled: self.disabled },
            Style { dir: Dir::Row, width: fw, height: Size::Fixed(m.height), pad: Sides { left: 11.0, right: 6.0, top: 0.0, bottom: 0.0 }, gap: 8.0, align: Align::Center, ..Default::default() },
        );
        if self.loading && !self.disabled {
            ui.spinner(15.0);
        } else {
            let ic = if self.disabled {
                ui.color(Token::TextDisabled)
            } else {
                ui.color(Token::TextMuted).mix(ui.color(Token::AccentStrong), f)
            };
            ui.icon_leaf("search", 16.0, ic);
        }

        let mut o = EditOpts::new(ui);
        o.placeholder = self.placeholder.clone();
        o.disabled = self.disabled;
        let ed = ui.inline_edit(edit_id, self.value, o);

        if let Some(n) = self.results.filter(|_| !self.value.is_empty()) {
            let st = ui.text_style(TextRole::Caption);
            let c = ui.color(Token::TextMuted);
            let txt = if n == 1 { "1 match".into() } else { format!("{n} matches") };
            ui.text_leaf(txt, st, c, false, false, Style::default());
        }

        let has_text = !self.value.is_empty() && !self.disabled;
        let clear_t = ui.spring(id, 3, if has_text { 1.0 } else { 0.0 }, mo.bouncy);
        let mut cleared = false;
        if clear_t > 0.02 {
            if ui.field_icon_button(clear_id, "x", clear_t, false) {
                self.value.clear();
                ui.inline_edit_caret_end(edit_id, self.value);
                ui.request_focus(edit_id);
                cleared = true;
            }
        } else if let Some(sc) = &self.shortcut {
            let sc_a = ui.fade(id, 4, self.value.is_empty() && !focused && !self.disabled, mo.hover_in, mo.hover_out, Ease::Linear);
            if sc_a > 0.01 {
                let mut mono = ui.text_style(TextRole::Mono);
                mono.size = (11.0 * ui.look().text_scale).round();
                mono.line_height = 16.0;
                let g = ui.galley(sc, &mono);
                let bg = ui.color(Token::SurfaceMuted).multiply_alpha(sc_a);
                let bd = ui.color(Token::BorderSubtle).multiply_alpha(sc_a);
                let fg = ui.color(Token::TextMuted).multiply_alpha(sc_a);
                let kw = (g.width + 10.0).max(20.0);
                ui.leaf(
                    Style { margin_top: 0.0, ..Default::default() },
                    vec2(kw + 2.0, 20.0),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, r| {
                        let kr = Rect::from_center_size(r.center(), vec2(kw, 19.0));
                        cx.painter.rrect(kr, Corners::all(4.0), bg, 1.0, bd);
                        cx.painter.text(cx.text, &g, vec2((kr.center().x - g.width / 2.0).round(), (kr.center().y - g.height / 2.0).round()), fg);
                    })),
                );
            }
        }
        ui.end();
        ui.end();

        if ui.clicked_id(id) && !self.disabled {
            ui.request_focus(edit_id);
        }
        if ed.key(Key::Escape) {
            if !self.value.is_empty() {
                self.value.clear();
                ui.inline_edit_caret_end(edit_id, self.value);
                cleared = true;
            } else {
                ui.surrender_focus(edit_id);
            }
        }

        let rect = ui.rect_of(id).unwrap_or_default();
        Response { id, rect, hovered, clicked: ed.ix.clicked || ui.clicked_id(id), changed: ed.changed || cleared, focused }
    }
}

show_on_drop!(SearchField);

impl Ui {
    /// Multi-line text field bound to `value` that grows between `3..=8` rows
    /// on a spring and scrolls thereafter.
    pub fn text_area<'u>(&'u mut self, value: &'u mut String) -> TextArea<'u> {
        TextArea {
            ui: self,
            value,
            label: None,
            placeholder: String::new(),
            helper: None,
            error: None,
            rows: (3, 8),
            max_chars: None,
            mono: false,
            disabled: false,
            width: Size::Hug,
            id_key: None,
            shown: false,
        }
    }

    /// Free-form tag input bound to `tags`. Press Enter or `,` to commit a tag;
    /// press Backspace twice on empty text to remove the last tag.
    pub fn tag_input<'u>(&'u mut self, tags: &'u mut Vec<String>) -> TagInput<'u> {
        TagInput {
            ui: self,
            tags,
            label: None,
            placeholder: "Add tag…".into(),
            helper: None,
            error: None,
            max_tags: None,
            lowercase: false,
            disabled: false,
            width: Size::Hug,
            id_key: None,
            shown: false,
        }
    }

    /// Password field with bullet masking, a show/hide toggle and an optional
    /// four-segment strength meter (`.strength(true)`).
    pub fn password_input<'u>(&'u mut self, value: &'u mut String) -> PasswordInput<'u> {
        PasswordInput {
            ui: self,
            value,
            label: None,
            placeholder: String::new(),
            icon: None,
            helper: None,
            error: None,
            strength: false,
            copyable: false,
            disabled: false,
            width: Size::Hug,
            id_key: None,
            shown: false,
        }
    }

    /// Search field with a leading magnifier, clear button, and optional
    /// shortcut badge and match count.
    pub fn search_field<'u>(&'u mut self, value: &'u mut String) -> SearchField<'u> {
        SearchField {
            ui: self,
            value,
            label: None,
            placeholder: "Search…".into(),
            shortcut: None,
            focus_key: None,
            cmd_key: None,
            loading: false,
            results: None,
            disabled: false,
            width: Size::Hug,
            id_key: None,
            shown: false,
        }
    }
}
