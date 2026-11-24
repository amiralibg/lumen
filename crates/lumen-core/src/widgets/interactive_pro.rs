//! Editable data grid, facet filter bar, AI citations, text effects, rotary
//! knobs, before/after split comparisons, countdown timers and checklists:
//! [`GridColumn`], [`FilterFacet`], [`CitationSource`], [`RotaryKnob`],
//! [`CompareSplit`], and [`ChecklistStep`].

use std::f32::consts::PI;
use std::ops::RangeInclusive;

use super::field::EditOpts;
use super::show_on_drop;
use super::toast::Tone;
use crate::anim::Ease;
use crate::color::Color;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};

// ================================================================ DataGrid

/// Column definition in [`Ui::data_grid`].
#[derive(Clone, Debug)]
pub struct GridColumn {
    pub title: String,
    pub mono: bool,
}

impl GridColumn {
    pub fn new(title: impl Into<String>) -> Self {
        GridColumn { title: title.into(), mono: false }
    }
    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }
}

#[derive(Default, Clone)]
struct GridMem {
    sel: (usize, usize),
    editing: bool,
    draft: String,
    backup: String,
    just_opened: bool,
}

// =============================================================== FilterBar

/// Active facet chip (`key`, `op`, `value`) in [`Ui::filter_bar`].
#[derive(Clone, Debug)]
pub struct FilterFacet {
    pub field: String,
    pub op: String,
    pub value: String,
}

impl FilterFacet {
    pub fn new(field: impl Into<String>, op: impl Into<String>, value: impl Into<String>) -> Self {
        FilterFacet { field: field.into(), op: op.into(), value: value.into() }
    }
}

// ============================================================= Citations

/// Grounding source card in [`Ui::cited_answer`].
#[derive(Clone, Debug)]
pub struct CitationSource {
    pub index: usize,
    pub title: String,
    pub domain: String,
    pub snippet: String,
}

impl CitationSource {
    pub fn new(index: usize, title: impl Into<String>, domain: impl Into<String>, snippet: impl Into<String>) -> Self {
        CitationSource {
            index,
            title: title.into(),
            domain: domain.into(),
            snippet: snippet.into(),
        }
    }
}

#[derive(Default, Clone, Copy)]
struct CiteMem {
    active_cite: Option<usize>,
}

// ============================================================== RotaryKnob

/// Precision rotary dial adjusted by vertical pointer drag or arrow keys,
/// rendered with a 270° `arc_band`, radial ticks, press squash and tabular readout.
pub struct RotaryKnob<'u> {
    ui: &'u mut Ui,
    label: String,
    value: &'u mut f32,
    range: RangeInclusive<f32>,
    unit: String,
    diameter: f32,
    shown: bool,
}

impl<'u> RotaryKnob<'u> {
    pub fn unit(mut self, u: impl Into<String>) -> Self {
        self.unit = u.into();
        self
    }
    pub fn diameter(mut self, px: f32) -> Self {
        self.diameter = px.clamp(56.0, 140.0);
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("rotary_knob", &self.label));
        let dial_id = id.with("dial");
        let ix = ui.interact(dial_id, Sense::DRAG, false);
        let mo = ui.motion();
        let lo = *self.range.start();
        let hi = (*self.range.end()).max(lo + 1e-5);
        let span = hi - lo;
        let mut changed = false;

        #[derive(Default, Clone, Copy)]
        struct KnobDrag {
            start_y: f32,
            start_val: f32,
        }
        if ix.press_started {
            if let Some(p) = ix.pointer {
                *ui.mem::<KnobDrag>(id) = KnobDrag { start_y: p.y, start_val: *self.value };
            }
        }
        if ix.held {
            if let Some(p) = ix.pointer {
                let dm = *ui.mem::<KnobDrag>(id);
                let dy = dm.start_y - p.y;
                let next = (dm.start_val + (dy / 130.0) * span).clamp(lo, hi);
                if (next - *self.value).abs() > 1e-4 {
                    *self.value = next;
                    changed = true;
                }
            }
        }
        if ix.focused {
            let step = span * 0.05;
            if ui.key_pressed_for(dial_id, Key::ArrowUp) || ui.key_pressed_for(dial_id, Key::ArrowRight) {
                *self.value = (*self.value + step).clamp(lo, hi);
                changed = true;
            } else if ui.key_pressed_for(dial_id, Key::ArrowDown) || ui.key_pressed_for(dial_id, Key::ArrowLeft) {
                *self.value = (*self.value - step).clamp(lo, hi);
                changed = true;
            } else if ui.key_pressed_for(dial_id, Key::Home) {
                *self.value = lo;
                changed = true;
            } else if ui.key_pressed_for(dial_id, Key::End) {
                *self.value = hi;
                changed = true;
            }
        }

        let norm_target = ((*self.value - lo) / span).clamp(0.0, 1.0);
        let norm = ui.spring(id, 0, norm_target, mo.snappy).clamp(0.0, 1.0);
        let press = ui.spring(id, 1, if ix.held { 1.0 } else { 0.0 }, mo.snappy);
        let hov = ui.fade(id, 2, ix.hovered, mo.hover_in, mo.hover_out, Ease::Out);
        let f_ring = ui.fade(id, 3, ix.focus_visible, mo.hover_in, mo.hover_out, Ease::Out);

        let track_c = ui.color(Token::Track);
        let fill_c = ui.color(Token::Accent);
        let cap_bg = ui.color(Token::SurfaceRaised).mix(ui.color(Token::Hover), hov * 0.5);
        let cap_bd = ui.color(Token::BorderStrong).mix(fill_c, hov * 0.5);
        let tick_c = ui.color(Token::BorderStrong);
        let needle_c = ui.color(Token::AccentStrong);
        let ring_c = ui.color(Token::FocusRing).multiply_alpha(f_ring);
        let ink = ui.color(Token::ShadowInk);

        let diam = self.diameter;
        ui.begin(Style { dir: Dir::Column, gap: 6.0, align: Align::Center, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);

        ui.leaf(
            Style::default(),
            vec2(diam + 18.0, diam + 14.0),
            Some(dial_id),
            Sense::DRAG,
            if ix.held { CursorIcon::Grabbing } else { CursorIcon::Grab },
            Some(Box::new(move |cx, r| {
                let c = r.center() + vec2(0.0, 2.0);
                let squash = 1.0 - 0.04 * press;
                let r_outer = (diam * 0.5) * squash;
                let r_inner = r_outer - 5.5;

                // 270° sweep centered at 12 o'clock: -135° (-3π/4) to +135° (+3π/4).
                let a_start = -0.75 * PI;
                let a_total = 1.5 * PI;
                let a_cur = a_start + a_total * norm;

                cx.painter.arc_band(c, r_inner, r_outer, a_start, a_start + a_total, track_c);
                if a_cur > a_start + 0.01 {
                    cx.painter.arc_band(c, r_inner, r_outer, a_start, a_cur, fill_c);
                }

                // 9 subtle outer radial ticks.
                for ti in 0..=8 {
                    let tf = ti as f32 / 8.0;
                    let ang = a_start + a_total * tf;
                    let dir = vec2(ang.sin(), -ang.cos());
                    let tc = if tf <= norm + 0.01 { fill_c } else { tick_c };
                    cx.painter.circle(c + dir * (r_outer + 5.0), 1.3, tc, 0.0, Color::TRANSPARENT);
                }

                // Elevated inner dial cap + directional pointer line.
                let cap_r = (r_inner - 5.0).max(12.0);
                cx.painter.shadow(
                    Rect::from_center_size(c, vec2(cap_r * 2.0, cap_r * 2.0)),
                    Corners::all(cap_r),
                    &crate::theme::Shadow { x: 0.0, y: 3.0, blur: 8.0, spread: 0.0, color: ink.with_alpha(0.18) },
                );
                cx.painter.circle(c, cap_r, cap_bg, 1.2, cap_bd);
                if f_ring > 0.005 {
                    cx.painter.circle(c, r_outer + 8.0, Color::TRANSPARENT, 2.0, ring_c);
                }

                let p_dir = vec2(a_cur.sin(), -a_cur.cos());
                cx.painter.polyline(&[c + p_dir * (cap_r * 0.32), c + p_dir * (cap_r * 0.78)], 2.4, needle_c);
            })),
        );

        let mut st_v = ui.text_style(TextRole::Mono);
        st_v.tabular = true;
        st_v.weight = 600;
        let readout = if self.unit.is_empty() {
            format!("{:.0}", *self.value)
        } else {
            format!("{:.0}{}", *self.value, self.unit)
        };
        ui.text_leaf(readout, st_v, ui.color(Token::Foreground), false, false, Style::default());
        ui.text_leaf(self.label.clone(), ui.text_style(TextRole::Caption), ui.color(Token::TextMuted), false, false, Style::default());
        ui.end();

        let rect = ui.rect_of(id).unwrap_or(ix.rect);
        Response { id, rect, hovered: ix.hovered, clicked: ix.clicked, changed, focused: ix.focused }
    }
}

show_on_drop!(RotaryKnob);

// ============================================================ CompareSplit

/// Interactive before/after overlay split slider with draggable vertical
/// divider and `< >` handle pill.
pub struct CompareSplit<'u> {
    ui: &'u mut Ui,
    key: String,
    ratio: &'u mut f32,
    left_label: String,
    right_label: String,
    height: f32,
    shown: bool,
}

impl<'u> CompareSplit<'u> {
    pub fn height(mut self, px: f32) -> Self {
        self.height = px.max(150.0);
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("compare_split", &self.key));
        *self.ratio = self.ratio.clamp(0.08, 0.92);
        let ix = ui.interact(id, Sense::DRAG, false);
        let mo = ui.motion();
        let mut changed = false;

        if (ix.clicked || ix.held) && ix.rect.width() > 30.0 {
            if let Some(p) = ix.pointer {
                let next = ((p.x - ix.rect.min.x) / ix.rect.width()).clamp(0.08, 0.92);
                if (next - *self.ratio).abs() > 1e-4 {
                    *self.ratio = next;
                    changed = true;
                }
            }
        }
        if ix.focused {
            if ui.key_pressed_for(id, Key::ArrowRight) {
                *self.ratio = (*self.ratio + 0.05).clamp(0.08, 0.92);
                changed = true;
            } else if ui.key_pressed_for(id, Key::ArrowLeft) {
                *self.ratio = (*self.ratio - 0.05).clamp(0.08, 0.92);
                changed = true;
            }
        }

        let r_anim = ui.spring(id, 0, *self.ratio, mo.snappy).clamp(0.05, 0.95);
        let press = ui.spring(id, 1, if ix.held { 1.0 } else { 0.0 }, mo.snappy);
        let f_ring = ui.fade(id, 2, ix.focus_visible, mo.hover_in, mo.hover_out, Ease::Out);

        let r_card = ui.look().card.radius;
        let bd_c = ui.color(Token::BorderSubtle);
        let left_bg = ui.color(Token::SurfaceMuted);
        let right_bg = ui.color(Token::Surface);
        let wire_c = ui.color(Token::TextMuted);
        let acc_c = ui.color(Token::Accent);
        let c2 = ui.color(Token::Series3);
        let fg_c = ui.color(Token::Foreground);
        let pill_bg = ui.color(Token::SurfaceRaised).multiply_alpha(0.88);
        let ring_c = ui.color(Token::FocusRing).multiply_alpha(f_ring);
        let ink = ui.color(Token::ShadowInk);

        let st_lbl = ui.text_style(TextRole::Caption);
        let l_lbl = self.left_label.clone();
        let r_lbl = self.right_label.clone();

        ui.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(340.0, self.height),
            Some(id),
            Sense::DRAG,
            CursorIcon::ResizeHorizontal,
            Some(Box::new(move |cx, r| {
                let split_x = r.min.x + r.width() * r_anim;
                let center = r.center();

                // Right side (full frame base: Shaded SDF scene).
                cx.painter.rrect(r, Corners::all(r_card), right_bg, 0.0, Color::TRANSPARENT);
                cx.painter.push_clip(r.expand(-1.0));
                cx.painter.shadow(
                    Rect::from_center_size(center + vec2(-32.0, -6.0), vec2(116.0, 116.0)),
                    Corners::all(58.0),
                    &crate::theme::Shadow { x: 0.0, y: 0.0, blur: 92.0, spread: 0.0, color: acc_c.with_alpha(0.40) },
                );
                cx.painter.shadow(
                    Rect::from_center_size(center + vec2(44.0, 10.0), vec2(104.0, 104.0)),
                    Corners::all(52.0),
                    &crate::theme::Shadow { x: 0.0, y: 0.0, blur: 88.0, spread: 0.0, color: c2.with_alpha(0.36) },
                );
                let card_demo = Rect::from_center_size(center, vec2(210.0, 88.0));
                cx.painter.shadow(
                    card_demo,
                    Corners::all(14.0),
                    &crate::theme::Shadow { x: 0.0, y: 8.0, blur: 22.0, spread: 0.0, color: ink.with_alpha(0.24) },
                );
                cx.painter.rrect(card_demo, Corners::all(14.0), pill_bg, 1.0, acc_c.with_alpha(0.55));
                cx.painter.circle(vec2(card_demo.min.x + 32.0, card_demo.center().y), 16.0, acc_c, 0.0, Color::TRANSPARENT);
                cx.painter.rrect(Rect::from_xywh(card_demo.min.x + 60.0, card_demo.center().y - 14.0, 120.0, 10.0), Corners::all(5.0), fg_c.with_alpha(0.85), 0.0, Color::TRANSPARENT);
                cx.painter.rrect(Rect::from_xywh(card_demo.min.x + 60.0, card_demo.center().y + 4.0, 84.0, 8.0), Corners::all(4.0), wire_c.with_alpha(0.55), 0.0, Color::TRANSPARENT);

                // Left side clipped to `split_x`: crisp CAD / wireframe view of the exact same geometry!
                cx.painter.push_clip(Rect::from_xywh(r.min.x, r.min.y, (split_x - r.min.x).max(0.0), r.height()));
                cx.painter.rrect(r, Corners::all(r_card), left_bg, 0.0, Color::TRANSPARENT);
                let mut gx = r.min.x + 22.0;
                while gx < split_x {
                    cx.painter.rect(Rect::from_xywh(gx, r.min.y + 8.0, 1.0, r.height() - 16.0), bd_c.with_alpha(0.5));
                    gx += 24.0;
                }
                let mut gy = r.min.y + 22.0;
                while gy < r.max.y - 8.0 {
                    cx.painter.rect(Rect::from_xywh(r.min.x + 8.0, gy, (split_x - r.min.x - 8.0).max(0.0), 1.0), bd_c.with_alpha(0.5));
                    gy += 24.0;
                }
                cx.painter.rrect(card_demo, Corners::all(14.0), Color::TRANSPARENT, 1.2, wire_c);
                cx.painter.circle(vec2(card_demo.min.x + 32.0, card_demo.center().y), 16.0, Color::TRANSPARENT, 1.2, acc_c);
                cx.painter.rrect(Rect::from_xywh(card_demo.min.x + 60.0, card_demo.center().y - 14.0, 120.0, 10.0), Corners::all(5.0), Color::TRANSPARENT, 1.0, wire_c);
                cx.painter.rrect(Rect::from_xywh(card_demo.min.x + 60.0, card_demo.center().y + 4.0, 84.0, 8.0), Corners::all(4.0), Color::TRANSPARENT, 1.0, wire_c);
                cx.painter.pop_clip();

                // Corner badges for Left & Right labels.
                let gl = cx.galley(&l_lbl, &st_lbl);
                let lp = Rect::from_xywh(r.min.x + 12.0, r.min.y + 12.0, gl.width + 14.0, 22.0);
                cx.painter.rrect(lp, Corners::all(11.0), pill_bg, 1.0, bd_c);
                cx.painter.text(cx.text, &gl, vec2(lp.min.x + 7.0, lp.center().y - gl.height * 0.5), fg_c);

                let gr = cx.galley(&r_lbl, &st_lbl);
                let rp = Rect::from_xywh(r.max.x - 12.0 - gr.width - 14.0, r.min.y + 12.0, gr.width + 14.0, 22.0);
                cx.painter.rrect(rp, Corners::all(11.0), pill_bg, 1.0, bd_c);
                cx.painter.text(cx.text, &gr, vec2(rp.min.x + 7.0, rp.center().y - gr.height * 0.5), fg_c);

                // Vertical divider line + circular `< >` handle.
                cx.painter.rect(Rect::from_xywh(split_x - 1.0, r.min.y, 2.0, r.height()), Color::WHITE.with_alpha(0.9));
                let hr = 16.0 * (1.0 - 0.06 * press);
                let hc = vec2(split_x, r.center().y);
                cx.painter.shadow(
                    Rect::from_center_size(hc, vec2(hr * 2.0, hr * 2.0)),
                    Corners::all(hr),
                    &crate::theme::Shadow { x: 0.0, y: 3.0, blur: 10.0, spread: 0.0, color: ink.with_alpha(0.28) },
                );
                cx.painter.circle(hc, hr, pill_bg, 1.5, acc_c);
                cx.icon("move-horizontal", hc, 15.0, fg_c);

                cx.painter.pop_clip();
                cx.painter.ring(r, Corners::all(r_card), 1.0, bd_c);
                if f_ring > 0.005 {
                    cx.painter.ring(r.expand(2.0), Corners::all(r_card + 2.0), 2.0, ring_c);
                }
            })),
        );
        Response { id, rect: ix.rect, hovered: ix.hovered, clicked: ix.clicked, changed, focused: ix.focused }
    }
}

show_on_drop!(CompareSplit);

// =========================================================== ChecklistStep

/// Step item in [`Ui::checklist_card`].
#[derive(Clone, Debug)]
pub struct ChecklistStep {
    pub title: String,
    pub detail: String,
    pub tag: String,
    pub done: bool,
}

impl ChecklistStep {
    pub fn new(title: impl Into<String>, detail: impl Into<String>, tag: impl Into<String>, done: bool) -> Self {
        ChecklistStep { title: title.into(), detail: detail.into(), tag: tag.into(), done }
    }
}

impl Ui {
    /// Spreadsheet-style editable cell grid with row numbers, keyboard cell
    /// navigation (`Arrow*`), formula/cell bar, and `Enter` inline editing.
    pub fn data_grid(&mut self, key: impl std::hash::Hash, columns: &[GridColumn], rows: &mut [Vec<String>]) -> bool {
        let id = self.make_id(("data_grid", key));
        let edit_id = id.with("cell_ed");
        let grid_focus_id = id.with("grid_nav");

        let n_rows = rows.len();
        let n_cols = columns.len();
        if n_rows == 0 || n_cols == 0 {
            return false;
        }

        let mut mem = self.mem::<GridMem>(id).clone();
        mem.sel.0 = mem.sel.0.min(n_rows - 1);
        mem.sel.1 = mem.sel.1.min(n_cols - 1);

        let nav_ix = self.interact(grid_focus_id, Sense::CLICK, false);
        let mut changed = false;

        if !mem.editing && nav_ix.focused {
            if self.key_pressed_for(grid_focus_id, Key::ArrowDown) {
                mem.sel.0 = (mem.sel.0 + 1).min(n_rows - 1);
            } else if self.key_pressed_for(grid_focus_id, Key::ArrowUp) {
                mem.sel.0 = mem.sel.0.saturating_sub(1);
            } else if self.key_pressed_for(grid_focus_id, Key::ArrowRight) {
                mem.sel.1 = (mem.sel.1 + 1).min(n_cols - 1);
            } else if self.key_pressed_for(grid_focus_id, Key::ArrowLeft) {
                mem.sel.1 = mem.sel.1.saturating_sub(1);
            } else if self.key_pressed_for(grid_focus_id, Key::Enter) {
                let cur_v = rows[mem.sel.0].get(mem.sel.1).cloned().unwrap_or_default();
                mem.editing = true;
                mem.draft = cur_v.clone();
                mem.backup = cur_v;
                mem.just_opened = true;
            }
        }

        let r_card = self.look().card.radius;
        let top_rad = (r_card - 1.0).max(0.0);
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let hdr_bg = self.color(Token::SurfaceMuted).multiply_alpha(0.65);

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, ..Default::default() },
            Some(grid_focus_id),
            Sense::CLICK,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        // Top formula / cell coordinate bar (`B2 · value`).
        let col_letter = (b'A' + (mem.sel.1 as u8).min(25)) as char;
        let cell_ref = format!("{col_letter}{}", mem.sel.0 + 1);
        let active_val = rows[mem.sel.0].get(mem.sel.1).cloned().unwrap_or_default();
        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(10.0, 6.0), gap: 8.0, align: Align::Center, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                let inner = Rect::from_xywh(r.min.x + 1.0, r.min.y + 1.0, (r.width() - 2.0).max(0.0), (r.height() - 1.0).max(0.0));
                cx.painter.rrect(inner, Corners([top_rad, top_rad, 0.0, 0.0]), hdr_bg, 0.0, Color::TRANSPARENT);
                cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.max.y - 1.0, (r.width() - 2.0).max(0.0), 1.0), bd_c);
            })),
        );
        self.badge(&cell_ref, Tone::Accent);
        self.text_leaf("fx", self.text_style(TextRole::Mono), self.color(Token::TextMuted), false, false, Style::default());
        self.text_leaf(active_val, self.text_style(TextRole::Mono), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
        self.text_leaf(if mem.editing { "Enter to save" } else { "Click or Enter to edit" }, self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, false, Style::default());
        self.end();

        // Column header row.
        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.min.y, (r.width() - 2.0).max(0.0), r.height()), hdr_bg.multiply_alpha(0.55));
                cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.max.y - 1.0, (r.width() - 2.0).max(0.0), 1.0), bd_c);
            })),
        );
        self.begin(Style { width: Size::Fixed(34.0), pad: Sides::xy(6.0, 6.0), justify: Justify::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf("#", self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, false, Style::default());
        self.end();
        for (ci, col) in columns.iter().enumerate() {
            self.begin(
                Style { width: Size::FILL, pad: Sides::xy(10.0, 6.0), ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| cx.painter.rect(Rect::from_xywh(r.min.x, r.min.y, 1.0, r.height()), bd_c))),
            );
            let mut st_c = self.text_style(TextRole::Caption);
            st_c.weight = 600;
            let col_c = if ci == mem.sel.1 { self.color(Token::AccentStrong) } else { self.color(Token::TextSecondary) };
            self.text_leaf(col.title.clone(), st_c, col_c, false, true, Style { width: Size::FILL, ..Default::default() });
            self.end();
        }
        self.end();

        // Data rows.
        for ri in 0..n_rows {
            let is_last = ri + 1 == n_rows;
            self.begin(
                Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                if !is_last {
                    Some(Box::new(move |cx, r| cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.max.y - 1.0, (r.width() - 2.0).max(0.0), 1.0), bd_c)))
                } else {
                    None
                },
            );
            // Row index gutter.
            let r_sel = ri == mem.sel.0;
            self.begin(Style { width: Size::Fixed(34.0), pad: Sides::xy(6.0, 7.0), justify: Justify::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let mut st_n = self.text_style(TextRole::Mono);
            st_n.size = (st_n.size - 1.0).max(11.0);
            self.text_leaf(format!("{}", ri + 1), st_n, self.color(if r_sel { Token::AccentStrong } else { Token::TextMuted }), false, false, Style::default());
            self.end();

            for ci in 0..n_cols {
                let cid = id.with(("c", ri, ci));
                let c_ix = self.interact(cid, Sense { click: true, drag: false, focusable: false, scroll: false }, false);
                let is_sel = mem.sel == (ri, ci);
                let sel_bd = self.color(Token::Accent);
                let sel_bg = self.color(Token::AccentSubtle).multiply_alpha(0.32);

                self.begin(
                    Style { width: Size::FILL, pad: Sides::xy(10.0, 6.0), align: Align::Center, ..Default::default() },
                    Some(cid),
                    Sense { click: true, drag: false, focusable: false, scroll: false },
                    CursorIcon::Pointer,
                    false,
                    Some(Box::new(move |cx, r| {
                        cx.painter.rect(Rect::from_xywh(r.min.x, r.min.y, 1.0, r.height()), bd_c);
                        if is_sel {
                            cx.painter.rrect(r.expand(-1.5), Corners::all(4.0), sel_bg, 1.5, sel_bd);
                        }
                    })),
                );

                if is_sel && mem.editing {
                    if mem.just_opened {
                        mem.just_opened = false;
                        self.request_focus(edit_id);
                        self.inline_edit_select_all(edit_id, &mem.draft);
                    }
                    let mut opts = EditOpts::new(self);
                    if columns[ci].mono {
                        opts.style = self.text_style(TextRole::Mono);
                    }
                    let ed = self.inline_edit(edit_id, &mut mem.draft, opts);
                    if ed.key(Key::Enter) || (self.focused() != Some(edit_id) && !c_ix.clicked) {
                        if let Some(cell) = rows[ri].get_mut(ci) {
                            *cell = mem.draft.clone();
                            changed = true;
                        }
                        mem.editing = false;
                        self.request_focus(grid_focus_id);
                    } else if ed.key(Key::Escape) {
                        mem.draft = mem.backup.clone();
                        mem.editing = false;
                        self.request_focus(grid_focus_id);
                    }
                } else {
                    let val = rows[ri].get(ci).cloned().unwrap_or_default();
                    let mut st = self.text_style(if columns[ci].mono { TextRole::Mono } else { TextRole::Body });
                    st.tabular = columns[ci].mono;
                    self.text_leaf(val, st, self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
                }
                self.end();

                if c_ix.clicked {
                    if mem.sel == (ri, ci) && !mem.editing {
                        let cur_v = rows[ri].get(ci).cloned().unwrap_or_default();
                        mem.editing = true;
                        mem.draft = cur_v.clone();
                        mem.backup = cur_v;
                        mem.just_opened = true;
                    } else {
                        if mem.editing {
                            if let Some(prev_cell) = rows.get_mut(mem.sel.0).and_then(|r| r.get_mut(mem.sel.1)) {
                                *prev_cell = mem.draft.clone();
                                changed = true;
                            }
                            mem.editing = false;
                        }
                        mem.sel = (ri, ci);
                        self.request_focus(grid_focus_id);
                    }
                }
            }
            self.end();
        }
        self.end();

        *self.mem::<GridMem>(id) = mem;
        changed
    }

    /// Structured Linear/GitHub-style facet filter bar (`Field | op | Value | ×`)
    /// with `+ Add filter` menu and `"Clear"` action.
    pub fn filter_bar(
        &mut self,
        key: impl std::hash::Hash,
        facets: &mut Vec<FilterFacet>,
        presets: &[(&str, &str, &str)],
    ) -> bool {
        let id = self.make_id(("filter_bar", key));
        let mut changed = false;

        let r_bar = (self.look().card.radius - 2.0).max(8.0);
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);

        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(10.0, 8.0), gap: 6.0, wrap: true, align: Align::Center, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_bar), bg_c, 1.0, bd_c))),
        );
        self.icon_leaf("list-filter", 15.0, self.color(Token::TextMuted));

        let mut remove_idx: Option<usize> = None;
        for (fi, f) in facets.iter().enumerate() {
            let p_bg = self.color(Token::SurfaceMuted);
            let p_bd = self.color(Token::Border);
            self.begin(
                Style { dir: Dir::Row, pad: Sides { left: 8.0, right: 4.0, top: 3.0, bottom: 3.0 }, gap: 5.0, align: Align::Center, ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(7.0), p_bg, 1.0, p_bd))),
            );
            let mut st_f = self.text_style(TextRole::Caption);
            st_f.weight = 600;
            self.text_leaf(f.field.clone(), st_f, self.color(Token::TextSecondary), false, false, Style::default());
            self.text_leaf(f.op.clone(), self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, false, Style::default());
            self.text_leaf(f.value.clone(), st_f, self.color(Token::AccentStrong), false, false, Style::default());

            let xid = id.with(("rm_f", fi));
            let xix = self.interact(xid, Sense::CLICK, false);
            let x_fg = self.color(if xix.hovered { Token::Foreground } else { Token::TextMuted });
            self.leaf(
                Style::default(),
                vec2(16.0, 16.0),
                Some(xid),
                Sense::CLICK,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| cx.icon("x", r.center(), 11.0, x_fg))),
            );
            if xix.clicked {
                remove_idx = Some(fi);
            }
            self.end();
        }
        if let Some(ri) = remove_idx {
            facets.remove(ri);
            changed = true;
        }

        // `+ Filter` popover button.
        let add_btn = self.scope("add_f_btn", |ui| ui.button("Add filter").ghost().small().icon("plus").show());
        let mut added_preset: Option<(String, String, String)> = None;
        self.popover(&add_btn).pad(4.0).show(|ui| {
            for &(fld, op, val) in presets {
                let lbl = format!("{fld} {op} {val}");
                if ui.menu_item("plus", &lbl).clicked() {
                    added_preset = Some((fld.to_owned(), op.to_owned(), val.to_owned()));
                }
            }
        });
        if let Some((fld, op, val)) = added_preset {
            if !facets.iter().any(|x| x.field == fld && x.value == val) {
                facets.push(FilterFacet::new(fld, op, val));
                changed = true;
            }
        }

        if !facets.is_empty() {
            self.begin(Style { width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.end();
            if self.scope("clr_f", |ui| ui.button("Clear").ghost().small().clicked()) {
                facets.clear();
                changed = true;
            }
        }
        self.end();
        changed
    }

    /// Grounded AI answer card with interactive numbered inline `[1]`, `[2]`
    /// citation pills and source cards below that highlight together on hover
    /// or click.
    pub fn cited_answer(
        &mut self,
        key: impl std::hash::Hash,
        segments: &[(&str, Option<usize>)],
        sources: &[CitationSource],
    ) -> Response {
        let id = self.make_id(("cited_answer", key));
        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);

        let mut active_cite = self.mem::<CiteMem>(id).active_cite;
        let mut next_hover_cite: Option<usize> = None;

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(16.0), gap: 14.0, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        // Header badge.
        self.begin(Style { dir: Dir::Row, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.icon_leaf("sparkles", 14.0, self.color(Token::AccentStrong));
        let mut st_h = self.text_style(TextRole::Caption);
        st_h.weight = 600;
        self.text_leaf("GROUNDED SYNTHESIS", st_h, self.color(Token::AccentStrong), false, false, Style::default());
        self.end();

        // Wrapping inline words + citation pills!
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 4.0, wrap: true, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let st_body = self.text_style(TextRole::Body);
        let fg_body = self.color(Token::Foreground);
        for (si, &(txt, cite_num)) in segments.iter().enumerate() {
            for w in txt.split_whitespace() {
                self.text_leaf(w.to_owned(), st_body, fg_body, false, false, Style::default());
            }
            if let Some(num) = cite_num {
                let pid = id.with(("inline_c", si, num));
                let p_ix = self.interact(pid, Sense::CLICK, false);
                if p_ix.hovered {
                    next_hover_cite = Some(num);
                }
                if p_ix.clicked {
                    active_cite = if active_cite == Some(num) { None } else { Some(num) };
                }
                let is_hi = active_cite == Some(num) || p_ix.hovered;
                let p_bg = if is_hi { self.color(Token::Accent) } else { self.color(Token::AccentSubtle) };
                let p_fg = if is_hi { self.color(Token::AccentForeground) } else { self.color(Token::AccentStrong) };
                self.begin(
                    Style { pad: Sides::xy(6.0, 1.0), ..Default::default() },
                    Some(pid),
                    Sense::CLICK,
                    CursorIcon::Pointer,
                    false,
                    Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(99.0), p_bg, 0.0, Color::TRANSPARENT))),
                );
                let mut st_n = self.text_style(TextRole::Mono);
                st_n.size = 11.0;
                st_n.weight = 600;
                self.text_leaf(format!("{num}"), st_n, p_fg, false, false, Style::default());
                self.end();
            }
        }
        self.end();

        // Source cards row!
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, align: Align::Stretch, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for src in sources {
            let sid = id.with(("src_card", src.index));
            let s_ix = self.interact(sid, Sense::CLICK, false);
            if s_ix.hovered {
                next_hover_cite = Some(src.index);
            }
            if s_ix.clicked {
                active_cite = if active_cite == Some(src.index) { None } else { Some(src.index) };
            }
            let is_hi = active_cite == Some(src.index) || next_hover_cite == Some(src.index);
            let c_bg = if is_hi { self.color(Token::AccentSubtle).multiply_alpha(0.65) } else { self.color(Token::SurfaceMuted).multiply_alpha(0.55) };
            let c_bd = if is_hi { self.color(Token::Accent) } else { self.color(Token::BorderSubtle) };

            self.begin(
                Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(10.0), gap: 4.0, ..Default::default() },
                Some(sid),
                Sense::CLICK,
                CursorIcon::Pointer,
                false,
                Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(9.0), c_bg, 1.0, c_bd))),
            );
            self.begin(Style { dir: Dir::Row, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.badge(format!("{}", src.index), if is_hi { Tone::Accent } else { Tone::Neutral });
            self.text_leaf(src.domain.clone(), self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, true, Style { width: Size::FILL, ..Default::default() });
            self.end();
            let mut st_t = self.text_style(TextRole::Caption);
            st_t.weight = 600;
            self.text_leaf(src.title.clone(), st_t, self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
            self.text_leaf(src.snippet.clone(), self.text_style(TextRole::Caption), self.color(Token::TextSecondary), false, true, Style { width: Size::FILL, ..Default::default() });
            self.end();
        }
        self.end();
        self.end();

        self.mem::<CiteMem>(id).active_cite = active_cite;
        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Ethereal kinetic typography card: staggered per-word spring-rise headline
    /// with perceptual OKLab accent gradient tint, plus a replay trigger.
    pub fn text_effects_card(&mut self, key: impl std::hash::Hash, tag: &str, headline: &str, caption: &str) -> Response {
        let id = self.make_id(("text_fx", key));
        #[derive(Default, Clone, Copy)]
        struct FxGen(u32);
        let gen_num = self.mem::<FxGen>(id).0;
        let mo = self.motion();
        let reduced = self.reduced_motion();

        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let fg_c = self.color(Token::Foreground);
        let acc_c = self.color(Token::AccentStrong);
        let c2 = self.color(Token::Series3);

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(20.0), gap: 12.0, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        self.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.badge(tag, Tone::Accent);
        self.begin(Style { width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.end();
        let replay = self.scope("replay_fx", |ui| ui.button("Replay").ghost().small().icon("sparkles").clicked());
        self.end();

        // Staggered per-word spring-rise + OKLab gradient across the final words!
        let words: Vec<&str> = headline.split_whitespace().collect();
        let n_words = words.len().max(1);
        let st_h = self.text_style(TextRole::Heading1);
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 7.0, wrap: true, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for (wi, &word) in words.iter().enumerate() {
            let wid = id.with(("w", gen_num, wi));
            let enter = if reduced {
                1.0
            } else {
                self.anims.spring_from(wid, 0, 0.0, 1.0, mo.bouncy).clamp(0.0, 1.15)
            };
            let grad_t = ((wi as f32 / (n_words.saturating_sub(1).max(1)) as f32) - 0.35).clamp(0.0, 1.0) / 0.65;
            let base_col = fg_c.mix(acc_c.mix(c2, grad_t * 0.5), grad_t);
            let col = base_col.multiply_alpha(enter.clamp(0.0, 1.0));
            let rise = (1.0 - enter.min(1.0)) * (10.0 + wi as f32 * 1.8);
            let g = self.text.layout(word, &st_h, None, lumen_text::TextAlign::Start);
            let sz = vec2(g.width, g.height);
            let word_str = word.to_owned();
            let st_cl = st_h;
            self.leaf(
                Style::default(),
                sz,
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    let g_w = cx.galley(&word_str, &st_cl);
                    cx.painter.text(cx.text, &g_w, vec2(r.min.x, r.min.y + rise), col);
                })),
            );
        }
        self.end();

        self.text_leaf(caption.to_owned(), self.text_style(TextRole::Body), self.color(Token::TextSecondary), true, false, Style { width: Size::FILL, ..Default::default() });
        self.end();

        if replay {
            self.mem::<FxGen>(id).0 = gen_num.wrapping_add(1);
        }
        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Precision rotary parameter knob.
    pub fn rotary_knob<'u>(&'u mut self, label: impl Into<String>, value: &'u mut f32, range: RangeInclusive<f32>) -> RotaryKnob<'u> {
        RotaryKnob {
            ui: self,
            label: label.into(),
            value,
            range,
            unit: String::new(),
            diameter: 74.0,
            shown: false,
        }
    }

    /// Interactive before/after split comparison frame.
    pub fn compare_split<'u>(
        &'u mut self,
        key: impl Into<String>,
        ratio: &'u mut f32,
        left_label: impl Into<String>,
        right_label: impl Into<String>,
    ) -> CompareSplit<'u> {
        CompareSplit {
            ui: self,
            key: key.into(),
            ratio,
            left_label: left_label.into(),
            right_label: right_label.into(),
            height: 196.0,
            shown: false,
        }
    }

    /// Circular `arc_band` countdown / Pomodoro timer card with segmented digit
    /// tiles and Play/Pause + Reset controls.
    pub fn countdown_card(
        &mut self,
        key: impl std::hash::Hash,
        title: &str,
        remaining_secs: &mut u32,
        total_secs: u32,
        running: &mut bool,
    ) -> Response {
        let id = self.make_id(("countdown", key));
        let total = total_secs.max(1);
        *remaining_secs = (*remaining_secs).min(total);

        #[derive(Default, Clone, Copy)]
        struct TickMem {
            last_t: f64,
        }
        if *running && *remaining_secs > 0 {
            let now = self.time();
            let last = self.mem::<TickMem>(id).last_t;
            if last > 0.0 && now - last >= 1.0 {
                let elapsed = (now - last).floor() as u32;
                *remaining_secs = remaining_secs.saturating_sub(elapsed);
                self.mem::<TickMem>(id).last_t = now;
                if *remaining_secs == 0 {
                    *running = false;
                }
            } else if last <= 0.0 {
                self.mem::<TickMem>(id).last_t = now;
            }
            if !self.reduced_motion() {
                self.request_repaint_after(0.25);
            }
        } else {
            self.mem::<TickMem>(id).last_t = 0.0;
        }

        let frac_target = *remaining_secs as f32 / total as f32;
        let frac = self.spring(id, 0, frac_target, self.motion().smooth).clamp(0.0, 1.0);

        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);

        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::all(18.0), gap: 18.0, align: Align::Center, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        // Circular progress ring on the left.
        let track_c = self.color(Token::Track);
        let ring_c = self.color(if *running { Token::Accent } else { Token::Success });
        let pct_str = format!("{:.0}%", frac_target * 100.0);
        let mut st_pct = self.text_style(TextRole::Caption);
        st_pct.tabular = true;
        st_pct.weight = 600;
        let fg_c = self.color(Token::Foreground);
        self.leaf(
            Style::default(),
            vec2(76.0, 76.0),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let c = r.center();
                cx.painter.arc_band(c, 28.0, 35.0, 0.0, 2.0 * PI, track_c);
                if frac > 0.005 {
                    cx.painter.arc_band(c, 28.0, 35.0, 0.0, 2.0 * PI * frac, ring_c);
                }
                let gp = cx.galley(&pct_str, &st_pct);
                cx.painter.text(cx.text, &gp, c - vec2(gp.width * 0.5, gp.height * 0.5), fg_c);
            })),
        );

        // Right column: title, MM : SS digit tiles, and Start/Pause + Reset.
        let mins = *remaining_secs / 60;
        let secs = *remaining_secs % 60;
        self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 8.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf(title.to_owned(), self.text_style(TextRole::Label), self.color(Token::TextSecondary), false, true, Style { width: Size::FILL, ..Default::default() });

        // Digits, then the controls; they wrap under the digits when narrow.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, wrap: true, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let mut st_dig = self.text_style(TextRole::Heading1);
        st_dig.tabular = true;
        for (part, lbl) in [(format!("{mins:02}"), "MIN"), (format!("{secs:02}"), "SEC")] {
            let t_bg = self.color(Token::SurfaceMuted);
            let t_bd = self.color(Token::BorderSubtle);
            self.begin(
                Style { dir: Dir::Row, pad: Sides::xy(10.0, 4.0), gap: 5.0, align: Align::End, ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(8.0), t_bg, 1.0, t_bd))),
            );
            self.text_leaf(part, st_dig, self.color(Token::Foreground), false, false, Style::default());
            self.text_leaf(lbl, self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, false, Style::default());
            self.end();
        }
        self.begin(Style { width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.end();
        let toggle_lbl = if *running { "Pause" } else { "Start" };
        let toggle_ic = if *running { "pause" } else { "play" };
        if self.scope("cd_play", |ui| ui.button(toggle_lbl).primary().small().icon(toggle_ic).clicked()) {
            *running = !*running;
        }
        if self.scope("cd_rst", |ui| ui.icon_button("rotate-ccw").small().tooltip("Reset").clicked()) {
            *running = false;
            *remaining_secs = total;
        }
        self.end();
        self.end();
        self.end();

        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Onboarding / release-readiness checklist card with animated progress bar
    /// and interactive spring-checked steps.
    pub fn checklist_card(&mut self, key: impl std::hash::Hash, title: &str, steps: &mut [ChecklistStep]) -> bool {
        let id = self.make_id(("checklist_card", key));
        let total = steps.len().max(1);
        let done_cnt = steps.iter().filter(|s| s.done).count();
        let prog = done_cnt as f32 / total as f32;

        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let mut changed = false;

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(16.0), gap: 12.0, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        // Header + completion count badge + progress bar.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf(title.to_owned(), self.text_style(TextRole::Heading4), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
        let tone = if done_cnt == steps.len() { Tone::Success } else { Tone::Accent };
        self.badge(format!("{done_cnt} / {} complete", steps.len()), tone);
        self.end();

        self.scope("chk_prog", |ui| ui.progress(Some(prog)));

        // Interactive checklist rows.
        self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 4.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for (si, st) in steps.iter_mut().enumerate() {
            let rid = id.with(("step", si));
            let r_ix = self.interact(rid, Sense::CLICK, false);
            let hov = self.fade(rid, 0, r_ix.hovered, self.motion().hover_in, self.motion().hover_out, Ease::Out);
            let r_bg = self.color(Token::Hover).multiply_alpha(hov);

            self.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(8.0, 6.0), gap: 10.0, align: Align::Center, ..Default::default() },
                Some(rid),
                Sense::CLICK,
                CursorIcon::Pointer,
                false,
                Some(Box::new(move |cx, r| {
                    if hov > 0.005 {
                        cx.painter.rrect(r, Corners::all(7.0), r_bg, 0.0, Color::TRANSPARENT);
                    }
                })),
            );
            let cb_ch = self.scope(("scb", si), |ui| ui.checkbox("", &mut st.done).changed());
            if r_ix.clicked && !cb_ch {
                st.done = !st.done;
                changed = true;
            } else if cb_ch {
                changed = true;
            }

            self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 1.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let t_col = self.color(if st.done { Token::TextMuted } else { Token::Foreground });
            self.text_leaf(st.title.clone(), self.text_style(TextRole::BodyStrong), t_col, false, true, Style { width: Size::FILL, ..Default::default() });
            self.text_leaf(st.detail.clone(), self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, true, Style { width: Size::FILL, ..Default::default() });
            self.end();

            self.badge(&st.tag, if st.done { Tone::Success } else { Tone::Neutral });
            self.end();
        }
        self.end();
        self.end();

        changed
    }
}
