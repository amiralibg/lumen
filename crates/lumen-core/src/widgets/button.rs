//! Button (spec section 3).

use crate::input::CursorIcon;
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{ControlSize, Shadow, Token};
use crate::ui::{Placement, Response, Sense, TooltipSpec, Ui};
use crate::Color;

use super::{blend3, key_activate, neutral_layers, show_on_drop};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Variant {
    Primary,
    #[default]
    Secondary,
    Ghost,
    Danger,
}

pub struct Button<'u> {
    ui: &'u mut Ui,
    label: String,
    variant: Variant,
    size: ControlSize,
    icon: Option<String>,
    trailing: Option<String>,
    icon_only: bool,
    disabled: bool,
    loading: bool,
    full_width: bool,
    tooltip: Option<(String, Option<String>)>,
    shown: bool,
}

impl<'u> Button<'u> {
    pub(crate) fn new(ui: &'u mut Ui, label: impl Into<String>) -> Self {
        Button {
            ui,
            label: label.into(),
            variant: Variant::Secondary,
            size: ControlSize::Md,
            icon: None,
            trailing: None,
            icon_only: false,
            disabled: false,
            loading: false,
            full_width: false,
            tooltip: None,
            shown: false,
        }
    }
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }
    /// Accent-filled; at most one per view.
    pub fn primary(self) -> Self {
        self.variant(Variant::Primary)
    }
    pub fn secondary(self) -> Self {
        self.variant(Variant::Secondary)
    }
    pub fn ghost(self) -> Self {
        self.variant(Variant::Ghost)
    }
    /// Destructive, irreversible actions only.
    pub fn danger(self) -> Self {
        self.variant(Variant::Danger)
    }
    pub fn size(mut self, s: ControlSize) -> Self {
        self.size = s;
        self
    }
    pub fn small(self) -> Self {
        self.size(ControlSize::Sm)
    }
    pub fn large(self) -> Self {
        self.size(ControlSize::Lg)
    }
    /// Leading Lucide icon.
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = Some(name.to_owned());
        self
    }
    /// Trailing Lucide icon.
    pub fn trailing_icon(mut self, name: &str) -> Self {
        self.trailing = Some(name.to_owned());
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    /// Show an inline spinner and prevent duplicate activation while busy.
    pub fn loading(mut self, l: bool) -> Self {
        self.loading = l;
        self
    }
    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }
    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some((text.into(), None));
        self
    }
    /// Tooltip with a shortcut chip.
    pub fn tooltip_shortcut(mut self, text: impl Into<String>, shortcut: impl Into<String>) -> Self {
        self.tooltip = Some((text.into(), Some(shortcut.into())));
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let key = if self.icon_only { self.icon.clone().unwrap_or_default() } else { self.label.clone() };
        let id = ui.make_id(("button", key));
        let busy = self.disabled || self.loading;
        let ix = ui.interact(id, Sense::CLICK, busy);
        let (key_act, key_held) = key_activate(ui, &ix, true);
        let clicked = (ix.clicked || key_act) && !busy;
        let (h, p) = ui.state_weights(&ix, key_held);
        let m = ui.control_metrics(self.size);
        let look = ui.look().button;
        let mo = ui.motion();
        let pressed = ix.pressed || key_held;
        let inset = ui.spring(id, 0, if pressed { m.inset } else { 0.0 }, if pressed { mo.snappy } else { mo.bouncy });
        let (ring_a, ring_gap) = ui.focus_anim(&ix);

        let c = |t| ui.color(t);
        let (fill, border, text, icon) = if self.disabled {
            match self.variant {
                Variant::Ghost => (Color::TRANSPARENT, Color::TRANSPARENT, c(Token::TextDisabled), c(Token::TextDisabled)),
                _ => (c(Token::SurfaceMuted), c(Token::BorderSubtle), c(Token::TextDisabled), c(Token::TextDisabled)),
            }
        } else {
            match self.variant {
                Variant::Primary => {
                    let f = blend3(c(Token::Accent), c(Token::AccentHover), c(Token::AccentPressed), h, p);
                    (f, Color::TRANSPARENT, c(Token::AccentForeground), c(Token::AccentForeground))
                }
                Variant::Danger => {
                    let f = blend3(c(Token::Danger), c(Token::DangerHover), c(Token::DangerPressed), h, p);
                    (f, Color::TRANSPARENT, c(Token::DangerForeground), c(Token::DangerForeground))
                }
                Variant::Secondary => {
                    let base = c(Token::Control);
                    let (hv, pr) = neutral_layers(ui, base);
                    let cb = c(Token::ControlBorder);
                    let b = if cb.a > 0.0 { cb.mix(c(Token::BorderStrong), h.max(p) * (1.0 - look.press_dim)) } else { cb };
                    let ic = c(Token::TextSecondary).mix(c(Token::Foreground), h.max(p));
                    (blend3(base, hv, pr, h, p), b, c(Token::Foreground), ic)
                }
                Variant::Ghost => {
                    let (hv, pr) = neutral_layers(ui, Color::TRANSPARENT);
                    let t = c(Token::TextSecondary).mix(c(Token::Foreground), h.max(p));
                    (blend3(Color::TRANSPARENT, hv, pr, h, p), Color::TRANSPARENT, t, t)
                }
            }
        };
        // Windows dims the label while pressed.
        let dim = look.press_dim * p;
        let (text, icon) = if dim > 0.0 && !self.disabled {
            match self.variant {
                Variant::Primary | Variant::Danger => (text.multiply_alpha(1.0 - 0.2 * dim), icon.multiply_alpha(1.0 - 0.2 * dim)),
                _ => (text.mix(c(Token::TextSecondary), dim), icon.mix(c(Token::TextSecondary), dim)),
            }
        } else {
            (text, icon)
        };
        let filled = matches!(self.variant, Variant::Primary | Variant::Danger) && !self.disabled;
        let neutral = self.variant == Variant::Secondary && !self.disabled;
        // macOS: a light top-to-bottom gradient on filled buttons and a hairline
        // drop shadow under bordered ones.
        let bevel = look.bevel * (1.0 - p);
        let grad = if filled && bevel > 0.0 {
            [fill.shift_lightness(0.035 * bevel), fill.shift_lightness(-0.015 * bevel)]
        } else {
            [fill; 2]
        };
        let shadow_a = if (filled || neutral) && look.shadow > 0.0 { look.shadow * (0.10 + 0.25 * ui.theme.darkness()) } else { 0.0 };
        let shadows = if shadow_a > 0.0 {
            vec![Shadow { x: 0.0, y: 0.5, blur: 1.5, spread: 0.0, color: c(Token::ShadowInk).with_alpha(shadow_a) }]
        } else {
            vec![]
        };
        // Windows: a darker bottom edge.
        let edge = if look.bottom_edge > 0.0 && !self.disabled {
            if filled {
                fill.shift_lightness(-0.10).multiply_alpha(look.bottom_edge)
            } else if neutral {
                c(Token::ControlEdge).multiply_alpha(look.bottom_edge)
            } else {
                Color::TRANSPARENT
            }
        } else {
            Color::TRANSPARENT
        };
        let ring_color = if self.variant == Variant::Danger { c(Token::DangerStrong) } else { c(Token::FocusRing) };
        let radius = m.radius;
        let bw = if border.a > 0.0 { 1.0 } else { 0.0 };

        let icon_only = self.icon_only;
        let pad = if icon_only {
            Sides::all(0.0)
        } else {
            Sides {
                left: if self.icon.is_some() || self.loading { m.pad_icon } else { m.pad_x },
                right: if self.trailing.is_some() { m.pad_icon } else { m.pad_x },
                top: 0.0,
                bottom: 0.0,
            }
        };
        let style = Style {
            dir: Dir::Row,
            width: if icon_only {
                Size::Fixed(m.height)
            } else if self.full_width {
                Size::FILL
            } else {
                Size::Hug
            },
            height: Size::Fixed(m.height),
            min: vec2(if icon_only { m.height } else { m.height * 2.0 }, m.height),
            pad,
            gap: m.gap,
            align: Align::Center,
            justify: Justify::Center,
            ..Default::default()
        };
        let cursor = if busy { CursorIcon::Default } else { CursorIcon::Pointer };
        ui.begin(
            style,
            Some(id),
            Sense::CLICK,
            cursor,
            false,
            Some(Box::new(move |cx, r| {
                let bg = cx.painter.snap_rect(r.expand(-inset));
                let corners = Corners::all((radius - inset).max(0.0));
                if !shadows.is_empty() {
                    cx.painter.shadows(bg, corners, &shadows);
                }
                cx.painter.rrect_unsnapped(bg, corners, grad, bw, border);
                if edge.a > 0.0 {
                    let band = (radius * 0.6).max(1.5);
                    cx.painter.push_clip(Rect::from_xywh(bg.min.x, bg.max.y - band, bg.width(), band));
                    cx.painter.ring(bg, corners, 1.0, edge);
                    cx.painter.pop_clip();
                }
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, radius, ring_a, ring_gap, ring_color)));
                }
            })),
        );
        if self.loading {
            let sz = m.icon;
            let spin_c = icon;
            let track_c = icon.multiply_alpha(0.28);
            let t = ui.time() as f32;
            let reduced = ui.reduced_motion();
            if !reduced {
                ui.request_repaint_after(0.0);
            }
            ui.leaf(
                Style::default(),
                vec2(sz, sz),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    let w = (sz / 7.5).max(1.8);
                    let (ro, ri) = (sz / 2.0, sz / 2.0 - w);
                    cx.painter.arc_band(r.center(), ri, ro, 0.0, std::f32::consts::TAU, track_c);
                    let a = if reduced { 0.8 } else { t * 5.2 };
                    let len = if reduced { 1.8 } else { 1.3 + 0.8 * (t * 2.4).sin() };
                    cx.painter.arc_band(r.center(), ri, ro, a, a + len, spin_c);
                })),
            );
        } else if let Some(i) = &self.icon {
            ui.icon_leaf(i, m.icon, icon);
        }
        if !icon_only {
            let label = std::mem::take(&mut self.label);
            ui.text_leaf(label, m.text, text, false, true, Style::default());
        }
        if let Some(i) = &self.trailing {
            ui.icon_leaf(i, m.icon, icon);
        }
        ui.end();
        if let Some((t, s)) = self.tooltip.take() {
            ui.tooltip_for(id, TooltipSpec { text: t, shortcut: s, placement: Placement::Top, gap: 6.0 });
        }
        Response { id, rect: ix.rect, hovered: ix.hovered, clicked, changed: clicked, focused: ix.focused }
    }
}

show_on_drop!(Button);

impl Ui {
    /// A button. Secondary by default; chain `.primary()`, `.ghost()`, `.danger()`.
    pub fn button(&mut self, label: impl Into<String>) -> Button<'_> {
        Button::new(self, label)
    }

    /// An icon-only button. Give it a `.tooltip(..)`.
    pub fn icon_button(&mut self, icon: &str) -> Button<'_> {
        let mut b = Button::new(self, "");
        b.icon = Some(icon.to_owned());
        b.icon_only = true;
        b
    }
}
