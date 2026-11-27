//! Aspect-ratio picker: a crop preview over a sample photo and a row of
//! ratio chips, each drawn as its own shape. The crop frame springs to the
//! new shape, rule-of-thirds guides flash in while it moves, the selection
//! pill glides between chips, and the orientation toggle turns the shapes.

use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

#[derive(Default)]
struct AspectMem {
    changed_at: f64,
}

impl Ui {
    /// Pick one of `ratios`, given as `(label, width / height)`; a ratio of
    /// `0.0` means free (no fixed shape). `portrait` turns fixed ratios on
    /// their side.
    pub fn aspect_ratio_picker(&mut self, key: impl std::hash::Hash, selected: &mut usize, portrait: &mut bool, ratios: &[(&str, f32)]) -> Response {
        let id = self.make_id(("aspect", key));
        let n = ratios.len().max(1);
        *selected = (*selected).min(n - 1);
        let old = (*selected, *portrait);
        let mo = self.motion();
        let reduced = self.reduced_motion();
        let lk = self.look().clone();
        let now = self.time();
        let ids: Vec<_> = (0..ratios.len()).map(|i| id.with(("chip", i))).collect();

        let mut ixs = Vec::with_capacity(ratios.len());
        for i in 0..ratios.len() {
            let ix = self.interact(ids[i], Sense::CLICK, false);
            if ix.clicked {
                *selected = i;
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
                    *selected = j;
                }
                if self.key_pressed_for(ids[i], Key::Space) || self.key_pressed_for(ids[i], Key::Enter) {
                    *selected = i;
                }
            }
            ixs.push(ix);
        }
        let ratio_of = |i: usize, portrait: bool| -> f32 {
            let r = ratios.get(i).map(|r| r.1).unwrap_or(0.0);
            if r <= 0.0 {
                0.0
            } else if portrait && r > 1.0 || !portrait && r < 1.0 {
                1.0 / r
            } else {
                r
            }
        };
        let flippable = ratios.get(*selected).is_some_and(|r| r.1 > 0.0 && (r.1 - 1.0).abs() > 0.01);

        // Preview: a sample "photo" with the crop frame over it.
        let height = (lk.control.height[1] * 5.2).round();
        // A dawn landscape built from tokens, so it follows the theme.
        let bgc = self.color(Token::Background);
        let photo_top = self.color(Token::Series0).mix(bgc, 0.5);
        let photo_bot = self.color(Token::Warning).mix(bgc, 0.55);
        let sun = self.color(Token::Warning).mix(Color::WHITE, 0.2);
        let hill_a = self.color(Token::Success).mix(bgc, 0.35);
        let hill_b = self.color(Token::Success).mix(self.color(Token::Foreground), 0.3);
        let scrim = self.color(Token::Scrim);
        let frame_c = Color::WHITE;
        let gap = lk.control.gap[1].max(6.0);

        let changed_now = (*selected, *portrait) != old;
        if changed_now {
            self.mem::<AspectMem>(id).changed_at = now;
        }
        let since = (now - self.mem::<AspectMem>(id).changed_at) as f32;
        let moved_once = self.mem::<AspectMem>(id).changed_at > 0.0;
        let guides = self.fade(id, 3, since < 0.7 && moved_once, mo.hover_in, mo.fade_out * 3.0, Ease::Out);
        if since < 0.7 {
            self.request_repaint_after((0.7 - since) as f64);
        }

        self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: gap * 1.5, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);
        let ratio_now = ratio_of(*selected, *portrait);
        let label_now = ratios.get(*selected).map(|r| r.0.to_owned()).unwrap_or_default();
        let orient = if ratio_now <= 0.0 { "" } else if (ratio_now - 1.0).abs() < 0.01 { "square" } else if ratio_now > 1.0 { "landscape" } else { "portrait" };
        let caption = if orient.is_empty() { label_now.clone() } else { format!("{label_now} · {orient}") };
        let st_cap = self.text_style(TextRole::Caption);
        let crop_key = id.with("crop");
        let smooth = mo.smooth;
        let stroke = lk.focus.width.max(1.0);
        let glyph_w = (stroke * 0.6).clamp(1.2, 1.6);
        let pad = vec2(lk.control.pad_x[0], (lk.control.pad_x[0] * 0.35).round());
        let inset = gap * 1.5;
        let preview = self.leaf(
            Style { width: Size::FILL, height: Size::Fixed(height), ..Default::default() },
            vec2(200.0, height),
            None,
            Sense::NONE,
            CursorIcon::Default,
            None,
        );
        self.set_paint(
            preview,
            Box::new(move |cx, r| {
                let inner = r.expand(-height * 0.08);
                // Target crop inside the inner area.
                let (w, h) = if ratio_now <= 0.0 {
                    (inner.width() * 0.86, inner.height() * 0.8)
                } else {
                    let w = inner.width().min(inner.height() * ratio_now);
                    (w, w / ratio_now)
                };
                let (w, h) = if reduced {
                    (w, h)
                } else {
                    let first = cx.anims.spring_state(crop_key, 0).is_none();
                    let sw = cx.anims.spring_from(crop_key, 0, w, w, smooth);
                    let sh = cx.anims.spring_from(crop_key, 1, h, h, smooth);
                    if first {
                        (w, h)
                    } else {
                        (sw, sh)
                    }
                };
                let crop = Rect::from_center_size(r.center(), vec2(w, h));
                // The photo: sky, sun and two hills.
                cx.painter.push_clip(r);
                cx.painter.rrect_gradient(r, Corners::all(0.0), photo_top, photo_bot);
                cx.painter.circle(vec2(r.min.x + r.width() * 0.68, r.min.y + r.height() * 0.34), r.height() * 0.13, sun, 0.0, Color::TRANSPARENT);
                let hill = |y0: f32, amp: f32, ph: f32| -> Vec<crate::math::Vec2> {
                    (0..=24).map(|k| {
                        let t = k as f32 / 24.0;
                        vec2(r.min.x + t * r.width(), r.min.y + r.height() * (y0 - amp * (t * 5.0 + ph).sin()))
                    }).collect()
                };
                let bottom = |pts: &[crate::math::Vec2]| pts.iter().map(|p| vec2(p.x, r.max.y)).collect::<Vec<_>>();
                let a = hill(0.7, 0.07, 0.3);
                cx.painter.fill_between(&a, &bottom(&a), hill_a, hill_a);
                let b = hill(0.82, 0.05, 2.1);
                cx.painter.fill_between(&b, &bottom(&b), hill_b, hill_b);
                // Dim outside the crop.
                let dim = scrim.multiply_alpha(0.55);
                cx.painter.rect(Rect { min: r.min, max: vec2(r.max.x, crop.min.y) }, dim);
                cx.painter.rect(Rect { min: vec2(r.min.x, crop.max.y), max: r.max }, dim);
                cx.painter.rect(Rect { min: vec2(r.min.x, crop.min.y), max: vec2(crop.min.x, crop.max.y) }, dim);
                cx.painter.rect(Rect { min: vec2(crop.max.x, crop.min.y), max: vec2(r.max.x, crop.max.y) }, dim);
                cx.painter.pop_clip();
                // Frame, thirds and corner handles.
                cx.painter.ring(crop, Corners::all(0.0), 1.0, frame_c.multiply_alpha(0.9));
                if guides > 0.01 {
                    let gc = frame_c.multiply_alpha(0.5 * guides);
                    for k in 1..3 {
                        let x = crop.min.x + crop.width() * k as f32 / 3.0;
                        let y = crop.min.y + crop.height() * k as f32 / 3.0;
                        cx.painter.rect(Rect { min: vec2(x, crop.min.y), max: vec2(x + 1.0, crop.max.y) }, gc);
                        cx.painter.rect(Rect { min: vec2(crop.min.x, y), max: vec2(crop.max.x, y + 1.0) }, gc);
                    }
                }
                let l = (crop.width().min(crop.height()) * 0.14).clamp(8.0, 18.0);
                let t = stroke + 1.0;
                for (cxr, sx, sy) in [(crop.min, 1.0, 1.0), (vec2(crop.max.x, crop.min.y), -1.0, 1.0), (crop.max, -1.0, -1.0), (vec2(crop.min.x, crop.max.y), 1.0, -1.0)] {
                    let o = cxr - vec2(sx * t * 0.5, sy * t * 0.5);
                    cx.painter.polyline(&[o + vec2(sx * l, 0.0), o, o + vec2(0.0, sy * l)], t, frame_c);
                }
                let g = cx.galley(&caption, &st_cap);
                // Inside the crop's top-left corner, clear of the handles.
                let pill = Rect::from_xywh(crop.min.x + t + inset * 0.5, crop.min.y + t + inset * 0.5, g.width + pad.x * 2.0, g.height + pad.y * 2.0);
                cx.painter.rrect(pill, Corners::all(pill.height() * 0.5), scrim.multiply_alpha(0.7), 0.0, Color::TRANSPARENT);
                cx.painter.text(cx.text, &g, pill.min + pad, frame_c);
            }),
        );
        let cap_a11y = format!("Crop preview, {}", if orient.is_empty() { label_now.clone() } else { format!("{label_now} {orient}") });
        self.a11y_node(preview, || A11y::new(Role::Image).label(cap_a11y));

        // Chips, wrapping on narrow widths, with the orientation toggle last.
        let sel = *selected;
        let pill_key = id.with("pill");
        let sel_id = ids.get(sel).copied();
        let pill_c = self.color(Token::AccentSubtle);
        let pill_bd = self.color(Token::Accent).multiply_alpha(0.5);
        let chip_r = lk.control.radius[1];
        let row = self.begin(
            Style { dir: Dir::Row, width: Size::FILL, gap: gap * 0.5, wrap: true, align: Align::Center, ..Default::default() },
            Some(id.with("chips")),
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );
        self.set_paint(
            row,
            Box::new(move |cx, _| {
                let Some(s) = sel_id.and_then(|id| cx.rects.get(&id).copied()) else { return };
                let r = if reduced {
                    s
                } else {
                    let first = cx.anims.spring_state(pill_key, 0).is_none();
                    let x = cx.anims.spring_from(pill_key, 0, s.min.x, s.min.x, smooth);
                    let y = cx.anims.spring_from(pill_key, 1, s.min.y, s.min.y, smooth);
                    let w = cx.anims.spring_from(pill_key, 2, s.width(), s.width(), smooth);
                    if first {
                        s
                    } else {
                        Rect::from_xywh(x, y, w, s.height())
                    }
                };
                cx.painter.rrect_unsnapped(r, Corners::all(chip_r), [pill_c; 2], 1.0, pill_bd);
            }),
        );
        let st = self.text_style(TextRole::Label);
        let fg = self.color(Token::Foreground);
        let sec = self.color(Token::TextSecondary);
        let accent = self.color(Token::Accent);
        let ring = self.color(Token::FocusRing);
        let glyph = st.size;
        for (i, (label, _)) in ratios.iter().enumerate() {
            let ix = ixs[i];
            let on = self.fade(ix.id, 0, i == sel, mo.selected, mo.selected, Ease::Out);
            let (hov, press) = self.state_weights(&ix, false);
            let squash = if reduced { 1.0 } else { self.spring(ix.id, 1, if ix.pressed { 0.94 } else { 1.0 }, mo.bouncy) };
            let (ra, rg) = self.focus_anim(&ix);
            let hover = self.color(Token::Hover).multiply_alpha(hov * (1.0 - on) + press * 0.4);
            // The chip's own shape, sprung so it turns with the orientation.
            let rt = ratio_of(i, *portrait);
            let (gw, gh) = if rt <= 0.0 { (glyph, glyph) } else if rt >= 1.0 { (glyph, glyph / rt) } else { (glyph * rt, glyph) };
            let (gw, gh) = if reduced { (gw, gh) } else { (self.spring(ix.id, 2, gw, mo.smooth), self.spring(ix.id, 3, gh, mo.smooth)) };
            let free = rt <= 0.0;
            let tc = sec.mix(fg, hov.max(on));
            let gc = sec.mix(accent, on);
            self.begin(
                Style { dir: Dir::Row, height: Size::Fixed(lk.control.height[1]), pad: Sides::xy(lk.control.pad_x[1] * 0.8, 0.0), gap: gap * 0.75, align: Align::Center, justify: Justify::Center, ..Default::default() },
                Some(ix.id),
                Sense::CLICK,
                CursorIcon::Pointer,
                false,
                Some(Box::new(move |cx, r| {
                    if hover.a > 0.0 {
                        cx.painter.rrect(Rect::from_center_size(r.center(), r.size() * squash), Corners::all(chip_r), hover, 0.0, Color::TRANSPARENT);
                    }
                    if ra > 0.0 {
                        cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, chip_r, ra, rg, ring)));
                    }
                })),
            );
            self.leaf(
                Style::default(),
                vec2(glyph, glyph),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    let g = Rect::from_center_size(r.center(), vec2(gw, gh) * squash);
                    if free {
                        // Free: four corner marks instead of a closed shape.
                        let l = glyph * 0.3;
                        for (c, sx, sy) in [(g.min, 1.0, 1.0), (vec2(g.max.x, g.min.y), -1.0, 1.0), (g.max, -1.0, -1.0), (vec2(g.min.x, g.max.y), 1.0, -1.0)] {
                            cx.painter.polyline(&[c + vec2(sx * l, 0.0), c, c + vec2(0.0, sy * l)], glyph_w, gc);
                        }
                    } else {
                        cx.painter.ring(g, Corners::all(glyph_w * 1.5), glyph_w, gc);
                    }
                })),
            );
            self.text_leaf(label.to_string(), st, tc, false, false, Style::default());
            self.end();
            let n_r = ratios.len();
            self.a11y(ix.id, || A11y::new(Role::RadioButton).label(*label).selected(i == sel).toggled(i == sel).position(i, n_r));
        }
        let flip_label = if *portrait { "Landscape" } else { "Portrait" };
        let flip_icon = if *portrait { "rectangle-horizontal" } else { "rectangle-vertical" };
        if self.scope(id.with("flip"), |ui| ui.icon_button(flip_icon).ghost().tooltip(flip_label).disabled(!flippable).show().clicked) {
            *portrait = !*portrait;
        }
        self.end();
        self.a11y(id.with("chips"), || A11y::new(Role::RadioGroup).label("Aspect ratio"));
        self.end();

        let changed = (*selected, *portrait) != old;
        if changed {
            self.mem::<AspectMem>(id).changed_at = now;
            self.request_repaint_after(0.0);
        }
        Response { id, rect: self.rect_of(id).unwrap_or_default(), hovered: false, clicked: false, changed, focused: ixs.iter().any(|i| i.focused) }
    }
}
