//! Numeric input with drag-to-scrub, inline text editing, arrow-key stepping,
//! prefix/suffix units and optional `−` / `+` buttons.

use std::ops::RangeInclusive;

use super::field::{Chrome, EditOpts};
use super::show_on_drop;
use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Sides};
use crate::theme::{ControlSize, TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

#[derive(Default)]
struct NumMem {
    text: String,
    editing: bool,
    was_focused: bool,
    scrub_origin_x: f32,
    scrub_origin_v: f64,
    scrubbing: bool,
}

fn auto_decimals(step: f64) -> usize {
    let s = step.abs();
    if s == 0.0 || (s - s.round()).abs() < 1e-6 {
        0
    } else if (s * 10.0 - (s * 10.0).round()).abs() < 1e-5 {
        1
    } else if (s * 100.0 - (s * 100.0).round()).abs() < 1e-4 {
        2
    } else {
        3
    }
}

fn format_num(v: f64, decimals: usize) -> String {
    let clean = if v.abs() < 1e-12 { 0.0 } else { v };
    format!("{clean:.decimals$}")
}

/// Numeric field built by [`Ui::number_field`]. Drag the leading scrub handle
/// (or the label) horizontally to scrub; click the value to type directly; use
/// `↑` / `↓` or `−` / `+` to step (`Shift` = 10×, `Alt` = 0.1× while scrubbing).
pub struct NumberField<'u> {
    ui: &'u mut Ui,
    value: &'u mut f64,
    range: RangeInclusive<f64>,
    step: f64,
    decimals: Option<usize>,
    scrub_px: f32,
    label: Option<String>,
    prefix: Option<String>,
    suffix: Option<String>,
    icon: Option<String>,
    controls: bool,
    helper: Option<String>,
    error: Option<String>,
    disabled: bool,
    width: Size,
    id_key: Option<String>,
    shown: bool,
}

impl<'u> NumberField<'u> {
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// Inclusive bounds (default unbounded).
    pub fn range(mut self, r: RangeInclusive<f64>) -> Self {
        self.range = r;
        self
    }
    /// Step size for `↑` / `↓`, `−` / `+` and normal scrubbing (default `1.0`).
    pub fn step(mut self, s: f64) -> Self {
        self.step = s.abs().max(1e-6);
        self
    }
    /// Fixed decimal places shown at rest (defaults from `step`).
    pub fn decimals(mut self, d: usize) -> Self {
        self.decimals = Some(d.min(8));
        self
    }
    /// Horizontal drag distance in logical pixels per step (default `4.0`).
    pub fn scrub_pixels(mut self, px: f32) -> Self {
        self.scrub_px = px.max(1.0);
        self
    }
    /// Leading prefix inside the scrub handle (e.g. `"$"`, `"X"`, `"W"`).
    pub fn prefix(mut self, p: impl Into<String>) -> Self {
        self.prefix = Some(p.into());
        self
    }
    /// Trailing unit label (e.g. `"ms"`, `"px"`, `"%"`, `"GB"`).
    pub fn suffix(mut self, s: impl Into<String>) -> Self {
        self.suffix = Some(s.into());
        self
    }
    /// Leading Lucide icon inside the scrub handle.
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = Some(name.to_owned());
        self
    }
    /// Show or hide the trailing `−` and `+` buttons (default `true`).
    pub fn controls(mut self, on: bool) -> Self {
        self.controls = on;
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
    pub fn width(mut self, w: f32) -> Self {
        self.width = Size::Fixed(w.max(110.0));
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
        let key = self
            .id_key
            .clone()
            .or_else(|| self.label.clone())
            .or_else(|| self.prefix.clone())
            .or_else(|| self.suffix.clone())
            .unwrap_or_else(|| "num".into());
        let decimals = self.decimals.unwrap_or_else(|| auto_decimals(self.step));
        let (min_v, max_v) = (*self.range.start(), *self.range.end());
        let ui = &mut *self.ui;
        let id = ui.make_id(("number_field", key));
        let edit_id = id.with("edit");
        let scrub_id = id.with("scrub");
        let label_id = id.with("label");
        let minus_id = id.with("minus");
        let plus_id = id.with("plus");
        let mo = ui.motion();
        let m = ui.control_metrics(ControlSize::Md);
        let mods = ui.modifiers();

        let scrub_sense = Sense { click: true, drag: true, focusable: false, scroll: false };
        let scrub_ix = ui.interact(scrub_id, scrub_sense, self.disabled);
        let label_ix = if self.label.is_some() {
            ui.interact(label_id, Sense { click: false, drag: true, focusable: false, scroll: false }, self.disabled)
        } else {
            Default::default()
        };

        let mut changed = false;
        let before_all = *self.value;

        // Horizontal drag-to-scrub on either the scrub handle or the field label.
        let scrub_started = scrub_ix.press_started || label_ix.press_started;
        let scrub_held = scrub_ix.held || label_ix.held;
        if scrub_started {
            if let Some(p) = ui.pointer() {
                let st = ui.mem::<NumMem>(id);
                st.scrub_origin_x = p.x;
                st.scrub_origin_v = *self.value;
                st.scrubbing = false;
                st.editing = false;
            }
        }
        let mut active_scrub = false;
        if scrub_held && !self.disabled {
            if let Some(p) = ui.pointer() {
                let (ox, ov, was_scrub) = {
                    let st = ui.mem::<NumMem>(id);
                    (st.scrub_origin_x, st.scrub_origin_v, st.scrubbing)
                };
                let dx = p.x - ox;
                if was_scrub || dx.abs() >= 2.0 {
                    active_scrub = true;
                    ui.mem::<NumMem>(id).scrubbing = true;
                    ui.surrender_focus(edit_id);
                    let mult = if mods.shift { 10.0 } else if mods.alt { 0.1 } else { 1.0 };
                    let eff_step = self.step * mult;
                    let raw_steps = (dx / self.scrub_px) as f64;
                    let delta = if mods.alt { raw_steps * eff_step } else { raw_steps.round() * eff_step };
                    let next = (ov + delta).clamp(min_v, max_v);
                    let factor = 10f64.powi(decimals.max(if mods.alt { decimals + 1 } else { decimals }) as i32);
                    let next = (next * factor).round() / factor;
                    if (next - *self.value).abs() > 1e-9 {
                        *self.value = next;
                        changed = true;
                    }
                }
            }
        } else {
            let was_scrub = ui.mem::<NumMem>(id).scrubbing;
            ui.mem::<NumMem>(id).scrubbing = false;
            // A plain click (no drag) on the scrub handle focuses the editor.
            if scrub_ix.clicked && !was_scrub && !self.disabled {
                ui.request_focus(edit_id);
            }
        }

        let focused = ui.focused() == Some(edit_id);
        let gained = focused && !ui.mem::<NumMem>(id).was_focused;

        // Sync formatted string when not actively typing.
        {
            let formatted = format_num(*self.value, decimals);
            let st = ui.mem::<NumMem>(id);
            if !st.editing && st.text != formatted {
                st.text = formatted;
            }
        }
        let mut text = std::mem::take(&mut ui.mem::<NumMem>(id).text);
        if gained {
            ui.inline_edit_select_all(edit_id, &text);
        }

        let hovered = ui.pointer().is_some_and(|p| ui.rect_of(id).is_some_and(|r| r.contains(p))) || label_ix.hovered;
        let fill_w = matches!(self.width, Size::Fill(_));
        let fw = match self.width {
            Size::Hug => Size::Fixed(200.0),
            w => w,
        };

        ui.begin(Style { width: if fill_w { Size::FILL } else { Size::Hug }, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if let Some(l) = self.label.take() {
            let lc = if self.disabled { ui.color(Token::TextDisabled) } else { ui.color(Token::Foreground) };
            let st = ui.text_style(TextRole::Label);
            let g = ui.galley(&l, &st);
            ui.leaf(
                Style::default(),
                vec2(g.width.ceil(), g.height),
                Some(label_id),
                Sense { click: false, drag: true, focusable: false, scroll: false },
                if self.disabled { CursorIcon::Default } else { CursorIcon::ResizeHorizontal },
                Some(Box::new(move |cx, r| cx.painter.text(cx.text, &g, r.min, lc))),
            );
        }

        ui.field_begin(
            id,
            Chrome { hovered, focused: focused || active_scrub, error: self.error.is_some(), disabled: self.disabled },
            Style { dir: Dir::Row, width: fw, height: Size::Fixed(m.height), pad: Sides { left: 4.0, right: if self.controls { 3.0 } else { 10.0 }, top: 0.0, bottom: 0.0 }, gap: 6.0, align: Align::Center, ..Default::default() },
        );

        // Leading scrub handle: prefix text, custom icon, or left-right scrub icon.
        let scrub_hot = ui.fade(scrub_id, 1, (scrub_ix.hovered || label_ix.hovered || active_scrub) && !self.disabled, mo.hover_in, mo.hover_out, Ease::Linear);
        let scrub_bg = ui.color(Token::Hover).multiply_alpha(scrub_hot * 0.9).mix(ui.color(Token::AccentSubtle), if active_scrub { 0.85 } else { 0.0 });
        let scrub_fg = if self.disabled {
            ui.color(Token::TextDisabled)
        } else {
            ui.color(Token::TextMuted).mix(ui.color(Token::AccentStrong), if active_scrub { 1.0 } else { scrub_hot * 0.8 })
        };
        let rad_in = (m.radius - 3.0).max(3.0);
        if let Some(pref) = &self.prefix {
            let mut st = ui.text_style(TextRole::Label);
            st.weight = 600;
            let g = ui.galley(pref, &st);
            let hw = (g.width + 14.0).max(26.0);
            ui.leaf(
                Style::default(),
                vec2(hw, m.height - 8.0),
                Some(scrub_id),
                scrub_sense,
                if self.disabled { CursorIcon::Default } else { CursorIcon::ResizeHorizontal },
                Some(Box::new(move |cx, r| {
                    if scrub_bg.a > 0.0 {
                        cx.painter.rrect(r, Corners::all(rad_in), scrub_bg, 0.0, Color::TRANSPARENT);
                    }
                    cx.painter.text(cx.text, &g, vec2((r.center().x - g.width / 2.0).round(), (r.center().y - g.height / 2.0).round()), scrub_fg);
                })),
            );
        } else {
            let ic_name = self.icon.clone().unwrap_or_else(|| "move-horizontal".into());
            ui.leaf(
                Style::default(),
                vec2(26.0, m.height - 8.0),
                Some(scrub_id),
                scrub_sense,
                if self.disabled { CursorIcon::Default } else { CursorIcon::ResizeHorizontal },
                Some(Box::new(move |cx, r| {
                    if scrub_bg.a > 0.0 {
                        cx.painter.rrect(r, Corners::all(rad_in), scrub_bg, 0.0, Color::TRANSPARENT);
                    }
                    cx.icon(&ic_name, r.center(), 15.0, scrub_fg);
                })),
            );
        }

        // Inline numeric text editor.
        let mut o = EditOpts::new(ui);
        o.style = ui.text_style(TextRole::Body);
        o.style.weight = 500;
        o.style.line_height = (20.0 * ui.look().text_scale).round();
        o.disabled = self.disabled;
        o.min_w = 36.0;
        o.accept = Some(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | ',' | ' '));
        let ed = ui.inline_edit(edit_id, &mut text, o);
        let a_label = self.label.clone().or_else(|| self.id_key.clone());
        let (cur, step) = (*self.value, self.step);
        ui.a11y_update(edit_id, |a| {
            a.role = crate::a11y::Role::SpinButton;
            if a.label.is_none() {
                a.label = a_label;
            }
            if min_v.is_finite() && max_v.is_finite() {
                a.numeric = Some(crate::a11y::Numeric { value: cur, min: min_v, max: max_v, step: Some(step) });
            }
            a.actions.increment = true;
        });

        if let Some(suf) = &self.suffix {
            let st = ui.text_style(TextRole::Label);
            let sc = if self.disabled { ui.color(Token::TextDisabled) } else { ui.color(Token::TextMuted) };
            ui.text_leaf(suf.clone(), st, sc, false, false, Style::default());
        }

        let mut step_dir = 0.0f64;
        let (a_st, _) = ui.a11y_step(edit_id);
        if a_st != 0 {
            step_dir = a_st.signum() as f64;
        }
        if self.controls {
            let line = ui.color(Token::BorderSubtle);
            ui.leaf(Style::default(), vec2(1.0, m.height - 12.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.rect(r, line))));
            for (bid, icon, at_limit, dir) in [(minus_id, "minus", *self.value <= min_v, -1.0), (plus_id, "plus", *self.value >= max_v, 1.0)] {
                let dis = self.disabled || at_limit;
                let bix = ui.interact(bid, Sense { click: true, drag: false, focusable: false, scroll: false }, dis);
                if bix.clicked && !dis {
                    step_dir = dir;
                }
                let bh = ui.fade(bid, 1, bix.hovered && !dis, mo.hover_in, mo.hover_out, Ease::Linear);
                let press = ui.spring(bid, 2, if bix.pressed { 1.0 } else { 0.0 }, mo.snappy);
                let bg = ui.color(Token::Hover).multiply_alpha(bh);
                let ic = if dis {
                    ui.color(Token::TextDisabled)
                } else {
                    ui.color(Token::TextSecondary).mix(ui.color(Token::Foreground), bh)
                };
                ui.leaf(
                    Style::default(),
                    vec2(24.0, m.height - 8.0),
                    Some(bid),
                    Sense { click: true, drag: false, focusable: false, scroll: false },
                    if dis { CursorIcon::Default } else { CursorIcon::Pointer },
                    Some(Box::new(move |cx, r| {
                        if bg.a > 0.0 {
                            cx.painter.rrect(r, Corners::all(rad_in), bg, 0.0, Color::TRANSPARENT);
                        }
                        cx.icon(icon, r.center(), 14.0 * (1.0 - 0.12 * press), ic);
                    })),
                );
            }
        }
        ui.end();

        if ui.clicked_id(id) && !self.disabled {
            ui.request_focus(edit_id);
        }

        if ed.changed {
            ui.mem::<NumMem>(id).editing = true;
        }
        if ed.key(Key::ArrowUp) {
            step_dir = 1.0;
        } else if ed.key(Key::ArrowDown) {
            step_dir = -1.0;
        }

        let parse_text = |raw: &str| -> Option<f64> {
            let cleaned: String = raw.chars().filter(|c| !matches!(c, ',' | ' ')).collect();
            cleaned.parse::<f64>().ok().filter(|n| n.is_finite())
        };

        if step_dir != 0.0 {
            let base = if ui.mem::<NumMem>(id).editing { parse_text(&text).unwrap_or(*self.value) } else { *self.value };
            let mult = if mods.shift { 10.0 } else { 1.0 };
            let next = (base + step_dir * self.step * mult).clamp(min_v, max_v);
            let factor = 10f64.powi(decimals as i32);
            let next = (next * factor).round() / factor;
            if (next - *self.value).abs() > 1e-9 {
                *self.value = next;
                changed = true;
            }
            text = format_num(*self.value, decimals);
            ui.inline_edit_caret_end(edit_id, &text);
            ui.mem::<NumMem>(id).editing = false;
        }

        let still_focused = ui.focused() == Some(edit_id);
        let blurred = !still_focused && ui.mem::<NumMem>(id).was_focused;
        if ed.key(Key::Enter) || blurred {
            if ui.mem::<NumMem>(id).editing {
                if let Some(parsed) = parse_text(&text) {
                    let clamped = parsed.clamp(min_v, max_v);
                    if (clamped - *self.value).abs() > 1e-9 {
                        *self.value = clamped;
                        changed = true;
                    }
                }
                ui.mem::<NumMem>(id).editing = false;
            }
            text = format_num(*self.value, decimals);
            ui.inline_edit_caret_end(edit_id, &text);
        }
        if ed.key(Key::Escape) {
            ui.mem::<NumMem>(id).editing = false;
            text = format_num(*self.value, decimals);
            ui.inline_edit_select_all(edit_id, &text);
        }

        ui.field_message(id, self.helper.as_deref(), self.error.as_deref(), None, self.disabled);
        ui.end();

        let _ = before_all;
        let st = ui.mem::<NumMem>(id);
        st.text = text;
        st.was_focused = still_focused;

        let rect = ui.rect_of(id).unwrap_or_default();
        Response { id, rect, hovered, clicked: ed.ix.clicked || scrub_ix.clicked, changed, focused: focused || active_scrub }
    }
}

show_on_drop!(NumberField);

impl Ui {
    /// Numeric field bound to `value`. Drag the left scrub handle or field
    /// label horizontally to scrub (`Shift` = 10×, `Alt` = 0.1×), click the
    /// number to edit inline, or step with `↑` / `↓` and `−` / `+`.
    pub fn number_field<'u>(&'u mut self, value: &'u mut f64) -> NumberField<'u> {
        NumberField {
            ui: self,
            value,
            range: f64::NEG_INFINITY..=f64::INFINITY,
            step: 1.0,
            decimals: None,
            scrub_px: 4.0,
            label: None,
            prefix: None,
            suffix: None,
            icon: None,
            controls: true,
            helper: None,
            error: None,
            disabled: false,
            width: Size::Hug,
            id_key: None,
            shown: false,
        }
    }
}
