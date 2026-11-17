//! Overlays: edge-docked sheet/drawer, delayed hover card, right-click
//! context menu, and anchored confirm popover.

use crate::anim::Ease;
use crate::id::Id;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Sides, Vec2};
use crate::theme::{Elevation, TextRole, Token};
use crate::ui::{LayerAlign, LayerPlace, Placement, Response, Sense, Ui};

use super::menu::MenuCtx;
use super::popover::PopState;

type UiFn<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

// ------------------------------------------------------------------- Sheet

#[derive(Default)]
struct SheetMem {
    was_open: bool,
    prev_focus: Option<Id>,
}

/// Builder for [`Ui::sheet`]: an edge-docked drawer over a scrim.
#[must_use = "call .show(|ui| ...)"]
pub struct Sheet<'u, 'a> {
    ui: &'u mut Ui,
    open: &'u mut bool,
    key: String,
    side: Placement,
    title: Option<String>,
    description: Option<String>,
    size: Option<f32>,
    footer: Option<UiFn<'a>>,
    dismissible: bool,
}

impl<'a> Sheet<'_, 'a> {
    /// Edge the drawer slides in from (default [`Placement::Right`]).
    pub fn side(mut self, side: Placement) -> Self {
        self.side = side;
        self
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    /// Width (for left/right sheets) or height (for top/bottom sheets).
    pub fn size(mut self, px: f32) -> Self {
        self.size = Some(px);
        self
    }
    /// Action row pinned to the bottom of the drawer.
    pub fn footer(mut self, f: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.footer = Some(Box::new(f));
        self
    }
    /// Allow Escape, scrim clicks and the close button to dismiss (default true).
    pub fn dismissible(mut self, d: bool) -> Self {
        self.dismissible = d;
        self
    }

    pub fn show<R>(self, body: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        let Sheet { ui, open, key, side, title, description, size, footer, dismissible } = self;
        let id = ui.make_id(("sheet", &key));
        let scrim_id = id.with("scrim");
        let panel_id = id.with("panel");
        let mo = ui.motion();

        let was_open = ui.mem::<SheetMem>(id).was_open;
        if *open && !was_open {
            ui.mem::<SheetMem>(id).prev_focus = ui.focused();
        }
        let mut closed_by_user = false;
        if *open && dismissible && (ui.key_pressed(Key::Escape) || ui.clicked_id(scrim_id)) {
            *open = false;
            closed_by_user = true;
        }
        let is_open = *open;
        let just_opened = is_open && !was_open;
        ui.mem::<SheetMem>(id).was_open = is_open;

        let t = ui.fade(id, 0, is_open, mo.layer_in * 1.6, mo.fade_out * 1.4, Ease::Out);
        let travel = ui.anims.spring_from(id, 1, 1.0, if is_open { 0.0 } else { 1.0 }, if is_open { mo.smooth } else { mo.snappy });
        if t <= 0.0 && !is_open {
            return None;
        }

        let screen = ui.screen_size();
        let horizontal = matches!(side, Placement::Left | Placement::Right);
        let span = size.unwrap_or(if horizontal { 380.0 } else { 280.0 }).min(if horizontal { (screen.x - 32.0).max(200.0) } else { (screen.y - 32.0).max(160.0) });
        let slide_vec = match side {
            Placement::Right => vec2(span * travel, 0.0),
            Placement::Left => vec2(-span * travel, 0.0),
            Placement::Bottom => vec2(0.0, span * travel),
            Placement::Top => vec2(0.0, -span * travel),
        };
        let rad = ui.look().card.radius + 2.0;
        let corners = match side {
            Placement::Right => Corners([rad, 0.0, 0.0, rad]),
            Placement::Left => Corners([0.0, rad, rad, 0.0]),
            Placement::Bottom => Corners([rad, rad, 0.0, 0.0]),
            Placement::Top => Corners([0.0, 0.0, rad, rad]),
        };

        let scrim = ui.color(Token::Scrim);
        let fill = ui.color(Token::SurfaceRaised);
        let border = ui.color(if ui.theme.darkness() > 0.5 { Token::BorderStrong } else { Token::Border });
        let shadows = Elevation::Level3.shadows(ui.theme.darkness(), ui.color(Token::ShadowInk));
        let pad = ui.look().card.pad + 2.0;

        let mut close_btn = false;
        let out = ui.layer(LayerPlace::Screen, vec2(0.0, 0.0), t, Some(scrim_id), |ui| {
            if let Some(root) = ui.layer_root_building() {
                let st = ui.node_style(root);
                match side {
                    Placement::Right => {
                        st.dir = Dir::Row;
                        st.justify = Justify::End;
                        st.align = Align::Stretch;
                    }
                    Placement::Left => {
                        st.dir = Dir::Row;
                        st.justify = Justify::Start;
                        st.align = Align::Stretch;
                    }
                    Placement::Bottom => {
                        st.dir = Dir::Column;
                        st.justify = Justify::End;
                        st.align = Align::Stretch;
                    }
                    Placement::Top => {
                        st.dir = Dir::Column;
                        st.justify = Justify::Start;
                        st.align = Align::Stretch;
                    }
                }
                ui.set_paint(root, Box::new(move |cx, r| cx.painter.rect(r, scrim)));
            }

            let mark = ui.focus_mark();
            let panel_style = if horizontal {
                Style { width: Size::Fixed(span), height: Size::FILL, ..Default::default() }
            } else {
                Style { width: Size::FILL, height: Size::Fixed(span), ..Default::default() }
            };
            ui.begin(
                panel_style,
                Some(panel_id),
                Sense::HOVER,
                CursorIcon::Default,
                true,
                Some(Box::new(move |cx, r| {
                    let r = r.translate(slide_vec);
                    cx.painter.shadows(r, corners, &shadows);
                    cx.painter.rrect_unsnapped(r, corners, [fill; 2], 1.0, border);
                })),
            );

            // Header.
            if title.is_some() || description.is_some() || dismissible {
                ui.begin(Style { dir: Dir::Row, width: Size::FILL, pad: Sides { left: pad, right: pad - 4.0, top: pad, bottom: 12.0 }, gap: 12.0, align: Align::Start, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                ui.begin(Style { width: Size::FILL, gap: 4.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                if let Some(t) = title {
                    let st = ui.text_style(TextRole::Heading3);
                    let fg = ui.color(Token::Foreground);
                    ui.text_leaf(t, st, fg, true, false, Style::default());
                }
                if let Some(d) = description {
                    let st = ui.text_style(TextRole::Body);
                    let sec = ui.color(Token::TextSecondary);
                    ui.text_leaf(d, st, sec, true, false, Style::default());
                }
                ui.end();
                if dismissible && ui.scope((id, "close"), |ui| ui.icon_button("x").ghost().small().tooltip("Close").clicked()) {
                    close_btn = true;
                }
                ui.end();
                let line = ui.color(Token::BorderSubtle);
                ui.leaf(Style { width: Size::FILL, ..Default::default() }, vec2(0.0, 1.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.rect(r, line))));
            }

            // Scrollable body.
            let body_mark = ui.focus_mark();
            ui.begin(Style { width: Size::FILL, height: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, true, None);
            let res = ui.scroll((id, "body")).pad(Sides::all(pad)).gap(14.0).show(body).0;
            ui.end();
            let body_items = ui.focusables_since(body_mark);

            // Optional footer.
            if let Some(f) = footer {
                let line = ui.color(Token::BorderSubtle);
                ui.leaf(Style { width: Size::FILL, ..Default::default() }, vec2(0.0, 1.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.rect(r, line))));
                ui.begin(Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(pad, 14.0), gap: 8.0, justify: Justify::End, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                f(ui);
                ui.end();
            }

            ui.end();
            let items = ui.focusables_since(mark);
            if just_opened {
                if let Some(&first) = body_items.first().or_else(|| items.last()) {
                    ui.request_focus(first);
                }
            }
            res
        });

        if close_btn || ui.take_layer_close() {
            *open = false;
            closed_by_user = true;
        }
        if closed_by_user {
            if let Some(prev) = ui.mem::<SheetMem>(id).prev_focus.take() {
                ui.request_focus(prev);
            }
        }
        is_open.then_some(out)
    }
}

// --------------------------------------------------------------- HoverCard

#[derive(Clone, Copy)]
struct HoverCardMem {
    entered_at: Option<f64>,
    left_at: Option<f64>,
    open: bool,
    dismissed: bool,
}

impl Default for HoverCardMem {
    fn default() -> Self {
        HoverCardMem { entered_at: None, left_at: None, open: false, dismissed: false }
    }
}

/// Builder for [`Ui::hover_card`]: a rich preview card that opens on hover
/// or keyboard focus and stays open while the pointer moves inside it.
#[must_use = "call .show(|ui| ...)"]
pub struct HoverCard<'u> {
    ui: &'u mut Ui,
    anchor: Response,
    placement: Placement,
    align: LayerAlign,
    gap: f32,
    width: f32,
    pad: f32,
    delay: f32,
    force_open: bool,
}

impl HoverCard<'_> {
    pub fn placement(mut self, p: Placement) -> Self {
        self.placement = p;
        self
    }
    pub fn align(mut self, a: LayerAlign) -> Self {
        self.align = a;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
    pub fn pad(mut self, p: f32) -> Self {
        self.pad = p;
        self
    }
    /// Hover delay in seconds before opening (default `0.28`).
    pub fn delay(mut self, secs: f32) -> Self {
        self.delay = secs.max(0.0);
        self
    }
    /// Force the card open immediately (useful in demos or screenshots).
    pub fn open(mut self, on: bool) -> Self {
        self.force_open = on;
        self
    }

    pub fn show<R>(self, f: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        let HoverCard { ui, anchor, placement, align, gap, width, pad, delay, force_open } = self;
        let id = anchor.id.with("hover-card");
        let panel_id = id.with("panel");
        let now = ui.time();
        let mut m = *ui.mem::<HoverCardMem>(id);

        let panel_rect = ui.rect_of(panel_id).unwrap_or_default();
        let in_panel = m.open && ui.pointer().is_some_and(|p| panel_rect.expand(gap + 4.0).contains(p));
        let in_anchor = anchor.hovered || ui.pointer().is_some_and(|p| anchor.rect.contains(p)) || (anchor.focused && ui.focus_visible());

        if !in_anchor && !in_panel {
            m.dismissed = false;
        }
        if m.open && (ui.key_pressed(Key::Escape) || ui.pressed_outside(&[panel_rect, anchor.rect])) {
            m.open = false;
            m.entered_at = None;
            m.left_at = None;
            m.dismissed = true;
        } else if force_open {
            m.open = true;
            m.left_at = None;
        } else if !m.dismissed && (in_anchor || in_panel) {
            m.left_at = None;
            if !m.open {
                let start = *m.entered_at.get_or_insert(now);
                let elapsed = (now - start) as f32;
                if elapsed >= delay {
                    m.open = true;
                } else {
                    ui.repaint_after_secs((delay - elapsed) as f64);
                }
            }
        } else {
            m.entered_at = None;
            if m.open {
                let left = *m.left_at.get_or_insert(now);
                let grace = 0.16f32;
                let elapsed = (now - left) as f32;
                if elapsed >= grace {
                    m.open = false;
                    m.left_at = None;
                } else {
                    ui.repaint_after_secs((grace - elapsed) as f64);
                }
            }
        }
        *ui.mem::<HoverCardMem>(id) = m;

        let mo = ui.motion();
        let t = ui.fade(id, 0, m.open, mo.layer_in, mo.fade_out, Ease::Out);
        if t <= 0.0 {
            return None;
        }
        let slide = ui.anims.spring_from(id, 1, 4.0, if m.open { 0.0 } else { 2.0 }, mo.snappy);
        let dir = match placement {
            Placement::Bottom => vec2(0.0, -1.0),
            Placement::Top => vec2(0.0, 1.0),
            Placement::Right => vec2(-1.0, 0.0),
            Placement::Left => vec2(1.0, 0.0),
        };
        let fill = ui.color(Token::SurfaceRaised);
        let border = ui.color(if ui.theme.darkness() > 0.5 { Token::BorderStrong } else { Token::Border });
        let shadows = Elevation::Level1.shadows(ui.theme.darkness(), ui.color(Token::ShadowInk));
        let radius = (ui.look().card.radius * 0.75).max(ui.look().control.radius[1]);
        let place = LayerPlace::Anchor { anchor: anchor.id, placement, gap, align, match_width: false };
        let out = ui.layer(place, dir * slide, t, None, |ui| {
            ui.begin(
                Style { width: Size::Fixed(width), pad: Sides::all(pad), gap: 10.0, ..Default::default() },
                Some(panel_id),
                Sense::HOVER,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| {
                    cx.painter.shadows(r, Corners::all(radius), &shadows);
                    cx.painter.rrect(r, Corners::all(radius), fill, 1.0, border);
                })),
            );
            let r = f(ui);
            ui.end();
            r
        });
        if ui.take_layer_close() {
            ui.mem::<HoverCardMem>(id).open = false;
        }
        m.open.then_some(out)
    }
}

// ------------------------------------------------------------- ContextMenu

#[derive(Default)]
struct ContextMenuMem {
    at: Vec2,
}

// ---------------------------------------------------------- ConfirmPopover

/// Builder for [`Ui::confirm_popover`]: an inline confirmation anchored to
/// a destructive or important button.
#[must_use = "call .confirmed() or .show()"]
pub struct ConfirmPopover<'u> {
    ui: &'u mut Ui,
    anchor: Response,
    title: String,
    description: Option<String>,
    confirm_label: String,
    cancel_label: String,
    danger: bool,
    placement: Placement,
    width: f32,
}

impl ConfirmPopover<'_> {
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    pub fn confirm_label(mut self, l: impl Into<String>) -> Self {
        self.confirm_label = l.into();
        self
    }
    pub fn cancel_label(mut self, l: impl Into<String>) -> Self {
        self.cancel_label = l.into();
        self
    }
    /// Style the confirm button as destructive (default `true`) or primary (`false`).
    pub fn danger(mut self, d: bool) -> Self {
        self.danger = d;
        self
    }
    pub fn placement(mut self, p: Placement) -> Self {
        self.placement = p;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    /// Show the popover when open; returns `true` on the frame the user
    /// confirms.
    pub fn confirmed(self) -> bool {
        self.show()
    }

    pub fn show(self) -> bool {
        let ConfirmPopover { ui, anchor, title, description, confirm_label, cancel_label, danger, placement, width } = self;
        let mut confirmed = false;
        let id = anchor.id;
        ui.popover(&anchor).placement(placement).width(width).pad(14.0).show(|ui| {
            ui.begin(Style { width: Size::FILL, gap: 12.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 10.0, align: Align::Start, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let ic = ui.color(if danger { Token::DangerStrong } else { Token::AccentStrong });
            ui.begin(Style { pad: Sides { top: 2.0, ..Default::default() }, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            ui.icon_leaf(if danger { "triangle-alert" } else { "info" }, 16.0, ic);
            ui.end();
            ui.begin(Style { width: Size::FILL, gap: 4.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let tst = ui.text_style(TextRole::BodyStrong);
            let fg = ui.color(Token::Foreground);
            ui.text_leaf(title, tst, fg, true, false, Style::default());
            if let Some(d) = description {
                let bst = ui.text_style(TextRole::Body);
                let sec = ui.color(Token::TextSecondary);
                ui.text_leaf(d, bst, sec, true, false, Style::default());
            }
            ui.end();
            ui.end();

            ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, justify: Justify::End, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            if ui.scope((id, "cancel"), |ui| ui.button(cancel_label).ghost().small().clicked()) {
                ui.close_layer();
            }
            let ok = ui.scope((id, "confirm"), |ui| {
                let b = ui.button(confirm_label).small();
                if danger { b.danger().clicked() } else { b.primary().clicked() }
            });
            if ok {
                confirmed = true;
                ui.close_layer();
            }
            ui.end();
            ui.end();
        });
        confirmed
    }
}

impl Ui {
    /// An edge-docked sheet (drawer) over a scrim, shown while `*open`.
    pub fn sheet<'u, 'a>(&'u mut self, key: impl Into<String>, open: &'u mut bool) -> Sheet<'u, 'a> {
        Sheet { ui: self, open, key: key.into(), side: Placement::Right, title: None, description: None, size: None, footer: None, dismissible: true }
    }

    /// A rich hover card anchored to `anchor`. Opens after a short delay on
    /// hover or keyboard focus, and stays open while the pointer is inside.
    pub fn hover_card(&mut self, anchor: &Response) -> HoverCard<'_> {
        HoverCard {
            ui: self,
            anchor: *anchor,
            placement: Placement::Bottom,
            align: LayerAlign::Start,
            gap: 6.0,
            width: 280.0,
            pad: 14.0,
            delay: 0.28,
            force_open: false,
        }
    }

    /// A right-click (or `Shift+Enter` while focused) context menu for
    /// `anchor`. Populate with [`menu_item`](Self::menu_item),
    /// [`menu_separator`](Self::menu_separator) and [`menu_label`](Self::menu_label).
    pub fn context_menu(&mut self, anchor: &Response, f: impl FnOnce(&mut Ui)) {
        let ctx_id = anchor.id.with("context-menu");
        if let Some(p) = self.secondary_pressed_in(anchor.rect) {
            self.mem::<ContextMenuMem>(ctx_id).at = p;
            self.set_popover_open(ctx_id, true);
        } else if anchor.focused && self.modifiers().shift && self.key_pressed_for(anchor.id, Key::Enter) {
            self.mem::<ContextMenuMem>(ctx_id).at = anchor.rect.center();
            let st = self.mem::<PopState>(ctx_id);
            st.open = true;
            st.just_opened = true;
            st.focus_in = true;
        }
        let at = self.mem::<ContextMenuMem>(ctx_id).at;
        let synth = Response { id: ctx_id, rect: anchor.rect, ..Default::default() };
        let prev = self.menu.replace(MenuCtx { anchor: ctx_id });
        self.popover(&synth).manual().at(at).return_focus(anchor.id).pad(4.0).show(|ui| {
            ui.begin(Style { width: Size::FILL, min: vec2(188.0, 0.0), gap: 1.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            f(ui);
            ui.end();
        });
        self.menu = prev;
    }

    /// Programmatically open the context menu attached to `anchor_id` at `at`.
    pub fn open_context_menu(&mut self, anchor_id: Id, at: Vec2) {
        let ctx_id = anchor_id.with("context-menu");
        self.mem::<ContextMenuMem>(ctx_id).at = at;
        self.set_popover_open(ctx_id, true);
    }

    /// An anchored confirmation popover for `anchor`. Returns a builder
    /// whose `.confirmed()` is `true` on the frame the user confirms.
    pub fn confirm_popover(&mut self, anchor: &Response, title: impl Into<String>) -> ConfirmPopover<'_> {
        ConfirmPopover {
            ui: self,
            anchor: *anchor,
            title: title.into(),
            description: None,
            confirm_label: "Confirm".into(),
            cancel_label: "Cancel".into(),
            danger: true,
            placement: Placement::Bottom,
            width: 284.0,
        }
    }
}
