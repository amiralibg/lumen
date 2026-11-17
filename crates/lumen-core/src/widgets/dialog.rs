//! Modal dialog over a scrim.

use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Sides};
use crate::theme::{Elevation, TextRole, Token};
use crate::ui::{LayerPlace, Sense, Ui};
use crate::Color;

type UiFn<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

#[derive(Default)]
struct DialogMem {
    was_open: bool,
}

/// Builder for [`Ui::dialog`].
#[must_use = "call .show(|ui| ...)"]
pub struct Dialog<'u, 'a> {
    ui: &'u mut Ui,
    open: &'u mut bool,
    key: String,
    title: Option<String>,
    description: Option<String>,
    width: f32,
    footer: Option<UiFn<'a>>,
    dismissible: bool,
}

impl<'a> Dialog<'_, 'a> {
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
    /// Buttons, right-aligned at the bottom.
    pub fn footer(mut self, f: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.footer = Some(Box::new(f));
        self
    }
    /// Close on Escape and scrim clicks (default true).
    pub fn dismissible(mut self, d: bool) -> Self {
        self.dismissible = d;
        self
    }

    pub fn show<R>(self, body: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        let Dialog { ui, open, key, title, description, width, footer, dismissible } = self;
        let id = ui.make_id(("dialog", &key));
        let scrim_id = id.with("scrim");
        let mo = ui.motion();
        if *open && dismissible && (ui.key_pressed(Key::Escape) || ui.clicked_id(scrim_id)) {
            *open = false;
        }
        let is_open = *open;
        let just_opened = is_open && !ui.mem::<DialogMem>(id).was_open;
        ui.mem::<DialogMem>(id).was_open = is_open;
        let t = ui.fade(id, 0, is_open, mo.layer_in * 1.6, mo.fade_out * 1.4, Ease::Out);
        if t <= 0.0 {
            return None;
        }
        let rise = ui.anims.spring_from(id, 1, 14.0, if is_open { 0.0 } else { 6.0 }, mo.bouncy);
        let scrim = ui.color(Token::Scrim);
        let fill = ui.color(Token::SurfaceRaised);
        let border = ui.color(Token::Border);
        let shadows = Elevation::Level3.shadows(ui.theme.darkness(), ui.color(Token::ShadowInk));
        let radius = ui.look().card.radius + 2.0;
        let pad = ui.look().card.pad + 4.0;
        let out = ui.layer(LayerPlace::Screen, vec2(0.0, 0.0), t, Some(scrim_id), |ui| {
            if let Some(r) = ui.layer_root_building() {
                let st = ui.node_style(r);
                st.align = Align::Center;
                st.justify = Justify::Center;
                st.pad = Sides::all(24.0);
                ui.set_paint(r, Box::new(move |cx, r| cx.painter.rect(r, scrim)));
            }
            let mark = ui.focus_mark();
            ui.begin(
                Style { width: Size::Fixed(width), max: vec2(f32::INFINITY, f32::INFINITY), pad: Sides::all(pad), gap: 16.0, margin_top: rise * 2.0, ..Default::default() },
                Some(id.with("panel")),
                Sense::HOVER,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| {
                    cx.painter.shadows(r, Corners::all(radius), &shadows);
                    cx.painter.rrect(r, Corners::all(radius), fill, 1.0, border);
                })),
            );
            if title.is_some() || description.is_some() {
                ui.begin(Style { width: Size::FILL, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                if let Some(t) = title {
                    let st = ui.text_style(TextRole::Heading3);
                    let c = ui.color(Token::Foreground);
                    ui.text_leaf(t, st, c, true, false, Style::default());
                }
                if let Some(d) = description {
                    let st = ui.text_style(TextRole::Body);
                    let c = ui.color(Token::TextSecondary);
                    ui.text_leaf(d, st, c, true, false, Style::default());
                }
                ui.end();
            }
            let r = body(ui);
            if let Some(f) = footer {
                ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, justify: Justify::End, align: Align::Center, margin_top: 4.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                f(ui);
                ui.end();
            }
            ui.end();
            let items = ui.focusables_since(mark);
            if just_opened {
                if let Some(&first) = items.last() {
                    // Focus the last button (usually the primary action).
                    ui.request_focus(first);
                }
            }
            r
        });
        if ui.take_layer_close() {
            *open = false;
        }
        let _ = Color::TRANSPARENT;
        is_open.then_some(out)
    }
}

impl Ui {
    /// A modal dialog shown while `*open`. Escape and clicks on the scrim
    /// close it; call [`close_layer`](Ui::close_layer) from a button too.
    pub fn dialog<'u, 'a>(&'u mut self, key: impl Into<String>, open: &'u mut bool) -> Dialog<'u, 'a> {
        Dialog { ui: self, open, key: key.into(), title: None, description: None, width: 440.0, footer: None, dismissible: true }
    }
}
