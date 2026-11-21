//! Guided tour: a modal walkthrough that dims the window, cuts a rounded
//! spotlight around one widget at a time, and explains it in a card next
//! to it. The spotlight glides from step to step; ← → Enter and Escape work.

use crate::anim::Ease;
use crate::id::Id;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{ControlSize, Elevation, TextRole, Token};
use crate::ui::{LayerPlace, Sense, Ui};
use crate::Color;

/// One stop on a [`Ui::tour`].
#[derive(Clone, Debug)]
pub struct TourStep {
    /// The widget to spotlight: a `Response::id` from this or the previous
    /// frame.
    pub anchor: Id,
    pub title: String,
    pub body: String,
}

impl TourStep {
    pub fn new(anchor: Id, title: impl Into<String>, body: impl Into<String>) -> Self {
        TourStep { anchor, title: title.into(), body: body.into() }
    }
}

#[derive(Default)]
struct TourMem {
    /// Last spotlight, kept while the tour fades out.
    hole: Option<Rect>,
    shown: Option<usize>,
}

/// Room around the spotlighted widget.
const HOLE_PAD: f32 = 6.0;

impl Ui {
    /// Run a guided tour over `steps`. `step` is the current stop, `None`
    /// when the tour is closed; set it to `Some(0)` to start. Next, Back,
    /// ← →, Enter, Skip and Escape move it. Returns true when it changed.
    pub fn tour(&mut self, key: impl std::hash::Hash, steps: &[TourStep], step: &mut Option<usize>) -> bool {
        let id = self.make_id(("tour", key));
        let old = *step;
        if steps.is_empty() {
            *step = None;
        }
        if let Some(s) = *step {
            *step = Some(s.min(steps.len().saturating_sub(1)));
        }
        let mo = self.motion();
        let reduced = self.reduced_motion();
        let open = step.is_some();
        let t = self.fade(id, 0, open, mo.layer_in, mo.fade_out, Ease::Out);

        // Keyboard, while open.
        if let Some(s) = *step {
            if self.key_pressed(Key::Escape) {
                *step = None;
            } else if self.key_pressed(Key::ArrowRight) || self.key_pressed(Key::Enter) {
                *step = if s + 1 < steps.len() { Some(s + 1) } else { None };
            } else if self.key_pressed(Key::ArrowLeft) && s > 0 {
                *step = Some(s - 1);
            }
        }
        if t <= 0.0 {
            self.mem::<TourMem>(id).hole = None;
            return *step != old;
        }

        // The spotlight follows the current anchor; springs glide it.
        let cur = step.or(self.mem::<TourMem>(id).shown).unwrap_or(0).min(steps.len().saturating_sub(1));
        if step.is_some() {
            self.mem::<TourMem>(id).shown = *step;
        }
        let target = steps.get(cur).and_then(|s| self.rect_of(s.anchor)).map(|r| r.expand(HOLE_PAD));
        if let Some(r) = target {
            self.mem::<TourMem>(id).hole = Some(r);
        }
        let Some(goal) = self.mem::<TourMem>(id).hole else {
            return *step != old;
        };
        let hole = if reduced {
            goal
        } else {
            let p = mo.smooth;
            let x = self.spring(id, 1, goal.min.x, p);
            let y = self.spring(id, 2, goal.min.y, p);
            let w = self.spring(id, 3, goal.width(), p);
            let h = self.spring(id, 4, goal.height(), p);
            Rect::from_xywh(x, y, w, h)
        };
        let scrim = self.color(Token::Scrim).multiply_alpha(t);
        let glow = self.color(Token::FocusRing).multiply_alpha(0.55 * t);
        let radius = self.look().control.radius[1] + 4.0;
        let reach = self.screen_size().x.max(self.screen_size().y) * 3.0 + 400.0;

        // Scrim with a rounded hole: a ring whose inner edge is the hole
        // and whose outer edge is far off-screen. Clicks outside the card
        // are swallowed.
        self.layer(LayerPlace::Screen, vec2(0.0, 0.0), 1.0, Some(id.with("scrim")), |ui| {
            ui.leaf(
                Style { width: Size::FILL, height: Size::FILL, ..Default::default() },
                vec2(0.0, 0.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, _| {
                    let outer = hole.expand(reach);
                    cx.painter.ring(outer, Corners::all(radius + reach), reach, scrim);
                    cx.painter.ring(hole.expand(1.5), Corners::all(radius + 1.5), 1.5, glow);
                })),
            );
        });

        // The card: below the spotlight, or above when there's no room.
        let screen = self.content_rect();
        let card_w = 300.0f32.min(screen.width() - 24.0);
        let below = hole.max.y + 12.0 + 170.0 < screen.max.y;
        let at = vec2((hole.min.x).clamp(screen.min.x + 12.0, (screen.max.x - card_w - 12.0).max(screen.min.x + 12.0)), if below { hole.max.y + 12.0 } else { (hole.min.y - 12.0 - 170.0).max(screen.min.y + 12.0) });
        // The card rises 6 px as it fades in (no movement with reduced motion).
        let slide = if reduced { 0.0 } else { (1.0 - t) * 6.0 };
        let fill = self.color(Token::SurfaceRaised);
        let border = self.color(Token::Border);
        let shadows = Elevation::Level3.shadows(self.theme.darkness(), self.color(Token::ShadowInk));
        let r_card = self.look().card.radius;
        let n = steps.len();
        let (title, body) = steps.get(cur).map(|s| (s.title.clone(), s.body.clone())).unwrap_or_default();
        let mut go: Option<Option<usize>> = None;
        let dots_on = self.color(Token::Accent);
        let dots_off = self.color(Token::BorderStrong);
        let dot_x = if reduced { cur as f32 } else { self.spring(id, 6, cur as f32, mo.snappy) };
        self.layer(LayerPlace::Point(at), vec2(0.0, if below { slide } else { -slide }), t, None, |ui| {
            ui.begin(
                Style { dir: Dir::Column, width: Size::Fixed(card_w), pad: Sides::all(16.0), gap: 8.0, ..Default::default() },
                Some(id.with("card")),
                Sense::HOVER,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| {
                    cx.painter.shadows(r, Corners::all(r_card), &shadows);
                    cx.painter.rrect(r, Corners::all(r_card), fill, 1.0, border);
                })),
            );
            let st = ui.text_style(TextRole::Heading4);
            let fg = ui.color(Token::Foreground);
            ui.text_leaf(title, st, fg, true, false, Style { width: Size::FILL, ..Default::default() });
            let st = ui.text_style(TextRole::Body);
            let sec = ui.color(Token::TextSecondary);
            ui.text_leaf(body, st, sec, true, false, Style { width: Size::FILL, ..Default::default() });
            ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, align: Align::Center, margin_top: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            // Step dots: the current one is a pill that slides along.
            let dots_w = n as f32 * 12.0;
            ui.leaf(
                Style::default(),
                vec2(dots_w, 12.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    for k in 0..n {
                        cx.painter.circle(vec2(r.min.x + 4.0 + k as f32 * 12.0, r.center().y), 3.0, dots_off, 0.0, Color::TRANSPARENT);
                    }
                    let pr = Rect::from_xywh(r.min.x + dot_x * 12.0 - 1.0, r.center().y - 3.0, 14.0, 6.0);
                    cx.painter.rrect_unsnapped(pr, Corners::all(3.0), [dots_on; 2], 0.0, Color::TRANSPARENT);
                })),
            );
            ui.spacer();
            if cur + 1 < n && ui.button("Skip").ghost().size(ControlSize::Sm).clicked() {
                go = Some(None);
            }
            if cur > 0 && ui.button("Back").size(ControlSize::Sm).clicked() {
                go = Some(Some(cur - 1));
            }
            let last = cur + 1 >= n;
            if ui.button(if last { "Done" } else { "Next" }).primary().size(ControlSize::Sm).clicked() {
                go = Some(if last { None } else { Some(cur + 1) });
            }
            ui.end();
            ui.end();
        });
        if let Some(g) = go {
            *step = g;
        }
        *step != old
    }
}
