//! Data display and disclosure widgets:
//! - [`Collapsible`] and [`Accordion`] (spring-height disclosure panels)
//! - [`Table`] (sortable columns, row checkboxes with indeterminate header, keyboard row navigation)
//! - [`Tree`] (expandable hierarchy with indent guides, rotating chevrons and sliding selection pill)
//! - [`Timeline`] (vertical activity / deploy feed with status rings and rails)

use std::collections::BTreeSet;

use lumen_text::TextAlign;

use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::id::Id;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides, Vec2};
use crate::paint::Painter;
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

use super::toast::Tone;
use super::{blend3, key_activate, neutral_layers};

/// Draw a crisp anti-aliased chevron centered at `c`, where `angle_rad = 0.0`
/// points right (`>`) and `PI / 2` points down (`v`).
pub(crate) fn draw_chevron(painter: &mut Painter, c: Vec2, radius: f32, angle_rad: f32, stroke: f32, color: Color) {
    let local = [vec2(-radius * 0.45, -radius * 0.85), vec2(radius * 0.50, 0.0), vec2(-radius * 0.45, radius * 0.85)];
    let (s, co) = angle_rad.sin_cos();
    let rot = |v: Vec2| c + vec2(v.x * co - v.y * s, v.x * s + v.y * co);
    painter.polyline(&[rot(local[0]), rot(local[1]), rot(local[2])], stroke, color);
}

// ============================================================== Collapsible

#[derive(Default, Clone, Copy)]
struct CollapsibleMem {
    content_h: f32,
}

/// Single disclosure section whose body expands and collapses on the smooth spring.
pub struct Collapsible<'u> {
    ui: &'u mut Ui,
    key: String,
    title: String,
    open: &'u mut bool,
    icon: Option<String>,
    badge: Option<String>,
    carded: bool,
    disabled: bool,
}

impl<'u> Collapsible<'u> {
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = Some(name.to_owned());
        self
    }
    pub fn badge(mut self, text: impl Into<String>) -> Self {
        self.badge = Some(text.into());
        self
    }
    pub fn carded(mut self, on: bool) -> Self {
        self.carded = on;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }

    pub fn show(self, body: impl FnOnce(&mut Ui)) -> Response {
        let ui = self.ui;
        let id = ui.make_id(("collapsible", &self.key));
        let hdr_id = id.with("hdr");
        let body_id = id.with("body_inner");

        if let Some(prev_r) = ui.rect_of(body_id) {
            if prev_r.height() > 1.0 {
                ui.mem::<CollapsibleMem>(id).content_h = prev_r.height();
            }
        }
        let cached_h = ui.mem::<CollapsibleMem>(id).content_h;

        let ix = ui.interact(hdr_id, Sense::CLICK, self.disabled);
        let (key_act, key_held) = key_activate(ui, &ix, true);
        let mut changed = false;
        if (ix.clicked || key_act) && !self.disabled {
            *self.open = !*self.open;
            changed = true;
        }

        let is_open = *self.open;
        let mo = ui.motion();
        let init_t = if is_open { 1.0 } else { 0.0 };
        let open_t = ui.anims.spring_from(id, 0, init_t, if is_open { 1.0 } else { 0.0 }, mo.smooth).clamp(0.0, 1.0);
        let chev_t = ui.anims.spring_from(id, 1, init_t, if is_open { 1.0 } else { 0.0 }, mo.snappy);
        let (h, p) = ui.state_weights(&ix, key_held);
        let (ring_a, ring_gap) = ui.focus_anim(&ix);

        let r_card = ui.look().card.radius;
        let (card_bg, card_bd, hdr_fill, fg_c, muted_c, badge_bg, ring_c, div_c) = {
            let bg = if self.carded { ui.color(Token::Surface) } else { Color::TRANSPARENT };
            let bd = if self.carded { ui.color(Token::BorderSubtle) } else { Color::TRANSPARENT };
            let (hv, pr) = neutral_layers(ui, Color::TRANSPARENT);
            let hf = blend3(Color::TRANSPARENT, hv, pr, h, p);
            let fg = if self.disabled { ui.color(Token::TextDisabled) } else { ui.color(Token::Foreground) };
            let mc = if self.disabled { ui.color(Token::TextDisabled) } else { ui.color(Token::TextSecondary) };
            (
                bg,
                bd,
                hf,
                fg,
                mc,
                ui.color(Token::SurfaceMuted),
                ui.color(Token::FocusRing),
                ui.color(Token::BorderSubtle),
            )
        };

        let carded = self.carded;
        ui.begin(
            Style { width: Size::FILL, dir: Dir::Column, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            carded,
            if carded {
                Some(Box::new(move |cx, r| {
                    cx.painter.rrect(r, Corners::all(r_card), card_bg, 1.0, card_bd);
                }))
            } else {
                None
            },
        );

        let cursor = if self.disabled { CursorIcon::Default } else { CursorIcon::Pointer };
        let hdr_rad = if carded { (r_card - 1.0).max(0.0) } else { ui.control_metrics(crate::theme::ControlSize::Md).radius };
        ui.begin(
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                pad: Sides::xy(14.0, 11.0),
                gap: 10.0,
                align: Align::Center,
                ..Default::default()
            },
            Some(hdr_id),
            Sense::CLICK,
            cursor,
            false,
            Some(Box::new(move |cx, r| {
                if hdr_fill.a > 0.0 {
                    cx.painter.rrect(r, Corners::all(hdr_rad), hdr_fill, 0.0, Color::TRANSPARENT);
                }
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, hdr_rad, ring_a, ring_gap, ring_c)));
                }
            })),
        );

        if let Some(ic) = &self.icon {
            ui.icon_leaf(ic, 15.0, muted_c);
        }
        let st_title = ui.text_style(TextRole::BodyStrong);
        ui.text_leaf(self.title, st_title, fg_c, false, true, Style { width: Size::FILL, ..Default::default() });

        if let Some(b) = self.badge {
            let mut st_b = ui.text_style(TextRole::Caption);
            st_b.weight = 500;
            ui.begin(
                Style { pad: Sides::xy(7.0, 2.0), ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| {
                    cx.painter.rrect(r, Corners::all(r.height() / 2.0), badge_bg, 0.0, Color::TRANSPARENT);
                })),
            );
            ui.text_leaf(b, st_b, muted_c, false, false, Style::default());
            ui.end();
        }

        // Smoothly rotating chevron (points down at PI/2 when closed, up at -PI/2 when open).
        let angle = std::f32::consts::FRAC_PI_2 - chev_t * std::f32::consts::PI;
        ui.leaf(
            Style::default(),
            vec2(16.0, 16.0),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                draw_chevron(&mut cx.painter, r.center(), 4.5, angle, 1.6, muted_c);
            })),
        );
        ui.end(); // header

        if is_open || open_t > 0.005 {
            let settled_open = is_open && (open_t > 0.995 || cached_h <= 0.0);
            let outer_h = if settled_open { Size::Hug } else { Size::Fixed((cached_h * open_t).round().max(0.0)) };
            let line_a = open_t.clamp(0.0, 1.0);
            ui.begin(
                Style { width: Size::FILL, height: outer_h, dir: Dir::Column, ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                true,
                Some(Box::new(move |cx, r| {
                    if carded && r.height() > 1.0 {
                        cx.painter.rect(Rect::from_xywh(r.min.x + 14.0, r.min.y, (r.width() - 28.0).max(0.0), 1.0), div_c.multiply_alpha(line_a));
                    }
                })),
            );
            ui.begin(
                Style {
                    width: Size::FILL,
                    dir: Dir::Column,
                    pad: Sides { left: 14.0, right: 14.0, top: 10.0, bottom: 14.0 },
                    gap: 8.0,
                    ..Default::default()
                },
                Some(body_id),
                Sense::NONE,
                CursorIcon::Default,
                false,
                None,
            );
            ui.scope("content", body);
            ui.end();
            ui.end();
            if is_open && cached_h <= 0.0 {
                // Ensure cached_h is recorded without scheduling an extra frame if already Hug.
            }
        }

        ui.end(); // root
        let rect = ui.rect_of(id).unwrap_or(ix.rect);
        Response { id, rect, hovered: ix.hovered, clicked: changed, changed, focused: ix.focused }
    }
}

/// Stacked accordion card where at most one section (`open: &mut Option<usize>`)
/// stays open at a time, with Up/Down arrow navigation across headers.
pub struct AccordionItem<'a> {
    pub title: &'a str,
    pub icon: Option<&'a str>,
    pub badge: Option<&'a str>,
    pub body: Box<dyn FnOnce(&mut Ui) + 'a>,
}

pub struct Accordion<'u, 'a> {
    ui: &'u mut Ui,
    key: String,
    open: &'u mut Option<usize>,
    items: Vec<AccordionItem<'a>>,
}

impl<'u, 'a> Accordion<'u, 'a> {
    pub fn item(mut self, title: &'a str, body: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.items.push(AccordionItem { title, icon: None, badge: None, body: Box::new(body) });
        self
    }

    pub fn item_with(mut self, title: &'a str, icon: Option<&'a str>, badge: Option<&'a str>, body: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.items.push(AccordionItem { title, icon, badge, body: Box::new(body) });
        self
    }

    pub fn show(self) -> Response {
        let Accordion { ui, key, open, items } = self;
        let id = ui.make_id(("accordion", &key));
        let n = items.len();
        let hdr_ids: Vec<Id> = (0..n).map(|i| id.with(("sec", i)).with("hdr")).collect();

        // Register interactions in visual order? Wait: Collapsible::show calls interact(hdr_id).
        // Handle Up/Down arrow navigation if any header was focused last frame!
        if let Some(cur_focus) = ui.focused() {
            if let Some(pos) = hdr_ids.iter().position(|&h| h == cur_focus) {
                if ui.key_pressed_for(cur_focus, Key::ArrowDown) {
                    ui.request_focus(hdr_ids[(pos + 1) % n]);
                } else if ui.key_pressed_for(cur_focus, Key::ArrowUp) {
                    ui.request_focus(hdr_ids[(pos + n - 1) % n]);
                } else if ui.key_pressed_for(cur_focus, Key::Home) {
                    ui.request_focus(hdr_ids[0]);
                } else if ui.key_pressed_for(cur_focus, Key::End) {
                    ui.request_focus(hdr_ids[n - 1]);
                }
            }
        }

        let r_card = ui.look().card.radius;
        let card_bg = ui.color(Token::Surface);
        let card_bd = ui.color(Token::BorderSubtle);
        let div_c = ui.color(Token::BorderSubtle);

        let mut any_changed = false;
        ui.begin(
            Style { width: Size::FILL, dir: Dir::Column, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_card), card_bg, 1.0, card_bd);
            })),
        );

        for (i, item) in items.into_iter().enumerate() {
            if i > 0 {
                ui.leaf(
                    Style { width: Size::FILL, ..Default::default() },
                    vec2(0.0, 1.0),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, r| cx.painter.rect(r, div_c))),
                );
            }
            let mut is_open = *open == Some(i);
            let sec_id = id.with(("sec", i));
            let body_id = sec_id.with("body_inner");
            if let Some(prev_r) = ui.rect_of(body_id) {
                if prev_r.height() > 1.0 {
                    ui.mem::<CollapsibleMem>(sec_id).content_h = prev_r.height();
                }
            }
            let cached_h = ui.mem::<CollapsibleMem>(sec_id).content_h;
            let hdr_id = hdr_ids[i];
            let ix = ui.interact(hdr_id, Sense::CLICK, false);
            let (key_act, key_held) = key_activate(ui, &ix, true);
            if ix.clicked || key_act {
                is_open = !is_open;
                *open = if is_open { Some(i) } else { None };
                any_changed = true;
            }
            let mo = ui.motion();
            let init_t = if is_open { 1.0 } else { 0.0 };
            let open_t = ui.anims.spring_from(sec_id, 0, init_t, if is_open { 1.0 } else { 0.0 }, mo.smooth).clamp(0.0, 1.0);
            let chev_t = ui.anims.spring_from(sec_id, 1, init_t, if is_open { 1.0 } else { 0.0 }, mo.snappy);
            let (h, p) = ui.state_weights(&ix, key_held);
            let (ring_a, ring_gap) = ui.focus_anim(&ix);

            let (hdr_fill, fg_c, muted_c, badge_bg, ring_c) = {
                let (hv, pr) = neutral_layers(ui, Color::TRANSPARENT);
                (
                    blend3(Color::TRANSPARENT, hv, pr, h, p),
                    ui.color(Token::Foreground),
                    ui.color(Token::TextSecondary),
                    ui.color(Token::SurfaceMuted),
                    ui.color(Token::FocusRing),
                )
            };

            ui.begin(
                Style {
                    dir: Dir::Row,
                    width: Size::FILL,
                    pad: Sides::xy(16.0, 12.0),
                    gap: 10.0,
                    align: Align::Center,
                    ..Default::default()
                },
                Some(hdr_id),
                Sense::CLICK,
                CursorIcon::Pointer,
                false,
                Some(Box::new(move |cx, r| {
                    if hdr_fill.a > 0.0 {
                        cx.painter.rect(r, hdr_fill);
                    }
                    if ring_a > 0.0 {
                        cx.defer(r.expand(-2.0), Box::new(move |cx, r| cx.focus_ring(r, 6.0, ring_a, ring_gap, ring_c)));
                    }
                })),
            );
            if let Some(ic) = item.icon {
                ui.icon_leaf(ic, 15.0, muted_c);
            }
            let st_t = ui.text_style(TextRole::BodyStrong);
            ui.text_leaf(item.title, st_t, fg_c, false, true, Style { width: Size::FILL, ..Default::default() });
            if let Some(b) = item.badge {
                let mut st_b = ui.text_style(TextRole::Caption);
                st_b.weight = 500;
                ui.begin(
                    Style { pad: Sides::xy(7.0, 2.0), ..Default::default() },
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r.height() / 2.0), badge_bg, 0.0, Color::TRANSPARENT))),
                );
                ui.text_leaf(b, st_b, muted_c, false, false, Style::default());
                ui.end();
            }
            let angle = std::f32::consts::FRAC_PI_2 - chev_t * std::f32::consts::PI;
            ui.leaf(
                Style::default(),
                vec2(16.0, 16.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| draw_chevron(&mut cx.painter, r.center(), 4.5, angle, 1.6, muted_c))),
            );
            ui.end();

            if is_open || open_t > 0.005 {
                let settled_open = is_open && (open_t > 0.995 || cached_h <= 0.0);
                let outer_h = if settled_open { Size::Hug } else { Size::Fixed((cached_h * open_t).round().max(0.0)) };
                ui.begin(Style { width: Size::FILL, height: outer_h, dir: Dir::Column, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, true, None);
                ui.begin(
                    Style {
                        width: Size::FILL,
                        dir: Dir::Column,
                        pad: Sides { left: 16.0, right: 16.0, top: 2.0, bottom: 14.0 },
                        gap: 8.0,
                        ..Default::default()
                    },
                    Some(body_id),
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    None,
                );
                ui.scope(("body", i), item.body);
                ui.end();
                ui.end();
            }
        }

        ui.end();
        let rect = ui.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, hovered: false, clicked: any_changed, changed: any_changed, focused: false }
    }
}

// =============================================================== Data Table

/// Sort direction of a [`Table`] column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}

impl SortDir {
    pub fn toggled(self) -> SortDir {
        match self {
            SortDir::Asc => SortDir::Desc,
            SortDir::Desc => SortDir::Asc,
        }
    }
}

/// Column specification for [`Table`].
#[derive(Clone)]
pub struct TableCol {
    pub label: String,
    pub size: Size,
    pub sortable: bool,
    pub right_align: bool,
}

impl TableCol {
    pub fn new(label: impl Into<String>) -> Self {
        TableCol { label: label.into(), size: Size::FILL, sortable: false, right_align: false }
    }
    pub fn fill(mut self, weight: f32) -> Self {
        self.size = Size::Fill(weight.max(0.1));
        self
    }
    pub fn width(mut self, px: f32) -> Self {
        self.size = Size::Fixed(px);
        self
    }
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }
    pub fn right(mut self) -> Self {
        self.right_align = true;
        self
    }
}

/// Rich cell content for [`Table`].
#[derive(Clone)]
pub enum TableCell {
    Text(String),
    Strong(String),
    Muted(String),
    Mono(String),
    Badge(String, Tone),
}

impl TableCell {
    pub fn text(s: impl Into<String>) -> Self {
        TableCell::Text(s.into())
    }
    pub fn strong(s: impl Into<String>) -> Self {
        TableCell::Strong(s.into())
    }
    pub fn muted(s: impl Into<String>) -> Self {
        TableCell::Muted(s.into())
    }
    pub fn mono(s: impl Into<String>) -> Self {
        TableCell::Mono(s.into())
    }
    pub fn badge(s: impl Into<String>, tone: Tone) -> Self {
        TableCell::Badge(s.into(), tone)
    }
}

impl From<&str> for TableCell {
    fn from(s: &str) -> Self {
        TableCell::Text(s.to_owned())
    }
}

impl From<String> for TableCell {
    fn from(s: String) -> Self {
        TableCell::Text(s)
    }
}

/// Interactive data table with sortable column headers, optional row selection
/// checkboxes (with an indeterminate header toggle), hover highlights and
/// keyboard navigation.
pub struct Table<'u> {
    ui: &'u mut Ui,
    key: String,
    cols: Vec<TableCol>,
    rows: Vec<(usize, Vec<TableCell>)>,
    sort: Option<&'u mut Option<(usize, SortDir)>>,
    selected: Option<&'u mut Vec<bool>>,
    clicked_row: Option<usize>,
    lazy: Option<LazyRows<'u>>,
    height: Option<f32>,
}

/// Rows produced on demand by [`Table::lazy_rows`].
struct LazyRows<'u> {
    count: usize,
    id_of: Box<dyn Fn(usize) -> usize + 'u>,
    cells_of: Box<dyn FnMut(usize) -> Vec<TableCell> + 'u>,
}

#[derive(Default)]
struct TableMem {
    /// Index of the row that had keyboard focus last frame.
    focus: Option<usize>,
    page: usize,
}

impl<'u> Table<'u> {
    pub fn columns(mut self, cols: impl IntoIterator<Item = TableCol>) -> Self {
        self.cols = cols.into_iter().collect();
        self
    }
    pub fn sort_by(mut self, sort: &'u mut Option<(usize, SortDir)>) -> Self {
        self.sort = Some(sort);
        self
    }
    pub fn selection(mut self, sel: &'u mut Vec<bool>) -> Self {
        self.selected = Some(sel);
        self
    }
    /// Append a row keyed by its stable `row_id` (so checkboxes and hover
    /// states follow the row across re-sorts).
    pub fn row(mut self, row_id: usize, cells: impl IntoIterator<Item = impl Into<TableCell>>) -> Self {
        self.rows.push((row_id, cells.into_iter().map(Into::into).collect()));
        self
    }
    /// `count` rows made on demand: `id_of(i)` is row `i`'s stable id and
    /// `cells_of(i)` its cells, asked only for rows in view. Use this for
    /// big tables (100k rows) instead of [`row`](Self::row).
    pub fn lazy_rows(mut self, count: usize, id_of: impl Fn(usize) -> usize + 'u, cells_of: impl FnMut(usize) -> Vec<TableCell> + 'u) -> Self {
        self.lazy = Some(LazyRows { count, id_of: Box::new(id_of), cells_of: Box::new(cells_of) });
        self
    }
    /// Give the body this height and scroll it under a sticky header.
    /// Only the rows in view are built, with or without this (a table in a
    /// [`Ui::scroll`] scrolls with the page).
    pub fn height(mut self, px: f32) -> Self {
        self.height = Some(px);
        self
    }

    pub fn show(mut self) -> Option<usize> {
        let ui = &mut *self.ui;
        let id = ui.make_id(("table", &self.key));
        let r_card = ui.look().card.radius;
        let card_bg = ui.color(Token::Surface);
        let card_bd = ui.color(Token::BorderSubtle);
        let hdr_bg = ui.color(Token::SurfaceMuted).multiply_alpha(0.55);
        let div_c = ui.color(Token::BorderSubtle);

        let has_sel = self.selected.is_some();
        let (lazy_ids, mut lazy_cells) = match self.lazy.take() {
            Some(l) => (Some((l.count, l.id_of)), Some(l.cells_of)),
            None => (None, None),
        };
        let eager = std::mem::take(&mut self.rows);
        let count = lazy_ids.as_ref().map_or(eager.len(), |l| l.0);
        // Row ids are looked up per row; the full list is only collected for
        // the select-all checkbox.
        let rid_of = |i: usize| match &lazy_ids {
            Some((_, f)) => f(i),
            None => eager[i].0,
        };
        let row_ids: Vec<usize> = if has_sel { (0..count).map(rid_of).collect() } else { vec![] };
        if let Some(sel) = self.selected.as_deref_mut() {
            let max_id = row_ids.iter().copied().max().map(|m| m + 1).unwrap_or(0);
            if sel.len() < max_id {
                sel.resize(max_id, false);
            }
        }

        // Keyboard navigation across rows, on the row focused last frame.
        // The row it moves to is scrolled into view (and built) this frame.
        let row_wid_of = |i: usize| id.with(("row", rid_of(i)));
        let focused_i = ui.mem::<TableMem>(id).focus.filter(|&i| i < count && ui.focused() == Some(row_wid_of(i)));
        let mut reveal = None;
        if let Some(pos) = focused_i {
            let cur = row_wid_of(pos);
            let page = ui.mem::<TableMem>(id).page.max(1);
            let go = if ui.key_pressed_for(cur, Key::ArrowDown) {
                Some((pos + 1) % count)
            } else if ui.key_pressed_for(cur, Key::ArrowUp) {
                Some((pos + count - 1) % count)
            } else if ui.key_pressed_for(cur, Key::PageDown) {
                Some((pos + page).min(count - 1))
            } else if ui.key_pressed_for(cur, Key::PageUp) {
                Some(pos.saturating_sub(page))
            } else if ui.key_pressed_for(cur, Key::Home) {
                Some(0)
            } else if ui.key_pressed_for(cur, Key::End) {
                Some(count - 1)
            } else {
                None
            };
            if let Some(g) = go {
                ui.request_focus(row_wid_of(g));
                reveal = Some(g);
            }
        }

        ui.a11y(id, || A11y::new(Role::Table));
        ui.begin(
            Style { width: Size::FILL, dir: Dir::Column, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_card), card_bg, 1.0, card_bd);
            })),
        );

        // Header row.
        let top_rad = (r_card - 1.0).max(0.0);
        ui.begin(
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                pad: Sides::xy(14.0, 9.0),
                gap: 12.0,
                align: Align::Center,
                ..Default::default()
            },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                let inner = Rect::from_xywh(r.min.x + 1.0, r.min.y + 1.0, (r.width() - 2.0).max(0.0), (r.height() - 1.0).max(0.0));
                cx.painter.rrect(inner, Corners([top_rad, top_rad, 0.0, 0.0]), hdr_bg, 0.0, Color::TRANSPARENT);
                cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.max.y - 1.0, (r.width() - 2.0).max(0.0), 1.0), div_c);
            })),
        );

        if let Some(sel) = self.selected.as_deref_mut() {
            let sel_count = row_ids.iter().filter(|&&rid| sel.get(rid).copied().unwrap_or(false)).count();
            let all_sel = !row_ids.is_empty() && sel_count == row_ids.len();
            let some_sel = sel_count > 0 && !all_sel;
            let mut hdr_chk = all_sel;
            let resp = ui.scope("hdr_chk", |ui| ui.checkbox("", &mut hdr_chk).indeterminate(some_sel).show());
            ui.a11y_update(resp.id, |a| a.label = Some("Select all rows".into()));
            if resp.changed {
                let target = if some_sel { true } else { hdr_chk };
                for &rid in &row_ids {
                    if let Some(slot) = sel.get_mut(rid) {
                        *slot = target;
                    }
                }
            }
        }

        let cur_sort = self.sort.as_deref().copied().flatten();
        let mut next_sort = cur_sort;

        for (ci, col) in self.cols.iter().enumerate() {
            let cid = id.with(("col", ci));
            let active_dir = cur_sort.and_then(|(sc, sd)| (sc == ci).then_some(sd));
            let can_sort = col.sortable && self.sort.is_some();
            let ix = ui.interact(cid, if can_sort { Sense::CLICK } else { Sense::NONE }, false);
            ui.a11y(cid, || match active_dir {
                Some(SortDir::Asc) => A11y::new(Role::ColumnHeader).description("Sorted ascending"),
                Some(SortDir::Desc) => A11y::new(Role::ColumnHeader).description("Sorted descending"),
                None => A11y::new(Role::ColumnHeader),
            });
            let (key_act, key_held) = if can_sort { key_activate(ui, &ix, true) } else { (false, false) };
            if can_sort && (ix.clicked || key_act) {
                next_sort = match active_dir {
                    None => Some((ci, SortDir::Asc)),
                    Some(SortDir::Asc) => Some((ci, SortDir::Desc)),
                    Some(SortDir::Desc) => None,
                };
            }
            let mo = ui.motion();
            let (h, p) = ui.state_weights(&ix, key_held);
            let (ring_a, ring_gap) = ui.focus_anim(&ix);
            let rot_target = match active_dir {
                Some(SortDir::Desc) => 1.0,
                _ => 0.0,
            };
            let rot_t = ui.anims.spring_from(cid, 0, rot_target, rot_target, mo.snappy);
            let show_t = ui.fade(cid, 1, active_dir.is_some() || (can_sort && ix.hovered), 0.10, 0.14, Ease::Out);

            let lbl_c = if active_dir.is_some() {
                ui.color(Token::Foreground)
            } else if can_sort && h > 0.01 {
                ui.color(Token::TextSecondary).mix(ui.color(Token::Foreground), h.max(p))
            } else {
                ui.color(Token::TextSecondary)
            };
            let arrow_c = if active_dir.is_some() { ui.color(Token::AccentStrong) } else { ui.color(Token::TextMuted) };
            let ring_c = ui.color(Token::FocusRing);

            let justify = if col.right_align { Justify::End } else { Justify::Start };
            ui.begin(
                Style {
                    dir: Dir::Row,
                    width: col.size,
                    gap: 4.0,
                    align: Align::Center,
                    justify,
                    ..Default::default()
                },
                Some(cid),
                if can_sort { Sense::CLICK } else { Sense::NONE },
                if can_sort { CursorIcon::Pointer } else { CursorIcon::Default },
                false,
                Some(Box::new(move |cx, r| {
                    if ring_a > 0.0 {
                        cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, 4.0, ring_a, ring_gap, ring_c)));
                    }
                })),
            );
            let mut st_h = ui.text_style(TextRole::Caption);
            st_h.weight = 600;
            ui.text_leaf(col.label.clone(), st_h, lbl_c, false, true, Style::default());
            if can_sort && show_t > 0.01 {
                // Up arrow at rot_t = 0 (-PI/2), Down arrow at rot_t = 1 (+PI/2).
                let angle = -std::f32::consts::FRAC_PI_2 + rot_t * std::f32::consts::PI;
                let ac = arrow_c.multiply_alpha(show_t);
                ui.leaf(
                    Style::default(),
                    vec2(12.0, 12.0),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, r| {
                        draw_chevron(&mut cx.painter, r.center(), 3.6, angle, 1.5, ac);
                    })),
                );
            }
            ui.end();
        }
        ui.end(); // header

        if let Some(sort_ref) = self.sort.as_deref_mut() {
            *sort_ref = next_sort;
        }

        // Body rows: only those in view are built.
        let cols = &self.cols;
        let selected = &mut self.selected;
        let clicked_row = &mut self.clicked_row;
        let mut focus_now = None;
        let mut body = |ui: &mut Ui, ri: usize| {
            let rid = rid_of(ri);
            let cells: Vec<TableCell> = match lazy_cells.as_mut() {
                Some(f) => f(ri),
                None => eager[ri].1.clone(),
            };
            let row_wid = id.with(("row", rid));
            let row_ix = ui.interact(row_wid, Sense::CLICK, false);
            if row_ix.focused {
                focus_now = Some(ri);
            }
            let row_sel = selected.as_deref().and_then(|s| s.get(rid).copied());
            ui.a11y(row_wid, || {
                let a = A11y::new(Role::Row).position(ri, count);
                match row_sel {
                    Some(s) => a.selected(s),
                    None => a,
                }
            });
            let (key_act, key_held) = key_activate(ui, &row_ix, true);
            let is_sel = selected.as_deref().and_then(|s| s.get(rid).copied()).unwrap_or(false);

            if row_ix.clicked || key_act {
                *clicked_row = Some(rid);
                if key_act {
                    if let Some(sel) = selected.as_deref_mut() {
                        if let Some(slot) = sel.get_mut(rid) {
                            *slot = !*slot;
                        }
                    }
                }
            }

            let mo = ui.motion();
            let (h, p) = ui.state_weights(&row_ix, key_held);
            let sel_t = ui.fade(row_wid, 0, is_sel, mo.selected, mo.selected, Ease::Out);
            let (ring_a, ring_gap) = ui.focus_anim(&row_ix);

            let (row_fill, sep_c, ring_c) = {
                let (hv, pr) = neutral_layers(ui, Color::TRANSPARENT);
                let base_h = blend3(Color::TRANSPARENT, hv, pr, h, p);
                let sel_bg = ui.color(Token::AccentSubtle).multiply_alpha(0.55);
                (base_h.over(sel_bg.multiply_alpha(sel_t)), ui.color(Token::BorderSubtle), ui.color(Token::FocusRing))
            };
            let draw_top_sep = ri > 0;
            let is_last_row = ri + 1 == count;
            let bot_rad = if is_last_row { (r_card - 1.0).max(0.0) } else { 0.0 };

            ui.begin(
                Style {
                    dir: Dir::Row,
                    width: Size::FILL,
                    pad: Sides::xy(14.0, 9.0),
                    gap: 12.0,
                    align: Align::Center,
                    ..Default::default()
                },
                Some(row_wid),
                Sense::CLICK,
                CursorIcon::Pointer,
                false,
                Some(Box::new(move |cx, r| {
                    if row_fill.a > 0.0 {
                        let inner = Rect::from_xywh(r.min.x + 1.0, r.min.y, (r.width() - 2.0).max(0.0), (r.height() - if is_last_row { 1.0 } else { 0.0 }).max(0.0));
                        cx.painter.rrect(inner, Corners([0.0, 0.0, bot_rad, bot_rad]), row_fill, 0.0, Color::TRANSPARENT);
                    }
                    if draw_top_sep {
                        cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.min.y, (r.width() - 2.0).max(0.0), 1.0), sep_c.multiply_alpha(0.7));
                    }
                    if ring_a > 0.0 {
                        cx.defer(r.expand(-2.0), Box::new(move |cx, r| cx.focus_ring(r, 4.0, ring_a, ring_gap, ring_c)));
                    }
                })),
            );

            if has_sel {
                if let Some(sel) = selected.as_deref_mut() {
                    let mut v = sel[rid];
                    let c_resp = ui.scope(("rchk", rid), |ui| ui.checkbox("", &mut v).show());
                    ui.a11y_update(c_resp.id, |a| a.label = Some("Select row".into()));
                    if c_resp.changed {
                        sel[rid] = v;
                    }
                }
            }

            for (ci, col) in cols.iter().enumerate() {
                let cell = cells.get(ci).cloned().unwrap_or(TableCell::Text(String::new()));
                let right = col.right_align;
                let justify = if right { Justify::End } else { Justify::Start };
                ui.begin(
                    Style { dir: Dir::Row, width: col.size, align: Align::Center, justify, ..Default::default() },
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    None,
                );
                match cell {
                    TableCell::Text(s) => {
                        let st = ui.text_style(TextRole::Body);
                        let c = ui.color(Token::Foreground);
                        let n = ui.text_leaf(s, st, c, false, true, Style { width: if right { Size::Hug } else { Size::FILL }, ..Default::default() });
                        if right {
                            ui.set_text_align(n, TextAlign::End);
                        }
                    }
                    TableCell::Strong(s) => {
                        let st = ui.text_style(TextRole::BodyStrong);
                        let c = ui.color(Token::Foreground);
                        let n = ui.text_leaf(s, st, c, false, true, Style { width: if right { Size::Hug } else { Size::FILL }, ..Default::default() });
                        if right {
                            ui.set_text_align(n, TextAlign::End);
                        }
                    }
                    TableCell::Muted(s) => {
                        let st = ui.text_style(TextRole::Body);
                        let c = ui.color(Token::TextSecondary);
                        let n = ui.text_leaf(s, st, c, false, true, Style { width: if right { Size::Hug } else { Size::FILL }, ..Default::default() });
                        if right {
                            ui.set_text_align(n, TextAlign::End);
                        }
                    }
                    TableCell::Mono(s) => {
                        let st = ui.text_style(TextRole::Mono);
                        let c = ui.color(Token::TextSecondary);
                        let n = ui.text_leaf(s, st, c, false, true, Style { width: if right { Size::Hug } else { Size::FILL }, ..Default::default() });
                        if right {
                            ui.set_text_align(n, TextAlign::End);
                        }
                    }
                    TableCell::Badge(s, tone) => {
                        ui.badge(&s, tone);
                    }
                }
                ui.end();
            }
            ui.end(); // row
        };
        let pin = focused_i.or(reveal);
        let vout = if let Some(h) = self.height {
            ui.scroll(("tbody", &self.key)).height(h).show(|ui| ui.virtual_rows("rows", count).estimate(40.0).pin(pin).reveal(reveal).show(&mut body)).0
        } else {
            ui.virtual_rows("rows", count).estimate(40.0).pin(pin).reveal(reveal).show(&mut body)
        };
        {
            let m = ui.mem::<TableMem>(id);
            m.focus = focus_now;
            if count > 0 && vout.total_height > 0.0 {
                m.page = ((vout.viewport / (vout.total_height / count as f32)).floor() as usize).saturating_sub(1).max(1);
            }
        }

        ui.end(); // table
        self.clicked_row
    }
}

// ==================================================================== Tree

/// Node in a hierarchical [`Tree`].
#[derive(Clone)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub meta: Option<String>,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn branch(id: impl Into<String>, label: impl Into<String>, children: impl IntoIterator<Item = TreeNode>) -> Self {
        TreeNode {
            id: id.into(),
            label: label.into(),
            icon: Some("folder".to_owned()),
            meta: None,
            children: children.into_iter().collect(),
        }
    }
    pub fn leaf(id: impl Into<String>, label: impl Into<String>) -> Self {
        TreeNode {
            id: id.into(),
            label: label.into(),
            icon: Some("file-text".to_owned()),
            meta: None,
            children: vec![],
        }
    }
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = Some(name.to_owned());
        self
    }
    pub fn meta(mut self, text: impl Into<String>) -> Self {
        self.meta = Some(text.into());
        self
    }
    pub fn is_branch(&self) -> bool {
        !self.children.is_empty()
    }
}

struct FlatTreeRow<'a> {
    node: &'a TreeNode,
    depth: usize,
    parent: Option<String>,
}

fn flatten_tree<'a>(nodes: &'a [TreeNode], expanded: &BTreeSet<String>, depth: usize, parent: Option<&str>, out: &mut Vec<FlatTreeRow<'a>>) {
    for n in nodes {
        out.push(FlatTreeRow { node: n, depth, parent: parent.map(str::to_owned) });
        if n.is_branch() && expanded.contains(&n.id) {
            flatten_tree(&n.children, expanded, depth + 1, Some(&n.id), out);
        }
    }
}

/// Hierarchical tree view with rotating branch chevrons, indent guides, a
/// spring-sliding selection pill, and full arrow-key navigation.
pub struct Tree<'u, 'a> {
    ui: &'u mut Ui,
    key: String,
    nodes: &'a [TreeNode],
    expanded: &'u mut BTreeSet<String>,
    selected: &'u mut String,
}

impl<'u, 'a> Tree<'u, 'a> {
    pub fn show(self) -> Response {
        let Tree { ui, key, nodes, expanded, selected } = self;
        let id = ui.make_id(("tree", &key));
        let ix = ui.interact(id, Sense::CLICK, false);
        let mo = ui.motion();
        let (ring_a, ring_gap) = ui.focus_anim(&ix);

        let mut rows = Vec::new();
        flatten_tree(nodes, expanded, 0, None, &mut rows);

        let mut changed = false;

        // Keyboard navigation when the tree container is focused.
        if ix.focused && !rows.is_empty() {
            let cur_idx = rows.iter().position(|r| r.node.id == *selected).unwrap_or(0);
            for (k, _, _) in ui.keys_for(id) {
                match k {
                    Key::ArrowDown => {
                        let next = (cur_idx + 1).min(rows.len() - 1);
                        if *selected != rows[next].node.id {
                            *selected = rows[next].node.id.clone();
                            changed = true;
                        }
                    }
                    Key::ArrowUp => {
                        let prev = cur_idx.saturating_sub(1);
                        if *selected != rows[prev].node.id {
                            *selected = rows[prev].node.id.clone();
                            changed = true;
                        }
                    }
                    Key::ArrowRight => {
                        let cur = &rows[cur_idx];
                        if cur.node.is_branch() {
                            if !expanded.contains(&cur.node.id) {
                                expanded.insert(cur.node.id.clone());
                                changed = true;
                            } else if let Some(first) = cur.node.children.first() {
                                *selected = first.id.clone();
                                changed = true;
                            }
                        }
                    }
                    Key::ArrowLeft => {
                        let cur = &rows[cur_idx];
                        if cur.node.is_branch() && expanded.contains(&cur.node.id) {
                            expanded.remove(&cur.node.id);
                            changed = true;
                        } else if let Some(parent_id) = &cur.parent {
                            *selected = parent_id.clone();
                            changed = true;
                        }
                    }
                    Key::Enter | Key::Space => {
                        let cur = &rows[cur_idx];
                        if cur.node.is_branch() {
                            if !expanded.remove(&cur.node.id) {
                                expanded.insert(cur.node.id.clone());
                            }
                            changed = true;
                        }
                    }
                    Key::Home => {
                        *selected = rows[0].node.id.clone();
                        changed = true;
                    }
                    Key::End => {
                        if let Some(last) = rows.last() {
                            *selected = last.node.id.clone();
                            changed = true;
                        }
                    }
                    _ => {}
                }
            }
            if changed {
                rows.clear();
                flatten_tree(nodes, expanded, 0, None, &mut rows);
            }
        }

        // Pre-interact rows (non-focusable so the tree itself is one clean Tab stop).
        let mut row_ixs = Vec::with_capacity(rows.len());
        for r in &rows {
            let rid = id.with(("node", &r.node.id));
            let rix = ui.interact(rid, Sense { click: true, drag: false, focusable: false, scroll: false }, false);
            if rix.clicked {
                ui.request_focus(id);
                if *selected != r.node.id {
                    *selected = r.node.id.clone();
                    changed = true;
                }
                if r.node.is_branch() {
                    if !expanded.remove(&r.node.id) {
                        expanded.insert(r.node.id.clone());
                    }
                    changed = true;
                }
            }
            row_ixs.push((rid, rix));
        }

        let sel_idx = rows.iter().position(|r| r.node.id == *selected);
        let row_h = 30.0;
        let row_gap = 2.0;
        let pad_v = 8.0;
        let target_y = sel_idx.map(|i| pad_v + i as f32 * (row_h + row_gap)).unwrap_or(pad_v);
        let pill_y = ui.anims.spring_from(id, 0, target_y, target_y, mo.snappy);
        let has_sel = sel_idx.is_some();

        let r_card = ui.look().card.radius;
        let card_bg = ui.color(Token::Surface);
        let card_bd = ui.color(Token::BorderSubtle);
        let pill_fill = ui.color(Token::AccentSubtle);
        let pill_bar = ui.color(Token::Accent);
        let ring_c = ui.color(Token::FocusRing);

        ui.begin(
            Style {
                width: Size::FILL,
                dir: Dir::Column,
                pad: Sides::xy(8.0, pad_v),
                gap: row_gap,
                ..Default::default()
            },
            Some(id),
            Sense::CLICK,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_card), card_bg, 1.0, card_bd);
                if has_sel {
                    let pr = Rect::from_xywh(r.min.x + 8.0, r.min.y + pill_y, (r.width() - 16.0).max(0.0), row_h);
                    cx.painter.rrect(pr, Corners::all(7.0), pill_fill, 0.0, Color::TRANSPARENT);
                    let bar = Rect::from_xywh(pr.min.x + 2.0, pr.min.y + 7.0, 2.5, row_h - 14.0);
                    cx.painter.rrect(bar, Corners::all(1.5), pill_bar, 0.0, Color::TRANSPARENT);
                }
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, r_card, ring_a, ring_gap, ring_c)));
                }
            })),
        );

        for (i, r) in rows.iter().enumerate() {
            let (rid, rix) = &row_ixs[i];
            let is_sel = r.node.id == *selected;
            let is_open = r.node.is_branch() && expanded.contains(&r.node.id);
            let chev_target = if is_open { 1.0 } else { 0.0 };
            let chev_t = ui.anims.spring_from(*rid, 0, chev_target, chev_target, mo.snappy);
            let h = ui.fade(*rid, 1, rix.hovered && !is_sel, mo.hover_in, mo.hover_out, Ease::Linear);

            let hov_c = ui.color(Token::Hover).multiply_alpha(h);
            let guide_c = ui.color(Token::BorderSubtle);
            let chev_c = ui.color(Token::TextMuted);
            let ic_c = if is_sel {
                ui.color(Token::AccentStrong)
            } else if r.node.is_branch() {
                ui.color(Token::Accent).mix(ui.color(Token::TextSecondary), 0.25)
            } else {
                ui.color(Token::TextMuted)
            };
            let fg_c = if is_sel { ui.color(Token::AccentStrong) } else { ui.color(Token::Foreground) };
            let meta_c = ui.color(Token::TextMuted);
            let depth = r.depth;
            let indent = depth as f32 * 16.0;

            ui.begin(
                Style {
                    dir: Dir::Row,
                    width: Size::FILL,
                    height: Size::Fixed(row_h),
                    pad: Sides { left: 10.0 + indent, right: 10.0, top: 0.0, bottom: 0.0 },
                    gap: 6.0,
                    align: Align::Center,
                    ..Default::default()
                },
                Some(*rid),
                Sense { click: true, drag: false, focusable: false, scroll: false },
                CursorIcon::Pointer,
                false,
                Some(Box::new(move |cx, rr| {
                    if hov_c.a > 0.0 {
                        cx.painter.rrect(rr, Corners::all(7.0), hov_c, 0.0, Color::TRANSPARENT);
                    }
                    for d in 0..depth {
                        let gx = rr.min.x + 17.0 + d as f32 * 16.0;
                        cx.painter.rect(Rect::from_xywh(gx, rr.min.y - 1.0, 1.0, rr.height() + 2.0), guide_c.multiply_alpha(0.75));
                    }
                })),
            );

            if r.node.is_branch() {
                let angle = chev_t * std::f32::consts::FRAC_PI_2;
                ui.leaf(
                    Style::default(),
                    vec2(14.0, 14.0),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, cr| draw_chevron(&mut cx.painter, cr.center(), 4.0, angle, 1.5, chev_c))),
                );
            } else {
                ui.space(14.0);
            }

            if let Some(ic) = &r.node.icon {
                let icon_name = if ic == "folder" && is_open { "folder-open" } else { ic.as_str() };
                ui.icon_leaf(icon_name, 15.0, ic_c);
            }

            let st_lbl = if is_sel { ui.text_style(TextRole::BodyStrong) } else { ui.text_style(TextRole::Body) };
            ui.text_leaf(r.node.label.clone(), st_lbl, fg_c, false, true, Style { width: Size::FILL, ..Default::default() });

            if let Some(meta) = &r.node.meta {
                let st_m = ui.text_style(TextRole::Caption);
                ui.text_leaf(meta.clone(), st_m, meta_c, false, false, Style::default());
            }

            ui.end();
        }

        ui.end();
        let rect = ui.rect_of(id).unwrap_or(ix.rect);
        Response { id, rect, hovered: ix.hovered, clicked: changed, changed, focused: ix.focused }
    }
}

// ================================================================ Timeline

/// Event entry in a vertical [`Timeline`].
#[derive(Clone)]
pub struct TimelineItem {
    pub title: String,
    pub time: String,
    pub description: Option<String>,
    pub tag: Option<String>,
    pub icon: Option<String>,
    pub tone: Tone,
}

impl TimelineItem {
    pub fn new(title: impl Into<String>, time: impl Into<String>) -> Self {
        TimelineItem {
            title: title.into(),
            time: time.into(),
            description: None,
            tag: None,
            icon: None,
            tone: Tone::Neutral,
        }
    }
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    pub fn tag(mut self, t: impl Into<String>) -> Self {
        self.tag = Some(t.into());
        self
    }
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = Some(name.to_owned());
        self
    }
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
}

fn tone_tokens(tone: Tone) -> (Token, Token, Token) {
    match tone {
        Tone::Neutral => (Token::TextMuted, Token::TextSecondary, Token::SurfaceMuted),
        Tone::Accent => (Token::Accent, Token::AccentStrong, Token::AccentSubtle),
        Tone::Success => (Token::Success, Token::SuccessStrong, Token::SuccessSubtle),
        Tone::Warning => (Token::Warning, Token::WarningStrong, Token::WarningSubtle),
        Tone::Info => (Token::Info, Token::InfoStrong, Token::InfoSubtle),
        Tone::Danger => (Token::Danger, Token::DangerStrong, Token::DangerSubtle),
    }
}

impl Ui {
    /// Single spring-height disclosure section.
    pub fn collapsible<'u>(&'u mut self, key: impl Into<String>, title: impl Into<String>, open: &'u mut bool) -> Collapsible<'u> {
        Collapsible {
            ui: self,
            key: key.into(),
            title: title.into(),
            open,
            icon: None,
            badge: None,
            carded: true,
            disabled: false,
        }
    }

    /// Single-open stacked accordion card.
    pub fn accordion<'u, 'a>(&'u mut self, key: impl Into<String>, open: &'u mut Option<usize>) -> Accordion<'u, 'a> {
        Accordion { ui: self, key: key.into(), open, items: vec![] }
    }

    /// Sortable, selectable data table.
    pub fn table(&mut self, key: impl Into<String>) -> Table<'_> {
        Table { ui: self, key: key.into(), cols: vec![], rows: vec![], sort: None, selected: None, clicked_row: None, lazy: None, height: None }
    }

    /// Expandable hierarchical tree view.
    pub fn tree<'u, 'a>(
        &'u mut self,
        key: impl Into<String>,
        nodes: &'a [TreeNode],
        expanded: &'u mut BTreeSet<String>,
        selected: &'u mut String,
    ) -> Tree<'u, 'a> {
        Tree { ui: self, key: key.into(), nodes, expanded, selected }
    }

    /// Vertical activity / deploy feed with status rings and connecting rails.
    pub fn timeline(&mut self, key: impl std::hash::Hash, items: &[TimelineItem]) -> Response {
        let id = self.make_id(("timeline", key));
        let mo = self.motion();
        let enter = self.anims.spring_from(id, 0, 1.0, 1.0, mo.gentle).clamp(0.0, 1.0);
        let rail_c = self.color(Token::BorderSubtle);

        self.begin(
            Style { width: Size::FILL, dir: Dir::Column, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );

        let last_idx = items.len().saturating_sub(1);
        for (i, item) in items.iter().enumerate() {
            let is_last = i == last_idx;
            let (dot_tok, strong_tok, sub_tok) = tone_tokens(item.tone);
            let dot_c = self.color(dot_tok).multiply_alpha(enter);
            let strong_c = self.color(strong_tok).multiply_alpha(enter);
            let sub_c = self.color(sub_tok).multiply_alpha(enter);
            let surf_c = self.color(Token::Surface);
            let rc = rail_c.multiply_alpha(enter);
            let fg_c = self.color(Token::Foreground).multiply_alpha(enter);
            let desc_c = self.color(Token::TextSecondary).multiply_alpha(enter);
            let time_c = self.color(Token::TextMuted).multiply_alpha(enter);
            let tag_bg = self.color(Token::SurfaceMuted).multiply_alpha(enter);
            let tag_bd = self.color(Token::BorderSubtle).multiply_alpha(enter);
            let icon_opt = item.icon.clone();

            self.begin(
                Style { dir: Dir::Row, width: Size::FILL, gap: 14.0, align: Align::Stretch, ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                None,
            );

            // Left rail + badge column.
            self.leaf(
                Style { self_align: Some(Align::Stretch), ..Default::default() },
                vec2(26.0, 28.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    let cx_x = r.center().x;
                    let cy = r.min.y + 13.0;
                    if !is_last {
                        let top = cy + 12.0;
                        if r.max.y > top {
                            cx.painter.rect(Rect::from_xywh(cx_x - 0.75, top, 1.5, r.max.y - top), rc);
                        }
                    }
                    if let Some(ic) = &icon_opt {
                        cx.painter.circle(vec2(cx_x, cy), 12.0, sub_c, 1.0, dot_c.multiply_alpha(0.35));
                        cx.icon(ic, vec2(cx_x, cy), 13.0, strong_c);
                    } else {
                        cx.painter.circle(vec2(cx_x, cy), 8.0, sub_c, 0.0, Color::TRANSPARENT);
                        cx.painter.circle(vec2(cx_x, cy), 4.5, dot_c, 1.5, surf_c);
                    }
                })),
            );

            // Right content column.
            let bottom_pad = if is_last { 2.0 } else { 18.0 };
            self.begin(
                Style {
                    dir: Dir::Column,
                    width: Size::FILL,
                    pad: Sides { top: 2.0, bottom: bottom_pad, left: 0.0, right: 0.0 },
                    gap: 4.0,
                    ..Default::default()
                },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                None,
            );

            self.begin(
                Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, align: Align::Center, ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                None,
            );
            let st_t = self.text_style(TextRole::BodyStrong);
            self.text_leaf(item.title.clone(), st_t, fg_c, false, true, Style { width: Size::FILL, ..Default::default() });
            if let Some(tag) = &item.tag {
                let mut st_tag = self.text_style(TextRole::Mono);
                st_tag.size = (st_tag.size - 1.0).max(10.0);
                self.begin(
                    Style { pad: Sides::xy(6.0, 1.5), ..Default::default() },
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(5.0), tag_bg, 1.0, tag_bd))),
                );
                self.text_leaf(tag.clone(), st_tag, desc_c, false, false, Style::default());
                self.end();
            }
            let st_time = self.text_style(TextRole::Caption);
            self.text_leaf(item.time.clone(), st_time, time_c, false, false, Style::default());
            self.end();

            if let Some(desc) = &item.description {
                let st_d = self.text_style(TextRole::Body);
                self.text_leaf(desc.clone(), st_d, desc_c, true, false, Style { width: Size::FILL, ..Default::default() });
            }

            self.end();
            self.end();
        }

        self.end();
        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, hovered: false, clicked: false, changed: false, focused: false }
    }
}
