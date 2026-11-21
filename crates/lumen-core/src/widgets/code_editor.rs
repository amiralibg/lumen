//! Code editor: a multi-line, syntax-colored text editor with a line-number
//! gutter, a current-line band and caret that glide between lines,
//! matching-bracket highlights, auto-indent and comment toggling.

use std::ops::Range;

use super::code_data::{syn_token, tokenize_line};
use super::edit::TextEdit;
use super::show_on_drop;
use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::Style;
use crate::math::{vec2, Corners, Rect};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, TextFieldState, TextIn, Ui};
use crate::Color;

const INDENT: &str = "    ";

/// A line to draw: its index, text and colored spans (range, kind index).
type VisibleLine = (usize, String, Vec<(Range<usize>, usize)>);
const PAD: f32 = 10.0;

#[derive(Default)]
struct CodeMem {
    edit: TextEdit,
    scroll: f32,
    goal_col: Option<usize>,
    last_activity: f64,
    dragging: bool,
    /// Escape was pressed: let Tab move focus until the next edit.
    tab_released: bool,
}

pub struct CodeEditor<'u> {
    ui: &'u mut Ui,
    key: String,
    code: &'u mut String,
    lang: String,
    rows: u32,
    shown: bool,
}

fn line_start(s: &str, i: usize) -> usize {
    s[..i].rfind('\n').map_or(0, |k| k + 1)
}
fn line_end(s: &str, i: usize) -> usize {
    s[i..].find('\n').map_or(s.len(), |k| i + k)
}
fn col_of(s: &str, i: usize) -> usize {
    s[line_start(s, i)..i].chars().count()
}
/// Byte index of column `col` (clamped) on the line starting at `ls`.
fn at_col(s: &str, ls: usize, col: usize) -> usize {
    let le = line_end(s, ls);
    s[ls..le].char_indices().nth(col).map_or(le, |(k, _)| ls + k)
}
fn line_of(s: &str, i: usize) -> usize {
    s[..i].bytes().filter(|&b| b == b'\n').count()
}
fn line_starts(s: &str) -> Vec<usize> {
    std::iter::once(0).chain(s.match_indices('\n').map(|(k, _)| k + 1)).collect()
}

/// The bracket at or just before `i`, and its partner.
fn bracket_pair(s: &str, i: usize) -> Option<(usize, usize)> {
    let b = s.as_bytes();
    let pick = |k: usize| -> Option<usize> { (k < b.len() && b"()[]{}".contains(&b[k])).then_some(k) };
    let at = pick(i).or_else(|| i.checked_sub(1).and_then(pick))?;
    let (open, close, fwd) = match b[at] {
        b'(' => (b'(', b')', true),
        b'[' => (b'[', b']', true),
        b'{' => (b'{', b'}', true),
        b')' => (b'(', b')', false),
        b']' => (b'[', b']', false),
        _ => (b'{', b'}', false),
    };
    let mut depth = 0i32;
    if fwd {
        for (k, &c) in b.iter().enumerate().skip(at) {
            depth += (c == open) as i32 - (c == close) as i32;
            if depth == 0 {
                return Some((at, k));
            }
        }
    } else {
        for k in (0..=at).rev() {
            depth += (b[k] == close) as i32 - (b[k] == open) as i32;
            if depth == 0 {
                return Some((k, at));
            }
        }
    }
    None
}

/// Insert typed or pasted text: newlines become real line breaks and tabs
/// become spaces.
fn insert_code(e: &mut TextEdit, s: &mut String, text: &str) {
    let text = text.replace('\t', INDENT).replace("\r\n", "\n");
    for (k, part) in text.split('\n').enumerate() {
        if k > 0 {
            e.insert_newline(s);
        }
        if !part.is_empty() {
            e.insert(s, part);
        }
    }
}

/// Lines touched by the selection (or the caret's line).
fn selected_lines(s: &str, e: &TextEdit) -> Range<usize> {
    let r = e.selection();
    let a = line_of(s, r.start);
    let mut b = line_of(s, r.end);
    if r.end > r.start && r.end == line_start(s, r.end) {
        b = b.saturating_sub(1).max(a);
    }
    a..b + 1
}

/// Rewrite whole lines in one undo step, keeping the caret on its line.
fn edit_lines(e: &mut TextEdit, s: &mut String, lines: Range<usize>, f: impl Fn(&str) -> String) {
    let starts = line_starts(s);
    let from = starts[lines.start];
    let to = line_end(s, starts[lines.end - 1]);
    let new: Vec<String> = s[from..to].split('\n').map(&f).collect();
    let new = new.join("\n");
    let (cl, cc) = (line_of(s, e.cursor), col_of(s, e.cursor));
    e.replace(s, from..to, &new);
    let starts = line_starts(s);
    let ls = starts[cl.min(starts.len() - 1)];
    e.cursor = at_col(s, ls, cc);
    e.anchor = e.cursor;
}

impl<'u> CodeEditor<'u> {
    /// Syntax: "rust" (default), "wgsl", "toml", "json", "sh", "js", "py"...
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.lang = lang.into();
        self
    }
    /// Visible lines (default 12); longer code scrolls.
    pub fn rows(mut self, rows: u32) -> Self {
        self.rows = rows.max(2);
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("code_editor", &self.key));
        let sense = Sense { click: true, drag: true, focusable: true, scroll: true };
        let ix = ui.interact(id, sense, false);
        let mo = ui.motion();
        let reduced = ui.reduced_motion();
        let now = ui.time();
        let style = ui.text_style(TextRole::Mono);
        let line_h = style.line_height;
        let adv = ui.galley("M", &style).width.max(1.0);
        let code = &mut *self.code;
        let old = code.clone();
        let n_lines = line_starts(code).len();
        let digits = (n_lines.max(10) as f32).log10().floor() + 1.0;
        let gutter = digits * adv + 2.0 * PAD;
        let view_h = self.rows as f32 * line_h;
        let footer_h = 26.0;
        let text_x0 = ix.rect.min.x + gutter + PAD;
        let text_y0 = ix.rect.min.y + PAD;

        // Pointer: place, drag-select, double click selects a word.
        {
            let scroll = ui.mem::<CodeMem>(id).scroll;
            let shift = ui.modifiers().shift;
            let hit = |code: &str, p: crate::Vec2| -> usize {
                let starts = line_starts(code);
                let line = (((p.y - text_y0 + scroll) / line_h).floor().max(0.0) as usize).min(starts.len() - 1);
                let col = ((p.x - text_x0) / adv).round().max(0.0) as usize;
                at_col(code, starts[line], col)
            };
            let st = ui.mem::<CodeMem>(id);
            st.edit.clamp(code);
            if ix.press_started {
                if let Some(p) = ix.pointer {
                    let i = hit(code, p);
                    if ix.click_count == 2 {
                        st.edit.select_word(code, i);
                    } else if ix.click_count >= 3 {
                        let (ls, le) = (line_start(code, i), line_end(code, i));
                        st.edit.anchor = ls;
                        st.edit.cursor = (le + 1).min(code.len());
                    } else {
                        st.edit.click(i, shift);
                    }
                    st.dragging = ix.click_count == 1;
                    st.goal_col = None;
                    st.last_activity = now;
                }
            } else if ix.held && st.dragging {
                if let Some(p) = ix.pointer {
                    st.edit.click(hit(code, p), true);
                }
            } else {
                st.dragging = false;
            }
        }

        // Keyboard and text.
        let mut copied = None;
        if ix.focused {
            ui.set_wants_text_input();
            let keys = ui.keys_for(id);
            let ops = ui.take_text_ops(id);
            let page = self.rows.saturating_sub(1).max(1) as usize;
            let st = ui.mem::<CodeMem>(id);
            if !keys.is_empty() || !ops.is_empty() {
                st.last_activity = now;
            }
            for op in ops {
                match op {
                    TextIn::Text(t) => insert_code(&mut st.edit, code, &t),
                    TextIn::Ime(e) => {
                        st.edit.ime(code, &e);
                    }
                }
                st.goal_col = None;
                st.tab_released = false;
            }
            for (k, m, _) in keys {
                let (ext, cmd, word) = (m.shift, m.command(), m.word());
                let vertical = |st: &mut CodeMem, code: &str, delta: isize| {
                    let starts = line_starts(code);
                    let line = line_of(code, st.edit.cursor) as isize;
                    let col = *st.goal_col.get_or_insert(col_of(code, st.edit.cursor));
                    let target = (line + delta).clamp(0, starts.len() as isize - 1) as usize;
                    let i = if line + delta < 0 { 0 } else if line + delta >= starts.len() as isize { code.len() } else { at_col(code, starts[target], col) };
                    let anchor = st.edit.anchor;
                    st.edit.click(i, ext);
                    if ext {
                        st.edit.anchor = anchor;
                    }
                };
                let keep_goal = matches!(k, Key::ArrowUp | Key::ArrowDown | Key::PageUp | Key::PageDown);
                match k {
                    Key::ArrowUp if cmd => st.edit.home(ext),
                    Key::ArrowDown if cmd => st.edit.end(code, ext),
                    Key::ArrowUp => vertical(st, code, -1),
                    Key::ArrowDown => vertical(st, code, 1),
                    Key::PageUp => vertical(st, code, -(page as isize)),
                    Key::PageDown => vertical(st, code, page as isize),
                    Key::ArrowLeft if cmd => {
                        let ls = line_start(code, st.edit.cursor);
                        let anchor = st.edit.anchor;
                        st.edit.click(ls, ext);
                        if ext {
                            st.edit.anchor = anchor;
                        }
                    }
                    Key::ArrowRight if cmd => {
                        let le = line_end(code, st.edit.cursor);
                        let anchor = st.edit.anchor;
                        st.edit.click(le, ext);
                        if ext {
                            st.edit.anchor = anchor;
                        }
                    }
                    Key::ArrowLeft => st.edit.left(code, ext, word),
                    Key::ArrowRight => st.edit.right(code, ext, word),
                    Key::Home => {
                        // First press: after the indent; second: column 0.
                        let ls = line_start(code, st.edit.cursor);
                        let ind = ls + code[ls..line_end(code, ls)].len() - code[ls..line_end(code, ls)].trim_start().len();
                        let to = if st.edit.cursor == ind { ls } else { ind };
                        let anchor = st.edit.anchor;
                        st.edit.click(to, ext);
                        if ext {
                            st.edit.anchor = anchor;
                        }
                    }
                    Key::End => {
                        let le = line_end(code, st.edit.cursor);
                        let anchor = st.edit.anchor;
                        st.edit.click(le, ext);
                        if ext {
                            st.edit.anchor = anchor;
                        }
                    }
                    Key::Backspace => {
                        // Inside the indent, delete back to the previous stop.
                        let c = st.edit.cursor;
                        let ls = line_start(code, c);
                        let before = &code[ls..c];
                        if !st.edit.has_selection() && !word && !cmd && !before.is_empty() && before.chars().all(|ch| ch == ' ') {
                            let n = before.len() % INDENT.len();
                            let n = if n == 0 { INDENT.len() } else { n };
                            st.edit.anchor = c - n;
                            st.edit.backspace(code, false, false);
                        } else {
                            st.edit.backspace(code, word, cmd);
                        }
                    }
                    Key::Delete => st.edit.delete(code, word),
                    Key::Enter => {
                        // Keep the indent; one level deeper after an opener.
                        let c = st.edit.cursor;
                        let ls = line_start(code, c);
                        let line = &code[ls..c];
                        let mut ind: String = line.chars().take_while(|ch| *ch == ' ').collect();
                        if matches!(line.trim_end().chars().last(), Some('{' | '(' | '[')) {
                            ind.push_str(INDENT);
                        }
                        st.edit.insert_newline(code);
                        if !ind.is_empty() {
                            st.edit.insert(code, &ind);
                        }
                    }
                    Key::Tab if m.shift => {
                        let lines = selected_lines(code, &st.edit);
                        edit_lines(&mut st.edit, code, lines, |l| {
                            let n = l.chars().take_while(|c| *c == ' ').count().min(INDENT.len());
                            l[n..].to_owned()
                        });
                    }
                    Key::Tab => {
                        if st.edit.has_selection() {
                            let lines = selected_lines(code, &st.edit);
                            edit_lines(&mut st.edit, code, lines, |l| format!("{INDENT}{l}"));
                        } else {
                            let n = INDENT.len() - col_of(code, st.edit.cursor) % INDENT.len();
                            st.edit.insert(code, &INDENT[..n]);
                        }
                    }
                    Key::Escape => st.tab_released = true,
                    Key::Char('/') if cmd => {
                        let lines = selected_lines(code, &st.edit);
                        let starts = line_starts(code);
                        let all_commented = lines.clone().all(|k| {
                            let ls = starts[k];
                            let t = code[ls..line_end(code, ls)].trim_start();
                            t.is_empty() || t.starts_with("//")
                        });
                        edit_lines(&mut st.edit, code, lines, |l| {
                            let ind = l.len() - l.trim_start().len();
                            if l.trim().is_empty() {
                                l.to_owned()
                            } else if all_commented {
                                let rest = l[ind..].strip_prefix("// ").or_else(|| l[ind..].strip_prefix("//")).unwrap_or(&l[ind..]);
                                format!("{}{rest}", &l[..ind])
                            } else {
                                format!("{}// {}", &l[..ind], &l[ind..])
                            }
                        });
                    }
                    Key::Char('a') if cmd => st.edit.select_all(code),
                    Key::Char('c') | Key::Char('x') if cmd => {
                        // No selection: copy (or cut) the whole line.
                        if !st.edit.has_selection() {
                            let ls = line_start(code, st.edit.cursor);
                            st.edit.anchor = ls;
                            st.edit.cursor = (line_end(code, ls) + 1).min(code.len());
                        }
                        copied = Some(st.edit.selected(code).to_owned());
                        if k == Key::Char('x') {
                            st.edit.backspace(code, false, false);
                        }
                    }
                    Key::Char('z') if cmd && ext => st.edit.redo(code),
                    Key::Char('z') if cmd => st.edit.undo(code),
                    Key::Char('y') if m.ctrl && !m.mac => st.edit.redo(code),
                    _ => {}
                }
                if !keep_goal {
                    st.goal_col = None;
                }
                if !matches!(k, Key::Escape) {
                    st.tab_released = false;
                }
            }
            if !ui.mem::<CodeMem>(id).tab_released {
                ui.capture_tab(id);
            }
        } else if let Some(st) = ui.mem_get::<CodeMem>(id) {
            if st.edit.has_selection() || st.edit.marked.is_some() {
                let st = ui.mem::<CodeMem>(id);
                st.edit.anchor = st.edit.cursor;
                st.edit.marked = None;
            }
        }
        if let Some(c) = copied {
            ui.set_copied_text(c);
        }
        let changed = *code != old;

        // Scroll: the wheel, and following the caret.
        let starts = line_starts(code);
        let content_h = starts.len() as f32 * line_h;
        let max_scroll = (content_h - view_h).max(0.0);
        let wheel = if max_scroll > 0.0 { ui.take_scroll(id).y } else { 0.0 };
        let (cursor, anchor, marked, last) = {
            let st = ui.mem::<CodeMem>(id);
            st.edit.clamp(code);
            (st.edit.cursor, st.edit.anchor, st.edit.marked.clone(), st.last_activity)
        };
        let caret_line = line_of(code, cursor);
        let caret_col = col_of(code, cursor);
        {
            let moved = changed || ix.press_started || ix.focused && (now - last) < 0.02;
            let st = ui.mem::<CodeMem>(id);
            if wheel != 0.0 {
                st.scroll = (st.scroll + wheel).clamp(0.0, max_scroll);
            } else if moved {
                let y = caret_line as f32 * line_h;
                if y < st.scroll {
                    st.scroll = y;
                }
                if y + line_h > st.scroll + view_h {
                    st.scroll = y + line_h - view_h;
                }
            }
            st.scroll = st.scroll.clamp(0.0, max_scroll);
        }
        let target_scroll = ui.mem::<CodeMem>(id).scroll;
        let scroll = if reduced { target_scroll } else { ui.spring(id, 1, target_scroll, mo.smooth) };
        // The current-line band and the caret glide; reduced motion jumps.
        let band_y = if reduced { caret_line as f32 * line_h } else { ui.spring(id, 2, caret_line as f32 * line_h, mo.snappy) };
        let caret_x = if reduced { caret_col as f32 * adv } else { ui.spring(id, 3, caret_col as f32 * adv, mo.snappy) };
        let caret_a = ui.caret_alpha(ix.focused, last);
        let focus_t = ui.fade(id, 4, ix.focused, mo.hover_in, mo.hover_out, Ease::Linear);
        let hover_t = ui.fade(id, 5, ix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
        let (ring_a, ring_gap) = ui.focus_anim(&ix);

        if ix.focused {
            let r = Rect::from_xywh(text_x0 + caret_col as f32 * adv, text_y0 + caret_line as f32 * line_h - scroll, 2.0, line_h);
            let (a, b) = marked.clone().map_or((cursor, cursor), |m| (m.start, m.end));
            let w = (col_of(code, b) as f32 - col_of(code, a) as f32).abs() * adv;
            let rr = if marked.is_some() { Rect::from_xywh(text_x0 + col_of(code, a) as f32 * adv, r.min.y, w.max(1.0), line_h) } else { r };
            ui.report_text_field(TextFieldState { text: code.clone(), cursor, anchor, marked: marked.clone(), rect: rr });
        }

        // Colors and what to draw.
        let c = |t| ui.color(t);
        let fill = c(Token::Surface);
        let gutter_fill = c(Token::SurfaceMuted);
        let border = c(Token::Border).mix(c(Token::BorderStrong), hover_t).mix(c(Token::FocusRing), focus_t);
        let band = c(Token::Hover).multiply_alpha(0.6 + 0.4 * focus_t);
        let sel_c = c(Token::Selection);
        let caret_c = c(Token::FocusRing);
        let num_c = c(Token::TextMuted);
        let num_cur = c(Token::Foreground);
        let match_c = c(Token::BorderStrong);
        let foot_c = c(Token::TextMuted);
        let ring = c(Token::FocusRing);
        let syn: Vec<Color> = (0..8).map(|k| ui.color(syn_token(SYN_KINDS[k]))).collect();
        let radius = ui.look().card.radius.min(12.0);
        let sel = cursor.min(anchor)..cursor.max(anchor);
        let brackets = if ix.focused && sel.is_empty() { bracket_pair(code, cursor) } else { None };
        let first = (scroll / line_h).floor().max(0.0) as usize;
        let last_line = (((scroll + view_h) / line_h).ceil() as usize + 1).min(starts.len());
        let visible: Vec<VisibleLine> = (first..last_line)
            .map(|k| {
                let ls = starts[k];
                let line = code[ls..line_end(code, ls)].to_owned();
                let toks = tokenize_line(&line, &self.lang).into_iter().map(|(r, kind)| (r, SYN_KINDS.iter().position(|x| *x == kind).unwrap_or(0))).collect();
                (k, line, toks)
            })
            .collect();
        let line_cols: Vec<(usize, usize)> = (first..last_line).map(|k| (starts[k], line_end(code, starts[k]))).collect();
        let sel_spans: Vec<(usize, usize, usize, bool)> = visible
            .iter()
            .zip(&line_cols)
            .filter_map(|((k, line, _), &(ls, le))| {
                let (a, b) = (sel.start.max(ls), sel.end.min(le));
                if sel.is_empty() || sel.end < ls || sel.start > le || (a == b && !(sel.start <= ls && sel.end > le)) {
                    return None;
                }
                let ca = line[..a - ls].chars().count();
                let cb = line[..b - ls].chars().count();
                Some((*k, ca, cb, sel.end > le))
            })
            .collect();
        let mark_span = marked.as_ref().map(|m| (line_of(code, m.start), col_of(code, m.start), col_of(code, m.end)));
        let bracket_cells: Vec<(usize, usize)> = brackets.map(|(a, b)| vec![(line_of(code, a), col_of(code, a)), (line_of(code, b), col_of(code, b))]).unwrap_or_default();
        let lang_label = match self.lang.as_str() {
            "rust" | "rs" => "Rust",
            "wgsl" => "WGSL",
            "toml" => "TOML",
            "json" => "JSON",
            "sh" | "bash" => "Shell",
            "js" | "ts" => "JavaScript",
            "py" => "Python",
            other => other,
        }
        .to_owned();
        let status = format!("Ln {}, Col {}  ·  {} lines  ·  {lang_label}", caret_line + 1, caret_col + 1, starts.len());
        let foot_style = ui.text_style(TextRole::Caption);
        let focused = ix.focused;

        let height = view_h + 2.0 * PAD + footer_h;
        ui.leaf(
            Style { width: crate::layout::Size::FILL, height: crate::layout::Size::Fixed(height), min: vec2(240.0, 0.0), ..Default::default() },
            vec2(480.0, height),
            Some(id),
            sense,
            CursorIcon::Text,
            Some(Box::new(move |cx, r| {
                let r = cx.painter.snap_rect(r);
                cx.painter.rrect(r, Corners::all(radius), fill, 1.0, border);
                let body = Rect::from_xywh(r.min.x + 1.0, r.min.y + 1.0, r.width() - 2.0, r.height() - footer_h - 1.0);
                let gut = Rect::from_xywh(body.min.x, body.min.y, gutter - 1.0, body.height());
                cx.painter.rrect(gut, Corners([radius - 1.0, 0.0, 0.0, 0.0]), gutter_fill, 0.0, Color::TRANSPARENT);
                let x0 = r.min.x + gutter + PAD;
                let y0 = r.min.y + PAD - scroll;
                // Lines show only inside the text viewport, so a half line
                // appears only while scrolling, never in the padding.
                let lines_r = Rect::from_xywh(body.min.x, r.min.y + PAD, body.width(), view_h);
                cx.painter.push_clip(lines_r);
                // Current line.
                cx.painter.rect(Rect::from_xywh(body.min.x, y0 + band_y, body.width(), line_h), band);
                // Selection.
                for &(k, a, b, past) in &sel_spans {
                    let w = (b - a) as f32 * adv + if past { adv * 0.6 } else { 0.0 };
                    let sr = Rect::from_xywh(x0 + a as f32 * adv, y0 + k as f32 * line_h, w.max(2.0), line_h);
                    cx.painter.rrect_unsnapped(sr, Corners::all(3.0), [sel_c; 2], 0.0, Color::TRANSPARENT);
                }
                // Matching brackets.
                for &(k, col) in &bracket_cells {
                    let br = Rect::from_xywh(x0 + col as f32 * adv - 1.0, y0 + k as f32 * line_h + 2.0, adv + 2.0, line_h - 4.0);
                    cx.painter.rrect_unsnapped(br, Corners::all(3.0), [Color::TRANSPARENT; 2], 1.0, match_c);
                }
                // Gutter numbers and code.
                for (k, line, toks) in &visible {
                    let y = y0 + *k as f32 * line_h;
                    let num = format!("{}", k + 1);
                    let g = cx.galley(&num, &style);
                    let nc = if *k == caret_line && focused { num_cur } else { num_c };
                    cx.painter.text(cx.text, &g, vec2(r.min.x + gutter - PAD - g.width, y), nc);
                    for (range, kind) in toks {
                        let piece = &line[range.clone()];
                        if piece.trim().is_empty() {
                            continue;
                        }
                        let col = line[..range.start].chars().count();
                        let g = cx.galley(piece, &style);
                        cx.painter.text(cx.text, &g, vec2(x0 + col as f32 * adv, y), syn[*kind]);
                    }
                }
                // Composition underline.
                if let Some((k, a, b)) = mark_span {
                    let y = y0 + k as f32 * line_h + line_h - 3.0;
                    cx.painter.rect(Rect::from_xywh(x0 + a as f32 * adv, y, (b.max(a + 1) - a) as f32 * adv, 1.0), syn[0]);
                }
                // Caret.
                if caret_a > 0.0 {
                    let cr = Rect::from_xywh(cx.painter.snap(x0 + caret_x - 1.0), y0 + band_y + 2.0, 2.0, line_h - 4.0);
                    cx.painter.rrect_unsnapped(cr, Corners::all(1.0), [caret_c.multiply_alpha(caret_a); 2], 0.0, Color::TRANSPARENT);
                }
                cx.painter.pop_clip();
                // Footer: position and language.
                let fy = r.max.y - footer_h;
                cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, fy, r.width() - 2.0, 1.0), gutter_fill);
                let g = cx.galley(&status, &foot_style);
                cx.painter.text(cx.text, &g, vec2(r.max.x - PAD - g.width, fy + (footer_h - g.height) * 0.5), foot_c);
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr, radius, ring_a, ring_gap, ring)));
                }
            })),
        );
        let rect = ui.rect_of(id).unwrap_or(ix.rect);
        Response { id, rect, hovered: ix.hovered, clicked: ix.clicked, changed, focused: ix.focused }
    }
}

use super::code_data::SynKind;
const SYN_KINDS: [SynKind; 8] = [
    SynKind::Plain,
    SynKind::Keyword,
    SynKind::TypeName,
    SynKind::StringLit,
    SynKind::NumberLit,
    SynKind::Comment,
    SynKind::Punct,
    SynKind::MacroOrAttr,
];

show_on_drop!(CodeEditor);

impl Ui {
    /// A code editor over `code`: syntax colors, line numbers, auto-indent,
    /// Tab / Shift+Tab to indent, Cmd/Ctrl+/ to toggle comments, matching
    /// brackets, undo and IME. While it's focused Tab indents; press
    /// Escape first to Tab out.
    pub fn code_editor<'u>(&'u mut self, key: impl Into<String>, code: &'u mut String) -> CodeEditor<'u> {
        CodeEditor { ui: self, key: key.into(), code, lang: "rust".into(), rows: 12, shown: false }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brackets_pair_both_ways() {
        let s = "fn a() { [1, (2)] }";
        assert_eq!(bracket_pair(s, 7), Some((7, 18)));
        assert_eq!(bracket_pair(s, 19), Some((7, 18)));
        assert_eq!(bracket_pair(s, 13), Some((13, 15)));
        assert_eq!(bracket_pair("abc", 1), None);
    }

    #[test]
    fn columns_and_lines() {
        let s = "ab\ncdé\n";
        assert_eq!(line_starts(s), vec![0, 3, 8]);
        assert_eq!(col_of(s, 7), 3);
        assert_eq!(at_col(s, 3, 9), 7);
        assert_eq!(line_of(s, 4), 1);
    }

    #[test]
    fn indent_and_comment_whole_lines() {
        let mut s = String::from("a\nb");
        let mut e = TextEdit::default();
        (e.cursor, e.anchor) = (3, 0);
        edit_lines(&mut e, &mut s, 0..2, |l| format!("    {l}"));
        assert_eq!(s, "    a\n    b");
        e.undo(&mut s);
        assert_eq!(s, "a\nb", "one undo step");
    }

    #[test]
    fn pasted_newlines_and_tabs() {
        let mut s = String::new();
        let mut e = TextEdit::default();
        insert_code(&mut e, &mut s, "if x {\n\ty\n}");
        assert_eq!(s, "if x {\n    y\n}");
    }
}
