//! Combobox (type to filter, pick one) and multi-select (chips inside the
//! field), sharing an option list with a sliding highlight, staggered rows
//! and highlighted matches.

use std::ops::Range;

use super::field::{fuzzy_match, match_score, paint_highlighted, Chrome, EditOpts};
use super::popover::PopState;
use crate::anim::{Ease, SpringParams};
use crate::id::Id;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides, Vec2};
use crate::theme::{ControlSize, TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

/// The leading mark of a list row.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Mark {
    None,
    /// A check when selected (single choice).
    Check(bool),
    /// A checkbox (multiple choice).
    Box(bool),
}

/// One row of an option list.
pub(crate) struct Row<'a> {
    /// Stable key (the option's index), so rows keep their animation state
    /// while the filter changes.
    pub key: usize,
    pub label: &'a str,
    pub hl: Vec<Range<usize>>,
    pub mark: Mark,
    pub icon: Option<&'a str>,
    pub hint: Option<&'a str>,
}

#[derive(Default)]
struct ListMem {
    pointer: Option<Vec2>,
    offset: f32,
    last_active: Option<usize>,
}

/// Rows drawn by [`Ui::option_list`]: (clicked, hovered-with-pointer-motion).
pub(crate) struct ListOut {
    pub clicked: Option<usize>,
    pub hovered: Option<usize>,
}

impl Ui {
    /// Height of one option row in the current look.
    pub(crate) fn option_row_height(&self) -> f32 {
        (self.look().control.height[1] - 4.0).max(28.0)
    }

    /// A list of options with a highlight pill that slides to `active`.
    /// Rows don't take focus: the owner keeps focus (usually in an editor)
    /// and moves `active` with the arrow keys. Long lists scroll and keep
    /// the active row in view.
    pub(crate) fn option_list(&mut self, id: Id, rows: &[Row<'_>], active: Option<usize>, max_rows: usize) -> ListOut {
        let mo = self.motion();
        let ch = self.option_row_height();
        let gap = 1.0;
        let n = rows.len();
        let pointer = self.pointer();
        let moved = {
            let m = self.mem::<ListMem>(id);
            let mv = m.pointer.is_some() && m.pointer != pointer;
            m.pointer = pointer;
            mv
        };
        let scrolls = n > max_rows;
        let view_h = max_rows.min(n) as f32 * (ch + gap) - gap;
        // Keep the active row in view when it moves.
        let scroll_id = self.scroll_id((id, "list"));
        if scrolls {
            let m = self.mem::<ListMem>(id);
            if active != m.last_active {
                if let Some(a) = active {
                    let top = a as f32 * (ch + gap);
                    let off = m.offset;
                    let target = if top < off { Some(top) } else if top + ch > off + view_h { Some(top + ch - view_h) } else { None };
                    if let Some(t) = target {
                        self.scroll_to(scroll_id, t);
                    }
                }
            }
        }
        self.mem::<ListMem>(id).last_active = active;
        let inner_id = id.with("rows");
        let row_ids: Vec<Id> = rows.iter().map(|r| id.with(("row", r.key))).collect();
        let inner_prev = self.rect_of(inner_id);
        let target = active.and_then(|a| Some((self.rect_of(row_ids[a])?, inner_prev?))).map(|(r, p)| r.min.y - p.min.y);
        let hl_a = self.fade(id, 51, target.is_some(), mo.hover_in, mo.hover_out, Ease::Linear);
        let y = match target {
            Some(t) => self.spring(id, 50, t, mo.snappy),
            None => self.anims.spring_state(id, 50).map(|s| s.value).unwrap_or(0.0),
        };
        if target.is_none() {
            // Keep the spring alive (and in place) while the pill fades.
            let v = self.anims.spring_state(id, 50).map(|s| s.value).unwrap_or(0.0);
            self.anims.set_spring(id, 50, v);
        }
        let mac = self.look().platform == crate::look::Platform::Mac;
        let pill = if mac { self.color(Token::Accent) } else { self.color(Token::Hover).over(self.color(Token::Hover)) };
        let radius = (self.look().control.radius[0] - 1.0).max(2.0);

        let mut out = ListOut { clicked: None, hovered: None };
        let body = self.text_style(TextRole::Body);
        let fg = self.color(Token::Foreground);
        let fg_hot = if mac { self.color(Token::AccentForeground) } else { fg };
        let hl_c = if mac { fg_hot } else { self.color(Token::AccentStrong) };
        let tint = if mac { Color::TRANSPARENT } else { self.color(Token::AccentSubtle).multiply_alpha(0.7) };
        let check_c = self.color(Token::AccentStrong);
        let accent = self.color(Token::Accent);
        let afg = self.color(Token::AccentForeground);
        let box_border = self.color(Token::BorderStrong);
        let muted = self.color(Token::TextMuted);
        let icon_c = self.color(Token::TextSecondary);

        let build = |ui: &mut Ui, out: &mut ListOut| {
            ui.begin(
                Style { width: Size::FILL, gap, ..Default::default() },
                Some(inner_id),
                Sense::NONE,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| {
                    if hl_a > 0.0 {
                        let pr = Rect::from_xywh(r.min.x, r.min.y + y, r.width(), ch);
                        cx.painter.rrect_unsnapped(pr, Corners::all(radius), [pill.multiply_alpha(hl_a); 2], 0.0, Color::TRANSPARENT);
                    }
                })),
            );
            for (i, row) in rows.iter().enumerate() {
                let rid = row_ids[i];
                let ix = ui.interact(rid, Sense { click: true, drag: false, focusable: false, scroll: false }, false);
                if ix.clicked {
                    out.clicked = Some(i);
                }
                if ix.hovered && moved {
                    out.hovered = Some(i);
                }
                let hot = active == Some(i);
                let hh = ui.fade(rid, 1, hot, 0.0, mo.hover_out, Ease::Linear);
                // Rows arrive one after another.
                let enter = ui.anims.spring_from(rid, 2, 0.0, 1.0, SpringParams::from_response(0.2 + 0.035 * i.min(8) as f32, 1.0)).clamp(0.0, 1.0);
                let on = matches!(row.mark, Mark::Check(true) | Mark::Box(true));
                let tick = ui.anims.spring_from(rid, 3, 0.0, if on { 1.0 } else { 0.0 }, mo.bouncy).max(0.0);
                let press = ui.spring(rid, 4, if ix.pressed { 1.0 } else { 0.0 }, mo.snappy);
                let text_c = fg.mix(fg_hot, hh).multiply_alpha(enter);
                let label = row.label.to_owned();
                let hl = row.hl.clone();
                let mark = row.mark;
                let (hl_c, tint) = (hl_c.mix(fg_hot, if mac { hh } else { 0.0 }), tint.multiply_alpha(1.0 - hh * if mac { 1.0 } else { 0.0 }));
                let icon = row.icon.map(str::to_owned);
                let hint = row.hint.map(str::to_owned);
                let ic = icon_c.mix(fg_hot, if mac { hh } else { 0.0 }).multiply_alpha(enter);
                let hint_c = muted.mix(fg_hot.multiply_alpha(0.8), if mac { hh } else { 0.0 }).multiply_alpha(enter);
                let g = ui.galley(&label, &body);
                ui.begin(
                    Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(ch), pad: Sides::xy(8.0, 0.0), gap: 8.0, align: Align::Center, ..Default::default() },
                    Some(rid),
                    Sense { click: true, drag: false, focusable: false, scroll: false },
                    CursorIcon::Pointer,
                    false,
                    None,
                );
                if mark != Mark::None {
                    ui.leaf(
                        Style::default(),
                        vec2(16.0, 16.0),
                        None,
                        Sense::NONE,
                        CursorIcon::Default,
                        Some(Box::new(move |cx, r| {
                            let r = Rect::from_center_size(r.center(), r.size() * (1.0 - 0.1 * press));
                            match mark {
                                Mark::Box(_) => {
                                    let t = tick.min(1.0);
                                    cx.painter.rrect_unsnapped(r, Corners::all(4.0), [accent.multiply_alpha(t * enter); 2], 1.0, box_border.mix(accent, t).multiply_alpha(enter));
                                    if tick > 0.01 {
                                        cx.icon("check", r.center(), 12.0 * tick, afg.multiply_alpha(t * enter));
                                    }
                                }
                                _ => {
                                    if tick > 0.01 {
                                        let c = if mac { check_c.mix(fg_hot, hh) } else { check_c };
                                        cx.icon("check", r.center(), 15.0 * tick, c.multiply_alpha(tick.min(1.0) * enter));
                                    }
                                }
                            }
                        })),
                    );
                }
                if let Some(i) = icon {
                    ui.leaf(Style::default(), vec2(16.0, 16.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.icon(&i, r.center(), 16.0, ic))));
                }
                let rise = (1.0 - enter) * 4.0;
                ui.leaf(
                    Style { width: Size::FILL, ..Default::default() },
                    vec2(g.width.min(240.0), g.height),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, r| {
                        cx.painter.push_clip(r.expand2(2.0, 4.0));
                        let pos = vec2(r.min.x, (r.center().y - g.height / 2.0 + rise).round());
                        paint_highlighted(cx, &label, &body, pos, text_c, &hl, hl_c.multiply_alpha(enter), tint.multiply_alpha(enter));
                        cx.painter.pop_clip();
                    })),
                );
                if let Some(h) = hint {
                    let st = ui.text_style(TextRole::Label);
                    ui.text_leaf(h, st, hint_c, false, false, Style::default());
                }
                ui.end();
            }
            ui.end();
        };
        if scrolls {
            ui_fixed_scroll(self, id, view_h, |ui| build(ui, &mut out));
        } else {
            build(self, &mut out);
        }
        out
    }
}

fn ui_fixed_scroll(ui: &mut Ui, id: Id, h: f32, f: impl FnOnce(&mut Ui)) {
    ui.begin(Style { width: Size::FILL, height: Size::Fixed(h), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
    let info = ui.scroll((id, "list")).show(f).1;
    ui.end();
    ui.mem::<ListMem>(id).offset = info.offset;
}

/// An empty-list row: an icon and a muted line of text.
fn no_results(ui: &mut Ui, text: String) {
    let muted = ui.color(Token::TextMuted);
    let st = ui.text_style(TextRole::Body);
    ui.begin(Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(ui.option_row_height() + 8.0), pad: Sides::xy(10.0, 0.0), gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
    ui.icon_leaf("search-x", 16.0, muted);
    ui.text_leaf(text, st, muted, false, true, Style { width: Size::FILL, ..Default::default() });
    ui.end();
}

/// Filter `options` by `query`: (option index, match ranges), best first.
fn filter(options: &[&str], query: &str) -> Vec<(usize, Vec<Range<usize>>)> {
    let mut v: Vec<_> = options.iter().enumerate().filter_map(|(i, l)| fuzzy_match(l, query).map(|m| (i, m))).collect();
    if !query.trim().is_empty() {
        v.sort_by_key(|(i, m)| (-match_score(m), *i));
    }
    v
}

#[derive(Default)]
struct ComboMem {
    query: String,
    /// The user typed since the list opened: filter by the query.
    typing: bool,
    active: usize,
    was_focused: bool,
}

/// Builder for [`Ui::combobox`].
pub struct Combobox<'u, 'o, T: Copy + PartialEq> {
    ui: &'u mut Ui,
    value: &'u mut Option<T>,
    options: &'o [(T, &'o str)],
    label: Option<String>,
    placeholder: String,
    width: Size,
    disabled: bool,
    max_rows: usize,
    shown: bool,
}

impl<T: Copy + PartialEq> Combobox<'_, '_, T> {
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Size::Fixed(w);
        self
    }
    pub fn fill_width(mut self) -> Self {
        self.width = Size::FILL;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    /// Rows shown before the list scrolls (default 7).
    pub fn max_rows(mut self, n: usize) -> Self {
        self.max_rows = n.max(3);
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
        let ui = &mut *self.ui;
        let labels: Vec<&str> = self.options.iter().map(|o| o.1).collect();
        let id = ui.make_id(("combobox", self.label.as_deref().unwrap_or(""), &labels));
        let edit_id = id.with("edit");
        let current = self.options.iter().position(|o| Some(o.0) == *self.value);
        let current_label = current.map(|i| labels[i].to_owned()).unwrap_or_default();
        let open = ui.mem::<PopState>(id).open;
        let focused = ui.focused() == Some(edit_id);
        let mo = ui.motion();

        // Sync the query with the value while the user isn't typing.
        {
            let m = ui.mem::<ComboMem>(id);
            if !m.typing && m.query != current_label {
                m.query = current_label.clone();
            }
        }
        let mut query = std::mem::take(&mut ui.mem::<ComboMem>(id).query);
        let gained = focused && !ui.mem::<ComboMem>(id).was_focused;
        if gained && !ui.mem::<ComboMem>(id).typing {
            ui.inline_edit_select_all(edit_id, &query);
        }

        let m = ui.control_metrics(ControlSize::Md);
        let hovered = ui.pointer().is_some_and(|p| ui.rect_of(id).is_some_and(|r| r.contains(p)));
        let chev = ui.spring(id, 3, if open { 1.0 } else { 0.0 }, mo.snappy);
        let fill_w = matches!(self.width, Size::Fill(_));
        ui.begin(Style { width: if fill_w { Size::FILL } else { Size::Hug }, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if let Some(l) = self.label.take() {
            ui.field_label(&l, self.disabled);
        }
        let fw = match self.width {
            Size::Hug => Size::Fixed(240.0),
            w => w,
        };
        ui.field_begin(
            id,
            Chrome { hovered, focused: focused || open, error: false, disabled: self.disabled },
            Style { dir: Dir::Row, width: fw, height: Size::Fixed(m.height), pad: Sides { left: 12.0, right: 6.0, top: 0.0, bottom: 0.0 }, gap: 6.0, align: Align::Center, ..Default::default() },
        );
        let mut o = EditOpts::new(ui);
        o.placeholder = self.placeholder.clone();
        o.disabled = self.disabled;
        let ed = ui.inline_edit(edit_id, &mut query, o);
        let icon_c = ui.color(Token::TextMuted);
        let icon_sz = m.icon;
        ui.leaf(
            Style::default(),
            vec2(icon_sz + 4.0, icon_sz),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let c0 = icon_c.multiply_alpha(1.0 - chev.clamp(0.0, 1.0));
                let c1 = icon_c.multiply_alpha(chev.clamp(0.0, 1.0));
                cx.icon("chevron-down", r.center() + vec2(0.0, 2.0 * chev), icon_sz, c0);
                cx.icon("chevron-up", r.center() + vec2(0.0, 2.0 * (chev - 1.0)), icon_sz, c1);
            })),
        );
        ui.end();
        let field_rect = ui.rect_of(id).unwrap_or_default();
        // Clicking the frame's padding focuses the editor.
        if ui.clicked_id(id) && !self.disabled {
            ui.request_focus(edit_id);
        }

        // Filter.
        let typing = ui.mem::<ComboMem>(id).typing || ed.changed;
        let list = if typing { filter(&labels, &query) } else { (0..labels.len()).map(|i| (i, vec![])).collect() };
        let mut active = ui.mem::<ComboMem>(id).active;
        let mut open_now = open;
        if ed.changed {
            ui.mem::<ComboMem>(id).typing = true;
            active = 0;
            open_now = true;
        }
        if (ed.ix.press_started || ui.clicked_id(id)) && !open && !self.disabled {
            open_now = true;
            active = current.and_then(|c| list.iter().position(|(i, _)| *i == c)).unwrap_or(0);
        }
        let mut chosen: Option<usize> = None;
        if ed.key(Key::ArrowDown) || ed.key(Key::ArrowUp) {
            if !open_now {
                open_now = true;
                active = current.and_then(|c| list.iter().position(|(i, _)| *i == c)).unwrap_or(0);
            } else if !list.is_empty() {
                let n = list.len();
                active = if ed.key(Key::ArrowDown) { (active + 1) % n } else { (active + n - 1) % n };
            }
        }
        if open_now && !list.is_empty() {
            if ed.key(Key::Home) {
                active = 0;
            }
            if ed.key(Key::End) {
                active = list.len() - 1;
            }
        }
        if ed.key(Key::Enter) && open_now {
            if let Some((i, _)) = list.get(active) {
                chosen = Some(*i);
            }
        }
        let mut revert = false;
        if ed.key(Key::Escape) {
            if open_now {
                open_now = false;
            }
            revert = true;
        }
        active = active.min(list.len().saturating_sub(1));
        if open_now != open {
            ui.set_popover_open(id, open_now);
        }

        let anchor = Response { id, rect: field_rect, hovered, clicked: false, changed: false, focused };
        let empty_text = format!("No matches for “{}”", query.trim());
        let rows: Vec<super::combo::Row<'_>> = list
            .iter()
            .map(|(i, hl)| Row { key: *i, label: labels[*i], hl: hl.clone(), mark: Mark::Check(Some(*i) == current), icon: None, hint: None })
            .collect();
        let max_rows = self.max_rows;
        let res = ui.popover(&anchor).manual().match_width().pad(4.0).return_focus(edit_id).show(|ui| {
            if rows.is_empty() {
                no_results(ui, empty_text);
                ListOut { clicked: None, hovered: None }
            } else {
                ui.option_list(id.with("list"), &rows, Some(active), max_rows)
            }
        });
        if let Some(out) = res {
            if let Some(h) = out.hovered {
                active = h;
            }
            if let Some(c) = out.clicked {
                chosen = Some(list[c].0);
            }
        }
        ui.end();

        let mut changed = false;
        if let Some(c) = chosen {
            let v = self.options[c].0;
            changed = *self.value != Some(v);
            *self.value = Some(v);
            query = labels[c].to_owned();
            ui.inline_edit_caret_end(edit_id, &query);
            ui.set_popover_open(id, false);
            ui.request_focus(edit_id);
            ui.mem::<ComboMem>(id).typing = false;
        }
        // Leaving the field: an empty query clears, anything else that isn't
        // an option goes back to the current value.
        let still_focused = ui.focused() == Some(edit_id);
        if (!still_focused && ui.mem::<ComboMem>(id).was_focused) || revert {
            ui.set_popover_open(id, false);
            if query.trim().is_empty() && !revert {
                changed |= self.value.is_some();
                *self.value = None;
            } else if let Some(i) = labels.iter().position(|l| l.eq_ignore_ascii_case(query.trim())).filter(|_| !revert) {
                let v = self.options[i].0;
                changed |= *self.value != Some(v);
                *self.value = Some(v);
            }
            ui.mem::<ComboMem>(id).typing = false;
            query = self.options.iter().find(|o| Some(o.0) == *self.value).map(|o| o.1.to_owned()).unwrap_or_default();
            ui.inline_edit_caret_end(edit_id, &query);
        }
        let st = ui.mem::<ComboMem>(id);
        st.query = query;
        st.active = active;
        st.was_focused = still_focused;
        Response { id, rect: field_rect, hovered, clicked: false, changed, focused }
    }
}

impl<T: Copy + PartialEq> Drop for Combobox<'_, '_, T> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

#[derive(Default)]
struct MultiMem {
    query: String,
    active: usize,
    /// Backspace on an empty query arms the last chip; a second press
    /// removes it.
    armed: bool,
}

/// Builder for [`Ui::multi_select`].
pub struct MultiSelect<'u, 'o, T: Copy + PartialEq> {
    ui: &'u mut Ui,
    values: &'u mut Vec<T>,
    options: &'o [(T, &'o str)],
    label: Option<String>,
    placeholder: String,
    width: Size,
    disabled: bool,
    max_rows: usize,
    shown: bool,
}

impl<T: Copy + PartialEq> MultiSelect<'_, '_, T> {
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Size::Fixed(w);
        self
    }
    pub fn fill_width(mut self) -> Self {
        self.width = Size::FILL;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn max_rows(mut self, n: usize) -> Self {
        self.max_rows = n.max(3);
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
        let ui = &mut *self.ui;
        let labels: Vec<&str> = self.options.iter().map(|o| o.1).collect();
        let id = ui.make_id(("multi_select", self.label.as_deref().unwrap_or(""), &labels));
        let edit_id = id.with("edit");
        let inner_id = id.with("wrap");
        let open = ui.mem::<PopState>(id).open;
        let focused = ui.focused() == Some(edit_id);
        let mo = ui.motion();
        let m = ui.control_metrics(ControlSize::Md);
        let mut query = std::mem::take(&mut ui.mem::<MultiMem>(id).query);
        let armed = ui.mem::<MultiMem>(id).armed;
        let hovered = ui.pointer().is_some_and(|p| ui.rect_of(id).is_some_and(|r| r.contains(p)));
        let fill_w = matches!(self.width, Size::Fill(_));
        let mut changed = false;
        let mut remove: Option<usize> = None;

        ui.begin(Style { width: if fill_w { Size::FILL } else { Size::Hug }, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if let Some(l) = self.label.take() {
            ui.field_label(&l, self.disabled);
        }
        let fw = match self.width {
            Size::Hug => Size::Fixed(320.0),
            w => w,
        };
        // The frame's height springs to the wrapped content's height.
        let inner_h = ui.rect_of(inner_id).map(|r| r.height()).unwrap_or(m.height - 8.0);
        let h = ui.spring(id, 5, (inner_h + 8.0).max(m.height), mo.smooth);
        ui.field_begin(
            id,
            Chrome { hovered, focused: focused || open, error: false, disabled: self.disabled },
            Style { dir: Dir::Row, width: fw, height: Size::Fixed(h), pad: Sides { left: 4.0, right: 6.0, top: 4.0, bottom: 4.0 }, gap: 4.0, align: Align::Start, ..Default::default() },
        );
        let clip = ui.begin(Style { width: Size::FILL, height: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, true, None);
        let _ = clip;
        ui.begin(Style { dir: Dir::Row, wrap: true, width: Size::FILL, gap: 4.0, align: Align::Center, ..Default::default() }, Some(inner_id), Sense::NONE, CursorIcon::Default, false, None);
        let n_sel = self.values.len();
        for (k, v) in self.values.iter().enumerate() {
            let Some(oi) = self.options.iter().position(|o| o.0 == *v) else { continue };
            let last_armed = armed && k + 1 == n_sel;
            if ui.scope(("chip", oi), |ui| chip_with_remove(ui, labels[oi], last_armed, self.disabled)) {
                remove = Some(k);
            }
        }
        let mut o = EditOpts::new(ui);
        o.placeholder = if self.values.is_empty() { self.placeholder.clone() } else { String::new() };
        o.disabled = self.disabled;
        o.min_w = 60.0;
        ui.begin(Style { width: Size::FILL, min: vec2(60.0, 0.0), height: Size::Fixed(m.height - 8.0), pad: Sides::xy(8.0, 0.0), align: Align::Center, justify: crate::layout::Justify::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let ed = ui.inline_edit(edit_id, &mut query, o);
        ui.end();
        ui.end();
        ui.end();
        let chev = ui.spring(id, 3, if open { 1.0 } else { 0.0 }, mo.snappy);
        let icon_c = ui.color(Token::TextMuted);
        let icon_sz = m.icon;
        ui.begin(Style { height: Size::Fixed(m.height - 8.0), justify: crate::layout::Justify::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        ui.leaf(
            Style::default(),
            vec2(icon_sz + 4.0, icon_sz),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let c0 = icon_c.multiply_alpha(1.0 - chev.clamp(0.0, 1.0));
                let c1 = icon_c.multiply_alpha(chev.clamp(0.0, 1.0));
                cx.icon("chevron-down", r.center() + vec2(0.0, 2.0 * chev), icon_sz, c0);
                cx.icon("chevron-up", r.center() + vec2(0.0, 2.0 * (chev - 1.0)), icon_sz, c1);
            })),
        );
        ui.end();
        ui.end();
        let field_rect = ui.rect_of(id).unwrap_or_default();
        if ui.clicked_id(id) && !self.disabled {
            ui.request_focus(edit_id);
        }

        let list = filter(&labels, &query);
        let mut active = ui.mem::<MultiMem>(id).active;
        let mut open_now = open;
        let mut armed_now = armed && ed.keys.iter().all(|(k, _)| *k == Key::Backspace) && !ed.changed && !ed.ix.press_started;
        if ed.changed {
            active = 0;
            open_now = true;
        }
        if (ed.ix.press_started || ui.clicked_id(id)) && !open && !self.disabled {
            open_now = true;
            active = 0;
        }
        if ed.key(Key::ArrowDown) || ed.key(Key::ArrowUp) {
            if !open_now {
                open_now = true;
                active = 0;
            } else if !list.is_empty() {
                let n = list.len();
                active = if ed.key(Key::ArrowDown) { (active + 1) % n } else { (active + n - 1) % n };
            }
        }
        let mut toggle: Option<usize> = None;
        if ed.key(Key::Enter) && open_now {
            toggle = list.get(active).map(|x| x.0);
        }
        if ed.key(Key::Backspace) && query.is_empty() && !self.values.is_empty() {
            if armed {
                remove = Some(self.values.len() - 1);
                armed_now = false;
            } else {
                armed_now = true;
            }
        }
        if ed.key(Key::Escape) {
            open_now = false;
        }
        active = active.min(list.len().saturating_sub(1));
        if open_now != open {
            ui.set_popover_open(id, open_now);
        }
        let anchor = Response { id, rect: field_rect, hovered, clicked: false, changed: false, focused };
        let empty_text = format!("No matches for “{}”", query.trim());
        let values = &*self.values;
        let opts = self.options;
        let rows: Vec<Row<'_>> = list.iter().map(|(i, hl)| Row { key: *i, label: labels[*i], hl: hl.clone(), mark: Mark::Box(values.contains(&opts[*i].0)), icon: None, hint: None }).collect();
        let max_rows = self.max_rows;
        let res = ui.popover(&anchor).manual().match_width().pad(4.0).return_focus(edit_id).show(|ui| {
            if rows.is_empty() {
                no_results(ui, empty_text);
                ListOut { clicked: None, hovered: None }
            } else {
                ui.option_list(id.with("list"), &rows, Some(active), max_rows)
            }
        });
        if let Some(out) = res {
            if let Some(h) = out.hovered {
                active = h;
            }
            if let Some(c) = out.clicked {
                toggle = Some(list[c].0);
                ui.request_focus(edit_id);
            }
        }
        ui.end();
        if let Some(i) = toggle {
            let v = self.options[i].0;
            if let Some(p) = self.values.iter().position(|x| *x == v) {
                self.values.remove(p);
            } else {
                self.values.push(v);
            }
            changed = true;
            if !query.is_empty() {
                query.clear();
                ui.inline_edit_caret_end(edit_id, &query);
            }
        }
        if let Some(k) = remove {
            if k < self.values.len() {
                self.values.remove(k);
                changed = true;
            }
        }
        let st = ui.mem::<MultiMem>(id);
        st.query = query;
        st.active = active;
        st.armed = armed_now;
        Response { id, rect: field_rect, hovered, clicked: false, changed, focused }
    }
}

impl<T: Copy + PartialEq> Drop for MultiSelect<'_, '_, T> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

/// A chip with a remove button, used inside multi-select and tag input
/// fields. It pops in on a spring. `armed` tints it (the next Backspace
/// removes it). Returns true when its × is clicked.
pub(crate) fn chip_with_remove(ui: &mut Ui, label: &str, armed: bool, disabled: bool) -> bool {
    let id = ui.make_id("chip");
    let x_id = id.with("x");
    let mo = ui.motion();
    let enter = ui.anims.spring_from(id, 0, 0.0, 1.0, mo.bouncy);
    let arm = ui.spring(id, 1, if armed { 1.0 } else { 0.0 }, mo.snappy);
    let c = |t| ui.color(t);
    let fill = c(Token::SurfaceMuted).over(c(Token::Hover)).mix(c(Token::AccentSubtle), arm);
    let border = c(Token::BorderSubtle).mix(c(Token::Accent).with_alpha(0.6), arm);
    let fg = c(Token::Foreground);
    let mut st = ui.text_style(TextRole::Label);
    st.weight = 500;
    let g = ui.galley(label, &st);
    let h = (ui.control_metrics(ControlSize::Md).height - 12.0).max(22.0);
    let e = enter.max(0.0);
    ui.begin(
        Style { dir: Dir::Row, height: Size::Fixed(h), pad: Sides { left: 9.0, right: 3.0, top: 0.0, bottom: 0.0 }, gap: 2.0, align: Align::Center, ..Default::default() },
        Some(id),
        Sense::NONE,
        CursorIcon::Default,
        false,
        Some(Box::new(move |cx, r| {
            let r = Rect::from_center_size(r.center(), r.size() * (0.7 + 0.3 * e));
            cx.painter.rrect_unsnapped(r, Corners::all(r.height() / 2.0), [fill.multiply_alpha(e.min(1.0)); 2], 1.0, border.multiply_alpha(e.min(1.0)));
        })),
    );
    let tc = fg.multiply_alpha(e.min(1.0));
    ui.leaf(Style::default(), vec2(g.width, g.height), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.text(cx.text, &g, vec2(r.min.x.round(), (r.center().y - g.height / 2.0).round()), tc))));
    let clicked = if disabled { false } else { small_x(ui, x_id, h - 6.0, e) };
    ui.end();
    clicked
}

/// A tiny × inside a chip.
fn small_x(ui: &mut Ui, id: Id, size: f32, alpha: f32) -> bool {
    let ix = ui.interact(id, Sense { click: true, drag: false, focusable: false, scroll: false }, false);
    let mo = ui.motion();
    let h = ui.fade(id, 1, ix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
    let press = ui.spring(id, 2, if ix.pressed { 1.0 } else { 0.0 }, mo.snappy);
    let hover = ui.color(Token::Hover).over(ui.color(Token::Hover));
    let ic = ui.color(Token::TextMuted).mix(ui.color(Token::Foreground), h);
    ui.leaf(
        Style::default(),
        vec2(size, size),
        Some(id),
        Sense { click: true, drag: false, focusable: false, scroll: false },
        CursorIcon::Pointer,
        Some(Box::new(move |cx, r| {
            let k = 1.0 - 0.1 * press;
            if h > 0.0 {
                cx.painter.circle(r.center(), size / 2.0 * k, hover.multiply_alpha(h * alpha.min(1.0)), 0.0, Color::TRANSPARENT);
            }
            cx.icon("x", r.center(), 12.0 * k, ic.multiply_alpha(alpha.min(1.0)));
        })),
    );
    ix.clicked
}

impl Ui {
    /// A text field that filters a list of options as you type and picks
    /// one. Matched characters are highlighted; ↑ ↓ move, Enter picks and
    /// Escape restores the current value. Clearing the text and leaving
    /// sets the value to `None`.
    pub fn combobox<'u, 'o, T: Copy + PartialEq>(&'u mut self, value: &'u mut Option<T>, options: &'o [(T, &'o str)]) -> Combobox<'u, 'o, T> {
        Combobox { ui: self, value, options, label: None, placeholder: "Search…".into(), width: Size::Hug, disabled: false, max_rows: 7, shown: false }
    }

    /// Pick several options. The picks show as chips inside the field,
    /// which grows (on a spring) as they wrap. Type to filter; Enter
    /// toggles the highlighted option; Backspace on an empty query arms
    /// the last chip, and a second Backspace removes it.
    pub fn multi_select<'u, 'o, T: Copy + PartialEq>(&'u mut self, values: &'u mut Vec<T>, options: &'o [(T, &'o str)]) -> MultiSelect<'u, 'o, T> {
        MultiSelect { ui: self, values, options, label: None, placeholder: "Add…".into(), width: Size::Hug, disabled: false, max_rows: 7, shown: false }
    }
}
