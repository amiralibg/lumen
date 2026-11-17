//! Popover: a floating panel anchored to a widget. The base of menus,
//! selects and the date picker.

use crate::anim::Ease;
use crate::id::Id;
use crate::input::{CursorIcon, Key};
use crate::layout::{Size, Style};
use crate::math::{vec2, Corners, Sides, Vec2};
use crate::theme::{Elevation, Token};
use crate::ui::{LayerAlign, LayerPlace, Placement, Response, Sense, Ui};

#[derive(Default)]
pub(crate) struct PopState {
    pub open: bool,
    /// Opened from the keyboard: move focus into the panel.
    focus_in: bool,
    /// Focusable ids inside the panel last frame.
    pub items: Vec<Id>,
}

/// Builder for a popover attached to a widget's [`Response`].
#[must_use = "call .show(|ui| ...)"]
pub struct Popover<'u> {
    ui: &'u mut Ui,
    anchor: Response,
    placement: Placement,
    align: LayerAlign,
    gap: f32,
    match_width: bool,
    pad: f32,
    width: Option<f32>,
    toggle: bool,
}

impl Popover<'_> {
    pub fn placement(mut self, p: Placement) -> Self {
        self.placement = p;
        self
    }
    pub fn align(mut self, a: LayerAlign) -> Self {
        self.align = a;
        self
    }
    pub fn gap(mut self, g: f32) -> Self {
        self.gap = g;
        self
    }
    /// At least as wide as the anchor.
    pub fn match_width(mut self) -> Self {
        self.match_width = true;
        self
    }
    pub fn pad(mut self, p: f32) -> Self {
        self.pad = p;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }
    /// Don't open or close when the anchor is clicked (control it with
    /// [`Ui::set_popover_open`]).
    pub fn manual(mut self) -> Self {
        self.toggle = false;
        self
    }

    /// Show the panel when open. Returns the closure's result while open.
    pub fn show<R>(self, f: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        let Popover { ui, anchor, placement, align, gap, match_width, pad, width, toggle } = self;
        let id = anchor.id;
        let panel_id = id.with("popover-panel");
        if toggle && anchor.clicked {
            let kb = ui.focus_visible();
            let st = ui.mem::<PopState>(id);
            st.open = !st.open;
            st.focus_in = st.open && kb;
        }
        let open = ui.mem::<PopState>(id).open;
        // Close on Escape, or a press outside the panel and the anchor.
        if open {
            let panel_r = ui.rect_of(panel_id).unwrap_or_default();
            let esc = ui.key_pressed(Key::Escape);
            if esc || ui.pressed_outside(&[panel_r, anchor.rect]) {
                ui.mem::<PopState>(id).open = false;
                if esc {
                    ui.request_focus(id);
                }
            }
        }
        let open = ui.mem::<PopState>(id).open;
        let mo = ui.motion();
        let t = ui.fade(id, 700, open, mo.layer_in, mo.fade_out, Ease::Out);
        if t <= 0.0 {
            ui.mem::<PopState>(id).items.clear();
            return None;
        }
        let slide = ui.anims.spring_from(id, 701, 6.0, if open { 0.0 } else { 3.0 }, mo.snappy);
        let dir = match placement {
            Placement::Bottom => vec2(0.0, -1.0),
            Placement::Top => vec2(0.0, 1.0),
            Placement::Right => vec2(-1.0, 0.0),
            Placement::Left => vec2(1.0, 0.0),
        };
        let fill = ui.color(Token::SurfaceRaised);
        let border = ui.color(Token::Border);
        let shadows = Elevation::Level2.shadows(ui.theme.darkness(), ui.color(Token::ShadowInk));
        let radius = (ui.look().card.radius * 0.75).max(ui.look().control.radius[1]);
        let place = LayerPlace::Anchor { anchor: id, placement, gap, align, match_width };
        let r = ui.layer(place, dir * slide, t, None, |ui| {
            let mark = ui.focus_mark();
            ui.begin(
                Style {
                    width: width.map(Size::Fixed).unwrap_or(if match_width { Size::FILL } else { Size::Hug }),
                    min: vec2(if match_width { 0.0 } else { 120.0 }, 0.0),
                    pad: Sides::all(pad),
                    gap: 0.0,
                    ..Default::default()
                },
                Some(panel_id),
                Sense::HOVER,
                CursorIcon::Default,
                true,
                Some(Box::new(move |cx, r| {
                    cx.painter.shadows(r, Corners::all(radius), &shadows);
                    cx.painter.rrect(r, Corners::all(radius), fill, 1.0, border);
                })),
            );
            let out = f(ui);
            ui.end();
            let items = ui.focusables_since(mark);
            (out, items)
        });
        let (out, items) = r;
        let close = ui.take_layer_close();
        let st = ui.mem::<PopState>(id);
        st.items = items.clone();
        if st.focus_in && !items.is_empty() {
            st.focus_in = false;
            ui.request_focus(items[0]);
        }
        if close {
            ui.mem::<PopState>(id).open = false;
            ui.request_focus(id);
        }
        open.then_some(out)
    }
}

impl Ui {
    /// A popover attached to `anchor` (usually a button's response). It
    /// toggles when the anchor is clicked and closes on Escape or a click
    /// outside.
    pub fn popover(&mut self, anchor: &Response) -> Popover<'_> {
        Popover {
            ui: self,
            anchor: *anchor,
            placement: Placement::Bottom,
            align: LayerAlign::Start,
            gap: 6.0,
            match_width: false,
            pad: 6.0,
            width: None,
            toggle: true,
        }
    }

    /// Open or close the popover anchored at `anchor`.
    pub fn set_popover_open(&mut self, anchor: Id, open: bool) {
        self.mem::<PopState>(anchor).open = open;
    }

    pub fn is_popover_open(&mut self, anchor: Id) -> bool {
        self.mem::<PopState>(anchor).open
    }

    /// Up/Down move focus between the focusable ids of an open popover.
    pub(crate) fn arrow_focus(&mut self, me: Id, items: &[Id]) {
        if let Some(i) = items.iter().position(|&x| x == me) {
            let n = items.len();
            if self.key_pressed_for(me, Key::ArrowDown) {
                self.request_focus(items[(i + 1) % n]);
            }
            if self.key_pressed_for(me, Key::ArrowUp) {
                self.request_focus(items[(i + n - 1) % n]);
            }
            if self.key_pressed_for(me, Key::Home) {
                self.request_focus(items[0]);
            }
            if self.key_pressed_for(me, Key::End) {
                self.request_focus(items[n - 1]);
            }
        }
    }
}

/// Offset helper for callers that animate their own layers.
pub fn slide_offset(placement: Placement, d: f32) -> Vec2 {
    match placement {
        Placement::Bottom => vec2(0.0, -d),
        Placement::Top => vec2(0.0, d),
        Placement::Right => vec2(-d, 0.0),
        Placement::Left => vec2(d, 0.0),
    }
}
