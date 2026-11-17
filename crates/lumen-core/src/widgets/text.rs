//! Label and heading (spec sections 1 and 2).

use crate::layout::Style;
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Ui};

use super::show_on_drop;

/// A run of text with a type role. Wraps at the available width.
pub struct Label<'u> {
    ui: &'u mut Ui,
    text: String,
    role: TextRole,
    color: Token,
    truncate: bool,
    tabular: bool,
    shown: bool,
}

impl<'u> Label<'u> {
    pub(crate) fn new(ui: &'u mut Ui, text: impl Into<String>) -> Self {
        Label { ui, text: text.into(), role: TextRole::Body, color: Token::Foreground, truncate: false, tabular: false, shown: false }
    }
    pub fn role(mut self, r: TextRole) -> Self {
        self.role = r;
        self
    }
    pub fn color(mut self, t: Token) -> Self {
        self.color = t;
        self
    }
    /// `TEXT_SECONDARY`.
    pub fn secondary(self) -> Self {
        self.color(Token::TextSecondary)
    }
    /// `TEXT_MUTED`.
    pub fn muted(self) -> Self {
        self.color(Token::TextMuted)
    }
    /// `TEXT_DISABLED`.
    pub fn disabled(self, d: bool) -> Self {
        if d {
            self.color(Token::TextDisabled)
        } else {
            self
        }
    }
    /// `body-strong` role.
    pub fn strong(self) -> Self {
        self.role(TextRole::BodyStrong)
    }
    pub fn caption(self) -> Self {
        self.role(TextRole::Caption).color(Token::TextMuted)
    }
    pub fn small(self) -> Self {
        self.role(TextRole::Label)
    }
    pub fn mono(self) -> Self {
        self.role(TextRole::Mono)
    }
    /// Single line, ending with an ellipsis when it doesn't fit.
    pub fn truncate(mut self) -> Self {
        self.truncate = true;
        self
    }
    pub fn tabular(mut self) -> Self {
        self.tabular = true;
        self
    }

    fn show_inner(&mut self) -> Response {
        let mut style = self.role.style();
        style.tabular = self.tabular;
        let c = self.ui.color(self.color);
        let text = std::mem::take(&mut self.text);
        let layout = if self.truncate { Style { width: crate::layout::Size::Hug, ..Default::default() } } else { Style::default() };
        self.ui.text_leaf(text, style, c, !self.truncate, self.truncate, layout);
        Response::default()
    }
}

show_on_drop!(Label);

/// A heading. Levels 1-4 plus display (spec section 2).
pub struct Heading<'u> {
    ui: &'u mut Ui,
    text: String,
    level: u8,
    shown: bool,
}

impl<'u> Heading<'u> {
    pub(crate) fn new(ui: &'u mut Ui, text: impl Into<String>) -> Self {
        Heading { ui, text: text.into(), level: 2, shown: false }
    }
    /// 0 = display, 1..=4 = heading levels.
    pub fn level(mut self, l: u8) -> Self {
        self.level = l.min(4);
        self
    }
    pub fn display(self) -> Self {
        self.level(0)
    }

    fn show_inner(&mut self) -> Response {
        let (role, above) = match self.level {
            0 => (TextRole::Display, 0.0),
            1 => (TextRole::Heading1, 0.0),
            2 => (TextRole::Heading2, 32.0),
            3 => (TextRole::Heading3, 24.0),
            _ => (TextRole::Heading4, 16.0),
        };
        let c = self.ui.color(Token::Foreground);
        let text = std::mem::take(&mut self.text);
        self.ui.text_leaf(text, role.style(), c, true, false, Style { margin_top: above, ..Default::default() });
        Response::default()
    }
}

show_on_drop!(Heading);

impl Ui {
    /// Body text (`body` role, `FOREGROUND`).
    pub fn label(&mut self, text: impl Into<String>) -> Label<'_> {
        Label::new(self, text)
    }
    /// A heading, level 2 by default.
    pub fn heading(&mut self, text: impl Into<String>) -> Heading<'_> {
        Heading::new(self, text)
    }
}
