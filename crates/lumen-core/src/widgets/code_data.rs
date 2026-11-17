//! Developer & structured-data widgets:
//! - [`CodeBlock`] (syntax-highlighted code viewer with line numbers, line highlights and Copy button)
//! - [`JsonViewer`] (zero-dep JSON parser + collapsible object/array tree)
//! - [`KeyValueEditor`] (editable environment-variable / HTTP-header rows with secret masking)
//! - [`LogViewer`] (filterable live console with level chips and highlighted search)

use std::collections::BTreeSet;
use std::ops::Range;

use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

use super::data::draw_chevron;
use super::field::{fuzzy_match, paint_highlighted, Chrome, EditOpts};

// ============================================================== CodeBlock

#[derive(Clone, Copy)]
enum SynKind {
    Plain,
    Keyword,
    TypeName,
    StringLit,
    NumberLit,
    Comment,
    Punct,
    MacroOrAttr,
}

fn syn_token(k: SynKind) -> Token {
    match k {
        SynKind::Plain => Token::Foreground,
        SynKind::Keyword => Token::AccentStrong,
        SynKind::TypeName => Token::Series3,
        SynKind::StringLit => Token::SuccessStrong,
        SynKind::NumberLit => Token::WarningStrong,
        SynKind::Comment => Token::TextMuted,
        SynKind::Punct => Token::TextSecondary,
        SynKind::MacroOrAttr => Token::InfoStrong,
    }
}

const KEYWORDS: &[&str] = &[
    "fn", "let", "mut", "pub", "struct", "enum", "impl", "trait", "for", "in", "if", "else", "match", "return",
    "use", "mod", "crate", "self", "Self", "super", "where", "const", "static", "async", "await", "move", "loop",
    "while", "break", "continue", "true", "false", "null", "None", "Some", "Ok", "Err", "function", "export",
    "import", "from", "default", "class", "new", "try", "catch", "def", "elif", "then", "fi",
];

fn tokenize_line(line: &str, lang: &str) -> Vec<(Range<usize>, SynKind)> {
    let bytes = line.as_bytes();
    let n = bytes.len();
    let mut out = Vec::new();
    let mut i = 0usize;
    let is_toml_or_sh = matches!(lang, "toml" | "sh" | "bash" | "shell" | "yaml" | "yml");

    while i < n {
        let b = bytes[i];
        if b.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        // Line comments (`//` or `#` in toml/shell).
        if (b == b'/' && i + 1 < n && bytes[i + 1] == b'/') || (b == b'#' && (is_toml_or_sh || i == 0)) {
            out.push((i..n, SynKind::Comment));
            break;
        }
        // Rust attributes `#[...]` or TOML sections `[package]`.
        if b == b'#' && i + 1 < n && bytes[i + 1] == b'[' {
            out.push((i..n, SynKind::MacroOrAttr));
            break;
        }
        if is_toml_or_sh && b == b'[' && lang == "toml" && line.trim_start().starts_with('[') && i == line.len() - line.trim_start().len() {
            out.push((i..n, SynKind::TypeName));
            break;
        }
        // String literals.
        if b == b'"' || (b == b'\'' && !matches!(lang, "rust" | "rs")) {
            let quote = b;
            let start = i;
            i += 1;
            while i < n {
                if bytes[i] == b'\\' && i + 1 < n {
                    i += 2;
                } else if bytes[i] == quote {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            // In JSON, `"key":` gets keyword/accent styling!
            let rest = line[i..].trim_start();
            let kind = if lang == "json" && rest.starts_with(':') { SynKind::Keyword } else { SynKind::StringLit };
            out.push((start..i, kind));
            continue;
        }
        // Numbers.
        if b.is_ascii_digit() || (b == b'-' && i + 1 < n && bytes[i + 1].is_ascii_digit()) {
            let start = i;
            i += 1;
            while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'.' || bytes[i] == b'_') {
                i += 1;
            }
            out.push((start..i, SynKind::NumberLit));
            continue;
        }
        // Identifiers / keywords / types / macros.
        if b.is_ascii_alphabetic() || b == b'_' {
            let start = i;
            i += 1;
            while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-') {
                i += 1;
            }
            if i < n && bytes[i] == b'!' {
                i += 1;
                out.push((start..i, SynKind::MacroOrAttr));
                continue;
            }
            let word = &line[start..i];
            let kind = if KEYWORDS.contains(&word) {
                SynKind::Keyword
            } else if word.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                SynKind::TypeName
            } else if lang == "toml" && line[i..].trim_start().starts_with('=') {
                SynKind::Keyword
            } else {
                SynKind::Plain
            };
            out.push((start..i, kind));
            continue;
        }
        // Multi-byte UTF-8 or punctuation.
        let ch_len = line[i..].chars().next().map(char::len_utf8).unwrap_or(1);
        out.push((i..i + ch_len, SynKind::Punct));
        i += ch_len;
    }
    out
}

/// Syntax-highlighted monospace code block with optional header, line numbers,
/// highlighted lines, and one-click clipboard copy.
pub struct CodeBlock<'u> {
    ui: &'u mut Ui,
    key: String,
    code: String,
    language: String,
    title: Option<String>,
    line_numbers: bool,
    highlight_lines: Vec<usize>,
    copyable: bool,
    shown: bool,
}

impl<'u> CodeBlock<'u> {
    pub fn language(mut self, lang: &str) -> Self {
        self.language = lang.to_owned();
        self
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
    pub fn line_numbers(mut self, on: bool) -> Self {
        self.line_numbers = on;
        self
    }
    /// 1-based line numbers to highlight with an accent bar.
    pub fn highlight_lines(mut self, lines: &[usize]) -> Self {
        self.highlight_lines = lines.to_vec();
        self
    }
    pub fn copyable(mut self, on: bool) -> Self {
        self.copyable = on;
        self
    }

    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("code_block", &self.key));
        let r_card = ui.look().card.radius;
        let card_bg = ui.color(Token::Surface);
        let card_bd = ui.color(Token::BorderSubtle);
        let hdr_bg = ui.color(Token::SurfaceMuted).multiply_alpha(0.65);
        let div_c = ui.color(Token::BorderSubtle);

        ui.begin(
            Style { width: Size::FILL, dir: Dir::Column, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_card), card_bg, 1.0, card_bd);
            })),
        );

        let show_hdr = self.title.is_some() || !self.language.is_empty() || self.copyable;
        let top_rad = (r_card - 1.0).max(0.0);
        if show_hdr {
            ui.begin(
                Style {
                    dir: Dir::Row,
                    width: Size::FILL,
                    pad: Sides::xy(12.0, 7.0),
                    gap: 8.0,
                    align: Align::Center,
                    ..Default::default()
                },
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
            ui.icon_leaf("file-code", 14.0, ic_c);
            let hdr_label = self.title.clone().unwrap_or_else(|| self.language.clone());
            let mut st_hdr = ui.text_style(TextRole::Caption);
            st_hdr.weight = 600;
            let hdr_c = ui.color(Token::TextSecondary);
            ui.text_leaf(hdr_label, st_hdr, hdr_c, false, true, Style { width: Size::FILL, ..Default::default() });
            if self.copyable {
                let code_copy = self.code.clone();
                ui.scope("copy", |ui| {
                    let mut st_cp = ui.mem::<f64>(id.with("copied_until")).to_owned();
                    let now = ui.time();
                    let copied = now < st_cp;
                    if copied {
                        ui.request_repaint_after((st_cp - now).max(0.02));
                    }
                    let btn_lbl = if copied { "Copied" } else { "Copy" };
                    let btn_ic = if copied { "check" } else { "copy" };
                    if ui.button(btn_lbl).ghost().small().icon(btn_ic).clicked() {
                        ui.set_copied_text(code_copy);
                        st_cp = now + 1.4;
                        *ui.mem::<f64>(id.with("copied_until")) = st_cp;
                    }
                });
            }
            ui.end();
        }

        let mut mono = ui.text_style(TextRole::Mono);
        mono.line_height = 20.0;
        let line_h = 21.0;
        let lines: Vec<&str> = self.code.lines().collect();
        let gutter_w = if self.line_numbers {
            let digits = lines.len().max(1).ilog10() as usize + 1;
            (digits.max(2) as f32 * 8.0 + 18.0).round()
        } else {
            12.0
        };

        ui.begin(
            Style {
                width: Size::FILL,
                dir: Dir::Column,
                pad: Sides::xy(0.0, 8.0),
                ..Default::default()
            },
            None,
            Sense::NONE,
            CursorIcon::Default,
            true,
            None,
        );

        let lang = self.language.to_ascii_lowercase();
        for (idx, &raw_line) in lines.iter().enumerate() {
            let line_no = idx + 1;
            let is_hl = self.highlight_lines.contains(&line_no);
            let spans = tokenize_line(raw_line, &lang);
            let colored_spans: Vec<(Range<usize>, Color)> = spans.into_iter().map(|(r, k)| (r, ui.color(syn_token(k)))).collect();
            let hl_bg = if is_hl { ui.color(Token::AccentSubtle).multiply_alpha(0.65) } else { Color::TRANSPARENT };
            let hl_bar = ui.color(Token::Accent);
            let ln_c = if is_hl { ui.color(Token::AccentStrong) } else { ui.color(Token::TextMuted) };
            let line_owned = raw_line.to_owned();
            let mono_cl = mono;
            let show_ln = self.line_numbers;

            ui.leaf(
                Style { width: Size::FILL, ..Default::default() },
                vec2(200.0, line_h),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    if is_hl {
                        cx.painter.rect(r, hl_bg);
                        cx.painter.rect(Rect::from_xywh(r.min.x, r.min.y, 2.5, r.height()), hl_bar);
                    }
                    let y = r.min.y + (r.height() - mono_cl.line_height) / 2.0;
                    if show_ln {
                        let ln_str = format!("{line_no}");
                        let g_ln = cx.galley(&ln_str, &mono_cl);
                        cx.painter.text(cx.text, &g_ln, vec2(r.min.x + gutter_w - 10.0 - g_ln.width, y), ln_c);
                    }
                    if !line_owned.is_empty() {
                        let code_x = r.min.x + gutter_w + 4.0;
                        let full = cx.galley(&line_owned, &mono_cl);
                        for (rg, col) in &colored_spans {
                            let piece = cx.galley(&line_owned[rg.clone()], &mono_cl);
                            let dx = full.x_for_index(rg.start);
                            cx.painter.text(cx.text, &piece, vec2(code_x + dx, y), *col);
                        }
                    }
                })),
            );
        }

        ui.end();
        ui.end();
        let rect = ui.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, hovered: false, clicked: false, changed: false, focused: false }
    }
}

impl Drop for CodeBlock<'_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

// ============================================================== JsonViewer

#[derive(Clone)]
enum JVal {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    Arr(Vec<JVal>),
    Obj(Vec<(String, JVal)>),
}

struct JParser<'a> {
    s: &'a str,
    b: &'a [u8],
    i: usize,
}

impl<'a> JParser<'a> {
    fn skip_ws(&mut self) {
        while self.i < self.b.len() && self.b[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn parse_val(&mut self) -> Result<JVal, String> {
        self.skip_ws();
        let Some(&c) = self.b.get(self.i) else {
            return Err("Unexpected end of JSON".into());
        };
        match c {
            b'{' => self.parse_obj(),
            b'[' => self.parse_arr(),
            b'"' => self.parse_str().map(JVal::Str),
            b't' if self.s[self.i..].starts_with("true") => {
                self.i += 4;
                Ok(JVal::Bool(true))
            }
            b'f' if self.s[self.i..].starts_with("false") => {
                self.i += 5;
                Ok(JVal::Bool(false))
            }
            b'n' if self.s[self.i..].starts_with("null") => {
                self.i += 4;
                Ok(JVal::Null)
            }
            b'-' | b'0'..=b'9' => {
                let start = self.i;
                self.i += 1;
                while self.i < self.b.len() && matches!(self.b[self.i], b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-') {
                    self.i += 1;
                }
                Ok(JVal::Num(self.s[start..self.i].to_owned()))
            }
            _ => Err(format!("Unexpected character at byte {}", self.i)),
        }
    }
    fn parse_str(&mut self) -> Result<String, String> {
        self.i += 1; // skip `"`
        let mut out = String::new();
        while self.i < self.b.len() {
            let ch = self.s[self.i..].chars().next().unwrap();
            self.i += ch.len_utf8();
            if ch == '"' {
                return Ok(out);
            } else if ch == '\\' && self.i < self.b.len() {
                let esc = self.s[self.i..].chars().next().unwrap();
                self.i += esc.len_utf8();
                match esc {
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    other => out.push(other),
                }
            } else {
                out.push(ch);
            }
        }
        Err("Unterminated JSON string".into())
    }
    fn parse_arr(&mut self) -> Result<JVal, String> {
        self.i += 1;
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.b.get(self.i) == Some(&b']') {
                self.i += 1;
                return Ok(JVal::Arr(items));
            }
            items.push(self.parse_val()?);
            self.skip_ws();
            if self.b.get(self.i) == Some(&b',') {
                self.i += 1;
            }
        }
    }
    fn parse_obj(&mut self) -> Result<JVal, String> {
        self.i += 1;
        let mut entries = Vec::new();
        loop {
            self.skip_ws();
            if self.b.get(self.i) == Some(&b'}') {
                self.i += 1;
                return Ok(JVal::Obj(entries));
            }
            if self.b.get(self.i) != Some(&b'"') {
                return Err(format!("Expected object key at byte {}", self.i));
            }
            let k = self.parse_str()?;
            self.skip_ws();
            if self.b.get(self.i) != Some(&b':') {
                return Err(format!("Expected ':' at byte {}", self.i));
            }
            self.i += 1;
            let v = self.parse_val()?;
            entries.push((k, v));
            self.skip_ws();
            if self.b.get(self.i) == Some(&b',') {
                self.i += 1;
            }
        }
    }
}

#[derive(Default)]
struct JsonViewerMem {
    initialized: bool,
    expanded: BTreeSet<String>,
    copied_until: f64,
}

fn seed_json_expanded(val: &JVal, path: &str, depth: usize, max_depth: usize, set: &mut BTreeSet<String>) {
    if depth >= max_depth {
        return;
    }
    match val {
        JVal::Obj(entries) => {
            set.insert(path.to_owned());
            for (k, v) in entries {
                seed_json_expanded(v, &format!("{path}.{k}"), depth + 1, max_depth, set);
            }
        }
        JVal::Arr(items) => {
            set.insert(path.to_owned());
            for (i, v) in items.iter().enumerate() {
                seed_json_expanded(v, &format!("{path}[{i}]"), depth + 1, max_depth, set);
            }
        }
        _ => {}
    }
}

/// Interactive collapsible JSON inspector with zero external dependencies.
pub struct JsonViewer<'u> {
    ui: &'u mut Ui,
    key: String,
    json: String,
    title: Option<String>,
    default_depth: usize,
    shown: bool,
}

impl<'u> JsonViewer<'u> {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
    pub fn default_depth(mut self, depth: usize) -> Self {
        self.default_depth = depth;
        self
    }
    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("json_viewer", &self.key));
        let parsed = JParser { s: &self.json, b: self.json.as_bytes(), i: 0 }.parse_val();

        if let Ok(ref root) = parsed {
            let mem = ui.mem::<JsonViewerMem>(id);
            if !mem.initialized {
                mem.initialized = true;
                seed_json_expanded(root, "$", 0, self.default_depth, &mut mem.expanded);
            }
        }

        let r_card = ui.look().card.radius;
        let card_bg = ui.color(Token::Surface);
        let card_bd = ui.color(Token::BorderSubtle);
        let hdr_bg = ui.color(Token::SurfaceMuted).multiply_alpha(0.65);
        let div_c = ui.color(Token::BorderSubtle);

        ui.begin(
            Style { width: Size::FILL, dir: Dir::Column, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), card_bg, 1.0, card_bd))),
        );

        // Header bar.
        let top_rad = (r_card - 1.0).max(0.0);
        ui.begin(
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
        let ic_c = ui.color(Token::TextMuted);
        ui.icon_leaf("braces", 14.0, ic_c);
        let mut st_hdr = ui.text_style(TextRole::Caption);
        st_hdr.weight = 600;
        let hdr_c = ui.color(Token::TextSecondary);
        ui.text_leaf(self.title.clone().unwrap_or_else(|| "JSON payload".into()), st_hdr, hdr_c, false, true, Style { width: Size::FILL, ..Default::default() });
        let raw_copy = self.json.clone();
        ui.scope("copy_json", |ui| {
            let now = ui.time();
            let copied_until = ui.mem::<JsonViewerMem>(id).copied_until;
            let copied = now < copied_until;
            if copied {
                ui.request_repaint_after((copied_until - now).max(0.02));
            }
            if ui.button(if copied { "Copied" } else { "Copy" }).ghost().small().icon(if copied { "check" } else { "copy" }).clicked() {
                ui.set_copied_text(raw_copy);
                ui.mem::<JsonViewerMem>(id).copied_until = now + 1.4;
            }
        });
        ui.end();

        ui.begin(
            Style { width: Size::FILL, dir: Dir::Column, pad: Sides::xy(10.0, 8.0), gap: 2.0, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );

        match parsed {
            Ok(root) => {
                let mut expanded = ui.mem::<JsonViewerMem>(id).expanded.clone();
                render_json_node(ui, id, "$", None, &root, 0, true, &mut expanded);
                ui.mem::<JsonViewerMem>(id).expanded = expanded;
            }
            Err(err) => {
                ui.alert(&err).danger().show();
            }
        }

        ui.end();
        ui.end();
        let rect = ui.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, hovered: false, clicked: false, changed: false, focused: false }
    }
}

fn render_json_node(
    ui: &mut Ui,
    root_id: crate::id::Id,
    path: &str,
    key_name: Option<&str>,
    val: &JVal,
    depth: usize,
    is_last: bool,
    expanded: &mut BTreeSet<String>,
) {
    let mono = ui.text_style(TextRole::Mono);
    let comma = if is_last { "" } else { "," };
    let indent = depth as f32 * 16.0;
    let guide_c = ui.color(Token::BorderSubtle);

    match val {
        JVal::Obj(_) | JVal::Arr(_) => {
            let is_obj = matches!(val, JVal::Obj(_));
            let count = match val {
                JVal::Obj(e) => e.len(),
                JVal::Arr(a) => a.len(),
                _ => 0,
            };
            let (open_ch, close_ch) = if is_obj { ("{", "}") } else { ("[", "]") };
            let is_open = count > 0 && expanded.contains(path);
            let nid = root_id.with(("jn", path));
            let ix = ui.interact(nid, if count > 0 { Sense::CLICK } else { Sense::NONE }, false);
            let (key_act, _) = if count > 0 { super::key_activate(ui, &ix, true) } else { (false, false) };
            if count > 0 && (ix.clicked || key_act) {
                if !expanded.remove(path) {
                    expanded.insert(path.to_owned());
                }
            }
            let mo = ui.motion();
            let chev_target = if is_open { 1.0 } else { 0.0 };
            let chev_t = ui.anims.spring_from(nid, 0, chev_target, chev_target, mo.snappy);
            let h = ui.fade(nid, 1, ix.hovered && count > 0, mo.hover_in, mo.hover_out, Ease::Linear);
            let hov_c = ui.color(Token::Hover).multiply_alpha(h);
            let chev_c = ui.color(Token::TextMuted);
            let key_c = ui.color(Token::AccentStrong);
            let punct_c = ui.color(Token::TextSecondary);
            let muted_c = ui.color(Token::TextMuted);

            ui.begin(
                Style {
                    dir: Dir::Row,
                    width: Size::FILL,
                    height: Size::Fixed(24.0),
                    pad: Sides { left: 4.0 + indent, right: 6.0, top: 0.0, bottom: 0.0 },
                    gap: 4.0,
                    align: Align::Center,
                    ..Default::default()
                },
                Some(nid),
                if count > 0 { Sense::CLICK } else { Sense::NONE },
                if count > 0 { CursorIcon::Pointer } else { CursorIcon::Default },
                false,
                Some(Box::new(move |cx, rr| {
                    if hov_c.a > 0.0 {
                        cx.painter.rrect(rr, Corners::all(5.0), hov_c, 0.0, Color::TRANSPARENT);
                    }
                    for d in 0..depth {
                        let gx = rr.min.x + 10.0 + d as f32 * 16.0;
                        cx.painter.rect(Rect::from_xywh(gx, rr.min.y - 1.0, 1.0, rr.height() + 2.0), guide_c.multiply_alpha(0.7));
                    }
                })),
            );
            if count > 0 {
                let angle = chev_t * std::f32::consts::FRAC_PI_2;
                ui.leaf(
                    Style::default(),
                    vec2(12.0, 12.0),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, cr| draw_chevron(&mut cx.painter, cr.center(), 3.6, angle, 1.5, chev_c))),
                );
            } else {
                ui.space(12.0);
            }
            if let Some(k) = key_name {
                ui.text_leaf(format!("\"{k}\""), mono, key_c, false, false, Style::default());
                ui.text_leaf(":", mono, punct_c, false, false, Style::default());
            }
            if count == 0 {
                ui.text_leaf(format!("{open_ch}{close_ch}{comma}"), mono, punct_c, false, false, Style::default());
            } else if !is_open {
                let unit = if is_obj {
                    if count == 1 { "1 key" } else { &format!("{count} keys") }
                } else if count == 1 {
                    "1 item"
                } else {
                    &format!("{count} items")
                };
                ui.text_leaf(open_ch, mono, punct_c, false, false, Style::default());
                ui.text_leaf(format!(" {unit} "), mono, muted_c, false, false, Style::default());
                ui.text_leaf(format!("{close_ch}{comma}"), mono, punct_c, false, false, Style::default());
            } else {
                ui.text_leaf(open_ch, mono, punct_c, false, false, Style::default());
            }
            ui.end();

            if is_open {
                match val {
                    JVal::Obj(entries) => {
                        let last = entries.len().saturating_sub(1);
                        for (idx, (k, child)) in entries.iter().enumerate() {
                            let child_path = format!("{path}.{k}");
                            render_json_node(ui, root_id, &child_path, Some(k), child, depth + 1, idx == last, expanded);
                        }
                    }
                    JVal::Arr(items) => {
                        let last = items.len().saturating_sub(1);
                        for (idx, child) in items.iter().enumerate() {
                            let child_path = format!("{path}[{idx}]");
                            render_json_node(ui, root_id, &child_path, None, child, depth + 1, idx == last, expanded);
                        }
                    }
                    _ => {}
                }
                let punct_c2 = ui.color(Token::TextSecondary);
                let guide_c2 = ui.color(Token::BorderSubtle);
                ui.begin(
                    Style {
                        dir: Dir::Row,
                        width: Size::FILL,
                        height: Size::Fixed(22.0),
                        pad: Sides { left: 20.0 + indent, right: 6.0, top: 0.0, bottom: 0.0 },
                        align: Align::Center,
                        ..Default::default()
                    },
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    Some(Box::new(move |cx, rr| {
                        for d in 0..depth {
                            let gx = rr.min.x + 10.0 + d as f32 * 16.0;
                            cx.painter.rect(Rect::from_xywh(gx, rr.min.y - 1.0, 1.0, rr.height() + 2.0), guide_c2.multiply_alpha(0.7));
                        }
                    })),
                );
                ui.text_leaf(format!("{close_ch}{comma}"), mono, punct_c2, false, false, Style::default());
                ui.end();
            }
        }
        _ => {
            let key_c = ui.color(Token::AccentStrong);
            let punct_c = ui.color(Token::TextSecondary);
            let (val_str, val_c) = match val {
                JVal::Null => ("null".to_owned(), ui.color(Token::TextMuted)),
                JVal::Bool(b) => (b.to_string(), ui.color(Token::Series4)),
                JVal::Num(n) => (n.clone(), ui.color(Token::WarningStrong)),
                JVal::Str(s) => (format!("\"{s}\""), ui.color(Token::SuccessStrong)),
                _ => unreachable!(),
            };
            ui.begin(
                Style {
                    dir: Dir::Row,
                    width: Size::FILL,
                    height: Size::Fixed(23.0),
                    pad: Sides { left: 20.0 + indent, right: 6.0, top: 0.0, bottom: 0.0 },
                    gap: 4.0,
                    align: Align::Center,
                    ..Default::default()
                },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, rr| {
                    for d in 0..depth {
                        let gx = rr.min.x + 10.0 + d as f32 * 16.0;
                        cx.painter.rect(Rect::from_xywh(gx, rr.min.y - 1.0, 1.0, rr.height() + 2.0), guide_c.multiply_alpha(0.7));
                    }
                })),
            );
            if let Some(k) = key_name {
                ui.text_leaf(format!("\"{k}\""), mono, key_c, false, false, Style::default());
                ui.text_leaf(":", mono, punct_c, false, false, Style::default());
            }
            ui.text_leaf(format!("{val_str}{comma}"), mono, val_c, false, true, Style { width: Size::FILL, ..Default::default() });
            ui.end();
        }
    }
}

impl Drop for JsonViewer<'_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

// =========================================================== KeyValueEditor

/// One row of a [`KeyValueEditor`] (environment variable or HTTP header).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KvPair {
    pub key: String,
    pub value: String,
    pub enabled: bool,
    pub secret: bool,
}

impl KvPair {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        KvPair { key: key.into(), value: value.into(), enabled: true, secret: false }
    }
    pub fn secret(mut self) -> Self {
        self.secret = true;
        self
    }
    pub fn disabled_row(mut self) -> Self {
        self.enabled = false;
        self
    }
}

#[derive(Default)]
struct KvMem {
    revealed: BTreeSet<usize>,
}

/// Editable table of `KEY = VALUE` pairs with per-row enable toggle, secret
/// bullet masking, delete action and an `+ Add` button.
pub struct KeyValueEditor<'u> {
    ui: &'u mut Ui,
    key: String,
    pairs: &'u mut Vec<KvPair>,
    key_placeholder: String,
    value_placeholder: String,
    add_label: String,
    shown: bool,
}

impl<'u> KeyValueEditor<'u> {
    pub fn placeholders(mut self, key_ph: impl Into<String>, val_ph: impl Into<String>) -> Self {
        self.key_ph_set(key_ph.into(), val_ph.into());
        self
    }
    fn key_ph_set(&mut self, k: String, v: String) {
        self.key_placeholder = k;
        self.value_placeholder = v;
    }
    pub fn add_label(mut self, label: impl Into<String>) -> Self {
        self.add_label = label.into();
        self
    }
    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("kv_editor", &self.key));
        let mut changed = false;

        let r_card = ui.look().card.radius;
        let card_bg = ui.color(Token::Surface);
        let card_bd = ui.color(Token::BorderSubtle);
        let hdr_bg = ui.color(Token::SurfaceMuted).multiply_alpha(0.55);
        let div_c = ui.color(Token::BorderSubtle);

        ui.begin(
            Style { width: Size::FILL, dir: Dir::Column, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), card_bg, 1.0, card_bd))),
        );

        // Column header.
        let top_rad = (r_card - 1.0).max(0.0);
        ui.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(12.0, 8.0), gap: 10.0, align: Align::Center, ..Default::default() },
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
        ui.space(20.0);
        let mut st_h = ui.text_style(TextRole::Caption);
        st_h.weight = 600;
        let hc = ui.color(Token::TextSecondary);
        ui.text_leaf("KEY", st_h, hc, false, false, Style { width: Size::Fill(0.95), ..Default::default() });
        ui.text_leaf("VALUE", st_h, hc, false, false, Style { width: Size::Fill(1.35), ..Default::default() });
        ui.space(52.0);
        ui.end();

        let mut remove_idx: Option<usize> = None;
        let mut revealed = ui.mem::<KvMem>(id).revealed.clone();

        for (i, pair) in self.pairs.iter_mut().enumerate() {
            let sep_c = ui.color(Token::BorderSubtle);
            ui.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(12.0, 7.0), gap: 10.0, align: Align::Center, ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| {
                    cx.painter.rect(Rect::from_xywh(r.min.x, r.max.y - 1.0, r.width(), 1.0), sep_c.multiply_alpha(0.65));
                })),
            );

            let chk = ui.scope(("en", i), |ui| ui.checkbox("", &mut pair.enabled).show());
            if chk.changed {
                changed = true;
            }

            // Key inline input.
            let k_eid = id.with(("k_ed", i));
            let k_fid = id.with(("k_fr", i));
            let k_focused = ui.focused() == Some(k_eid);
            let k_hovered = ui.pointer().zip(ui.rect_of(k_fid)).is_some_and(|(p, r)| r.contains(p));
            if ui.interact(k_fid, Sense { click: true, drag: false, focusable: false, scroll: false }, false).clicked {
                ui.request_focus(k_eid);
            }
            ui.field_begin(
                k_fid,
                Chrome { hovered: k_hovered, focused: k_focused, error: false, disabled: false },
                Style { dir: Dir::Row, width: Size::Fill(0.95), pad: Sides::xy(9.0, 5.0), align: Align::Center, ..Default::default() },
            );
            let mut k_opts = EditOpts::new(ui);
            k_opts.style = ui.text_style(TextRole::Mono);
            k_opts.style.line_height = 18.0;
            k_opts.placeholder = self.key_placeholder.clone();
            if !pair.enabled {
                k_opts.color = Some(ui.color(Token::TextMuted));
            }
            let k_out = ui.inline_edit(k_eid, &mut pair.key, k_opts);
            if k_out.changed {
                changed = true;
            }
            ui.end();

            // Value inline input.
            let v_eid = id.with(("v_ed", i));
            let v_fid = id.with(("v_fr", i));
            let v_focused = ui.focused() == Some(v_eid);
            let v_hovered = ui.pointer().zip(ui.rect_of(v_fid)).is_some_and(|(p, r)| r.contains(p));
            if ui.interact(v_fid, Sense { click: true, drag: false, focusable: false, scroll: false }, false).clicked {
                ui.request_focus(v_eid);
            }
            let is_rev = revealed.contains(&i);
            ui.field_begin(
                v_fid,
                Chrome { hovered: v_hovered, focused: v_focused, error: false, disabled: false },
                Style { dir: Dir::Row, width: Size::Fill(1.35), pad: Sides::xy(9.0, 5.0), gap: 4.0, align: Align::Center, ..Default::default() },
            );
            let mut v_opts = EditOpts::new(ui);
            v_opts.style = ui.text_style(TextRole::Mono);
            v_opts.style.line_height = 18.0;
            v_opts.placeholder = self.value_placeholder.clone();
            v_opts.mask = pair.secret && !is_rev;
            if !pair.enabled {
                v_opts.color = Some(ui.color(Token::TextMuted));
            }
            let v_out = ui.inline_edit(v_eid, &mut pair.value, v_opts);
            if v_out.changed {
                changed = true;
            }
            ui.end();

            // Trailing secret toggle + remove button.
            ui.begin(Style { dir: Dir::Row, width: Size::Fixed(52.0), gap: 2.0, align: Align::Center, justify: Justify::End, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            if pair.secret {
                let eye_id = id.with(("eye", i));
                if ui.field_icon_button(eye_id, if is_rev { "eye-off" } else { "eye" }, 1.0, true) {
                    if !revealed.remove(&i) {
                        revealed.insert(i);
                    }
                }
            } else {
                let lock_id = id.with(("sec", i));
                if ui.field_icon_button(lock_id, "key-round", 1.0, false) {
                    pair.secret = true;
                    changed = true;
                }
            }
            let rm_id = id.with(("rm", i));
            if ui.field_icon_button(rm_id, "x", 1.0, true) {
                remove_idx = Some(i);
            }
            ui.end();

            ui.end(); // row
        }

        if let Some(ri) = remove_idx {
            self.pairs.remove(ri);
            revealed.remove(&ri);
            changed = true;
        }
        ui.mem::<KvMem>(id).revealed = revealed;

        // Footer bar.
        ui.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(12.0, 8.0), align: Align::Center, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );
        let add_lbl = self.add_label.clone();
        if ui.scope("add_kv", |ui| ui.button(&add_lbl).secondary().small().icon("plus").clicked()) {
            self.pairs.push(KvPair::new("", ""));
            changed = true;
        }
        ui.spacer();
        let active_n = self.pairs.iter().filter(|p| p.enabled && !p.key.is_empty()).count();
        let st_c = ui.text_style(TextRole::Caption);
        let mc = ui.color(Token::TextMuted);
        ui.text_leaf(format!("{active_n} active"), st_c, mc, false, false, Style::default());
        ui.end();

        ui.end();
        let rect = ui.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, hovered: false, clicked: changed, changed, focused: false }
    }
}

impl Drop for KeyValueEditor<'_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

// =============================================================== LogViewer

/// Severity level of a [`LogEntry`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn label(self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERR",
        }
    }
}

/// Structured console entry displayed by [`LogViewer`].
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub time: String,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
}

impl LogEntry {
    pub fn new(time: impl Into<String>, level: LogLevel, target: impl Into<String>, message: impl Into<String>) -> Self {
        LogEntry { time: time.into(), level, target: target.into(), message: message.into() }
    }
}

#[derive(Default)]
struct LogViewerMem {
    level_filter: Option<LogLevel>,
    query: String,
}

/// Filterable structured console with `All` / `INFO` / `WARN` / `ERROR` chips,
/// live search highlighting and a scrollable monospace viewport.
pub struct LogViewer<'u, 'a> {
    ui: &'u mut Ui,
    key: String,
    entries: &'a [LogEntry],
    height: f32,
    shown: bool,
}

impl<'u, 'a> LogViewer<'u, 'a> {
    pub fn height(mut self, px: f32) -> Self {
        self.height = px.max(120.0);
        self
    }
    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("log_viewer", &self.key));
        let r_card = ui.look().card.radius;
        let card_bg = ui.color(Token::Surface);
        let card_bd = ui.color(Token::BorderSubtle);
        let hdr_bg = ui.color(Token::SurfaceMuted).multiply_alpha(0.6);
        let div_c = ui.color(Token::BorderSubtle);

        let mut level_filter = ui.mem::<LogViewerMem>(id).level_filter;
        let mut query = ui.mem::<LogViewerMem>(id).query.clone();

        let n_info = self.entries.iter().filter(|e| e.level == LogLevel::Info).count();
        let n_warn = self.entries.iter().filter(|e| e.level == LogLevel::Warn).count();
        let n_err = self.entries.iter().filter(|e| e.level == LogLevel::Error).count();

        ui.begin(
            Style { width: Size::FILL, dir: Dir::Column, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), card_bg, 1.0, card_bd))),
        );

        // Top toolbar: level filter chips + compact search + copy.
        let top_rad = (r_card - 1.0).max(0.0);
        ui.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(10.0, 8.0), gap: 6.0, align: Align::Center, ..Default::default() },
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

        let chips: &[(Option<LogLevel>, String)] = &[
            (None, format!("All {}", self.entries.len())),
            (Some(LogLevel::Info), format!("Info {n_info}")),
            (Some(LogLevel::Warn), format!("Warn {n_warn}")),
            (Some(LogLevel::Error), format!("Error {n_err}")),
        ];
        for (ci, (lvl, lbl)) in chips.iter().enumerate() {
            let mut active = level_filter == *lvl;
            if ui.scope(("lvl_chip", ci), |ui| ui.chip(lbl, &mut active).clicked) {
                level_filter = if active { *lvl } else { None };
            }
        }

        ui.spacer();

        // Compact search box.
        let s_eid = id.with("search_ed");
        let s_fid = id.with("search_fr");
        let s_focused = ui.focused() == Some(s_eid);
        let s_hovered = ui.pointer().zip(ui.rect_of(s_fid)).is_some_and(|(p, r)| r.contains(p));
        if ui.interact(s_fid, Sense { click: true, drag: false, focusable: false, scroll: false }, false).clicked {
            ui.request_focus(s_eid);
        }
        ui.field_begin(
            s_fid,
            Chrome { hovered: s_hovered, focused: s_focused, error: false, disabled: false },
            Style { dir: Dir::Row, width: Size::Fixed(164.0), pad: Sides::xy(8.0, 4.0), gap: 6.0, align: Align::Center, ..Default::default() },
        );
        let ic_c = ui.color(Token::TextMuted);
        ui.icon_leaf("search", 13.0, ic_c);
        let mut s_opts = EditOpts::new(ui);
        s_opts.placeholder = "Filter logs…".into();
        s_opts.style = ui.text_style(TextRole::Caption);
        s_opts.style.line_height = 17.0;
        let s_out = ui.inline_edit(s_eid, &mut query, s_opts);
        if s_out.key(Key::Escape) && !query.is_empty() {
            query.clear();
        }
        if !query.is_empty() && ui.field_icon_button(id.with("clr_q"), "x", 1.0, false) {
            query.clear();
        }
        ui.end();

        ui.end(); // toolbar

        let filtered: Vec<(&LogEntry, Vec<Range<usize>>)> = self
            .entries
            .iter()
            .filter(|e| level_filter.is_none_or(|lf| e.level == lf))
            .filter_map(|e| {
                if query.trim().is_empty() {
                    Some((e, vec![]))
                } else {
                    let m_msg = fuzzy_match(&e.message, &query);
                    let m_tgt = fuzzy_match(&e.target, &query);
                    if let Some(hl) = m_msg {
                        Some((e, hl))
                    } else {
                        m_tgt.map(|_| (e, vec![]))
                    }
                }
            })
            .collect();

        ui.mem::<LogViewerMem>(id).level_filter = level_filter;
        ui.mem::<LogViewerMem>(id).query = query;

        // Scrollable log body.
        ui.begin(
            Style { width: Size::FILL, height: Size::Fixed(self.height), dir: Dir::Column, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            true,
            None,
        );
        ui.scroll(id.with("log_scroll")).pad(Sides::xy(10.0, 8.0)).gap(2.0).show(|ui| {
            let mut mono = ui.text_style(TextRole::Mono);
            mono.line_height = 19.0;
            if filtered.is_empty() {
                let mc = ui.color(Token::TextMuted);
                ui.begin(Style { width: Size::FILL, pad: Sides::xy(6.0, 24.0), align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                ui.text_leaf("No matching log entries.", mono, mc, false, false, Style::default());
                ui.end();
            }
            for (idx, (entry, msg_hl)) in filtered.iter().enumerate() {
                let row_id = id.with(("lrow", idx));
                let rix = ui.interact(row_id, Sense { click: true, drag: false, focusable: false, scroll: false }, false);
                if rix.clicked {
                    ui.set_copied_text(format!("{} [{}] {}: {}", entry.time, entry.level.label().trim(), entry.target, entry.message));
                }
                let h = ui.fade(row_id, 0, rix.hovered, 0.08, 0.12, Ease::Linear);
                let hov_c = ui.color(Token::Hover).multiply_alpha(h);
                let row_tint = match entry.level {
                    LogLevel::Error => ui.color(Token::DangerSubtle).multiply_alpha(0.32),
                    LogLevel::Warn => ui.color(Token::WarningSubtle).multiply_alpha(0.22),
                    LogLevel::Info => Color::TRANSPARENT,
                };
                let (lvl_fg, lvl_bg) = match entry.level {
                    LogLevel::Info => (ui.color(Token::InfoStrong), ui.color(Token::InfoSubtle)),
                    LogLevel::Warn => (ui.color(Token::WarningStrong), ui.color(Token::WarningSubtle)),
                    LogLevel::Error => (ui.color(Token::DangerStrong), ui.color(Token::DangerSubtle)),
                };
                let time_c = ui.color(Token::TextMuted);
                let tgt_c = ui.color(Token::AccentStrong);
                let msg_c = if entry.level == LogLevel::Error { ui.color(Token::DangerStrong) } else { ui.color(Token::Foreground) };
                let hl_fg = ui.color(Token::AccentStrong);
                let hl_bg = ui.color(Token::AccentSubtle);

                let time_s = entry.time.clone();
                let lvl_s = entry.level.label();
                let tgt_s = entry.target.clone();
                let msg_s = entry.message.clone();
                let hl_cl = msg_hl.clone();
                let mono_cl = mono;

                ui.leaf(
                    Style { width: Size::FILL, ..Default::default() },
                    vec2(260.0, 23.0),
                    Some(row_id),
                    Sense { click: true, drag: false, focusable: false, scroll: false },
                    CursorIcon::Pointer,
                    Some(Box::new(move |cx, r| {
                        let bg = row_tint.over(hov_c);
                        if bg.a > 0.0 {
                            cx.painter.rrect(r, Corners::all(5.0), bg, 0.0, Color::TRANSPARENT);
                        }
                        let y = r.min.y + (r.height() - mono_cl.line_height) / 2.0;
                        let mut x = r.min.x + 6.0;
                        let g_t = cx.galley(&time_s, &mono_cl);
                        cx.painter.text(cx.text, &g_t, vec2(x, y), time_c);
                        x += g_t.width + 8.0;

                        let g_l = cx.galley(lvl_s, &mono_cl);
                        let pill_w = 38.0;
                        let pill = Rect::from_xywh(x, r.min.y + 3.0, pill_w, r.height() - 6.0);
                        cx.painter.rrect(pill, Corners::all(4.0), lvl_bg, 0.0, Color::TRANSPARENT);
                        cx.painter.text(cx.text, &g_l, vec2(x + (pill_w - g_l.width) / 2.0, y), lvl_fg);
                        x += pill.width() + 8.0;

                        let g_tg = cx.galley(&tgt_s, &mono_cl);
                        cx.painter.text(cx.text, &g_tg, vec2(x, y), tgt_c);
                        x += g_tg.width + 8.0;

                        cx.painter.push_clip(Rect::from_xywh(x, r.min.y, (r.max.x - x - 4.0).max(0.0), r.height()));
                        paint_highlighted(cx, &msg_s, &mono_cl, vec2(x, y), msg_c, &hl_cl, hl_fg, hl_bg);
                        cx.painter.pop_clip();
                    })),
                );
            }
        });
        ui.end();

        ui.end();
        let rect = ui.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, hovered: false, clicked: false, changed: false, focused: false }
    }
}

impl Drop for LogViewer<'_, '_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

impl Ui {
    /// Syntax-highlighted code block with optional line numbers and Copy button.
    pub fn code_block(&mut self, key: impl Into<String>, code: impl Into<String>) -> CodeBlock<'_> {
        CodeBlock {
            ui: self,
            key: key.into(),
            code: code.into(),
            language: "rust".into(),
            title: None,
            line_numbers: true,
            highlight_lines: vec![],
            copyable: true,
            shown: false,
        }
    }

    /// Interactive collapsible JSON viewer.
    pub fn json_viewer(&mut self, key: impl Into<String>, json: impl Into<String>) -> JsonViewer<'_> {
        JsonViewer { ui: self, key: key.into(), json: json.into(), title: None, default_depth: 2, shown: false }
    }

    /// Editable key–value table for environment variables or HTTP headers.
    pub fn key_value_editor<'u>(&'u mut self, key: impl Into<String>, pairs: &'u mut Vec<KvPair>) -> KeyValueEditor<'u> {
        KeyValueEditor {
            ui: self,
            key: key.into(),
            pairs,
            key_placeholder: "VARIABLE_NAME".into(),
            value_placeholder: "value".into(),
            add_label: "Add variable".into(),
            shown: false,
        }
    }

    /// Filterable structured log console.
    pub fn log_viewer<'u, 'a>(&'u mut self, key: impl Into<String>, entries: &'a [LogEntry]) -> LogViewer<'u, 'a> {
        LogViewer { ui: self, key: key.into(), entries, height: 220.0, shown: false }
    }
}
