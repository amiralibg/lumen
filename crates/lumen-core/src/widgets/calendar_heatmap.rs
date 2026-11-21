//! Calendar heatmap: month calendars whose days are tinted by a value
//! (deploys, minutes practiced, spend). Days fill in on first show, the
//! hovered day lifts, the selection ring glides, and the arrow keys walk
//! days and weeks across months.

use std::collections::HashMap;

use crate::anim::Ease;
use crate::date::Date;
use crate::input::{CursorIcon, Key};
use crate::layout::{Size, Style};
use crate::math::{vec2, Corners, Rect};
use crate::theme::{TextRole, Token};
use crate::ui::{Sense, Ui};
use crate::Color;

const CELL: f32 = 26.0;
const GAP: f32 = 4.0;
const MONTH_GAP: f32 = 28.0;
const LEVELS: usize = 5;

#[derive(Default)]
struct HeatMem {
    born: Option<f64>,
}

impl Ui {
    /// `months` month calendars starting at `first_month`, each day tinted
    /// by its value in `values` (days not listed are 0). Click a day or use
    /// the arrow keys to move `selected`. Returns true when it changed.
    /// `unit` labels the readout, such as "deploys".
    pub fn calendar_heatmap(&mut self, key: impl std::hash::Hash, first_month: Date, months: usize, values: &[(Date, f32)], unit: &str, selected: &mut Option<Date>) -> bool {
        let id = self.make_id(("calendar_heatmap", key));
        let months = months.max(1);
        let first = first_month.first_of_month();
        let end = first.add_months(months as i32).add_days(-1);
        let map: HashMap<i64, f32> = values.iter().map(|(d, v)| (d.to_days(), *v)).collect();
        let max = values.iter().map(|v| v.1).fold(0.0f32, f32::max).max(1e-6);
        let level = |v: f32| if v <= 0.0 { 0 } else { ((v / max) * (LEVELS - 1) as f32).ceil().clamp(1.0, (LEVELS - 1) as f32) as usize };
        let old = *selected;
        let sense = Sense { click: true, drag: false, focusable: true, scroll: false };
        let ix = self.interact(id, sense, false);
        let mo = self.motion();
        let reduced = self.reduced_motion();
        let now = self.time();

        // How many months fit per row at the last width.
        let month_w = 7.0 * CELL + 6.0 * GAP;
        let head_h = 46.0;
        let month_h = head_h + 6.0 * (CELL + GAP);
        let avail = if ix.rect.width() > 0.0 { ix.rect.width() } else { months as f32 * (month_w + MONTH_GAP) };
        let per_row = (((avail + MONTH_GAP) / (month_w + MONTH_GAP)).floor() as usize).clamp(1, months);
        let rows = months.div_ceil(per_row);
        let origin = ix.rect.min;
        let cell_rect = |d: Date, origin: crate::Vec2| -> Rect {
            let mi = ((d.year - first.year) * 12 + d.month as i32 - first.month as i32) as usize;
            let (mr, mc) = (mi / per_row, mi % per_row);
            let mx = origin.x + mc as f32 * (month_w + MONTH_GAP);
            let my = origin.y + mr as f32 * (month_h + 18.0);
            let lead = d.first_of_month().weekday() as usize;
            let k = lead + d.day as usize - 1;
            Rect::from_xywh(mx + (k % 7) as f32 * (CELL + GAP), my + head_h + (k / 7) as f32 * (CELL + GAP), CELL, CELL)
        };
        let in_range = |d: Date| d.to_days() >= first.to_days() && d.to_days() <= end.to_days();
        let day_at = |p: crate::Vec2| -> Option<Date> {
            let mut d = first;
            while d.to_days() <= end.to_days() {
                if cell_rect(d, origin).expand(GAP * 0.5).contains(p) {
                    return Some(d);
                }
                d = d.add_days(1);
            }
            None
        };
        let hover_day = ix.pointer.filter(|_| ix.hovered).and_then(day_at);
        if ix.clicked {
            if let Some(d) = ix.pointer.and_then(day_at) {
                *selected = Some(d);
            }
        }
        if ix.focused {
            let cur = selected.unwrap_or(first);
            let mut d = cur;
            for (k, m, _) in self.keys_for(id) {
                if m.command() {
                    continue;
                }
                d = match k {
                    Key::ArrowLeft => d.add_days(-1),
                    Key::ArrowRight => d.add_days(1),
                    Key::ArrowUp => d.add_days(-7),
                    Key::ArrowDown => d.add_days(7),
                    Key::Home => d.first_of_month(),
                    Key::End => d.first_of_month().add_months(1).add_days(-1),
                    Key::PageUp => d.add_months(-1),
                    Key::PageDown => d.add_months(1),
                    _ => d,
                };
            }
            if d != cur || selected.is_none() && !self.keys_for(id).is_empty() {
                let d = if in_range(d) { d } else { cur };
                *selected = Some(d);
            }
        }

        // Days fill in by date on first show.
        let born = *self.mem::<HeatMem>(id).born.get_or_insert(now);
        let total_days = (end.to_days() - first.to_days() + 1) as usize;
        let mut pending = false;
        let mut cells = Vec::with_capacity(total_days);
        let mut d = first;
        let mut i = 0usize;
        while d.to_days() <= end.to_days() {
            let v = map.get(&d.to_days()).copied().unwrap_or(0.0);
            let enter = if reduced {
                1.0
            } else {
                let t = ((now - born - i as f64 * 0.006) / 0.32).clamp(0.0, 1.0) as f32;
                pending |= t < 1.0;
                Ease::Out.apply(t)
            };
            let did = id.with(("day", d.to_days()));
            let hot = hover_day == Some(d);
            let lift = if reduced { if hot { 1.0 } else { 0.0 } } else { self.spring(did, 0, if hot { 1.0 } else { 0.0 }, mo.bouncy) };
            cells.push((cell_rect(d, vec2(0.0, 0.0)), level(v), enter, lift, d == Date::today()));
            d = d.add_days(1);
            i += 1;
        }
        if pending {
            self.anims.keep_alive();
        }
        let sel_r = selected.filter(|d| in_range(*d)).map(|d| cell_rect(d, vec2(0.0, 0.0)));
        let ring_x = sel_r.map(|r| if reduced { r.min.x } else { self.spring(id, 1, r.min.x, mo.snappy) });
        let ring_y = sel_r.map(|r| if reduced { r.min.y } else { self.spring(id, 2, r.min.y, mo.snappy) });
        let ring_vis = self.fade(id, 3, sel_r.is_some(), mo.hover_in, mo.hover_out, Ease::Linear);
        let (focus_a, focus_gap) = self.focus_anim(&ix);

        // Colors: an accent ramp over the empty-day fill.
        let empty = self.color(Token::SurfaceMuted);
        let accent = self.color(Token::Accent);
        let ramp: Vec<Color> = (0..LEVELS).map(|k| if k == 0 { empty } else { empty.mix(accent, 0.22 + 0.78 * (k as f32 / (LEVELS - 1) as f32)) }).collect();
        let fg = self.color(Token::Foreground);
        let muted = self.color(Token::TextMuted);
        let ring_c = self.color(Token::FocusRing);
        let today_c = self.color(Token::AccentStrong);
        let r_cell = (self.look().control.radius[0] * 0.75).max(3.0);
        let st_month = self.text_style(TextRole::BodyStrong);
        let st_wd = self.text_style(TextRole::Caption);
        let month_titles: Vec<String> = (0..months).map(|k| {
            let m = first.add_months(k as i32);
            let total: f32 = values.iter().filter(|(d, _)| d.year == m.year && d.month == m.month).map(|v| v.1).sum();
            format!("{} {} · {}", m.month_name(), m.year, fmt_num(total))
        }).collect();

        let legend = ramp.clone();
        let height = rows as f32 * (month_h + 18.0) - 18.0;
        let width = per_row as f32 * (month_w + MONTH_GAP) - MONTH_GAP;
        self.leaf(
            Style { width: Size::FILL, height: Size::Fixed(height), min: vec2(month_w, 0.0), ..Default::default() },
            vec2(width.max(month_w), height),
            Some(id),
            sense,
            CursorIcon::Pointer,
            Some(Box::new(move |cx, r| {
                let o = r.min;
                for (k, title) in month_titles.iter().enumerate() {
                    let (mr, mc) = (k / per_row, k % per_row);
                    let mx = o.x + mc as f32 * (month_w + MONTH_GAP);
                    let my = o.y + mr as f32 * (month_h + 18.0);
                    let g = cx.galley(title, &st_month);
                    cx.painter.text(cx.text, &g, vec2(mx, my), fg);
                    for (w, l) in ["M", "T", "W", "T", "F", "S", "S"].iter().enumerate() {
                        let g = cx.galley(l, &st_wd);
                        cx.painter.text(cx.text, &g, vec2(mx + w as f32 * (CELL + GAP) + (CELL - g.width) * 0.5, my + 24.0), muted);
                    }
                }
                for &(cr, lv, enter, lift, today) in &cells {
                    let cr = cr.translate(o);
                    let grow = 1.5 * lift;
                    let s = 0.6 + 0.4 * enter;
                    let rr = Rect::from_center_size(cr.center() - vec2(0.0, lift), cr.size() * s + vec2(grow, grow));
                    cx.painter.rrect_unsnapped(rr, Corners::all(r_cell), [ramp[lv].multiply_alpha(enter); 2], 0.0, Color::TRANSPARENT);
                    if today {
                        cx.painter.circle(vec2(rr.center().x, rr.max.y - 4.0), 1.5, today_c, 0.0, Color::TRANSPARENT);
                    }
                }
                if let (Some(x), Some(y)) = (ring_x, ring_y) {
                    let rr = Rect::from_xywh(o.x + x, o.y + y, CELL, CELL).expand(2.5);
                    cx.painter.ring(rr, Corners::all(r_cell + 2.5), 2.0, fg.multiply_alpha(ring_vis));
                }
                if focus_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr.expand(4.0), r_cell + 4.0, focus_a, focus_gap, ring_c)));
                }
            })),
        );

        // Readout and legend.
        let shown = hover_day.or(*selected);
        let read = match shown {
            Some(d) => format!("{} · {} {unit}", d.short(), fmt_num(map.get(&d.to_days()).copied().unwrap_or(0.0))),
            None => format!("{} {unit} in {} months", fmt_num(values.iter().map(|v| v.1).sum()), months),
        };
        let row = Style { dir: crate::layout::Dir::Row, width: Size::FILL, gap: 4.0, align: crate::layout::Align::Center, margin_top: 10.0, ..Default::default() };
        self.begin(row, None, Sense::NONE, CursorIcon::Default, false, None);
        let st = self.text_style(TextRole::Label);
        self.text_leaf(read, st, fg, false, true, Style { width: Size::FILL, ..Default::default() });
        let st = self.text_style(TextRole::Caption);
        self.text_leaf("Less".to_owned(), st, muted, false, false, Style::default());
        for c in legend {
            self.leaf(Style::default(), vec2(12.0, 12.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(3.0), c, 0.0, Color::TRANSPARENT))));
        }
        self.text_leaf("More".to_owned(), st, muted, false, false, Style::default());
        self.end();
        *selected != old
    }
}

fn fmt_num(v: f32) -> String {
    if v.fract().abs() < 1e-3 {
        format!("{}", v as i64)
    } else {
        format!("{v:.1}")
    }
}
