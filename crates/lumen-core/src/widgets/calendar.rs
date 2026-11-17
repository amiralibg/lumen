//! Calendar (single date and range) and the date picker field.

use crate::anim::Ease;
use crate::date::Date;
use crate::id::Id;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{ControlSize, TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

use super::popover::PopState;

const CELL: f32 = 34.0;
const WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

#[derive(Clone, Copy)]
struct CalMem {
    view: Date,
    cursor: Date,
    /// +1 / -1: direction of the last month change (for the slide).
    dir: f32,
    init: bool,
    /// Range mode: the first click, waiting for the second.
    anchor: Option<Date>,
}

impl Default for CalMem {
    fn default() -> Self {
        let t = Date::today();
        CalMem { view: t.first_of_month(), cursor: t, dir: 0.0, init: false, anchor: None }
    }
}

enum Sel<'a> {
    One(&'a mut Date),
    Range(&'a mut Option<(Date, Date)>),
}

impl Ui {
    /// An inline month calendar bound to `date`.
    pub fn calendar(&mut self, date: &mut Date) -> Response {
        let id = self.make_id("calendar");
        self.calendar_impl(id, Sel::One(date))
    }

    /// A calendar that picks a range: first click sets the start, second
    /// the end.
    pub fn calendar_range(&mut self, range: &mut Option<(Date, Date)>) -> Response {
        let id = self.make_id("calendar_range");
        self.calendar_impl(id, Sel::Range(range))
    }

    fn calendar_impl(&mut self, id: Id, mut sel: Sel<'_>) -> Response {
        let today = Date::today();
        let mut m = *self.mem::<CalMem>(id);
        if !m.init {
            let start = match &sel {
                Sel::One(d) => **d,
                Sel::Range(r) => r.map(|r| r.0).unwrap_or(today),
            };
            m = CalMem { view: start.first_of_month(), cursor: start, dir: 0.0, init: true, anchor: None };
        }
        let mo = self.motion();
        let mut changed = false;

        // Header: month title and prev/next.
        let grid_id = id.with("grid");
        let gix = self.interact(grid_id, Sense { click: false, drag: false, focusable: true, scroll: false }, false);
        let mut go_month = 0i32;
        if gix.focused {
            let mut cur = m.cursor;
            let keys: [(Key, i64); 4] = [(Key::ArrowLeft, -1), (Key::ArrowRight, 1), (Key::ArrowUp, -7), (Key::ArrowDown, 7)];
            for (k, d) in keys {
                if self.key_pressed_for(grid_id, k) {
                    cur = cur.add_days(d);
                }
            }
            if self.key_pressed_for(grid_id, Key::PageUp) {
                cur = cur.add_months(-1);
            }
            if self.key_pressed_for(grid_id, Key::PageDown) {
                cur = cur.add_months(1);
            }
            if self.key_pressed_for(grid_id, Key::Home) {
                cur = cur.first_of_month();
            }
            if cur != m.cursor {
                m.cursor = cur;
                let delta = (cur.year - m.view.year) * 12 + cur.month as i32 - m.view.month as i32;
                if delta != 0 {
                    go_month = delta;
                }
            }
            if self.key_pressed_for(grid_id, Key::Enter) || self.key_pressed_for(grid_id, Key::Space) {
                let cur = m.cursor;
                changed |= pick(&mut sel, &mut m, cur);
            }
        }

        let title_st = self.text_style(TextRole::BodyStrong);
        let fg = self.color(Token::Foreground);
        self.begin(Style { gap: 8.0, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);
        self.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, pad: Sides { left: 6.0, ..Default::default() }, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf(format!("{} {}", m.view.month_name(), m.view.year), title_st, fg, false, false, Style { width: Size::FILL, ..Default::default() });
        let prev = self.scope((id, "prev"), |ui| ui.icon_button("chevron-left").ghost().small().tooltip("Previous month").clicked());
        let next = self.scope((id, "next"), |ui| ui.icon_button("chevron-right").ghost().small().tooltip("Next month").clicked());
        self.end();
        if prev {
            go_month = -1;
        }
        if next {
            go_month = 1;
        }
        if go_month != 0 {
            m.view = m.view.add_months(go_month).first_of_month();
            m.dir = go_month.signum() as f32;
            if prev || next {
                m.cursor = Date::new(m.view.year, m.view.month, m.cursor.day);
            }
            // Restart the slide from the side we came from.
            self.anims.set_spring(id, 0, 28.0 * m.dir);
        }
        let slide = self.spring(id, 0, 0.0, mo.smooth);
        // Fade the new month in as it slides.
        let appear = (1.0 - slide.abs() / 28.0).clamp(0.0, 1.0);

        // Weekday header.
        let cap = self.text_style(TextRole::Caption);
        let muted = self.color(Token::TextMuted);
        self.begin(Style { dir: Dir::Row, gap: 2.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for w in WEEKDAYS {
            self.begin(Style { width: Size::Fixed(CELL), height: Size::Fixed(20.0), align: Align::Center, justify: crate::layout::Justify::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.text_leaf(w, cap, muted, false, false, Style::default());
            self.end();
        }
        self.end();

        // Day grid: 6 weeks starting on the Monday on/before the 1st.
        let first = m.view;
        let start = first.add_days(-(first.weekday() as i64));
        let (r0, r1) = match &sel {
            Sel::One(d) => (Some(**d), Some(**d)),
            Sel::Range(r) => match (m.anchor, r) {
                (Some(a), _) => (Some(a), Some(a)),
                (None, Some((a, b))) => (Some(*a), Some(*b)),
                (None, None) => (None, None),
            },
        };
        let is_range = matches!(sel, Sel::Range(_));
        let c = |t| self.color(t);
        let (accent, afg, subtle, strong, hover_c, dis) = (c(Token::Accent), c(Token::AccentForeground), c(Token::AccentSubtle), c(Token::AccentStrong), c(Token::Hover), c(Token::TextDisabled));
        let ring = c(Token::FocusRing);
        let radius = (self.look().control.radius[1]).min(CELL / 2.0);
        let body = self.text_style(TextRole::Body);
        let mut tab = body;
        tab.tabular = true;
        let (ring_a, ring_gap) = self.focus_anim(&gix);
        let grid = self.begin(Style { gap: 2.0, ..Default::default() }, Some(grid_id), Sense { click: false, drag: false, focusable: true, scroll: false }, CursorIcon::Default, true, None);
        let _ = grid;
        let mut clicked_day = None;
        for w in 0..6 {
            self.begin(Style { dir: Dir::Row, gap: 2.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            for d in 0..7 {
                let day = start.add_days(w * 7 + d);
                let did = id.with(("day", day.to_days()));
                let dix = self.interact(did, Sense { click: true, drag: false, focusable: false, scroll: false }, false);
                if dix.clicked {
                    clicked_day = Some(day);
                }
                let in_month = day.month == m.view.month;
                let h = self.fade(did, 0, dix.hovered, 0.0, mo.hover_out, Ease::Linear);
                let selected = r0 == Some(day) || r1 == Some(day);
                let between = matches!((r0, r1), (Some(a), Some(b)) if day > a && day < b);
                let s = self.fade(did, 1, selected, mo.selected, mo.selected, Ease::Out);
                let pop = self.spring(did, 2, if selected { 1.0 } else { 0.92 }, mo.bouncy);
                let is_today = day == today;
                let cursor = gix.focus_visible && day == m.cursor;
                let text_c = if selected {
                    fg.mix(afg, s)
                } else if !in_month {
                    dis
                } else if is_today {
                    strong
                } else {
                    fg
                };
                let band_l = is_range && (between || (selected && r1 != r0 && Some(day) == r1));
                let band_r = is_range && (between || (selected && r1 != r0 && Some(day) == r0));
                let (ra, rga) = (ring_a, ring_gap);
                let label = day.day.to_string();
                let text_c = text_c.multiply_alpha(if selected { 1.0 } else { appear });
                self.leaf(
                    Style::default(),
                    vec2(CELL, CELL),
                    Some(did),
                    Sense { click: true, drag: false, focusable: false, scroll: false },
                    CursorIcon::Pointer,
                    Some(Box::new(move |cx, r| {
                        let r = r.translate(vec2(slide, 0.0));
                        // Range band joins neighbours.
                        if band_l || band_r {
                            let x0 = if band_l { r.min.x - 1.0 } else { r.center().x };
                            let x1 = if band_r { r.max.x + 1.0 } else { r.center().x };
                            cx.painter.rect(Rect { min: vec2(x0, r.min.y + 2.0), max: vec2(x1, r.max.y - 2.0) }, subtle);
                        }
                        if h > 0.0 && !selected {
                            cx.painter.rrect_unsnapped(r.expand(-2.0), Corners::all(radius), [hover_c.multiply_alpha(h); 2], 0.0, Color::TRANSPARENT);
                        }
                        if s > 0.0 {
                            let sr = Rect::from_center_size(r.center(), (r.size() - vec2(4.0, 4.0)) * pop);
                            cx.painter.rrect_unsnapped(sr, Corners::all(radius * pop), [accent.multiply_alpha(s); 2], 0.0, Color::TRANSPARENT);
                        }
                        let g = cx.galley(&label, &tab);
                        let pos = vec2((r.center().x - g.width / 2.0).round(), (r.center().y - g.height / 2.0).round());
                        cx.painter.text(cx.text, &g, pos, text_c);
                        if is_today && !selected {
                            cx.painter.circle(vec2(r.center().x, r.max.y - 6.0), 1.75, strong, 0.0, Color::TRANSPARENT);
                        }
                        if cursor && ra > 0.0 {
                            let rr = r.expand(-2.0);
                            cx.defer(rr, Box::new(move |cx, r| cx.focus_ring(r, radius, ra, rga, ring)));
                        }
                    })),
                );
            }
            self.end();
        }
        self.end();
        self.end();
        if let Some(d) = clicked_day {
            m.cursor = d;
            changed |= pick(&mut sel, &mut m, d);
            if d.month != m.view.month || d.year != m.view.year {
                let delta = (d.year - m.view.year) * 12 + d.month as i32 - m.view.month as i32;
                m.view = d.first_of_month();
                m.dir = delta.signum() as f32;
                self.anims.set_spring(id, 0, 28.0 * m.dir);
            }
            self.request_focus(grid_id);
        }
        *self.mem::<CalMem>(id) = m;
        let rect = self.rect_of(id).unwrap_or_default();
        Response { id, rect, clicked: changed, changed, hovered: false, focused: gix.focused }
    }

    /// A date field that opens a calendar popover.
    pub fn date_picker(&mut self, label: &str, date: &mut Date) -> Response {
        let id = self.make_id(("date_picker", label));
        let r = self.picker_field(id, label, &date.short());
        let mut changed = false;
        self.popover(&r).pad(10.0).show(|ui| {
            ui.scope(id, |ui| {
                if ui.calendar(date).changed() {
                    changed = true;
                    ui.close_layer();
                }
            });
        });
        Response { changed, ..r }
    }

    /// A range field that opens a range calendar.
    pub fn date_range_picker(&mut self, label: &str, range: &mut Option<(Date, Date)>) -> Response {
        let id = self.make_id(("date_range_picker", label));
        let text = match range {
            Some((a, b)) if a == b => a.short(),
            Some((a, b)) => format!("{} – {}", a.short(), b.short()),
            None => "Pick a range".into(),
        };
        let r = self.picker_field(id, label, &text);
        let mut changed = false;
        self.popover(&r).pad(10.0).show(|ui| {
            ui.scope(id, |ui| {
                if ui.calendar_range(range).changed() {
                    changed = true;
                    if range.is_some_and(|(a, b)| a != b) {
                        ui.close_layer();
                    }
                }
            });
        });
        Response { changed, ..r }
    }

    /// Field chrome shared by the date pickers.
    fn picker_field(&mut self, id: Id, label: &str, text: &str) -> Response {
        let ix = self.interact(id, Sense::CLICK, false);
        let key_open = ix.focused && (self.key_pressed_for(id, Key::Space) || self.key_pressed_for(id, Key::Enter) || self.key_pressed_for(id, Key::ArrowDown));
        let m = self.control_metrics(ControlSize::Md);
        let mo = self.motion();
        let open = self.mem::<PopState>(id).open;
        let h = self.fade(id, 1, ix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
        let f = self.fade(id, 2, open || ix.focused, mo.hover_in, mo.hover_out, Ease::Linear);
        let (ring_a, ring_gap) = self.focus_anim(&ix);
        let c = |t| self.color(t);
        let fill = c(Token::Control).mix(c(Token::Control).over(c(Token::Hover)), h);
        let cb = c(Token::ControlBorder);
        let halo = self.look().input.halo;
        let border = if cb.a > 0.0 { cb.mix(c(Token::BorderStrong), h).mix(c(Token::FocusRing), f * halo) } else { cb };
        let fg = c(Token::Foreground);
        let ic = c(Token::TextMuted);
        let ring = c(Token::FocusRing);
        let radius = m.radius;
        self.begin(Style { gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if !label.is_empty() {
            let st = self.text_style(TextRole::Label);
            self.text_leaf(label, st, fg, false, true, Style::default());
        }
        let bw = if border.a > 0.0 { 1.0 } else { 0.0 };
        self.begin(
            Style { dir: Dir::Row, height: Size::Fixed(m.height), min: vec2(200.0, 0.0), pad: Sides::xy(m.pad_x, 0.0), gap: 8.0, align: Align::Center, ..Default::default() },
            Some(id),
            Sense::CLICK,
            CursorIcon::Pointer,
            false,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(radius), fill, bw, border);
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, radius, ring_a, ring_gap, ring)));
                }
            })),
        );
        self.icon_leaf("calendar", m.icon, ic);
        let st = self.text_style(TextRole::Body);
        self.text_leaf(text, st, fg, false, true, Style { width: Size::FILL, ..Default::default() });
        self.end();
        self.end();
        if key_open {
            let st = self.mem::<PopState>(id);
            st.open = true;
            // Hand focus to the grid.
            self.request_focus(id.with("calendar").with("grid"));
        }
        Response { id, rect: ix.rect, hovered: ix.hovered, clicked: ix.clicked, changed: false, focused: ix.focused }
    }
}

fn pick(sel: &mut Sel<'_>, m: &mut CalMem, d: Date) -> bool {
    match sel {
        Sel::One(v) => {
            let ch = **v != d;
            **v = d;
            ch
        }
        Sel::Range(r) => match m.anchor.take() {
            None => {
                m.anchor = Some(d);
                **r = Some((d, d));
                true
            }
            Some(a) => {
                **r = Some(if d < a { (d, a) } else { (a, d) });
                true
            }
        },
    }
}
