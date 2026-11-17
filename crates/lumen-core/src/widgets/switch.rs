//! Switch (spec section 4).

use crate::input::CursorIcon;
use crate::layout::{Align, Dir, Style};
use crate::math::{lerp, vec2, Corners, Rect};
use crate::theme::{springs, Elevation, TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

use super::{key_activate, show_on_drop};

pub struct Switch<'u> {
    ui: &'u mut Ui,
    label: String,
    value: &'u mut bool,
    disabled: bool,
    shown: bool,
}

impl<'u> Switch<'u> {
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("switch", &self.label));
        let ix = ui.interact(id, Sense::CLICK, self.disabled);
        // Space toggles; Enter does nothing.
        let (key_act, key_held) = key_activate(ui, &ix, false);
        let mut changed = false;
        if (ix.clicked || key_act) && !self.disabled {
            *self.value = !*self.value;
            changed = true;
        }
        let on = *self.value;
        let t = ui.spring(id, 0, if on { 1.0 } else { 0.0 }, springs::SNAPPY);
        let h = ui.fade(id, 1, ix.hovered || ix.held, 0.06, 0.16, crate::anim::Ease::Linear);
        let pressing = (ix.pressed || key_held) && !self.disabled;
        let widen = ui.spring(id, 2, if pressing { 4.0 } else { 0.0 }, springs::SNAPPY);
        let (ring_a, ring_gap) = ui.focus_anim(&ix);
        let c = |tk| ui.color(tk);
        let (track_off, track_on, thumb_off, thumb_on, border, shadow) = if self.disabled {
            (c(Token::SurfaceMuted), c(Token::Track), c(Token::BorderStrong), c(Token::Surface), c(Token::Border), false)
        } else {
            (
                c(Token::Track).mix(c(Token::BorderStrong), h),
                c(Token::Accent).mix(c(Token::AccentHover), h),
                c(Token::Knob),
                c(Token::AccentForeground),
                Color::TRANSPARENT,
                true,
            )
        };
        let track = track_off.mix(track_on, t);
        let thumb = thumb_off.mix(thumb_on, t);
        let border_w = if self.disabled && !on { 1.0 } else { 0.0 };
        let shadows = if shadow { Elevation::Knob.shadows(ui.theme.darkness(), c(Token::ShadowInk)) } else { vec![] };
        let ring = c(Token::FocusRing);

        let cursor = if self.disabled { CursorIcon::Default } else { CursorIcon::Pointer };
        let has_label = !self.label.is_empty();
        let row = ui.begin(Style { dir: Dir::Row, gap: 8.0, align: Align::Center, ..Default::default() }, Some(id), Sense::CLICK, cursor, false, None);
        ui.set_hit_pad(row, 4.0);
        ui.leaf(
            Style::default(),
            vec2(36.0, 20.0),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let tr = cx.painter.snap_rect(r);
                cx.painter.rrect(tr, Corners::all(10.0), track, border_w, border);
                let w = 16.0 + widen;
                let x_off = tr.min.x + 2.0;
                let x_on = tr.max.x - 2.0 - w;
                let thumb_r = Rect::from_xywh(lerp(x_off, x_on, t), tr.min.y + 2.0, w, 16.0);
                cx.painter.shadows(thumb_r, Corners::all(8.0), &shadows);
                cx.painter.rrect_unsnapped(thumb_r, Corners::all(8.0), [thumb; 2], 0.0, Color::TRANSPARENT);
                if ring_a > 0.0 {
                    cx.defer(tr, Box::new(move |cx, r| cx.focus_ring(r, 10.0, ring_a, ring_gap, ring)));
                }
            })),
        );
        if has_label {
            let col = if self.disabled { ui.color(Token::TextDisabled) } else { ui.color(Token::Foreground) };
            ui.text_leaf(std::mem::take(&mut self.label), TextRole::Body.style(), col, false, true, Style::default());
        }
        ui.end();
        Response { id, rect: ix.rect, hovered: ix.hovered, clicked: changed, changed, focused: ix.focused }
    }
}

show_on_drop!(Switch);

impl Ui {
    /// An on/off switch. Pass `""` for no inline label (e.g. in settings rows).
    pub fn switch<'u>(&'u mut self, label: impl Into<String>, value: &'u mut bool) -> Switch<'u> {
        Switch { ui: self, label: label.into(), value, disabled: false, shown: false }
    }
}
