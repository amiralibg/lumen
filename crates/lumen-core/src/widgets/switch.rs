//! Switch (spec section 4).

use crate::input::CursorIcon;
use crate::layout::{Align, Dir, Style};
use crate::math::{lerp, vec2, Corners, Rect};
use crate::theme::{Elevation, TextRole, Token};
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
        let lk = ui.look().switch;
        let mo = ui.motion();
        let t = ui.spring(id, 0, if on { 1.0 } else { 0.0 }, mo.snappy);
        let h = ui.fade(id, 1, ix.hovered || ix.held, mo.hover_in.max(0.04), mo.hover_out, crate::anim::Ease::Linear);
        let pressing = (ix.pressed || key_held) && !self.disabled;
        let widen = ui.spring(id, 2, if pressing { lk.press_widen } else { 0.0 }, mo.snappy);
        let grow = ui.spring(id, 3, if (ix.hovered || pressing) && !self.disabled { lk.hover_grow } else { 0.0 }, mo.bouncy);
        let (ring_a, ring_gap) = ui.focus_anim(&ix);
        let c = |tk| ui.color(tk);
        let outline = lk.outline_off;
        // Off state: a filled track with a white knob, or (Windows) an
        // outlined track with a dark knob.
        let filled_off = c(Token::Track).mix(c(Token::BorderStrong), h);
        let outlined_off = c(Token::Hover).multiply_alpha(h);
        let (track_off, track_on, thumb_off, thumb_on, border_off, shadow) = if self.disabled {
            (c(Token::SurfaceMuted), c(Token::Track), c(Token::BorderStrong), c(Token::Surface), c(Token::Border), false)
        } else {
            let knob_on = c(Token::Knob).mix(c(Token::AccentForeground), lk.knob_tint_on);
            (
                filled_off.mix(outlined_off, outline),
                c(Token::Accent).mix(c(Token::AccentHover), h),
                c(Token::Knob).mix(c(Token::TextSecondary).mix(c(Token::Foreground), h), outline),
                knob_on,
                c(Token::BorderStrong).mix(c(Token::TextSecondary), h).multiply_alpha(outline),
                true,
            )
        };
        let track = track_off.mix(track_on, t);
        let thumb = thumb_off.mix(thumb_on, t);
        let border = border_off.multiply_alpha(1.0 - t);
        let border_w = if (self.disabled && !on) || border.a > 0.0 { 1.0 } else { 0.0 };
        let knob_shadow = if shadow { lk.knob_shadow * (1.0 - outline * (1.0 - t)) } else { 0.0 };
        let shadows: Vec<_> = if knob_shadow > 0.0 {
            Elevation::Knob.shadows(ui.theme.darkness(), c(Token::ShadowInk)).into_iter().map(|mut s| {
                s.color = s.color.multiply_alpha(knob_shadow);
                s
            }).collect()
        } else {
            vec![]
        };
        let ring = c(Token::FocusRing);

        let cursor = if self.disabled { CursorIcon::Default } else { CursorIcon::Pointer };
        let has_label = !self.label.is_empty();
        let row = ui.begin(Style { dir: Dir::Row, gap: 8.0, align: Align::Center, ..Default::default() }, Some(id), Sense::CLICK, cursor, false, None);
        ui.set_hit_pad(row, 4.0);
        let (tw, th) = (lk.width, lk.height);
        ui.leaf(
            Style::default(),
            vec2(tw, th),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let tr = cx.painter.snap_rect(r);
                let tr_r = tr.height() / 2.0;
                cx.painter.rrect(tr, Corners::all(tr_r), track, border_w, border);
                let k = (lk.knob + grow).min(tr.height() - 2.0);
                let inset = (tr.height() - k) / 2.0;
                let w = k + widen;
                let x_off = tr.min.x + inset;
                let x_on = tr.max.x - inset - w;
                let thumb_r = Rect::from_xywh(lerp(x_off, x_on, t), tr.center().y - k / 2.0, w, k);
                if !shadows.is_empty() {
                    cx.painter.shadows(thumb_r, Corners::all(k / 2.0), &shadows);
                }
                cx.painter.rrect_unsnapped(thumb_r, Corners::all(k / 2.0), [thumb; 2], 0.0, Color::TRANSPARENT);
                if ring_a > 0.0 {
                    cx.defer(tr, Box::new(move |cx, r| cx.focus_ring(r, tr_r, ring_a, ring_gap, ring)));
                }
            })),
        );
        if has_label {
            let col = if self.disabled { ui.color(Token::TextDisabled) } else { ui.color(Token::Foreground) };
            let st = ui.text_style(TextRole::Body);
            ui.text_leaf(std::mem::take(&mut self.label), st, col, false, true, Style::default());
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
