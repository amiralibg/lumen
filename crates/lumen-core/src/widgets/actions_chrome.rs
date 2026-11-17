//! Action variants and application chrome: [`SplitButton`], [`HoldButton`],
//! [`Ui::copy_button`], [`Dropzone`], [`Toolbar`], [`StatusBar`] and
//! segmented [`Meter`].

use super::button::Variant;
use super::toast::Tone;
use super::{blend3, key_activate, neutral_layers, show_on_drop};
use crate::anim::Ease;
use crate::id::Id;
use crate::input::CursorIcon;
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{ControlSize, Elevation, TextRole, Token};
use crate::ui::{Placement, Response, Sense, TooltipSpec, Ui};
use crate::Color;

// ============================================================= SplitButton

/// Primary or secondary action joined with a trailing chevron that opens a
/// [`menu`](Ui::menu) of related actions.
pub struct SplitButton<'u> {
    ui: &'u mut Ui,
    label: String,
    variant: Variant,
    icon: Option<String>,
    size: ControlSize,
    disabled: bool,
}

impl<'u> SplitButton<'u> {
    pub fn primary(mut self) -> Self {
        self.variant = Variant::Primary;
        self
    }
    pub fn secondary(mut self) -> Self {
        self.variant = Variant::Secondary;
        self
    }
    pub fn danger(mut self) -> Self {
        self.variant = Variant::Danger;
        self
    }
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = Some(name.to_owned());
        self
    }
    pub fn small(mut self) -> Self {
        self.size = ControlSize::Sm;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }

    /// Show the split button and attach `menu_fn` to its trailing chevron.
    /// Returns `true` when the main action half is clicked.
    pub fn show(self, menu_fn: impl FnOnce(&mut Ui)) -> Response {
        let SplitButton { ui, label, variant, icon, size, disabled } = self;
        let id = ui.make_id(("split_button", &label));
        let main_id = id.with("main");
        let drop_id = id.with("drop");
        let m = ui.control_metrics(size);
        let mo = ui.motion();

        let main_ix = ui.interact(main_id, Sense::CLICK, disabled);
        let drop_ix = ui.interact(drop_id, Sense::CLICK, disabled);
        let (mk_act, mk_held) = key_activate(ui, &main_ix, true);
        let (dk_act, dk_held) = key_activate(ui, &drop_ix, true);
        let main_clicked = (main_ix.clicked || mk_act) && !disabled;
        let open = ui.is_popover_open(drop_id);

        let (mh, mp) = ui.state_weights(&main_ix, mk_held);
        let (dh, dp) = ui.state_weights(&drop_ix, dk_held);
        let m_inset = ui.spring(main_id, 0, if main_ix.pressed || mk_held { m.inset } else { 0.0 }, mo.snappy);
        let d_inset = ui.spring(drop_id, 0, if drop_ix.pressed || dk_held { m.inset } else { 0.0 }, mo.snappy);
        let chev = ui.spring(drop_id, 3, if open { 1.0 } else { 0.0 }, mo.snappy);
        let (m_ra, m_rg) = ui.focus_anim(&main_ix);
        let (d_ra, d_rg) = ui.focus_anim(&drop_ix);

        let c = |t| ui.color(t);
        let colors = |h: f32, p: f32| -> (Color, Color, Color, Color) {
            if disabled {
                return (c(Token::SurfaceMuted), c(Token::BorderSubtle), c(Token::TextDisabled), c(Token::TextDisabled));
            }
            match variant {
                Variant::Primary => (
                    blend3(c(Token::Accent), c(Token::AccentHover), c(Token::AccentPressed), h, p),
                    Color::TRANSPARENT,
                    c(Token::AccentForeground),
                    c(Token::AccentForeground).with_alpha(0.24),
                ),
                Variant::Danger => (
                    blend3(c(Token::Danger), c(Token::DangerHover), c(Token::DangerPressed), h, p),
                    Color::TRANSPARENT,
                    c(Token::DangerForeground),
                    c(Token::DangerForeground).with_alpha(0.24),
                ),
                _ => {
                    let base = c(Token::Control);
                    let (hv, pr) = neutral_layers(ui, base);
                    let cb = c(Token::ControlBorder);
                    let b = if cb.a > 0.0 { cb.mix(c(Token::BorderStrong), h.max(p)) } else { cb };
                    (blend3(base, hv, pr, h, p), b, c(Token::Foreground), c(Token::BorderSubtle))
                }
            }
        };
        let (m_fill, m_bd, fg, seam) = colors(mh, mp);
        let (d_fill, d_bd, _, _) = colors(dh.max(if open { 0.8 } else { 0.0 }), dp);
        let ring = if variant == Variant::Danger { c(Token::DangerStrong) } else { c(Token::FocusRing) };
        let rad = m.radius;
        let bw = if m_bd.a > 0.0 { 1.0 } else { 0.0 };

        ui.begin(Style { dir: Dir::Row, height: Size::Fixed(m.height), align: Align::Center, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);

        // Left action button.
        let left_pad = Sides { left: if icon.is_some() { m.pad_icon } else { m.pad_x }, right: m.pad_x - 2.0, top: 0.0, bottom: 0.0 };
        ui.begin(
            Style { dir: Dir::Row, height: Size::Fixed(m.height), pad: left_pad, gap: m.gap, align: Align::Center, justify: Justify::Center, ..Default::default() },
            Some(main_id),
            Sense::CLICK,
            if disabled { CursorIcon::Default } else { CursorIcon::Pointer },
            false,
            Some(Box::new(move |cx, r| {
                let bg = cx.painter.snap_rect(Rect { min: r.min + vec2(m_inset, m_inset), max: r.max - vec2(0.0, m_inset) });
                let cr = Corners([rad, 0.0, 0.0, rad]);
                cx.painter.rrect_unsnapped(bg, cr, [m_fill; 2], bw, m_bd);
                if m_ra > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, rad, m_ra, m_rg, ring)));
                }
            })),
        );
        if let Some(ic) = &icon {
            ui.icon_leaf(ic, m.icon, fg);
        }
        ui.text_leaf(label, m.text, fg, false, true, Style::default());
        ui.end();

        // Seam divider.
        ui.leaf(
            Style::default(),
            vec2(1.0, m.height),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                cx.painter.rect(r, m_fill);
                cx.painter.rect(Rect::from_xywh(r.min.x, r.min.y + 5.0, 1.0, (r.height() - 10.0).max(4.0)), seam);
            })),
        );

        // Right chevron trigger.
        let drop_w = (m.height * 0.88).round().max(28.0);
        let icon_sz = (m.icon - 1.0).max(13.0);
        ui.leaf(
            Style::default(),
            vec2(drop_w, m.height),
            Some(drop_id),
            Sense::CLICK,
            if disabled { CursorIcon::Default } else { CursorIcon::Pointer },
            Some(Box::new(move |cx, r| {
                let bg = cx.painter.snap_rect(Rect { min: r.min + vec2(0.0, d_inset), max: r.max - vec2(d_inset, d_inset) });
                let cr = Corners([0.0, rad, rad, 0.0]);
                cx.painter.rrect_unsnapped(bg, cr, [d_fill; 2], bw, d_bd);
                let c0 = fg.multiply_alpha(1.0 - chev.clamp(0.0, 1.0));
                let c1 = fg.multiply_alpha(chev.clamp(0.0, 1.0));
                cx.icon("chevron-down", bg.center() + vec2(0.0, 1.5 * chev), icon_sz, c0);
                cx.icon("chevron-up", bg.center() + vec2(0.0, 1.5 * (chev - 1.0)), icon_sz, c1);
                if d_ra > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, rad, d_ra, d_rg, ring)));
                }
            })),
        );
        ui.end();

        let anchor = Response {
            id: drop_id,
            rect: ui.rect_of(id).unwrap_or(drop_ix.rect),
            hovered: drop_ix.hovered,
            clicked: (drop_ix.clicked || dk_act) && !disabled,
            changed: false,
            focused: drop_ix.focused,
        };
        ui.menu(&anchor, menu_fn);

        let rect = ui.rect_of(id).unwrap_or(main_ix.rect);
        Response { id, rect, hovered: main_ix.hovered || drop_ix.hovered, clicked: main_clicked, changed: main_clicked, focused: main_ix.focused || drop_ix.focused }
    }
}

// ============================================================== HoldButton

#[derive(Default)]
struct HoldMem {
    started: Option<f64>,
    fired: bool,
}

/// Hold-to-confirm button: press and hold for `duration` seconds to fill the
/// button and fire the action; releasing early springs the fill back to zero.
pub struct HoldButton<'u> {
    ui: &'u mut Ui,
    label: String,
    duration: f32,
    variant: Variant,
    icon: Option<String>,
    size: ControlSize,
    disabled: bool,
    shown: bool,
}

impl<'u> HoldButton<'u> {
    pub fn primary(mut self) -> Self {
        self.variant = Variant::Primary;
        self
    }
    pub fn secondary(mut self) -> Self {
        self.variant = Variant::Secondary;
        self
    }
    pub fn danger(mut self) -> Self {
        self.variant = Variant::Danger;
        self
    }
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = Some(name.to_owned());
        self
    }
    pub fn small(mut self) -> Self {
        self.size = ControlSize::Sm;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    /// Alias for `.clicked()`: `true` on the frame the hold completes.
    pub fn confirmed(self) -> bool {
        self.clicked()
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("hold_button", &self.label));
        let ix = ui.interact(id, Sense::CLICK, self.disabled);
        let (_, k_held) = key_activate(ui, &ix, true);
        let holding = (ix.pressed || k_held) && !self.disabled;
        let now = ui.time();
        let dur = self.duration.max(0.2) as f64;

        let mut confirmed = false;
        let raw_prog = {
            let st = ui.mem::<HoldMem>(id);
            if holding {
                let s = *st.started.get_or_insert(now);
                let p = ((now - s) / dur).clamp(0.0, 1.0) as f32;
                if p >= 1.0 && !st.fired {
                    st.fired = true;
                    confirmed = true;
                }
                p
            } else {
                st.started = None;
                st.fired = false;
                0.0
            }
        };
        if holding && raw_prog < 1.0 {
            ui.request_repaint_after(0.0);
        }

        let mo = ui.motion();
        let m = ui.control_metrics(self.size);
        let prog = if holding {
            ui.anims.set_spring(id, 4, raw_prog);
            raw_prog
        } else {
            ui.spring(id, 4, 0.0, mo.snappy).clamp(0.0, 1.0)
        };
        let hv = ui.fade(id, 1, ix.hovered && !self.disabled, mo.hover_in, mo.hover_out, Ease::Linear);
        let squash_t = ui.spring(id, 2, if holding { 1.0 } else { 0.0 }, mo.snappy);
        let (ring_a, ring_gap) = ui.focus_anim(&ix);

        let c = |t| ui.color(t);
        let (base_fill, sweep_fill, border, fg, ring) = if self.disabled {
            (c(Token::SurfaceMuted), Color::TRANSPARENT, c(Token::BorderSubtle), c(Token::TextDisabled), c(Token::FocusRing))
        } else {
            match self.variant {
                Variant::Danger => (
                    c(Token::DangerSubtle).mix(c(Token::Surface), 0.35),
                    c(Token::Danger),
                    c(Token::Danger).with_alpha(0.45),
                    c(Token::DangerStrong).mix(c(Token::DangerForeground), prog),
                    c(Token::DangerStrong),
                ),
                Variant::Primary => (
                    c(Token::AccentSubtle).mix(c(Token::Surface), 0.35),
                    c(Token::Accent),
                    c(Token::Accent).with_alpha(0.45),
                    c(Token::AccentStrong).mix(c(Token::AccentForeground), prog),
                    c(Token::FocusRing),
                ),
                _ => (
                    c(Token::Control).mix(c(Token::Control).over(c(Token::Hover)), hv),
                    c(Token::Accent),
                    c(Token::ControlBorder),
                    c(Token::Foreground).mix(c(Token::AccentForeground), prog),
                    c(Token::FocusRing),
                ),
            }
        };
        let rad = m.radius;
        let pad = Sides { left: if self.icon.is_some() { m.pad_icon } else { m.pad_x }, right: m.pad_x, top: 0.0, bottom: 0.0 };

        ui.begin(
            Style { dir: Dir::Row, height: Size::Fixed(m.height), pad, gap: m.gap, align: Align::Center, justify: Justify::Center, ..Default::default() },
            Some(id),
            Sense::CLICK,
            if self.disabled { CursorIcon::Default } else { CursorIcon::Pointer },
            false,
            Some(Box::new(move |cx, r| {
                let sr = Rect::from_center_size(r.center(), r.size() * (1.0 - 0.03 * squash_t));
                cx.painter.rrect_unsnapped(sr, Corners::all(rad), [base_fill; 2], 1.0, border);
                if prog > 0.005 {
                    cx.painter.push_clip(Rect::from_xywh(sr.min.x, sr.min.y, sr.width() * prog, sr.height()));
                    cx.painter.rrect_unsnapped(sr, Corners::all(rad), [sweep_fill; 2], 0.0, Color::TRANSPARENT);
                    cx.painter.pop_clip();
                }
                if ring_a > 0.0 {
                    cx.defer(sr, Box::new(move |cx, r| cx.focus_ring(r, rad, ring_a, ring_gap, ring)));
                }
            })),
        );
        if let Some(ic) = &self.icon {
            ui.icon_leaf(ic, m.icon, fg);
        }
        ui.text_leaf(self.label.clone(), m.text, fg, false, true, Style::default());
        ui.end();

        let rect = ui.rect_of(id).unwrap_or(ix.rect);
        Response { id, rect, hovered: ix.hovered, clicked: confirmed, changed: confirmed, focused: ix.focused }
    }
}

show_on_drop!(HoldButton);

// =============================================================== Dropzone

/// File drop / upload surface with an animated upload tile, format hint and
/// removable file list.
pub struct Dropzone<'u> {
    ui: &'u mut Ui,
    key: String,
    title: String,
    hint: Option<String>,
    files: Option<&'u mut Vec<(String, String)>>,
    disabled: bool,
    shown: bool,
}

impl<'u> Dropzone<'u> {
    pub fn hint(mut self, h: impl Into<String>) -> Self {
        self.hint = Some(h.into());
        self
    }
    /// Attach a mutable list of `(filename, size_label)` entries rendered below the zone.
    pub fn files(mut self, list: &'u mut Vec<(String, String)>) -> Self {
        self.files = Some(list);
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("dropzone", &self.key));
        let ix = ui.interact(id, Sense::CLICK, self.disabled);
        let (k_act, k_held) = key_activate(ui, &ix, true);
        let clicked = (ix.clicked || k_act) && !self.disabled;
        let mo = ui.motion();

        let hv = ui.fade(id, 0, ix.hovered && !self.disabled, mo.hover_in, mo.hover_out, Ease::Linear);
        let lift = ui.spring(id, 1, if ix.hovered && !self.disabled { 1.0 } else { 0.0 }, mo.bouncy);
        let press = ui.spring(id, 2, if (ix.pressed || k_held) && !self.disabled { 1.0 } else { 0.0 }, mo.snappy);
        let (ring_a, ring_gap) = ui.focus_anim(&ix);

        let (fill, border, tile_bg, ic_c, ring, tc, hc) = {
            let c = |t| ui.color(t);
            let fill = if self.disabled { c(Token::SurfaceMuted) } else { c(Token::Surface).mix(c(Token::AccentSubtle), hv * 0.38) };
            let border = if self.disabled { c(Token::BorderSubtle) } else { c(Token::BorderStrong).mix(c(Token::Accent), hv * 0.75) };
            let tile_bg = c(Token::SurfaceMuted).mix(c(Token::AccentSubtle), hv);
            let ic_c = if self.disabled { c(Token::TextDisabled) } else { c(Token::TextSecondary).mix(c(Token::AccentStrong), hv) };
            let tc = if self.disabled { c(Token::TextDisabled) } else { c(Token::Foreground) };
            let hc = if self.disabled { c(Token::TextDisabled) } else { c(Token::TextMuted) };
            (fill, border, tile_bg, ic_c, c(Token::FocusRing), tc, hc)
        };
        let rad = ui.look().card.radius;

        ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 10.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);

        ui.begin(
            Style { dir: Dir::Column, width: Size::FILL, pad: Sides::xy(20.0, 22.0), gap: 8.0, align: Align::Center, justify: Justify::Center, ..Default::default() },
            Some(id),
            Sense::CLICK,
            if self.disabled { CursorIcon::Default } else { CursorIcon::Pointer },
            false,
            Some(Box::new(move |cx, r| {
                let sr = Rect::from_center_size(r.center(), r.size() * (1.0 - 0.02 * press));
                cx.painter.rrect_unsnapped(sr, Corners::all(rad), [fill; 2], 1.0, border);
                if ring_a > 0.0 {
                    cx.defer(sr, Box::new(move |cx, r| cx.focus_ring(r, rad, ring_a, ring_gap, ring)));
                }
            })),
        );

        ui.leaf(
            Style::default(),
            vec2(40.0, 40.0),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let tr = r.translate(vec2(0.0, -2.5 * lift));
                cx.painter.rrect_unsnapped(tr, Corners::all(10.0), [tile_bg; 2], 0.0, Color::TRANSPARENT);
                cx.icon("cloud-upload", tr.center(), 19.0, ic_c);
            })),
        );

        let st_t = ui.text_style(TextRole::BodyStrong);
        ui.text_leaf(self.title.clone(), st_t, tc, false, true, Style::default());

        if let Some(h) = &self.hint {
            let st_h = ui.text_style(TextRole::Caption);
            ui.text_leaf(h.clone(), st_h, hc, false, true, Style::default());
        }
        ui.end();

        let mut removed = false;
        if let Some(files) = &mut self.files {
            let mut rm_idx = None;
            for (i, (name, sz)) in files.iter().enumerate() {
                let fid = id.with(("file", name.as_str()));
                let enter = ui.anims.spring_from(fid, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);
                let row_bg = ui.color(Token::Surface).multiply_alpha(enter);
                let row_bd = ui.color(Token::BorderSubtle).multiply_alpha(enter);
                let r_rad = ui.look().control.radius[1];
                ui.begin(
                    Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(34.0), pad: Sides { left: 10.0, right: 6.0, top: 0.0, bottom: 0.0 }, gap: 8.0, align: Align::Center, ..Default::default() },
                    Some(fid),
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_rad), row_bg, 1.0, row_bd))),
                );
                let ic = ui.color(Token::AccentStrong).multiply_alpha(enter);
                ui.icon_leaf("file", 15.0, ic);
                let st_n = ui.text_style(TextRole::Body);
                let nc = ui.color(Token::Foreground).multiply_alpha(enter);
                ui.text_leaf(name.clone(), st_n, nc, false, true, Style { width: Size::FILL, ..Default::default() });
                let st_s = ui.text_style(TextRole::Caption);
                let sc = ui.color(Token::TextMuted).multiply_alpha(enter);
                ui.text_leaf(sz.clone(), st_s, sc, false, false, Style::default());
                if !self.disabled && ui.field_icon_button(fid.with("rm"), "x", enter, true) {
                    rm_idx = Some(i);
                }
                ui.end();
            }
            if let Some(i) = rm_idx {
                files.remove(i);
                removed = true;
            }
        }
        ui.end();

        let rect = ui.rect_of(id).unwrap_or(ix.rect);
        Response { id, rect, hovered: ix.hovered, clicked, changed: clicked || removed, focused: ix.focused }
    }
}

show_on_drop!(Dropzone);

// ================================================================ Toolbar

/// Builder passed to [`Ui::toolbar`].
pub struct ToolbarScope<'u> {
    pub ui: &'u mut Ui,
    id: Id,
    count: usize,
}

impl ToolbarScope<'_> {
    /// Icon toggle button (e.g. Bold, Italic, Grid).
    pub fn toggle(&mut self, icon: &str, tooltip: &str, on: &mut bool) -> bool {
        let tid = self.id.with(("t", self.count, icon));
        self.count += 1;
        let ix = self.ui.interact(tid, Sense::CLICK, false);
        let (k_act, k_held) = key_activate(self.ui, &ix, true);
        let mut changed = false;
        if ix.clicked || k_act {
            *on = !*on;
            changed = true;
        }
        let mo = self.ui.motion();
        let sel = self.ui.spring(tid, 0, if *on { 1.0 } else { 0.0 }, mo.snappy).clamp(0.0, 1.0);
        let hv = self.ui.fade(tid, 1, ix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
        let press = self.ui.spring(tid, 2, if ix.pressed || k_held { 1.0 } else { 0.0 }, mo.snappy);
        let (ring_a, ring_gap) = self.ui.focus_anim(&ix);
        let bg = self.ui.color(Token::Hover).multiply_alpha(hv * (1.0 - sel)).mix(self.ui.color(Token::AccentSubtle), sel);
        let bd = Color::TRANSPARENT.mix(self.ui.color(Token::Accent).with_alpha(0.45), sel);
        let ic = self
            .ui
            .color(Token::TextSecondary)
            .mix(self.ui.color(Token::Foreground), hv)
            .mix(self.ui.color(Token::AccentStrong), sel);
        let ring = self.ui.color(Token::FocusRing);
        let rad = self.ui.look().control.radius[0].max(5.0);
        let name = icon.to_owned();
        self.ui.leaf(
            Style::default(),
            vec2(30.0, 30.0),
            Some(tid),
            Sense::CLICK,
            CursorIcon::Pointer,
            Some(Box::new(move |cx, r| {
                let sr = Rect::from_center_size(r.center(), r.size() * (1.0 - 0.06 * press));
                if bg.a > 0.0 || bd.a > 0.0 {
                    cx.painter.rrect_unsnapped(sr, Corners::all(rad), [bg; 2], 1.0, bd);
                }
                cx.icon(&name, sr.center(), 15.0, ic);
                if ring_a > 0.0 {
                    cx.defer(sr, Box::new(move |cx, r| cx.focus_ring(r, rad, ring_a, ring_gap, ring)));
                }
            })),
        );
        if !tooltip.is_empty() {
            self.ui.tooltip_for(tid, TooltipSpec { text: tooltip.into(), shortcut: None, placement: Placement::Top, gap: 6.0 });
        }
        changed
    }

    /// Momentary icon action button.
    pub fn action(&mut self, icon: &str, tooltip: &str) -> bool {
        let mut dummy = false;
        let clicked = self.toggle(icon, tooltip, &mut dummy);
        clicked
    }

    /// Vertical hairline separator between groups.
    pub fn separator(&mut self) {
        let line = self.ui.color(Token::BorderSubtle);
        self.ui.leaf(Style { margin_top: 0.0, ..Default::default() }, vec2(5.0, 18.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| {
            cx.painter.rect(Rect::from_xywh(r.center().x.round(), r.min.y, 1.0, r.height()), line);
        })));
    }

    /// Trailing primary pill action inside the toolbar.
    pub fn primary(&mut self, label: &str) -> bool {
        self.ui.button(label).primary().small().clicked()
    }
}

// ============================================================== StatusBar

/// Builder passed to [`Ui::status_bar`].
pub struct StatusBarScope<'u> {
    pub ui: &'u mut Ui,
    id: Id,
    count: usize,
}

impl StatusBarScope<'_> {
    /// Colored live status dot + label.
    pub fn status(&mut self, label: &str, tone: Tone) {
        let (_, _, solid) = tone.tokens();
        let dot = self.ui.color(solid);
        let fg = self.ui.color(Token::Foreground);
        let st = self.ui.text_style(TextRole::Caption);
        self.ui.begin(Style { dir: Dir::Row, pad: Sides::xy(6.0, 0.0), gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.ui.leaf(Style::default(), vec2(8.0, 8.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| {
            cx.painter.circle(r.center(), 4.0, dot.with_alpha(0.24), 0.0, Color::TRANSPARENT);
            cx.painter.circle(r.center(), 2.6, dot, 0.0, Color::TRANSPARENT);
        })));
        self.ui.text_leaf(label, st, fg, false, true, Style::default());
        self.ui.end();
    }

    /// Clickable icon + caption segment (e.g. git branch, region, problems count).
    pub fn item(&mut self, icon: &str, label: &str) -> bool {
        let sid = self.id.with(("sb", self.count, label));
        self.count += 1;
        let ix = self.ui.interact(sid, Sense::CLICK, false);
        let (k_act, k_held) = key_activate(self.ui, &ix, true);
        let mo = self.ui.motion();
        let hv = self.ui.fade(sid, 0, ix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
        let press = self.ui.spring(sid, 1, if ix.pressed || k_held { 1.0 } else { 0.0 }, mo.snappy);
        let (ring_a, ring_gap) = self.ui.focus_anim(&ix);
        let bg = self.ui.color(Token::Hover).multiply_alpha(hv);
        let fg = self.ui.color(Token::TextSecondary).mix(self.ui.color(Token::Foreground), hv);
        let ring = self.ui.color(Token::FocusRing);
        let mut st = self.ui.text_style(TextRole::Caption);
        st.tabular = true;
        self.ui.begin(
            Style { dir: Dir::Row, height: Size::Fixed(22.0), pad: Sides::xy(7.0, 0.0), gap: 5.0, align: Align::Center, ..Default::default() },
            Some(sid),
            Sense::CLICK,
            CursorIcon::Pointer,
            false,
            Some(Box::new(move |cx, r| {
                let sr = Rect::from_center_size(r.center(), r.size() * (1.0 - 0.04 * press));
                if bg.a > 0.0 {
                    cx.painter.rrect_unsnapped(sr, Corners::all(4.0), [bg; 2], 0.0, Color::TRANSPARENT);
                }
                if ring_a > 0.0 {
                    cx.defer(sr, Box::new(move |cx, r| cx.focus_ring(r, 4.0, ring_a, ring_gap, ring)));
                }
            })),
        );
        if !icon.is_empty() {
            self.ui.icon_leaf(icon, 12.5, fg);
        }
        self.ui.text_leaf(label, st, fg, false, true, Style::default());
        self.ui.end();
        ix.clicked || k_act
    }

    /// Read-only tabular caption segment (e.g. `"Ln 142, Col 18"`, `"UTF-8"`).
    pub fn text(&mut self, label: &str) {
        let mut st = self.ui.text_style(TextRole::Caption);
        st.tabular = true;
        let c = self.ui.color(Token::TextMuted);
        self.ui.begin(Style { pad: Sides::xy(6.0, 0.0), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.ui.text_leaf(label, st, c, false, true, Style::default());
        self.ui.end();
    }

    pub fn spacer(&mut self) {
        self.ui.spacer();
    }
}

// =================================================================== Meter

/// Multi-segment quota / resource breakdown bar with an interactive legend.
/// Hovering a legend item highlights its segment and dims the others.
pub struct Meter<'u, 'a> {
    ui: &'u mut Ui,
    key: String,
    capacity: f32,
    unit: String,
    segments: &'a [(&'a str, f32)],
    label: Option<String>,
    shown: bool,
}

impl<'u, 'a> Meter<'u, 'a> {
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn unit(mut self, u: impl Into<String>) -> Self {
        self.unit = u.into();
        self
    }
    pub fn show(mut self) {
        self.shown = true;
        self.show_inner();
    }

    fn show_inner(&mut self) {
        let ui = &mut *self.ui;
        let id = ui.make_id(("meter", &self.key));
        let mo = ui.motion();
        let cap = self.capacity.max(f32::EPSILON);
        let used: f32 = self.segments.iter().map(|s| s.1.max(0.0)).sum();

        let series_toks = [Token::Series0, Token::Series1, Token::Series2, Token::Series3, Token::Series4, Token::Series5];
        let mut hovered_seg: Option<usize> = None;
        let mut leg_ids = Vec::with_capacity(self.segments.len());
        for i in 0..self.segments.len() {
            let lid = id.with(("leg", i));
            let lix = ui.interact(lid, Sense::HOVER, false);
            if lix.hovered {
                hovered_seg = Some(i);
            }
            leg_ids.push(lid);
        }

        let mut fracs = Vec::with_capacity(self.segments.len());
        let mut cols = Vec::with_capacity(self.segments.len());
        for (i, &(_, val)) in self.segments.iter().enumerate() {
            let target = (val.max(0.0) / cap).clamp(0.0, 1.0);
            let f = ui.spring(id, 10 + i as u32, target, mo.smooth).clamp(0.0, 1.0);
            let dim = hovered_seg.is_some_and(|h| h != i);
            let a = ui.fade(id, 30 + i as u32, !dim, mo.hover_in, mo.hover_out, Ease::Linear);
            fracs.push(f);
            cols.push(ui.color(series_toks[i % 6]).multiply_alpha(0.32 + 0.68 * a));
        }

        ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 10.0, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);

        if self.label.is_some() || !self.unit.is_empty() {
            ui.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            if let Some(l) = &self.label {
                let st = ui.text_style(TextRole::BodyStrong);
                let c = ui.color(Token::Foreground);
                ui.text_leaf(l.clone(), st, c, false, true, Style::default());
            }
            ui.spacer();
            let mut st_u = ui.text_style(TextRole::Caption);
            st_u.tabular = true;
            let cu = ui.color(Token::TextSecondary);
            let unit = &self.unit;
            let summary = if unit.is_empty() {
                format!("{:.0}% used", (used / cap * 100.0).min(100.0))
            } else {
                format!("{:.1} / {:.0} {unit}", used, self.capacity)
            };
            ui.text_leaf(summary, st_u, cu, false, false, Style::default());
            ui.end();
        }

        let track = ui.color(Token::Track).multiply_alpha(0.75);
        ui.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(200.0, 10.0),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(5.0), track, 0.0, Color::TRANSPARENT);
                cx.painter.push_clip(r);
                let mut x = r.min.x;
                let gap = 2.0;
                for (i, (&f, &col)) in fracs.iter().zip(cols.iter()).enumerate() {
                    let w = (r.width() * f - if i > 0 { gap } else { 0.0 }).max(0.0);
                    if w <= 0.5 {
                        continue;
                    }
                    let sx = x + if i > 0 { gap } else { 0.0 };
                    let sr = Rect::from_xywh(sx, r.min.y, w.min(r.max.x - sx), r.height());
                    let rad = if i == 0 { Corners([5.0, 2.0, 2.0, 5.0]) } else { Corners::all(2.0) };
                    cx.painter.rrect_unsnapped(sr, rad, [col; 2], 0.0, Color::TRANSPARENT);
                    x = sx + w;
                }
                cx.painter.pop_clip();
            })),
        );

        // Legend items.
        ui.begin(Style { dir: Dir::Row, wrap: true, width: Size::FILL, gap: 14.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for (i, &(name, val)) in self.segments.iter().enumerate() {
            let dot_c = ui.color(series_toks[i % 6]);
            let hot = hovered_seg == Some(i);
            let fg = if hot { ui.color(Token::Foreground) } else { ui.color(Token::TextSecondary) };
            let mut st_v = ui.text_style(TextRole::Caption);
            st_v.tabular = true;
            let vc = ui.color(Token::TextMuted);
            let unit = &self.unit;
            let val_txt = if unit.is_empty() { format!("{val:.0}") } else { format!("{val:.1} {unit}") };

            ui.begin(Style { dir: Dir::Row, gap: 6.0, align: Align::Center, ..Default::default() }, Some(leg_ids[i]), Sense::HOVER, CursorIcon::Default, false, None);
            ui.leaf(Style::default(), vec2(8.0, 8.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| {
                cx.painter.circle(r.center(), 4.0, dot_c, 0.0, Color::TRANSPARENT);
            })));
            let st_n = ui.text_style(TextRole::Caption);
            ui.text_leaf(name, st_n, fg, false, false, Style::default());
            ui.text_leaf(val_txt, st_v, vc, false, false, Style::default());
            ui.end();
        }
        ui.end();

        ui.end();
    }
}

impl Drop for Meter<'_, '_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

// =========================================================== Ui Entrypoints

impl Ui {
    /// Action button joined with a trailing menu chevron.
    pub fn split_button(&mut self, label: impl Into<String>) -> SplitButton<'_> {
        SplitButton { ui: self, label: label.into(), variant: Variant::Primary, icon: None, size: ControlSize::Md, disabled: false }
    }

    /// Destructive (or primary) button that must be held for `duration` seconds to fire.
    pub fn hold_button(&mut self, label: impl Into<String>, duration: f32) -> HoldButton<'_> {
        HoldButton { ui: self, label: label.into(), duration, variant: Variant::Danger, icon: None, size: ControlSize::Md, disabled: false, shown: false }
    }

    /// Button that copies `text_to_copy` to the clipboard and morphs its icon
    /// from `"copy"` to `"check"` for 1.4 seconds.
    pub fn copy_button(&mut self, label: &str, text_to_copy: &str) -> Response {
        #[derive(Default)]
        struct CopyMem(f64);
        let id = self.make_id(("copy_btn", label));
        let now = self.time();
        let until = self.mem::<CopyMem>(id).0;
        let done = now < until;
        if done {
            self.request_repaint_after(until - now);
        }
        let shown_label = if done && !label.is_empty() { "Copied" } else { label };
        let icon = if done { "check" } else { "copy" };
        let r = if label.is_empty() {
            self.scope(id, |ui| ui.icon_button(icon).tooltip(if done { "Copied" } else { "Copy" }).show())
        } else {
            self.scope(id, |ui| ui.button(shown_label).icon(icon).show())
        };
        if r.clicked {
            self.set_copied_text(text_to_copy.to_owned());
            self.mem::<CopyMem>(id).0 = now + 1.4;
            self.request_repaint_after(1.4);
        }
        r
    }

    /// File upload dropzone with an animated icon tile and optional file list.
    pub fn dropzone(&mut self, key: impl Into<String>, title: impl Into<String>) -> Dropzone<'_> {
        Dropzone { ui: self, key: key.into(), title: title.into(), hint: None, files: None, disabled: false, shown: false }
    }

    /// Raised formatting or canvas toolbar strip.
    pub fn toolbar<R>(&mut self, key: impl Into<String>, f: impl FnOnce(&mut ToolbarScope<'_>) -> R) -> R {
        let k = key.into();
        let id = self.make_id(("toolbar", &k));
        let fill = self.color(Token::SurfaceRaised);
        let bd = self.color(if self.theme.darkness() > 0.5 { Token::BorderStrong } else { Token::Border });
        let shadows = Elevation::Level1.shadows(self.theme.darkness(), self.color(Token::ShadowInk));
        let rad = (self.look().control.radius[2]).max(8.0);
        self.begin(
            Style { dir: Dir::Row, pad: Sides::all(4.0), gap: 2.0, align: Align::Center, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                cx.painter.shadows(r, Corners::all(rad), &shadows);
                cx.painter.rrect(r, Corners::all(rad), fill, 1.0, bd);
            })),
        );
        let mut scope = ToolbarScope { ui: self, id, count: 0 };
        let out = f(&mut scope);
        self.end();
        out
    }

    /// Compact application status bar (28px tall).
    pub fn status_bar<R>(&mut self, f: impl FnOnce(&mut StatusBarScope<'_>) -> R) -> R {
        let id = self.make_id("status_bar");
        let fill = self.color(Token::Surface);
        let bd = self.color(Token::BorderSubtle);
        let rad = self.look().control.radius[1];
        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(28.0), pad: Sides::xy(6.0, 0.0), gap: 4.0, align: Align::Center, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(rad), fill, 1.0, bd))),
        );
        let mut scope = StatusBarScope { ui: self, id, count: 0 };
        let out = f(&mut scope);
        self.end();
        out
    }

    /// Segmented quota / storage breakdown bar (`segments` of `(label, amount)` out of `capacity`).
    pub fn meter<'u, 'a>(&'u mut self, key: impl Into<String>, capacity: f32, segments: &'a [(&'a str, f32)]) -> Meter<'u, 'a> {
        Meter { ui: self, key: key.into(), capacity, unit: String::new(), segments, label: None, shown: false }
    }
}
