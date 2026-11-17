//! Card and settings rows (spec section 8).

use crate::input::CursorIcon;
use crate::layout::{Align, Dir, NodeId, Size, Style};
use crate::math::{Corners, Sides};
use crate::theme::{Shadow, TextRole, Token};
use crate::ui::{Sense, Shown, Ui};
use crate::Color;

type UiFn<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

/// A bordered surface grouping one topic's controls.
#[must_use = "call .show(|ui| ...)"]
pub struct Card<'u, 'a> {
    ui: &'u mut Ui,
    title: Option<String>,
    description: Option<String>,
    action: Option<UiFn<'a>>,
    footer_hint: Option<String>,
    footer: Option<UiFn<'a>>,
    pad: f32,
}

impl<'u, 'a> Card<'u, 'a> {
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    /// One button, right-aligned in the header.
    pub fn action(mut self, f: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.action = Some(Box::new(f));
        self
    }
    /// Inner padding (default: the look's).
    pub fn pad(mut self, p: f32) -> Self {
        self.pad = p.max(0.0);
        self
    }
    /// Footer: hint text on the left, `f` (buttons) on the right.
    pub fn footer(mut self, hint: impl Into<String>, f: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.footer_hint = Some(hint.into());
        self.footer = Some(Box::new(f));
        self
    }

    pub fn show<R>(self, body: impl FnOnce(&mut Ui) -> R) -> Shown<R> {
        let Card { ui, title, description, action, footer_hint, footer, pad } = self;
        let lk = ui.look().card;
        let fill = ui.color(Token::Surface);
        let border = ui.color(Token::Border).multiply_alpha(lk.border);
        let radius = lk.radius;
        let pad = if pad < 0.0 { lk.pad } else { pad };
        let shadows: Vec<Shadow> = if lk.shadow > 0.0 {
            let ink = ui.color(Token::ShadowInk);
            let k = lk.shadow * (1.0 + ui.theme.darkness() * 2.0);
            vec![
                Shadow { x: 0.0, y: 0.0, blur: 0.0, spread: 1.0, color: ink.with_alpha(0.03 * k) },
                Shadow { x: 0.0, y: 1.0, blur: 3.0, spread: 1.0, color: ink.with_alpha(0.07 * k) },
                Shadow { x: 0.0, y: 2.0, blur: 6.0, spread: 2.0, color: ink.with_alpha(0.03 * k) },
            ]
        } else {
            vec![]
        };
        let heading = ui.text_style(TextRole::Heading4);
        let body_st = ui.text_style(TextRole::Body);
        let key = title.clone().unwrap_or_default();
        let id = ui.make_id(("card", key));
        let rect = ui.rect_of(id).unwrap_or_default();
        ui.begin(
            Style { width: Size::FILL, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                if !shadows.is_empty() {
                    cx.painter.shadows(r, Corners::all(radius), &shadows);
                }
                cx.painter.rrect(r, Corners::all(radius), fill, 0.0, Color::TRANSPARENT)
            })),
        );
        let card_node = ui.last_node();
        if border.a > 0.0 {
            ui.set_paint_after(card_node, Box::new(move |cx, r| cx.painter.ring(r, Corners::all(radius), 1.0, border)));
        }
        let inner = ui.scope(id, |ui| {
            ui.begin(Style { width: Size::FILL, pad: Sides::all(pad), gap: 20.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            if title.is_some() || description.is_some() || action.is_some() {
                ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 16.0, align: Align::Start, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                ui.begin(Style { width: Size::FILL, gap: 4.0, max: crate::math::vec2(f32::INFINITY, f32::INFINITY), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                if let Some(t) = title {
                    let c = ui.color(Token::Foreground);
                    ui.text_leaf(t, heading, c, true, false, Style::default());
                }
                if let Some(d) = description {
                    let c = ui.color(Token::TextSecondary);
                    ui.text_leaf(d, body_st, c, true, false, Style { max: crate::math::vec2(520.0, f32::INFINITY), ..Default::default() });
                }
                ui.end();
                if let Some(a) = action {
                    a(ui);
                }
                ui.end();
            }
            ui.begin(Style { width: Size::FILL, gap: 16.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let r = body(ui);
            ui.end();
            ui.end();
            if let Some(f) = footer {
                let line = ui.color(Token::BorderSubtle);
                let muted = ui.color(Token::SurfaceMuted);
                ui.begin(
                    Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(pad, 12.0), gap: 16.0, align: Align::Center, ..Default::default() },
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    Some(Box::new(move |cx, r| {
                        // Inside the card border: inset 1px, bottom radii one less.
                        let fr = crate::math::Rect { min: r.min + crate::math::vec2(1.0, 0.0), max: r.max - crate::math::vec2(1.0, 1.0) };
                        let br = (radius - 1.0).max(0.0);
                        cx.painter.rrect(fr, Corners([0.0, 0.0, br, br]), muted, 0.0, Color::TRANSPARENT);
                        cx.painter.rect(crate::math::Rect::from_xywh(r.min.x, r.min.y, r.width(), 1.0), line);
                    })),
                );
                if let Some(h) = footer_hint {
                    let c = ui.color(Token::TextSecondary);
                    ui.text_leaf(h, body_st, c, true, false, Style::default());
                }
                ui.spacer();
                ui.begin(Style { dir: Dir::Row, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                f(ui);
                ui.end();
                ui.end();
            }
            r
        });
        ui.end();
        Shown { inner, id, rect }
    }
}

/// Builder for a group of settings rows separated by dividers.
pub struct Settings<'u> {
    ui: &'u mut Ui,
    rows: Vec<NodeId>,
}

impl Settings<'_> {
    /// A row: title and description on the left, `control` on the right.
    pub fn row(&mut self, title: &str, description: &str, control: impl FnOnce(&mut Ui)) -> &mut Self {
        self.row_with(|ui| {
            let c = ui.color(Token::Foreground);
            let st = ui.text_style(TextRole::BodyStrong);
            ui.text_leaf(title, st, c, true, false, Style::default());
            if !description.is_empty() {
                let c = ui.color(Token::TextSecondary);
                let st = ui.text_style(TextRole::Body);
                ui.text_leaf(description, st, c, true, false, Style::default());
            }
        }, control)
    }

    /// A row with a custom left side.
    pub fn row_with(&mut self, left: impl FnOnce(&mut Ui), control: impl FnOnce(&mut Ui)) -> &mut Self {
        let ui = &mut *self.ui;
        if !self.rows.is_empty() {
            ui.divider();
        }
        let first = self.rows.is_empty();
        let n = ui.begin(
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                gap: 16.0,
                align: Align::Center,
                pad: Sides { top: if first { 0.0 } else { 12.0 }, bottom: 12.0, ..Default::default() },
                ..Default::default()
            },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );
        ui.begin(Style { width: Size::FILL, gap: 2.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        left(ui);
        ui.end();
        control(ui);
        ui.end();
        self.rows.push(n);
        self
    }
}

impl Ui {
    /// A card; chain `.title()`, `.description()`, `.action()`, `.footer()`.
    pub fn card<'a>(&mut self) -> Card<'_, 'a> {
        Card { ui: self, title: None, description: None, action: None, footer_hint: None, footer: None, pad: -1.0 }
    }

    /// Settings rows with dividers (the card body pattern).
    pub fn settings(&mut self, f: impl FnOnce(&mut Settings<'_>)) {
        self.begin(Style { width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let mut s = Settings { ui: self, rows: vec![] };
        f(&mut s);
        let rows = std::mem::take(&mut s.rows);
        if let Some(&last) = rows.last() {
            self.node_style(last).pad.bottom = 0.0;
        }
        self.end();
    }
}
