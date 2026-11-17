//! Time picker: a field that opens scrollable hour and minute columns (plus
//! AM/PM in 12-hour mode) with sliding selection pills and keyboard stepping.

use crate::anim::{Ease, SpringParams};
use crate::date::Time;
use crate::id::Id;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

use super::calendar::Lead;
use super::show_on_drop;

#[derive(Clone, Copy)]
struct TimeMem {
    /// Focused column inside the popover: 0 = hour, 1 = minute, 2 = period.
    col: u8,
    last_time: Option<Time>,
}

impl Default for TimeMem {
    fn default() -> Self {
        TimeMem { col: 0, last_time: None }
    }
}

/// Builder for [`Ui::time_picker`].
pub struct TimePicker<'u> {
    ui: &'u mut Ui,
    label: String,
    time: &'u mut Time,
    twelve_hour: bool,
    step: u8,
    width: Size,
    disabled: bool,
    shown: bool,
}

impl TimePicker<'_> {
    /// Display and edit in 12-hour format with an AM/PM column.
    pub fn twelve_hour(mut self, on: bool) -> Self {
        self.twelve_hour = on;
        self
    }

    /// Minute interval shown in the list (default `5`, clamped to `1..=30`).
    pub fn step(mut self, minutes: u8) -> Self {
        self.step = minutes.clamp(1, 30);
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

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("time_picker", &self.label));
        let grid_id = id.with("grid");
        let text = if self.twelve_hour { self.time.format_12() } else { self.time.format_24() };
        let r = ui.picker_field_sized(id, &self.label, Lead::Icon("clock"), &text, false, Some(grid_id), self.width, self.disabled);
        let mut changed = false;
        let twelve = self.twelve_hour;
        let step = self.step;
        let time = &mut *self.time;
        ui.popover(&r).pad(10.0).show(|ui| {
            changed |= time_panel(ui, id, grid_id, time, twelve, step);
        });
        Response { changed, ..r }
    }
}

show_on_drop!(TimePicker);

fn time_panel(ui: &mut Ui, root_id: Id, grid_id: Id, time: &mut Time, twelve: bool, step: u8) -> bool {
    let mo = ui.motion();
    let cols: u8 = if twelve { 3 } else { 2 };
    let mut mem = *ui.mem::<TimeMem>(root_id);
    mem.col = mem.col.min(cols - 1);
    let mut changed = false;
    let before = *time;

    // Keyboard controller: ↑ ↓ step the active column, ← → switch columns, Enter confirms.
    let gix = ui.interact(grid_id, Sense { click: false, drag: false, focusable: true, scroll: false }, false);
    if gix.focused {
        if ui.key_pressed_for(grid_id, Key::ArrowLeft) {
            mem.col = (mem.col + cols - 1) % cols;
        }
        if ui.key_pressed_for(grid_id, Key::ArrowRight) {
            mem.col = (mem.col + 1) % cols;
        }
        let up = ui.key_pressed_for(grid_id, Key::ArrowUp);
        let down = ui.key_pressed_for(grid_id, Key::ArrowDown);
        if up || down {
            let delta: i32 = if down { 1 } else { -1 };
            match mem.col {
                0 if twelve => {
                    let pm = time.hour >= 12;
                    let h12 = (time.hour % 12) as i32;
                    let nh = (h12 + delta).rem_euclid(12) as u8;
                    time.hour = if pm { nh + 12 } else { nh };
                }
                0 => {
                    time.hour = (time.hour as i32 + delta).rem_euclid(24) as u8;
                }
                1 => {
                    let s = step as i32;
                    let snapped = ((time.minute as i32 + if delta > 0 { 0 } else { s - 1 }) / s) * s;
                    let nm = if time.minute % step == 0 {
                        (time.minute as i32 + delta * s).rem_euclid(60)
                    } else if delta > 0 {
                        (snapped + s).rem_euclid(60)
                    } else {
                        snapped.rem_euclid(60)
                    };
                    time.minute = nm as u8;
                }
                _ => {
                    time.hour = (time.hour + 12) % 24;
                }
            }
        }
        if ui.key_pressed_for(grid_id, Key::Home) {
            match mem.col {
                0 if twelve => time.hour = if time.hour >= 12 { 12 } else { 0 },
                0 => time.hour = 0,
                1 => time.minute = 0,
                _ => time.hour %= 12,
            }
        }
        if ui.key_pressed_for(grid_id, Key::End) {
            match mem.col {
                0 if twelve => time.hour = if time.hour >= 12 { 23 } else { 11 },
                0 => time.hour = 23,
                1 => time.minute = (60 - step as u16).min(59) as u8 - ((60 - step as u16) % step as u16) as u8,
                _ => time.hour = (time.hour % 12) + 12,
            }
        }
        if ui.key_pressed_for(grid_id, Key::Enter) || ui.key_pressed_for(grid_id, Key::Space) {
            ui.close_layer();
        }
    }

    let (ring_a, ring_gap) = ui.focus_anim(&gix);
    let ring = ui.color(Token::FocusRing);

    ui.begin(Style { gap: 8.0, ..Default::default() }, Some(grid_id), Sense { click: false, drag: false, focusable: true, scroll: false }, CursorIcon::Default, false, None);

    // Header: live tabular readout + Now button.
    ui.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, pad: Sides { left: 4.0, ..Default::default() }, gap: 8.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
    let mut head_st = ui.text_style(TextRole::BodyStrong);
    head_st.tabular = true;
    let readout = if twelve { time.format_12() } else { time.format_24() };
    let fg = ui.color(Token::Foreground);
    ui.text_leaf(readout, head_st, fg, false, false, Style { width: Size::FILL, ..Default::default() });
    if ui.scope("now", |ui| ui.button("Now").ghost().small().clicked()) {
        *time = Time::now();
        ui.request_focus(grid_id);
    }
    ui.end();

    let line = ui.color(Token::BorderSubtle);
    ui.leaf(Style { width: Size::FILL, ..Default::default() }, vec2(0.0, 1.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.rect(r, line))));

    // Columns: Hour | Minute (| AM/PM).
    let hours: Vec<(u8, String)> = if twelve {
        (0..12u8)
            .map(|i| {
                let disp = if i == 0 { 12 } else { i };
                let h24 = if time.hour >= 12 { i + 12 } else { i };
                (h24, format!("{disp:02}"))
            })
            .collect()
    } else {
        (0..24u8).map(|h| (h, format!("{h:02}"))).collect()
    };
    let sel_h = if twelve { (time.hour % 12) as usize } else { time.hour as usize };

    let mut mins: Vec<(u8, String)> = (0..60u8).step_by(step as usize).map(|m| (m, format!("{m:02}"))).collect();
    if !mins.iter().any(|(m, _)| *m == time.minute) {
        mins.push((time.minute, format!("{:02}", time.minute)));
        mins.sort_by_key(|(m, _)| *m);
    }
    let sel_m = mins.iter().position(|(m, _)| *m == time.minute).unwrap_or(0);

    let scroll_sync = mem.last_time != Some(*time);

    ui.begin(Style { dir: Dir::Row, gap: 6.0, align: Align::Stretch, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);

    if let Some(idx) = wheel_column(ui, root_id.with("hr"), "Hr", &hours, sel_h, 62.0, 5, scroll_sync, gix.focus_visible && mem.col == 0, ring_a, ring_gap, ring) {
        time.hour = hours[idx].0;
        mem.col = 0;
        ui.request_focus(grid_id);
    }

    let line = ui.color(Token::BorderSubtle);
    ui.leaf(Style { self_align: Some(Align::Stretch), ..Default::default() }, vec2(1.0, 0.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.rect(r, line))));

    if let Some(idx) = wheel_column(ui, root_id.with("min"), "Min", &mins, sel_m, 62.0, 5, scroll_sync, gix.focus_visible && mem.col == 1, ring_a, ring_gap, ring) {
        time.minute = mins[idx].0;
        mem.col = 1;
        ui.request_focus(grid_id);
    }

    if twelve {
        let line = ui.color(Token::BorderSubtle);
        ui.leaf(Style { self_align: Some(Align::Stretch), ..Default::default() }, vec2(1.0, 0.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.rect(r, line))));
        let periods = [(0u8, "AM".to_string()), (1u8, "PM".to_string())];
        let sel_p = usize::from(time.hour >= 12);
        if let Some(idx) = wheel_column(ui, root_id.with("ampm"), "Period", &periods, sel_p, 56.0, 5, false, gix.focus_visible && mem.col == 2, ring_a, ring_gap, ring) {
            let h12 = time.hour % 12;
            time.hour = if idx == 1 { h12 + 12 } else { h12 };
            mem.col = 2;
            ui.request_focus(grid_id);
        }
    }

    ui.end();
    ui.end();

    changed |= *time != before;
    mem.last_time = Some(*time);
    *ui.mem::<TimeMem>(root_id) = mem;
    let _ = mo;
    changed
}

#[derive(Default)]
struct WheelMem {
    offset: f32,
    init: bool,
}

#[allow(clippy::too_many_arguments)]
fn wheel_column(
    ui: &mut Ui,
    id: Id,
    heading: &str,
    items: &[(u8, String)],
    selected: usize,
    width: f32,
    visible_rows: usize,
    sync_scroll: bool,
    col_focused: bool,
    ring_a: f32,
    ring_gap: f32,
    ring: Color,
) -> Option<usize> {
    let mo = ui.motion();
    let row_h = (ui.look().control.height[0]).max(28.0);
    let gap = 2.0;
    let radius = (ui.look().control.radius[0]).max(4.0);
    let cap = ui.text_style(TextRole::Caption);
    let muted = ui.color(Token::TextMuted);

    ui.begin(
        Style { width: Size::Fixed(width), gap: 4.0, align: Align::Center, ..Default::default() },
        Some(id),
        Sense::NONE,
        CursorIcon::Default,
        false,
        Some(Box::new(move |cx, r| {
            if col_focused && ring_a > 0.0 {
                cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, radius + 2.0, ring_a, ring_gap, ring)));
            }
        })),
    );
    ui.text_leaf(heading, cap, muted, false, false, Style::default());

    let n = items.len();
    let scrolls = n > visible_rows;
    let view_h = visible_rows.min(n) as f32 * (row_h + gap) - gap;
    let scroll_id = ui.scroll_id((id, "col"));
    let first_open = !ui.mem::<WheelMem>(id).init;
    if scrolls && (first_open || sync_scroll) {
        let top = selected as f32 * (row_h + gap);
        let max_off = (n as f32 * (row_h + gap) - gap - view_h).max(0.0);
        let off = ui.mem::<WheelMem>(id).offset;
        if first_open {
            // Center the active item on initial open.
            let centered = (top - (view_h - row_h) * 0.5).clamp(0.0, max_off);
            ui.scroll_set(scroll_id, centered);
        } else if top < off {
            ui.scroll_to(scroll_id, top);
        } else if top + row_h > off + view_h {
            ui.scroll_to(scroll_id, (top + row_h - view_h).min(max_off));
        }
    }
    ui.mem::<WheelMem>(id).init = true;

    let list_id = id.with("rows");
    let target_y = selected as f32 * (row_h + gap);
    let pill_y = if first_open {
        ui.anims.set_spring(id, 0, target_y);
        target_y
    } else {
        ui.spring(id, 0, target_y, mo.snappy)
    };

    let accent = ui.color(Token::Accent);
    let afg = ui.color(Token::AccentForeground);
    let fg = ui.color(Token::Foreground);
    let hover_c = ui.color(Token::Hover);
    let mut tab = ui.text_style(TextRole::Body);
    tab.tabular = true;
    tab.weight = 500;

    let mut clicked = None;
    let build = |ui: &mut Ui, clicked: &mut Option<usize>| {
        ui.begin(
            Style { width: Size::FILL, gap, ..Default::default() },
            Some(list_id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                let pr = Rect::from_xywh(r.min.x, r.min.y + pill_y, r.width(), row_h);
                cx.painter.rrect_unsnapped(pr, Corners::all(radius), [accent; 2], 0.0, Color::TRANSPARENT);
            })),
        );
        for (i, (_, label)) in items.iter().enumerate() {
            let rid = id.with(("item", i));
            let ix = ui.interact(rid, Sense { click: true, drag: false, focusable: false, scroll: false }, false);
            if ix.clicked {
                *clicked = Some(i);
            }
            let is_sel = i == selected;
            let h = ui.fade(rid, 1, ix.hovered && !is_sel, mo.hover_in, mo.hover_out, Ease::Linear);
            let sel_w = ui.fade(rid, 2, is_sel, mo.selected, mo.selected, Ease::Out);
            let press = ui.spring(rid, 3, if ix.pressed { 1.0 } else { 0.0 }, mo.snappy);
            let enter = ui.anims.spring_from(rid, 4, 0.0, 1.0, SpringParams::from_response(0.18 + 0.025 * i.min(7) as f32, 1.0)).clamp(0.0, 1.0);
            let tc = fg.mix(afg, sel_w).multiply_alpha(enter);
            let hl = hover_c.multiply_alpha(h * enter);
            let g = ui.galley(label, &tab);
            ui.leaf(
                Style { width: Size::FILL, ..Default::default() },
                vec2(width, row_h),
                Some(rid),
                Sense { click: true, drag: false, focusable: false, scroll: false },
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    let r = Rect::from_center_size(r.center(), r.size() * (1.0 - 0.05 * press));
                    if hl.a > 0.0 {
                        cx.painter.rrect_unsnapped(r, Corners::all(radius), [hl; 2], 0.0, Color::TRANSPARENT);
                    }
                    let pos = vec2((r.center().x - g.width / 2.0).round(), (r.center().y - g.height / 2.0).round());
                    cx.painter.text(cx.text, &g, pos, tc);
                })),
            );
        }
        ui.end();
    };

    if scrolls {
        ui.begin(Style { width: Size::FILL, height: Size::Fixed(view_h), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let info = ui.scroll((id, "col")).show(|ui| build(ui, &mut clicked)).1;
        ui.end();
        ui.mem::<WheelMem>(id).offset = info.offset;
    } else {
        build(ui, &mut clicked);
        // Pad short columns (like AM/PM) cleanly from the top.
        let _ = Justify::Start;
    }

    ui.end();
    clicked
}

impl Ui {
    /// A time field bound to `time` that opens scrollable hour and minute
    /// columns in a popover. Supports 24-hour and `.twelve_hour(true)` modes.
    pub fn time_picker<'u>(&'u mut self, label: impl Into<String>, time: &'u mut Time) -> TimePicker<'u> {
        TimePicker { ui: self, label: label.into(), time, twelve_hour: false, step: 5, width: Size::Hug, disabled: false, shown: false }
    }
}
