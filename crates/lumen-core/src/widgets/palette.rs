//! Command palette (`⌘K` / `Ctrl+K`): a modal fuzzy-filtered command runner
//! with grouped results, highlighted matches, icons and shortcut chips.

use std::ops::Range;

use crate::anim::{Ease, SpringParams};
use crate::id::Id;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::look::Platform;
use crate::math::{vec2, Corners, Rect, Sides, Vec2};
use crate::theme::{Elevation, TextRole, Token};
use crate::ui::{LayerPlace, Sense, Ui};
use crate::Color;

use super::field::{fuzzy_match, match_score, paint_highlighted, EditOpts};

/// One entry in a [`CommandPalette`].
#[derive(Clone, Copy, Debug)]
pub struct Command<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub group: &'a str,
    pub icon: &'a str,
    pub shortcut: &'a str,
    pub keywords: &'a str,
}

impl<'a> Command<'a> {
    pub const fn new(id: &'a str, label: &'a str) -> Self {
        Command { id, label, group: "", icon: "", shortcut: "", keywords: "" }
    }
    pub const fn group(mut self, g: &'a str) -> Self {
        self.group = g;
        self
    }
    pub const fn icon(mut self, icon: &'a str) -> Self {
        self.icon = icon;
        self
    }
    pub const fn shortcut(mut self, s: &'a str) -> Self {
        self.shortcut = s;
        self
    }
    pub const fn keywords(mut self, kw: &'a str) -> Self {
        self.keywords = kw;
        self
    }
}

#[derive(Default)]
struct PaletteMem {
    query: String,
    active: usize,
    was_open: bool,
    prev_focus: Option<Id>,
    pointer: Option<Vec2>,
    offset: f32,
    last_active: Option<usize>,
}

/// Builder for [`Ui::command_palette`].
#[must_use = "call .show()"]
pub struct CommandPalette<'u, 'a> {
    ui: &'u mut Ui,
    key: String,
    open: &'u mut bool,
    commands: &'a [Command<'a>],
    placeholder: String,
    width: f32,
    global_shortcut: bool,
}

impl<'u, 'a> CommandPalette<'u, 'a> {
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
    /// Toggle `*open` on `⌘K` / `Ctrl+K` (default `true`).
    pub fn global_shortcut(mut self, on: bool) -> Self {
        self.global_shortcut = on;
        self
    }

    /// Show the palette while `*open`. Returns `Some(cmd.id)` on the frame a
    /// command is triggered.
    pub fn show(self) -> Option<&'a str> {
        let CommandPalette { ui, key, open, commands, placeholder, width, global_shortcut } = self;
        let id = ui.make_id(("command_palette", &key));
        let scrim_id = id.with("scrim");
        let edit_id = id.with("edit");
        let mo = ui.motion();

        if global_shortcut && ui.shortcut_pressed('k') {
            *open = !*open;
        }
        let was_open = ui.mem::<PaletteMem>(id).was_open;
        if *open && !was_open {
            let prev = ui.focused();
            let m = ui.mem::<PaletteMem>(id);
            m.prev_focus = prev;
            m.query.clear();
            m.active = 0;
        }
        let mut closed = false;
        if *open && (ui.key_pressed(Key::Escape) || ui.clicked_id(scrim_id)) {
            *open = false;
            closed = true;
        }
        let is_open = *open;
        let just_opened = is_open && !was_open;
        ui.mem::<PaletteMem>(id).was_open = is_open;

        let t = ui.fade(id, 0, is_open, mo.layer_in * 1.5, mo.fade_out * 1.3, Ease::Out);
        if t <= 0.0 && !is_open {
            if closed {
                if let Some(prev) = ui.mem::<PaletteMem>(id).prev_focus.take() {
                    ui.request_focus(prev);
                }
            }
            return None;
        }
        let rise = ui.anims.spring_from(id, 1, 10.0, if is_open { 0.0 } else { 5.0 }, if is_open { mo.smooth } else { mo.snappy });

        let mut query = std::mem::take(&mut ui.mem::<PaletteMem>(id).query);
        let mut active = ui.mem::<PaletteMem>(id).active;
        let mut chosen: Option<&'a str> = None;
        let mut esc_clicked = false;

        let screen = ui.screen_size();
        let panel_w = width.min((screen.x - 32.0).max(260.0));
        let top_pad = (screen.y * 0.13).clamp(24.0, 96.0);
        let scrim = ui.color(Token::Scrim);
        let fill = ui.color(Token::SurfaceRaised);
        let border = ui.color(if ui.theme.darkness() > 0.5 { Token::BorderStrong } else { Token::Border });
        let shadows = Elevation::Level3.shadows(ui.theme.darkness(), ui.color(Token::ShadowInk));
        let radius = ui.look().card.radius + 2.0;

        ui.layer(LayerPlace::Screen, vec2(0.0, 0.0), t, Some(scrim_id), |ui| {
            if let Some(root) = ui.layer_root_building() {
                let st = ui.node_style(root);
                st.align = Align::Center;
                st.justify = Justify::Start;
                st.pad = Sides { top: top_pad + rise, left: 16.0, right: 16.0, bottom: 24.0 };
                ui.set_paint(root, Box::new(move |cx, r| cx.painter.rect(r, scrim)));
            }

            ui.begin(
                Style { width: Size::Fixed(panel_w), ..Default::default() },
                Some(id.with("panel")),
                Sense::HOVER,
                CursorIcon::Default,
                true,
                Some(Box::new(move |cx, r| {
                    cx.painter.shadows(r, Corners::all(radius), &shadows);
                    cx.painter.rrect(r, Corners::all(radius), fill, 1.0, border);
                })),
            );

            // Search bar.
            ui.begin(Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(46.0), pad: Sides::xy(14.0, 0.0), gap: 10.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let ic = ui.color(Token::TextMuted);
            ui.icon_leaf("search", 17.0, ic);
            let mut o = EditOpts::new(ui);
            o.placeholder = placeholder;
            o.style.size = (14.0 * ui.look().text_scale * 4.0).round() / 4.0;
            o.style.line_height = (22.0 * ui.look().text_scale).round();
            let ed = ui.inline_edit(edit_id, &mut query, o);
            if just_opened {
                ui.inline_edit_caret_end(edit_id, &query);
                ui.request_focus(edit_id);
            }
            if mini_kbd_button(ui, id.with("esc"), "Esc") {
                esc_clicked = true;
            }
            ui.end();

            let line = ui.color(Token::BorderSubtle);
            ui.leaf(Style { width: Size::FILL, ..Default::default() }, vec2(0.0, 1.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.rect(r, line))));

            // Filter commands.
            let q_trimmed = query.trim();
            let mut matches: Vec<(usize, Vec<Range<usize>>, i32)> = commands
                .iter()
                .enumerate()
                .filter_map(|(i, c)| {
                    if q_trimmed.is_empty() {
                        return Some((i, vec![], 0));
                    }
                    if let Some(hl) = fuzzy_match(c.label, q_trimmed) {
                        return Some((i, hl.clone(), match_score(&hl) + 200));
                    }
                    let extra = format!("{} {}", c.group, c.keywords);
                    fuzzy_match(&extra, q_trimmed).map(|hl| (i, vec![], match_score(&hl)))
                })
                .collect();
            if !q_trimmed.is_empty() {
                matches.sort_by_key(|(i, _, sc)| (-*sc, *i));
            }
            if ed.changed {
                active = 0;
            }
            if !matches.is_empty() {
                let n = matches.len();
                if ed.key(Key::ArrowDown) {
                    active = (active + 1) % n;
                }
                if ed.key(Key::ArrowUp) {
                    active = (active + n - 1) % n;
                }
                if ed.key(Key::Home) {
                    active = 0;
                }
                if ed.key(Key::End) {
                    active = n - 1;
                }
                active = active.min(n - 1);
                if ed.key(Key::Enter) {
                    chosen = Some(commands[matches[active].0].id);
                }
            } else {
                active = 0;
            }
            if ed.key(Key::Escape) {
                esc_clicked = true;
            }

            // Results list.
            if matches.is_empty() {
                let muted = ui.color(Token::TextMuted);
                let st = ui.text_style(TextRole::Body);
                ui.begin(Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(64.0), gap: 8.0, align: Align::Center, justify: Justify::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                ui.icon_leaf("search-x", 16.0, muted);
                ui.text_leaf(format!("No commands matching “{q_trimmed}”"), st, muted, false, true, Style::default());
                ui.end();
            } else {
                let pointer = ui.pointer();
                let moved = {
                    let m = ui.mem::<PaletteMem>(id);
                    let mv = m.pointer.is_some() && m.pointer != pointer;
                    m.pointer = pointer;
                    mv
                };
                let row_h = (ui.look().control.height[1]).max(34.0);
                let head_h = 24.0;
                let show_groups = q_trimmed.is_empty() && commands.iter().any(|c| !c.group.is_empty());

                // Compute total content height and active row y-offset for scroll clamping.
                let mut y_cursor = 0.0f32;
                let mut active_band = (0.0f32, row_h);
                let mut prev_g = "";
                for (pos, &(ci, _, _)) in matches.iter().enumerate() {
                    let g = commands[ci].group;
                    if show_groups && !g.is_empty() && g != prev_g {
                        if pos > 0 {
                            y_cursor += 4.0;
                        }
                        y_cursor += head_h;
                        prev_g = g;
                    }
                    if pos == active {
                        active_band = (y_cursor, y_cursor + row_h);
                    }
                    y_cursor += row_h + 2.0;
                }
                let total_h = (y_cursor - 2.0).max(row_h) + 12.0;
                let max_view = 308.0f32;
                let scrolls = total_h > max_view;
                let view_h = total_h.min(max_view);
                let scroll_id = ui.scroll_id((id, "results"));
                if scrolls && ui.mem::<PaletteMem>(id).last_active != Some(active) {
                    let off = ui.mem::<PaletteMem>(id).offset;
                    if active_band.0 < off {
                        ui.scroll_to(scroll_id, active_band.0);
                    } else if active_band.1 + 12.0 > off + view_h {
                        ui.scroll_to(scroll_id, active_band.1 + 12.0 - view_h);
                    }
                }
                ui.mem::<PaletteMem>(id).last_active = Some(active);

                let rows_id = id.with("rows");
                let row_ids: Vec<Id> = matches.iter().map(|(ci, _, _)| id.with(("cmd", *ci))).collect();
                let target_y = ui.rect_of(row_ids[active]).zip(ui.rect_of(rows_id)).map(|(r, p)| r.min.y - p.min.y);
                let pill_y = match target_y {
                    Some(ty) => ui.spring(id, 50, ty, mo.snappy),
                    None => active_band.0 + 6.0,
                };
                let mac = ui.look().platform == Platform::Mac;
                let pill_c = if mac { ui.color(Token::Accent) } else { ui.color(Token::Hover).over(ui.color(Token::Hover)) };
                let r_opt = (ui.look().control.radius[1] - 1.0).max(4.0);

                let build_list = |ui: &mut Ui, active: &mut usize, chosen: &mut Option<&'a str>| {
                    ui.begin(
                        Style { width: Size::FILL, pad: Sides::all(6.0), gap: 2.0, ..Default::default() },
                        Some(rows_id),
                        Sense::NONE,
                        CursorIcon::Default,
                        false,
                        Some(Box::new(move |cx, r| {
                            let pr = Rect::from_xywh(r.min.x + 6.0, r.min.y + pill_y, r.width() - 12.0, row_h);
                            cx.painter.rrect_unsnapped(pr, Corners::all(r_opt), [pill_c; 2], 0.0, Color::TRANSPARENT);
                        })),
                    );
                    let mut prev_group = "";
                    for (pos, (ci, hl, _)) in matches.iter().enumerate() {
                        let cmd = commands[*ci];
                        if show_groups && !cmd.group.is_empty() && cmd.group != prev_group {
                            let cap = ui.text_style(TextRole::Caption);
                            let mc = ui.color(Token::TextMuted);
                            ui.begin(Style { height: Size::Fixed(head_h), pad: Sides { left: 10.0, top: if pos > 0 { 6.0 } else { 2.0 }, ..Default::default() }, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                            ui.text_leaf(cmd.group, cap, mc, false, true, Style::default());
                            ui.end();
                            prev_group = cmd.group;
                        }
                        let rid = row_ids[pos];
                        let rix = ui.interact(rid, Sense { click: true, drag: false, focusable: false, scroll: false }, false);
                        if rix.hovered && moved {
                            *active = pos;
                        }
                        if rix.clicked {
                            *chosen = Some(cmd.id);
                        }
                        let hot = *active == pos;
                        let hh = ui.fade(rid, 1, hot, 0.0, mo.hover_out, Ease::Linear);
                        let enter = ui.anims.spring_from(rid, 2, 0.0, 1.0, SpringParams::from_response(0.18 + 0.025 * pos.min(8) as f32, 1.0)).clamp(0.0, 1.0);
                        let fg = ui.color(Token::Foreground);
                        let fg_hot = if mac { ui.color(Token::AccentForeground) } else { fg };
                        let text_c = fg.mix(fg_hot, hh).multiply_alpha(enter);
                        let hl_c = (if mac { fg_hot } else { ui.color(Token::AccentStrong) }).multiply_alpha(enter);
                        let tint = if mac { Color::TRANSPARENT } else { ui.color(Token::AccentSubtle).multiply_alpha(0.7 * enter) };
                        let ic_c = ui.color(Token::TextSecondary).mix(fg_hot, if mac { hh } else { 0.0 }).multiply_alpha(enter);
                        let body = ui.text_style(TextRole::Body);
                        let g = ui.galley(cmd.label, &body);
                        let label_owned = cmd.label.to_owned();
                        let hl_owned = hl.clone();

                        ui.begin(
                            Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(row_h), pad: Sides::xy(10.0, 0.0), gap: 10.0, align: Align::Center, ..Default::default() },
                            Some(rid),
                            Sense { click: true, drag: false, focusable: false, scroll: false },
                            CursorIcon::Pointer,
                            false,
                            None,
                        );
                        if !cmd.icon.is_empty() {
                            ui.icon_leaf(cmd.icon, 16.0, ic_c);
                        }
                        let rise_row = (1.0 - enter) * 3.0;
                        ui.leaf(
                            Style { width: Size::FILL, ..Default::default() },
                            vec2(g.width.min(280.0), g.height),
                            None,
                            Sense::NONE,
                            CursorIcon::Default,
                            Some(Box::new(move |cx, r| {
                                cx.painter.push_clip(r.expand2(2.0, 4.0));
                                let p = vec2(r.min.x, (r.center().y - g.height / 2.0 + rise_row).round());
                                paint_highlighted(cx, &label_owned, &body, p, text_c, &hl_owned, hl_c, tint);
                                cx.painter.pop_clip();
                            })),
                        );
                        if !cmd.group.is_empty() && !show_groups {
                            let cap = ui.text_style(TextRole::Caption);
                            let gc = ui.color(Token::TextMuted).mix(fg_hot.multiply_alpha(0.8), if mac { hh } else { 0.0 }).multiply_alpha(enter);
                            ui.text_leaf(cmd.group, cap, gc, false, true, Style::default());
                        }
                        if !cmd.shortcut.is_empty() {
                            let kbd_bg = ui.color(Token::SurfaceMuted).multiply_alpha(if mac { 1.0 - 0.6 * hh } else { 1.0 } * enter);
                            let kbd_b = ui.color(Token::BorderSubtle).multiply_alpha(enter);
                            let kbd_fg = ui.color(Token::TextSecondary).mix(fg_hot, if mac { hh } else { 0.0 }).multiply_alpha(enter);
                            mini_kbd_chip(ui, cmd.shortcut, kbd_bg, kbd_b, kbd_fg);
                        }
                        ui.end();
                    }
                    ui.end();
                };

                if scrolls {
                    ui.begin(Style { width: Size::FILL, height: Size::Fixed(view_h), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                    let info = ui.scroll((id, "results")).show(|ui| build_list(ui, &mut active, &mut chosen)).1;
                    ui.end();
                    ui.mem::<PaletteMem>(id).offset = info.offset;
                } else {
                    build_list(ui, &mut active, &mut chosen);
                }
            }

            // Footer hints.
            let line = ui.color(Token::BorderSubtle);
            ui.leaf(Style { width: Size::FILL, ..Default::default() }, vec2(0.0, 1.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.rect(r, line))));
            ui.begin(Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(30.0), pad: Sides::xy(14.0, 0.0), gap: 16.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let cap = ui.text_style(TextRole::Caption);
            let mc = ui.color(Token::TextMuted);
            ui.text_leaf("↑↓ Navigate", cap, mc, false, false, Style::default());
            ui.text_leaf("↵ Run", cap, mc, false, false, Style::default());
            ui.text_leaf("Esc Close", cap, mc, false, false, Style::default());
            ui.end();

            ui.end();
        });

        if esc_clicked || chosen.is_some() || ui.take_layer_close() {
            *open = false;
            closed = true;
        }
        {
            let m = ui.mem::<PaletteMem>(id);
            m.query = query;
            m.active = active;
            if closed {
                if let Some(prev) = m.prev_focus.take() {
                    ui.request_focus(prev);
                }
            }
        }
        chosen
    }
}

fn mini_kbd_chip(ui: &mut Ui, text: &str, bg: Color, border: Color, fg: Color) {
    let mut st = ui.text_style(TextRole::Mono);
    st.size = (11.0 * ui.look().text_scale).round();
    st.line_height = 16.0;
    let g = ui.galley(text, &st);
    let w = (g.width + 10.0).max(20.0);
    ui.leaf(
        Style::default(),
        vec2(w, 20.0),
        None,
        Sense::NONE,
        CursorIcon::Default,
        Some(Box::new(move |cx, r| {
            cx.painter.rrect(r, Corners::all(4.0), bg, 1.0, border);
            cx.painter.text(cx.text, &g, vec2((r.center().x - g.width / 2.0).round(), (r.center().y - g.height / 2.0).round()), fg);
        })),
    );
}

fn mini_kbd_button(ui: &mut Ui, id: Id, text: &str) -> bool {
    let ix = ui.interact(id, Sense { click: true, drag: false, focusable: false, scroll: false }, false);
    let mo = ui.motion();
    let h = ui.fade(id, 0, ix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
    let press = ui.spring(id, 1, if ix.pressed { 1.0 } else { 0.0 }, mo.snappy);
    let bg = ui.color(Token::SurfaceMuted).mix(ui.color(Token::SurfaceMuted).over(ui.color(Token::Hover)), h);
    let border = ui.color(Token::Border);
    let fg = ui.color(Token::TextMuted).mix(ui.color(Token::Foreground), h);
    let mut st = ui.text_style(TextRole::Mono);
    st.size = (11.0 * ui.look().text_scale).round();
    st.line_height = 16.0;
    let g = ui.galley(text, &st);
    let w = (g.width + 10.0).max(24.0);
    ui.leaf(
        Style::default(),
        vec2(w, 22.0),
        Some(id),
        Sense { click: true, drag: false, focusable: false, scroll: false },
        CursorIcon::Pointer,
        Some(Box::new(move |cx, r| {
            let r = Rect::from_center_size(r.center(), r.size() * (1.0 - 0.05 * press));
            cx.painter.rrect_unsnapped(r, Corners::all(4.0), [bg; 2], 1.0, border);
            cx.painter.text(cx.text, &g, vec2((r.center().x - g.width / 2.0).round(), (r.center().y - g.height / 2.0).round()), fg);
        })),
    );
    ix.clicked
}

impl Ui {
    /// A modal command palette (`⌘K` / `Ctrl+K` by default) that fuzzy-filters
    /// `commands`, highlights matches, and returns the chosen command's `id`.
    pub fn command_palette<'u, 'a>(&'u mut self, key: impl Into<String>, open: &'u mut bool, commands: &'a [Command<'a>]) -> CommandPalette<'u, 'a> {
        CommandPalette { ui: self, key: key.into(), open, commands, placeholder: "Type a command or search…".into(), width: 500.0, global_shortcut: true }
    }
}
