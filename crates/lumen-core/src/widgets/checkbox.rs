//! Checkbox (spec section 5).

use crate::anim::Ease;
use crate::input::CursorIcon;
use crate::layout::{Align, Dir, Style};
use crate::math::{vec2, Corners, Rect};
use crate::theme::{durations, springs, TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

use super::{blend3, key_activate, neutral_layers, show_on_drop};

#[must_use = "checkboxes are added when dropped or shown"]
pub struct Checkbox<'u> {
    ui: &'u mut Ui,
    label: String,
    value: &'u mut bool,
    indeterminate: bool,
    description: Option<String>,
    disabled: bool,
    shown: bool,
}

impl<'u> Checkbox<'u> {
    /// Show the mixed state (for a parent summarizing children). Clicking
    /// it makes it checked.
    pub fn indeterminate(mut self, on: bool) -> Self {
        self.indeterminate = on;
        self
    }
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("checkbox", &self.label));
        let ix = ui.interact(id, Sense::CLICK, self.disabled);
        let (key_act, key_held) = key_activate(ui, &ix, false);
        let mut changed = false;
        if (ix.clicked || key_act) && !self.disabled {
            *self.value = if self.indeterminate { true } else { !*self.value };
            changed = true;
        }
        let indet = self.indeterminate && !changed;
        let checked = *self.value && !indet;
        let filled = checked || indet;
        let (h, p) = ui.state_weights(&ix, key_held);
        let sel = ui.fade(id, 3, filled, durations::SELECTED, durations::SELECTED, Ease::Out);
        let pressed = (ix.pressed || key_held) && !self.disabled;
        let inset = ui.spring(id, 0, if pressed { 1.0 } else { 0.0 }, if pressed { springs::SNAPPY } else { springs::BOUNCY });
        let check_wipe = ui.spring(id, 1, if checked { 1.0 } else { 0.0 }, springs::SNAPPY);
        let check_alpha = ui.fade(id, 4, checked, 0.0, durations::MARK_OUT, Ease::Linear);
        let bar_wipe = ui.spring(id, 2, if indet { 1.0 } else { 0.0 }, springs::SNAPPY);
        let bar_alpha = ui.fade(id, 5, indet, 0.0, durations::MARK_OUT, Ease::Linear);
        let (ring_a, ring_gap) = ui.focus_anim(&ix);
        let c = |t| ui.color(t);
        let (fill, border, mark, label_c) = if self.disabled {
            if filled {
                (c(Token::Track), Color::TRANSPARENT, c(Token::Surface), c(Token::TextDisabled))
            } else {
                (c(Token::SurfaceMuted), c(Token::Border), Color::TRANSPARENT, c(Token::TextDisabled))
            }
        } else {
            let surf = c(Token::Surface);
            let (_, surf_p) = neutral_layers(ui, surf);
            let un_fill = blend3(surf, surf, surf_p, h, p);
            let un_border = c(Token::BorderStrong).mix(c(Token::TextMuted), h.max(p));
            let on_fill = blend3(c(Token::Accent), c(Token::AccentHover), c(Token::AccentPressed), h, p);
            (un_fill.mix(on_fill, sel), un_border.mix(Color::TRANSPARENT, sel), c(Token::AccentForeground), c(Token::Foreground))
        };
        let ring = c(Token::FocusRing);
        let cursor = if self.disabled { CursorIcon::Default } else { CursorIcon::Pointer };
        let row = ui.begin(Style { dir: Dir::Row, gap: 8.0, align: Align::Start, ..Default::default() }, Some(id), Sense::CLICK, cursor, false, None);
        ui.set_hit_pad(row, 4.0);
        ui.leaf(
            Style::default(),
            vec2(16.0, 20.0),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let bx = cx.painter.snap_rect(Rect::from_xywh(r.min.x, r.min.y + 2.0, 16.0, 16.0));
                let bg = bx.expand(-inset);
                let bw = if border.a > 0.0 { 1.0 } else { 0.0 };
                cx.painter.rrect_unsnapped(cx.painter.snap_rect(bg), Corners::all(4.0 - inset), [fill; 2], bw, border);
                if check_alpha > 0.0 {
                    // Left-to-right wipe while checking; alpha fade when unchecking.
                    let w = if checked { 14.0 * check_wipe } else { 14.0 };
                    let icon_r = Rect::from_center_size(bx.center(), vec2(14.0, 14.0));
                    cx.painter.push_clip(Rect::from_xywh(icon_r.min.x, icon_r.min.y - 1.0, w, 16.0));
                    cx.icon("check", bx.center(), 14.0, mark.multiply_alpha(check_alpha));
                    cx.painter.pop_clip();
                }
                if bar_alpha > 0.0 {
                    let w = if indet { 8.0 * bar_wipe } else { 8.0 };
                    let bar = Rect::from_center_size(bx.center(), vec2(w, 2.0));
                    cx.painter.rrect_unsnapped(bar, Corners::all(1.0), [mark.multiply_alpha(bar_alpha); 2], 0.0, Color::TRANSPARENT);
                }
                if ring_a > 0.0 {
                    cx.defer(bx, Box::new(move |cx, r| cx.focus_ring(r, 4.0, ring_a, ring_gap, ring)));
                }
            })),
        );
        let has_text = !self.label.is_empty() || self.description.is_some();
        if has_text {
            ui.begin(Style { gap: 2.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            if !self.label.is_empty() {
                ui.text_leaf(std::mem::take(&mut self.label), TextRole::Body.style(), label_c, true, false, Style::default());
            }
            if let Some(d) = self.description.take() {
                let dc = if self.disabled { ui.color(Token::TextDisabled) } else { ui.color(Token::TextSecondary) };
                ui.text_leaf(d, TextRole::Body.style(), dc, true, false, Style::default());
            }
            ui.end();
        }
        ui.end();
        Response { id, rect: ix.rect, hovered: ix.hovered, clicked: changed, changed, focused: ix.focused }
    }
}

show_on_drop!(Checkbox);

impl Ui {
    pub fn checkbox<'u>(&'u mut self, label: impl Into<String>, value: &'u mut bool) -> Checkbox<'u> {
        Checkbox { ui: self, label: label.into(), value, indeterminate: false, description: None, disabled: false, shown: false }
    }
}
