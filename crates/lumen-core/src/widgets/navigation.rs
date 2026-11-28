//! Navigation widgets: [`Breadcrumb`] path trails with overflow menu,
//! [`Pagination`] bars with a sliding active pill, and [`Steps`] multi-step
//! wizards with spring-filled connectors.

use super::{key_activate, show_on_drop};
use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::id::Id;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{ControlSize, TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

fn linear_arrow_focus(ui: &mut Ui, me: Id, items: &[Id], vertical: bool) {
    let Some(i) = items.iter().position(|&x| x == me) else { return };
    let n = items.len();
    if n == 0 {
        return;
    }
    let next_k = if vertical { Key::ArrowDown } else { Key::ArrowRight };
    let prev_k = if vertical { Key::ArrowUp } else { Key::ArrowLeft };
    if ui.key_pressed_for(me, next_k) {
        ui.request_focus(items[(i + 1) % n]);
    } else if ui.key_pressed_for(me, prev_k) {
        ui.request_focus(items[(i + n - 1) % n]);
    } else if ui.key_pressed_for(me, Key::Home) {
        ui.request_focus(items[0]);
    } else if ui.key_pressed_for(me, Key::End) {
        ui.request_focus(items[n - 1]);
    }
}

// =============================================================== Breadcrumb

/// Trail of ancestor locations ending at the current page. Middle items can
/// collapse into an interactive `…` menu via [`.max_items`](Breadcrumb::max_items).
pub struct Breadcrumb<'u, 'a> {
    ui: &'u mut Ui,
    key: String,
    items: &'a [&'a str],
    root_icon: Option<String>,
    max_items: Option<usize>,
    slash: bool,
    shown: bool,
}

impl<'u, 'a> Breadcrumb<'u, 'a> {
    /// Leading Lucide icon on the first crumb (e.g. `"home"` or `"folder"`).
    pub fn root_icon(mut self, name: &str) -> Self {
        self.root_icon = Some(name.to_owned());
        self
    }
    /// Collapse middle crumbs into an interactive `…` menu when `items.len() > max` (`max >= 3`).
    pub fn max_items(mut self, max: usize) -> Self {
        self.max_items = Some(max.max(3));
        self
    }
    /// Use `/` separators instead of `chevron-right` icons.
    pub fn slash(mut self) -> Self {
        self.slash = true;
        self
    }
    /// Add the breadcrumb and return `Some(index)` if an ancestor crumb was clicked.
    pub fn show(mut self) -> Option<usize> {
        self.shown = true;
        self.show_inner()
    }

    fn show_inner(&mut self) -> Option<usize> {
        let n = self.items.len();
        if n == 0 {
            return None;
        }
        let ui = &mut *self.ui;
        let id = ui.make_id(("breadcrumb", &self.key));
        let mo = ui.motion();
        let mut clicked: Option<usize> = None;

        #[derive(Clone, Copy)]
        enum Slot {
            Crumb(usize),
            Ellipsis(usize, usize), // inclusive..exclusive range of collapsed indices
        }

        let slots: Vec<Slot> = match self.max_items {
            Some(max) if n > max && max >= 3 => {
                let tail = max - 2; // 1 for root + 1 for ellipsis + tail
                let hide_end = n - tail;
                let mut v = vec![Slot::Crumb(0), Slot::Ellipsis(1, hide_end)];
                for i in hide_end..n {
                    v.push(Slot::Crumb(i));
                }
                v
            }
            _ => (0..n).map(Slot::Crumb).collect(),
        };

        let focus_ids: Vec<Id> = slots
            .iter()
            .filter_map(|s| match *s {
                Slot::Crumb(i) if i + 1 < n => Some(id.with(("c", i))),
                Slot::Ellipsis(_, _) => Some(id.with("more")),
                _ => None,
            })
            .collect();

        ui.begin(Style { dir: Dir::Row, wrap: true, gap: 4.0, align: Align::Center, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);

        for (pos, slot) in slots.iter().copied().enumerate() {
            if pos > 0 {
                let sep_c = ui.color(Token::TextMuted);
                if self.slash {
                    let st = ui.text_style(TextRole::Body);
                    ui.text_leaf("/", st, sep_c, false, false, Style::default());
                } else {
                    ui.icon_leaf("chevron-right", 14.0, sep_c);
                }
            }
            match slot {
                Slot::Crumb(i) if i + 1 == n => {
                    // Current location (non-interactive, strong foreground).
                    let st = ui.text_style(TextRole::BodyStrong);
                    let fg = ui.color(Token::Foreground);
                    ui.begin(Style { dir: Dir::Row, height: Size::Fixed(26.0), pad: Sides::xy(6.0, 0.0), gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                    if i == 0 {
                        if let Some(ic) = &self.root_icon {
                            ui.icon_leaf(ic, 14.0, fg);
                        }
                    }
                    ui.text_leaf(self.items[i], st, fg, false, true, Style::default());
                    ui.end();
                }
                Slot::Crumb(i) => {
                    let cid = id.with(("c", i));
                    let ix = ui.interact(cid, Sense::CLICK, false);
                    if ix.focused {
                        linear_arrow_focus(ui, cid, &focus_ids, false);
                    }
                    let (k_act, k_held) = key_activate(ui, &ix, true);
                    if ix.clicked || k_act {
                        clicked = Some(i);
                    }
                    let h = ui.fade(cid, 0, ix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
                    let press = ui.spring(cid, 1, if ix.pressed || k_held { 1.0 } else { 0.0 }, mo.snappy);
                    let (ring_a, ring_gap) = ui.focus_anim(&ix);
                    let hover_bg = ui.color(Token::Hover).multiply_alpha(h);
                    let fg = ui.color(Token::TextSecondary).mix(ui.color(Token::Foreground), h);
                    let ring = ui.color(Token::FocusRing);
                    let rad = (ui.look().control.radius[0]).max(4.0);
                    let st = ui.text_style(TextRole::Body);

                    ui.begin(
                        Style { dir: Dir::Row, height: Size::Fixed(26.0), pad: Sides::xy(7.0, 0.0), gap: 5.0, align: Align::Center, ..Default::default() },
                        Some(cid),
                        Sense::CLICK,
                        CursorIcon::Pointer,
                        false,
                        Some(Box::new(move |cx, r| {
                            let sr = Rect::from_center_size(r.center(), r.size() * (1.0 - 0.04 * press));
                            if hover_bg.a > 0.0 {
                                cx.painter.rrect_unsnapped(sr, Corners::all(rad), [hover_bg; 2], 0.0, Color::TRANSPARENT);
                            }
                            if ring_a > 0.0 {
                                cx.defer(sr, Box::new(move |cx, r| cx.focus_ring(r, rad, ring_a, ring_gap, ring)));
                            }
                        })),
                    );
                    if i == 0 {
                        if let Some(ic) = &self.root_icon {
                            ui.icon_leaf(ic, 14.0, fg);
                        }
                    }
                    ui.text_leaf(self.items[i], st, fg, false, true, Style::default());
                    ui.end();
                }
                Slot::Ellipsis(a, b) => {
                    let mid = id.with("more");
                    let ix = ui.interact(mid, Sense::CLICK, false);
                    if ix.focused {
                        linear_arrow_focus(ui, mid, &focus_ids, false);
                    }
                    let (k_act, k_held) = key_activate(ui, &ix, true);
                    let h = ui.fade(mid, 0, ix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
                    let press = ui.spring(mid, 1, if ix.pressed || k_held { 1.0 } else { 0.0 }, mo.snappy);
                    let (ring_a, ring_gap) = ui.focus_anim(&ix);
                    let bg = ui.color(Token::SurfaceMuted).mix(ui.color(Token::SurfaceMuted).over(ui.color(Token::Hover)), h);
                    let bd = ui.color(Token::BorderSubtle);
                    let ic = ui.color(Token::TextSecondary).mix(ui.color(Token::Foreground), h);
                    let ring = ui.color(Token::FocusRing);
                    let rad = (ui.look().control.radius[0]).max(4.0);
                    ui.leaf(
                        Style::default(),
                        vec2(26.0, 22.0),
                        Some(mid),
                        Sense::CLICK,
                        CursorIcon::Pointer,
                        Some(Box::new(move |cx, r| {
                            let sr = Rect::from_center_size(r.center(), r.size() * (1.0 - 0.05 * press));
                            cx.painter.rrect_unsnapped(sr, Corners::all(rad), [bg; 2], 1.0, bd);
                            cx.icon("ellipsis", sr.center(), 14.0, ic);
                            if ring_a > 0.0 {
                                cx.defer(sr, Box::new(move |cx, r| cx.focus_ring(r, rad, ring_a, ring_gap, ring)));
                            }
                        })),
                    );
                    let anchor = Response { id: mid, rect: ui.rect_of(mid).unwrap_or(ix.rect), hovered: ix.hovered, clicked: ix.clicked || k_act, changed: false, focused: ix.focused };
                    let items = self.items;
                    ui.menu(&anchor, |ui| {
                        for k in a..b {
                            if ui.menu_item("folder", items[k]).clicked() {
                                clicked = Some(k);
                            }
                        }
                    });
                }
            }
        }
        ui.end();
        clicked
    }
}

impl Drop for Breadcrumb<'_, '_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

// =============================================================== Pagination

#[derive(Clone, Copy)]
enum PageItem {
    Page(usize),
    Ellipsis,
}

fn page_items(current: usize, total: usize, siblings: usize) -> Vec<PageItem> {
    let window = siblings * 2 + 5;
    if total <= window {
        return (0..total).map(PageItem::Page).collect();
    }
    let left = current.saturating_sub(siblings).max(1);
    let right = (current + siblings).min(total - 2);
    let show_left_dots = left > 2;
    let show_right_dots = right < total - 3;

    let mut out = vec![PageItem::Page(0)];
    match (show_left_dots, show_right_dots) {
        (false, true) => {
            let end = (3 + 2 * siblings).min(total - 2);
            for p in 1..=end {
                out.push(PageItem::Page(p));
            }
            out.push(PageItem::Ellipsis);
        }
        (true, false) => {
            out.push(PageItem::Ellipsis);
            let start = total.saturating_sub(4 + 2 * siblings).max(1);
            for p in start..total - 1 {
                out.push(PageItem::Page(p));
            }
        }
        (true, true) => {
            out.push(PageItem::Ellipsis);
            for p in left..=right {
                out.push(PageItem::Page(p));
            }
            out.push(PageItem::Ellipsis);
        }
        (false, false) => {
            for p in 1..total - 1 {
                out.push(PageItem::Page(p));
            }
        }
    }
    out.push(PageItem::Page(total - 1));
    out
}

/// Page switcher (`0..total`, displayed `1..=total`) with a sliding accent
/// indicator, smart ellipsis window, optional item-range caption and compact
/// rolling-number mode.
pub struct Pagination<'u> {
    ui: &'u mut Ui,
    key: String,
    page: &'u mut usize,
    total: usize,
    siblings: usize,
    compact: bool,
    summary: Option<(usize, usize)>,
    shown: bool,
}

impl<'u> Pagination<'u> {
    /// Neighboring page buttons shown on each side of the current page (default `1`).
    pub fn siblings(mut self, n: usize) -> Self {
        self.siblings = n.min(3);
        self
    }
    /// Compact `← Page n of total →` layout with a vertically rolling page number.
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }
    /// Show `"a–b of total_items"` alongside the page controls.
    pub fn summary(mut self, total_items: usize, per_page: usize) -> Self {
        self.summary = Some((total_items, per_page.max(1)));
        self
    }

    fn show_inner(&mut self) -> Response {
        let total = self.total.max(1);
        *self.page = (*self.page).min(total - 1);
        let before = *self.page;

        let ui = &mut *self.ui;
        let id = ui.make_id(("pagination", &self.key));
        let strip_id = id.with("strip");
        let prev_id = id.with("prev");
        let next_id = id.with("next");
        let mo = ui.motion();
        let m = ui.control_metrics(ControlSize::Sm);
        let h_px = m.height.max(30.0);
        let rad = m.radius;

        let has_summary = self.summary.is_some();
        if has_summary {
            ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 12.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            if let Some((items, per)) = self.summary {
                let a = if items == 0 { 0 } else { *self.page * per + 1 };
                let b = ((*self.page + 1) * per).min(items);
                let st = ui.text_style(TextRole::Label);
                let mut st = st;
                st.weight = 400;
                let c = ui.color(Token::TextMuted);
                ui.text_leaf(format!("{a}–{b} of {items}"), st, c, false, false, Style::default());
                ui.spacer();
            }
        }

        let prev_dis = *self.page == 0;
        let next_dis = *self.page + 1 >= total;
        let prev_ix = ui.interact(prev_id, Sense::CLICK, prev_dis);
        ui.a11y(prev_id, || A11y::new(Role::Button).label("Previous page").disabled(prev_dis));
        ui.a11y(next_id, || A11y::new(Role::Button).label("Next page").disabled(next_dis));
        ui.a11y(strip_id, || A11y::new(Role::Navigation).label("Pagination"));
        let (pk_act, pk_held) = key_activate(ui, &prev_ix, true);

        let mut next_page = *self.page;
        let mut any_hovered = prev_ix.hovered;
        let mut any_focused = prev_ix.focused;

        if self.compact {
            let next_ix = ui.interact(next_id, Sense::CLICK, next_dis);
            let (nk_act, nk_held) = key_activate(ui, &next_ix, true);
            any_hovered |= next_ix.hovered;
            any_focused |= next_ix.focused;
            if (prev_ix.clicked || pk_act) && !prev_dis {
                next_page = next_page.saturating_sub(1);
            }
            if (next_ix.clicked || nk_act) && !next_dis {
                next_page = (next_page + 1).min(total - 1);
            }
            if any_focused {
                if ui.key_pressed(Key::ArrowLeft) {
                    next_page = next_page.saturating_sub(1);
                } else if ui.key_pressed(Key::ArrowRight) {
                    next_page = (next_page + 1).min(total - 1);
                } else if ui.key_pressed(Key::Home) {
                    next_page = 0;
                } else if ui.key_pressed(Key::End) {
                    next_page = total - 1;
                }
            }
            if next_page != before {
                let dir = if next_page > before { -1.0 } else { 1.0 };
                ui.anims.set_spring(id, 9, dir);
            }
            let roll = ui.anims.spring_from(id, 9, 0.0, 0.0, mo.bouncy);
            *self.page = next_page;

            ui.begin(Style { dir: Dir::Row, gap: 6.0, align: Align::Center, ..Default::default() }, Some(strip_id), Sense::NONE, CursorIcon::Default, false, None);
            nav_arrow_btn(ui, prev_id, &prev_ix, pk_held, "chevron-left", h_px, rad);
            let st = ui.text_style(TextRole::Body);
            let fg = ui.color(Token::Foreground);
            let txt = format!("Page {} of {total}", *self.page + 1);
            let g = ui.galley(&txt, &st);
            ui.leaf(
                Style { min: vec2((g.width + 16.0).max(96.0), 0.0), ..Default::default() },
                vec2((g.width + 16.0).max(96.0), h_px),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    cx.painter.push_clip(r);
                    let y = r.center().y - g.height / 2.0 + roll * 9.0;
                    let a = (1.0 - roll.abs() * 0.85).clamp(0.0, 1.0);
                    cx.painter.text(cx.text, &g, vec2((r.center().x - g.width / 2.0).round(), y.round()), fg.multiply_alpha(a));
                    cx.painter.pop_clip();
                })),
            );
            nav_arrow_btn(ui, next_id, &next_ix, nk_held, "chevron-right", h_px, rad);
            ui.end();
        } else {
            let items = page_items(*self.page, total, self.siblings);
            let mut page_ixs = Vec::with_capacity(items.len());
            let mut focus_ring_ids = Vec::new();
            if !prev_dis {
                focus_ring_ids.push(prev_id);
            }
            for it in &items {
                if let PageItem::Page(p) = *it {
                    let pid = id.with(("p", p));
                    focus_ring_ids.push(pid);
                    page_ixs.push((p, ui.interact(pid, Sense::CLICK, false)));
                    let cur = p == *self.page;
                    ui.a11y(pid, || A11y::new(Role::Button).label(format!("Page {}", p + 1)).selected(cur).position(p, total));
                }
            }
            let next_ix = ui.interact(next_id, Sense::CLICK, next_dis);
            let (nk_act, nk_held) = key_activate(ui, &next_ix, true);
            any_hovered |= next_ix.hovered;
            any_focused |= next_ix.focused;
            if !next_dis {
                focus_ring_ids.push(next_id);
            }
            if (prev_ix.clicked || pk_act) && !prev_dis {
                next_page = next_page.saturating_sub(1);
            }
            if (next_ix.clicked || nk_act) && !next_dis {
                next_page = (next_page + 1).min(total - 1);
            }

            for &(p, ref pix) in &page_ixs {
                any_hovered |= pix.hovered;
                any_focused |= pix.focused;
                let (act, _) = key_activate(ui, pix, true);
                if pix.clicked || act {
                    next_page = p;
                }
            }
            if any_focused {
                if ui.key_pressed(Key::ArrowLeft) {
                    next_page = (*self.page).saturating_sub(1);
                    ui.request_focus(id.with(("p", next_page)));
                } else if ui.key_pressed(Key::ArrowRight) {
                    next_page = (*self.page + 1).min(total - 1);
                    ui.request_focus(id.with(("p", next_page)));
                } else if ui.key_pressed(Key::Home) {
                    next_page = 0;
                    ui.request_focus(id.with(("p", 0usize)));
                } else if ui.key_pressed(Key::End) {
                    next_page = total - 1;
                    ui.request_focus(id.with(("p", total - 1)));
                }
            }
            let _ = focus_ring_ids;
            *self.page = next_page;

            let sel_id = id.with(("p", *self.page));
            let pill_key = id.with("pill");
            let accent = ui.color(Token::Accent);
            let reduced = ui.reduced_motion();
            let snappy = mo.snappy;

            ui.begin(
                Style { dir: Dir::Row, gap: 4.0, align: Align::Center, ..Default::default() },
                Some(strip_id),
                Sense::NONE,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, _r| {
                    let Some(sr) = cx.rects.get(&sel_id).copied() else { return };
                    let (x, w) = if reduced {
                        (sr.min.x, sr.width())
                    } else {
                        let first = cx.anims.spring_state(pill_key, 0).is_none();
                        let x = cx.anims.spring_from(pill_key, 0, sr.min.x, sr.min.x, snappy);
                        let w = cx.anims.spring_from(pill_key, 1, sr.width(), sr.width(), snappy);
                        if first { (sr.min.x, sr.width()) } else { (x, w) }
                    };
                    let pr = Rect::from_xywh(x, sr.min.y, w, sr.height());
                    cx.painter.rrect_unsnapped(pr, Corners::all(rad), [accent; 2], 0.0, Color::TRANSPARENT);
                })),
            );

            nav_arrow_btn(ui, prev_id, &prev_ix, pk_held, "chevron-left", h_px, rad);

            let mut pix_iter = page_ixs.into_iter();
            for it in items {
                match it {
                    PageItem::Ellipsis => {
                        let c = ui.color(Token::TextMuted);
                        ui.leaf(
                            Style::default(),
                            vec2(h_px * 0.85, h_px),
                            None,
                            Sense::NONE,
                            CursorIcon::Default,
                            Some(Box::new(move |cx, r| cx.icon("ellipsis", r.center(), 14.0, c))),
                        );
                    }
                    PageItem::Page(p) => {
                        let (_, pix) = pix_iter.next().expect("page ix");
                        let (_, p_held) = key_activate(ui, &pix, true);
                        let selected = p == *self.page;
                        let sel_t = ui.fade(pix.id, 0, selected, mo.selected, mo.selected, Ease::Out);
                        let hv = ui.fade(pix.id, 1, pix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
                        let press = ui.spring(pix.id, 2, if pix.pressed || p_held { 1.0 } else { 0.0 }, mo.snappy);
                        let (ring_a, ring_gap) = ui.focus_anim(&pix);
                        let hover_bg = ui.color(Token::Hover).multiply_alpha(hv * (1.0 - sel_t));
                        let border = ui.color(Token::BorderSubtle).multiply_alpha((1.0 - sel_t) * 0.7);
                        let fg = ui
                            .color(Token::TextSecondary)
                            .mix(ui.color(Token::Foreground), hv)
                            .mix(ui.color(Token::AccentForeground), sel_t);
                        let ring = ui.color(Token::FocusRing);
                        let mut st = ui.text_style(TextRole::Label);
                        st.weight = if selected { 600 } else { 500 };
                        let label = (p + 1).to_string();
                        let g = ui.galley(&label, &st);
                        let w = (g.width + 16.0).max(h_px);
                        ui.leaf(
                            Style::default(),
                            vec2(w, h_px),
                            Some(pix.id),
                            Sense::CLICK,
                            CursorIcon::Pointer,
                            Some(Box::new(move |cx, r| {
                                let sr = Rect::from_center_size(r.center(), r.size() * (1.0 - 0.05 * press));
                                if hover_bg.a > 0.0 || border.a > 0.0 {
                                    cx.painter.rrect_unsnapped(sr, Corners::all(rad), [hover_bg; 2], 1.0, border);
                                }
                                cx.painter.text(cx.text, &g, vec2((sr.center().x - g.width / 2.0).round(), (sr.center().y - g.height / 2.0).round()), fg);
                                if ring_a > 0.0 {
                                    cx.defer(sr, Box::new(move |cx, r| cx.focus_ring(r, rad, ring_a, ring_gap, ring)));
                                }
                            })),
                        );
                    }
                }
            }

            nav_arrow_btn(ui, next_id, &next_ix, nk_held, "chevron-right", h_px, rad);
            ui.end();
        }

        if has_summary {
            ui.end();
        }

        let changed = *self.page != before;
        let rect = ui.rect_of(strip_id).unwrap_or_default();
        Response { id, rect, hovered: any_hovered, clicked: changed, changed, focused: any_focused }
    }
}

fn nav_arrow_btn(ui: &mut Ui, id: Id, ix: &crate::ui::Interact, key_held: bool, icon: &'static str, size: f32, rad: f32) {
    let mo = ui.motion();
    let h = ui.fade(id, 1, ix.hovered && !ix.disabled, mo.hover_in, mo.hover_out, Ease::Linear);
    let press = ui.spring(id, 2, if (ix.pressed || key_held) && !ix.disabled { 1.0 } else { 0.0 }, mo.snappy);
    let (ring_a, ring_gap) = ui.focus_anim(ix);
    let bg = ui.color(Token::Control).mix(ui.color(Token::Control).over(ui.color(Token::Hover)), h);
    let bd = if ix.disabled { ui.color(Token::BorderSubtle) } else { ui.color(Token::ControlBorder).mix(ui.color(Token::BorderStrong), h) };
    let ic = if ix.disabled {
        ui.color(Token::TextDisabled)
    } else {
        ui.color(Token::TextSecondary).mix(ui.color(Token::Foreground), h)
    };
    let ring = ui.color(Token::FocusRing);
    ui.leaf(
        Style::default(),
        vec2(size, size),
        Some(id),
        Sense::CLICK,
        if ix.disabled { CursorIcon::Default } else { CursorIcon::Pointer },
        Some(Box::new(move |cx, r| {
            let sr = Rect::from_center_size(r.center(), r.size() * (1.0 - 0.05 * press));
            cx.painter.rrect_unsnapped(sr, Corners::all(rad), [bg; 2], 1.0, bd);
            cx.icon(icon, sr.center(), 15.0, ic);
            if ring_a > 0.0 {
                cx.defer(sr, Box::new(move |cx, r| cx.focus_ring(r, rad, ring_a, ring_gap, ring)));
            }
        })),
    );
}

show_on_drop!(Pagination);

// ==================================================================== Steps

#[derive(Default)]
struct StepsMem {
    max_reached: usize,
}

/// Multi-step wizard header or vertical checklist (`0..steps.len()`). Completed
/// steps grow a check on the bouncy spring, and connector lines fill on the
/// smooth spring as you advance.
pub struct Steps<'u, 'a> {
    ui: &'u mut Ui,
    key: String,
    current: &'u mut usize,
    steps: &'a [(&'a str, &'a str)],
    linear: bool,
    vertical: bool,
    shown: bool,
}

impl<'u, 'a> Steps<'u, 'a> {
    /// Only allow clicking steps up to the furthest step reached so far.
    pub fn linear(mut self, on: bool) -> Self {
        self.linear = on;
        self
    }
    /// Stack steps vertically instead of across a horizontal bar.
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }

    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }
    pub fn changed(self) -> bool {
        self.show().changed
    }

    fn show_inner(&mut self) -> Response {
        let n = self.steps.len();
        let ui = &mut *self.ui;
        let id = ui.make_id(("steps", &self.key));
        if n == 0 {
            return Response { id, ..Default::default() };
        }
        *self.current = (*self.current).min(n - 1);
        let before = *self.current;
        let max_reached = {
            let st = ui.mem::<StepsMem>(id);
            st.max_reached = st.max_reached.max(*self.current);
            st.max_reached
        };
        let mo = ui.motion();

        let mut step_ixs = Vec::with_capacity(n);
        let mut focus_ids = Vec::new();
        for i in 0..n {
            let sid = id.with(("s", i));
            let dis = self.linear && i > max_reached;
            if !dis {
                focus_ids.push(sid);
            }
            step_ixs.push(ui.interact(sid, Sense::CLICK, dis));
        }

        let mut next = *self.current;
        let mut any_hovered = false;
        let mut any_focused = false;
        for (i, ix) in step_ixs.iter().enumerate() {
            any_hovered |= ix.hovered;
            any_focused |= ix.focused;
            if ix.focused {
                linear_arrow_focus(ui, ix.id, &focus_ids, self.vertical);
            }
            let (k_act, _) = key_activate(ui, ix, true);
            if (ix.clicked || k_act) && !ix.disabled {
                next = i;
            }
        }
        *self.current = next;
        ui.mem::<StepsMem>(id).max_reached = max_reached.max(*self.current);

        let dir = if self.vertical { Dir::Column } else { Dir::Row };
        let align = if self.vertical { Align::Start } else { Align::Center };
        ui.begin(Style { dir, width: Size::FILL, gap: if self.vertical { 6.0 } else { 8.0 }, align, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);

        for (i, &(title, sub)) in self.steps.iter().enumerate() {
            let ix = &step_ixs[i];
            let (_, k_held) = key_activate(ui, ix, true);
            let done = i < *self.current;
            let active = i == *self.current;

            let done_t = ui.spring(ix.id, 0, if done { 1.0 } else { 0.0 }, mo.bouncy);
            let act_t = ui.fade(ix.id, 1, active, mo.selected, mo.selected, Ease::Out);
            let hv = ui.fade(ix.id, 2, ix.hovered && !ix.disabled, mo.hover_in, mo.hover_out, Ease::Linear);
            let press = ui.spring(ix.id, 3, if (ix.pressed || k_held) && !ix.disabled { 1.0 } else { 0.0 }, mo.snappy);
            let (ring_a, ring_gap) = ui.focus_anim(ix);

            let c = |t| ui.color(t);
            let accent = c(Token::Accent);
            let halo = c(Token::FocusHalo);
            let badge_fill = c(Token::Surface)
                .mix(c(Token::Control).over(c(Token::Hover)), hv * (1.0 - done_t.clamp(0.0, 1.0)))
                .mix(accent, done_t.clamp(0.0, 1.0));
            let badge_bd = c(Token::BorderStrong).mix(accent, act_t.max(done_t.clamp(0.0, 1.0)));
            let num_c = if ix.disabled {
                c(Token::TextDisabled)
            } else {
                c(Token::TextMuted).mix(c(Token::Foreground), hv).mix(c(Token::AccentStrong), act_t)
            };
            let check_c = c(Token::AccentForeground);
            let ring = c(Token::FocusRing);
            let pill_rad = (ui.look().control.radius[1]).max(6.0);

            let mut num_st = ui.text_style(TextRole::Label);
            num_st.weight = 600;
            let num_g = ui.galley(&(i + 1).to_string(), &num_st);

            ui.begin(
                Style { dir: Dir::Row, pad: Sides::xy(6.0, 4.0), gap: 10.0, align: Align::Center, ..Default::default() },
                Some(ix.id),
                Sense::CLICK,
                if ix.disabled { CursorIcon::Default } else { CursorIcon::Pointer },
                false,
                Some(Box::new(move |cx, r| {
                    if ring_a > 0.0 {
                        cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, pill_rad, ring_a, ring_gap, ring)));
                    }
                })),
            );

            ui.leaf(
                Style::default(),
                vec2(26.0, 26.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    let s = 1.0 - 0.06 * press;
                    let cen = r.center();
                    let rad = 13.0 * s;
                    if act_t > 0.01 {
                        let hw = 3.0 * act_t;
                        cx.painter.ring(Rect::from_center_size(cen, vec2((rad + hw) * 2.0, (rad + hw) * 2.0)), Corners::all(rad + hw), hw, halo.multiply_alpha(act_t));
                    }
                    let bw = 1.0 + act_t;
                    cx.painter.circle(cen, rad, badge_fill, bw, badge_bd);
                    let dt = done_t.clamp(0.0, 1.0);
                    if done_t > 0.02 {
                        cx.icon("check", cen, 14.0 * done_t.max(0.0), check_c.multiply_alpha(dt));
                    }
                    if dt < 0.98 {
                        cx.painter.text(cx.text, &num_g, vec2((cen.x - num_g.width / 2.0).round(), (cen.y - num_g.height / 2.0).round()), num_c.multiply_alpha(1.0 - dt));
                    }
                })),
            );

            ui.begin(Style { dir: Dir::Column, gap: 1.0, justify: Justify::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let st_t = ui.text_style(if active { TextRole::BodyStrong } else { TextRole::Body });
            let tc = if ix.disabled {
                ui.color(Token::TextDisabled)
            } else if active || done {
                ui.color(Token::Foreground)
            } else {
                ui.color(Token::TextSecondary).mix(ui.color(Token::Foreground), hv)
            };
            ui.text_leaf(title, st_t, tc, false, true, Style::default());
            if !sub.is_empty() {
                let st_s = ui.text_style(TextRole::Caption);
                let sc = if ix.disabled { ui.color(Token::TextDisabled) } else { ui.color(Token::TextMuted) };
                ui.text_leaf(sub, st_s, sc, false, true, Style::default());
            }
            ui.end();
            ui.end();

            // Connector wire between step `i` and `i + 1`.
            if i + 1 < n {
                let fill_t = ui.spring(id, 30 + i as u32, if *self.current > i { 1.0 } else { 0.0 }, mo.smooth).clamp(0.0, 1.0);
                let track = ui.color(Token::Track);
                let acc = ui.color(Token::Accent);
                if self.vertical {
                    ui.begin(Style { pad: Sides { left: 18.0, ..Default::default() }, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                    ui.leaf(
                        Style::default(),
                        vec2(2.0, 18.0),
                        None,
                        Sense::NONE,
                        CursorIcon::Default,
                        Some(Box::new(move |cx, r| {
                            cx.painter.rrect(r, Corners::all(1.0), track, 0.0, Color::TRANSPARENT);
                            if fill_t > 0.01 {
                                let fr = Rect::from_xywh(r.min.x, r.min.y, r.width(), r.height() * fill_t);
                                cx.painter.rrect_unsnapped(fr, Corners::all(1.0), [acc; 2], 0.0, Color::TRANSPARENT);
                            }
                        })),
                    );
                    ui.end();
                } else {
                    ui.leaf(
                        Style { width: Size::FILL, min: vec2(16.0, 2.0), ..Default::default() },
                        vec2(24.0, 2.0),
                        None,
                        Sense::NONE,
                        CursorIcon::Default,
                        Some(Box::new(move |cx, r| {
                            let line = Rect::from_xywh(r.min.x, r.center().y - 1.0, r.width(), 2.0);
                            cx.painter.rrect(line, Corners::all(1.0), track, 0.0, Color::TRANSPARENT);
                            if fill_t > 0.01 {
                                let fr = Rect::from_xywh(line.min.x, line.min.y, line.width() * fill_t, line.height());
                                cx.painter.rrect_unsnapped(fr, Corners::all(1.0), [acc; 2], 0.0, Color::TRANSPARENT);
                            }
                        })),
                    );
                }
            }
        }
        ui.end();

        let rect = ui.rect_of(id).unwrap_or_default();
        let changed = *self.current != before;
        Response { id, rect, hovered: any_hovered, clicked: changed, changed, focused: any_focused }
    }
}

impl Drop for Steps<'_, '_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

impl Ui {
    /// Path trail ending at the current view (`items.last()`).
    pub fn breadcrumb<'u, 'a>(&'u mut self, key: impl Into<String>, items: &'a [&'a str]) -> Breadcrumb<'u, 'a> {
        Breadcrumb { ui: self, key: key.into(), items, root_icon: None, max_items: None, slash: false, shown: false }
    }

    /// Page switcher for `total` pages (`*page` in `0..total`).
    pub fn pagination<'u>(&'u mut self, key: impl Into<String>, page: &'u mut usize, total: usize) -> Pagination<'u> {
        Pagination { ui: self, key: key.into(), page, total, siblings: 1, compact: false, summary: None, shown: false }
    }

    /// Multi-step wizard (`steps` of `(title, subtitle)` pairs, `*current` in `0..steps.len()`).
    pub fn steps<'u, 'a>(&'u mut self, key: impl Into<String>, current: &'u mut usize, steps: &'a [(&'a str, &'a str)]) -> Steps<'u, 'a> {
        Steps { ui: self, key: key.into(), current, steps, linear: false, vertical: false, shown: false }
    }
}
