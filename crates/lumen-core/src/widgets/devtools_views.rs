//! Developer-tool views: hex viewer, tree table, problems panel, stack
//! trace, blame and coverage. The long ones (hex, blame, coverage) are one
//! tall leaf inside a scroll area that paints only the rows in view, so a
//! 10,000-line file costs the same as a short one.

use std::collections::BTreeSet;
use std::ops::Range;

use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides, Vec2};
use crate::theme::{TextRole, Token};
use crate::ui::{PaintCx, Response, Sense, Ui};
use crate::Color;
use lumen_text::TextStyle;

use super::code_data::{syn_token, tokenize_line, SynKind};

// ============================================================ shared bits

const SYN: [SynKind; 8] = [SynKind::Plain, SynKind::Keyword, SynKind::TypeName, SynKind::StringLit, SynKind::NumberLit, SynKind::Comment, SynKind::Punct, SynKind::MacroOrAttr];

fn syn_colors(ui: &Ui) -> [Color; 8] {
    SYN.map(|k| ui.color(syn_token(k)))
}

fn syn_ix(k: SynKind) -> usize {
    SYN.iter().position(|s| *s == k).unwrap_or(0)
}

/// One line of code in a monospace font, syntax colored.
fn paint_code(cx: &mut PaintCx, line: &str, lang: &str, pos: Vec2, st: &TextStyle, cols: &[Color; 8], char_w: f32, alpha: f32) {
    let line = line.replace('\t', "    ");
    let toks = tokenize_line(&line, lang);
    let mut at = 0usize;
    let seg = |cx: &mut PaintCx, r: Range<usize>, c: Color| {
        if r.is_empty() || r.end > line.len() {
            return;
        }
        let s = &line[r.clone()];
        if s.trim().is_empty() {
            return;
        }
        let x = pos.x + line[..r.start].chars().count() as f32 * char_w;
        let g = cx.galley(s, st);
        cx.painter.text(cx.text, &g, vec2(x.round(), pos.y.round()), c.multiply_alpha(alpha));
    };
    for (r, k) in toks {
        if r.start > at {
            seg(cx, at..r.start, cols[0]);
        }
        let end = r.end;
        seg(cx, r, cols[syn_ix(k)]);
        at = at.max(end);
    }
    if at < line.len() {
        seg(cx, at..line.len(), cols[0]);
    }
}

fn mono_style(ui: &Ui) -> TextStyle {
    ui.text_style(TextRole::Mono)
}

/// Rows for a code view: the mono line plus a little air from the look.
fn code_row_h(ui: &Ui) -> f32 {
    (mono_style(ui).line_height + ui.look().control.gap[0] * 0.5).round()
}

/// 0..1 staggered entrance for item `i`, from the first time `id` showed.
fn stagger(ui: &mut Ui, id: crate::Id, i: usize) -> f32 {
    #[derive(Default)]
    struct Born(f64);
    if ui.reduced_motion() {
        return 1.0;
    }
    let now = ui.time();
    let born = {
        let b = ui.mem::<Born>(id);
        if b.0 == 0.0 {
            b.0 = now;
        }
        b.0
    };
    let mo = ui.motion();
    let dur = (mo.layer_in * 2.0).max(0.2);
    let t = (((now - born) as f32 - i.min(12) as f32 * 0.03) / dur).clamp(0.0, 1.0);
    if t < 1.0 {
        ui.anims.keep_alive();
    }
    Ease::Out.apply(t)
}

fn severity_tokens(s: Severity) -> (Token, &'static str, &'static str) {
    match s {
        Severity::Error => (Token::Danger, "circle-x", "Error"),
        Severity::Warning => (Token::Warning, "triangle-alert", "Warning"),
        Severity::Info => (Token::Info, "info", "Info"),
    }
}

// ============================================================ hex viewer

/// What the hex viewer is pointing at.
#[derive(Clone, Debug, Default)]
pub struct HexOut {
    pub response: Response,
    /// The byte under the cursor.
    pub cursor: usize,
    /// Selected bytes (at least the cursor).
    pub selection: Range<usize>,
}

#[derive(Default)]
struct HexMem {
    cursor: usize,
    anchor: usize,
    hover: Option<usize>,
    last_w: f32,
    view: (f32, f32),
}

pub struct HexViewer<'u, 'a> {
    ui: &'u mut Ui,
    key: String,
    bytes: &'a [u8],
    height: Option<f32>,
}

impl HexViewer<'_, '_> {
    /// Height of the scrolling byte area.
    pub fn height(mut self, h: f32) -> Self {
        self.height = Some(h);
        self
    }

    pub fn show(self) -> HexOut {
        let HexViewer { ui, key, bytes, height } = self;
        let id = ui.make_id(("hex", &key));
        let lk = ui.look().clone();
        let mo = ui.motion();
        let reduced = ui.reduced_motion();
        let n = bytes.len();
        let st = mono_style(ui);
        let char_w = ui.galley("0", &st).width;
        let row_h = code_row_h(ui);
        let pad = lk.control.pad_x[0];
        // 16 columns when they fit, else 8.
        let need = |cols: usize| (10 + cols * 3 + 2 + cols) as f32 * char_w + pad * 2.0;
        let last_w = ui.mem::<HexMem>(id).last_w;
        let cols = if last_w == 0.0 || last_w >= need(16) { 16 } else { 8 };
        // Narrow: shorter offsets (when they fit) and no gap between groups.
        let digits = if cols == 8 && n <= 0xFFFF { 4 } else { 8 };
        let rows = n.div_ceil(cols).max(1);
        let off_x = pad;
        let hex_x0 = pad + (digits + 2) as f32 * char_w;
        let group = if cols == 8 { cols } else { cols / 2 };
        let hex_x = move |c: usize| hex_x0 + (c * 3 + if c >= group { 1 } else { 0 }) as f32 * char_w;
        let ascii_x0 = hex_x0 + (cols * 3 + if cols == 8 { 1 } else { 2 }) as f32 * char_w;
        let width = ascii_x0 + cols as f32 * char_w + pad;
        let total_h = rows as f32 * row_h;
        let view_h = (height.unwrap_or(row_h * 10.0) / row_h).floor().max(1.0) * row_h;
        let view_h = view_h.min(total_h);

        let leaf_id = id.with("grid");
        let sense = Sense { click: true, drag: true, focusable: true, scroll: false };
        let ix = ui.interact(leaf_id, sense, false);
        let shift = ui.modifiers().shift;
        let hit = |p: Vec2, r: Rect| -> Option<usize> {
            let row = ((p.y - r.min.y) / row_h).floor();
            if row < 0.0 {
                return None;
            }
            let x = p.x - r.min.x;
            let col = if x >= ascii_x0 - char_w * 0.5 {
                ((x - ascii_x0) / char_w).floor()
            } else {
                let mut best = None;
                for c in 0..cols {
                    let cx0 = hex_x(c) - char_w * 0.5;
                    if x >= cx0 && x < cx0 + 3.0 * char_w {
                        best = Some(c as f32);
                    }
                }
                best?
            };
            if col < 0.0 || col >= cols as f32 {
                return None;
            }
            let i = row as usize * cols + col as usize;
            (i < n).then_some(i)
        };
        let mut moved_by_key = false;
        {
            let r = ix.rect;
            let m = ui.mem::<HexMem>(id);
            m.cursor = m.cursor.min(n.saturating_sub(1));
            m.hover = if ix.hovered { ix.pointer.and_then(|p| hit(p, r)) } else { None };
            if let Some(i) = ix.pointer.filter(|_| ix.press_started).and_then(|p| hit(p, r)) {
                m.cursor = i;
                if !shift {
                    m.anchor = i;
                }
            } else if ix.held {
                if let Some(i) = ix.pointer.and_then(|p| hit(p, r)) {
                    m.cursor = i;
                }
            }
        }
        if ix.focused && n > 0 {
            for (k, mods, _) in ui.keys_for(leaf_id) {
                let m = ui.mem::<HexMem>(id);
                let c = m.cursor as i64;
                let to = match k {
                    Key::ArrowRight => c + 1,
                    Key::ArrowLeft => c - 1,
                    Key::ArrowDown => c + cols as i64,
                    Key::ArrowUp => c - cols as i64,
                    Key::PageDown => c + 8 * cols as i64,
                    Key::PageUp => c - 8 * cols as i64,
                    Key::Home if mods.command() => 0,
                    Key::End if mods.command() => n as i64 - 1,
                    Key::Home => c - c % cols as i64,
                    Key::End => (c - c % cols as i64 + cols as i64 - 1).min(n as i64 - 1),
                    _ => continue,
                };
                m.cursor = to.clamp(0, n as i64 - 1) as usize;
                if !mods.shift {
                    m.anchor = m.cursor;
                }
                moved_by_key = true;
            }
        }
        let (cursor, anchor, hover) = {
            let m = ui.mem::<HexMem>(id);
            (m.cursor, m.anchor.min(n.saturating_sub(1)), m.hover)
        };
        let sel = cursor.min(anchor)..cursor.max(anchor) + 1;

        // Keep the cursor in view after a key move.
        let scroll_id = ui.scroll_id(("hex_scroll", &key));
        if moved_by_key {
            let (off, vh) = ui.mem::<HexMem>(id).view;
            let y = (cursor / cols) as f32 * row_h;
            if y < off {
                ui.scroll_to(scroll_id, y);
            } else if y + row_h > off + vh {
                ui.scroll_to(scroll_id, y + row_h - vh);
            }
        }

        // The cursor cell glides; hover fades.
        let (crow, ccol) = ((cursor / cols) as f32, (cursor % cols) as f32);
        let (gy, gx) = if reduced { (crow, ccol) } else { (ui.spring(id, 0, crow, mo.snappy), ui.spring(id, 1, ccol, mo.snappy)) };
        let hover_a = ui.fade(id, 2, hover.is_some(), mo.hover_in, mo.hover_out, Ease::Linear);
        let (ra, rg) = ui.focus_anim(&ix);
        let cols_c = [
            ui.color(Token::TextMuted),
            ui.color(Token::Foreground),
            ui.color(Token::InfoStrong),
            ui.color(Token::WarningStrong),
            ui.color(Token::Series3),
        ];
        let class = |b: u8| match b {
            0 => 0,
            0x21..=0x7e => 1,
            0x09 | 0x0a | 0x0d | 0x20 => 2,
            0x80..=0xff => 4,
            _ => 3,
        };
        let sec = ui.color(Token::TextSecondary);
        let muted = ui.color(Token::TextMuted);
        let hov_c = ui.color(Token::Hover);
        let sel_c = ui.color(Token::Selection);
        let cur_c = ui.color(Token::AccentSubtle);
        let cur_bd = ui.color(Token::Accent);
        let ring = ui.color(Token::FocusRing);
        let bg = ui.color(Token::Surface);
        let edge = ui.color(Token::BorderSubtle);
        let rad = lk.card.radius.min(lk.control.radius[2] * 1.5);
        let data: Vec<u8> = bytes.to_vec();
        let sel_paint = sel.clone();

        ui.begin(Style { dir: Dir::Column, width: Size::FILL, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, true, Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(rad), bg, 1.0, edge))));
        // Header: column offsets.
        let st_h = st;
        ui.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(width, row_h + lk.control.gap[0]),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let y = r.min.y + lk.control.gap[0] * 0.5;
                let g = cx.galley(if digits == 4 { "Addr" } else { "Offset" }, &st_h);
                cx.painter.text(cx.text, &g, vec2(r.min.x + off_x, y), muted);
                for c in 0..cols {
                    let g = cx.galley(&format!("{c:02X}"), &st_h);
                    cx.painter.text(cx.text, &g, vec2(r.min.x + hex_x(c), y), muted);
                }
                let g = cx.galley("ASCII", &st_h);
                cx.painter.text(cx.text, &g, vec2(r.min.x + ascii_x0, y), muted);
                cx.painter.rect(Rect { min: vec2(r.min.x, r.max.y - 1.0), max: r.max }, edge);
            })),
        );
        let (_, info) = ui.scroll(("hex_scroll", &key)).height(view_h).show(|ui| {
            ui.leaf(
                Style { width: Size::FILL, ..Default::default() },
                vec2(width, total_h),
                Some(leaf_id),
                sense,
                CursorIcon::Text,
                Some(Box::new(move |cx, r| {
                    let vis = cx.visible_rect();
                    let first = (((vis.min.y - r.min.y) / row_h).floor().max(0.0)) as usize;
                    let last = ((((vis.max.y - r.min.y) / row_h).ceil()) as usize).min(rows);
                    let cell = |i: usize, ascii: bool| {
                        let (row, c) = (i / cols, i % cols);
                        let y = r.min.y + row as f32 * row_h;
                        if ascii {
                            Rect::from_xywh(r.min.x + ascii_x0 + c as f32 * char_w, y, char_w, row_h)
                        } else {
                            Rect::from_xywh(r.min.x + hex_x(c) - char_w * 0.5, y, char_w * 3.0, row_h)
                        }
                    };
                    for i in sel_paint.clone() {
                        if i / cols >= first && i / cols < last && sel_paint.len() > 1 {
                            cx.painter.rect(cell(i, false), sel_c);
                            cx.painter.rect(cell(i, true), sel_c);
                        }
                    }
                    if let Some(h) = hover {
                        for a in [false, true] {
                            cx.painter.rrect(cell(h, a), Corners::all(3.0), hov_c.multiply_alpha(hover_a), 0.0, Color::TRANSPARENT);
                        }
                    }
                    if n > 0 {
                        let y = r.min.y + gy * row_h;
                        // Between two columns while gliding: interpolate their x.
                        let c0 = (gx.floor().max(0.0) as usize).min(cols - 1);
                        let c1 = (c0 + 1).min(cols - 1);
                        let hx = r.min.x + hex_x(c0) + (hex_x(c1) - hex_x(c0)) * gx.fract() - char_w * 0.5;
                        cx.painter.rrect_unsnapped(Rect::from_xywh(hx, y, char_w * 3.0, row_h), Corners::all(3.0), [cur_c; 2], 1.0, cur_bd);
                        let ax = r.min.x + ascii_x0 + gx * char_w;
                        cx.painter.rrect_unsnapped(Rect::from_xywh(ax, y, char_w, row_h), Corners::all(2.0), [cur_c; 2], 1.0, cur_bd);
                    }
                    let ty = (row_h - st.line_height) * 0.5;
                    for row in first..last {
                        let y = r.min.y + row as f32 * row_h + ty;
                        let g = cx.galley(&if digits == 4 { format!("{:04X}", row * cols) } else { format!("{:08X}", row * cols) }, &st);
                        cx.painter.text(cx.text, &g, vec2(r.min.x + off_x, y), sec);
                        for c in 0..cols {
                            let i = row * cols + c;
                            let Some(&b) = data.get(i) else { break };
                            let col = cols_c[class(b)];
                            let g = cx.galley(&format!("{b:02X}"), &st);
                            cx.painter.text(cx.text, &g, vec2(r.min.x + hex_x(c), y), col);
                            let ch = if (0x20..0x7f).contains(&b) { (b as char).to_string() } else { "·".to_string() };
                            let g = cx.galley(&ch, &st);
                            cx.painter.text(cx.text, &g, vec2(r.min.x + ascii_x0 + c as f32 * char_w, y), if class(b) == 1 { col } else { muted });
                        }
                    }
                    if ra > 0.0 {
                        cx.defer(vis.intersect(&r), Box::new(move |cx, rr| cx.focus_ring(rr, 4.0, ra, rg, ring)));
                    }
                })),
            );
        });
        {
            let w = ui.rect_of(id).map(|r| r.width()).unwrap_or(0.0);
            let m = ui.mem::<HexMem>(id);
            m.last_w = w;
            m.view = (info.offset, info.viewport_height);
        }
        // Status line.
        let b = bytes.get(cursor).copied().unwrap_or(0);
        let ch = if (0x20..0x7f).contains(&b) { format!(" · '{}'", b as char) } else { String::new() };
        let mut status = format!("0x{cursor:08X} · 0x{b:02X} · {b}{ch}");
        if sel.len() > 1 {
            status.push_str(&format!(" · {} bytes selected", sel.len()));
        }
        let mut st_s = ui.text_style(TextRole::Caption);
        st_s.tabular = true;
        ui.begin(Style { width: Size::FILL, pad: Sides::xy(pad, lk.control.gap[0]), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, Some(Box::new(move |cx, r| cx.painter.rect(Rect { min: r.min, max: vec2(r.max.x, r.min.y + 1.0) }, edge))));
        let sn = ui.text_leaf(status.clone(), st_s, sec, false, true, Style { width: Size::FILL, ..Default::default() });
        ui.a11y_hide(sn);
        ui.end();
        ui.end();
        let label = key.clone();
        ui.a11y(leaf_id, || A11y::new(Role::Grid).label(format!("{label} bytes")).value(status).description("Arrow keys move, Shift extends the selection."));
        HexOut { response: Response { id: leaf_id, rect: ix.rect, hovered: ix.hovered, clicked: ix.clicked, changed: moved_by_key || ix.press_started, focused: ix.focused }, cursor, selection: sel }
    }
}

// ============================================================ tree table

/// A row of a tree table: one cell per column, and children.
#[derive(Clone, Debug, Default)]
pub struct TreeTableRow {
    pub id: String,
    pub cells: Vec<String>,
    pub icon: Option<String>,
    pub children: Vec<TreeTableRow>,
}

impl TreeTableRow {
    pub fn new(id: impl Into<String>, cells: &[&str]) -> Self {
        TreeTableRow { id: id.into(), cells: cells.iter().map(|s| s.to_string()).collect(), icon: None, children: vec![] }
    }
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = Some(name.into());
        self
    }
    pub fn children(mut self, c: impl IntoIterator<Item = TreeTableRow>) -> Self {
        self.children = c.into_iter().collect();
        self
    }
}

struct FlatRow<'a> {
    row: &'a TreeTableRow,
    depth: usize,
    parent: Option<&'a str>,
    pos: (usize, usize),
}

fn flatten<'a>(rows: &'a [TreeTableRow], expanded: &BTreeSet<String>, depth: usize, parent: Option<&'a str>, out: &mut Vec<FlatRow<'a>>) {
    for (i, r) in rows.iter().enumerate() {
        out.push(FlatRow { row: r, depth, parent, pos: (i, rows.len()) });
        if !r.children.is_empty() && expanded.contains(&r.id) {
            flatten(&r.children, expanded, depth + 1, Some(&r.id), out);
        }
    }
}

#[derive(Default)]
struct TreeTableMem {
    seen: BTreeSet<String>,
}

impl Ui {
    /// A hex dump with offsets, grouped bytes and an ASCII column; click or
    /// drag to select, arrows move (Shift extends). Call `.show()`.
    pub fn hex_viewer<'u, 'a>(&'u mut self, key: impl Into<String>, bytes: &'a [u8]) -> HexViewer<'u, 'a> {
        HexViewer { ui: self, key: key.into(), bytes, height: None }
    }

    /// A tree with columns. `columns` are `(title, width)`, where width 0
    /// fills. The first column holds the tree.
    pub fn tree_table(&mut self, key: impl std::hash::Hash, columns: &[(&str, f32)], rows: &[TreeTableRow], expanded: &mut BTreeSet<String>, selected: &mut String) -> Response {
        let id = self.make_id(("tree_table", key));
        let ix = self.interact(id, Sense { click: false, drag: false, focusable: true, scroll: false }, false);
        let lk = self.look().clone();
        let mo = self.motion();
        let reduced = self.reduced_motion();
        let mut flat = vec![];
        flatten(rows, expanded, 0, None, &mut flat);
        let mut changed = false;
        let toggle = |expanded: &mut BTreeSet<String>, id: &str| {
            if !expanded.remove(id) {
                expanded.insert(id.to_owned());
            }
        };
        if ix.focused && !flat.is_empty() {
            let cur = flat.iter().position(|r| r.row.id == *selected).unwrap_or(0);
            for (k, _, _) in self.keys_for(id) {
                let r = &flat[cur];
                let branch = !r.row.children.is_empty();
                match k {
                    Key::ArrowDown => *selected = flat[(cur + 1).min(flat.len() - 1)].row.id.clone(),
                    Key::ArrowUp => *selected = flat[cur.saturating_sub(1)].row.id.clone(),
                    Key::Home => *selected = flat[0].row.id.clone(),
                    Key::End => *selected = flat[flat.len() - 1].row.id.clone(),
                    Key::ArrowRight if branch && !expanded.contains(&r.row.id) => {
                        expanded.insert(r.row.id.clone());
                    }
                    Key::ArrowRight if branch => *selected = r.row.children[0].id.clone(),
                    Key::ArrowLeft if branch && expanded.contains(&r.row.id) => {
                        expanded.remove(&r.row.id);
                    }
                    Key::ArrowLeft => {
                        if let Some(p) = r.parent {
                            *selected = p.to_owned();
                        }
                    }
                    Key::Enter | Key::Space if branch => toggle(expanded, &r.row.id),
                    _ => continue,
                }
                changed = true;
                break;
            }
        }
        // Rows: click selects, a click on a branch (or its chevron) toggles.
        let mut clicks = vec![];
        for r in &flat {
            let rid = id.with(("row", &r.row.id));
            let rix = self.interact(rid, Sense { click: true, drag: false, focusable: false, scroll: false }, false);
            if rix.clicked {
                clicks.push((r.row.id.clone(), !r.row.children.is_empty()));
            }
        }
        for (rid, branch) in clicks {
            self.request_focus(id);
            *selected = rid.clone();
            if branch {
                toggle(expanded, &rid);
            }
            changed = true;
        }
        if changed {
            flat.clear();
            flatten(rows, expanded, 0, None, &mut flat);
        }

        let row_h = lk.control.height[0].max(26.0);
        let indent = row_h * 0.7;
        let gap = lk.control.gap[1];
        let st = self.text_style(TextRole::Body);
        let st_h = self.text_style(TextRole::Label);
        let mut st_num = st;
        st_num.tabular = true;
        let fg = self.color(Token::Foreground);
        let sec = self.color(Token::TextSecondary);
        let edge = self.color(Token::BorderSubtle);
        let bg = self.color(Token::Surface);
        let pill = self.color(Token::AccentSubtle);
        let bar = self.color(Token::Accent);
        let ring = self.color(Token::FocusRing);
        let rad = lk.card.radius;
        let (ra, rg) = self.focus_anim(&ix);
        let sel_id = id.with(("row", selected.as_str()));
        let pill_key = id.with("pill");
        let smooth = mo.snappy;
        let row_r = lk.control.radius[0];

        let node = self.begin(Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(gap * 0.5), ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);
        self.set_paint(
            node,
            Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(rad), bg, 1.0, edge);
                if let Some(s) = cx.rects.get(&sel_id).copied() {
                    let y = if reduced {
                        s.min.y
                    } else {
                        let first = cx.anims.spring_state(pill_key, 0).is_none();
                        let y = cx.anims.spring_from(pill_key, 0, s.min.y, s.min.y, smooth);
                        if first {
                            s.min.y
                        } else {
                            y
                        }
                    };
                    let pr = Rect::from_xywh(s.min.x, y, s.width(), s.height());
                    cx.painter.rrect(pr, Corners::all(row_r), pill, 0.0, Color::TRANSPARENT);
                    cx.painter.rrect(Rect::from_xywh(pr.min.x + 2.0, pr.min.y + pr.height() * 0.25, 2.5, pr.height() * 0.5), Corners::all(1.25), bar, 0.0, Color::TRANSPARENT);
                }
                if ra > 0.0 {
                    cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr, rad, ra, rg, ring)));
                }
            }),
        );
        let col_style = |w: f32| Style { width: if w > 0.0 { Size::Fixed(w) } else { Size::FILL }, ..Default::default() };
        // Header.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(row_h), pad: Sides::xy(gap, 0.0), gap, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, Some(Box::new(move |cx, r| cx.painter.rect(Rect { min: vec2(r.min.x, r.max.y - 1.0), max: r.max }, edge))));
        for (i, (title, w)) in columns.iter().enumerate() {
            let n = self.text_leaf(title.to_string(), st_h, sec, false, true, col_style(*w));
            if i > 0 {
                self.set_text_align(n, lumen_text::TextAlign::End);
            }
        }
        self.end();

        // Rows; ones that just appeared rise in.
        let seen: BTreeSet<String> = std::mem::take(&mut self.mem::<TreeTableMem>(id).seen);
        let first_show = seen.is_empty();
        let mut now_seen = BTreeSet::new();
        for (i, r) in flat.iter().enumerate() {
            let rid = id.with(("row", &r.row.id));
            now_seen.insert(r.row.id.clone());
            let fresh = !first_show && !seen.contains(&r.row.id);
            let enter = if reduced {
                1.0
            } else if fresh {
                self.anims.spring_from(rid, 5, 0.0, 1.0, mo.smooth)
            } else {
                self.spring(rid, 5, 1.0, mo.smooth)
            };
            let branch = !r.row.children.is_empty();
            let open = expanded.contains(&r.row.id);
            let turn = if reduced { if open { 1.0 } else { 0.0 } } else { self.spring(rid, 6, if open { 1.0 } else { 0.0 }, mo.snappy) };
            let is_sel = r.row.id == *selected;
            let enter_c = enter.clamp(0.0, 1.0);
            self.row().fill_width().layer(enter_c, vec2(0.0, (1.0 - enter_c) * -row_h * 0.4)).show(|ui| {
                ui.begin(Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(row_h), pad: Sides::xy(gap, 0.0), gap, align: Align::Center, ..Default::default() }, Some(rid), Sense::CLICK, CursorIcon::Pointer, false, None);
                // First column: indent, chevron, icon, name.
                ui.begin(Style { dir: Dir::Row, width: if columns.first().is_some_and(|c| c.1 > 0.0) { Size::Fixed(columns[0].1) } else { Size::FILL }, gap: gap * 0.75, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                ui.leaf(Style::default(), vec2(r.depth as f32 * indent, 1.0), None, Sense::NONE, CursorIcon::Default, None);
                let csz = st.size + 2.0;
                ui.leaf(
                    Style::default(),
                    vec2(csz, csz),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, rr| {
                        if branch {
                            let c = rr.center();
                            let a = turn * std::f32::consts::FRAC_PI_2;
                            let k = csz * 0.22;
                            let rot = |p: Vec2| c + vec2(p.x * a.cos() - p.y * a.sin(), p.x * a.sin() + p.y * a.cos());
                            cx.painter.polyline(&[rot(vec2(-k * 0.5, -k)), rot(vec2(k * 0.5, 0.0)), rot(vec2(-k * 0.5, k))], 1.5, sec);
                        }
                    })),
                );
                if let Some(icon) = &r.row.icon {
                    let ic = ui.color(if branch { Token::Accent } else { Token::TextSecondary });
                    ui.icon_leaf(icon, st.size + 2.0, ic);
                }
                let name = r.row.cells.first().cloned().unwrap_or_default();
                ui.text_leaf(name.clone(), if is_sel { ui.text_style(TextRole::BodyStrong) } else { st }, fg, false, true, Style { width: Size::FILL, ..Default::default() });
                ui.end();
                for (c, (_, w)) in columns.iter().enumerate().skip(1) {
                    let v = r.row.cells.get(c).cloned().unwrap_or_default();
                    let n = ui.text_leaf(v, st_num, sec, false, true, col_style(*w));
                    ui.set_text_align(n, lumen_text::TextAlign::End);
                }
                ui.end();
                let value: Vec<String> = columns.iter().enumerate().skip(1).map(|(c, (t, _))| format!("{t} {}", r.row.cells.get(c).map(|s| s.as_str()).unwrap_or(""))).collect();
                let (pi, pn) = r.pos;
                let depth = r.depth;
                ui.a11y(rid, || {
                    let mut a = A11y::new(Role::TreeItem).label(name).value(value.join(", ")).selected(is_sel).position(pi, pn).level((depth + 1) as u8);
                    if branch {
                        a = a.expanded(open);
                    }
                    a
                });
            });
            let _ = i;
        }
        self.mem::<TreeTableMem>(id).seen = now_seen;
        self.end();
        self.a11y(id, || A11y::new(Role::Tree));
        Response { id, rect: self.rect_of(id).unwrap_or_default(), hovered: ix.hovered, clicked: false, changed, focused: ix.focused }
    }
}

// ============================================================ problems

/// How bad a problem is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// One diagnostic.
#[derive(Clone, Debug)]
pub struct Problem {
    pub severity: Severity,
    pub file: String,
    pub line: u32,
    pub col: u32,
    pub message: String,
    /// Tool and code, such as "rustc E0308".
    pub source: String,
}

#[derive(Default)]
struct ProblemsMem {
    hide: [bool; 3],
    collapsed: BTreeSet<String>,
    selected: Option<usize>,
}

// ============================================================ stack trace

/// One frame of a stack trace. `context` is source around the line:
/// `(line number, text)`.
#[derive(Clone, Debug, Default)]
pub struct StackFrame {
    pub function: String,
    pub file: String,
    pub line: u32,
    /// Your code (shown) rather than a library (folded).
    pub in_app: bool,
    pub context: Vec<(u32, String)>,
}

#[derive(Default)]
struct TraceMem {
    open: Option<usize>,
    init: bool,
    groups: BTreeSet<usize>,
}

// ============================================================ blame

/// A line with the commit that last touched it.
#[derive(Clone, Debug, Default)]
pub struct BlameLine {
    pub commit: String,
    pub author: String,
    pub age_days: u32,
    pub summary: String,
    pub text: String,
}

#[derive(Default)]
struct BlameMem {
    hover: Option<String>,
    shown_hover: Option<String>,
    cursor: usize,
    pinned: Option<String>,
    view: (f32, f32),
}

pub struct BlameView<'u, 'a> {
    ui: &'u mut Ui,
    key: String,
    lines: &'a [BlameLine],
    lang: String,
    height: Option<f32>,
}

// ============================================================ coverage

/// Coverage of one line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    /// Not executable (blank, comment, braces).
    Skip,
    /// Ran this many times.
    Hit(u32),
    /// Never ran.
    Miss,
    /// Ran, but some branches never did.
    Partial(u32),
}

#[derive(Clone, Debug)]
pub struct CoverLine {
    pub text: String,
    pub cover: Coverage,
}

#[derive(Default)]
struct CoverMem {
    cursor: Option<usize>,
    pulse_at: f64,
    view: (f32, f32),
}

pub struct CoverageView<'u, 'a> {
    ui: &'u mut Ui,
    key: String,
    file: String,
    lines: &'a [CoverLine],
    lang: String,
    height: Option<f32>,
}

impl Ui {
    /// Diagnostics grouped by file, with Error / Warning / Info filters.
    /// Returns the index of the problem opened (clicked, or Enter).
    pub fn problems_panel(&mut self, key: impl std::hash::Hash, problems: &[Problem]) -> Option<usize> {
        let id = self.make_id(("problems", key));
        let lk = self.look().clone();
        let mo = self.motion();
        let reduced = self.reduced_motion();
        let gap = lk.control.gap[1];
        let mut opened = None;
        let counts = [Severity::Error, Severity::Warning, Severity::Info].map(|s| problems.iter().filter(|p| p.severity == s).count());

        // Files in order of first appearance.
        let mut files: Vec<&str> = vec![];
        for p in problems {
            if !files.contains(&p.file.as_str()) {
                files.push(&p.file);
            }
        }
        let hide = self.mem::<ProblemsMem>(id).hide;
        let shown = |p: &Problem| !hide[p.severity as usize];
        let visible: Vec<usize> = files
            .iter()
            .flat_map(|f| problems.iter().enumerate().filter(move |(_, p)| p.file == *f).map(|(i, _)| i))
            .filter(|i| shown(&problems[*i]))
            .collect();

        // Keyboard on the list: up/down through visible rows, Enter opens.
        let list_id = id.with("list");
        let lix = self.interact(list_id, Sense { click: false, drag: false, focusable: true, scroll: false }, false);
        if lix.focused {
            let collapsed = self.mem::<ProblemsMem>(id).collapsed.clone();
            let rows: Vec<usize> = visible.iter().copied().filter(|i| !collapsed.contains(&problems[*i].file)).collect();
            for (k, _, _) in self.keys_for(list_id) {
                let m = self.mem::<ProblemsMem>(id);
                let cur = m.selected.and_then(|s| rows.iter().position(|r| *r == s));
                match k {
                    Key::ArrowDown => m.selected = rows.get(cur.map_or(0, |c| (c + 1).min(rows.len().saturating_sub(1)))).copied(),
                    Key::ArrowUp => m.selected = rows.get(cur.map_or(0, |c| c.saturating_sub(1))).copied(),
                    Key::Enter => opened = m.selected,
                    _ => {}
                }
            }
        }

        let st = self.text_style(TextRole::Body);
        let st_c = self.text_style(TextRole::Caption);
        let fg = self.color(Token::Foreground);
        let sec = self.color(Token::TextSecondary);
        let edge = self.color(Token::BorderSubtle);
        let bg = self.color(Token::Surface);
        let rad = lk.card.radius;
        let row_h = lk.control.height[0].max(26.0);
        let ring = self.color(Token::FocusRing);
        let (ra, rg) = self.focus_anim(&lix);

        self.begin(Style { dir: Dir::Column, width: Size::FILL, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(rad), bg, 1.0, edge))));
        // Header: title and the three filters.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(gap * 1.5, gap), gap, align: Align::Center, wrap: true, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, Some(Box::new(move |cx, r| cx.painter.rect(Rect { min: vec2(r.min.x, r.max.y - 1.0), max: r.max }, edge))));
        let st_t = self.text_style(TextRole::Heading4);
        self.text_leaf("Problems".to_owned(), st_t, fg, false, false, Style { width: Size::FILL, min: vec2(80.0, 0.0), ..Default::default() });
        for (k, s) in [Severity::Error, Severity::Warning, Severity::Info].into_iter().enumerate() {
            let (tok, icon, name) = severity_tokens(s);
            let fid = id.with(("filter", k));
            let fix = self.interact(fid, Sense::CLICK, false);
            let (act, held) = super::key_activate(self, &fix, true);
            if fix.clicked || act {
                let m = self.mem::<ProblemsMem>(id);
                m.hide[k] = !m.hide[k];
            }
            let on = !self.mem::<ProblemsMem>(id).hide[k];
            let on_t = self.fade(fid, 0, on, mo.selected, mo.selected, Ease::Out);
            let (hov, press) = self.state_weights(&fix, held);
            let squash = if reduced { 1.0 } else { self.spring(fid, 1, if fix.pressed || held { 0.94 } else { 1.0 }, mo.bouncy) };
            let (fa, fg2) = self.focus_anim(&fix);
            let col = self.color(tok);
            let fill = self.color(Token::SurfaceMuted).mix(col.multiply_alpha(0.16), on_t).over(self.color(Token::Hover).multiply_alpha(hov * 0.6 + press * 0.4));
            let txt = sec.mix(fg, on_t);
            let icon_c = sec.mix(col, on_t);
            let label = format!("{}", counts[k]);
            let g = self.galley(&label, &st_c);
            let isz = st_c.size + 3.0;
            let w = isz + g.width + lk.control.pad_x[0] * 1.6 + 4.0;
            let h = lk.control.height[0] * 0.85;
            let cr = h * 0.5;
            self.leaf(
                Style::default(),
                vec2(w, h),
                Some(fid),
                Sense::CLICK,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    let rr = Rect::from_center_size(r.center(), r.size() * squash);
                    cx.painter.rrect(rr, Corners::all(cr), fill, 0.0, Color::TRANSPARENT);
                    let x = rr.min.x + (rr.width() - isz - 4.0 - g.width) * 0.5;
                    cx.icon(icon, vec2(x + isz * 0.5, rr.center().y), isz, icon_c);
                    let gg = cx.galley(&label, &st_c);
                    cx.painter.text(cx.text, &gg, vec2((x + isz + 4.0).round(), (rr.center().y - gg.height * 0.5).round()), txt);
                    if fa > 0.0 {
                        cx.defer(r, Box::new(move |cx, r2| cx.focus_ring(r2, cr, fa, fg2, ring)));
                    }
                })),
            );
            let n = counts[k];
            self.a11y(fid, || A11y::new(Role::Button).label(format!("{name}s, {n}")).toggled(on));
        }
        self.end();

        // The list.
        let list = self.begin(Style { dir: Dir::Column, width: Size::FILL, pad: Sides::xy(gap * 0.5, gap * 0.5), ..Default::default() }, Some(list_id), Sense { click: false, drag: false, focusable: true, scroll: false }, CursorIcon::Default, false, None);
        self.set_paint(list, Box::new(move |cx, r| {
            if ra > 0.0 {
                cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr, rad * 0.6, ra, rg, ring)));
            }
        }));
        let mut row_i = 0usize;
        let mut any = false;
        for (fi, f) in files.iter().enumerate() {
            let items: Vec<usize> = (0..problems.len()).filter(|i| problems[*i].file == *f && shown(&problems[*i])).collect();
            if items.is_empty() {
                continue;
            }
            any = true;
            let gid = id.with(("file", fi));
            let gix = self.interact(gid, Sense::CLICK, false);
            let (act, _) = super::key_activate(self, &gix, true);
            if gix.clicked || act {
                let m = self.mem::<ProblemsMem>(id);
                if !m.collapsed.remove(*f) {
                    m.collapsed.insert(f.to_string());
                }
            }
            let open = !self.mem::<ProblemsMem>(id).collapsed.contains(*f);
            let turn = if reduced { if open { 1.0 } else { 0.0 } } else { self.spring(gid, 0, if open { 1.0 } else { 0.0 }, mo.snappy) };
            let hov = self.fade(gid, 1, gix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
            let (ga, gg) = self.focus_anim(&gix);
            let hover_c = self.color(Token::Hover).multiply_alpha(hov);
            let rr = lk.control.radius[0];
            let enter = stagger(self, id, row_i);
            row_i += 1;
            let (name, dir) = match f.rfind('/') {
                Some(k) => (&f[k + 1..], &f[..k]),
                None => (*f, ""),
            };
            let csz = st.size;
            self.row().fill_width().layer(enter, vec2(0.0, (1.0 - enter) * row_h * 0.3)).show(|ui| {
                ui.begin(
                    Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(row_h), pad: Sides::xy(gap * 0.5, 0.0), gap: gap * 0.75, align: Align::Center, ..Default::default() },
                    Some(gid),
                    Sense::CLICK,
                    CursorIcon::Pointer,
                    false,
                    Some(Box::new(move |cx, r| {
                        cx.painter.rrect(r, Corners::all(rr), hover_c, 0.0, Color::TRANSPARENT);
                        if ga > 0.0 {
                            cx.defer(r, Box::new(move |cx, r2| cx.focus_ring(r2, rr, ga, gg, ring)));
                        }
                    })),
                );
                ui.leaf(Style::default(), vec2(csz, csz), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| {
                    let c = r.center();
                    let a = turn * std::f32::consts::FRAC_PI_2;
                    let k = csz * 0.22;
                    let rot = |p: Vec2| c + vec2(p.x * a.cos() - p.y * a.sin(), p.x * a.sin() + p.y * a.cos());
                    cx.painter.polyline(&[rot(vec2(-k * 0.5, -k)), rot(vec2(k * 0.5, 0.0)), rot(vec2(-k * 0.5, k))], 1.5, sec);
                })));
                let ic = ui.color(Token::TextSecondary);
                ui.icon_leaf("file-code", st.size + 1.0, ic);
                let sb = ui.text_style(TextRole::BodyStrong);
                ui.text_leaf(name.to_owned(), sb, fg, false, false, Style::default());
                ui.text_leaf(dir.to_owned(), st_c, sec, false, true, Style { width: Size::FILL, ..Default::default() });
                ui.badge(items.len().to_string(), super::Tone::Neutral);
                ui.end();
            });
            let fname = f.to_string();
            let n_items = items.len();
            self.a11y(gid, || A11y::new(Role::Button).label(format!("{fname}, {n_items} problems")).expanded(open));
            if !open {
                continue;
            }
            for i in items {
                let p = &problems[i];
                let pid = id.with(("p", i));
                let pix = self.interact(pid, Sense { click: true, drag: false, focusable: false, scroll: false }, false);
                if pix.clicked {
                    self.mem::<ProblemsMem>(id).selected = Some(i);
                    self.request_focus(list_id);
                    opened = Some(i);
                }
                let sel = self.mem::<ProblemsMem>(id).selected == Some(i);
                let hov = self.fade(pid, 0, pix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
                let on = self.fade(pid, 1, sel, mo.selected, mo.selected, Ease::Out);
                let fill = self.color(Token::Hover).multiply_alpha(hov * (1.0 - on)).over(self.color(Token::AccentSubtle).multiply_alpha(on));
                let (tok, icon, sname) = severity_tokens(p.severity);
                let ic = self.color(tok);
                let enter = stagger(self, id, row_i);
                row_i += 1;
                let rr = lk.control.radius[0];
                let loc = format!("{} · {}:{}", p.source, p.line, p.col);
                self.row().fill_width().layer(enter, vec2(0.0, (1.0 - enter) * row_h * 0.3)).show(|ui| {
                    ui.begin(
                        Style { dir: Dir::Row, width: Size::FILL, min: vec2(0.0, row_h), pad: Sides { left: gap * 0.5 + csz + gap * 0.75, right: gap * 0.5, top: 3.0, bottom: 3.0 }, gap: gap * 0.75, align: Align::Center, ..Default::default() },
                        Some(pid),
                        Sense { click: true, drag: false, focusable: false, scroll: false },
                        CursorIcon::Pointer,
                        false,
                        Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(rr), fill, 0.0, Color::TRANSPARENT))),
                    );
                    ui.icon_leaf(icon, st.size + 1.0, ic);
                    ui.text_leaf(p.message.clone(), st, fg, false, true, Style { width: Size::FILL, min: vec2(60.0, 0.0), ..Default::default() });
                    ui.text_leaf(loc.clone(), st_c, sec, false, false, Style::default());
                    ui.end();
                });
                let msg = p.message.clone();
                let desc = format!("{sname}, {} line {}", p.file, p.line);
                self.a11y(pid, || A11y::new(Role::ListItem).label(msg).description(desc).selected(sel));
            }
        }
        if !any {
            let ok = self.color(Token::Success);
            self.begin(Style { dir: Dir::Row, width: Size::FILL, pad: Sides::all(gap * 1.5), gap, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.icon_leaf("circle-check", st.size + 2.0, ok);
            let msg = if problems.is_empty() { "No problems" } else { "All problems are filtered out" };
            self.text_leaf(msg.to_owned(), st, sec, false, false, Style::default());
            self.end();
        }
        self.end();
        let total = visible.len();
        self.a11y(list_id, || A11y::new(Role::List).label(format!("Problems, {total} shown")));
        self.end();
        if let Some(i) = opened {
            self.mem::<ProblemsMem>(id).selected = Some(i);
        }
        opened
    }

    /// An error with its stack: your frames shown, library frames folded
    /// into a row you can open, and the chosen frame's source around the
    /// line. Returns the frame clicked.
    pub fn stack_trace(&mut self, key: impl std::hash::Hash, error: &str, message: &str, frames: &[StackFrame], lang: &str) -> Option<usize> {
        let id = self.make_id(("stack", key));
        let lk = self.look().clone();
        let mo = self.motion();
        let reduced = self.reduced_motion();
        let gap = lk.control.gap[1];
        {
            let m = self.mem::<TraceMem>(id);
            if !m.init {
                m.init = true;
                m.open = frames.iter().position(|f| f.in_app && !f.context.is_empty());
            }
        }
        let st = self.text_style(TextRole::Body);
        let st_m = mono_style(self);
        let st_c = self.text_style(TextRole::Caption);
        let fg = self.color(Token::Foreground);
        let sec = self.color(Token::TextSecondary);
        let danger = self.color(Token::Danger);
        let edge = self.color(Token::BorderSubtle);
        let bg = self.color(Token::Surface);
        let rad = lk.card.radius;
        let row_h = lk.control.height[0].max(26.0);
        let ring = self.color(Token::FocusRing);
        let syn = syn_colors(self);
        let char_w = self.galley("0", &st_m).width;
        let line_h = code_row_h(self);
        let mut clicked = None;

        self.begin(Style { dir: Dir::Column, width: Size::FILL, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, true, Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(rad), bg, 1.0, edge))));
        // Header: the error.
        let dsub = self.color(Token::DangerSubtle);
        self.begin(Style { dir: Dir::Row, width: Size::FILL, pad: Sides::all(gap * 1.5), gap, align: Align::Start, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, Some(Box::new(move |cx, r| {
            cx.painter.rrect(r, Corners([rad, rad, 0.0, 0.0]), dsub.multiply_alpha(0.6), 0.0, Color::TRANSPARENT);
            cx.painter.rect(Rect { min: vec2(r.min.x, r.max.y - 1.0), max: r.max }, edge);
        })));
        self.icon_leaf("octagon-alert", st.size + 5.0, danger);
        self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 2.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let mut st_e = mono_style(self);
        st_e.weight = 600;
        let en = self.text_leaf(error.to_owned(), st_e, danger, false, true, Style { width: Size::FILL, ..Default::default() });
        let _ = en;
        self.text_leaf(message.to_owned(), st, fg, true, false, Style { width: Size::FILL, ..Default::default() });
        self.end();
        let full: String = std::iter::once(format!("{error}: {message}")).chain(frames.iter().map(|f| format!("    at {} ({}:{})", f.function, f.file, f.line))).collect::<Vec<_>>().join("\n");
        self.scope(id.with("copy"), |ui| ui.copy_button("", &full));
        self.end();

        // Frames: runs of library frames fold into one row.
        self.begin(Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(gap * 0.5), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let mut i = 0;
        let mut row_i = 0;
        while i < frames.len() {
            if !frames[i].in_app {
                let start = i;
                while i < frames.len() && !frames[i].in_app {
                    i += 1;
                }
                let n = i - start;
                let gid = id.with(("lib", start));
                let gix = self.interact(gid, Sense::CLICK, false);
                let (act, held) = super::key_activate(self, &gix, true);
                if gix.clicked || act {
                    let m = self.mem::<TraceMem>(id);
                    if !m.groups.remove(&start) {
                        m.groups.insert(start);
                    }
                }
                let open = self.mem::<TraceMem>(id).groups.contains(&start);
                let turn = if reduced { if open { 1.0 } else { 0.0 } } else { self.spring(gid, 0, if open { 1.0 } else { 0.0 }, mo.snappy) };
                let (hov, press) = self.state_weights(&gix, held);
                let (ga, gg) = self.focus_anim(&gix);
                let fill = self.color(Token::Hover).multiply_alpha(hov * 0.7 + press * 0.3);
                let rr = lk.control.radius[0];
                let enter = stagger(self, id, row_i);
                row_i += 1;
                let csz = st.size;
                self.row().fill_width().layer(enter, vec2(0.0, (1.0 - enter) * row_h * 0.3)).show(|ui| {
                    ui.begin(Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(row_h), pad: Sides::xy(gap, 0.0), gap: gap * 0.75, align: Align::Center, ..Default::default() }, Some(gid), Sense::CLICK, CursorIcon::Pointer, false, Some(Box::new(move |cx, r| {
                        cx.painter.rrect(r, Corners::all(rr), fill, 0.0, Color::TRANSPARENT);
                        if ga > 0.0 {
                            cx.defer(r, Box::new(move |cx, r2| cx.focus_ring(r2, rr, ga, gg, ring)));
                        }
                    })));
                    ui.leaf(Style::default(), vec2(csz, csz), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| {
                        let c = r.center();
                        let a = turn * std::f32::consts::FRAC_PI_2;
                        let k = csz * 0.22;
                        let rot = |p: Vec2| c + vec2(p.x * a.cos() - p.y * a.sin(), p.x * a.sin() + p.y * a.cos());
                        cx.painter.polyline(&[rot(vec2(-k * 0.5, -k)), rot(vec2(k * 0.5, 0.0)), rot(vec2(-k * 0.5, k))], 1.5, sec);
                    })));
                    ui.text_leaf(format!("{n} library frame{}", if n == 1 { "" } else { "s" }), st_c, sec, false, true, Style { width: Size::FILL, ..Default::default() });
                    ui.end();
                });
                self.a11y(gid, || A11y::new(Role::Button).label(format!("{n} library frames")).expanded(open));
                if open {
                    for k in start..i {
                        frame_row(self, id, k, &frames[k], false, &mut clicked, row_h, gap, st_m, st_c, sec.multiply_alpha(0.8), sec, ring, &lk, 1.0);
                    }
                }
                continue;
            }
            let open = self.mem::<TraceMem>(id).open == Some(i);
            let enter = stagger(self, id, row_i);
            row_i += 1;
            frame_row(self, id, i, &frames[i], open, &mut clicked, row_h, gap, st_m, st_c, fg, sec, ring, &lk, enter);
            if open && !frames[i].context.is_empty() {
                let cid = id.with(("ctx", i));
                let reveal = if reduced { 1.0 } else { self.anims.spring_from(cid, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0) };
                let ctx = frames[i].context.clone();
                let cur = frames[i].line;
                let num_w = (ctx.iter().map(|c| c.0).max().unwrap_or(0).to_string().len() as f32 + 1.0) * char_w;
                let code_bg = self.color(Token::SurfaceMuted);
                let hl = self.color(Token::DangerSubtle);
                let hl_bar = danger;
                let muted = self.color(Token::TextMuted);
                let lang = lang.to_owned();
                let h = ctx.len() as f32 * line_h + gap;
                let crad = lk.control.radius[1];
                self.row().fill_width().layer(reveal, vec2(0.0, (1.0 - reveal) * -line_h)).show(|ui| {
                    ui.begin(Style { width: Size::FILL, pad: Sides { left: gap * 2.0, right: gap, top: 0.0, bottom: gap * 0.5 }, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                    ui.leaf(
                        Style { width: Size::FILL, ..Default::default() },
                        vec2(200.0, h),
                        None,
                        Sense::NONE,
                        CursorIcon::Default,
                        Some(Box::new(move |cx, r| {
                            cx.painter.rrect(r, Corners::all(crad), code_bg, 0.0, Color::TRANSPARENT);
                            cx.painter.push_clip(r);
                            let pad = gap * 0.5;
                            for (k, (ln, text)) in ctx.iter().enumerate() {
                                let y = r.min.y + pad + k as f32 * line_h;
                                if *ln == cur {
                                    cx.painter.rect(Rect::from_xywh(r.min.x, y, r.width(), line_h), hl);
                                    cx.painter.rect(Rect::from_xywh(r.min.x, y, 2.0, line_h), hl_bar);
                                }
                                let ty = y + (line_h - st_m.line_height) * 0.5;
                                let g = cx.galley(&ln.to_string(), &st_m);
                                cx.painter.text(cx.text, &g, vec2(r.min.x + pad + num_w - g.width - char_w, ty), muted);
                                paint_code(cx, text, &lang, vec2(r.min.x + pad + num_w + char_w, ty), &st_m, &syn, char_w, 1.0);
                            }
                            cx.painter.pop_clip();
                        })),
                    );
                    ui.end();
                });
            }
            i += 1;
        }
        self.end();
        self.end();
        if let Some(k) = clicked {
            let m = self.mem::<TraceMem>(id);
            m.open = if m.open == Some(k) { None } else { Some(k) };
            if !reduced {
                self.anims.set_spring(id.with(("ctx", k)), 0, 0.0);
            }
        }
        clicked
    }

    /// Code with who last changed each line: chunks per commit with author,
    /// summary and age (a bar that's brighter the newer it is). Hovering a
    /// line lights every line of its commit. Returns the line picked.
    pub fn blame_view<'u, 'a>(&'u mut self, key: impl Into<String>, lines: &'a [BlameLine]) -> BlameView<'u, 'a> {
        BlameView { ui: self, key: key.into(), lines, lang: "rust".into(), height: None }
    }

    /// Code with line coverage: hit counts in the gutter, misses and
    /// partial branches tinted, a summary meter, and Next uncovered.
    pub fn coverage_view<'u, 'a>(&'u mut self, key: impl Into<String>, file: impl Into<String>, lines: &'a [CoverLine]) -> CoverageView<'u, 'a> {
        CoverageView { ui: self, key: key.into(), file: file.into(), lines, lang: "rust".into(), height: None }
    }
}

#[allow(clippy::too_many_arguments)]
fn frame_row(ui: &mut Ui, id: crate::Id, k: usize, f: &StackFrame, open: bool, clicked: &mut Option<usize>, row_h: f32, gap: f32, st_m: TextStyle, st_c: TextStyle, fg: Color, sec: Color, ring: Color, lk: &crate::look::Look, enter: f32) {
    let mo = ui.motion();
    let fid = id.with(("frame", k));
    let fix = ui.interact(fid, Sense::CLICK, false);
    let (act, held) = super::key_activate(ui, &fix, true);
    if fix.clicked || act {
        *clicked = Some(k);
    }
    let (hov, press) = ui.state_weights(&fix, held);
    let on = ui.fade(fid, 0, open, mo.selected, mo.selected, Ease::Out);
    let (fa, fg2) = ui.focus_anim(&fix);
    let fill = ui.color(Token::Hover).multiply_alpha(hov * 0.7 + press * 0.3).over(ui.color(Token::AccentSubtle).multiply_alpha(on * 0.7));
    let rr = lk.control.radius[0];
    let loc = format!("{}:{}", f.file, f.line);
    ui.row().fill_width().layer(enter, vec2(0.0, (1.0 - enter) * row_h * 0.3)).show(|ui| {
        ui.begin(
            Style { dir: Dir::Row, width: Size::FILL, min: vec2(0.0, row_h), pad: Sides::xy(gap, 2.0), gap, align: Align::Center, wrap: true, ..Default::default() },
            Some(fid),
            Sense::CLICK,
            CursorIcon::Pointer,
            false,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(rr), fill, 0.0, Color::TRANSPARENT);
                if fa > 0.0 {
                    cx.defer(r, Box::new(move |cx, r2| cx.focus_ring(r2, rr, fa, fg2, ring)));
                }
            })),
        );
        ui.text_leaf(f.function.clone(), st_m, fg, false, true, Style { width: Size::FILL, min: vec2(80.0, 0.0), ..Default::default() });
        ui.text_leaf(loc.clone(), st_c, sec, false, false, Style::default());
        ui.end();
    });
    let func = f.function.clone();
    let has_ctx = !f.context.is_empty();
    ui.a11y(fid, || {
        let mut a = A11y::new(Role::Button).label(format!("{func} at {loc}"));
        if has_ctx {
            a = a.expanded(open);
        }
        a
    });
}

fn age_label(days: u32) -> String {
    match days {
        0 => "today".into(),
        1..=6 => format!("{days}d"),
        7..=59 => format!("{}w", days / 7),
        60..=729 => format!("{}mo", days / 30),
        _ => format!("{}y", days / 365),
    }
}

fn author_tone(name: &str) -> Token {
    let h = name.bytes().fold(7u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32));
    [Token::Series0, Token::Series1, Token::Series2, Token::Series3, Token::Series4, Token::Series5][(h % 6) as usize]
}

impl BlameView<'_, '_> {
    pub fn lang(mut self, l: &str) -> Self {
        self.lang = l.into();
        self
    }
    pub fn height(mut self, h: f32) -> Self {
        self.height = Some(h);
        self
    }

    pub fn show(self) -> Option<usize> {
        let BlameView { ui, key, lines, lang, height } = self;
        let id = ui.make_id(("blame", &key));
        let lk = ui.look().clone();
        let mo = ui.motion();
        let reduced = ui.reduced_motion();
        let st = mono_style(ui);
        let st_c = ui.text_style(TextRole::Caption);
        let char_w = ui.galley("0", &st).width;
        let row_h = code_row_h(ui);
        let n = lines.len();
        let total_h = n as f32 * row_h;
        let view_h = ((height.unwrap_or(row_h * 14.0) / row_h).floor().max(1.0) * row_h).min(total_h);
        let leaf_id = id.with("code");
        let sense = Sense { click: true, drag: false, focusable: true, scroll: false };
        let ix = ui.interact(leaf_id, sense, false);
        let mut picked = None;
        let row_at = |p: Vec2, r: Rect| (((p.y - r.min.y) / row_h).floor() as i64).clamp(0, n as i64 - 1) as usize;
        let mut moved = false;
        {
            let r = ix.rect;
            let m = ui.mem::<BlameMem>(id);
            m.hover = if ix.hovered { ix.pointer.map(|p| lines[row_at(p, r)].commit.clone()) } else { None };
            if m.hover.is_some() {
                m.shown_hover = m.hover.clone();
            }
            if ix.clicked {
                if let Some(p) = ix.pointer {
                    let i = row_at(p, r);
                    m.cursor = i;
                    m.pinned = Some(lines[i].commit.clone());
                    picked = Some(i);
                }
            }
        }
        if ix.focused && n > 0 {
            for (k, _, _) in ui.keys_for(leaf_id) {
                let m = ui.mem::<BlameMem>(id);
                match k {
                    Key::ArrowDown => m.cursor = (m.cursor + 1).min(n - 1),
                    Key::ArrowUp => m.cursor = m.cursor.saturating_sub(1),
                    Key::PageDown => m.cursor = (m.cursor + 10).min(n - 1),
                    Key::PageUp => m.cursor = m.cursor.saturating_sub(10),
                    Key::Enter | Key::Space => {
                        m.pinned = Some(lines[m.cursor].commit.clone());
                        picked = Some(m.cursor);
                    }
                    _ => continue,
                }
                moved = true;
            }
        }
        let scroll_id = ui.scroll_id(("blame_scroll", &key));
        let (cursor, hover_on, shown_hover, pinned, view) = {
            let m = ui.mem::<BlameMem>(id);
            (m.cursor.min(n.saturating_sub(1)), m.hover.is_some(), m.shown_hover.clone(), m.pinned.clone(), m.view)
        };
        if moved {
            let y = cursor as f32 * row_h;
            if y < view.0 {
                ui.scroll_to(scroll_id, y);
            } else if y + row_h > view.0 + view.1 {
                ui.scroll_to(scroll_id, y + row_h - view.1);
            }
        }
        let hover_a = ui.fade(id, 0, hover_on, mo.hover_in, mo.hover_out, Ease::Linear);
        let grow = if reduced { 1.0 } else { ui.anims.spring_from(id, 1, 0.0, 1.0, mo.gentle).clamp(0.0, 1.0) };
        let (ra, rg) = ui.focus_anim(&ix);
        let syn = syn_colors(ui);
        let fg = ui.color(Token::Foreground);
        let sec = ui.color(Token::TextSecondary);
        let muted = ui.color(Token::TextMuted);
        let edge = ui.color(Token::BorderSubtle);
        let bg = ui.color(Token::Surface);
        let gutter_bg = ui.color(Token::SurfaceMuted);
        let hov_c = ui.color(Token::Hover);
        let pin_c = ui.color(Token::AccentSubtle);
        let accent = ui.color(Token::Accent);
        let old_c = ui.color(Token::BorderStrong);
        let ring = ui.color(Token::FocusRing);
        let tones: Vec<Color> = lines.iter().map(|l| ui.color(author_tone(&l.author))).collect();
        let max_age = lines.iter().map(|l| l.age_days).max().unwrap_or(1).max(1) as f32;
        let data: Vec<BlameLine> = lines.to_vec();
        let wide = ui.rect_of(id).map_or(true, |r| r.width() >= 520.0);
        let gutter_w = if wide { char_w * 34.0 } else { char_w * 9.0 };
        let num_w = (n.max(1).to_string().len() as f32 + 1.0) * char_w;
        let rad = lk.card.radius;
        let pad = lk.control.gap[1];

        // The gutter's fill lives on the frame so it follows the rounded corners.
        ui.begin(Style { dir: Dir::Column, width: Size::FILL, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, true, Some(Box::new(move |cx, r| {
            cx.painter.rrect(r, Corners::all(rad), bg, 0.0, Color::TRANSPARENT);
            cx.painter.rrect(Rect { min: r.min, max: vec2(r.min.x + gutter_w, r.max.y) }, Corners([rad, 0.0, 0.0, rad]), gutter_bg, 0.0, Color::TRANSPARENT);
            cx.painter.rrect(r, Corners::all(rad), Color::TRANSPARENT, 1.0, edge);
        })));
        let (_, info) = ui.scroll(("blame_scroll", &key)).height(view_h).show(|ui| {
            ui.leaf(
                Style { width: Size::FILL, ..Default::default() },
                vec2(gutter_w + num_w + 40.0 * char_w, total_h),
                Some(leaf_id),
                sense,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    let vis = cx.visible_rect();
                    let first = (((vis.min.y - r.min.y) / row_h).floor().max(0.0)) as usize;
                    let last = ((((vis.max.y - r.min.y) / row_h).ceil()) as usize).min(n);
                    let ty = (row_h - st.line_height) * 0.5;
                    let cty = (row_h - st_c.line_height) * 0.5;
                    for i in first..last {
                        let l = &data[i];
                        let y = r.min.y + i as f32 * row_h;
                        let row = Rect::from_xywh(r.min.x, y, r.width(), row_h);
                        if pinned.as_deref() == Some(l.commit.as_str()) {
                            cx.painter.rect(row, pin_c);
                        }
                        if shown_hover.as_deref() == Some(l.commit.as_str()) && hover_a > 0.0 {
                            cx.painter.rect(row, hov_c.multiply_alpha(hover_a));
                        }
                        if i == cursor && ra > 0.0 {
                            cx.painter.rect(Rect::from_xywh(r.min.x + gutter_w, y, 2.0, row_h), accent.multiply_alpha(ra));
                        }
                        let chunk_start = i == 0 || data[i - 1].commit != l.commit;
                        // Age bar: brighter when newer.
                        let fresh = 1.0 - (l.age_days as f32 / max_age).sqrt();
                        let bar_c = old_c.mix(accent, fresh);
                        cx.painter.rect(Rect::from_xywh(r.min.x, y, 3.0 * grow, row_h), bar_c);
                        if chunk_start {
                            if i > 0 {
                                cx.painter.rect(Rect::from_xywh(r.min.x, y, gutter_w, 1.0), edge);
                            }
                            let ar = row_h * 0.36;
                            let ac = vec2(r.min.x + pad + ar, y + row_h * 0.5);
                            cx.painter.circle(ac, ar, tones[i], 0.0, Color::TRANSPARENT);
                            let ini: String = l.author.chars().take(1).collect();
                            let mut sti = st_c;
                            sti.size = (ar * 1.2).round();
                            sti.weight = 600;
                            let gi = cx.galley(&ini, &sti);
                            cx.painter.text(cx.text, &gi, vec2((ac.x - gi.width * 0.5).round(), (ac.y - gi.height * 0.5).round()), Color::WHITE);
                            let age = age_label(l.age_days);
                            let ga = cx.galley(&age, &st_c);
                            let ax = r.min.x + gutter_w - pad - ga.width;
                            cx.painter.text(cx.text, &ga, vec2(ax, y + cty), muted);
                            if wide {
                                let x0 = ac.x + ar + pad * 0.75;
                                let room = ax - pad - x0;
                                let mut s = format!("{} · {}", l.author, l.summary);
                                let mut g = cx.galley(&s, &st_c);
                                while g.width > room && s.chars().count() > 2 {
                                    s.pop();
                                    while s.ends_with(' ') {
                                        s.pop();
                                    }
                                    let t = format!("{s}…");
                                    g = cx.galley(&t, &st_c);
                                    if g.width <= room {
                                        break;
                                    }
                                }
                                cx.painter.text(cx.text, &g, vec2(x0, y + cty), sec);
                            }
                        }
                        let gn = cx.galley(&(i + 1).to_string(), &st);
                        cx.painter.text(cx.text, &gn, vec2(r.min.x + gutter_w + num_w - gn.width - char_w * 0.5, y + ty), muted);
                        paint_code(cx, &l.text, &lang, vec2(r.min.x + gutter_w + num_w + char_w, y + ty), &st, &syn, char_w, 1.0);
                    }
                    let _ = fg;
                    if ra > 0.0 {
                        cx.defer(vis.intersect(&r), Box::new(move |cx, rr| cx.focus_ring(rr, 4.0, ra, rg, ring)));
                    }
                })),
            );
        });
        ui.mem::<BlameMem>(id).view = (info.offset, info.viewport_height);
        ui.end();
        let cur = lines.get(cursor).map(|l| format!("Line {}, {}, {}: {}", cursor + 1, l.author, age_label(l.age_days), l.summary)).unwrap_or_default();
        ui.a11y(leaf_id, || A11y::new(Role::Grid).label(format!("{key} blame")).value(cur).description("Arrow keys move by line, Enter shows the commit."));
        picked
    }
}

impl CoverageView<'_, '_> {
    pub fn lang(mut self, l: &str) -> Self {
        self.lang = l.into();
        self
    }
    pub fn height(mut self, h: f32) -> Self {
        self.height = Some(h);
        self
    }

    pub fn show(self) -> Response {
        let CoverageView { ui, key, file, lines, lang, height } = self;
        let id = ui.make_id(("coverage", &key));
        let lk = ui.look().clone();
        let mo = ui.motion();
        let reduced = ui.reduced_motion();
        let st = mono_style(ui);
        let st_c = ui.text_style(TextRole::Caption);
        let char_w = ui.galley("0", &st).width;
        let row_h = code_row_h(ui);
        let n = lines.len();
        let exec = lines.iter().filter(|l| l.cover != Coverage::Skip).count();
        let covered = lines.iter().filter(|l| matches!(l.cover, Coverage::Hit(_) | Coverage::Partial(_))).count();
        let missed: Vec<usize> = (0..n).filter(|i| lines[*i].cover == Coverage::Miss).collect();
        let pct = if exec == 0 { 100.0 } else { covered as f32 / exec as f32 * 100.0 };
        let gap = lk.control.gap[1];
        let now = ui.time();
        let scroll_id = ui.scroll_id(("cover_scroll", &key));
        let total_h = n as f32 * row_h;
        let view_h = ((height.unwrap_or(row_h * 14.0) / row_h).floor().max(1.0) * row_h).min(total_h);

        // Header: file, percent, meter, Next uncovered.
        let fg = ui.color(Token::Foreground);
        let sec = ui.color(Token::TextSecondary);
        let muted = ui.color(Token::TextMuted);
        let edge = ui.color(Token::BorderSubtle);
        let bg = ui.color(Token::Surface);
        let ok = ui.color(Token::Success);
        let warn = ui.color(Token::Warning);
        let bad = ui.color(Token::Danger);
        let bad_sub = ui.color(Token::DangerSubtle);
        let warn_sub = ui.color(Token::WarningSubtle);
        let track = ui.color(Token::Track);
        let accent = ui.color(Token::Accent);
        let rad = lk.card.radius;
        let tone = if pct >= 80.0 { ok } else if pct >= 50.0 { warn } else { bad };
        let fill = if reduced { pct / 100.0 } else { ui.anims.spring_from(id, 0, 0.0, pct / 100.0, mo.gentle).clamp(0.0, 1.0) };
        let mut next = false;
        ui.begin(Style { dir: Dir::Column, width: Size::FILL, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, true, Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(rad), bg, 1.0, edge))));
        ui.begin(Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(gap * 1.5, gap), gap, align: Align::Center, wrap: true, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, Some(Box::new(move |cx, r| cx.painter.rect(Rect { min: vec2(r.min.x, r.max.y - 1.0), max: r.max }, edge))));
        let icon_c = ui.color(Token::TextSecondary);
        ui.icon_leaf("file-code", st_c.size + 4.0, icon_c);
        let sb = ui.text_style(TextRole::BodyStrong);
        ui.text_leaf(file.clone(), sb, fg, false, true, Style { width: Size::FILL, min: vec2(60.0, 0.0), ..Default::default() });
        let mut st_p = ui.text_style(TextRole::BodyStrong);
        st_p.tabular = true;
        let pct_s = format!("{pct:.1}%");
        let pn = ui.text_leaf(pct_s.clone(), st_p, tone, false, false, Style::default());
        ui.a11y_hide(pn);
        let mw = lk.control.height[0] * 2.6;
        let mh = lk.slider.track.max(4.0);
        let meter = ui.leaf(Style::default(), vec2(mw, mh), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| {
            cx.painter.rrect(r, Corners::all(mh * 0.5), track, 0.0, Color::TRANSPARENT);
            let f = Rect { min: r.min, max: vec2(r.min.x + r.width() * fill, r.max.y) };
            cx.painter.rrect_unsnapped(f, Corners::all(mh * 0.5), [tone; 2], 0.0, Color::TRANSPARENT);
        })));
        let ex = exec as f64;
        let cv = covered as f64;
        ui.a11y_node(meter, || A11y::new(Role::Meter).label("Line coverage").numeric(cv, 0.0, ex, None).value(format!("{covered} of {exec} lines")));
        next |= ui.scope(id.with("next"), |ui| ui.button(format!("{} uncovered", missed.len())).icon("arrow-down").small().ghost().disabled(missed.is_empty()).tooltip("Next uncovered line (N)").show().clicked);
        ui.end();

        let leaf_id = id.with("code");
        let sense = Sense { click: true, drag: false, focusable: true, scroll: false };
        let ix = ui.interact(leaf_id, sense, false);
        if ix.focused {
            for (k, mods, _) in ui.keys_for(leaf_id) {
                if k == Key::Char('n') && !mods.command() {
                    next = true;
                }
            }
        }
        if ix.clicked {
            if let Some(p) = ix.pointer {
                let i = (((p.y - ix.rect.min.y) / row_h).floor() as i64).clamp(0, n as i64 - 1) as usize;
                ui.mem::<CoverMem>(id).cursor = Some(i);
            }
        }
        if next && !missed.is_empty() {
            let m = ui.mem::<CoverMem>(id);
            let from = m.cursor.map_or(0, |c| c + 1);
            let to = missed.iter().copied().find(|i| *i >= from).unwrap_or(missed[0]);
            m.cursor = Some(to);
            m.pulse_at = now;
            let (_, vh) = m.view;
            let y = (to as f32 * row_h - vh * 0.35).max(0.0);
            ui.scroll_to(scroll_id, y);
        }
        let (cursor, pulse_at) = {
            let m = ui.mem::<CoverMem>(id);
            (m.cursor, m.pulse_at)
        };
        let pulse_t = mo.layer_in * 4.0 + 0.3;
        let pulse = if pulse_at > 0.0 && !reduced { (1.0 - ((now - pulse_at) as f32 / pulse_t)).clamp(0.0, 1.0) } else { 0.0 };
        if pulse > 0.0 {
            ui.anims.keep_alive();
        }
        let (ra, rg) = ui.focus_anim(&ix);
        let ring = ui.color(Token::FocusRing);
        let syn = syn_colors(ui);
        let data: Vec<CoverLine> = lines.to_vec();
        let num_w = (n.max(1).to_string().len() as f32 + 1.0) * char_w;
        let hits_w = char_w * 6.0;
        let sel_c = ui.color(Token::AccentSubtle);
        ui.scroll(("cover_scroll", &key)).height(view_h).show(|ui| {
            ui.leaf(
                Style { width: Size::FILL, ..Default::default() },
                vec2(num_w + hits_w + 40.0 * char_w, total_h),
                Some(leaf_id),
                sense,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    let vis = cx.visible_rect();
                    let first = (((vis.min.y - r.min.y) / row_h).floor().max(0.0)) as usize;
                    let last = ((((vis.max.y - r.min.y) / row_h).ceil()) as usize).min(n);
                    let ty = (row_h - st.line_height) * 0.5;
                    let cty = (row_h - st_c.line_height) * 0.5;
                    for i in first..last {
                        let l = &data[i];
                        let y = r.min.y + i as f32 * row_h;
                        let row = Rect::from_xywh(r.min.x, y, r.width(), row_h);
                        let (stripe, tint) = match l.cover {
                            Coverage::Skip => (Color::TRANSPARENT, Color::TRANSPARENT),
                            Coverage::Hit(_) => (ok, Color::TRANSPARENT),
                            Coverage::Miss => (bad, bad_sub.multiply_alpha(0.7)),
                            Coverage::Partial(_) => (warn, warn_sub.multiply_alpha(0.7)),
                        };
                        cx.painter.rect(row, tint);
                        if cursor == Some(i) {
                            cx.painter.rect(row, sel_c.multiply_alpha(0.6));
                            if pulse > 0.0 {
                                cx.painter.rect(row, accent.multiply_alpha(0.25 * pulse));
                            }
                        }
                        cx.painter.rect(Rect::from_xywh(r.min.x, y, 3.0, row_h), stripe);
                        let gn = cx.galley(&(i + 1).to_string(), &st);
                        cx.painter.text(cx.text, &gn, vec2(r.min.x + num_w - gn.width, y + ty), muted);
                        let hits = match l.cover {
                            Coverage::Hit(h) | Coverage::Partial(h) => Some(if h >= 1000 { format!("{}k×", h / 1000) } else { format!("{h}×") }),
                            Coverage::Miss => Some("0×".to_owned()),
                            Coverage::Skip => None,
                        };
                        if let Some(h) = hits {
                            let gh = cx.galley(&h, &st_c);
                            cx.painter.text(cx.text, &gh, vec2(r.min.x + num_w + hits_w - gh.width - char_w, y + cty), if stripe.a > 0.0 { stripe } else { sec });
                        }
                        paint_code(cx, &l.text, &lang, vec2(r.min.x + num_w + hits_w, y + ty), &st, &syn, char_w, if l.cover == Coverage::Skip { 0.75 } else { 1.0 });
                    }
                    if ra > 0.0 {
                        cx.defer(vis.intersect(&r), Box::new(move |cx, rr| cx.focus_ring(rr, 4.0, ra, rg, ring)));
                    }
                })),
            );
        });
        ui.mem::<CoverMem>(id).view = (0.0, view_h);
        ui.end();
        let f2 = file.clone();
        let cur = cursor.map(|i| format!("Line {}: {}", i + 1, match lines[i].cover {
            Coverage::Skip => "not executable".to_owned(),
            Coverage::Hit(h) => format!("covered, {h} hits"),
            Coverage::Miss => "not covered".to_owned(),
            Coverage::Partial(h) => format!("partly covered, {h} hits"),
        })).unwrap_or_else(|| format!("{pct:.1}% of lines covered"));
        ui.a11y(leaf_id, || A11y::new(Role::Grid).label(format!("{f2} coverage")).value(cur).description("N jumps to the next uncovered line."));
        Response { id: leaf_id, rect: ix.rect, hovered: ix.hovered, clicked: ix.clicked, changed: next, focused: ix.focused }
    }
}
