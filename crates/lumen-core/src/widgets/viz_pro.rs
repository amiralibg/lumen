//! Pro visualizations, calendars, flow canvases and developer tools:
//! [`SankeyLink`], [`Ui::contribution_graph`], [`HistogramMarker`],
//! [`KpiMetric`], [`ScheduleEvent`], [`GraphNode`], [`CommitRow`], and
//! [`TestCase`].

use super::chart::{compact, tooltip_panel, Tip};
use super::toast::Tone;
use crate::anim::Ease;
use crate::color::Color;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides, Vec2};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};

const SENSE_HOVER: Sense = Sense { click: true, drag: false, focusable: false, scroll: false };

// ============================================================== SankeyChart

/// Directed flow link `(source_node, target_node, value)` in [`Ui::sankey_chart`].
#[derive(Clone, Copy, Debug)]
pub struct SankeyLink {
    pub from: usize,
    pub to: usize,
    pub value: f32,
}

impl SankeyLink {
    pub fn new(from: usize, to: usize, value: f32) -> Self {
        SankeyLink { from, to, value }
    }
}

// ========================================================= HistogramMarker

/// Vertical reference line (e.g. `p50`, `p95`, `p99`) in [`Ui::histogram`].
#[derive(Clone, Debug)]
pub struct HistogramMarker {
    /// Fractional horizontal position `0.0..=1.0` across the histogram domain.
    pub position: f32,
    pub label: String,
    pub tone: Tone,
}

impl HistogramMarker {
    pub fn new(position: f32, label: impl Into<String>, tone: Tone) -> Self {
        HistogramMarker { position: position.clamp(0.0, 1.0), label: label.into(), tone }
    }
}

// =============================================================== KpiMetric

/// Metric cell in [`Ui::kpi_strip`].
#[derive(Clone, Debug)]
pub struct KpiMetric {
    pub label: String,
    pub value: String,
    pub delta_pct: f32,
    pub trend: Vec<f32>,
}

impl KpiMetric {
    pub fn new(label: impl Into<String>, value: impl Into<String>, delta_pct: f32, trend: Vec<f32>) -> Self {
        KpiMetric { label: label.into(), value: value.into(), delta_pct, trend }
    }
}

// =========================================================== ScheduleEvent

/// Time-block event in [`Ui::day_schedule`].
#[derive(Clone, Debug)]
pub struct ScheduleEvent {
    pub day: usize,
    pub start_hour: f32,
    pub duration_hours: f32,
    pub title: String,
    pub room: Option<String>,
    pub tone: Tone,
}

impl ScheduleEvent {
    pub fn new(day: usize, start_hour: f32, duration_hours: f32, title: impl Into<String>, tone: Tone) -> Self {
        ScheduleEvent {
            day,
            start_hour,
            duration_hours: duration_hours.max(0.5),
            title: title.into(),
            room: None,
            tone,
        }
    }
    pub fn room(mut self, r: impl Into<String>) -> Self {
        self.room = Some(r.into());
        self
    }
}

// ============================================================== NodeEditor

/// Draggable node in [`Ui::node_editor`].
#[derive(Clone, Debug)]
pub struct GraphNode {
    pub title: String,
    pub pos: Vec2,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub tone: Tone,
}

impl GraphNode {
    pub fn new(title: impl Into<String>, pos: Vec2, inputs: &[&str], outputs: &[&str], tone: Tone) -> Self {
        GraphNode {
            title: title.into(),
            pos,
            inputs: inputs.iter().map(|s| (*s).to_owned()).collect(),
            outputs: outputs.iter().map(|s| (*s).to_owned()).collect(),
            tone,
        }
    }
}

/// Directed wire `(from_node, from_port, to_node, to_port)` in [`Ui::node_editor`].
#[derive(Clone, Copy, Debug)]
pub struct GraphWire {
    pub from_node: usize,
    pub from_port: usize,
    pub to_node: usize,
    pub to_port: usize,
}

impl GraphWire {
    pub fn new(from_node: usize, from_port: usize, to_node: usize, to_port: usize) -> Self {
        GraphWire { from_node, from_port, to_node, to_port }
    }
}

#[derive(Default, Clone, Copy)]
struct NodeDragMem {
    active_node: Option<usize>,
    grab_offset: Vec2,
}

// ============================================================= CommitGraph

/// Commit entry in [`Ui::commit_graph`].
#[derive(Clone, Debug)]
pub struct CommitRow {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub time: String,
    pub lane: usize,
    /// Optional parent lane connections `(to_lane)` drawn down to the next row.
    pub parents: Vec<usize>,
    pub ref_badge: Option<String>,
}

impl CommitRow {
    pub fn new(sha: impl Into<String>, message: impl Into<String>, author: impl Into<String>, time: impl Into<String>, lane: usize, parents: Vec<usize>) -> Self {
        CommitRow {
            sha: sha.into(),
            message: message.into(),
            author: author.into(),
            time: time.into(),
            lane,
            parents,
            ref_badge: None,
        }
    }
    pub fn badge(mut self, b: impl Into<String>) -> Self {
        self.ref_badge = Some(b.into());
        self
    }
}

// ============================================================ TestExplorer

/// Status of a [`TestCase`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestState {
    Passed,
    Failed,
    Skipped,
}

/// Suite entry in [`Ui::test_explorer`].
#[derive(Clone, Debug)]
pub struct TestCase {
    pub name: String,
    pub suite: String,
    pub state: TestState,
    pub duration_ms: f32,
    pub failure_msg: Option<String>,
}

impl TestCase {
    pub fn passed(suite: impl Into<String>, name: impl Into<String>, duration_ms: f32) -> Self {
        TestCase { suite: suite.into(), name: name.into(), state: TestState::Passed, duration_ms, failure_msg: None }
    }
    pub fn failed(suite: impl Into<String>, name: impl Into<String>, duration_ms: f32, msg: impl Into<String>) -> Self {
        TestCase { suite: suite.into(), name: name.into(), state: TestState::Failed, duration_ms, failure_msg: Some(msg.into()) }
    }
    pub fn skipped(suite: impl Into<String>, name: impl Into<String>) -> Self {
        TestCase { suite: suite.into(), name: name.into(), state: TestState::Skipped, duration_ms: 0.0, failure_msg: None }
    }
}

#[derive(Default)]
struct TestMem {
    failed_only: bool,
    expanded_idx: Option<usize>,
}

fn tone_pair(ui: &Ui, tone: Tone) -> (Color, Color) {
    match tone {
        Tone::Neutral => (ui.color(Token::SurfaceMuted), ui.color(Token::TextSecondary)),
        Tone::Accent | Tone::Info => (ui.color(Token::AccentSubtle), ui.color(Token::AccentStrong)),
        Tone::Success => (ui.color(Token::SuccessSubtle), ui.color(Token::SuccessStrong)),
        Tone::Warning => (ui.color(Token::WarningSubtle), ui.color(Token::WarningStrong)),
        Tone::Danger => (ui.color(Token::DangerSubtle), ui.color(Token::DangerStrong)),
    }
}

impl Ui {
    /// Multi-stage Sankey flow chart drawn with smooth cubic-Bezier ribbons.
    pub fn sankey_chart(&mut self, key: impl std::hash::Hash, nodes: &[&str], links: &[SankeyLink], height: f32) -> Response {
        let id = self.make_id(("sankey", key));
        let ix = self.interact(id, SENSE_HOVER, false);
        let mo = self.motion();
        let enter = self.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);

        let n = nodes.len();
        let mut stage = vec![0usize; n];
        for _ in 0..n {
            for lk in links {
                if lk.from < n && lk.to < n && stage[lk.to] <= stage[lk.from] {
                    stage[lk.to] = stage[lk.from] + 1;
                }
            }
        }
        let max_stage = stage.iter().copied().max().unwrap_or(1).max(1);

        let mut in_sum = vec![0.0f32; n];
        let mut out_sum = vec![0.0f32; n];
        for lk in links {
            if lk.from < n && lk.to < n {
                out_sum[lk.from] += lk.value;
                in_sum[lk.to] += lk.value;
            }
        }
        let node_val: Vec<f32> = (0..n).map(|i| in_sum[i].max(out_sum[i]).max(1.0)).collect();

        let palette = [
            self.color(Token::Series0),
            self.color(Token::Series1),
            self.color(Token::Series2),
            self.color(Token::Series3),
            self.color(Token::Series4),
            self.color(Token::Series5),
        ];
        let fg_c = self.color(Token::Foreground);
        let muted_c = self.color(Token::TextMuted);
        let st_lbl = self.text_style(TextRole::Label);
        let mut st_val = self.text_style(TextRole::Caption);
        st_val.tabular = true;
        let tip = Tip::new(self);

        let nodes_vec: Vec<String> = nodes.iter().map(|s| (*s).to_owned()).collect();
        let links_vec = links.to_vec();
        let ptr = ix.pointer.filter(|_| ix.hovered);

        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(320.0, height.max(160.0)),
            Some(id),
            SENSE_HOVER,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                if n == 0 {
                    return;
                }
                let pad_x = 14.0;
                let pad_y = 14.0;
                let bar_w = 14.0;
                let usable_w = (r.width() - pad_x * 2.0 - bar_w).max(60.0);
                let usable_h = (r.height() - pad_y * 2.0).max(60.0);

                // Compute node rectangles per stage column.
                let mut node_rects = vec![Rect::ZERO; n];
                for stg in 0..=max_stage {
                    let members: Vec<usize> = (0..n).filter(|&i| stage[i] == stg).collect();
                    if members.is_empty() {
                        continue;
                    }
                    let col_total: f32 = members.iter().map(|&i| node_val[i]).sum::<f32>().max(1.0);
                    let gap = 14.0;
                    let avail_h = (usable_h - gap * (members.len().saturating_sub(1)) as f32).max(24.0);
                    let x = r.min.x + pad_x + (stg as f32 / max_stage as f32) * usable_w;
                    let mut cur_y = r.min.y + pad_y + (usable_h - (avail_h + gap * (members.len().saturating_sub(1)) as f32)) * 0.5;
                    for &ni in &members {
                        let h = (node_val[ni] / col_total * avail_h).max(10.0) * enter;
                        node_rects[ni] = Rect::from_xywh(x, cur_y, bar_w, h);
                        cur_y += h + gap;
                    }
                }

                // Track vertical cursor offsets along each node's right (out) and left (in) edges.
                let mut out_off = vec![0.0f32; n];
                let mut in_off = vec![0.0f32; n];
                let mut hov_link: Option<(usize, Vec2)> = None;

                // Draw ribbons first!
                for (li, lk) in links_vec.iter().enumerate() {
                    if lk.from >= n || lk.to >= n {
                        continue;
                    }
                    let src = node_rects[lk.from];
                    let dst = node_rects[lk.to];
                    let h_src = (lk.value / node_val[lk.from] * src.height()).max(2.0);
                    let h_dst = (lk.value / node_val[lk.to] * dst.height()).max(2.0);
                    let y0_top = src.min.y + out_off[lk.from];
                    let y1_top = dst.min.y + in_off[lk.to];
                    out_off[lk.from] += h_src;
                    in_off[lk.to] += h_dst;

                    let x0 = src.max.x;
                    let x1 = dst.min.x;
                    if x1 <= x0 + 2.0 {
                        continue;
                    }

                    let steps = 24usize;
                    let mut upper = Vec::with_capacity(steps + 1);
                    let mut lower = Vec::with_capacity(steps + 1);
                    for s in 0..=steps {
                        let t = s as f32 / steps as f32;
                        let smooth_t = t * t * (3.0 - 2.0 * t);
                        let x = x0 + (x1 - x0) * t;
                        let yt = y0_top + (y1_top - y0_top) * smooth_t;
                        let hh = h_src + (h_dst - h_src) * smooth_t;
                        upper.push(vec2(x, yt));
                        lower.push(vec2(x, yt + hh));
                    }

                    let mid = vec2((x0 + x1) * 0.5, (y0_top + y1_top + (h_src + h_dst) * 0.5) * 0.5);
                    let is_h = ptr.is_some_and(|p| p.x >= x0 && p.x <= x1 && {
                        let t = ((p.x - x0) / (x1 - x0)).clamp(0.0, 1.0);
                        let st = t * t * (3.0 - 2.0 * t);
                        let yt = y0_top + (y1_top - y0_top) * st;
                        let hh = h_src + (h_dst - h_src) * st;
                        p.y >= yt - 3.0 && p.y <= yt + hh + 3.0
                    });
                    if is_h {
                        hov_link = Some((li, mid));
                    }

                    let c_src = palette[lk.from % palette.len()];
                    let c_dst = palette[lk.to % palette.len()];
                    let alpha = if is_h { 0.54 } else { 0.28 };
                    cx.painter.fill_between(&upper, &lower, c_src.with_alpha(alpha), c_dst.with_alpha(alpha));
                }

                // Draw node bars & labels on top. A label points into the gap
                // beside its column; when it faces a label from the other side
                // at the same height, each gets half the gap and ends in "…".
                let on_left_of = |i: usize| stage[i] == max_stage;
                let lbl_w: Vec<f32> = (0..n).map(|i| cx.galley(&nodes_vec[i], &st_lbl).width.max(cx.galley(&compact(node_val[i]), &st_val).width)).collect();
                let lbl_max: Vec<f32> = (0..n)
                    .map(|i| {
                        let nr = node_rects[i];
                        let mut max = f32::INFINITY;
                        for j in 0..n {
                            if on_left_of(i) == on_left_of(j) {
                                continue;
                            }
                            let nj = node_rects[j];
                            // The facing label's column is beside this one,
                            // across the gap the two labels share.
                            let (l, rr) = if on_left_of(i) { (nj, nr) } else { (nr, nj) };
                            let gap = rr.min.x - l.max.x;
                            if gap <= 0.0 || (nr.center().y - nj.center().y).abs() > 34.0 {
                                continue;
                            }
                            if lbl_w[i] + lbl_w[j] + 32.0 > gap {
                                max = max.min((gap * 0.5 - 12.0).max(24.0));
                            }
                        }
                        max
                    })
                    .collect();
                for i in 0..n {
                    let nr = node_rects[i];
                    let col = palette[i % palette.len()];
                    cx.painter.rrect(nr, Corners::all(4.0), col, 0.0, Color::TRANSPARENT);

                    let g_n = cx.galley_fit(&nodes_vec[i], &st_lbl, lbl_max[i]);
                    let val_s = compact(node_val[i]);
                    let g_v = cx.galley(&val_s, &st_val);
                    let on_left = stage[i] == max_stage;
                    let tx = if on_left { nr.min.x - 8.0 - g_n.width } else { nr.max.x + 8.0 };
                    let vx = if on_left { nr.min.x - 8.0 - g_v.width } else { nr.max.x + 8.0 };
                    let cy = nr.center().y;
                    cx.painter.text(cx.text, &g_n, vec2(tx, cy - g_n.height + 1.0), fg_c);
                    cx.painter.text(cx.text, &g_v, vec2(vx, cy + 1.0), muted_c);
                }

                if let Some((li, anchor)) = hov_link {
                    let lk = &links_vec[li];
                    let col = palette[lk.from % palette.len()];
                    let pct = (lk.value / node_val[lk.from] * 100.0).clamp(0.0, 100.0);
                    let title = format!("{} → {}", nodes_vec[lk.from], nodes_vec[lk.to]);
                    let rows = vec![
                        (col, "Volume".into(), compact(lk.value)),
                        (col, "Conversion".into(), format!("{pct:.1}%")),
                    ];
                    let tip_cl = tip.clone();
                    let bounds = r.expand(12.0);
                    cx.defer(r, Box::new(move |cx, _| tooltip_panel(cx, anchor, bounds, &title, &rows, &tip_cl)));
                }
            })),
        );
        Response { id, rect: ix.rect, hovered: ix.hovered, ..Default::default() }
    }

    /// GitHub-style multi-week calendar contribution heatmap (`7` weekday rows × `N` weeks).
    pub fn contribution_graph(&mut self, key: impl std::hash::Hash, weeks: &[[u8; 7]], months: &[&str]) -> Response {
        let id = self.make_id(("contrib_graph", key));
        let ix = self.interact(id, SENSE_HOVER, false);
        let mo = self.motion();
        let enter = self.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);

        let n_weeks = weeks.len().max(1);
        let total: u32 = weeks.iter().flat_map(|w| w.iter()).map(|&v| v as u32).sum();
        let max_v = weeks.iter().flat_map(|w| w.iter()).copied().max().unwrap_or(1).max(1) as f32;

        let r_card = self.look().card.radius;
        let card_bg = self.color(Token::Surface);
        let card_bd = self.color(Token::BorderSubtle);
        let empty_cell = self.color(Token::SurfaceMuted);
        let accent = self.color(Token::Success);
        let fg_c = self.color(Token::Foreground);
        let muted_c = self.color(Token::TextMuted);
        let st_cap = self.text_style(TextRole::Caption);
        let st_strong = self.text_style(TextRole::BodyStrong);
        let tip = Tip::new(self);

        let weeks_vec = weeks.to_vec();
        let months_vec: Vec<String> = months.iter().map(|s| (*s).to_owned()).collect();
        let ptr = ix.pointer.filter(|_| ix.hovered);

        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(340.0, 214.0),
            Some(id),
            SENSE_HOVER,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_card), card_bg, 1.0, card_bd);

                let pad = 16.0;
                let ramp = |lvl: f32| -> Color {
                    if lvl <= 0.001 {
                        empty_cell
                    } else {
                        empty_cell.mix(accent, (0.28 + 0.72 * lvl).clamp(0.0, 1.0))
                    }
                };

                // Top summary header + top-right Less [][][][][] More legend!
                // The legend gives way to the header when both don't fit.
                let hdr_txt = format!("{total} contributions in the last {} weeks", weeks_vec.len());
                let g_more = cx.galley("More", &st_cap);
                let g_less = cx.galley("Less", &st_cap);
                let legend_w = g_less.width + 3.0 + 5.0 * 13.0 + 3.0 + g_more.width;
                let full_w = r.width() - pad * 2.0;
                let show_legend = cx.galley(&hdr_txt, &st_strong).width + 16.0 + legend_w <= full_w;
                let hdr_w = if show_legend { full_w - legend_w - 16.0 } else { full_w };
                let g_hdr = cx.galley_fit(&hdr_txt, &st_strong, hdr_w);
                cx.painter.text(cx.text, &g_hdr, vec2(r.min.x + pad, r.min.y + 14.0), fg_c);

                if show_legend {
                    let leg_y = r.min.y + 17.0;
                    let mut rx = r.max.x - pad - g_more.width;
                    cx.painter.text(cx.text, &g_more, vec2(rx, leg_y - 2.0), muted_c);
                    rx -= 6.0;
                    for step_i in (0..5).rev() {
                        rx -= 10.0;
                        let c = ramp(step_i as f32 / 4.0);
                        cx.painter.rrect(Rect::from_xywh(rx, leg_y, 10.0, 10.0), Corners::all(2.5), c, 0.0, Color::TRANSPARENT);
                        rx -= 3.0;
                    }
                    cx.painter.text(cx.text, &g_less, vec2(rx - 3.0 - g_less.width, leg_y - 2.0), muted_c);
                }

                let left_lbl_w = 30.0;
                let grid_x0 = r.min.x + pad + left_lbl_w;
                let grid_y0 = r.min.y + 58.0;
                let avail_w = (r.width() - pad * 2.0 - left_lbl_w).max(80.0);
                let gap = 3.5;
                let cell = ((avail_w - gap * (n_weeks.saturating_sub(1)) as f32) / n_weeks as f32).clamp(6.0, 18.0);
                let step = cell + gap;

                // Month headers evenly spaced across weeks.
                if !months_vec.is_empty() {
                    for (mi, m) in months_vec.iter().enumerate() {
                        let wx = grid_x0 + (mi as f32 / months_vec.len() as f32) * (n_weeks as f32 * step);
                        let gm = cx.galley(m, &st_cap);
                        cx.painter.text(cx.text, &gm, vec2(wx, grid_y0 - 17.0), muted_c);
                    }
                }

                // Weekday labels (Mon, Wed, Fri).
                for (di, lbl) in [(1usize, "Mon"), (3, "Wed"), (5, "Fri")] {
                    let gd = cx.galley(lbl, &st_cap);
                    let y = grid_y0 + di as f32 * step + (cell - gd.height) * 0.5;
                    cx.painter.text(cx.text, &gd, vec2(r.min.x + pad, y), muted_c);
                }

                let mut hov_cell: Option<(usize, usize, u8, Vec2, Color)> = None;
                for (wi, wk) in weeks_vec.iter().enumerate() {
                    for (di, &val) in wk.iter().enumerate() {
                        let cr = Rect::from_xywh(grid_x0 + wi as f32 * step, grid_y0 + di as f32 * step, cell, cell);
                        let lvl = (val as f32 / max_v) * enter;
                        let col = ramp(lvl);
                        let is_h = ptr.is_some_and(|p| cr.contains(p));
                        cx.painter.rrect(cr, Corners::all(3.0), col, if is_h { 1.2 } else { 0.0 }, fg_c);
                        if is_h {
                            hov_cell = Some((wi, di, val, cr.center(), col));
                        }
                    }
                }

                if let Some((wi, di, val, anchor, col)) = hov_cell {
                    let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
                    let title = format!("Week {} · {}", wi + 1, days[di % 7]);
                    let rows = vec![(col, "Commits".into(), format!("{val}"))];
                    let tip_cl = tip.clone();
                    let bounds = r.expand(12.0);
                    cx.defer(r, Box::new(move |cx, _| tooltip_panel(cx, anchor, bounds, &title, &rows, &tip_cl)));
                }
            })),
        );
        Response { id, rect: ix.rect, hovered: ix.hovered, ..Default::default() }
    }

    /// Distribution histogram with spring-rising bars and vertical percentile markers (`p50`, `p95`, `p99`).
    pub fn histogram(
        &mut self,
        key: impl std::hash::Hash,
        buckets: &[&str],
        counts: &[f32],
        markers: &[HistogramMarker],
        height: f32,
    ) -> Response {
        let id = self.make_id(("histogram", key));
        let ix = self.interact(id, SENSE_HOVER, false);
        let mo = self.motion();
        let enter = self.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);

        let n = counts.len().min(buckets.len());
        let max_c = counts.iter().copied().fold(1.0f32, f32::max);
        let total: f32 = counts.iter().sum::<f32>().max(1.0);

        let bar_c = self.color(Token::Accent);
        let bar_hov = self.color(Token::AccentHover);
        let grid_c = self.color(Token::BorderSubtle);
        let muted_c = self.color(Token::TextMuted);
        let marker_cols: Vec<(Color, Color)> = markers.iter().map(|m| tone_pair(self, m.tone)).collect();
        let mut st_lbl = self.text_style(TextRole::Caption);
        st_lbl.tabular = true;
        let tip = Tip::new(self);

        let b_vec: Vec<String> = buckets.iter().map(|s| (*s).to_owned()).collect();
        let c_vec = counts.to_vec();
        let m_vec = markers.to_vec();
        let ptr = ix.pointer.filter(|_| ix.hovered);

        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(300.0, height.max(150.0)),
            Some(id),
            SENSE_HOVER,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                if n == 0 {
                    return;
                }
                let pad_x = 8.0;
                let top_pad = 24.0;
                let bot_pad = 24.0;
                let plot = Rect::from_xywh(r.min.x + pad_x, r.min.y + top_pad, (r.width() - pad_x * 2.0).max(40.0), (r.height() - top_pad - bot_pad).max(40.0));

                for gi in 0..=3 {
                    let y = plot.min.y + (gi as f32 / 3.0) * plot.height();
                    cx.painter.dashed_hline(plot.min.x, plot.max.x, y, 3.0, 3.0, grid_c);
                }

                let gap = 3.0;
                let bw = ((plot.width() - gap * (n.saturating_sub(1)) as f32) / n as f32).max(3.0);
                let mut hov_idx: Option<(usize, Vec2)> = None;
                // Sparse x-axis labels: a stride wide enough for the widest
                // label, and the last bin only when it clears the one before.
                let widest = b_vec.iter().map(|b| cx.galley(b, &st_lbl).width).fold(0.0f32, f32::max) + 8.0;
                let stride = ((widest / (bw + gap)).ceil() as usize).max(1);
                let mut last_right = f32::NEG_INFINITY;

                for i in 0..n {
                    let bx = plot.min.x + i as f32 * (bw + gap);
                    let bh = (c_vec[i] / max_c) * plot.height() * enter;
                    let br = Rect::from_xywh(bx, plot.max.y - bh, bw, bh);
                    let is_h = ptr.is_some_and(|p| p.x >= bx && p.x <= bx + bw && p.y >= plot.min.y && p.y <= plot.max.y);
                    cx.painter.rrect(br, Corners([4.0, 4.0, 1.0, 1.0]), if is_h { bar_hov } else { bar_c.with_alpha(0.82) }, 0.0, Color::TRANSPARENT);
                    if is_h {
                        hov_idx = Some((i, vec2(bx + bw * 0.5, br.min.y)));
                    }

                    if i % stride == 0 || i + 1 == n {
                        let g = cx.galley(&b_vec[i], &st_lbl);
                        let lx = (bx + (bw - g.width) * 0.5).clamp(r.min.x, (r.max.x - g.width).max(r.min.x));
                        if lx >= last_right + 6.0 {
                            cx.painter.text(cx.text, &g, vec2(lx, plot.max.y + 6.0), muted_c);
                            last_right = lx + g.width;
                        }
                    }
                }

                // Vertical percentile marker lines + pill flags at top. A flag
                // that would touch the previous one drops a row.
                let mut prev_flag: Option<(Rect, u32)> = None;
                for (mi, mk) in m_vec.iter().enumerate() {
                    let mx = plot.min.x + mk.position * plot.width();
                    let (sub_bg, strong_c) = marker_cols[mi];
                    let mut y = plot.min.y;
                    while y < plot.max.y {
                        let seg = 4.0f32.min(plot.max.y - y);
                        cx.painter.rect(Rect::from_xywh(mx - 0.75, y, 1.5, seg), strong_c);
                        y += 7.0;
                    }
                    let gm = cx.galley(&mk.label, &st_lbl);
                    let pw = gm.width + 10.0;
                    let px = (mx - pw * 0.5).clamp(r.min.x + 2.0, (r.max.x - pw - 2.0).max(r.min.x));
                    let row = match prev_flag {
                        Some((p, row)) if px < p.max.x + 4.0 => row ^ 1,
                        _ => 0,
                    };
                    let pr = Rect::from_xywh(px, r.min.y + 2.0 + row as f32 * 21.0, pw, 18.0);
                    prev_flag = Some((pr, row));
                    cx.painter.rrect(pr, Corners::all(9.0), sub_bg, 1.0, strong_c.with_alpha(0.4));
                    cx.painter.text(cx.text, &gm, vec2(pr.min.x + 5.0, pr.min.y + (18.0 - gm.height) * 0.5), strong_c);
                }

                if let Some((bi, anchor)) = hov_idx {
                    let share = c_vec[bi] / total * 100.0;
                    let rows = vec![
                        (bar_c, "Samples".into(), compact(c_vec[bi])),
                        (bar_c, "Share".into(), format!("{share:.1}%")),
                    ];
                    let title = b_vec[bi].clone();
                    let tip_cl = tip.clone();
                    let bounds = r.expand(12.0);
                    cx.defer(r, Box::new(move |cx, _| tooltip_panel(cx, anchor, bounds, &title, &rows, &tip_cl)));
                }
            })),
        );
        Response { id, rect: ix.rect, hovered: ix.hovered, ..Default::default() }
    }

    /// Multi-metric KPI comparison strip inside one divided glass surface with
    /// delta pills and synchronized mini sparklines.
    pub fn kpi_strip(&mut self, key: impl std::hash::Hash, metrics: &[KpiMetric]) -> Response {
        let id = self.make_id(("kpi_strip", key));
        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let div_c = self.color(Token::BorderSubtle);

        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, align: Align::Stretch, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        for (i, m) in metrics.iter().enumerate() {
            let is_first = i == 0;
            self.begin(
                Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(16.0), gap: 8.0, ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                if !is_first {
                    Some(Box::new(move |cx, r| cx.painter.rect(Rect::from_xywh(r.min.x, r.min.y + 12.0, 1.0, (r.height() - 24.0).max(0.0)), div_c)))
                } else {
                    None
                },
            );
            self.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.text_leaf(m.label.clone(), self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, true, Style { width: Size::FILL, ..Default::default() });
            let up = m.delta_pct >= 0.0;
            let sign = if up { "+" } else { "" };
            self.badge(format!("{sign}{:.1}%", m.delta_pct), if up { Tone::Success } else { Tone::Danger });
            self.end();

            let mut st_v = self.text_style(TextRole::Heading2);
            st_v.tabular = true;
            self.text_leaf(m.value.clone(), st_v, self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });

            if m.trend.len() >= 2 {
                let col = self.color(if m.delta_pct >= 0.0 { Token::Success } else { Token::Danger });
                let trend_cl = m.trend.clone();
                self.leaf(
                    Style { width: Size::FILL, ..Default::default() },
                    vec2(80.0, 28.0),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, r| {
                        let min_v = trend_cl.iter().copied().fold(f32::MAX, f32::min);
                        let max_v = trend_cl.iter().copied().fold(f32::MIN, f32::max);
                        let span = (max_v - min_v).max(1e-4);
                        let n = trend_cl.len();
                        let mut upper = Vec::with_capacity(n);
                        let mut lower = Vec::with_capacity(n);
                        for (ti, &v) in trend_cl.iter().enumerate() {
                            let x = r.min.x + (ti as f32 / (n - 1) as f32) * r.width();
                            let y = r.max.y - 2.0 - ((v - min_v) / span) * (r.height() - 4.0);
                            upper.push(vec2(x, y));
                            lower.push(vec2(x, r.max.y));
                        }
                        cx.painter.fill_between(&upper, &lower, col.with_alpha(0.22), col.with_alpha(0.0));
                        cx.painter.polyline(&upper, 1.75, col);
                    })),
                );
            }
            self.end();
        }
        self.end();

        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Multi-day hourly week/day schedule calendar with tinted event cards and a
    /// live current-time needle.
    pub fn day_schedule(
        &mut self,
        key: impl std::hash::Hash,
        days: &[&str],
        start_hour: u8,
        end_hour: u8,
        current_hour: Option<(usize, f32)>,
        events: &[ScheduleEvent],
        height: f32,
    ) -> Response {
        let id = self.make_id(("day_schedule", key));
        let ix = self.interact(id, SENSE_HOVER, false);

        let r_card = self.look().card.radius;
        let top_rad = (r_card - 1.0).max(0.0);
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let hdr_bg = self.color(Token::SurfaceMuted).multiply_alpha(0.6);
        let grid_c = self.color(Token::BorderSubtle).multiply_alpha(0.75);
        let fg_c = self.color(Token::Foreground);
        let muted_c = self.color(Token::TextMuted);
        let needle_c = self.color(Token::Danger);

        let ev_colors: Vec<(Color, Color)> = events.iter().map(|e| tone_pair(self, e.tone)).collect();
        let mut st_hr = self.text_style(TextRole::Caption);
        st_hr.tabular = true;
        let st_day = self.text_style(TextRole::Label);
        let mut st_ev = self.text_style(TextRole::Caption);
        st_ev.weight = 600;
        let st_room = self.text_style(TextRole::Caption);

        let days_vec: Vec<String> = days.iter().map(|s| (*s).to_owned()).collect();
        let ev_vec = events.to_vec();
        let h0 = start_hour.min(22) as f32;
        let h1 = end_hour.max(start_hour + 1).min(24) as f32;

        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(340.0, height.max(200.0)),
            Some(id),
            SENSE_HOVER,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_card), bg_c, 0.0, Color::TRANSPARENT);
                let hdr_h = 30.0;
                let inner_hdr = Rect::from_xywh(r.min.x + 1.0, r.min.y + 1.0, (r.width() - 2.0).max(0.0), hdr_h - 1.0);
                cx.painter.rrect(inner_hdr, Corners([top_rad, top_rad, 0.0, 0.0]), hdr_bg, 0.0, Color::TRANSPARENT);
                cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.min.y + hdr_h, (r.width() - 2.0).max(0.0), 1.0), grid_c);

                let gutter_w = 50.0;
                let n_days = days_vec.len().max(1);
                let col_w = ((r.width() - gutter_w - 6.0) / n_days as f32).max(40.0);
                let body_y0 = r.min.y + hdr_h + 6.0;
                let body_h = (r.max.y - 10.0 - body_y0).max(40.0);
                let span_h = (h1 - h0).max(1.0);

                // Day headers + vertical column dividers.
                for (di, d) in days_vec.iter().enumerate() {
                    let cx0 = r.min.x + gutter_w + di as f32 * col_w;
                    cx.painter.rect(Rect::from_xywh(cx0, r.min.y + 1.0, 1.0, r.height() - 2.0), grid_c);
                    let gd = cx.galley(d, &st_day);
                    let col = if current_hour.is_some_and(|(cd, _)| cd == di) { fg_c } else { muted_c };
                    cx.painter.text(cx.text, &gd, vec2(cx0 + (col_w - gd.width) * 0.5, r.min.y + (hdr_h - gd.height) * 0.5), col);
                }

                // Hour grid lines.
                for hr in (start_hour..=end_hour).step_by(2) {
                    let y = body_y0 + ((hr as f32 - h0) / span_h) * body_h;
                    cx.painter.rect(Rect::from_xywh(r.min.x + gutter_w, y, r.width() - gutter_w - 4.0, 1.0), grid_c.multiply_alpha(0.6));
                    let lbl = format!("{hr:02}:00");
                    let gh = cx.galley(&lbl, &st_hr);
                    cx.painter.text(cx.text, &gh, vec2(r.min.x + gutter_w - 8.0 - gh.width, y - gh.height * 0.5), muted_c);
                }

                // Event cards.
                cx.painter.push_clip(Rect::from_xywh(r.min.x + gutter_w, r.min.y + hdr_h + 1.0, (r.width() - gutter_w - 2.0).max(1.0), (r.height() - hdr_h - 2.0).max(1.0)));
                for (ei, ev) in ev_vec.iter().enumerate() {
                    if ev.day >= n_days {
                        continue;
                    }
                    let x0 = r.min.x + gutter_w + ev.day as f32 * col_w + 4.0;
                    let y0 = body_y0 + ((ev.start_hour - h0) / span_h) * body_h;
                    let eh = ((ev.duration_hours / span_h) * body_h - 2.0).max(20.0);
                    let er = Rect::from_xywh(x0, y0, (col_w - 8.0).max(20.0), eh);
                    let (sub_bg, strong_c) = ev_colors[ei];

                    cx.painter.rrect(er, Corners::all(6.0), sub_bg, 1.0, strong_c.with_alpha(0.35));
                    cx.painter.rrect(Rect::from_xywh(er.min.x + 2.0, er.min.y + 4.0, 3.0, (er.height() - 8.0).max(4.0)), Corners::all(1.5), strong_c, 0.0, Color::TRANSPARENT);

                    let max_tw = (er.width() - 14.0).max(10.0);
                    let gt = cx.galley(&ev.title, &st_ev);
                    if gt.width <= max_tw {
                        cx.painter.text(cx.text, &gt, vec2(er.min.x + 9.0, er.min.y + 4.0), strong_c);
                    } else {
                        let chars: Vec<char> = ev.title.chars().collect();
                        let mut hi = chars.len();
                        while hi > 1 {
                            hi -= 1;
                            let cand = chars[..hi].iter().collect::<String>() + "…";
                            let gc = cx.galley(&cand, &st_ev);
                            if gc.width <= max_tw {
                                cx.painter.text(cx.text, &gc, vec2(er.min.x + 9.0, er.min.y + 4.0), strong_c);
                                break;
                            }
                        }
                    }
                    if eh >= 36.0 {
                        if let Some(ref rm) = ev.room {
                            let gr = cx.galley(rm, &st_room);
                            if gr.width <= max_tw {
                                cx.painter.text(cx.text, &gr, vec2(er.min.x + 9.0, er.min.y + 5.0 + gt.height), strong_c.multiply_alpha(0.8));
                            }
                        }
                    }
                }

                // Current-time red needle!
                if let Some((cur_day, cur_hr)) = current_hour {
                    if cur_day < n_days && cur_hr >= h0 && cur_hr <= h1 {
                        let nx0 = r.min.x + gutter_w + cur_day as f32 * col_w;
                        let ny = body_y0 + ((cur_hr - h0) / span_h) * body_h;
                        cx.painter.rect(Rect::from_xywh(nx0, ny - 0.75, col_w, 1.5), needle_c);
                        cx.painter.circle(vec2(nx0 + 3.0, ny), 3.5, needle_c, 0.0, Color::TRANSPARENT);
                    }
                }
                cx.painter.pop_clip();
                cx.painter.ring(r, Corners::all(r_card), 1.0, bd_c);
            })),
        );
        Response { id, rect: ix.rect, hovered: ix.hovered, ..Default::default() }
    }

    /// Interactive shader / workflow node graph with draggable nodes, typed
    /// ports, and cubic-Bezier wires.
    pub fn node_editor(
        &mut self,
        key: impl std::hash::Hash,
        nodes: &mut [GraphNode],
        wires: &[GraphWire],
        height: f32,
    ) -> Response {
        let id = self.make_id(("node_editor", key));
        let ix = self.interact(id, Sense::DRAG, false);
        let mut changed = false;

        let node_w = 148.0;
        let hdr_h = 26.0;
        let port_h = 20.0;
        let node_size = |nd: &GraphNode| -> Vec2 {
            let rows = nd.inputs.len().max(nd.outputs.len()).max(1);
            vec2(node_w, hdr_h + 8.0 + rows as f32 * port_h)
        };

        // Drag node repositioning in canvas-local coordinates!
        if ix.rect.width() > 20.0 {
            if ix.press_started {
                if let Some(p) = ix.pointer {
                    let local = p - ix.rect.min;
                    let hit = nodes.iter().enumerate().rev().find_map(|(ni, nd)| {
                        let nr = Rect::from_xywh(nd.pos.x, nd.pos.y, node_size(nd).x, node_size(nd).y);
                        nr.contains(local).then_some((ni, local - nd.pos))
                    });
                    if let Some((ni, off)) = hit {
                        let mem = self.mem::<NodeDragMem>(id);
                        mem.active_node = Some(ni);
                        mem.grab_offset = off;
                    }
                }
            }
            let drag_mem = *self.mem::<NodeDragMem>(id);
            if ix.held {
                if let (Some(ni), Some(p)) = (drag_mem.active_node, ix.pointer) {
                    if let Some(nd) = nodes.get_mut(ni) {
                        let sz = node_size(nd);
                        let local = p - ix.rect.min - drag_mem.grab_offset;
                        let nx = local.x.clamp(8.0, (ix.rect.width() - sz.x - 8.0).max(8.0));
                        let ny = local.y.clamp(8.0, (ix.rect.height() - sz.y - 8.0).max(8.0));
                        if (nd.pos - vec2(nx, ny)).length() > 0.1 {
                            nd.pos = vec2(nx, ny);
                            changed = true;
                        }
                    }
                }
            } else if drag_mem.active_node.is_some() {
                self.mem::<NodeDragMem>(id).active_node = None;
            }
        }

        let r_card = self.look().card.radius;
        let canvas_bg = self.color(Token::SurfaceMuted).multiply_alpha(0.45);
        let canvas_bd = self.color(Token::BorderSubtle);
        let dot_c = self.color(Token::BorderStrong).multiply_alpha(0.45);
        let node_bg = self.color(Token::Surface);
        let node_bd = self.color(Token::Border);
        let fg_c = self.color(Token::Foreground);
        let sec_c = self.color(Token::TextSecondary);
        let ink = self.color(Token::ShadowInk);

        let node_tones: Vec<(Color, Color)> = nodes.iter().map(|nd| tone_pair(self, nd.tone)).collect();
        let mut st_title = self.text_style(TextRole::Caption);
        st_title.weight = 600;
        let st_port = self.text_style(TextRole::Caption);

        let nodes_cl = nodes.to_vec();
        let wires_cl = wires.to_vec();
        let active_drag = self.mem::<NodeDragMem>(id).active_node;

        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(360.0, height.max(200.0)),
            Some(id),
            Sense::DRAG,
            if active_drag.is_some() { CursorIcon::Grabbing } else { CursorIcon::Grab },
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_card), canvas_bg, 0.0, Color::TRANSPARENT);
                cx.painter.push_clip(r.expand(-1.0));

                // Subtle 20px dot grid.
                let mut gx = r.min.x + 16.0;
                while gx < r.max.x - 8.0 {
                    let mut gy = r.min.y + 16.0;
                    while gy < r.max.y - 8.0 {
                        cx.painter.rect(Rect::from_xywh(gx, gy, 1.5, 1.5), dot_c);
                        gy += 20.0;
                    }
                    gx += 20.0;
                }

                let port_pos = |ni: usize, port: usize, is_out: bool| -> Vec2 {
                    let nd = &nodes_cl[ni];
                    let x = r.min.x + nd.pos.x + if is_out { node_w } else { 0.0 };
                    let y = r.min.y + nd.pos.y + hdr_h + 14.0 + port as f32 * port_h;
                    vec2(x, y)
                };

                // Draw Bezier wires under nodes.
                for w in &wires_cl {
                    if w.from_node >= nodes_cl.len() || w.to_node >= nodes_cl.len() {
                        continue;
                    }
                    let p0 = port_pos(w.from_node, w.from_port, true);
                    let p3 = port_pos(w.to_node, w.to_port, false);
                    let ctrl = ((p3.x - p0.x).abs() * 0.5).max(36.0);
                    let p1 = p0 + vec2(ctrl, 0.0);
                    let p2 = p3 - vec2(ctrl, 0.0);
                    let mut pts = Vec::with_capacity(25);
                    for s in 0..=24 {
                        let t = s as f32 / 24.0;
                        let u = 1.0 - t;
                        let pt = p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t);
                        pts.push(pt);
                    }
                    let wire_c = node_tones[w.from_node].1;
                    cx.painter.polyline(&pts, 2.2, wire_c.with_alpha(0.85));
                }

                // Draw nodes.
                for (ni, nd) in nodes_cl.iter().enumerate() {
                    let rows = nd.inputs.len().max(nd.outputs.len()).max(1);
                    let nh = hdr_h + 8.0 + rows as f32 * port_h;
                    let nr = Rect::from_xywh(r.min.x + nd.pos.x, r.min.y + nd.pos.y, node_w, nh);
                    let (sub_bg, strong_c) = node_tones[ni];
                    let is_act = active_drag == Some(ni);

                    cx.painter.shadow(
                        nr,
                        Corners::all(10.0),
                        &crate::theme::Shadow { x: 0.0, y: 4.0, blur: 12.0, spread: 0.0, color: ink.with_alpha(0.14) },
                    );
                    cx.painter.rrect(nr, Corners::all(10.0), node_bg, if is_act { 1.5 } else { 1.0 }, if is_act { strong_c } else { node_bd });
                    let hr = Rect::from_xywh(nr.min.x + 1.0, nr.min.y + 1.0, nr.width() - 2.0, hdr_h - 1.0);
                    cx.painter.rrect(hr, Corners([9.0, 9.0, 0.0, 0.0]), sub_bg, 0.0, Color::TRANSPARENT);

                    let gt = cx.galley(&nd.title, &st_title);
                    cx.painter.text(cx.text, &gt, vec2(nr.min.x + 10.0, nr.min.y + (hdr_h - gt.height) * 0.5), strong_c);

                    // Input sockets & labels.
                    for (pi, inp) in nd.inputs.iter().enumerate() {
                        let py = nr.min.y + hdr_h + 14.0 + pi as f32 * port_h;
                        cx.painter.circle(vec2(nr.min.x, py), 4.5, node_bg, 2.0, strong_c);
                        let gp = cx.galley(inp, &st_port);
                        cx.painter.text(cx.text, &gp, vec2(nr.min.x + 9.0, py - gp.height * 0.5), sec_c);
                    }
                    // Output sockets & labels.
                    for (pi, outp) in nd.outputs.iter().enumerate() {
                        let py = nr.min.y + hdr_h + 14.0 + pi as f32 * port_h;
                        cx.painter.circle(vec2(nr.max.x, py), 4.5, strong_c, 1.5, fg_c);
                        let gp = cx.galley(outp, &st_port);
                        cx.painter.text(cx.text, &gp, vec2(nr.max.x - 9.0 - gp.width, py - gp.height * 0.5), fg_c);
                    }
                }
                cx.painter.pop_clip();
                cx.painter.ring(r, Corners::all(r_card), 1.0, canvas_bd);
            })),
        );
        Response { id, rect: ix.rect, hovered: ix.hovered, clicked: ix.clicked, changed, focused: ix.focused }
    }

    /// Git commit branch/merge rail graph with colored lane polylines, short
    /// SHAs, ref badges (`HEAD → ethereal`, tags), and keyboard selection.
    pub fn commit_graph(&mut self, key: impl std::hash::Hash, commits: &[CommitRow], selected: &mut usize) -> Response {
        let id = self.make_id(("commit_graph", key));
        let n = commits.len().max(1);
        *selected = (*selected).min(n - 1);
        let ix = self.interact(id, Sense::CLICK, false);
        let mut changed = false;

        let row_h = 38.0;
        let hdr_h = 30.0;
        let total_h = hdr_h + commits.len() as f32 * row_h + 6.0;

        if ix.clicked && ix.rect.height() > hdr_h {
            if let Some(p) = ix.pointer {
                let idx = (((p.y - ix.rect.min.y - hdr_h) / row_h).floor() as isize).clamp(0, n as isize - 1) as usize;
                if idx != *selected {
                    *selected = idx;
                    changed = true;
                }
            }
        }
        if ix.focused {
            if self.key_pressed_for(id, Key::ArrowDown) {
                *selected = (*selected + 1) % n;
                changed = true;
            } else if self.key_pressed_for(id, Key::ArrowUp) {
                *selected = (*selected + n - 1) % n;
                changed = true;
            }
        }

        let r_card = self.look().card.radius;
        let top_rad = (r_card - 1.0).max(0.0);
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let hdr_bg = self.color(Token::SurfaceMuted).multiply_alpha(0.6);
        let sel_bg = self.color(Token::AccentSubtle).multiply_alpha(0.65);
        let hov_bg = self.color(Token::Hover);
        let badge_bg = self.color(Token::AccentSubtle);
        let badge_fg = self.color(Token::AccentStrong);
        let fg_c = self.color(Token::Foreground);
        let muted_c = self.color(Token::TextMuted);
        let lanes = [
            self.color(Token::Series0),
            self.color(Token::Series2),
            self.color(Token::Series3),
            self.color(Token::Series1),
        ];

        let st_hdr = self.text_style(TextRole::Caption);
        let st_msg = self.text_style(TextRole::BodyStrong);
        let mut st_sha = self.text_style(TextRole::Mono);
        st_sha.size = (st_sha.size - 0.5).max(11.0);
        let mut st_ref = self.text_style(TextRole::Caption);
        st_ref.weight = 600;

        let commits_cl = commits.to_vec();
        let sel_idx = *selected;
        let ptr = ix.pointer.filter(|_| ix.hovered);
        let max_lane = commits.iter().map(|c| c.lane).max().unwrap_or(0);

        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(360.0, total_h),
            Some(id),
            Sense::CLICK,
            CursorIcon::Pointer,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_card), bg_c, 0.0, Color::TRANSPARENT);
                let hr = Rect::from_xywh(r.min.x + 1.0, r.min.y + 1.0, (r.width() - 2.0).max(0.0), hdr_h - 1.0);
                cx.painter.rrect(hr, Corners([top_rad, top_rad, 0.0, 0.0]), hdr_bg, 0.0, Color::TRANSPARENT);
                cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.min.y + hdr_h, (r.width() - 2.0).max(0.0), 1.0), bd_c);

                let rail_x0 = r.min.x + 18.0;
                let lane_step = 16.0;
                let text_x0 = rail_x0 + (max_lane + 1) as f32 * lane_step + 10.0;

                let gh1 = cx.galley("Graph", &st_hdr);
                cx.painter.text(cx.text, &gh1, vec2(r.min.x + 14.0, r.min.y + (hdr_h - gh1.height) * 0.5), muted_c);
                let gh2 = cx.galley("Commit", &st_hdr);
                cx.painter.text(cx.text, &gh2, vec2(text_x0, r.min.y + (hdr_h - gh2.height) * 0.5), muted_c);

                // Row highlights first.
                for ci in 0..commits_cl.len() {
                    let ry = r.min.y + hdr_h + 2.0 + ci as f32 * row_h;
                    let rr = Rect::from_xywh(r.min.x + 4.0, ry, (r.width() - 8.0).max(0.0), row_h - 2.0);
                    if ci == sel_idx {
                        cx.painter.rrect(rr, Corners::all(7.0), sel_bg, 0.0, Color::TRANSPARENT);
                    } else if ptr.is_some_and(|p| rr.contains(p)) {
                        cx.painter.rrect(rr, Corners::all(7.0), hov_bg, 0.0, Color::TRANSPARENT);
                    }
                }

                // Draw rail curves from row `ci` down to the next commit row on each parent lane!
                for (ci, cm) in commits_cl.iter().enumerate() {
                    let y0 = r.min.y + hdr_h + 2.0 + ci as f32 * row_h + row_h * 0.5;
                    let x0 = rail_x0 + cm.lane as f32 * lane_step;
                    for &pl in &cm.parents {
                        if let Some(target_row) = ((ci + 1)..commits_cl.len()).find(|&rj| commits_cl[rj].lane == pl) {
                            let y1 = r.min.y + hdr_h + 2.0 + target_row as f32 * row_h + row_h * 0.5;
                            let x1 = rail_x0 + pl as f32 * lane_step;
                            let col = lanes[cm.lane.max(pl) % lanes.len()];
                            if (x1 - x0).abs() < 0.5 {
                                cx.painter.polyline(&[vec2(x0, y0), vec2(x1, y1)], 2.0, col);
                            } else {
                                let bend_y = y0 + row_h;
                                let mut pts = Vec::with_capacity(14);
                                for s in 0..=12 {
                                    let t = s as f32 / 12.0;
                                    let st = t * t * (3.0 - 2.0 * t);
                                    pts.push(vec2(x0 + (x1 - x0) * st, y0 + (bend_y.min(y1) - y0) * t));
                                }
                                if y1 > bend_y + 0.5 {
                                    pts.push(vec2(x1, y1));
                                }
                                cx.painter.polyline(&pts, 2.0, col);
                            }
                        }
                    }
                }

                // Commit nodes + SHA + ref badge + message + author/time.
                for (ci, cm) in commits_cl.iter().enumerate() {
                    let cy = r.min.y + hdr_h + 2.0 + ci as f32 * row_h + row_h * 0.5;
                    let cx_pos = rail_x0 + cm.lane as f32 * lane_step;
                    let col = lanes[cm.lane % lanes.len()];
                    cx.painter.circle(vec2(cx_pos, cy), 4.5, col, 2.0, bg_c);

                    let mut cur_x = text_x0;
                    let g_sha = cx.galley(&cm.sha, &st_sha);
                    cx.painter.text(cx.text, &g_sha, vec2(cur_x, cy - g_sha.height * 0.5), muted_c);
                    cur_x += g_sha.width + 10.0;

                    if let Some(ref b) = cm.ref_badge {
                        let gb = cx.galley(b, &st_ref);
                        let pr = Rect::from_xywh(cur_x, cy - 10.0, gb.width + 12.0, 20.0);
                        cx.painter.rrect(pr, Corners::all(10.0), badge_bg, 1.0, badge_fg.with_alpha(0.3));
                        cx.painter.text(cx.text, &gb, vec2(pr.min.x + 6.0, cy - gb.height * 0.5), badge_fg);
                        cur_x = pr.max.x + 8.0;
                    }

                    let meta = format!("{} · {}", cm.author, cm.time);
                    let gm_r = cx.galley(&meta, &st_hdr);
                    let right_x = r.max.x - 14.0 - gm_r.width;
                    cx.painter.text(cx.text, &gm_r, vec2(right_x, cy - gm_r.height * 0.5), muted_c);

                    // Long messages end in an ellipsis; with no room left after
                    // the ref badges, the message is left out.
                    let avail_msg = right_x - 10.0 - cur_x;
                    if avail_msg >= 24.0 {
                        let g_msg = cx.galley_fit(&cm.message, &st_msg, avail_msg);
                        cx.painter.text(cx.text, &g_msg, vec2(cur_x, cy - g_msg.height * 0.5), fg_c);
                    }
                }
                cx.painter.ring(r, Corners::all(r_card), 1.0, bd_c);
            })),
        );
        Response { id, rect: ix.rect, hovered: ix.hovered, clicked: ix.clicked, changed, focused: ix.focused }
    }

    /// Interactive test suite explorer with pass/fail summary, `"Failed"` filter
    /// toggle, expandable assertion output, and a `"Re-run"` action button.
    pub fn test_explorer(&mut self, key: impl std::hash::Hash, tests: &[TestCase]) -> bool {
        let id = self.make_id(("test_explorer", key));
        let passed = tests.iter().filter(|t| t.state == TestState::Passed).count();
        let failed = tests.iter().filter(|t| t.state == TestState::Failed).count();
        let skipped = tests.iter().filter(|t| t.state == TestState::Skipped).count();

        let r_card = self.look().card.radius;
        let top_rad = (r_card - 1.0).max(0.0);
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let hdr_bg = self.color(Token::SurfaceMuted).multiply_alpha(0.6);
        let mut rerun_clicked = false;

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        // Header bar.
        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(12.0, 8.0), gap: 8.0, align: Align::Center, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                let inner = Rect::from_xywh(r.min.x + 1.0, r.min.y + 1.0, (r.width() - 2.0).max(0.0), (r.height() - 1.0).max(0.0));
                cx.painter.rrect(inner, Corners([top_rad, top_rad, 0.0, 0.0]), hdr_bg, 0.0, Color::TRANSPARENT);
                cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.max.y - 1.0, (r.width() - 2.0).max(0.0), 1.0), bd_c);
            })),
        );
        self.badge(format!("{passed} passed"), Tone::Success);
        if failed > 0 {
            self.badge(format!("{failed} failed"), Tone::Danger);
        }
        if skipped > 0 {
            self.badge(format!("{skipped} skipped"), Tone::Neutral);
        }
        self.begin(Style { width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.end();

        let mut failed_only = self.mem::<TestMem>(id).failed_only;
        self.scope("flt_fail", |ui| {
            ui.chip("Failed only", &mut failed_only);
        });
        self.mem::<TestMem>(id).failed_only = failed_only;

        if self.scope("rerun_btn", |ui| ui.button("Re-run").secondary().small().icon("play").clicked()) {
            rerun_clicked = true;
        }
        self.end();

        // Rows.
        let mut expanded = self.mem::<TestMem>(id).expanded_idx;
        if expanded.is_none() {
            // Auto-expand first failing test initially!
            expanded = tests.iter().position(|t| t.state == TestState::Failed);
        }

        self.begin(Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(6.0), gap: 2.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for (ti, tc) in tests.iter().enumerate() {
            if failed_only && tc.state != TestState::Failed {
                continue;
            }
            let rid = id.with(("tc", ti));
            let r_ix = self.interact(rid, Sense::CLICK, false);
            let hov = self.fade(rid, 0, r_ix.hovered, self.motion().hover_in, self.motion().hover_out, Ease::Out);
            let r_bg = self.color(Token::Hover).multiply_alpha(hov);

            self.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(10.0, 6.0), gap: 8.0, align: Align::Center, ..Default::default() },
                Some(rid),
                Sense::CLICK,
                if tc.failure_msg.is_some() { CursorIcon::Pointer } else { CursorIcon::Default },
                false,
                Some(Box::new(move |cx, r| {
                    if hov > 0.005 {
                        cx.painter.rrect(r, Corners::all(6.0), r_bg, 0.0, Color::TRANSPARENT);
                    }
                })),
            );
            let (ic, ic_c) = match tc.state {
                TestState::Passed => ("circle-check", self.color(Token::SuccessStrong)),
                TestState::Failed => ("circle-x", self.color(Token::DangerStrong)),
                TestState::Skipped => ("circle-minus", self.color(Token::TextMuted)),
            };
            self.icon_leaf(ic, 15.0, ic_c);
            self.text_leaf(tc.suite.clone(), self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, false, Style::default());
            self.text_leaf("::", self.text_style(TextRole::Mono), self.color(Token::TextMuted), false, false, Style::default());
            self.text_leaf(tc.name.clone(), self.text_style(TextRole::Mono), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
            if tc.state != TestState::Skipped {
                let mut st_d = self.text_style(TextRole::Mono);
                st_d.tabular = true;
                st_d.size = (st_d.size - 1.0).max(11.0);
                self.text_leaf(format!("{:.2} ms", tc.duration_ms), st_d, self.color(Token::TextMuted), false, false, Style::default());
            }
            self.end();

            if r_ix.clicked && tc.failure_msg.is_some() {
                expanded = if expanded == Some(ti) { None } else { Some(ti) };
            }

            if expanded == Some(ti) {
                if let Some(ref err) = tc.failure_msg {
                    let e_bg = self.color(Token::DangerSubtle).multiply_alpha(0.5);
                    let e_bd = self.color(Token::Danger).multiply_alpha(0.35);
                    let e_fg = self.color(Token::DangerStrong);
                    self.begin(
                        Style { width: Size::FILL, pad: Sides::xy(12.0, 8.0), ..Default::default() },
                        None,
                        Sense::NONE,
                        CursorIcon::Default,
                        false,
                        Some(Box::new(move |cx, r| cx.painter.rrect(r.expand2(-6.0, 0.0), Corners::all(7.0), e_bg, 1.0, e_bd))),
                    );
                    let mut mono = self.text_style(TextRole::Mono);
                    mono.size = (mono.size - 0.5).max(11.0);
                    self.text_leaf(err.clone(), mono, e_fg, true, false, Style { width: Size::FILL, pad: Sides::xy(6.0, 0.0), ..Default::default() });
                    self.end();
                }
            }
        }
        self.end();
        self.end();

        self.mem::<TestMem>(id).expanded_idx = expanded;
        rerun_clicked
    }
}
