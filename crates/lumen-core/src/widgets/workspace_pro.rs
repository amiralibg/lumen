//! Desktop workspace & developer-inspector widgets:
//! - [`SplitPane`] (resizable horizontal or vertical split container with double-click reset & keyboard step)
//! - [`MenuBarScope`] (desktop File / Edit / View menu bar with hover-across switching)
//! - [`Terminal`] (interactive monospace command shell with history and semantic output tones)
//! - [`PropertyInspector`] (compact 2-column Xcode/Figma-style inspector rows)
//! - [`Ui::flame_graph`] (interactive click-to-zoom CPU / GPU profiler icicle flame chart)

use std::ops::RangeInclusive;

use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::color::Color;
use crate::id::Id;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides, Vec2};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};

use super::chart::{tooltip_panel, Tip};
use super::field::EditOpts;
use super::key_activate;
use super::popover::PopState;

// ============================================================== SplitPane

/// Resizable two-pane container. Drag the divider (or press arrow keys when
/// focused) to adjust `ratio` (`0.15..=0.85`); double-click resets to `0.5`.
pub struct SplitPane<'u> {
    ui: &'u mut Ui,
    key: String,
    ratio: &'u mut f32,
    vertical: bool,
    height: f32,
}

impl<'u> SplitPane<'u> {
    /// Stack panes top-to-bottom instead of left-to-right.
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }
    pub fn height(mut self, px: f32) -> Self {
        self.height = px.max(120.0);
        self
    }

    pub fn show(self, first: impl FnOnce(&mut Ui), second: impl FnOnce(&mut Ui)) -> Response {
        let SplitPane { ui, key, ratio, vertical, height } = self;
        let id = ui.make_id(("split_pane", &key));
        let handle_id = id.with("handle");
        let mo = ui.motion();

        *ratio = ratio.clamp(0.16, 0.84);
        let before = *ratio;

        let container_rect = ui.rect_of(id).unwrap_or(Rect::from_xywh(0.0, 0.0, 520.0, height));
        let sense_drag = Sense { click: true, drag: true, focusable: true, scroll: false };
        let hix = ui.interact(handle_id, sense_drag, false);
        ui.a11y(handle_id, || A11y::new(Role::Separator).label("Resize").numeric(*ratio as f64 * 100.0, 0.0, 100.0, Some(5.0)));
        let (a_st, a_set) = ui.a11y_step(handle_id);
        if a_st != 0 || a_set.is_some() {
            *ratio = ((a_set.map(|v| v as f32 / 100.0).unwrap_or(*ratio)) + a_st as f32 * 0.05).clamp(0.05, 0.95);
        }

        if hix.clicked && hix.click_count >= 2 {
            *ratio = 0.5;
        } else if hix.held {
            if let Some(p) = hix.pointer {
                let raw = if vertical {
                    (p.y - container_rect.min.y) / container_rect.height().max(1.0)
                } else {
                    (p.x - container_rect.min.x) / container_rect.width().max(1.0)
                };
                *ratio = raw.clamp(0.16, 0.84);
            }
        }
        if hix.focused {
            for (k, m, _) in ui.keys_for(handle_id) {
                let step = if m.shift { 0.08 } else { 0.03 };
                match k {
                    Key::ArrowRight if !vertical => *ratio = (*ratio + step).clamp(0.16, 0.84),
                    Key::ArrowLeft if !vertical => *ratio = (*ratio - step).clamp(0.16, 0.84),
                    Key::ArrowDown if vertical => *ratio = (*ratio + step).clamp(0.16, 0.84),
                    Key::ArrowUp if vertical => *ratio = (*ratio - step).clamp(0.16, 0.84),
                    Key::Home => *ratio = 0.5,
                    _ => {}
                }
            }
        }

        let r_anim = if hix.held {
            ui.anims.set_spring(id, 0, *ratio);
            *ratio
        } else {
            ui.anims.spring_from(id, 0, *ratio, *ratio, mo.snappy).clamp(0.15, 0.85)
        };

        let active_t = ui.fade(handle_id, 1, hix.hovered || hix.held || hix.focused, mo.hover_in, mo.hover_out, Ease::Linear);
        let (ring_a, ring_gap) = ui.focus_anim(&hix);

        let r_card = ui.look().card.radius;
        let bg_c = ui.color(Token::Surface);
        let bd_c = ui.color(Token::BorderSubtle);
        let line_c = ui.color(Token::BorderSubtle).mix(ui.color(Token::Accent), active_t);
        let pill_c = ui.color(Token::BorderStrong).mix(ui.color(Token::Accent), active_t);
        let ring_c = ui.color(Token::FocusRing);

        let dir = if vertical { Dir::Column } else { Dir::Row };
        ui.begin(
            Style {
                dir,
                width: Size::FILL,
                height: Size::Fixed(height),
                ..Default::default()
            },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        // First pane.
        let w1 = r_anim.max(0.05);
        let w2 = (1.0 - r_anim).max(0.05);
        ui.begin(
            Style {
                dir: Dir::Column,
                width: if vertical { Size::FILL } else { Size::Fill(w1) },
                height: if vertical { Size::Fill(w1) } else { Size::FILL },
                pad: Sides::all(14.0),
                gap: 8.0,
                ..Default::default()
            },
            None,
            Sense::NONE,
            CursorIcon::Default,
            true,
            None,
        );
        ui.scope("pane_a", first);
        ui.end();

        // Resizer sash.
        let cursor = if vertical { CursorIcon::Grab } else { CursorIcon::ResizeHorizontal };
        let sash_sz = if vertical { vec2(0.0, 9.0) } else { vec2(9.0, 0.0) };
        ui.leaf(
            Style { self_align: Some(Align::Stretch), ..Default::default() },
            sash_sz,
            Some(handle_id),
            sense_drag,
            cursor,
            Some(Box::new(move |cx, r| {
                let c = r.center();
                if vertical {
                    let thick = 1.0 + active_t;
                    cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, c.y - thick * 0.5, (r.width() - 2.0).max(0.0), thick), line_c);
                    let pill = Rect::from_center_size(c, vec2(28.0, 4.0));
                    cx.painter.rrect(pill, Corners::all(2.0), pill_c, 0.0, Color::TRANSPARENT);
                } else {
                    let thick = 1.0 + active_t;
                    cx.painter.rect(Rect::from_xywh(c.x - thick * 0.5, r.min.y + 1.0, thick, (r.height() - 2.0).max(0.0)), line_c);
                    let pill = Rect::from_center_size(c, vec2(4.0, 28.0));
                    cx.painter.rrect(pill, Corners::all(2.0), pill_c, 0.0, Color::TRANSPARENT);
                }
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, 4.0, ring_a, ring_gap, ring_c)));
                }
            })),
        );

        // Second pane.
        ui.begin(
            Style {
                dir: Dir::Column,
                width: if vertical { Size::FILL } else { Size::Fill(w2) },
                height: if vertical { Size::Fill(w2) } else { Size::FILL },
                pad: Sides::all(14.0),
                gap: 8.0,
                ..Default::default()
            },
            None,
            Sense::NONE,
            CursorIcon::Default,
            true,
            None,
        );
        ui.scope("pane_b", second);
        ui.end();

        ui.end();
        let changed = (*ratio - before).abs() > 1e-5;
        Response { id, rect: container_rect, hovered: hix.hovered, clicked: hix.clicked, changed, focused: hix.focused }
    }
}

// =============================================================== MenuBar

#[derive(Default)]
struct MenuBarMem {
    triggers: Vec<Id>,
}

/// Scope passed to [`Ui::menu_bar`].
pub struct MenuBarScope<'u> {
    pub ui: &'u mut Ui,
    bar_id: Id,
    any_open_prev: Option<Id>,
    triggers: Vec<Id>,
}

impl MenuBarScope<'_> {
    /// Add a top-level menu trigger (`"File"`, `"Edit"`, `"View"`, …).
    pub fn menu(&mut self, label: &str, items: impl FnOnce(&mut Ui)) {
        let tid = self.bar_id.with(("mb_trig", label));
        self.triggers.push(tid);
        let ix = self.ui.interact(tid, Sense::CLICK, false);
        let (k_act, _) = key_activate(self.ui, &ix, true);

        // Hover-across switching when another menu in the same bar is already open!
        if let Some(open_id) = self.any_open_prev {
            if open_id != tid && ix.hovered {
                self.ui.mem::<PopState>(open_id).open = false;
                let st = self.ui.mem::<PopState>(tid);
                st.open = true;
                st.just_opened = true;
                self.any_open_prev = Some(tid);
            }
        }

        let is_open = self.ui.mem_get::<PopState>(tid).is_some_and(|s| s.open);
        let mo = self.ui.motion();
        let h = self.ui.fade(tid, 0, ix.hovered || is_open, mo.hover_in, mo.hover_out, Ease::Linear);
        let (ring_a, ring_gap) = self.ui.focus_anim(&ix);

        let pill_c = if is_open {
            self.ui.color(Token::AccentSubtle)
        } else {
            self.ui.color(Token::Hover).multiply_alpha(h)
        };
        let fg_c = if is_open {
            self.ui.color(Token::AccentStrong)
        } else {
            self.ui.color(Token::Foreground)
        };
        let ring_c = self.ui.color(Token::FocusRing);

        self.ui.begin(
            Style { dir: Dir::Row, pad: Sides::xy(10.0, 5.0), align: Align::Center, ..Default::default() },
            Some(tid),
            Sense::CLICK,
            CursorIcon::Pointer,
            false,
            Some(Box::new(move |cx, r| {
                if pill_c.a > 0.0 {
                    cx.painter.rrect(r, Corners::all(6.0), pill_c, 0.0, Color::TRANSPARENT);
                }
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, 6.0, ring_a, ring_gap, ring_c)));
                }
            })),
        );
        let mut st = self.ui.text_style(TextRole::Body);
        st.weight = 500;
        self.ui.text_leaf(label.to_owned(), st, fg_c, false, false, Style::default());
        self.ui.end();

        let anchor = Response {
            id: tid,
            rect: self.ui.rect_of(tid).unwrap_or(ix.rect),
            hovered: ix.hovered,
            clicked: ix.clicked || k_act,
            changed: false,
            focused: ix.focused,
        };
        self.ui.menu(&anchor, items);
    }
}

// =============================================================== Terminal

/// Semantic tone of a [`TermLine`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TermTone {
    Command,
    Output,
    Success,
    Warn,
    Error,
}

/// Line in an interactive [`Terminal`].
#[derive(Clone, Debug)]
pub struct TermLine {
    pub tone: TermTone,
    pub text: String,
}

impl TermLine {
    pub fn cmd(s: impl Into<String>) -> Self {
        TermLine { tone: TermTone::Command, text: s.into() }
    }
    pub fn out(s: impl Into<String>) -> Self {
        TermLine { tone: TermTone::Output, text: s.into() }
    }
    pub fn ok(s: impl Into<String>) -> Self {
        TermLine { tone: TermTone::Success, text: s.into() }
    }
    pub fn warn(s: impl Into<String>) -> Self {
        TermLine { tone: TermTone::Warn, text: s.into() }
    }
    pub fn err(s: impl Into<String>) -> Self {
        TermLine { tone: TermTone::Error, text: s.into() }
    }
}

#[derive(Default)]
struct TermMem {
    history: Vec<String>,
    hist_cursor: Option<usize>,
}

/// Interactive lumen-drawn terminal console with command history (`↑`/`↓`)
/// and live `$` prompt. Returns `Some(command)` when Enter is pressed.
pub struct Terminal<'u> {
    ui: &'u mut Ui,
    key: String,
    lines: &'u [TermLine],
    input: &'u mut String,
    title: String,
    height: f32,
}

impl<'u> Terminal<'u> {
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = t.into();
        self
    }
    pub fn height(mut self, px: f32) -> Self {
        self.height = px.max(120.0);
        self
    }

    pub fn show(self) -> Option<String> {
        let Terminal { ui, key, lines, input, title, height } = self;
        let id = ui.make_id(("terminal", &key));
        let ed_id = id.with("cmd_ed");
        let r_card = ui.look().card.radius;
        let top_rad = (r_card - 1.0).max(0.0);
        let card_bg = ui.color(Token::Surface);
        let card_bd = ui.color(Token::BorderSubtle);
        let hdr_bg = ui.color(Token::SurfaceMuted).multiply_alpha(0.65);
        let div_c = ui.color(Token::BorderSubtle);

        if ui.interact(id.with("click_focus"), Sense { click: true, drag: false, focusable: false, scroll: false }, false).clicked {
            ui.request_focus(ed_id);
        }

        ui.begin(
            Style { width: Size::FILL, dir: Dir::Column, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), card_bg, 1.0, card_bd))),
        );

        // Header bar.
        ui.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(12.0, 8.0), gap: 8.0, align: Align::Center, ..Default::default() },
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
        let ic_c = ui.color(Token::TextMuted);
        ui.icon_leaf("terminal", 14.0, ic_c);
        let mut st_t = ui.text_style(TextRole::Mono);
        st_t.weight = 600;
        let hdr_fg = ui.color(Token::TextSecondary);
        ui.text_leaf(title, st_t, hdr_fg, false, true, Style { width: Size::FILL, ..Default::default() });
        ui.end();

        // Scrollable output + prompt.
        let mut submitted: Option<String> = None;
        ui.begin(
            Style { width: Size::FILL, height: Size::Fixed(height), dir: Dir::Column, ..Default::default() },
            Some(id.with("click_focus")),
            Sense { click: true, drag: false, focusable: false, scroll: false },
            CursorIcon::Text,
            true,
            None,
        );
        ui.scroll(id.with("term_scroll")).pad(Sides::xy(12.0, 10.0)).gap(4.0).show(|ui| {
            let mut mono = ui.text_style(TextRole::Mono);
            mono.line_height = 19.0;
            for line in lines {
                let col = match line.tone {
                    TermTone::Command => ui.color(Token::Foreground),
                    TermTone::Output => ui.color(Token::TextSecondary),
                    TermTone::Success => ui.color(Token::SuccessStrong),
                    TermTone::Warn => ui.color(Token::WarningStrong),
                    TermTone::Error => ui.color(Token::DangerStrong),
                };
                if line.tone == TermTone::Command {
                    let prompt_c = ui.color(Token::AccentStrong);
                    ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                    ui.text_leaf("$", mono, prompt_c, false, false, Style::default());
                    let mut st_b = mono;
                    st_b.weight = 600;
                    ui.text_leaf(line.text.clone(), st_b, col, true, false, Style { width: Size::FILL, ..Default::default() });
                    ui.end();
                } else {
                    ui.text_leaf(line.text.clone(), mono, col, true, false, Style { width: Size::FILL, ..Default::default() });
                }
            }

            // Live input prompt row.
            let prompt_c = ui.color(Token::AccentStrong);
            ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, pad: Sides { top: 2.0, ..Default::default() }, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Text, false, None);
            ui.text_leaf("$", mono, prompt_c, false, false, Style::default());
            let mut opts = EditOpts::new(ui);
            opts.style = mono;
            opts.placeholder = "Type a command (help, status, clear)…".into();
            let ed_out = ui.inline_edit(ed_id, input, opts);
            if ed_out.key(Key::Enter) {
                let cmd = input.trim().to_owned();
                if !cmd.is_empty() {
                    let mem = ui.mem::<TermMem>(id);
                    mem.history.push(cmd.clone());
                    mem.hist_cursor = None;
                    input.clear();
                    submitted = Some(cmd);
                }
            } else if ed_out.key(Key::ArrowUp) {
                let mem = ui.mem::<TermMem>(id);
                if !mem.history.is_empty() {
                    let next = mem.hist_cursor.map(|c| c.saturating_sub(1)).unwrap_or(mem.history.len() - 1);
                    mem.hist_cursor = Some(next);
                    *input = mem.history[next].clone();
                    ui.inline_edit_caret_end(ed_id, input);
                }
            } else if ed_out.key(Key::ArrowDown) {
                let mem = ui.mem::<TermMem>(id);
                if let Some(c) = mem.hist_cursor {
                    if c + 1 < mem.history.len() {
                        mem.hist_cursor = Some(c + 1);
                        *input = mem.history[c + 1].clone();
                        ui.inline_edit_caret_end(ed_id, input);
                    } else {
                        mem.hist_cursor = None;
                        input.clear();
                    }
                }
            }
            ui.end();
        });
        ui.end();

        ui.end();
        submitted
    }
}

// ====================================================== PropertyInspector

/// Compact two-column Xcode / Figma-style inspector scope.
pub struct PropertyInspector<'u> {
    pub ui: &'u mut Ui,
    row_idx: usize,
}

impl PropertyInspector<'_> {
    pub fn section(&mut self, title: &str) {
        let c = self.ui.color(Token::TextMuted);
        let div = self.ui.color(Token::BorderSubtle);
        let is_first = self.row_idx == 0;
        self.row_idx += 1;
        self.ui.begin(
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                pad: Sides { left: 12.0, right: 12.0, top: if is_first { 8.0 } else { 12.0 }, bottom: 4.0 },
                align: Align::Center,
                ..Default::default()
            },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            if !is_first {
                Some(Box::new(move |cx, r| cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.min.y + 2.0, (r.width() - 2.0).max(0.0), 1.0), div)))
            } else {
                None
            },
        );
        let mut st = self.ui.text_style(TextRole::Caption);
        st.weight = 600;
        self.ui.text_leaf(title.to_uppercase(), st, c, false, false, Style::default());
        self.ui.end();
    }

    pub fn row(&mut self, label: &str, control: impl FnOnce(&mut Ui)) {
        let idx = self.row_idx;
        self.row_idx += 1;
        let lbl_c = self.ui.color(Token::TextSecondary);
        self.ui.begin(
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                pad: Sides::xy(12.0, 5.0),
                gap: 10.0,
                align: Align::Center,
                ..Default::default()
            },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );
        let st = self.ui.text_style(TextRole::Label);
        self.ui.text_leaf(label.to_owned(), st, lbl_c, false, true, Style { width: Size::Fixed(96.0), ..Default::default() });
        self.ui.begin(
            Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, align: Align::Center, justify: Justify::End, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );
        self.ui.scope(("prop_r", idx), control);
        self.ui.end();
        self.ui.end();
    }

    pub fn number(&mut self, label: &str, val: &mut f64, range: RangeInclusive<f64>, suffix: &str) {
        self.row(label, |ui| {
            ui.number_field(val).range(range).step(1.0).suffix(suffix).controls(false).fill_width().show();
        });
    }

    pub fn vec2_row(&mut self, label: &str, x: &mut f64, y: &mut f64) {
        self.row(label, |ui| {
            ui.number_field(x).prefix("X").step(1.0).controls(false).fill_width().show();
            ui.number_field(y).prefix("Y").step(1.0).controls(false).fill_width().show();
        });
    }

    pub fn toggle(&mut self, label: &str, on: &mut bool) {
        self.row(label, |ui| {
            ui.switch("", on);
        });
    }
}

// ============================================================= FlameGraph

/// Stack frame span in [`Ui::flame_graph`].
#[derive(Clone, Debug)]
pub struct FlameSpan {
    pub name: String,
    pub depth: usize,
    pub start_ms: f32,
    pub duration_ms: f32,
}

impl FlameSpan {
    pub fn new(name: impl Into<String>, depth: usize, start_ms: f32, duration_ms: f32) -> Self {
        FlameSpan { name: name.into(), depth, start_ms, duration_ms }
    }
}

#[derive(Clone, Copy)]
struct FlameZoom {
    start: f32,
    end: f32,
}

impl Default for FlameZoom {
    fn default() -> Self {
        FlameZoom { start: 0.0, end: -1.0 }
    }
}

impl Ui {
    /// Resizable horizontal or vertical split view.
    pub fn split_pane<'u>(&'u mut self, key: impl Into<String>, ratio: &'u mut f32) -> SplitPane<'u> {
        SplitPane { ui: self, key: key.into(), ratio, vertical: false, height: 220.0 }
    }

    /// Classic desktop menu bar (`File`, `Edit`, `View`, …) with hover-across menu switching.
    pub fn menu_bar(&mut self, key: impl std::hash::Hash, f: impl FnOnce(&mut MenuBarScope<'_>)) -> Response {
        let id = self.make_id(("menu_bar", key));
        let prev_triggers = self.mem::<MenuBarMem>(id).triggers.clone();
        let any_open_prev = prev_triggers.iter().copied().find(|&tid| self.mem_get::<PopState>(tid).is_some_and(|s| s.open));

        // Left/Right arrow navigation across focused menu triggers.
        if let Some(cur) = self.focused() {
            if let Some(pos) = prev_triggers.iter().position(|&t| t == cur) {
                let len = prev_triggers.len();
                if len > 0 {
                    if self.key_pressed_for(cur, Key::ArrowRight) {
                        self.request_focus(prev_triggers[(pos + 1) % len]);
                    } else if self.key_pressed_for(cur, Key::ArrowLeft) {
                        self.request_focus(prev_triggers[(pos + len - 1) % len]);
                    }
                }
            }
        }

        let r_bar = (self.look().card.radius - 2.0).max(8.0);
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);

        self.begin(
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                pad: Sides::xy(6.0, 4.0),
                gap: 2.0,
                align: Align::Center,
                ..Default::default()
            },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_bar), bg_c, 1.0, bd_c))),
        );
        let mut scope = MenuBarScope { ui: self, bar_id: id, any_open_prev, triggers: Vec::new() };
        f(&mut scope);
        let recorded = scope.triggers;
        self.end();

        self.mem::<MenuBarMem>(id).triggers = recorded;
        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Interactive monospace terminal shell with command history.
    pub fn terminal<'u>(
        &'u mut self,
        key: impl Into<String>,
        lines: &'u [TermLine],
        input: &'u mut String,
    ) -> Terminal<'u> {
        Terminal { ui: self, key: key.into(), lines, input, title: "zsh — lumen".into(), height: 196.0 }
    }

    /// Compact two-column inspector card for editing object properties.
    pub fn property_inspector(&mut self, key: impl std::hash::Hash, f: impl FnOnce(&mut PropertyInspector<'_>)) -> Response {
        let id = self.make_id(("prop_inspector", key));
        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);

        self.begin(
            Style {
                dir: Dir::Column,
                width: Size::FILL,
                pad: Sides::xy(0.0, 6.0),
                gap: 2.0,
                ..Default::default()
            },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );
        let mut insp = PropertyInspector { ui: self, row_idx: 0 };
        f(&mut insp);
        self.end();
        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Interactive CPU / GPU icicle flame graph. Click any frame to smoothly
    /// zoom into its time range; click `"Reset zoom"` to spring back out.
    pub fn flame_graph(&mut self, key: impl std::hash::Hash, spans: &[FlameSpan]) -> Response {
        let id = self.make_id(("flame_graph", key));
        let canvas_id = id.with("canvas");
        let mo = self.motion();

        let total_ms = spans.iter().map(|s| s.start_ms + s.duration_ms).fold(1.0f32, f32::max);
        let max_depth = spans.iter().map(|s| s.depth).max().unwrap_or(0);

        let mut zoom = *self.mem::<FlameZoom>(id);
        if zoom.end <= zoom.start {
            zoom = FlameZoom { start: 0.0, end: total_ms };
            *self.mem::<FlameZoom>(id) = zoom;
        }
        let is_zoomed = zoom.start > 0.01 || (zoom.end - total_ms).abs() > 0.01;

        let v_start = self.anims.spring_from(id, 0, zoom.start, zoom.start, mo.smooth);
        let v_end = self.anims.spring_from(id, 1, zoom.end, zoom.end, mo.smooth).max(v_start + 0.01);

        let r_card = self.look().card.radius;
        let top_rad = (r_card - 1.0).max(0.0);
        let card_bg = self.color(Token::Surface);
        let card_bd = self.color(Token::BorderSubtle);
        let hdr_bg = self.color(Token::SurfaceMuted).multiply_alpha(0.6);
        let div_c = self.color(Token::BorderSubtle);

        self.begin(
            Style { width: Size::FILL, dir: Dir::Column, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), card_bg, 1.0, card_bd))),
        );

        // Header bar with total frame time + Reset zoom button.
        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(12.0, 7.0), gap: 8.0, align: Align::Center, ..Default::default() },
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
        let ic_c = self.color(Token::WarningStrong);
        self.icon_leaf("flame", 14.0, ic_c);
        let mut st_h = self.text_style(TextRole::Caption);
        st_h.weight = 600;
        let hdr_c = self.color(Token::TextSecondary);
        self.text_leaf(
            format!("Frame profile · {:.2} ms window", v_end - v_start),
            st_h,
            hdr_c,
            false,
            true,
            Style { width: Size::FILL, ..Default::default() },
        );
        if is_zoomed && self.scope("rst_zm", |ui| ui.button("Reset zoom").ghost().small().icon("zoom-out").clicked()) {
            *self.mem::<FlameZoom>(id) = FlameZoom { start: 0.0, end: total_ms };
        }
        self.end();

        let ix = self.interact(canvas_id, Sense { click: true, drag: false, focusable: false, scroll: false }, false);
        let row_h = 26.0;
        let gap = 3.0;
        let pad = 10.0;
        let body_h = pad * 2.0 + (max_depth + 1) as f32 * row_h + max_depth as f32 * gap;

        // Check click on a span using previous frame's canvas rect!
        if ix.clicked && ix.rect.width() > 20.0 {
            if let Some(p) = ix.pointer {
                let inner_w = (ix.rect.width() - pad * 2.0).max(1.0);
                for sp in spans {
                    let x0 = ix.rect.min.x + pad + ((sp.start_ms - v_start) / (v_end - v_start)) * inner_w;
                    let x1 = ix.rect.min.x + pad + ((sp.start_ms + sp.duration_ms - v_start) / (v_end - v_start)) * inner_w;
                    let y0 = ix.rect.min.y + pad + sp.depth as f32 * (row_h + gap);
                    let sr = Rect { min: vec2(x0, y0), max: vec2(x1, y0 + row_h) };
                    if sr.contains(p) {
                        *self.mem::<FlameZoom>(id) = FlameZoom { start: sp.start_ms, end: sp.start_ms + sp.duration_ms };
                        break;
                    }
                }
            }
        }

        let palette = [
            self.color(Token::Series1),
            self.color(Token::Warning),
            self.color(Token::Danger).mix(self.color(Token::Warning), 0.45),
            self.color(Token::Series0),
            self.color(Token::Series2),
        ];
        let txt_on_bar = Color::WHITE;
        let hover_ring = self.color(Token::Foreground);
        let mut mono = self.text_style(TextRole::Mono);
        mono.size = (mono.size - 0.5).max(11.0);
        let tip = Tip::new(self);
        let spans_cl = spans.to_vec();
        let ptr = ix.pointer.filter(|_| ix.hovered);

        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(280.0, body_h),
            Some(canvas_id),
            Sense { click: true, drag: false, focusable: false, scroll: false },
            CursorIcon::Pointer,
            Some(Box::new(move |cx, r| {
                let inner_x = r.min.x + pad;
                let inner_w = (r.width() - pad * 2.0).max(1.0);
                let span_win = (v_end - v_start).max(1e-4);

                cx.painter.push_clip(r.expand(-2.0));
                let mut hov_idx: Option<(usize, Vec2)> = None;
                for (si, sp) in spans_cl.iter().enumerate() {
                    let x0 = inner_x + ((sp.start_ms - v_start) / span_win) * inner_w;
                    let x1 = inner_x + ((sp.start_ms + sp.duration_ms - v_start) / span_win) * inner_w;
                    let cx0 = x0.max(inner_x);
                    let cx1 = x1.min(inner_x + inner_w);
                    if cx1 - cx0 < 1.5 {
                        continue;
                    }
                    let y0 = r.min.y + pad + sp.depth as f32 * (row_h + gap);
                    let br = Rect::from_xywh(cx0, y0, (cx1 - cx0 - 1.0).max(1.5), row_h);
                    let col = palette[(sp.depth + si) % palette.len()];
                    let is_h = ptr.is_some_and(|p| br.contains(p));
                    cx.painter.rrect(
                        br,
                        Corners::all(5.0),
                        col.multiply_alpha(if is_h { 0.96 } else { 0.84 }),
                        if is_h { 1.2 } else { 0.0 },
                        hover_ring,
                    );
                    if br.width() > 34.0 {
                        let max_txt_w = br.width() - 12.0;
                        let full_lbl = format!("{} ({:.1} ms)", sp.name, sp.duration_ms);
                        let g_full = cx.galley(&full_lbl, &mono);
                        if g_full.width <= max_txt_w {
                            cx.painter.text(cx.text, &g_full, vec2(br.min.x + 6.0, br.min.y + (row_h - g_full.height) * 0.5), txt_on_bar);
                        } else {
                            let g_name = cx.galley(&sp.name, &mono);
                            if g_name.width <= max_txt_w {
                                cx.painter.text(cx.text, &g_name, vec2(br.min.x + 6.0, br.min.y + (row_h - g_name.height) * 0.5), txt_on_bar);
                            } else {
                                let chars: Vec<char> = sp.name.chars().collect();
                                let mut hi = chars.len();
                                while hi > 1 {
                                    hi -= 1;
                                    let cand: String = chars[..hi].iter().collect::<String>() + "…";
                                    let gc = cx.galley(&cand, &mono);
                                    if gc.width <= max_txt_w {
                                        cx.painter.text(cx.text, &gc, vec2(br.min.x + 6.0, br.min.y + (row_h - gc.height) * 0.5), txt_on_bar);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if is_h {
                        hov_idx = Some((si, br.center()));
                    }
                }
                cx.painter.pop_clip();

                if let Some((si, anchor)) = hov_idx {
                    let sp = &spans_cl[si];
                    let col = palette[(sp.depth + si) % palette.len()];
                    let share = (sp.duration_ms / total_ms * 100.0).clamp(0.0, 100.0);
                    let rows = vec![
                        (col, "Duration".into(), format!("{:.2} ms", sp.duration_ms)),
                        (col, "Frame share".into(), format!("{share:.1}%")),
                    ];
                    let title = sp.name.clone();
                    let tip_cl = tip.clone();
                    let bounds = r.expand(12.0);
                    cx.defer(r, Box::new(move |cx, _| tooltip_panel(cx, anchor, bounds, &title, &rows, &tip_cl)));
                }
            })),
        );

        self.end();
        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, hovered: ix.hovered, clicked: ix.clicked, ..Default::default() }
    }
}
