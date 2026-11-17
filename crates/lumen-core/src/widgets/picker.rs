//! Accent picker: eight swatches that switch the theme accent (gallery spec 11.1).

use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Style};
use crate::math::{vec2, Rect};
use crate::theme::{springs, Accent, Token};
use crate::ui::{Placement, Response, Sense, TooltipSpec, Ui};
use crate::Color;

impl Ui {
    /// Eight accent swatches; clicking one blends the theme to that accent.
    pub fn accent_picker(&mut self) -> Response {
        let mode = self.mode();
        let current = self.accent();
        let mut changed = false;
        let ring_focus = self.color(Token::FocusRing);
        let group = self.make_id("accent_picker");
        self.begin(Style { dir: Dir::Row, gap: 0.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let ids: Vec<_> = Accent::ALL.iter().map(|a| group.with(a.name())).collect();
        for (i, &acc) in Accent::ALL.iter().enumerate() {
            let id = ids[i];
            let ix = self.interact(id, Sense::CLICK, false);
            let (act, held) = super::key_activate(self, &ix, true);
            if ix.focused {
                if self.key_pressed_for(id, Key::ArrowRight) && i + 1 < ids.len() {
                    self.request_focus(ids[i + 1]);
                }
                if self.key_pressed_for(id, Key::ArrowLeft) && i > 0 {
                    self.request_focus(ids[i - 1]);
                }
            }
            if (ix.clicked || act) && acc != current {
                self.set_accent(acc);
                changed = true;
            }
            let selected = self.accent() == acc;
            let pressed = ix.pressed || held;
            let target = if pressed { 18.0 } else if ix.hovered { 22.0 } else { 20.0 };
            let size = self.spring(id, 0, target, if target > 20.0 { springs::BOUNCY } else { springs::SNAPPY });
            let sel_t = self.spring_ring(id, selected);
            let wipe = self.spring(id, 2, if selected { 1.0 } else { 0.0 }, springs::SNAPPY);
            let (focus_a, focus_gap) = self.focus_anim(&ix);
            let [fill, _, _, fg] = acc.colors(mode);
            // Swatches follow the theme blend: mix with the live palette for
            // the current accent so mode changes animate too.
            let fill = if acc == current { self.color(Token::Accent) } else { fill };
            let fg = if acc == current { self.color(Token::AccentForeground) } else { fg };
            // 28 hit area; swatches 8 apart => 20 + 8 = 28 pitch.
            self.leaf(
                Style::default(),
                vec2(28.0, 28.0),
                Some(id),
                Sense::CLICK,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    let c = r.center();
                    cx.painter.circle(c, size / 2.0, fill, 0.0, Color::TRANSPARENT);
                    if sel_t.0 > 0.0 && focus_a <= 0.0 {
                        let gap = sel_t.1;
                        let rr = Rect::from_center_size(c, vec2(20.0, 20.0)).expand(gap + 2.0);
                        cx.painter.circle(c, rr.width() / 2.0, Color::TRANSPARENT, 2.0, fill.multiply_alpha(sel_t.0));
                    }
                    if wipe > 0.0 {
                        let ir = Rect::from_center_size(c, vec2(12.0, 12.0));
                        cx.painter.push_clip(Rect::from_xywh(ir.min.x, ir.min.y - 1.0, 12.0 * wipe, 14.0));
                        cx.icon("check", c, 12.0, fg);
                        cx.painter.pop_clip();
                    }
                    if focus_a > 0.0 {
                        let rr = Rect::from_center_size(c, vec2(20.0, 20.0));
                        cx.defer(rr, Box::new(move |cx, r| cx.focus_ring(r, 10.0, focus_a, focus_gap, ring_focus)));
                    }
                })),
            );
            self.tooltip_for(id, TooltipSpec { text: acc.name().into(), shortcut: None, placement: Placement::Bottom, gap: 6.0 });
        }
        self.end();
        Response { id: group, changed, clicked: changed, ..Default::default() }
    }

    /// Selection ring animation like the focus ring: (alpha, gap).
    fn spring_ring(&mut self, id: crate::Id, on: bool) -> (f32, f32) {
        let a = self.fade(id, 10, on, 0.0, 0.1, Ease::Linear);
        let t = self.anims.spring_from(id, 11, 0.0, if on { 1.0 } else { 0.0 }, springs::SNAPPY).clamp(0.0, 1.0);
        if on {
            (t, 4.0 - 2.0 * t)
        } else {
            (a, 2.0)
        }
    }
}
