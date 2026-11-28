//! Developer tools you work with: a query builder, a regex tester, a
//! feature-flag table and a dependency graph.

use std::collections::BTreeSet;
use std::ops::Range;

use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides, Vec2};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

const SERIES: [Token; 6] = [Token::Series0, Token::Series1, Token::Series2, Token::Series3, Token::Series4, Token::Series5];

/// A card frame from the look.
fn card(ui: &mut Ui, id: Option<crate::Id>, pad: f32, gap: f32) {
    let rad = ui.look().card.radius;
    let bg = ui.color(Token::Surface);
    let edge = ui.color(Token::BorderSubtle);
    ui.begin(Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(pad), gap, ..Default::default() }, id, Sense::NONE, CursorIcon::Default, false, Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(rad), bg, 1.0, edge))));
}

fn spacer(ui: &mut Ui) {
    ui.begin(Style { width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
    ui.end();
}

/// Rows rising in: the newest is reset to 0 when it's added.
fn rise(ui: &mut Ui, id: crate::Id) -> f32 {
    if ui.reduced_motion() {
        return 1.0;
    }
    let mo = ui.motion();
    ui.spring(id, 0, 1.0, mo.smooth).clamp(0.0, 1.0)
}

// ============================================================ query builder

/// What a field holds, which decides its operators and value input.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldKind {
    Text,
    Number,
    Choice(Vec<String>),
}

#[derive(Clone, Debug)]
pub struct QueryField {
    pub name: String,
    pub kind: FieldKind,
}

impl QueryField {
    pub fn text(name: &str) -> Self {
        QueryField { name: name.into(), kind: FieldKind::Text }
    }
    pub fn number(name: &str) -> Self {
        QueryField { name: name.into(), kind: FieldKind::Number }
    }
    pub fn choice(name: &str, options: &[&str]) -> Self {
        QueryField { name: name.into(), kind: FieldKind::Choice(options.iter().map(|s| s.to_string()).collect()) }
    }
}

/// `field` and `op` index into the fields and that field's operators.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct QueryRule {
    pub field: usize,
    pub op: usize,
    pub value: String,
}

/// Rules joined by AND (or OR when `any`), and nested groups.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct QueryGroup {
    pub any: bool,
    pub rules: Vec<QueryRule>,
    pub groups: Vec<QueryGroup>,
}

/// Operators for a kind of field, as shown and as written in the query.
pub fn query_ops(kind: &FieldKind) -> &'static [(&'static str, &'static str)] {
    match kind {
        FieldKind::Text => &[("is", "="), ("is not", "!="), ("contains", "CONTAINS"), ("starts with", "STARTS WITH")],
        FieldKind::Number => &[("=", "="), ("≠", "!="), (">", ">"), ("<", "<"), ("≥", ">="), ("≤", "<=")],
        FieldKind::Choice(_) => &[("is", "="), ("is not", "!=")],
    }
}

impl QueryGroup {
    /// The group as a query string: `status = "open" AND (a > 2 OR b = "x")`.
    pub fn to_query(&self, fields: &[QueryField]) -> String {
        let mut parts: Vec<String> = self
            .rules
            .iter()
            .filter_map(|r| {
                let f = fields.get(r.field)?;
                let (_, op) = query_ops(&f.kind).get(r.op).copied().unwrap_or(("", "="));
                let v = if f.kind == FieldKind::Number { if r.value.trim().is_empty() { "0".to_owned() } else { r.value.trim().to_owned() } } else { format!("\"{}\"", r.value) };
                Some(format!("{} {op} {v}", f.name.to_lowercase().replace(' ', "_")))
            })
            .collect();
        for g in &self.groups {
            let q = g.to_query(fields);
            if !q.is_empty() {
                parts.push(if g.rules.len() + g.groups.len() > 1 { format!("({q})") } else { q });
            }
        }
        parts.join(if self.any { " OR " } else { " AND " })
    }
}

#[derive(Default)]
struct QueryMem {
    fresh: BTreeSet<Vec<usize>>,
}

fn query_group(ui: &mut Ui, id: crate::Id, path: Vec<usize>, fields: &[QueryField], g: &mut QueryGroup, changed: &mut bool, remove_me: &mut bool) {
    let lk = ui.look().clone();
    let gap = lk.control.gap[1];
    let depth = path.len();
    let gid = id.with(("g", path.clone()));
    let accent = ui.color(Token::Accent);
    let nested_bg = ui.color(Token::SurfaceMuted);
    let rr = lk.control.radius[2];
    let enter = rise(ui, gid);
    let nested = depth > 0;
    ui.column().fill_width().layer(enter, vec2(0.0, (1.0 - enter) * -12.0)).show(|ui| {
        ui.begin(
            Style { dir: Dir::Column, width: Size::FILL, gap, pad: if nested { Sides { left: gap * 1.5, right: gap, top: gap, bottom: gap } } else { Sides::all(0.0) }, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                if nested {
                    cx.painter.rrect(r, Corners::all(rr), nested_bg, 0.0, Color::TRANSPARENT);
                    cx.painter.rrect(Rect::from_xywh(r.min.x, r.min.y + rr * 0.5, 3.0, r.height() - rr), Corners::all(1.5), accent, 0.0, Color::TRANSPARENT);
                }
            })),
        );
        // Header: All / Any, then add buttons.
        ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: gap * 0.75, align: Align::Center, wrap: true, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let mut any = g.any;
        let seg = ui.scope(("join", path.clone()), |ui| ui.segmented(&mut any, &[(false, "All"), (true, "Any")]));
        if any != g.any {
            g.any = any;
            *changed = true;
        }
        let _ = seg;
        let sec = ui.color(Token::TextSecondary);
        let st = ui.text_style(TextRole::Body);
        ui.text_leaf("of these match".to_owned(), st, sec, false, false, Style::default());
        spacer(ui);
        let add_rule = ui.scope(("add_rule", path.clone()), |ui| ui.button("Rule").icon("plus").ghost().small().tooltip("Add a rule").show().clicked);
        let add_group = depth < 2 && ui.scope(("add_group", path.clone()), |ui| ui.button("Group").icon("brackets").ghost().small().tooltip("Add a nested group").show().clicked);
        if nested && ui.scope(("rm_group", path.clone()), |ui| ui.icon_button("trash-2").ghost().small().tooltip("Remove group").show().clicked) {
            *remove_me = true;
        }
        ui.end();

        // Rules.
        let mut remove = None;
        for (i, r) in g.rules.iter_mut().enumerate() {
            let mut rp = path.clone();
            rp.push(i);
            let rid = id.with(("r", rp.clone()));
            let enter = rise(ui, rid);
            ui.row().fill_width().layer(enter, vec2(0.0, (1.0 - enter) * -10.0)).show(|ui| {
                ui.scope(("rule", rp.clone()), |ui| {
                    ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: gap * 0.5, align: Align::Center, wrap: true, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                    let names: Vec<(usize, &str)> = fields.iter().enumerate().map(|(k, f)| (k, f.name.as_str())).collect();
                    let mut field = r.field.min(fields.len().saturating_sub(1));
                    let fr = ui.select(&mut field, &names).width(132.0).show();
                    ui.a11y_update(fr.id, |a| a.label = Some("Field".into()));
                    if field != r.field {
                        r.field = field;
                        r.op = 0;
                        r.value.clear();
                        *changed = true;
                    }
                    let kind = fields.get(r.field).map(|f| f.kind.clone()).unwrap_or(FieldKind::Text);
                    let ops: Vec<(usize, &str)> = query_ops(&kind).iter().enumerate().map(|(k, o)| (k, o.0)).collect();
                    let mut op = r.op.min(ops.len() - 1);
                    let or = ui.select(&mut op, &ops).width(if kind == FieldKind::Number { 64.0 } else { 116.0 }).show();
                    ui.a11y_update(or.id, |a| a.label = Some("Operator".into()));
                    if op != r.op {
                        r.op = op;
                        *changed = true;
                    }
                    match &kind {
                        FieldKind::Choice(opts) => {
                            let list: Vec<(usize, &str)> = opts.iter().enumerate().map(|(k, o)| (k, o.as_str())).collect();
                            let mut v = opts.iter().position(|o| *o == r.value).unwrap_or(usize::MAX);
                            ui.begin(Style { width: Size::FILL, min: vec2(96.0, 0.0), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                            let vr = ui.select(&mut v, &list).placeholder("Value").fill_width().show();
                            ui.a11y_update(vr.id, |a| a.label = Some("Value".into()));
                            ui.end();
                            if let Some(o) = opts.get(v) {
                                if *o != r.value {
                                    r.value = o.clone();
                                    *changed = true;
                                }
                            }
                        }
                        _ => {
                            let bad = kind == FieldKind::Number && !r.value.trim().is_empty() && r.value.trim().parse::<f64>().is_err();
                            ui.begin(Style { width: Size::FILL, min: vec2(96.0, 0.0), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                            let before = r.value.clone();
                            let t = ui.text_input(&mut r.value).placeholder(if kind == FieldKind::Number { "0" } else { "Value" }).error(bad.then_some("Not a number")).fill_width().show();
                            ui.a11y_update(t.id, |a| a.label = Some("Value".into()));
                            ui.end();
                            if r.value != before {
                                *changed = true;
                            }
                        }
                    }
                    if ui.icon_button("x").ghost().small().tooltip("Remove rule").show().clicked {
                        remove = Some(i);
                    }
                    ui.end();
                });
            });
        }
        if let Some(i) = remove {
            g.rules.remove(i);
            *changed = true;
        }
        // Nested groups.
        let mut drop_group = None;
        for (k, sub) in g.groups.iter_mut().enumerate() {
            let mut p = path.clone();
            p.push(1000 + k);
            let mut rm = false;
            query_group(ui, id, p, fields, sub, changed, &mut rm);
            if rm {
                drop_group = Some(k);
            }
        }
        if let Some(k) = drop_group {
            g.groups.remove(k);
            *changed = true;
        }
        if add_rule {
            let mut rp = path.clone();
            rp.push(g.rules.len());
            g.rules.push(QueryRule::default());
            ui.anims.set_spring(id.with(("r", rp)), 0, 0.0);
            *changed = true;
        }
        if add_group {
            let mut p = path.clone();
            p.push(1000 + g.groups.len());
            g.groups.push(QueryGroup { any: !g.any, rules: vec![QueryRule::default()], groups: vec![] });
            ui.anims.set_spring(id.with(("g", p)), 0, 0.0);
            *changed = true;
        }
        ui.end();
    });
}

// ============================================================ regex tester

/// Regex options the tester's toggles control.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RegexFlags {
    pub ignore_case: bool,
    pub multi_line: bool,
    pub dot_all: bool,
}

/// One match: byte ranges into the test text, the whole match and each
/// capture group (`None` when a group didn't take part).
#[derive(Clone, Debug, PartialEq)]
pub struct RegexMatch {
    pub range: Range<usize>,
    pub groups: Vec<Option<Range<usize>>>,
}

#[derive(Default)]
struct RegexMem {
    key: u64,
    result: Option<Result<Vec<RegexMatch>, String>>,
    err_at: f64,
    hover: Option<usize>,
}

// ============================================================ feature flags

/// A flag with its state per environment and a rollout for the last one.
#[derive(Clone, Debug, Default)]
pub struct FeatureFlag {
    pub key: String,
    pub description: String,
    pub on: Vec<bool>,
    /// Percent of traffic in the last environment, 0..100.
    pub rollout: f32,
    /// Not changed in a long time: shown with a Stale tag.
    pub stale: bool,
}

#[derive(Default)]
struct FlagsMem {
    filter: String,
    flashed: Vec<(usize, f64)>,
}

// ============================================================ dependency graph

/// A package in the graph.
#[derive(Clone, Debug, Default)]
pub struct DepNode {
    pub name: String,
    pub version: String,
}

#[derive(Default)]
struct GraphMem {
    hover: Option<usize>,
    last_w: f32,
}

struct GraphLayout {
    rects: Vec<Rect>,
    size: Vec2,
    horizontal: bool,
    layer: Vec<usize>,
    order: Vec<Vec<usize>>,
}

fn layout_graph(n: usize, edges: &[(usize, usize)], widths: &[f32], node_h: f32, gap: f32, avail: f32) -> GraphLayout {
    // Layer = longest path from a root (a package nothing depends on).
    let mut layer = vec![0usize; n];
    for _ in 0..n {
        let mut moved = false;
        for &(a, b) in edges {
            if a < n && b < n && layer[b] < layer[a] + 1 && layer[a] + 1 < n {
                layer[b] = layer[a] + 1;
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
    let layers = layer.iter().copied().max().map_or(0, |m| m + 1);
    let mut order: Vec<Vec<usize>> = vec![vec![]; layers];
    for i in 0..n {
        order[layer[i]].push(i);
    }
    // Two barycenter sweeps to cut crossings.
    for _ in 0..2 {
        for l in 1..layers {
            let pos: Vec<f32> = (0..n).map(|i| order[layer[i]].iter().position(|x| *x == i).unwrap_or(0) as f32).collect();
            order[l].sort_by(|a, b| {
                let bc = |v: usize| {
                    let ps: Vec<f32> = edges.iter().filter(|e| e.1 == v && layer[e.0] + 1 == l).map(|e| pos[e.0]).collect();
                    if ps.is_empty() { pos[v] } else { ps.iter().sum::<f32>() / ps.len() as f32 }
                };
                bc(*a).partial_cmp(&bc(*b)).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }
    let widest = widths.iter().copied().fold(0.0, f32::max);
    let col_min = widest + gap * 4.0;
    let horizontal = layers as f32 * col_min <= avail || avail <= 0.0;
    let mut rects = vec![Rect::ZERO; n];
    let tallest = order.iter().map(|o| o.len()).max().unwrap_or(1);
    let size;
    if horizontal {
        let col_w = if avail > 0.0 { (avail / layers.max(1) as f32).max(col_min) } else { col_min };
        let h = tallest as f32 * (node_h + gap) - gap;
        for (l, ids) in order.iter().enumerate() {
            let col_h = ids.len() as f32 * (node_h + gap) - gap;
            for (k, &i) in ids.iter().enumerate() {
                let x = l as f32 * col_w + (col_w - widths[i]) * 0.5;
                let y = (h - col_h) * 0.5 + k as f32 * (node_h + gap);
                rects[i] = Rect::from_xywh(x, y, widths[i], node_h);
            }
        }
        size = vec2(col_w * layers as f32, h.max(node_h));
    } else {
        // Too narrow: layers become rows, packages wrap inside a row.
        let w = avail.max(widest);
        let mut y = 0.0;
        for ids in &order {
            let mut lines: Vec<Vec<usize>> = vec![vec![]];
            let mut lw = 0.0;
            for &i in ids {
                if lw + widths[i] > w && !lines.last().unwrap().is_empty() {
                    lines.push(vec![]);
                    lw = 0.0;
                }
                lines.last_mut().unwrap().push(i);
                lw += widths[i] + gap;
            }
            for line in lines {
                let lw: f32 = line.iter().map(|i| widths[*i]).sum::<f32>() + gap * (line.len().saturating_sub(1)) as f32;
                let mut x = (w - lw) * 0.5;
                for i in line {
                    rects[i] = Rect::from_xywh(x, y, widths[i], node_h);
                    x += widths[i] + gap;
                }
                y += node_h + gap;
            }
            y += gap * 2.0;
        }
        size = vec2(w, (y - gap * 3.0).max(node_h));
    }
    GraphLayout { rects, size, horizontal, layer, order }
}

/// Everything reachable from `start` along edges (`down`) or against them.
fn reach(n: usize, edges: &[(usize, usize)], start: usize, down: bool) -> BTreeSet<usize> {
    let mut seen = BTreeSet::new();
    let mut stack = vec![start];
    while let Some(v) = stack.pop() {
        for &(a, b) in edges {
            let (from, to) = if down { (a, b) } else { (b, a) };
            if from == v && to < n && seen.insert(to) {
                stack.push(to);
            }
        }
    }
    seen
}

impl Ui {
    /// Build a filter from rules and nested groups. Shows the query it
    /// makes underneath. Returns `changed` when the query changed.
    pub fn query_builder(&mut self, key: impl std::hash::Hash, fields: &[QueryField], query: &mut QueryGroup) -> Response {
        let id = self.make_id(("query", key));
        let lk = self.look().clone();
        let gap = lk.control.gap[1];
        let mut changed = false;
        let _ = self.mem::<QueryMem>(id).fresh.len();
        card(self, Some(id), gap * 1.5, gap);
        let mut rm = false;
        query_group(self, id, vec![], fields, query, &mut changed, &mut rm);
        // The query it makes.
        let q = query.to_query(fields);
        let st = self.text_style(TextRole::Mono);
        let sec = self.color(Token::TextSecondary);
        let fg = self.color(Token::Foreground);
        let muted_bg = self.color(Token::SurfaceMuted);
        let rr = lk.control.radius[1];
        self.begin(Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(gap, gap * 0.75), gap, align: Align::Start, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(rr), muted_bg, 0.0, Color::TRANSPARENT))));
        let ic = self.color(Token::TextMuted);
        self.icon_leaf("filter", st.size + 2.0, ic);
        let text = if q.is_empty() { "Everything matches".to_owned() } else { q.clone() };
        let n = self.text_leaf(text, st, if q.is_empty() { sec } else { fg }, true, false, Style { width: Size::FILL, ..Default::default() });
        let qa = q.clone();
        self.a11y_node(n, || A11y::new(Role::Status).label("Query").value(if qa.is_empty() { "Everything matches".to_owned() } else { qa }));
        if !q.is_empty() {
            self.scope(id.with("copy"), |ui| ui.copy_button("", &q));
        }
        self.end();
        self.end();
        Response { id, rect: self.rect_of(id).unwrap_or_default(), changed, ..Default::default() }
    }

    /// Edit a pattern and test text side by side with the matches: the text
    /// with every match and group marked, and a list of matches with their
    /// groups. `run` does the matching (bring your regex engine); it's only
    /// called when the pattern, flags or text change.
    pub fn regex_tester(
        &mut self,
        key: impl std::hash::Hash,
        pattern: &mut String,
        flags: &mut RegexFlags,
        text: &mut String,
        run: impl Fn(&str, RegexFlags, &str) -> Result<Vec<RegexMatch>, String>,
    ) -> Response {
        let id = self.make_id(("regex", key));
        let lk = self.look().clone();
        let mo = self.motion();
        let reduced = self.reduced_motion();
        let gap = lk.control.gap[1];
        let now = self.time();
        let mono = self.text_style(TextRole::Mono);
        let fg = self.color(Token::Foreground);
        let sec = self.color(Token::TextSecondary);
        let muted = self.color(Token::TextMuted);
        let danger = self.color(Token::Danger);
        let ring = self.color(Token::FocusRing);
        let accent = self.color(Token::Accent);
        let mut changed = false;

        card(self, Some(id), gap * 1.5, gap);
        // Pattern row, which shakes once when the pattern turns invalid.
        let err_at = self.mem::<RegexMem>(id).err_at;
        let shake_t = (now - err_at) as f32;
        let shake = if reduced || err_at == 0.0 || shake_t > 0.4 {
            0.0
        } else {
            self.anims.keep_alive();
            (shake_t * 50.0).sin() * 6.0 * (1.0 - shake_t / 0.4)
        };
        let st_l = self.text_style(TextRole::Label);
        let pl = self.text_leaf("Pattern".to_owned(), st_l, sec, false, false, Style::default());
        self.a11y_hide(pl);
        self.row().fill_width().gap(gap * 0.5).align(Align::Center).layer(1.0, vec2(shake, 0.0)).show(|ui| {
            let slash = ui.text_leaf("/".to_owned(), mono, muted, false, false, Style::default());
            ui.a11y_hide(slash);
            ui.begin(Style { width: Size::FILL, min: vec2(80.0, 0.0), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let before = pattern.clone();
            let t = ui.scope(id.with("pat"), |ui| ui.text_input(pattern).mono().placeholder("pattern").fill_width().show());
            ui.a11y_update(t.id, |a| a.label = Some("Pattern".into()));
            if *pattern != before {
                changed = true;
            }
            ui.end();
            let slash = ui.text_leaf("/".to_owned(), mono, muted, false, false, Style::default());
            ui.a11y_hide(slash);
            for (k, (letter, name)) in [("i", "Ignore case"), ("m", "Multiline: ^ and $ match at lines"), ("s", "Dot matches newline")].into_iter().enumerate() {
                let fid = id.with(("flag", k));
                let fix = ui.interact(fid, Sense::CLICK, false);
                let (act, held) = super::key_activate(ui, &fix, true);
                let cur = match k {
                    0 => &mut flags.ignore_case,
                    1 => &mut flags.multi_line,
                    _ => &mut flags.dot_all,
                };
                if fix.clicked || act {
                    *cur = !*cur;
                    changed = true;
                }
                let on = *cur;
                let on_t = ui.fade(fid, 0, on, mo.selected, mo.selected, Ease::Out);
                let (hov, press) = ui.state_weights(&fix, held);
                let squash = if reduced { 1.0 } else { ui.spring(fid, 1, if fix.pressed || held { 0.9 } else { 1.0 }, mo.bouncy) };
                let (fa, fgp) = ui.focus_anim(&fix);
                let fill = ui.color(Token::SurfaceMuted).mix(ui.color(Token::AccentSubtle), on_t).over(ui.color(Token::Hover).multiply_alpha(hov * 0.6 + press * 0.4));
                let tc = sec.mix(accent, on_t);
                let sz = lk.control.height[0] * 0.85;
                let r0 = lk.control.radius[0];
                ui.leaf(Style::default(), vec2(sz, sz), Some(fid), Sense::CLICK, CursorIcon::Pointer, Some(Box::new(move |cx, r| {
                    let rr = Rect::from_center_size(r.center(), r.size() * squash);
                    cx.painter.rrect(rr, Corners::all(r0), fill, 0.0, Color::TRANSPARENT);
                    let g = cx.galley(letter, &mono);
                    cx.painter.text(cx.text, &g, vec2((rr.center().x - g.width * 0.5).round(), (rr.center().y - g.height * 0.5).round()), tc);
                    if fa > 0.0 {
                        cx.defer(r, Box::new(move |cx, r2| cx.focus_ring(r2, r0, fa, fgp, ring)));
                    }
                })));
                ui.a11y(fid, || A11y::new(Role::Button).label(name).toggled(on));
            }
        });

        // Run the matcher only when something changed.
        let hash = {
            use std::hash::{Hash, Hasher};
            let mut h = ahash::AHasher::default();
            pattern.hash(&mut h);
            flags.hash(&mut h);
            text.hash(&mut h);
            h.finish()
        };
        let stale = {
            let m = self.mem::<RegexMem>(id);
            m.key != hash || m.result.is_none()
        };
        if stale {
            let res = if pattern.is_empty() { Ok(vec![]) } else { run(pattern, *flags, text) };
            let m = self.mem::<RegexMem>(id);
            let was_err = matches!(m.result, Some(Err(_)));
            if res.is_err() && !was_err && m.result.is_some() {
                m.err_at = now;
            }
            m.key = hash;
            m.result = Some(res);
        }
        let result = self.mem::<RegexMem>(id).result.clone().unwrap_or(Ok(vec![]));

        // Status.
        let st_c = self.text_style(TextRole::Caption);
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: gap * 0.5, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let (icon, msg, col) = match &result {
            Err(e) => ("circle-x", e.clone(), danger),
            Ok(m) if pattern.is_empty() => ("regex", "Type a pattern".to_owned(), muted),
            Ok(m) if m.is_empty() => ("circle-dashed", "No matches".to_owned(), muted),
            Ok(m) => {
                let groups = m.first().map_or(0, |x| x.groups.len());
                ("circle-check", format!("{} match{}{}", m.len(), if m.len() == 1 { "" } else { "es" }, if groups > 0 { format!(", {groups} group{}", if groups == 1 { "" } else { "s" }) } else { String::new() }), self.color(Token::Success))
            }
        };
        self.icon_leaf(icon, st_c.size + 3.0, col);
        let sn = self.text_leaf(msg.clone(), st_c, if result.is_err() { danger } else { sec }, true, false, Style { width: Size::FILL, ..Default::default() });
        self.a11y_node(sn, || A11y::new(Role::Status).label(msg));
        self.end();

        // Test text.
        let before = text.clone();
        self.scope(id.with("text"), |ui| ui.text_area(text).label("Test text").mono().rows(2, 5).fill_width().show());
        if *text != before {
            changed = true;
        }

        // The text with matches marked.
        let matches = result.clone().unwrap_or_default();
        let hover = self.mem::<RegexMem>(id).hover;
        let hover_a = self.fade(id, 1, hover.is_some(), mo.hover_in, mo.hover_out, Ease::Linear);
        let show = if reduced { 1.0 } else { self.anims.spring(id.with(("marks", hash)), 0, 1.0, mo.smooth) };
        let reveal = if reduced { 1.0 } else { self.anims.spring_from(id.with(("reveal", hash)), 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0) };
        let _ = show;
        let char_w = self.galley("0", &mono).width;
        let row_h = mono.line_height;
        let pad = gap;
        // Soft-wrap at spaces to the width the preview had last frame (the
        // font is monospace, so columns are exact). Rows keep their byte
        // offsets, so matches spanning a break are marked on both rows.
        let pid = id.with("preview");
        let cols = self.rect_of(pid).map(|r| ((r.width() - pad * 2.0) / char_w).floor().max(8.0) as usize).unwrap_or(usize::MAX);
        let lines: Vec<(usize, String)> = {
            let mut rows = Vec::new();
            let mut at = 0;
            for l in text.split('\n') {
                let mut s = at;
                let mut rest = l;
                while rest.chars().count() > cols {
                    let cut = rest.char_indices().nth(cols).map(|x| x.0).unwrap_or(rest.len());
                    let brk = rest[..cut].rfind(' ').map(|i| i + 1).filter(|i| *i > 0).unwrap_or(cut);
                    rows.push((s, rest[..brk].to_owned()));
                    s += brk;
                    rest = &rest[brk..];
                }
                rows.push((s, rest.to_owned()));
                at += l.len() + 1;
            }
            rows
        };
        let mark = self.color(Token::AccentSubtle);
        let mark_bd = accent.multiply_alpha(0.5);
        let series: Vec<Color> = SERIES.iter().map(|t| self.color(*t)).collect();
        let hi = accent.multiply_alpha(0.25);
        let code_bg = self.color(Token::SurfaceMuted);
        let rr = lk.control.radius[1];
        let h = lines.len() as f32 * row_h + pad * 2.0;
        let ms = matches.clone();
        let txt = text.clone();
        let series_p = series.clone();
        let preview = self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(200.0, h),
            Some(pid),
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(rr), code_bg, 0.0, Color::TRANSPARENT);
                cx.painter.push_clip(r);
                let x0 = r.min.x + pad;
                let col_of = |line_start: usize, line: &str, at: usize| line[..(at - line_start).min(line.len())].chars().count() as f32;
                for (li, (ls, line)) in lines.iter().enumerate() {
                    let y = r.min.y + pad + li as f32 * row_h;
                    let le = ls + line.len();
                    for (mi, m) in ms.iter().enumerate() {
                        let (a, b) = (m.range.start.max(*ls), m.range.end.min(le));
                        if m.range.start > le || m.range.end < *ls || (a >= b && !(m.range.is_empty() && m.range.start >= *ls && m.range.start <= le)) {
                            continue;
                        }
                        let xa = x0 + col_of(*ls, line, a) * char_w;
                        let xb = x0 + col_of(*ls, line, b) * char_w;
                        let w = ((xb - xa) * reveal).max(2.0);
                        let rect = Rect::from_xywh(xa - 1.0, y, w + 2.0, row_h);
                        cx.painter.rrect_unsnapped(rect, Corners::all(3.0), [mark; 2], 1.0, mark_bd);
                        if hover == Some(mi) {
                            cx.painter.rrect_unsnapped(rect.expand(1.0), Corners::all(4.0), [hi.multiply_alpha(hover_a); 2], 0.0, Color::TRANSPARENT);
                        }
                        for (gi, g) in m.groups.iter().enumerate() {
                            let Some(g) = g else { continue };
                            let (ga, gb) = (g.start.max(*ls), g.end.min(le));
                            if ga >= gb {
                                continue;
                            }
                            let gxa = x0 + col_of(*ls, line, ga) * char_w;
                            let gxb = x0 + col_of(*ls, line, gb) * char_w;
                            cx.painter.rect(Rect::from_xywh(gxa, y + row_h - 3.0, (gxb - gxa) * reveal, 2.0), series_p[(gi + 1) % series_p.len()]);
                        }
                    }
                    let g = cx.galley(line, &mono);
                    cx.painter.text(cx.text, &g, vec2(x0, y), fg);
                }
                let _ = &txt;
                cx.painter.pop_clip();
            })),
        );
        let n_m = matches.len();
        self.a11y_node(preview, || A11y::new(Role::Image).label(format!("Test text with {n_m} matches marked")));

        // The match list.
        let mut hovered = None;
        if !matches.is_empty() {
            let show_n = matches.len().min(8);
            for (mi, m) in matches.iter().take(show_n).enumerate() {
                let rid = id.with(("m", mi));
                let rix = self.interact(rid, Sense::HOVER, false);
                if rix.hovered {
                    hovered = Some(mi);
                }
                let hv = self.fade(rid, 0, rix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
                let fill = self.color(Token::Hover).multiply_alpha(hv);
                let r0 = lk.control.radius[0];
                self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: gap * 0.75, pad: Sides::xy(gap * 0.5, 3.0), align: Align::Center, wrap: true, ..Default::default() }, Some(rid), Sense::HOVER, CursorIcon::Default, false, Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r0), fill, 0.0, Color::TRANSPARENT))));
                let mut stn = st_c;
                stn.tabular = true;
                self.text_leaf(format!("{}", mi + 1), stn, muted, false, false, Style { width: Size::Fixed(char_w * 2.5), ..Default::default() });
                let whole = text.get(m.range.clone()).unwrap_or("").to_owned();
                self.text_leaf(if whole.is_empty() { "(empty)".to_owned() } else { whole.clone() }, mono, fg, false, true, Style { min: vec2(40.0, 0.0), ..Default::default() });
                for (gi, g) in m.groups.iter().enumerate() {
                    let v = g.clone().and_then(|g| text.get(g)).unwrap_or("–").to_owned();
                    let c = series[(gi + 1) % series.len()];
                    let bg = c.multiply_alpha(0.14);
                    let r1 = lk.control.radius[0];
                    self.begin(Style { dir: Dir::Row, pad: Sides::xy(6.0, 1.0), gap: 4.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r1), bg, 0.0, Color::TRANSPARENT))));
                    self.text_leaf(format!("{}", gi + 1), st_c, c, false, false, Style::default());
                    self.text_leaf(v, mono, fg, false, true, Style::default());
                    self.end();
                }
                self.end();
                let desc: Vec<String> = m.groups.iter().enumerate().map(|(gi, g)| format!("group {}: {}", gi + 1, g.clone().and_then(|g| text.get(g)).unwrap_or("none"))).collect();
                self.a11y(rid, || A11y::new(Role::ListItem).label(format!("Match {}: {whole}", mi + 1)).description(desc.join(", ")));
            }
            if matches.len() > show_n {
                self.text_leaf(format!("and {} more", matches.len() - show_n), st_c, muted, false, false, Style::default());
            }
        }
        self.mem::<RegexMem>(id).hover = hovered;
        self.end();
        Response { id, rect: self.rect_of(id).unwrap_or_default(), changed, ..Default::default() }
    }

    /// Flags with a switch per environment and a rollout for the last
    /// environment; filter by name. Returns the flag that changed.
    pub fn feature_flags(&mut self, key: impl std::hash::Hash, envs: &[&str], flags: &mut [FeatureFlag]) -> Option<usize> {
        let id = self.make_id(("flags", key));
        let lk = self.look().clone();
        let mo = self.motion();
        let gap = lk.control.gap[1];
        let now = self.time();
        let narrow = self.rect_of(id).is_some_and(|r| r.width() < 600.0);
        let fg = self.color(Token::Foreground);
        let sec = self.color(Token::TextSecondary);
        let edge = self.color(Token::BorderSubtle);
        let mono = self.text_style(TextRole::Mono);
        let st = self.text_style(TextRole::Caption);
        let st_l = self.text_style(TextRole::Label);
        let env_w = lk.control.height[0] * 2.6;
        let roll_w = lk.control.height[0] * 5.0;
        let mut changed = None;

        card(self, Some(id), 0.0, 0.0);
        // Filter.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, pad: Sides::all(gap * 1.25), gap, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, Some(Box::new(move |cx, r| cx.painter.rect(Rect { min: vec2(r.min.x, r.max.y - 1.0), max: r.max }, edge))));
        let mut filter = std::mem::take(&mut self.mem::<FlagsMem>(id).filter);
        let fr = self.scope(id.with("filter"), |ui| ui.text_input(&mut filter).icon("search").placeholder("Filter flags").fill_width().show());
        self.a11y_update(fr.id, |a| a.label = Some("Filter flags".into()));
        let shown: Vec<usize> = (0..flags.len()).filter(|i| filter.is_empty() || flags[*i].key.to_lowercase().contains(&filter.to_lowercase()) || flags[*i].description.to_lowercase().contains(&filter.to_lowercase())).collect();
        let on_count = flags.iter().filter(|f| f.on.last().copied().unwrap_or(false)).count();
        let summary = format!("{} of {} live", on_count, flags.len());
        let sn = self.text_leaf(summary.clone(), st, sec, false, false, Style::default());
        self.a11y_node(sn, || A11y::new(Role::Status).label(summary));
        self.end();
        self.mem::<FlagsMem>(id).filter = filter;

        // Column heads (wide only).
        if !narrow {
            self.begin(Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(gap * 1.25, gap * 0.5), gap, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, Some(Box::new(move |cx, r| cx.painter.rect(Rect { min: vec2(r.min.x, r.max.y - 1.0), max: r.max }, edge))));
            self.text_leaf("Flag".to_owned(), st_l, sec, false, false, Style { width: Size::FILL, ..Default::default() });
            for e in envs {
                let n = self.text_leaf(e.to_string(), st_l, sec, false, true, Style { width: Size::Fixed(env_w), ..Default::default() });
                self.set_text_align(n, lumen_text::TextAlign::Center);
            }
            let last = envs.last().copied().unwrap_or("");
            self.text_leaf(format!("{last} rollout"), st_l, sec, false, true, Style { width: Size::Fixed(roll_w), ..Default::default() });
            self.end();
        }

        for (row_i, &i) in shown.iter().enumerate() {
            let f = &mut flags[i];
            let rid = id.with(("flag", &f.key));
            // A row flashes when it changes.
            let flash_at = self.mem::<FlagsMem>(id).flashed.iter().find(|(k, _)| *k == i).map(|x| x.1).unwrap_or(0.0);
            let ft = ((now - flash_at) as f32 / (mo.layer_in * 4.0 + 0.3)).clamp(0.0, 1.0);
            let flash = if flash_at > 0.0 && ft < 1.0 && !self.reduced_motion() {
                self.anims.keep_alive();
                1.0 - ft
            } else {
                0.0
            };
            let flash_c = self.color(Token::AccentSubtle).multiply_alpha(flash);
            let last_row = row_i + 1 == shown.len();
            let enter = rise(self, rid);
            let fkey = f.key.clone();
            self.column().fill_width().layer(enter, vec2(0.0, 0.0)).show(|ui| {
                ui.begin(
                    Style { dir: if narrow { Dir::Column } else { Dir::Row }, width: Size::FILL, pad: Sides::xy(gap * 1.25, gap), gap, align: if narrow { Align::Start } else { Align::Center }, ..Default::default() },
                    Some(rid),
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    Some(Box::new(move |cx, r| {
                        cx.painter.rect(r, flash_c);
                        if !last_row {
                            cx.painter.rect(Rect { min: vec2(r.min.x, r.max.y - 1.0), max: r.max }, edge);
                        }
                    })),
                );
                ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 2.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                ui.begin(Style { dir: Dir::Row, gap: gap * 0.5, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                let mut mk = mono;
                mk.weight = 600;
                ui.text_leaf(f.key.clone(), mk, fg, false, true, Style::default());
                if f.stale {
                    ui.badge("Stale", super::Tone::Warning);
                }
                ui.end();
                ui.text_leaf(f.description.clone(), st, sec, true, false, Style { width: Size::FILL, ..Default::default() });
                ui.end();
                ui.begin(Style { dir: Dir::Row, gap, align: Align::Center, wrap: true, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                for (e, env) in envs.iter().enumerate() {
                    if f.on.len() <= e {
                        f.on.resize(e + 1, false);
                    }
                    ui.begin(Style { dir: Dir::Row, width: if narrow { Size::Hug } else { Size::Fixed(env_w) }, gap: gap * 0.5, align: Align::Center, justify: crate::layout::Justify::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                    if narrow {
                        let n = ui.text_leaf(env.to_string(), st, sec, false, false, Style::default());
                        ui.a11y_hide(n);
                    }
                    let r = ui.scope(("sw", &fkey, e), |ui| ui.switch("", &mut f.on[e]).show());
                    let name = format!("{} in {env}", f.key);
                    ui.a11y_update(r.id, |a| a.label = Some(name));
                    if r.changed {
                        changed = Some(i);
                    }
                    ui.end();
                }
                ui.end();
                let live = f.on.last().copied().unwrap_or(false);
                ui.begin(Style { dir: Dir::Row, width: if narrow { Size::FILL } else { Size::Fixed(roll_w) }, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                let before = f.rollout;
                let readout = format!("{:.0}%", f.rollout);
                let sr = ui.scope(("roll", &fkey), |ui| ui.slider("", &mut f.rollout, 0.0..=100.0).step(5.0).width(f32::INFINITY).readout(readout).disabled(!live).show());
                let name = format!("{} rollout", f.key);
                ui.a11y_update(sr.id, |a| a.label = Some(name));
                if f.rollout != before {
                    changed = Some(i);
                }
                ui.end();
                ui.end();
            });
        }
        if shown.is_empty() {
            self.begin(Style { width: Size::FILL, pad: Sides::all(gap * 1.5), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.text_leaf("No flags match the filter".to_owned(), st, sec, false, false, Style::default());
            self.end();
        }
        self.end();
        if let Some(i) = changed {
            let m = self.mem::<FlagsMem>(id);
            m.flashed.retain(|(k, _)| *k != i);
            m.flashed.push((i, now));
        }
        changed
    }

    /// Packages laid out in layers (each one right of what depends on it),
    /// with curved edges. Selecting a package lights what it depends on and
    /// what depends on it, and dims the rest. Edges are `(dependent,
    /// dependency)` index pairs.
    pub fn dependency_graph(&mut self, key: impl std::hash::Hash, nodes: &[DepNode], edges: &[(usize, usize)], selected: &mut Option<usize>) -> Response {
        let id = self.make_id(("depgraph", key));
        let lk = self.look().clone();
        let mo = self.motion();
        let reduced = self.reduced_motion();
        let gap = lk.control.gap[1];
        let n = nodes.len();
        let st = self.text_style(TextRole::Label);
        let st_v = self.text_style(TextRole::Caption);
        let pad_x = lk.control.pad_x[0];
        let widths: Vec<f32> = nodes.iter().map(|d| self.galley(&d.name, &st).width + self.galley(&d.version, &st_v).width + pad_x * 2.0 + gap * 0.5).collect();
        let node_h = lk.control.height[0];
        let avail = self.mem::<GraphMem>(id).last_w - gap * 2.0;
        let lay = layout_graph(n, edges, &widths, node_h, gap, avail);
        let leaf_id = id.with("canvas");
        let sense = Sense { click: true, drag: false, focusable: true, scroll: false };
        let ix = self.interact(leaf_id, sense, false);
        let hit = |p: Vec2, r: Rect| lay.rects.iter().position(|nr| nr.translate(r.min).contains(p));
        let old = *selected;
        let hover = if ix.hovered { ix.pointer.and_then(|p| hit(p, ix.rect)) } else { None };
        self.mem::<GraphMem>(id).hover = hover;
        if ix.clicked {
            let h = ix.pointer.and_then(|p| hit(p, ix.rect));
            *selected = if h == *selected { None } else { h };
        }
        if ix.focused && n > 0 {
            for (k, _, _) in self.keys_for(leaf_id) {
                let cur = selected.unwrap_or(lay.order.first().and_then(|o| o.first()).copied().unwrap_or(0));
                let l = lay.layer[cur];
                let pos = lay.order[l].iter().position(|x| *x == cur).unwrap_or(0);
                let (along_next, along_prev, across_next, across_prev) = if lay.horizontal { (Key::ArrowRight, Key::ArrowLeft, Key::ArrowDown, Key::ArrowUp) } else { (Key::ArrowDown, Key::ArrowUp, Key::ArrowRight, Key::ArrowLeft) };
                let nearest = |layer: usize| -> Option<usize> {
                    let c = lay.rects[cur].center();
                    lay.order.get(layer)?.iter().copied().min_by(|a, b| (lay.rects[*a].center() - c).length().partial_cmp(&(lay.rects[*b].center() - c).length()).unwrap())
                };
                let next = if selected.is_none() {
                    Some(cur)
                } else if k == along_next {
                    nearest(l + 1)
                } else if k == along_prev && l > 0 {
                    nearest(l - 1)
                } else if k == across_next {
                    lay.order[l].get(pos + 1).copied()
                } else if k == across_prev && pos > 0 {
                    lay.order[l].get(pos - 1).copied()
                } else if k == Key::Escape {
                    *selected = None;
                    None
                } else {
                    None
                };
                if let Some(v) = next {
                    *selected = Some(v);
                }
            }
        }
        // Light the selection (or hover): dependencies and dependents.
        let focus = selected.or(hover);
        let down = focus.map(|f| reach(n, edges, f, true)).unwrap_or_default();
        let up = focus.map(|f| reach(n, edges, f, false)).unwrap_or_default();
        let lit = self.fade(id, 0, focus.is_some(), mo.hover_in, mo.hover_out * 2.0, Ease::Linear);
        let (ra, rg) = self.focus_anim(&ix);
        let grow = if reduced { 1.0 } else { self.anims.spring_from(id, 1, 0.0, 1.0, mo.gentle).clamp(0.0, 1.0) };
        let sel_pop = if reduced { 1.0 } else { self.spring(id.with(("sel", *selected)), 0, 1.0, mo.bouncy) };
        if old != *selected && !reduced {
            self.anims.set_spring(id.with(("sel", *selected)), 0, 0.92);
        }

        let fg = self.color(Token::Foreground);
        let sec = self.color(Token::TextSecondary);
        let accent = self.color(Token::Accent);
        let up_c = self.color(Token::Series3);
        let edge_c = self.color(Token::BorderStrong);
        let node_bg = self.color(Token::SurfaceRaised);
        let node_bd = self.color(Token::Border);
        let acc_sub = self.color(Token::AccentSubtle);
        let ring = self.color(Token::FocusRing);
        let bg = self.color(Token::Surface);
        let frame_bd = self.color(Token::BorderSubtle);
        let rad = lk.card.radius;
        let nr = lk.control.radius[1];
        let layers = lay.order.len().max(1);
        let names: Vec<(String, String)> = nodes.iter().map(|d| (d.name.clone(), d.version.clone())).collect();
        let rects = lay.rects.clone();
        let horizontal = lay.horizontal;
        let layer_of = lay.layer.clone();
        let es: Vec<(usize, usize)> = edges.iter().copied().filter(|e| e.0 < n && e.1 < n).collect();
        let size = lay.size;
        let sel_now = *selected;

        self.begin(Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(gap), gap, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(rad), bg, 1.0, frame_bd))));
        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            size,
            Some(leaf_id),
            sense,
            if hover.is_some() { CursorIcon::Pointer } else { CursorIcon::Default },
            Some(Box::new(move |cx, r| {
                let o = vec2(r.min.x + (r.width() - size.x).max(0.0) * 0.5, r.min.y);
                // Layers arrive in turn; edges draw in after their source.
                let layer_t = |l: usize| (grow * (layers as f32 + 1.0) - l as f32).clamp(0.0, 1.0);
                let state = |i: usize| -> (f32, Color) {
                    if focus.is_none() {
                        (1.0, edge_c)
                    } else if Some(i) == focus {
                        (1.0, accent)
                    } else if down.contains(&i) {
                        (1.0, accent)
                    } else if up.contains(&i) {
                        (1.0, up_c)
                    } else {
                        (1.0 - 0.65 * lit, edge_c)
                    }
                };
                for &(a, b) in &es {
                    let ra_ = rects[a].translate(o);
                    let rb = rects[b].translate(o);
                    let (p0, p3) = if horizontal { (vec2(ra_.max.x, ra_.center().y), vec2(rb.min.x, rb.center().y)) } else { (vec2(ra_.center().x, ra_.max.y), vec2(rb.center().x, rb.min.y)) };
                    let d = if horizontal { vec2((p3.x - p0.x) * 0.5, 0.0) } else { vec2(0.0, (p3.y - p0.y) * 0.5) };
                    let (p1, p2) = (p0 + d, p3 - d);
                    let t_end = layer_t(layer_of[a] + 1);
                    if t_end <= 0.0 {
                        continue;
                    }
                    let on_path = focus.is_some() && ((Some(a) == focus || down.contains(&a)) && down.contains(&b) || (Some(b) == focus || up.contains(&b)) && up.contains(&a));
                    let col = if on_path { if down.contains(&b) { accent } else { up_c } } else { edge_c.multiply_alpha(1.0 - 0.7 * lit) };
                    let w = if on_path { 2.0 } else { 1.25 };
                    let steps = 20;
                    let pts: Vec<Vec2> = (0..=((steps as f32 * t_end) as usize)).map(|k| {
                        let t = k as f32 / steps as f32;
                        let u = 1.0 - t;
                        p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
                    }).collect();
                    if pts.len() > 1 {
                        cx.painter.polyline(&pts, w, col);
                    }
                    if t_end >= 1.0 {
                        let k = 5.0;
                        let dir = if horizontal { vec2(1.0, 0.0) } else { vec2(0.0, 1.0) };
                        let side = vec2(-dir.y, dir.x);
                        cx.painter.polyline(&[p3 - dir * k + side * k * 0.6, p3, p3 - dir * k - side * k * 0.6], w, col);
                    }
                }
                for (i, nrect) in rects.iter().enumerate() {
                    let t = layer_t(layer_of[i]);
                    if t <= 0.0 {
                        continue;
                    }
                    let (a, c) = state(i);
                    let is_sel = sel_now == Some(i);
                    let s = if is_sel { sel_pop } else { 1.0 } * (0.9 + 0.1 * t);
                    let rr = Rect::from_center_size(nrect.translate(o).center(), nrect.size() * s);
                    let alpha = a * t;
                    let fill = if is_sel { acc_sub } else { node_bg };
                    let bd = if focus.is_some() && (Some(i) == focus || down.contains(&i) || up.contains(&i)) { c } else { node_bd };
                    cx.painter.rrect(rr, Corners::all(nr), fill.multiply_alpha(alpha), if is_sel { 1.5 } else { 1.0 }, bd.multiply_alpha(alpha));
                    let (name, ver) = &names[i];
                    let g = cx.galley(name, &st);
                    let gv = cx.galley(ver, &st_v);
                    let tw = g.width + gap * 0.5 + gv.width;
                    let x = rr.center().x - tw * 0.5;
                    cx.painter.text(cx.text, &g, vec2(x.round(), (rr.center().y - g.height * 0.5).round()), fg.multiply_alpha(alpha));
                    cx.painter.text(cx.text, &gv, vec2((x + g.width + gap * 0.5).round(), (rr.center().y - gv.height * 0.5).round()), sec.multiply_alpha(alpha));
                }
                if ra > 0.0 {
                    cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr, 6.0, ra, rg, ring)));
                }
            })),
        );
        // Caption: what the selection depends on and what depends on it.
        let info = match *selected {
            Some(i) => {
                let deps = edges.iter().filter(|e| e.0 == i).count();
                let users = edges.iter().filter(|e| e.1 == i).count();
                format!("{} {} · {} direct dependenc{} · used by {} · {} in total below it", nodes[i].name, nodes[i].version, deps, if deps == 1 { "y" } else { "ies" }, users, reach(n, edges, i, true).len())
            }
            None => format!("{} packages · {} edges · select one to trace it", n, edges.len()),
        };
        let st_c = self.text_style(TextRole::Caption);
        let inn = self.text_leaf(info.clone(), st_c, sec, true, false, Style { width: Size::FILL, ..Default::default() });
        self.a11y_hide(inn);
        self.end();
        let w = self.rect_of(id).map(|r| r.width()).unwrap_or(0.0);
        self.mem::<GraphMem>(id).last_w = w;
        self.a11y(leaf_id, || A11y::new(Role::Canvas).label("Dependency graph").value(info).description("Arrow keys move between packages, Escape clears."));
        Response { id: leaf_id, rect: ix.rect, hovered: ix.hovered, clicked: ix.clicked, changed: old != *selected, focused: ix.focused }
    }
}
