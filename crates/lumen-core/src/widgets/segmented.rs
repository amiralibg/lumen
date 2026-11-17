//! Segmented control: one choice from a few, with a pill that slides to
//! the selection.

use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{ControlSize, Elevation, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

impl Ui {
    /// A segmented control bound to `value`. `options` pairs each value
    /// with its label. Arrow keys move the selection.
    ///
    /// ```ignore
    /// ui.segmented(&mut s.view, &[(View::List, "List"), (View::Grid, "Grid")]);
    /// ```
    pub fn segmented<T: Copy + PartialEq>(&mut self, value: &mut T, options: &[(T, &str)]) -> Response {
        let key: Vec<&str> = options.iter().map(|o| o.1).collect();
        let group = self.make_id(("segmented", key));
        let m = self.control_metrics(ControlSize::Md);
        let mo = self.motion();
        let ids: Vec<_> = options.iter().map(|o| group.with(o.1)).collect();
        let old = *value;

        // Interaction first so the pill targets this frame's choice.
        let mut ixs = Vec::with_capacity(options.len());
        for (i, &(v, _)) in options.iter().enumerate() {
            let ix = self.interact(ids[i], Sense::CLICK, false);
            if ix.clicked {
                *value = v;
            }
            if ix.focused {
                let mut go = None;
                if (self.key_pressed_for(ids[i], Key::ArrowRight) || self.key_pressed_for(ids[i], Key::ArrowDown)) && i + 1 < ids.len() {
                    go = Some(i + 1);
                }
                if (self.key_pressed_for(ids[i], Key::ArrowLeft) || self.key_pressed_for(ids[i], Key::ArrowUp)) && i > 0 {
                    go = Some(i - 1);
                }
                if let Some(j) = go {
                    self.request_focus(ids[j]);
                    *value = options[j].0;
                }
                if self.key_pressed_for(ids[i], Key::Space) || self.key_pressed_for(ids[i], Key::Enter) {
                    *value = v;
                }
            }
            ixs.push(ix);
        }
        let changed = *value != old;
        let sel = options.iter().position(|o| o.0 == *value);
        let sel_id = sel.map(|i| ids[i]);
        let pressed_sel = sel.is_some_and(|i| ixs[i].pressed);
        let squeeze = self.spring(group, 0, if pressed_sel { 1.0 } else { 0.0 }, if pressed_sel { mo.snappy } else { mo.bouncy });

        let c = |t| self.color(t);
        let track = c(Token::Pressed);
        let dark = self.theme.darkness();
        let pill = c(Token::SurfaceRaised).over(Color::WHITE.with_alpha(0.12 * dark));
        let shadows = Elevation::Knob.shadows(dark, c(Token::ShadowInk));
        let radius = m.radius + 1.0;
        let inner_r = (m.radius - 1.0).max(0.0);
        let smooth = mo.smooth;
        let reduced = self.reduced_motion();
        let anim_key = group.with("pill");
        let ring = c(Token::FocusRing);

        let node = self.begin(
            Style { dir: Dir::Row, height: Size::Fixed(m.height + 4.0), pad: Sides::all(2.0), gap: 0.0, align: Align::Center, ..Default::default() },
            Some(group),
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );
        self.set_paint(
            node,
            Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(radius), track, 0.0, Color::TRANSPARENT);
                let Some(s) = sel_id.and_then(|id| cx.rects.get(&id).copied()) else { return };
                let (x, w) = if reduced {
                    (s.min.x, s.width())
                } else {
                    let first = cx.anims.spring_state(anim_key, 0).is_none();
                    let x = cx.anims.spring_from(anim_key, 0, s.min.x, s.min.x, smooth);
                    let w = cx.anims.spring_from(anim_key, 1, s.width(), s.width(), smooth);
                    if first {
                        (s.min.x, s.width())
                    } else {
                        (x, w)
                    }
                };
                let pr = Rect::from_xywh(x, s.min.y, w, s.height()).expand(-0.75 * squeeze);
                cx.painter.shadows(pr, Corners::all(inner_r), &shadows);
                cx.painter.rrect_unsnapped(pr, Corners::all(inner_r), [pill; 2], 0.0, Color::TRANSPARENT);
            }),
        );
        for (i, &(_, label)) in options.iter().enumerate() {
            let ix = ixs[i];
            let selected = sel == Some(i);
            let h = self.fade(ix.id, 1, ix.hovered && !selected, mo.hover_in, mo.hover_out, Ease::Linear);
            let st = self.fade(ix.id, 2, selected, mo.selected, mo.selected, Ease::Out);
            let (ring_a, ring_gap) = self.focus_anim(&ix);
            let text = self.color(Token::TextSecondary).mix(self.color(Token::Foreground), h.max(st));
            let hover_fill = self.color(Token::Hover).multiply_alpha(h);
            let ring_r = inner_r;
            self.begin(
                Style {
                    dir: Dir::Row,
                    height: Size::FILL,
                    min: vec2(m.height * 1.5, 0.0),
                    pad: Sides::xy(m.pad_x, 0.0),
                    align: Align::Center,
                    justify: Justify::Center,
                    ..Default::default()
                },
                Some(ix.id),
                Sense::CLICK,
                CursorIcon::Pointer,
                false,
                Some(Box::new(move |cx, r| {
                    if hover_fill.a > 0.0 {
                        cx.painter.rrect(r, Corners::all(ring_r), hover_fill, 0.0, Color::TRANSPARENT);
                    }
                    if ring_a > 0.0 {
                        cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, ring_r, ring_a, ring_gap, ring)));
                    }
                })),
            );
            self.text_leaf(label, m.text, text, false, true, Style::default());
            self.end();
        }
        self.end();
        let rect = self.rect_of(group).unwrap_or_default();
        Response { id: group, rect, hovered: ixs.iter().any(|i| i.hovered), clicked: changed, changed, focused: ixs.iter().any(|i| i.focused) }
    }
}
