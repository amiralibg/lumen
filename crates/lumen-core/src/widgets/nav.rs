//! Sidebar navigation (spec section 9).

use crate::anim::Ease;
use crate::id::Id;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{TextRole, Token};
use crate::ui::{Placement, Response, Sense, TooltipSpec, Ui};
use crate::Color;

use super::{neutral_layers, show_on_drop};

pub(crate) struct NavCtx {
    id: Id,
    /// 0 expanded .. 1 collapsed.
    collapse_t: f32,
    label_alpha: f32,
    selected: Option<Id>,
    items: Vec<Id>,
}

#[derive(Default)]
struct NavMem {
    items: Vec<Id>,
}

/// A navigation sidebar. Items go inside `.show(|ui| ...)`.
#[must_use = "call .show(|ui| ...)"]
pub struct Sidebar<'u> {
    ui: &'u mut Ui,
    collapsed: bool,
    width: f32,
}

impl Sidebar<'_> {
    /// Collapse to icons only (56 wide), animated.
    pub fn collapsed(mut self, c: bool) -> Self {
        self.collapsed = c;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    pub fn show<R>(self, f: impl FnOnce(&mut Ui) -> R) -> R {
        let Sidebar { ui, collapsed, width } = self;
        let id = ui.make_id("sidebar");
        let mo = ui.motion();
        let nl = ui.look().nav;
        let w = ui.spring(id, 0, if collapsed { 56.0 } else { width }, mo.smooth);
        let collapse_t = ((width - w) / (width - 56.0)).clamp(0.0, 1.0);
        let label_on = !collapsed && collapse_t < 0.2;
        let label_alpha = ui.fade(id, 1, label_on, mo.layer_in, mo.fade_out, Ease::Linear);
        let bg = ui.color(Token::Background);
        let edge = ui.color(Token::BorderSubtle);
        let node = ui.begin(
            Style { width: Size::Fixed(w.round()), height: Size::FILL, pad: Sides::all(12.0), ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            None,
        );
        let prev = ui.nav.replace(NavCtx { id, collapse_t, label_alpha, selected: None, items: vec![] });
        let r = ui.scope(id, f);
        let ctx = std::mem::replace(&mut ui.nav, prev).unwrap();
        ui.end();
        ui.mem::<NavMem>(id).items = ctx.items;

        // The selected pill: one shared rect that slides between items.
        let neutral_pill = bg.over(ui.color(Token::Pressed));
        let pill = neutral_pill.mix(ui.color(Token::AccentSubtle), nl.accent_selection);
        let pill = pill.mix(bg.over(ui.color(Token::Hover)), nl.indicator);
        let bar = ui.color(Token::Accent).multiply_alpha(nl.indicator);
        let smooth = mo.smooth;
        let pill_r = nl.radius;
        let selected = ctx.selected;
        let anim_key = id.with("pill");
        let reduced = ui.reduced_motion();
        ui.set_paint(
            node,
            Box::new(move |cx, r| {
                cx.painter.rect(r, bg);
                cx.painter.rect(Rect::from_xywh(r.max.x - 1.0, r.min.y, 1.0, r.height()), edge);
                if let Some(sel) = selected.and_then(|s| cx.rects.get(&s).copied()) {
                    let y = if reduced {
                        sel.min.y
                    } else {
                        let first = cx.anims.spring_state(anim_key, 0).is_none();
                        let y = cx.anims.spring_from(anim_key, 0, sel.min.y, sel.min.y, smooth);
                        if first { sel.min.y } else { y }
                    };
                    let pr = Rect::from_xywh(sel.min.x, y, sel.width(), sel.height());
                    cx.painter.rrect(pr, Corners::all(pill_r), pill, 0.0, Color::TRANSPARENT);
                    if bar.a > 0.0 {
                        // Windows: an accent bar at the leading edge.
                        let bh = (sel.height() * 0.45).round();
                        let br = Rect::from_xywh(pr.min.x, pr.center().y - bh / 2.0, 3.0, bh);
                        cx.painter.rrect_unsnapped(br, Corners::all(1.5), [bar; 2], 0.0, Color::TRANSPARENT);
                    }
                }
            }),
        );
        r
    }
}

/// A sidebar item; `.clicked()` when chosen.
pub struct NavItem<'u> {
    ui: &'u mut Ui,
    icon: String,
    label: String,
    badge: Option<String>,
    selected: bool,
    disabled: bool,
    shown: bool,
}

impl NavItem<'_> {
    pub fn selected(mut self, s: bool) -> Self {
        self.selected = s;
        self
    }
    /// A count; shows `99+` above 99.
    pub fn badge(mut self, n: u32) -> Self {
        self.badge = Some(if n > 99 { "99+".into() } else { n.to_string() });
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("nav_item", &self.label));
        let ix = ui.interact(id, Sense::CLICK, self.disabled);
        let (collapse_t, label_alpha, nav_id) = ui.nav.as_ref().map(|n| (n.collapse_t, n.label_alpha, n.id)).unwrap_or((0.0, 1.0, Id::ROOT));
        // Up/Down move focus between items.
        if ix.focused {
            let order = ui.mem_get::<NavMem>(nav_id).map(|m| m.items.clone()).unwrap_or_default();
            if let Some(i) = order.iter().position(|&x| x == id) {
                if ui.key_pressed_for(id, Key::ArrowDown) && i + 1 < order.len() {
                    ui.request_focus(order[i + 1]);
                }
                if ui.key_pressed_for(id, Key::ArrowUp) && i > 0 {
                    ui.request_focus(order[i - 1]);
                }
            }
        }
        let (key_act, key_held) = super::key_activate(ui, &ix, true);
        let clicked = (ix.clicked || key_act) && !self.disabled;
        let selected = self.selected && !self.disabled;
        if let Some(n) = ui.nav.as_mut() {
            if selected {
                n.selected = Some(id);
            }
            if !self.disabled {
                n.items.push(id);
            }
        }
        let (h, p) = ui.state_weights(&ix, key_held);
        let mo = ui.motion();
        let nl = ui.look().nav;
        let press_inset = ui.look().button.press_inset;
        let sel = ui.fade(id, 3, selected, mo.selected, mo.selected, Ease::Out);
        let pressed = (ix.pressed || key_held) && !self.disabled;
        let inset = ui.spring(id, 0, if pressed { press_inset } else { 0.0 }, if pressed { mo.snappy } else { mo.bouncy });
        let sel_c = c_sel(ui, nl.accent_selection);
        let (ring_a, ring_gap) = ui.focus_anim(&ix);
        let c = |t| ui.color(t);
        let (hv, pr) = neutral_layers(ui, Color::TRANSPARENT);
        let (fill, icon_c, label_c) = if self.disabled {
            (Color::TRANSPARENT, c(Token::TextDisabled), c(Token::TextDisabled))
        } else {
            let fill = Color::TRANSPARENT.mix(hv, h).mix(pr, p).mix(Color::TRANSPARENT, sel);
            let ic = c(Token::TextMuted).mix(c(Token::Foreground), h.max(p)).mix(sel_c, sel);
            let lc = c(Token::TextSecondary).mix(c(Token::Foreground), h.max(p)).mix(sel_c, sel);
            (fill, ic, lc)
        };
        let (badge_fill, badge_text) = if self.disabled {
            (c(Token::SurfaceMuted), c(Token::TextDisabled))
        } else {
            (c(Token::SurfaceMuted).mix(Color::TRANSPARENT, sel * nl.accent_selection), c(Token::TextSecondary).mix(sel_c, sel))
        };
        let dot = c(Token::Accent);
        let ring = c(Token::FocusRing);
        let cursor = if self.disabled { CursorIcon::Default } else { CursorIcon::Pointer };
        let has_badge = self.badge.is_some();
        let item_r = nl.radius;
        let body_strong = ui.text_style(TextRole::BodyStrong);
        let caption = ui.text_style(TextRole::Caption);
        ui.begin(
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                height: Size::Fixed(nl.item_height),
                pad: Sides::xy(8.0, 0.0),
                gap: 8.0,
                align: Align::Center,
                margin_top: 2.0,
                ..Default::default()
            },
            Some(id),
            Sense::CLICK,
            cursor,
            true,
            Some(Box::new(move |cx, r| {
                let bg = r.expand(-inset);
                cx.painter.rrect(bg, Corners::all((item_r - inset).max(0.0)), fill, 0.0, Color::TRANSPARENT);
                if collapse_t > 0.5 && has_badge {
                    // Collapsed: a count becomes a dot on the icon's corner.
                    cx.painter.circle(vec2(r.min.x + 8.0 + 16.0 - 1.0, r.center().y - 8.0 + 1.0), 3.0, dot, 0.0, Color::TRANSPARENT);
                }
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, item_r, ring_a, ring_gap, ring)));
                }
            })),
        );
        ui.icon_leaf(&self.icon, 16.0, icon_c);
        if label_alpha > 0.0 {
            let label = std::mem::take(&mut self.label);
            let st = Style { width: Size::FILL, ..Default::default() };
            ui.text_leaf(label.clone(), body_strong, label_c.multiply_alpha(label_alpha), false, true, st);
            self.label = label;
            if let Some(b) = self.badge.take() {
                let mut ts = caption;
                ts.weight = 500;
                ts.tabular = true;
                let bf = badge_fill.multiply_alpha(label_alpha);
                ui.begin(
                    Style { height: Size::Fixed(18.0), min: vec2(18.0, 18.0), pad: Sides::xy(5.0, 0.0), justify: Justify::Center, align: Align::Center, ..Default::default() },
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(9.0), bf, 0.0, Color::TRANSPARENT))),
                );
                ui.text_leaf(b, ts, badge_text.multiply_alpha(label_alpha), false, false, Style::default());
                ui.end();
            }
        }
        ui.end();
        if collapse_t > 0.5 {
            ui.tooltip_for(id, TooltipSpec { text: self.label.clone(), shortcut: None, placement: Placement::Right, gap: 8.0 });
        }
        Response { id, rect: ix.rect, hovered: ix.hovered, clicked, changed: clicked, focused: ix.focused }
    }
}

show_on_drop!(NavItem);

/// Selected item text: accent-strong for accent selections, foreground for
/// neutral ones.
fn c_sel(ui: &Ui, accent: f32) -> Color {
    ui.color(Token::Foreground).mix(ui.color(Token::AccentStrong), accent)
}

impl Ui {
    /// A navigation sidebar (240 wide, collapsible to 56).
    pub fn sidebar(&mut self) -> Sidebar<'_> {
        Sidebar { ui: self, collapsed: false, width: 240.0 }
    }

    /// Sidebar header: avatar initial and workspace name.
    pub fn nav_header(&mut self, initial: &str, name: &str) {
        let (_, alpha) = self.nav.as_ref().map(|n| (n.collapse_t, n.label_alpha)).unwrap_or((0.0, 1.0));
        let accent = self.color(Token::Accent);
        let afg = self.color(Token::AccentForeground);
        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(40.0), pad: Sides::xy(4.0, 0.0), gap: 8.0, align: Align::Center, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );
        self.begin(
            Style { width: Size::Fixed(24.0), height: Size::Fixed(24.0), align: Align::Center, justify: Justify::Center, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(6.0), accent, 0.0, Color::TRANSPARENT))),
        );
        let st = self.text_style(TextRole::Label);
        self.text_leaf(initial, st, afg, false, false, Style::default());
        self.end();
        if alpha > 0.0 {
            let c = self.color(Token::Foreground).multiply_alpha(alpha);
            let st = self.text_style(TextRole::BodyStrong);
            self.text_leaf(name, st, c, false, true, Style { width: Size::FILL, ..Default::default() });
        }
        self.end();
    }

    /// Sidebar section label (a divider when collapsed).
    pub fn nav_section(&mut self, label: &str) {
        let (t, alpha) = self.nav.as_ref().map(|n| (n.collapse_t, n.label_alpha)).unwrap_or((0.0, 1.0));
        if t > 0.5 {
            let c = self.color(Token::BorderSubtle);
            self.leaf(
                Style { width: Size::FILL, margin_top: 16.0, ..Default::default() },
                vec2(0.0, 17.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| cx.painter.rect(Rect::from_xywh(r.min.x + 8.0, r.min.y + 8.0, (r.width() - 16.0).max(0.0), 1.0), c))),
            );
            return;
        }
        let c = self.color(Token::TextMuted).multiply_alpha(alpha);
        self.begin(
            Style { width: Size::FILL, height: Size::Fixed(28.0), pad: Sides::xy(8.0, 0.0), justify: Justify::Center, margin_top: 16.0, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            true,
            None,
        );
        let st = self.text_style(TextRole::Label);
        self.text_leaf(label, st, c, false, true, Style::default());
        self.end();
    }

    /// A sidebar item with a Lucide icon.
    pub fn nav_item(&mut self, icon: &str, label: impl Into<String>) -> NavItem<'_> {
        NavItem { ui: self, icon: icon.to_owned(), label: label.into(), badge: None, selected: false, disabled: false, shown: false }
    }

    /// Content pinned to the bottom of the sidebar.
    pub fn nav_footer<R>(&mut self, f: impl FnOnce(&mut Ui) -> R) -> R {
        self.spacer();
        self.begin(Style { width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let r = f(self);
        self.end();
        r
    }
}
