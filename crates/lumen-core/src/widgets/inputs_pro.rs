//! Advanced inputs, discrete selection and list shuttles:
//! [`EditableText`], [`MoneyField`], [`MaskedInput`], [`MentionInput`],
//! [`StepSlider`], [`TransferList`], and [`ReorderableList`].

use super::field::{Chrome, EditOpts};
use super::show_on_drop;
use super::toast::Tone;
use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::id::Id;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{ControlSize, TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

// ============================================================= EditableText

#[derive(Default)]
struct EditableMem {
    editing: bool,
    backup: String,
    just_opened: bool,
}

/// Click-to-edit inline text or heading: rests as clean typography with a
/// subtle hover frame and pencil cue; clicking or pressing `Enter` morphs it
/// into an inline text editor (`Enter` commits, `Escape` reverts).
pub struct EditableText<'u> {
    ui: &'u mut Ui,
    key: String,
    value: &'u mut String,
    placeholder: String,
    role: TextRole,
    disabled: bool,
    shown: bool,
}

impl<'u> EditableText<'u> {
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }
    pub fn role(mut self, r: TextRole) -> Self {
        self.role = r;
        self
    }
    pub fn heading(mut self) -> Self {
        self.role = TextRole::Heading3;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("editable_text", &self.key));
        let edit_id = id.with("ed");
        let mo = ui.motion();

        let editing = ui.mem::<EditableMem>(id).editing;
        let mut changed = false;

        if editing && !self.disabled {
            let just_opened = std::mem::replace(&mut ui.mem::<EditableMem>(id).just_opened, false);
            if just_opened {
                ui.request_focus(edit_id);
                ui.inline_edit_select_all(edit_id, self.value);
            }
            let focused = ui.focused() == Some(edit_id);
            let hovered = ui.pointer().is_some_and(|p| ui.rect_of(id).is_some_and(|r| r.contains(p)));
            ui.field_begin(
                id,
                Chrome { hovered, focused, error: false, disabled: false },
                Style {
                    dir: Dir::Row,
                    width: Size::FILL,
                    pad: Sides::xy(10.0, 6.0),
                    gap: 8.0,
                    align: Align::Center,
                    ..Default::default()
                },
            );
            let mut opts = EditOpts::new(ui);
            opts.style = ui.text_style(self.role);
            opts.placeholder = self.placeholder.clone();
            let ed = ui.inline_edit(edit_id, self.value, opts);
            changed |= ed.changed;

            let check_clicked = ui.scope("ok", |ui| ui.icon_button("check").small().tooltip("Save (Enter)").clicked());
            ui.end();

            if ed.key(Key::Enter) || check_clicked || (!focused && !just_opened) {
                if self.value.trim().is_empty() {
                    let backup = ui.mem::<EditableMem>(id).backup.clone();
                    if !backup.is_empty() {
                        *self.value = backup;
                    }
                }
                ui.mem::<EditableMem>(id).editing = false;
                changed = true;
            } else if ed.key(Key::Escape) {
                let backup = ui.mem::<EditableMem>(id).backup.clone();
                *self.value = backup;
                ui.mem::<EditableMem>(id).editing = false;
            }
            let rect = ui.rect_of(id).unwrap_or(Rect::ZERO);
            Response { id, rect, hovered, clicked: ed.ix.clicked, changed, focused }
        } else {
            let ix = ui.interact(id, Sense::CLICK, self.disabled);
            let hov = ui.fade(id, 0, ix.hovered, mo.hover_in, mo.hover_out, Ease::Out);
            let f_ring = ui.fade(id, 1, ix.focus_visible, mo.hover_in, mo.hover_out, Ease::Out);
            let r_ctl = ui.control_metrics(ControlSize::Md).radius;
            let hov_bg = ui.color(Token::Hover).multiply_alpha(hov);
            let hov_bd = ui.color(Token::BorderSubtle).multiply_alpha(hov);
            let ring_c = ui.color(Token::FocusRing).multiply_alpha(f_ring);
            let fl = ui.look().focus;

            ui.begin(
                Style {
                    dir: Dir::Row,
                    width: Size::FILL,
                    pad: Sides::xy(10.0, 6.0),
                    gap: 8.0,
                    align: Align::Center,
                    ..Default::default()
                },
                Some(id),
                Sense::CLICK,
                if self.disabled { CursorIcon::Default } else { CursorIcon::Text },
                false,
                Some(Box::new(move |cx, r| {
                    if hov > 0.005 {
                        cx.painter.rrect(r, Corners::all(r_ctl), hov_bg, 1.0, hov_bd);
                    }
                    if f_ring > 0.005 {
                        cx.painter.ring(r.expand(fl.gap), Corners::all(r_ctl + fl.gap), fl.width, ring_c);
                    }
                })),
            );
            let st = ui.text_style(self.role);
            let is_empty = self.value.trim().is_empty();
            let txt = if is_empty { self.placeholder.clone() } else { self.value.clone() };
            let txt_c = if self.disabled {
                ui.color(Token::TextDisabled)
            } else if is_empty {
                ui.color(Token::TextMuted)
            } else {
                ui.color(Token::Foreground)
            };
            ui.text_leaf(txt, st, txt_c, false, true, Style { width: Size::FILL, ..Default::default() });
            if !self.disabled {
                let ic_c = ui.color(Token::TextMuted).multiply_alpha(0.35 + 0.65 * hov);
                ui.icon_leaf("pencil", 14.0, ic_c);
            }
            ui.end();

            if ix.clicked && !self.disabled {
                let mem = ui.mem::<EditableMem>(id);
                mem.backup = self.value.clone();
                mem.editing = true;
                mem.just_opened = true;
            }
            let rect = ui.rect_of(id).unwrap_or(ix.rect);
            Response { id, rect, hovered: ix.hovered, clicked: ix.clicked, changed, focused: ix.focused }
        }
    }
}

show_on_drop!(EditableText);

// =============================================================== MoneyField

/// Currency unit for [`MoneyField`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Currency {
    #[default]
    Usd,
    Eur,
    Gbp,
    Jpy,
}

impl Currency {
    pub const ALL: [Currency; 4] = [Currency::Usd, Currency::Eur, Currency::Gbp, Currency::Jpy];

    pub fn code(self) -> &'static str {
        match self {
            Currency::Usd => "USD",
            Currency::Eur => "EUR",
            Currency::Gbp => "GBP",
            Currency::Jpy => "JPY",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Currency::Usd => "$",
            Currency::Eur => "€",
            Currency::Gbp => "£",
            Currency::Jpy => "¥",
        }
    }

    pub fn decimals(self) -> usize {
        match self {
            Currency::Jpy => 0,
            _ => 2,
        }
    }
}

#[derive(Default)]
struct MoneyMem {
    draft: String,
    was_focused: bool,
}

fn format_money(val: f64, decimals: usize) -> String {
    let neg = val < 0.0;
    let abs = val.abs();
    let scaled = if decimals == 0 { abs.round() } else { (abs * 10f64.powi(decimals as i32)).round() / 10f64.powi(decimals as i32) };
    let raw = format!("{scaled:.decimals$}");
    let (int_part, frac_part) = match raw.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (raw.as_str(), None),
    };
    let mut grouped = String::with_capacity(int_part.len() + 4);
    for (idx, ch) in int_part.chars().rev().enumerate() {
        if idx > 0 && idx % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    let int_rev: String = grouped.chars().rev().collect();
    let mut out = String::new();
    if neg && scaled > 0.0 {
        out.push('-');
    }
    out.push_str(&int_rev);
    if let Some(f) = frac_part {
        out.push('.');
        out.push_str(f);
    }
    out
}

/// Formatted currency input with live thousands grouping, optional currency
/// switcher pill (`USD`, `EUR`, `GBP`, `JPY`), and step buttons.
pub struct MoneyField<'u> {
    ui: &'u mut Ui,
    value: &'u mut f64,
    currency: Currency,
    currency_mut: Option<&'u mut Currency>,
    label: Option<String>,
    helper: Option<String>,
    min: f64,
    max: f64,
    step: f64,
    disabled: bool,
    width: Size,
    shown: bool,
}

impl<'u> MoneyField<'u> {
    pub fn currency(mut self, c: Currency) -> Self {
        self.currency = c;
        self
    }
    /// Allow clicking the currency badge to cycle currencies (`USD` → `EUR` → `GBP` → `JPY`).
    pub fn switchable_currency(mut self, c: &'u mut Currency) -> Self {
        self.currency = *c;
        self.currency_mut = Some(c);
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
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max.max(min);
        self
    }
    pub fn step(mut self, s: f64) -> Self {
        self.step = s.abs().max(0.01);
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn fill_width(mut self) -> Self {
        self.width = Size::FILL;
        self
    }

    fn show_inner(&mut self) -> Response {
        let key = self.label.clone().unwrap_or_else(|| "money".into());
        let ui = &mut *self.ui;
        let id = ui.make_id(("money_field", key));
        let edit_id = id.with("ed");
        let focused = ui.focused() == Some(edit_id);
        let hovered = ui.pointer().is_some_and(|p| ui.rect_of(id).is_some_and(|r| r.contains(p)));
        let cm = ui.control_metrics(ControlSize::Md);
        let cur = self.currency_mut.as_deref().copied().unwrap_or(self.currency);
        let decimals = cur.decimals();

        let was_focused = ui.mem::<MoneyMem>(id).was_focused;
        if !focused {
            ui.mem::<MoneyMem>(id).draft = format_money(*self.value, decimals);
        } else if !was_focused {
            ui.mem::<MoneyMem>(id).draft = format!("{:.decimals$}", *self.value);
            let d = ui.mem::<MoneyMem>(id).draft.clone();
            ui.inline_edit_select_all(edit_id, &d);
        }

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
            Style {
                dir: Dir::Row,
                width: fw,
                height: Size::Fixed(cm.height),
                pad: Sides { left: 6.0, right: 6.0, top: 0.0, bottom: 0.0 },
                gap: 8.0,
                align: Align::Center,
                ..Default::default()
            },
        );

        let mut changed = false;
        // Leading currency badge (interactive if `currency_mut` is provided).
        let can_switch = self.currency_mut.is_some() && !self.disabled;
        let badge_id = id.with("cur_btn");
        let b_ix = ui.interact(badge_id, if can_switch { Sense::CLICK } else { Sense::NONE }, self.disabled);
        let b_hov = ui.fade(badge_id, 0, b_ix.hovered, ui.motion().hover_in, ui.motion().hover_out, Ease::Out);
        let b_bg = ui.color(Token::SurfaceMuted).mix(ui.color(Token::Hover), b_hov);
        let b_bd = ui.color(Token::BorderSubtle);
        let r_pill = (cm.radius - 2.0).max(5.0);
        ui.begin(
            Style { dir: Dir::Row, pad: Sides::xy(8.0, 4.0), gap: 4.0, align: Align::Center, ..Default::default() },
            Some(badge_id),
            if can_switch { Sense::CLICK } else { Sense::NONE },
            if can_switch { CursorIcon::Pointer } else { CursorIcon::Default },
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_pill), b_bg, 1.0, b_bd))),
        );
        let mut st_cur = ui.text_style(TextRole::Caption);
        st_cur.weight = 600;
        let c_fg = ui.color(Token::TextSecondary);
        let sym_fg = ui.color(Token::AccentStrong);
        ui.text_leaf(cur.symbol().to_owned(), st_cur, sym_fg, false, false, Style::default());
        ui.text_leaf(cur.code().to_owned(), st_cur, c_fg, false, false, Style::default());
        if can_switch {
            ui.icon_leaf("chevrons-up-down", 11.0, ui.color(Token::TextMuted));
        }
        ui.end();
        if b_ix.clicked {
            if let Some(cm_ref) = self.currency_mut.as_deref_mut() {
                let pos = Currency::ALL.iter().position(|x| *x == *cm_ref).unwrap_or(0);
                *cm_ref = Currency::ALL[(pos + 1) % Currency::ALL.len()];
                let new_dec = cm_ref.decimals();
                ui.mem::<MoneyMem>(id).draft = format_money(*self.value, new_dec);
                changed = true;
            }
        }

        let mut draft = ui.mem::<MoneyMem>(id).draft.clone();
        let mut opts = EditOpts::new(ui);
        opts.style.tabular = true;
        opts.disabled = self.disabled;
        opts.accept = Some(|c| c.is_ascii_digit() || matches!(c, '.' | ',' | '-'));
        let ed = ui.inline_edit(edit_id, &mut draft, opts);
        if ed.changed {
            let cleaned: String = draft.chars().filter(|&c| c != ',').collect();
            if let Ok(parsed) = cleaned.parse::<f64>() {
                *self.value = parsed.clamp(self.min, self.max);
                changed = true;
            }
        }
        if ed.key(Key::ArrowUp) && !self.disabled {
            *self.value = (*self.value + self.step).clamp(self.min, self.max);
            draft = format!("{:.decimals$}", *self.value);
            changed = true;
        } else if ed.key(Key::ArrowDown) && !self.disabled {
            *self.value = (*self.value - self.step).clamp(self.min, self.max);
            draft = format!("{:.decimals$}", *self.value);
            changed = true;
        } else if ed.key(Key::Enter) {
            draft = format_money(*self.value, decimals);
        }
        ui.mem::<MoneyMem>(id).draft = draft;

        // Trailing - / + step buttons.
        if !self.disabled {
            let dec_clicked = ui.scope("dec", |ui| ui.icon_button("minus").small().clicked());
            let inc_clicked = ui.scope("inc", |ui| ui.icon_button("plus").small().clicked());
            if dec_clicked {
                *self.value = (*self.value - self.step).clamp(self.min, self.max);
                ui.mem::<MoneyMem>(id).draft = format_money(*self.value, decimals);
                changed = true;
            }
            if inc_clicked {
                *self.value = (*self.value + self.step).clamp(self.min, self.max);
                ui.mem::<MoneyMem>(id).draft = format_money(*self.value, decimals);
                changed = true;
            }
        }
        ui.end();

        ui.mem::<MoneyMem>(id).was_focused = ui.focused() == Some(edit_id);
        if let Some(ref h) = self.helper {
            ui.field_message(id, Some(h), None, None, self.disabled);
        }
        ui.end();

        let rect = ui.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, hovered, clicked: ed.ix.clicked, changed, focused }
    }
}

show_on_drop!(MoneyField);

// ============================================================== MaskedInput

fn apply_mask(raw: &str, mask: &str) -> String {
    let mut chars = raw.chars().peekable();
    let mut out = String::with_capacity(mask.len());
    for mc in mask.chars() {
        match mc {
            '#' => {
                let mut found = None;
                while let Some(c) = chars.next() {
                    if c.is_ascii_digit() {
                        found = Some(c);
                        break;
                    }
                }
                match found {
                    Some(d) => out.push(d),
                    None => break,
                }
            }
            'A' => {
                let mut found = None;
                while let Some(c) = chars.next() {
                    if c.is_ascii_alphabetic() {
                        found = Some(c.to_ascii_uppercase());
                        break;
                    }
                }
                match found {
                    Some(a) => out.push(a),
                    None => break,
                }
            }
            '*' => {
                let mut found = None;
                while let Some(c) = chars.next() {
                    if c.is_ascii_alphanumeric() {
                        found = Some(c.to_ascii_uppercase());
                        break;
                    }
                }
                match found {
                    Some(a) => out.push(a),
                    None => break,
                }
            }
            lit => {
                if chars.peek() == Some(&lit) {
                    chars.next();
                    out.push(lit);
                } else if chars.clone().any(|c| c.is_ascii_alphanumeric()) {
                    out.push(lit);
                } else {
                    break;
                }
            }
        }
    }
    out
}

/// Pattern-masked field (`#` = digit, `A` = letter, `*` = alphanumeric; any
/// other char is a literal separator inserted automatically) with a live ghost
/// template showing remaining slots.
pub struct MaskedInput<'u> {
    ui: &'u mut Ui,
    value: &'u mut String,
    mask: String,
    label: Option<String>,
    helper: Option<String>,
    icon: Option<String>,
    disabled: bool,
    width: Size,
    shown: bool,
}

impl<'u> MaskedInput<'u> {
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn helper(mut self, h: impl Into<String>) -> Self {
        self.helper = Some(h.into());
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
    pub fn fill_width(mut self) -> Self {
        self.width = Size::FILL;
        self
    }

    fn show_inner(&mut self) -> Response {
        let key = self.label.clone().unwrap_or_else(|| self.mask.clone());
        let ui = &mut *self.ui;
        let id = ui.make_id(("masked_input", key));
        let edit_id = id.with("ed");
        let focused = ui.focused() == Some(edit_id);
        let hovered = ui.pointer().is_some_and(|p| ui.rect_of(id).is_some_and(|r| r.contains(p)));
        let cm = ui.control_metrics(ControlSize::Md);

        *self.value = apply_mask(self.value, &self.mask);

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
            Style {
                dir: Dir::Row,
                width: fw,
                height: Size::Fixed(cm.height),
                pad: Sides::xy(10.0, 0.0),
                gap: 8.0,
                align: Align::Center,
                ..Default::default()
            },
        );
        if let Some(ref ic) = self.icon {
            ui.icon_leaf(ic, 15.0, ui.color(Token::TextMuted));
        }

        let mut mono = ui.text_style(TextRole::Mono);
        mono.tabular = true;
        let mut opts = EditOpts::new(ui);
        opts.style = mono;
        opts.placeholder = self.mask.clone();
        opts.max_chars = Some(self.mask.chars().count());
        opts.disabled = self.disabled;
        let prev = self.value.clone();
        let ed = ui.inline_edit(edit_id, self.value, opts);
        let mut changed = false;
        if ed.changed {
            // If user pressed Backspace over a literal separator, drop one extra alphanumeric char.
            let shrunk = self.value.len() < prev.len();
            let mut formatted = apply_mask(self.value, &self.mask);
            if shrunk && formatted == prev && !prev.is_empty() {
                let mut alnum: Vec<char> = prev.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
                alnum.pop();
                let s: String = alnum.into_iter().collect();
                formatted = apply_mask(&s, &self.mask);
            }
            *self.value = formatted;
            ui.inline_edit_caret_end(edit_id, self.value);
            changed = *self.value != prev;
        }

        // Remaining mask hint badge on the right so user sees exact completion!
        let mask_chars: Vec<char> = self.mask.chars().collect();
        let cur_len = self.value.chars().count();
        let complete = cur_len == mask_chars.len() && !mask_chars.is_empty();
        if complete {
            ui.icon_leaf("circle-check", 15.0, ui.color(Token::SuccessStrong));
        } else if cur_len > 0 && cur_len < mask_chars.len() {
            let rem: String = mask_chars[cur_len..].iter().collect();
            let mut st_rem = mono;
            st_rem.size = (st_rem.size - 1.0).max(11.0);
            ui.text_leaf(rem, st_rem, ui.color(Token::TextMuted).multiply_alpha(0.65), false, false, Style::default());
        }
        ui.end();

        if ui.clicked_id(id) && !self.disabled {
            ui.request_focus(edit_id);
        }
        if let Some(ref h) = self.helper {
            ui.field_message(id, Some(h), None, None, self.disabled);
        }
        ui.end();

        let rect = ui.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, hovered, clicked: ed.ix.clicked, changed, focused }
    }
}

show_on_drop!(MaskedInput);

// ============================================================= MentionInput

/// Candidate in [`MentionInput`].
#[derive(Clone, Debug)]
pub struct MentionCandidate {
    pub handle: String,
    pub name: String,
    pub role: String,
}

impl MentionCandidate {
    pub fn new(handle: impl Into<String>, name: impl Into<String>, role: impl Into<String>) -> Self {
        MentionCandidate { handle: handle.into(), name: name.into(), role: role.into() }
    }
}

#[derive(Default)]
struct MentionMem {
    active: usize,
}

fn active_mention_token(text: &str) -> Option<(usize, &str)> {
    let at_idx = text.rfind('@')?;
    if at_idx > 0 {
        let prev = text[..at_idx].chars().next_back()?;
        if !prev.is_whitespace() {
            return None;
        }
    }
    let query = &text[at_idx + 1..];
    if query.chars().any(char::is_whitespace) {
        return None;
    }
    Some((at_idx, query))
}

/// Inline `@mention` input: typing `@` opens an anchored candidate popover
/// filtered by handle or display name, and displays referenced `@handles` as
/// pill badges underneath.
pub struct MentionInput<'u> {
    ui: &'u mut Ui,
    value: &'u mut String,
    candidates: &'u [MentionCandidate],
    label: Option<String>,
    placeholder: String,
    shown: bool,
}

impl<'u> MentionInput<'u> {
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }

    fn show_inner(&mut self) -> Response {
        let key = self.label.clone().unwrap_or_else(|| "mention".into());
        let ui = &mut *self.ui;
        let id = ui.make_id(("mention_input", key));
        let edit_id = id.with("ed");
        let focused = ui.focused() == Some(edit_id);
        let hovered = ui.pointer().is_some_and(|p| ui.rect_of(id).is_some_and(|r| r.contains(p)));
        let cm = ui.control_metrics(ControlSize::Md);

        ui.begin(Style { width: Size::FILL, gap: 8.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if let Some(l) = self.label.take() {
            ui.field_label(&l, false);
        }

        ui.field_begin(
            id,
            Chrome { hovered, focused, error: false, disabled: false },
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                height: Size::Fixed(cm.height),
                pad: Sides::xy(10.0, 0.0),
                gap: 8.0,
                align: Align::Center,
                ..Default::default()
            },
        );
        ui.icon_leaf("at-sign", 15.0, ui.color(Token::AccentStrong));
        let mut opts = EditOpts::new(ui);
        opts.placeholder = self.placeholder.clone();
        let ed = ui.inline_edit(edit_id, self.value, opts);
        ui.end();

        let field_rect = ui.rect_of(id).unwrap_or(Rect::ZERO);
        let mut changed = ed.changed;

        // Check if trailing token starts with `@`!
        let trigger = if focused { active_mention_token(self.value).map(|(idx, q)| (idx, q.to_lowercase())) } else { None };
        let matches: Vec<usize> = if let Some((_, ref q)) = trigger {
            self.candidates
                .iter()
                .enumerate()
                .filter(|(_, c)| q.is_empty() || c.handle.to_lowercase().contains(q) || c.name.to_lowercase().contains(q))
                .map(|(i, _)| i)
                .collect()
        } else {
            Vec::new()
        };

        let should_open = trigger.is_some() && !matches.is_empty();
        ui.set_popover_open(id, should_open);

        let mut active = ui.mem::<MentionMem>(id).active.min(matches.len().saturating_sub(1));
        let mut commit_idx: Option<usize> = None;

        if should_open {
            if ed.key(Key::ArrowDown) {
                active = (active + 1) % matches.len();
            } else if ed.key(Key::ArrowUp) {
                active = (active + matches.len() - 1) % matches.len();
            } else if ed.key(Key::Enter) {
                commit_idx = Some(matches[active]);
            } else if ed.key(Key::Escape) {
                ui.set_popover_open(id, false);
            }
        }

        let anchor = Response { id, rect: field_rect, hovered, clicked: false, changed: false, focused };
        let cands = self.candidates;
        if let Some(picked) = ui.popover(&anchor).manual().match_width().pad(4.0).return_focus(edit_id).show(|ui| {
            let mut clicked_cand = None;
            ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 2.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            for (mi, &ci) in matches.iter().enumerate() {
                let c = &cands[ci];
                let rid = id.with(("mrow", ci));
                let r_ix = ui.interact(rid, Sense::CLICK, false);
                if r_ix.hovered {
                    active = mi;
                }
                let is_act = mi == active;
                let bg = if is_act { ui.color(Token::AccentSubtle) } else { Color::TRANSPARENT };
                ui.begin(
                    Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(10.0, 6.0), gap: 8.0, align: Align::Center, ..Default::default() },
                    Some(rid),
                    Sense::CLICK,
                    CursorIcon::Pointer,
                    false,
                    Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(6.0), bg, 0.0, Color::TRANSPARENT))),
                );
                let initials: String = c.name.split_whitespace().filter_map(|w| w.chars().next()).take(2).collect();
                ui.scope(("av", ci), |ui| {
                    ui.avatar(&initials, ci, 22.0, None);
                });
                ui.text_leaf(c.name.clone(), ui.text_style(TextRole::BodyStrong), ui.color(Token::Foreground), false, false, Style::default());
                ui.text_leaf(format!("@{}", c.handle), ui.text_style(TextRole::Mono), ui.color(Token::AccentStrong), false, true, Style { width: Size::FILL, ..Default::default() });
                ui.text_leaf(c.role.clone(), ui.text_style(TextRole::Caption), ui.color(Token::TextMuted), false, false, Style::default());
                ui.end();
                if r_ix.clicked {
                    clicked_cand = Some(ci);
                }
            }
            ui.end();
            clicked_cand
        }) {
            if picked.is_some() {
                commit_idx = picked;
            }
        }
        ui.mem::<MentionMem>(id).active = active;

        if let (Some(ci), Some((at_idx, _))) = (commit_idx, active_mention_token(self.value).map(|(i, q)| (i, q.to_owned()))) {
            let replacement = format!("@{} ", self.candidates[ci].handle);
            self.value.replace_range(at_idx.., &replacement);
            ui.inline_edit_caret_end(edit_id, self.value);
            ui.set_popover_open(id, false);
            ui.request_focus(edit_id);
            changed = true;
        }

        // Render active @mentions detected in the message as subtle accent pills!
        let mentioned: Vec<&MentionCandidate> = self
            .candidates
            .iter()
            .filter(|c| {
                let tag = format!("@{}", c.handle);
                self.value.split_whitespace().any(|w| w.trim_end_matches(|p: char| ".,:;!?".contains(p)) == tag)
            })
            .collect();
        if !mentioned.is_empty() {
            ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, wrap: true, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            ui.text_leaf("Notifying:", ui.text_style(TextRole::Caption), ui.color(Token::TextMuted), false, false, Style::default());
            for mc in mentioned {
                let p_bg = ui.color(Token::AccentSubtle);
                let p_bd = ui.color(Token::Accent).multiply_alpha(0.35);
                let p_fg = ui.color(Token::AccentStrong);
                ui.begin(
                    Style { dir: Dir::Row, pad: Sides::xy(7.0, 2.0), gap: 4.0, align: Align::Center, ..Default::default() },
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(99.0), p_bg, 1.0, p_bd))),
                );
                let mut st = ui.text_style(TextRole::Caption);
                st.weight = 600;
                ui.text_leaf(format!("@{}", mc.handle), st, p_fg, false, false, Style::default());
                ui.end();
            }
            ui.end();
        }
        ui.end();

        Response { id, rect: field_rect, hovered, clicked: ed.ix.clicked, changed, focused }
    }
}

show_on_drop!(MentionInput);

// =============================================================== StepSlider

/// Discrete snapped slider with labeled tick stops (`"128 MB"`, `"512 MB"`, …),
/// spring thumb travel and press squash.
pub struct StepSlider<'u, 's> {
    ui: &'u mut Ui,
    label: String,
    index: &'u mut usize,
    stops: &'s [&'s str],
    disabled: bool,
    shown: bool,
}

impl<'u, 's> StepSlider<'u, 's> {
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
        let id = ui.make_id(("step_slider", &self.label));
        let n = self.stops.len().max(2);
        *self.index = (*self.index).min(n - 1);

        let track_id = id.with("track");
        let ix = ui.interact(
            track_id,
            Sense { click: true, drag: true, focusable: true, scroll: false },
            self.disabled,
        );
        let mo = ui.motion();
        let mut changed = false;

        if !self.disabled {
            if (ix.clicked || ix.held) && ix.rect.width() > 24.0 {
                if let Some(p) = ix.pointer {
                    let pad = 12.0;
                    let t = ((p.x - (ix.rect.min.x + pad)) / (ix.rect.width() - pad * 2.0)).clamp(0.0, 1.0);
                    let snapped = (t * (n - 1) as f32).round() as usize;
                    if snapped != *self.index {
                        *self.index = snapped;
                        changed = true;
                    }
                }
            }
            if ix.focused {
                if ui.key_pressed_for(track_id, Key::ArrowRight) || ui.key_pressed_for(track_id, Key::ArrowUp) {
                    let next = (*self.index + 1).min(n - 1);
                    changed |= next != *self.index;
                    *self.index = next;
                } else if ui.key_pressed_for(track_id, Key::ArrowLeft) || ui.key_pressed_for(track_id, Key::ArrowDown) {
                    let next = self.index.saturating_sub(1);
                    changed |= next != *self.index;
                    *self.index = next;
                } else if ui.key_pressed_for(track_id, Key::Home) {
                    changed |= *self.index != 0;
                    *self.index = 0;
                } else if ui.key_pressed_for(track_id, Key::End) {
                    changed |= *self.index != n - 1;
                    *self.index = n - 1;
                }
            }
        }

        let frac_target = *self.index as f32 / (n - 1) as f32;
        let frac = ui.spring(id, 0, frac_target, mo.snappy).clamp(0.0, 1.0);
        let press = ui.spring(id, 1, if ix.pressed || ix.held { 1.0 } else { 0.0 }, mo.snappy);
        let hov = ui.fade(id, 2, ix.hovered, mo.hover_in, mo.hover_out, Ease::Out);
        let f_ring = ui.fade(id, 3, ix.focus_visible, mo.hover_in, mo.hover_out, Ease::Out);

        ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 8.0, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);

        // Header row: label + active stop pill.
        if !self.label.is_empty() {
            ui.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            ui.text_leaf(
                self.label.clone(),
                ui.text_style(TextRole::Label),
                ui.color(if self.disabled { Token::TextDisabled } else { Token::Foreground }),
                false,
                true,
                Style { width: Size::FILL, ..Default::default() },
            );
            let cur_lbl = self.stops.get(*self.index).copied().unwrap_or("");
            ui.badge(cur_lbl, Tone::Accent);
            ui.end();
        }

        let sl = ui.look().slider;
        let track_bg = ui.color(Token::Track);
        let fill_c = ui.color(if self.disabled { Token::TextDisabled } else { Token::Accent });
        let dot_idle = ui.color(Token::BorderStrong);
        let dot_ring = ui.color(Token::Surface);
        let thumb_c = Color::WHITE;
        let thumb_bd = ui.color(Token::Accent);
        let ring_c = ui.color(Token::FocusRing).multiply_alpha(f_ring);
        let stops_vec: Vec<String> = self.stops.iter().map(|s| (*s).to_owned()).collect();
        let active_idx = *self.index;
        let mut st_tick = ui.text_style(TextRole::Caption);
        st_tick.tabular = true;
        let tick_fg_act = ui.color(Token::Foreground);
        let tick_fg_muted = ui.color(Token::TextMuted);

        ui.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(240.0, 44.0),
            Some(track_id),
            Sense { click: true, drag: true, focusable: true, scroll: false },
            if self.disabled { CursorIcon::Default } else { CursorIcon::Pointer },
            Some(Box::new(move |cx, r| {
                let pad = 12.0;
                let x0 = r.min.x + pad;
                let span = (r.width() - pad * 2.0).max(1.0);
                let cy = r.min.y + 12.0;
                let th = sl.track;

                // Base & filled track.
                cx.painter.rrect(Rect::from_xywh(x0, cy - th * 0.5, span, th), Corners::all(th * 0.5), track_bg, 0.0, Color::TRANSPARENT);
                let fill_w = (span * frac).max(0.0);
                if fill_w > 0.5 {
                    cx.painter.rrect(Rect::from_xywh(x0, cy - th * 0.5, fill_w, th), Corners::all(th * 0.5), fill_c, 0.0, Color::TRANSPARENT);
                }

                // Tick dots & stop labels.
                for (i, lbl) in stops_vec.iter().enumerate() {
                    let tf = i as f32 / (n - 1) as f32;
                    let tx = x0 + span * tf;
                    let passed = tf <= frac + 0.01;
                    cx.painter.circle(vec2(tx, cy), 3.5, if passed { fill_c } else { dot_idle }, 1.5, dot_ring);
                    let g = cx.galley(lbl, &st_tick);
                    let lx = if i == 0 {
                        x0
                    } else if i + 1 == n {
                        (x0 + span - g.width).max(x0)
                    } else {
                        (tx - g.width * 0.5).clamp(r.min.x, (r.max.x - g.width).max(r.min.x))
                    };
                    let col = if i == active_idx { tick_fg_act } else { tick_fg_muted };
                    cx.painter.text(cx.text, &g, vec2(lx, cy + 13.0), col);
                }

                // Thumb with press squash & focus halo.
                let thumb_x = x0 + span * frac;
                let rad = (sl.thumb * 0.5 + hov * 1.2) * (1.0 - 0.06 * press);
                if f_ring > 0.005 {
                    cx.painter.circle(vec2(thumb_x, cy), rad + 4.0, Color::TRANSPARENT, 2.0, ring_c);
                }
                cx.painter.circle(vec2(thumb_x, cy), rad, thumb_c, 2.5, thumb_bd);
            })),
        );
        ui.end();

        let rect = ui.rect_of(id).unwrap_or(ix.rect);
        Response { id, rect, hovered: ix.hovered, clicked: ix.clicked, changed, focused: ix.focused }
    }
}

impl Drop for StepSlider<'_, '_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

// ============================================================ TransferList

#[derive(Default)]
struct TransferMem {
    filter: String,
    sel_left: Vec<String>,
    sel_right: Vec<String>,
}

/// Dual-list shuttle (`Available` ↔ `Assigned`) with live search filter,
/// multi-selection checkboxes, item counts, and shuttle action buttons.
pub struct TransferList<'u> {
    ui: &'u mut Ui,
    key: String,
    available: &'u mut Vec<String>,
    assigned: &'u mut Vec<String>,
    left_title: String,
    right_title: String,
    height: f32,
    shown: bool,
}

impl<'u> TransferList<'u> {
    pub fn titles(mut self, left: impl Into<String>, right: impl Into<String>) -> Self {
        self.left_title = left.into();
        self.right_title = right.into();
        self
    }
    pub fn height(mut self, px: f32) -> Self {
        self.height = px.max(150.0);
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("transfer_list", &self.key));
        let mut changed = false;

        let mut filter = ui.mem::<TransferMem>(id).filter.clone();
        ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 10.0, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);

        ui.scope("tf_search", |ui| {
            ui.search_field(&mut filter).placeholder("Filter permissions or items…").fill_width().show();
        });
        ui.mem::<TransferMem>(id).filter = filter.clone();
        let q = filter.trim().to_lowercase();

        ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 10.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);

        // Left pane (Available).
        let mut sel_left = ui.mem::<TransferMem>(id).sel_left.clone();
        sel_left.retain(|x| self.available.contains(x));
        render_transfer_pane(ui, id.with("left"), &self.left_title, self.available, &mut sel_left, &q, self.height);

        // Center shuttle controls.
        ui.begin(Style { dir: Dir::Column, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let move_sel_r = ui.scope("mv_r", |ui| ui.icon_button("chevron-right").small().disabled(sel_left.is_empty()).tooltip("Assign selected").clicked());
        let move_all_r = ui.scope("mv_all_r", |ui| ui.icon_button("chevrons-right").small().disabled(self.available.is_empty()).tooltip("Assign all").clicked());
        let mut sel_right = ui.mem::<TransferMem>(id).sel_right.clone();
        sel_right.retain(|x| self.assigned.contains(x));
        let move_sel_l = ui.scope("mv_l", |ui| ui.icon_button("chevron-left").small().disabled(sel_right.is_empty()).tooltip("Remove selected").clicked());
        let move_all_l = ui.scope("mv_all_l", |ui| ui.icon_button("chevrons-left").small().disabled(self.assigned.is_empty()).tooltip("Remove all").clicked());
        ui.end();

        // Right pane (Assigned).
        render_transfer_pane(ui, id.with("right"), &self.right_title, self.assigned, &mut sel_right, &q, self.height);

        ui.end();
        ui.end();

        if move_sel_r {
            let mut moved = Vec::new();
            self.available.retain(|it| {
                if sel_left.contains(it) {
                    moved.push(it.clone());
                    false
                } else {
                    true
                }
            });
            self.assigned.extend(moved);
            sel_left.clear();
            changed = true;
        } else if move_all_r {
            self.assigned.append(self.available);
            sel_left.clear();
            changed = true;
        } else if move_sel_l {
            let mut moved = Vec::new();
            self.assigned.retain(|it| {
                if sel_right.contains(it) {
                    moved.push(it.clone());
                    false
                } else {
                    true
                }
            });
            self.available.extend(moved);
            sel_right.clear();
            changed = true;
        } else if move_all_l {
            self.available.append(self.assigned);
            sel_right.clear();
            changed = true;
        }

        let mem = ui.mem::<TransferMem>(id);
        mem.sel_left = sel_left;
        mem.sel_right = sel_right;

        let rect = ui.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, changed, ..Default::default() }
    }
}

fn render_transfer_pane(
    ui: &mut Ui,
    pane_id: Id,
    title: &str,
    items: &[String],
    selected: &mut Vec<String>,
    query: &str,
    height: f32,
) {
    let r_card = ui.look().card.radius;
    let top_rad = (r_card - 1.0).max(0.0);
    let bg_c = ui.color(Token::Surface);
    let bd_c = ui.color(Token::BorderSubtle);
    let hdr_bg = ui.color(Token::SurfaceMuted).multiply_alpha(0.6);
    let div_c = ui.color(Token::BorderSubtle);

    ui.begin(
        Style { dir: Dir::Column, width: Size::FILL, ..Default::default() },
        Some(pane_id),
        Sense::NONE,
        CursorIcon::Default,
        true,
        Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
    );
    // Header.
    ui.begin(
        Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(10.0, 7.0), gap: 6.0, align: Align::Center, ..Default::default() },
        None,
        Sense::NONE,
        CursorIcon::Default,
        false,
        Some(Box::new(move |cx, r| {
            let inner = Rect::from_xywh(r.min.x + 1.0, r.min.y + 1.0, (r.width() - 2.0).max(0.0), (r.height() - 1.0).max(0.0));
            cx.painter.rrect(inner, Corners([top_rad, top_rad, 0.0, 0.0]), hdr_bg, 0.0, Color::TRANSPARENT);
            cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.max.y - 1.0, (r.width() - 2.0).max(0.0), 1.0), div_c);
        })),
    );
    let mut st_h = ui.text_style(TextRole::Caption);
    st_h.weight = 600;
    ui.text_leaf(title.to_owned(), st_h, ui.color(Token::TextSecondary), false, true, Style { width: Size::FILL, ..Default::default() });
    let cnt_lbl = if selected.is_empty() { format!("{}", items.len()) } else { format!("{}/{}", selected.len(), items.len()) };
    ui.badge(&cnt_lbl, Tone::Neutral);
    ui.end();

    // Scrollable rows.
    ui.begin(Style { width: Size::FILL, height: Size::Fixed(height), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, true, None);
    ui.scroll(pane_id.with("sc")).pad(Sides::all(6.0)).gap(2.0).show(|ui| {
        let mut shown_any = false;
        for (idx, it) in items.iter().enumerate() {
            if !query.is_empty() && !it.to_lowercase().contains(query) {
                continue;
            }
            shown_any = true;
            let mut is_on = selected.contains(it);
            let rid = pane_id.with(("row", idx));
            let r_ix = ui.interact(rid, Sense::CLICK, false);
            let bg = if is_on {
                ui.color(Token::AccentSubtle)
            } else if r_ix.hovered {
                ui.color(Token::Hover)
            } else {
                Color::TRANSPARENT
            };
            ui.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(8.0, 5.0), gap: 8.0, align: Align::Center, ..Default::default() },
                Some(rid),
                Sense::CLICK,
                CursorIcon::Pointer,
                false,
                Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(6.0), bg, 0.0, Color::TRANSPARENT))),
            );
            ui.a11y(rid, || A11y::new(Role::CheckBox).label(it.as_str()).toggled(is_on));
            let cb_changed = ui.scope(("cb", idx), |ui| {
                let r = ui.checkbox("", &mut is_on).show();
                ui.a11y_update(r.id, |a| a.hidden = true);
                r.changed
            });
            ui.text_leaf(it.clone(), ui.text_style(TextRole::Body), ui.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
            ui.end();
            if r_ix.clicked && !cb_changed {
                is_on = !is_on;
            }
            if is_on && !selected.contains(it) {
                selected.push(it.clone());
            } else if !is_on {
                selected.retain(|x| x != it);
            }
        }
        if !shown_any {
            ui.text_leaf("No items", ui.text_style(TextRole::Caption), ui.color(Token::TextMuted), false, false, Style { pad: Sides::all(10.0), ..Default::default() });
        }
    });
    ui.end();
    ui.end();
}

show_on_drop!(TransferList);

// ========================================================= ReorderableList

/// Item in [`Ui::reorderable_list`].
#[derive(Clone, Debug)]
pub struct ReorderItem {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub badge: Option<String>,
}

impl ReorderItem {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        ReorderItem { id: id.into(), title: title.into(), subtitle: None, badge: None }
    }
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.subtitle = Some(s.into());
        self
    }
    pub fn badge(mut self, b: impl Into<String>) -> Self {
        self.badge = Some(b.into());
        self
    }
}

#[derive(Default)]
struct ReorderMem {
    dragging_id: Option<String>,
}

/// Sortable priority list with drag-handle pointer reordering, keyboard
/// `ArrowUp` / `ArrowDown` swaps, and `↑` / `↓` step controls.
pub struct ReorderableList<'u> {
    ui: &'u mut Ui,
    key: String,
    items: &'u mut Vec<ReorderItem>,
    shown: bool,
}

impl<'u> ReorderableList<'u> {
    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("reorder_list", &self.key));
        let r_card = ui.look().card.radius;
        let bg_c = ui.color(Token::Surface);
        let bd_c = ui.color(Token::BorderSubtle);
        let mo = ui.motion();

        ui.begin(
            Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(6.0), gap: 4.0, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        let mut swap: Option<(usize, usize)> = None;
        let len = self.items.len();
        let dragging_id = ui.mem::<ReorderMem>(id).dragging_id.clone();

        for idx in 0..len {
            let item = &self.items[idx];
            let rid = id.with(("r_it", &item.id));
            let r_ix = ui.interact(
                rid,
                Sense { click: true, drag: true, focusable: true, scroll: false },
                false,
            );
            if r_ix.press_started {
                ui.mem::<ReorderMem>(id).dragging_id = Some(item.id.clone());
            }
            if ui.mem::<ReorderMem>(id).dragging_id.as_deref() == Some(item.id.as_str()) {
                if r_ix.held {
                    if let Some(p) = r_ix.pointer {
                        let row_h = r_ix.rect.height().max(36.0);
                        if p.y < r_ix.rect.min.y - row_h * 0.35 && idx > 0 {
                            swap = Some((idx, idx - 1));
                        } else if p.y > r_ix.rect.max.y + row_h * 0.35 && idx + 1 < len {
                            swap = Some((idx, idx + 1));
                        }
                    }
                } else {
                    ui.mem::<ReorderMem>(id).dragging_id = None;
                }
            } else if dragging_id.as_deref() == Some(item.id.as_str()) && !r_ix.pressed {
                ui.mem::<ReorderMem>(id).dragging_id = None;
            }

            if r_ix.focused {
                if ui.key_pressed_for(rid, Key::ArrowUp) && idx > 0 {
                    swap = Some((idx, idx - 1));
                } else if ui.key_pressed_for(rid, Key::ArrowDown) && idx + 1 < len {
                    swap = Some((idx, idx + 1));
                }
            }

            let is_drag = ui.mem::<ReorderMem>(id).dragging_id.as_deref() == Some(item.id.as_str());
            let lift = ui.spring(rid, 0, if is_drag { 1.0 } else { 0.0 }, mo.snappy);
            let hov = ui.fade(rid, 1, r_ix.hovered, mo.hover_in, mo.hover_out, Ease::Out);
            let f_ring = ui.fade(rid, 2, r_ix.focus_visible, mo.hover_in, mo.hover_out, Ease::Out);

            let row_bg = ui
                .color(Token::SurfaceMuted)
                .multiply_alpha(0.35 + 0.45 * hov)
                .mix(ui.color(Token::AccentSubtle), lift);
            let row_bd = ui.color(Token::BorderSubtle).multiply_alpha(hov).mix(ui.color(Token::Accent), lift);
            let ring_c = ui.color(Token::FocusRing).multiply_alpha(f_ring);

            ui.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(10.0, 8.0), gap: 10.0, align: Align::Center, ..Default::default() },
                Some(rid),
                Sense { click: true, drag: true, focusable: true, scroll: false },
                if is_drag { CursorIcon::Grabbing } else { CursorIcon::Grab },
                false,
                Some(Box::new(move |cx, r| {
                    cx.painter.rrect(r, Corners::all(8.0), row_bg, 1.0, row_bd);
                    if f_ring > 0.005 {
                        cx.painter.ring(r.expand(1.5), Corners::all(9.5), 1.5, ring_c);
                    }
                })),
            );
            ui.icon_leaf("grip-vertical", 15.0, ui.color(Token::TextMuted));
            let mut st_idx = ui.text_style(TextRole::Mono);
            st_idx.tabular = true;
            ui.text_leaf(format!("{:02}", idx + 1), st_idx, ui.color(Token::TextMuted), false, false, Style::default());

            ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 2.0, justify: Justify::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            ui.text_leaf(item.title.clone(), ui.text_style(TextRole::BodyStrong), ui.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
            if let Some(ref sub) = item.subtitle {
                ui.text_leaf(sub.clone(), ui.text_style(TextRole::Caption), ui.color(Token::TextMuted), false, true, Style { width: Size::FILL, ..Default::default() });
            }
            ui.end();

            if let Some(ref b) = item.badge {
                ui.badge(b, Tone::Neutral);
            }
            let up_click = ui.scope(("up", &item.id), |ui| ui.icon_button("chevron-up").small().disabled(idx == 0).tooltip("Move up").clicked());
            let dn_click = ui.scope(("dn", &item.id), |ui| ui.icon_button("chevron-down").small().disabled(idx + 1 >= len).tooltip("Move down").clicked());
            if up_click && idx > 0 {
                swap = Some((idx, idx - 1));
            } else if dn_click && idx + 1 < len {
                swap = Some((idx, idx + 1));
            }
            ui.end();
        }
        ui.end();

        let mut changed = false;
        if let Some((a, b)) = swap {
            self.items.swap(a, b);
            changed = true;
        }
        let rect = ui.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, changed, ..Default::default() }
    }
}

show_on_drop!(ReorderableList);

impl Ui {
    /// Click-to-edit inline text or heading.
    pub fn editable_text<'u>(&'u mut self, key: impl Into<String>, value: &'u mut String) -> EditableText<'u> {
        EditableText {
            ui: self,
            key: key.into(),
            value,
            placeholder: "Click to edit…".into(),
            role: TextRole::BodyStrong,
            disabled: false,
            shown: false,
        }
    }

    /// Formatted currency field with thousands grouping and step buttons.
    pub fn money_field<'u>(&'u mut self, value: &'u mut f64) -> MoneyField<'u> {
        MoneyField {
            ui: self,
            value,
            currency: Currency::Usd,
            currency_mut: None,
            label: None,
            helper: None,
            min: 0.0,
            max: 1_000_000_000.0,
            step: 50.0,
            disabled: false,
            width: Size::Hug,
            shown: false,
        }
    }

    /// Pattern-masked input (`#` = digit, `A` = letter, `*` = alphanumeric).
    pub fn masked_input<'u>(&'u mut self, value: &'u mut String, mask: impl Into<String>) -> MaskedInput<'u> {
        MaskedInput {
            ui: self,
            value,
            mask: mask.into(),
            label: None,
            helper: None,
            icon: None,
            disabled: false,
            width: Size::Hug,
            shown: false,
        }
    }

    /// `@mention` input with floating candidate autocomplete.
    pub fn mention_input<'u>(&'u mut self, value: &'u mut String, candidates: &'u [MentionCandidate]) -> MentionInput<'u> {
        MentionInput {
            ui: self,
            value,
            candidates,
            label: None,
            placeholder: "Write a comment — type @ to mention…".into(),
            shown: false,
        }
    }

    /// Discrete snapped slider with labeled tick stops.
    pub fn step_slider<'u, 's>(
        &'u mut self,
        label: impl Into<String>,
        index: &'u mut usize,
        stops: &'s [&'s str],
    ) -> StepSlider<'u, 's> {
        StepSlider {
            ui: self,
            label: label.into(),
            index,
            stops,
            disabled: false,
            shown: false,
        }
    }

    /// Dual-pane available ↔ assigned shuttle list.
    pub fn transfer_list<'u>(
        &'u mut self,
        key: impl Into<String>,
        available: &'u mut Vec<String>,
        assigned: &'u mut Vec<String>,
    ) -> TransferList<'u> {
        TransferList {
            ui: self,
            key: key.into(),
            available,
            assigned,
            left_title: "Available".into(),
            right_title: "Assigned".into(),
            height: 176.0,
            shown: false,
        }
    }

    /// Sortable priority list with drag grip and `↑` / `↓` controls.
    pub fn reorderable_list<'u>(&'u mut self, key: impl Into<String>, items: &'u mut Vec<ReorderItem>) -> ReorderableList<'u> {
        ReorderableList { ui: self, key: key.into(), items, shown: false }
    }
}
