//! Shared parts of text fields: the field frame (fill, border, focus halo,
//! Windows underline), an inline editor that lives inside it, small icon
//! buttons for its trailing slot, and match highlighting for filtered
//! lists. Combobox, multi-select, tag input, search, password, number field
//! and text area are built from these.

use std::sync::Arc;

use lumen_text::{Galley, TextAlign, TextStyle};
use unicode_segmentation::UnicodeSegmentation;

use super::edit::TextEdit;
use crate::anim::Ease;
use crate::id::Id;
use crate::input::{CursorIcon, Key, Modifiers};
use crate::layout::{NodeId, Size, Style};
use crate::math::{vec2, Corners, Rect, Vec2};
use crate::theme::{TextRole, Token};
use crate::ui::{Interact, Sense, Ui};
use crate::Color;

/// What the frame of a field shows.
#[derive(Clone, Copy, Default)]
pub(crate) struct Chrome {
    pub hovered: bool,
    pub focused: bool,
    pub error: bool,
    pub disabled: bool,
}

/// Persistent state of an inline editor.
#[derive(Default)]
pub(crate) struct InlineState {
    pub edit: TextEdit,
    scroll: Vec2,
    last_activity: f64,
    dragging: bool,
    /// Column kept while moving the caret up and down a multi-line text.
    goal_x: Option<f32>,
}

/// Options of [`Ui::inline_edit`].
pub(crate) struct EditOpts {
    pub placeholder: String,
    pub style: TextStyle,
    /// Draw each character as a bullet (passwords).
    pub mask: bool,
    /// Wrap, insert newlines on Enter and grow between `rows`.
    pub multiline: bool,
    pub rows: (u32, u32),
    pub disabled: bool,
    pub accept: Option<fn(char) -> bool>,
    pub max_chars: Option<usize>,
    pub width: Size,
    pub min_w: f32,
    pub color: Option<Color>,
}

impl EditOpts {
    pub fn new(ui: &Ui) -> Self {
        let mut style = ui.text_style(TextRole::Body);
        style.line_height = (20.0 * ui.look().text_scale).round();
        EditOpts {
            placeholder: String::new(),
            style,
            mask: false,
            multiline: false,
            rows: (3, 8),
            disabled: false,
            accept: None,
            max_chars: None,
            width: Size::FILL,
            min_w: 40.0,
            color: None,
        }
    }
}

/// Result of [`Ui::inline_edit`].
pub(crate) struct EditOut {
    pub ix: Interact,
    pub changed: bool,
    /// Keys the editor left for its owner: Enter (single line), Up/Down
    /// (single line), Escape, PageUp/PageDown, and Backspace on an empty
    /// value.
    pub keys: Vec<(Key, Modifiers)>,
}

impl EditOut {
    pub fn key(&self, k: Key) -> bool {
        self.keys.iter().any(|(x, m)| *x == k && !m.command())
    }
}

const BULLET: &str = "•";

fn masked(s: &str) -> String {
    BULLET.repeat(s.graphemes(true).count())
}
/// Byte index in the masked text for byte index `i` of `s`.
fn to_mask(s: &str, i: usize) -> usize {
    s[..i.min(s.len())].graphemes(true).count() * BULLET.len()
}
fn from_mask(s: &str, d: usize) -> usize {
    let n = d / BULLET.len();
    s.grapheme_indices(true).nth(n).map(|(j, _)| j).unwrap_or(s.len())
}

/// The line holding caret index `i`, and its x.
pub(crate) fn caret_at(g: &Galley, i: usize) -> (f32, usize) {
    let mut line = 0;
    for (k, l) in g.lines.iter().enumerate() {
        if l.start <= i {
            line = k;
        }
    }
    let mut x = None;
    let mut last = 0.0;
    for gl in g.glyphs.iter().filter(|gl| gl.line == line) {
        let r = gl.range();
        if x.is_none() {
            if i <= r.start {
                x = Some(gl.x());
            } else if i < r.end {
                let t = (i - r.start) as f32 / (r.end - r.start).max(1) as f32;
                x = Some(gl.x() + gl.width() * t);
            }
        }
        last = gl.x() + gl.width();
    }
    (x.unwrap_or(last), line)
}

/// The caret index nearest to `p` (relative to the galley origin).
pub(crate) fn index_at(g: &Galley, p: Vec2) -> usize {
    let mut line = 0;
    for (k, l) in g.lines.iter().enumerate() {
        if p.y >= l.top {
            line = k;
        }
    }
    let l = g.lines[line];
    let mut best = (f32::INFINITY, l.start);
    let mut consider = |pos: f32, idx: usize| {
        let d = (pos - p.x).abs();
        if d < best.0 {
            best = (d, idx);
        }
    };
    consider(0.0, l.start);
    for gl in g.glyphs.iter().filter(|gl| gl.line == line) {
        let r = gl.range();
        consider(gl.x(), r.start);
        // A trailing newline or wrap space ends the line before it.
        let end_is_break = g.text[r.clone()].chars().all(|c| c == '\n');
        if !end_is_break {
            consider(gl.x() + gl.width(), r.end);
        }
    }
    best.1
}

impl Ui {
    /// Caret opacity: solid for 500 ms after activity, then blinking.
    pub(crate) fn caret_alpha(&mut self, focused: bool, last_activity: f64) -> f32 {
        if !focused {
            return 0.0;
        }
        let since = (self.time() - last_activity) as f32;
        if since < 0.5 || self.reduced_motion() || !self.caret_blink {
            if !self.reduced_motion() && self.caret_blink {
                self.request_repaint_after((0.5 - since) as f64);
            }
            return 1.0;
        }
        let t = (since - 0.5) % 1.0;
        let (a, next) = if t < 0.08 {
            (1.0 - t / 0.08, 0.0)
        } else if t < 0.5 {
            (0.0, 0.5 - t)
        } else if t < 0.58 {
            ((t - 0.5) / 0.08, 0.0)
        } else {
            (1.0, 1.0 - t)
        };
        self.request_repaint_after(next as f64);
        a
    }

    /// Open a field frame: a row with the input's fill, border, focus halo
    /// and (Windows) accent underline, animated on `id`. Add children, then
    /// call [`end`](Ui::end). `style` sets the row's size and padding.
    pub(crate) fn field_begin(&mut self, id: Id, ch: Chrome, style: Style) -> NodeId {
        let lk = self.look().input;
        let mo = self.motion();
        let h = self.fade(id, 40, ch.hovered && !ch.disabled, 0.06, 0.16, Ease::Linear);
        let f = self.fade(id, 41, ch.focused, 0.06, 0.16, Ease::Linear);
        let halo_w = self.spring(id, 42, if ch.focused { 3.0 * lk.halo } else { 0.0 }, mo.snappy);
        let halo_a = self.fade(id, 43, ch.focused, 0.0, 0.1, Ease::Linear);
        let err = self.fade(id, 44, ch.error, 0.12, 0.12, Ease::Linear);
        let c = |t| self.color(t);
        let (fill, border) = if ch.disabled {
            (c(Token::SurfaceMuted), c(Token::BorderSubtle))
        } else {
            let b = c(Token::Border).mix(c(Token::BorderStrong), h).mix(c(Token::FocusRing), f);
            let rest = c(Token::Surface).mix(c(Token::Control), lk.filled.max(lk.underline * (1.0 - f)));
            let fill = rest.mix(c(Token::Control).mix(c(Token::Control).over(c(Token::Hover)), h), lk.filled * (1.0 - f));
            let b = b.mix(c(Token::FocusRing).multiply_alpha(f), lk.filled).mix(c(Token::Danger), err);
            (fill, b)
        };
        let border_w = 1.0 + lk.filled * f;
        let under_c = c(Token::BorderStrong).mix(c(Token::Accent), f).mix(c(Token::Danger), err);
        let under_h = (1.0 + f) * lk.underline;
        let halo_c = c(Token::FocusHalo).mix(c(Token::Danger).with_alpha(c(Token::FocusHalo).a), err);
        let radius = self.control_metrics(crate::theme::ControlSize::Md).radius;
        self.begin(
            style,
            Some(id),
            Sense { click: true, drag: false, focusable: false, scroll: false },
            if ch.disabled { CursorIcon::Default } else { CursorIcon::Text },
            false,
            Some(Box::new(move |cx, r| {
                let r = cx.painter.snap_rect(r);
                if halo_w > 0.01 && halo_a > 0.0 {
                    cx.painter.ring(r.expand(halo_w), Corners::all(radius + halo_w), halo_w, halo_c.multiply_alpha(halo_a));
                }
                cx.painter.rrect(r, Corners::all(radius), fill, if border.a > 0.0 { border_w } else { 0.0 }, border);
                if under_h > 0.01 {
                    cx.painter.push_clip(Rect::from_xywh(r.min.x, r.max.y - under_h, r.width(), under_h));
                    cx.painter.rrect(r, Corners::all(radius), Color::TRANSPARENT, under_h, under_c);
                    cx.painter.pop_clip();
                }
            })),
        )
    }

    /// Field label above a control.
    pub(crate) fn field_label(&mut self, label: &str, disabled: bool) {
        if label.is_empty() {
            return;
        }
        let lc = if disabled { self.color(Token::TextDisabled) } else { self.color(Token::Foreground) };
        let st = self.text_style(TextRole::Label);
        self.text_leaf(label, st, lc, false, true, Style::default());
    }

    /// Helper or error line under a control. The error fades in.
    pub(crate) fn field_message(&mut self, id: Id, helper: Option<&str>, error: Option<&str>, disabled: bool) {
        let err_a = self.fade(id, 45, error.is_some(), 0.12, 0.10, Ease::Linear);
        let mut st = self.text_style(TextRole::Label);
        st.weight = 400;
        if let Some(e) = error {
            let dc = self.color(Token::DangerStrong).multiply_alpha(err_a);
            self.begin(Style { dir: crate::layout::Dir::Row, gap: 6.0, align: crate::layout::Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.icon_leaf("circle-alert", 14.0, dc);
            self.text_leaf(e, st, dc, true, false, Style { width: Size::FILL, ..Default::default() });
            self.end();
        } else if let Some(h) = helper {
            let hc = if disabled { self.color(Token::TextDisabled) } else { self.color(Token::TextMuted) };
            self.text_leaf(h, st, hc, true, false, Style::default());
        }
    }

    /// A round icon button sized for a field's trailing slot. `shown`
    /// scales it in and out on a spring; returns true when clicked.
    pub(crate) fn field_icon_button(&mut self, id: Id, icon: &str, shown: f32, focusable: bool) -> bool {
        let sense = Sense { click: true, drag: false, focusable, scroll: false };
        let ix = self.interact(id, sense, shown < 0.5);
        let mo = self.motion();
        let (key_act, held) = super::key_activate(self, &ix, true);
        let h = self.fade(id, 1, ix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
        let press = self.spring(id, 2, if ix.pressed || held { 1.0 } else { 0.0 }, mo.snappy);
        let (ring_a, ring_gap) = self.focus_anim(&ix);
        let hover = self.color(Token::Hover);
        let ic = self.color(Token::TextMuted).mix(self.color(Token::Foreground), h);
        let ring = self.color(Token::FocusRing);
        let icon = icon.to_owned();
        let s = shown.max(0.0);
        self.leaf(
            Style::default(),
            vec2(24.0 * s.min(1.0), 24.0),
            Some(id),
            sense,
            CursorIcon::Pointer,
            Some(Box::new(move |cx, r| {
                let c = r.center();
                let k = s * (1.0 - 0.08 * press);
                if h > 0.0 {
                    cx.painter.circle(c, 12.0 * k, hover.multiply_alpha(h), 0.0, Color::TRANSPARENT);
                }
                if s > 0.02 {
                    cx.icon(&icon, c, 15.0 * k, ic.multiply_alpha(s.min(1.0)));
                }
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, 12.0, ring_a, ring_gap, ring)));
                }
            })),
        );
        (ix.clicked || key_act) && shown >= 0.5
    }

    /// An editor without chrome: text, selection, caret and placeholder,
    /// with pointer and keyboard editing. Put it inside a field frame.
    pub(crate) fn inline_edit(&mut self, id: Id, value: &mut String, o: EditOpts) -> EditOut {
        let sense = Sense { click: true, drag: true, focusable: true, scroll: false };
        let ix = self.interact(id, sense, o.disabled);
        let now = self.time();
        let style = o.style;
        let line_h = style.line_height;
        let old = value.clone();
        let wrap_w = if o.multiline { Some(ix.rect.width().max(40.0)) } else { None };
        let layout = |ui: &mut Ui, v: &str| -> Arc<Galley> {
            let shown = if o.mask { masked(v) } else { v.to_owned() };
            ui.text.layout(&shown, &style, wrap_w, TextAlign::Start)
        };
        let to_d = |v: &str, i: usize| if o.mask { to_mask(v, i) } else { i };
        let from_d = |v: &str, d: usize| if o.mask { from_mask(v, d) } else { d };
        let mut left = vec![];
        let mut copied = None;
        let shift = self.modifiers().shift;

        // Pointer.
        if !o.disabled {
            let g = layout(self, value);
            let st = self.mem::<InlineState>(id);
            st.edit.clamp(value);
            let origin = ix.rect.min - st.scroll;
            if ix.press_started {
                if let Some(p) = ix.pointer {
                    let i = from_d(value, index_at(&g, p - origin));
                    match ix.click_count {
                        2 if !o.mask => st.edit.select_word(value, i),
                        n if n >= 2 => st.edit.select_all(value),
                        _ => st.edit.click(i, shift),
                    }
                    st.dragging = ix.click_count == 1;
                    st.last_activity = now;
                    st.goal_x = None;
                }
            } else if ix.held && st.dragging {
                if let Some(p) = ix.pointer {
                    st.edit.click(from_d(value, index_at(&g, p - origin)), true);
                }
            } else {
                st.dragging = false;
            }
        }

        // Keyboard.
        if ix.focused && !o.disabled {
            self.set_wants_text_input();
            let keys = self.keys_for(id);
            let inputs = self.take_text_input(id);
            let g = layout(self, value);
            let st = self.mem::<InlineState>(id);
            let mut activity = !keys.is_empty() || !inputs.is_empty();
            for t in inputs {
                let mut t: String = t.chars().filter(|c| o.accept.is_none_or(|f| f(*c)) && (o.multiline || *c != '\n')).collect();
                if let Some(max) = o.max_chars {
                    let room = max.saturating_sub(value.chars().count() - st.edit.selected(value).chars().count());
                    t = t.chars().take(room).collect();
                }
                if !t.is_empty() {
                    st.edit.insert(value, &t);
                    st.goal_x = None;
                }
            }
            for (k, mods, _) in keys {
                let ext = mods.shift;
                let word = mods.word() && !o.mask;
                let cmd = mods.command();
                let vertical = |st: &mut InlineState, down: bool, value: &str| {
                    let (x, line) = caret_at(&g, to_d(value, st.edit.cursor));
                    let gx = *st.goal_x.get_or_insert(x);
                    let target = if down { line + 1 } else { line.wrapping_sub(1) };
                    match g.lines.get(target) {
                        Some(l) => {
                            let i = from_d(value, index_at(&g, vec2(gx, l.top + 1.0)));
                            let anchor = st.edit.anchor;
                            st.edit.click(i, true);
                            if !ext {
                                st.edit.anchor = st.edit.cursor;
                            } else {
                                st.edit.anchor = anchor;
                            }
                        }
                        None if down => st.edit.end(value, ext),
                        None => st.edit.home(ext),
                    }
                    st.goal_x = Some(gx);
                };
                match k {
                    Key::Backspace if value.is_empty() => left.push((k, mods)),
                    Key::Backspace => st.edit.backspace(value, word, cmd),
                    Key::Delete => st.edit.delete(value, word),
                    Key::ArrowLeft if cmd => st.edit.home(ext),
                    Key::ArrowRight if cmd => st.edit.end(value, ext),
                    Key::ArrowLeft => st.edit.left(value, ext, word),
                    Key::ArrowRight => st.edit.right(value, ext, word),
                    Key::ArrowUp if o.multiline => vertical(st, false, value),
                    Key::ArrowDown if o.multiline => vertical(st, true, value),
                    Key::Home => st.edit.home(ext),
                    Key::End => st.edit.end(value, ext),
                    Key::Enter if o.multiline && !cmd => {
                        if o.max_chars.is_none_or(|m| value.chars().count() < m) {
                            st.edit.insert_newline(value);
                        }
                    }
                    Key::Char('a') if cmd => st.edit.select_all(value),
                    Key::Char('c') if cmd && !o.mask => {
                        if st.edit.has_selection() {
                            copied = Some(st.edit.selected(value).to_owned());
                        }
                    }
                    Key::Char('x') if cmd && !o.mask => {
                        if st.edit.has_selection() {
                            copied = Some(st.edit.selected(value).to_owned());
                            st.edit.backspace(value, false, false);
                        }
                    }
                    Key::Char('z') if cmd && ext => st.edit.redo(value),
                    Key::Char('z') if cmd => st.edit.undo(value),
                    Key::Char('y') if mods.ctrl && !mods.mac => st.edit.redo(value),
                    Key::Tab => activity = false,
                    Key::Char(_) | Key::Space => {}
                    _ => left.push((k, mods)),
                }
                if !matches!(k, Key::ArrowUp | Key::ArrowDown) {
                    st.goal_x = None;
                }
            }
            if activity {
                st.last_activity = now;
            }
        }
        if let Some(c) = copied {
            self.set_copied_text(c);
        }
        if !ix.focused {
            if let Some(st) = self.mem_get::<InlineState>(id) {
                if st.edit.has_selection() {
                    let st = self.mem::<InlineState>(id);
                    st.edit.anchor = st.edit.cursor;
                }
            }
        }
        let changed = *value != old;
        let last = self.mem::<InlineState>(id).last_activity;
        let caret_a = if o.disabled { 0.0 } else { self.caret_alpha(ix.focused, last) };

        // Geometry: keep the caret in view.
        let g = layout(self, value);
        let vis = vec2(ix.rect.width().max(1.0), ix.rect.height().max(line_h));
        let (cursor, sel) = {
            let st = self.mem::<InlineState>(id);
            st.edit.clamp(value);
            let (cx, line) = caret_at(&g, to_d(value, st.edit.cursor));
            let ly = g.lines[line].top;
            if o.multiline {
                st.scroll.x = 0.0;
                if ly + line_h - st.scroll.y > vis.y {
                    st.scroll.y = ly + line_h - vis.y;
                }
                if ly - st.scroll.y < 0.0 {
                    st.scroll.y = ly;
                }
                st.scroll.y = st.scroll.y.clamp(0.0, (g.height - vis.y).max(0.0));
            } else if g.width <= vis.x - 2.0 {
                st.scroll.x = 0.0;
            } else {
                if cx - st.scroll.x > vis.x - 2.0 {
                    st.scroll.x = cx - vis.x + 2.0;
                }
                if cx - st.scroll.x < 0.0 {
                    st.scroll.x = cx;
                }
                st.scroll.x = st.scroll.x.clamp(0.0, (g.width - vis.x + 2.0).max(0.0));
            }
            let r = st.edit.selection();
            (to_d(value, st.edit.cursor), to_d(value, r.start)..to_d(value, r.end))
        };
        let scroll = self.mem::<InlineState>(id).scroll;
        let ph = (value.is_empty() && !o.placeholder.is_empty()).then(|| self.galley(&o.placeholder, &style));
        let text_c = o.color.unwrap_or_else(|| if o.disabled { self.color(Token::TextDisabled) } else { self.color(Token::Foreground) });
        let ph_c = if o.disabled { self.color(Token::TextDisabled) } else { self.color(Token::TextMuted) };
        let sel_c = self.color(Token::Selection);
        let caret_c = self.color(Token::FocusRing);
        let focused = ix.focused;
        let height = if o.multiline {
            let rows = (g.height / line_h).round().clamp(o.rows.0 as f32, o.rows.1 as f32);
            rows * line_h
        } else {
            line_h
        };
        let min_w = if o.multiline { o.min_w } else { o.min_w.max(ph.as_ref().map(|p| p.width.ceil() + 2.0).unwrap_or(0.0)) };
        let pref_w = if o.multiline { min_w } else { (g.width.ceil() + 2.0).max(min_w) };
        let height = if o.multiline {
            // Height springs as the text grows and shrinks.
            let mo = self.motion();
            self.spring(id, 30, height, mo.smooth)
        } else {
            height
        };
        self.leaf(
            Style { width: o.width, min: vec2(min_w, 0.0), height: Size::Fixed(height), ..Default::default() },
            vec2(pref_w, height),
            Some(id),
            sense,
            if o.disabled { CursorIcon::Default } else { CursorIcon::Text },
            Some(Box::new(move |cx, r| {
                let origin = vec2(r.min.x - scroll.x, cx.painter.snap(r.min.y - scroll.y));
                cx.painter.push_clip(r.expand2(2.0, 1.0));
                if focused && sel.start != sel.end {
                    for (k, l) in g.lines.iter().enumerate() {
                        let a = sel.start.max(l.start);
                        let b = sel.end.min(l.end.max(l.start));
                        if a > b || (a == b && !(sel.start <= l.start && sel.end > l.end)) {
                            continue;
                        }
                        let (x0, _) = caret_at(&g, a);
                        let (mut x1, _) = caret_at(&g, b);
                        if caret_at(&g, b).1 != k {
                            x1 = l.width;
                        }
                        let x1 = if sel.end > l.end && k + 1 < g.lines.len() { x1 + 4.0 } else { x1 };
                        let sr = Rect::from_xywh(origin.x + x0, origin.y + l.top, (x1 - x0).max(0.0), l.height);
                        cx.painter.rrect_unsnapped(sr, Corners::all(3.0), [sel_c; 2], 0.0, Color::TRANSPARENT);
                    }
                }
                if let Some(p) = &ph {
                    cx.painter.text(cx.text, p, vec2(r.min.x, origin.y), ph_c);
                } else {
                    cx.painter.text(cx.text, &g, origin, text_c);
                }
                if caret_a > 0.0 {
                    let (x, line) = caret_at(&g, cursor);
                    let l = g.lines[line];
                    let x = cx.painter.snap(origin.x + x - 1.0);
                    let cr = Rect::from_xywh(x, origin.y + l.top + 2.0, 2.0, l.height.max(line_h) - 4.0);
                    cx.painter.rrect_unsnapped(cr, Corners::all(1.0), [caret_c.multiply_alpha(caret_a); 2], 0.0, Color::TRANSPARENT);
                }
                cx.painter.pop_clip();
            })),
        );
        EditOut { ix, changed, keys: left }
    }

    /// Put the caret of inline editor `id` at the end and select nothing.
    pub(crate) fn inline_edit_caret_end(&mut self, id: Id, value: &str) {
        let st = self.mem::<InlineState>(id);
        st.edit.cursor = value.len();
        st.edit.anchor = value.len();
    }

    /// Select all of inline editor `id`'s text.
    pub(crate) fn inline_edit_select_all(&mut self, id: Id, value: &str) {
        let st = self.mem::<InlineState>(id);
        st.edit.select_all(value);
    }
}

/// Case-insensitive match of `query` in `text`: byte ranges of the matched
/// characters, or None. A substring match wins; otherwise each query
/// character must appear in order (so "gh" finds "GitHub").
pub fn fuzzy_match(text: &str, query: &str) -> Option<Vec<std::ops::Range<usize>>> {
    let q = query.trim();
    if q.is_empty() {
        return Some(vec![]);
    }
    let lower: String = text.to_lowercase();
    let ql = q.to_lowercase();
    if lower.len() == text.len() {
        if let Some(i) = lower.find(&ql) {
            return Some(vec![i..i + ql.len()]);
        }
    }
    let mut out: Vec<std::ops::Range<usize>> = vec![];
    let mut qi = ql.chars().peekable();
    for (i, c) in text.char_indices() {
        let Some(&want) = qi.peek() else { break };
        if c.to_lowercase().eq(want.to_lowercase()) {
            qi.next();
            let end = i + c.len_utf8();
            match out.last_mut() {
                Some(r) if r.end == i => r.end = end,
                _ => out.push(i..end),
            }
        }
    }
    qi.peek().is_none().then_some(out)
}

/// Score for ordering matches: substring matches first, earlier and
/// tighter matches next.
pub(crate) fn match_score(ranges: &[std::ops::Range<usize>]) -> i32 {
    match ranges {
        [] => 0,
        [r] => 1000 - r.start as i32,
        rs => 500 - rs.len() as i32 * 20 - rs[0].start as i32,
    }
}

/// Draw `text` at `pos` with the `hl` ranges in `hl_c` over a soft tint.
pub(crate) fn paint_highlighted(cx: &mut crate::ui::PaintCx<'_>, text: &str, style: &TextStyle, pos: Vec2, fg: Color, hl: &[std::ops::Range<usize>], hl_c: Color, tint: Color) {
    let g = cx.galley(text, style);
    if hl.is_empty() {
        cx.painter.text(cx.text, &g, pos, fg);
        return;
    }
    for r in hl {
        let x0 = g.x_for_index(r.start);
        let x1 = g.x_for_index(r.end);
        let tr = Rect::from_xywh(pos.x + x0 - 1.0, pos.y + 1.0, x1 - x0 + 2.0, g.height - 2.0);
        cx.painter.rrect_unsnapped(tr, Corners::all(3.0), [tint; 2], 0.0, Color::TRANSPARENT);
    }
    // Runs: plain and matched pieces, each placed where the full line puts it.
    let mut i = 0;
    let mut runs = vec![];
    for r in hl {
        if r.start > i {
            runs.push((i..r.start, false));
        }
        runs.push((r.clone(), true));
        i = r.end;
    }
    if i < text.len() {
        runs.push((i..text.len(), false));
    }
    for (r, on) in runs {
        let piece = cx.galley(&text[r.clone()], style);
        let x = g.x_for_index(r.start);
        cx.painter.text(cx.text, &piece, vec2(pos.x + x, pos.y), if on { hl_c } else { fg });
    }
}
