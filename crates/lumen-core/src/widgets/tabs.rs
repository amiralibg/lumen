//! Tabs with a sliding underline.

use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

impl Ui {
    /// A row of tabs bound to `value`. Draw the selected panel yourself
    /// below it. Left/Right move between tabs.
    pub fn tabs<T: Copy + PartialEq>(&mut self, value: &mut T, tabs: &[(T, &str)]) -> Response {
        let key: Vec<&str> = tabs.iter().map(|t| t.1).collect();
        let group = self.make_id(("tabs", key));
        let ids: Vec<_> = tabs.iter().map(|t| group.with(t.1)).collect();
        let old = *value;
        let mut ixs = vec![];
        for (i, &(v, _)) in tabs.iter().enumerate() {
            let ix = self.interact(ids[i], Sense::CLICK, false);
            if ix.clicked {
                *value = v;
            }
            if ix.focused {
                let n = ids.len();
                let go = if self.key_pressed_for(ids[i], Key::ArrowRight) {
                    Some((i + 1) % n)
                } else if self.key_pressed_for(ids[i], Key::ArrowLeft) {
                    Some((i + n - 1) % n)
                } else {
                    None
                };
                if let Some(j) = go {
                    self.request_focus(ids[j]);
                    *value = tabs[j].0;
                }
            }
            ixs.push(ix);
        }
        let sel = tabs.iter().position(|t| t.0 == *value);
        let sel_id = sel.map(|i| ids[i]);
        let mo = self.motion();
        let smooth = mo.smooth;
        let accent = self.color(Token::Accent);
        let line = self.color(Token::BorderSubtle);
        let reduced = self.reduced_motion();
        let key = group.with("ink");
        // Windows draws a short rounded pill; others a full-width underline.
        let short = self.look().nav.indicator;
        let node = self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 4.0, align: Align::End, ..Default::default() }, Some(group), Sense::NONE, CursorIcon::Default, false, None);
        self.set_paint_after(
            node,
            Box::new(move |cx, r| {
                cx.painter.rect(Rect::from_xywh(r.min.x, r.max.y - 1.0, r.width(), 1.0), line);
                let Some(s) = sel_id.and_then(|i| cx.rects.get(&i).copied()) else { return };
                let (x, w) = if reduced {
                    (s.min.x, s.width())
                } else {
                    let first = cx.anims.spring_state(key, 0).is_none();
                    let x = cx.anims.spring_from(key, 0, s.min.x, s.min.x, smooth);
                    let w = cx.anims.spring_from(key, 1, s.width(), s.width(), smooth);
                    if first {
                        (s.min.x, s.width())
                    } else {
                        (x, w)
                    }
                };
                let inset = (w * 0.5 - 8.0).max(0.0) * short;
                let h = 2.0 + short;
                let ir = Rect::from_xywh(x + 8.0 + inset, r.max.y - h, (w - 16.0 - inset * 2.0).max(8.0), h);
                cx.painter.rrect_unsnapped(ir, Corners::all(h / 2.0), [accent; 2], 0.0, Color::TRANSPARENT);
            }),
        );
        let st = self.text_style(TextRole::BodyStrong);
        let h = self.look().control.height[1] + 8.0;
        for (i, &(_, label)) in tabs.iter().enumerate() {
            let ix = ixs[i];
            let selected = sel == Some(i);
            let hv = self.fade(ix.id, 1, ix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
            let s = self.fade(ix.id, 2, selected, mo.selected, mo.selected, Ease::Out);
            let (ring_a, ring_gap) = self.focus_anim(&ix);
            let fg = self.color(Token::TextSecondary).mix(self.color(Token::Foreground), hv.max(s));
            let hover = self.color(Token::Hover).multiply_alpha(hv * (1.0 - s));
            let ring = self.color(Token::FocusRing);
            let rr = self.look().control.radius[0];
            self.begin(
                Style { dir: Dir::Row, height: Size::Fixed(h), pad: Sides { left: 12.0, right: 12.0, top: 0.0, bottom: 4.0 }, align: Align::Center, ..Default::default() },
                Some(ix.id),
                Sense::CLICK,
                CursorIcon::Pointer,
                false,
                Some(Box::new(move |cx, r| {
                    let hr = Rect { min: r.min + vec2(2.0, 4.0), max: r.max - vec2(2.0, 8.0) };
                    cx.painter.rrect(hr, Corners::all(rr), hover, 0.0, Color::TRANSPARENT);
                    if ring_a > 0.0 {
                        cx.defer(hr, Box::new(move |cx, r| cx.focus_ring(r, rr, ring_a, ring_gap, ring)));
                    }
                })),
            );
            self.text_leaf(label, st, fg, false, true, Style::default());
            self.end();
        }
        self.end();
        let rect = self.rect_of(group).unwrap_or_default();
        let changed = *value != old;
        Response { id: group, rect, clicked: changed, changed, hovered: ixs.iter().any(|i| i.hovered), focused: ixs.iter().any(|i| i.focused) }
    }
}
