//! Menus: a popover of actions.

use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::id::Id;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::look::Platform;
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

use super::popover::PopState;
use super::show_on_drop;

pub(crate) struct MenuCtx {
    pub(crate) anchor: Id,
}

/// One action in a menu.
pub struct MenuItem<'u> {
    ui: &'u mut Ui,
    icon: Option<String>,
    label: String,
    shortcut: Option<String>,
    danger: bool,
    disabled: bool,
    checked: Option<bool>,
    shown: bool,
}

impl MenuItem<'_> {
    /// Keyboard hint on the right, e.g. `"⌘R"`.
    pub fn shortcut(mut self, s: impl Into<String>) -> Self {
        self.shortcut = Some(s.into());
        self
    }
    /// Destructive action (red).
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    /// Show a check mark slot (for toggles and single choice).
    pub fn checked(mut self, c: bool) -> Self {
        self.checked = Some(c);
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("menu_item", &self.label));
        let ix = ui.interact(id, Sense::CLICK, self.disabled);
        let anchor = ui.menu.as_ref().map(|m| m.anchor);
        if let Some(a) = anchor {
            let items = ui.mem_get::<PopState>(a).map(|s| s.items.clone()).unwrap_or_default();
            if ix.focused {
                ui.arrow_focus(id, &items);
            }
        }
        let key_act = ix.focused && !self.disabled && (ui.key_pressed_for(id, Key::Enter) || ui.key_pressed_for(id, Key::Space));
        let clicked = (ix.clicked || key_act) && !self.disabled;
        if clicked {
            ui.close_layer();
        }
        let mo = ui.motion();
        let hot = (ix.hovered || ix.focused) && !self.disabled;
        let h = ui.fade(id, 1, hot, 0.0, mo.hover_out, Ease::Linear);
        let mac = ui.look().platform == Platform::Mac;
        let c = |t| ui.color(t);
        // macOS highlights menu rows with the accent; others use a neutral layer.
        let (hl, fg_hot) = if mac { (c(Token::Accent), c(Token::AccentForeground)) } else { (c(Token::Hover).over(c(Token::Hover)), c(Token::Foreground)) };
        let base_fg = if self.disabled {
            c(Token::TextDisabled)
        } else if self.danger {
            c(Token::DangerStrong)
        } else {
            c(Token::Foreground)
        };
        let fg = if self.danger && !mac { base_fg } else { base_fg.mix(fg_hot, h) };
        let icon_c = if self.disabled { c(Token::TextDisabled) } else if self.danger { fg } else { c(Token::TextSecondary).mix(fg_hot, h) };
        let sc_c = c(Token::TextMuted).mix(fg_hot.multiply_alpha(0.8), if mac { h } else { 0.0 });
        let fill = hl.multiply_alpha(h);
        let ch = ui.look().control.height[0].max(24.0) + 2.0;
        let radius = (ui.look().control.radius[0] - 1.0).max(0.0);
        let body = ui.text_style(TextRole::Body);
        let checked = self.checked;
        let disabled = self.disabled;
        ui.a11y(id, || {
            let a = match checked {
                Some(on) => A11y::new(Role::MenuItemCheckBox).toggled(on),
                None => A11y::new(Role::MenuItem),
            };
            let mut a = a.label(self.label.as_str()).disabled(disabled);
            if let Some(s) = &self.shortcut {
                a = a.description(s.as_str());
            }
            a
        });
        ui.begin(
            Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(ch), pad: Sides::xy(8.0, 0.0), gap: 8.0, align: Align::Center, ..Default::default() },
            Some(id),
            Sense::CLICK,
            if self.disabled { CursorIcon::Default } else { CursorIcon::Pointer },
            false,
            Some(Box::new(move |cx, r| {
                if fill.a > 0.0 {
                    cx.painter.rrect(r, Corners::all(radius), fill, 0.0, Color::TRANSPARENT);
                }
            })),
        );
        if let Some(on) = checked {
            let col = icon_c;
            ui.leaf(
                Style::default(),
                vec2(16.0, 16.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r: Rect| {
                    if on {
                        cx.icon("check", r.center(), 15.0, col);
                    }
                })),
            );
        }
        if let Some(i) = &self.icon {
            ui.icon_leaf(i, 16.0, icon_c);
        }
        ui.text_leaf(std::mem::take(&mut self.label), body, fg, false, true, Style { width: Size::FILL, ..Default::default() });
        if let Some(s) = self.shortcut.take() {
            let st = ui.text_style(TextRole::Label);
            ui.text_leaf(s, st, sc_c, false, false, Style { margin_top: 0.0, ..Default::default() });
            ui.space(2.0);
        }
        ui.end();
        Response { id, rect: ix.rect, hovered: ix.hovered, clicked, changed: clicked, focused: ix.focused }
    }
}

show_on_drop!(MenuItem);

impl Ui {
    /// A menu attached to `anchor`; it opens when the anchor is clicked.
    /// Fill it with [`menu_item`](Self::menu_item), [`menu_separator`](Self::menu_separator)
    /// and [`menu_label`](Self::menu_label).
    pub fn menu(&mut self, anchor: &Response, f: impl FnOnce(&mut Ui)) {
        let prev = self.menu.replace(MenuCtx { anchor: anchor.id });
        self.popover(anchor).pad(4.0).role(Role::Menu).show(|ui| {
            ui.begin(Style { width: Size::FILL, min: vec2(180.0, 0.0), gap: 1.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            f(ui);
            ui.end();
        });
        self.menu = prev;
    }

    pub fn menu_item(&mut self, icon: &str, label: impl Into<String>) -> MenuItem<'_> {
        let icon = (!icon.is_empty()).then(|| icon.to_owned());
        MenuItem { ui: self, icon, label: label.into(), shortcut: None, danger: false, disabled: false, checked: None, shown: false }
    }

    pub fn menu_separator(&mut self) {
        let c = self.color(Token::BorderSubtle);
        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(0.0, 9.0),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| cx.painter.rect(Rect::from_xywh(r.min.x - 4.0, r.center().y, r.width() + 8.0, 1.0), c))),
        );
    }

    /// A small section heading inside a menu.
    pub fn menu_label(&mut self, text: impl Into<String>) {
        let st = self.text_style(TextRole::Caption);
        let c = self.color(Token::TextMuted);
        self.begin(Style { pad: Sides { left: 8.0, right: 8.0, top: 6.0, bottom: 2.0 }, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let n = self.text_leaf(text, st, c, false, true, Style::default());
        self.a11y_node(n, || A11y::new(Role::Heading).level(4));
        self.end();
    }
}
