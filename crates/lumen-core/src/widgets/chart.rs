//! Charts: line, area, bar (grouped or stacked), donut, gauge, sparkline.
//!
//! Values animate: a chart grows in from its baseline the first time it is
//! shown, and later data changes spring to their new values. Hovering
//! shows a crosshair or highlight with a tooltip.

use std::f32::consts::{PI, TAU};

use lumen_text::TextStyle;

use crate::anim::Ease;
use crate::color::Color;
use crate::input::CursorIcon;
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Vec2};
use crate::theme::{Elevation, TextRole, Token};
use crate::ui::{PaintCx, Response, Sense, Ui};

use super::show_on_drop;

/// A named run of values.
#[derive(Clone, Debug)]
pub struct Series {
    pub name: String,
    pub values: Vec<f32>,
    /// Override the palette color.
    pub color: Option<Color>,
}

impl Series {
    pub fn new(name: impl Into<String>, values: impl Into<Vec<f32>>) -> Series {
        Series { name: name.into(), values: values.into(), color: None }
    }
    pub fn color(mut self, c: Color) -> Series {
        self.color = Some(c);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChartKind {
    Line,
    Area,
    Bar,
    StackedBar,
}

/// Compact number formatting for axes: `1.2k`, `3.4M`, `0.25`.
pub fn compact(v: f32) -> String {
    let a = v.abs();
    let (x, suffix) = if a >= 1e9 {
        (v / 1e9, "B")
    } else if a >= 1e6 {
        (v / 1e6, "M")
    } else if a >= 1e3 {
        (v / 1e3, "k")
    } else {
        (v, "")
    };
    let s = if x.fract().abs() < 1e-3 {
        format!("{x:.0}")
    } else if x.abs() >= 10.0 {
        format!("{x:.1}")
    } else {
        format!("{x:.2}").trim_end_matches('0').trim_end_matches('.').to_string()
    };
    format!("{s}{suffix}")
}

/// "Nice" axis ticks covering `lo..hi` with about `n` steps.
pub fn nice_ticks(lo: f32, hi: f32, n: usize) -> Vec<f32> {
    let span = (hi - lo).max(1e-6);
    let raw = span / n.max(1) as f32;
    let mag = 10f32.powf(raw.log10().floor());
    let step = [1.0, 2.0, 2.5, 5.0, 10.0].iter().map(|m| m * mag).find(|s| *s >= raw).unwrap_or(10.0 * mag);
    let start = (lo / step).floor() * step;
    let mut v = vec![];
    let mut t = start;
    while v.len() < 20 {
        v.push(if t.abs() < step * 1e-4 { 0.0 } else { t });
        if t >= hi - step * 1e-4 {
            break;
        }
        t += step;
    }
    v
}

/// Monotone cubic interpolation (no overshoot) sampled `k` times per gap.
fn monotone(pts: &[Vec2], k: usize) -> Vec<Vec2> {
    let n = pts.len();
    if n < 3 {
        return pts.to_vec();
    }
    let d: Vec<f32> = (0..n - 1).map(|i| (pts[i + 1].y - pts[i].y) / (pts[i + 1].x - pts[i].x).max(1e-6)).collect();
    let mut m = vec![0.0; n];
    m[0] = d[0];
    m[n - 1] = d[n - 2];
    for i in 1..n - 1 {
        m[i] = if d[i - 1] * d[i] <= 0.0 { 0.0 } else { (d[i - 1] + d[i]) / 2.0 };
    }
    for i in 0..n - 1 {
        if d[i] == 0.0 {
            m[i] = 0.0;
            m[i + 1] = 0.0;
        } else {
            let (a, b) = (m[i] / d[i], m[i + 1] / d[i]);
            let s = a * a + b * b;
            if s > 9.0 {
                let t = 3.0 / s.sqrt();
                m[i] = t * a * d[i];
                m[i + 1] = t * b * d[i];
            }
        }
    }
    let mut out = Vec::with_capacity((n - 1) * k + 1);
    for i in 0..n - 1 {
        let (p0, p1) = (pts[i], pts[i + 1]);
        let h = p1.x - p0.x;
        for j in 0..k {
            let t = j as f32 / k as f32;
            let (t2, t3) = (t * t, t * t * t);
            let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
            let h10 = t3 - 2.0 * t2 + t;
            let h01 = -2.0 * t3 + 3.0 * t2;
            let h11 = t3 - t2;
            out.push(vec2(p0.x + h * t, h00 * p0.y + h10 * h * m[i] + h01 * p1.y + h11 * h * m[i + 1]));
        }
    }
    out.push(pts[n - 1]);
    out
}

fn series_color(ui: &Ui, i: usize, n: usize, own: Option<Color>) -> Color {
    if let Some(c) = own {
        return c;
    }
    if n == 1 {
        return ui.color(Token::Accent);
    }
    const S: [Token; 6] = [Token::Series0, Token::Series1, Token::Series2, Token::Series3, Token::Series4, Token::Series5];
    ui.color(S[i % 6])
}

/// Draw a tooltip panel at `anchor` (kept on screen) with a title and rows.
fn tooltip_panel(cx: &mut PaintCx<'_>, anchor: Vec2, bounds: Rect, title: &str, rows: &[(Color, String, String)], st: &Tip) {
    let tg = cx.galley(title, &st.title);
    let mut w = tg.width;
    let rows_g: Vec<_> = rows.iter().map(|(c, n, v)| (*c, cx.galley(n, &st.body), cx.galley(v, &st.value))).collect();
    for (_, n, v) in &rows_g {
        w = w.max(14.0 + n.width + 16.0 + v.width);
    }
    let line = st.body.line_height;
    let h = 10.0 + tg.height + 4.0 + rows_g.len() as f32 * line + 8.0;
    let w = w + 20.0;
    let mut x = anchor.x + 14.0;
    if x + w > bounds.max.x {
        x = anchor.x - 14.0 - w;
    }
    let y = (anchor.y - h / 2.0).clamp(bounds.min.y, (bounds.max.y - h).max(bounds.min.y));
    let r = Rect::from_xywh(x.round(), y.round(), w, h);
    cx.painter.shadows(r, Corners::all(8.0), &st.shadows);
    cx.painter.rrect(r, Corners::all(8.0), st.fill, 1.0, st.border);
    cx.painter.text(cx.text, &tg, vec2(r.min.x + 10.0, r.min.y + 8.0), st.muted);
    let mut yy = r.min.y + 8.0 + tg.height + 4.0;
    for (c, n, v) in rows_g {
        cx.painter.rrect(Rect::from_xywh(r.min.x + 10.0, yy + line / 2.0 - 4.0, 8.0, 8.0), Corners::all(2.0), c, 0.0, Color::TRANSPARENT);
        cx.painter.text(cx.text, &n, vec2(r.min.x + 24.0, yy), st.fg);
        cx.painter.text(cx.text, &v, vec2(r.max.x - 10.0 - v.width, yy), st.fg);
        yy += line;
    }
}

#[derive(Clone)]
struct Tip {
    title: TextStyle,
    body: TextStyle,
    value: TextStyle,
    fill: Color,
    border: Color,
    fg: Color,
    muted: Color,
    shadows: Vec<crate::theme::Shadow>,
}

impl Tip {
    fn new(ui: &Ui) -> Tip {
        let mut value = ui.text_style(TextRole::Label);
        value.tabular = true;
        Tip {
            title: ui.text_style(TextRole::Caption),
            body: ui.text_style(TextRole::Label),
            value,
            fill: ui.color(Token::SurfaceRaised),
            border: ui.color(Token::Border),
            fg: ui.color(Token::Foreground),
            muted: ui.color(Token::TextMuted),
            shadows: Elevation::Level2.shadows(ui.theme.darkness(), ui.color(Token::ShadowInk)),
        }
    }
}

fn legend(ui: &mut Ui, items: &[(Color, String)]) {
    let st = ui.text_style(TextRole::Label);
    let c = ui.color(Token::TextSecondary);
    ui.begin(Style { dir: Dir::Row, gap: 16.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
    for (col, name) in items {
        let col = *col;
        ui.begin(Style { dir: Dir::Row, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        ui.leaf(Style::default(), vec2(10.0, 10.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(3.0), col, 0.0, Color::TRANSPARENT))));
        ui.text_leaf(name.clone(), st, c, false, false, Style::default());
        ui.end();
    }
    ui.end();
}

const SENSE_HOVER: Sense = Sense { click: true, drag: false, focusable: false, scroll: false };

// ------------------------------------------------------------ XY charts

/// Builder for line, area and bar charts.
pub struct Chart<'u> {
    ui: &'u mut Ui,
    key: String,
    kind: ChartKind,
    labels: Vec<String>,
    series: Vec<Series>,
    height: f32,
    smooth: bool,
    legend: bool,
    points: bool,
    format: fn(f32) -> String,
    shown: bool,
}

impl Chart<'_> {
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    /// Curved lines (monotone, never overshooting the data).
    pub fn smooth(mut self, s: bool) -> Self {
        self.smooth = s;
        self
    }
    /// Fill under lines.
    pub fn area(mut self) -> Self {
        if self.kind == ChartKind::Line {
            self.kind = ChartKind::Area;
        }
        self
    }
    pub fn stacked(mut self) -> Self {
        if self.kind == ChartKind::Bar {
            self.kind = ChartKind::StackedBar;
        }
        self
    }
    pub fn legend(mut self, on: bool) -> Self {
        self.legend = on;
        self
    }
    /// Dots on every data point.
    pub fn points(mut self, on: bool) -> Self {
        self.points = on;
        self
    }
    /// Format values for the axis and tooltip.
    pub fn format(mut self, f: fn(f32) -> String) -> Self {
        self.format = f;
        self
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("chart", &self.key));
        let ix = ui.interact(id, SENSE_HOVER, false);
        let mo = ui.motion();
        let n = self.labels.len().max(self.series.iter().map(|s| s.values.len()).max().unwrap_or(0));
        let ns = self.series.len();
        let kind = self.kind;
        let stacked = kind == ChartKind::StackedBar;

        // Animated values: each point springs from the baseline.
        let mut vals: Vec<Vec<f32>> = Vec::with_capacity(ns);
        for (si, s) in self.series.iter().enumerate() {
            let mut v = Vec::with_capacity(n);
            for i in 0..n.min(2000) {
                let target = s.values.get(i).copied().unwrap_or(0.0);
                v.push(ui.anims.spring_from(id, (si * 2000 + i) as u32, 0.0, target, mo.smooth));
            }
            vals.push(v);
        }
        // Range from the targets (not the animated values) so axes don't jump.
        let (mut lo, mut hi) = (0.0f32, 0.0f32);
        if stacked {
            for i in 0..n {
                let (mut p, mut q) = (0.0, 0.0);
                for s in &self.series {
                    let v = s.values.get(i).copied().unwrap_or(0.0);
                    if v >= 0.0 {
                        p += v
                    } else {
                        q += v
                    }
                }
                hi = hi.max(p);
                lo = lo.min(q);
            }
        } else {
            for s in &self.series {
                for &v in &s.values {
                    hi = hi.max(v);
                    lo = lo.min(v);
                }
            }
        }
        if hi == lo {
            hi = lo + 1.0;
        }
        let ticks = nice_ticks(lo, hi, (self.height / 48.0).round().max(2.0) as usize);
        let (y0, y1) = (*ticks.first().unwrap_or(&lo), *ticks.last().unwrap_or(&hi));
        let fmt = self.format;
        let tick_labels: Vec<String> = ticks.iter().map(|t| fmt(*t)).collect();
        let cap = ui.text_style(TextRole::Caption);
        let axis_w = tick_labels.iter().map(|t| ui.galley(t, &cap).width).fold(0.0, f32::max).ceil() + 10.0;
        let colors: Vec<Color> = self.series.iter().enumerate().map(|(i, s)| series_color(ui, i, ns, s.color)).collect();

        // Hovered index from last frame's rect.
        let plot_of = move |r: Rect| Rect { min: vec2(r.min.x + axis_w, r.min.y + 6.0), max: vec2(r.max.x - 4.0, r.max.y - 22.0) };
        let x_of = move |plot: Rect, i: usize| -> f32 {
            if matches!(kind, ChartKind::Bar | ChartKind::StackedBar) {
                plot.min.x + plot.width() * (i as f32 + 0.5) / n.max(1) as f32
            } else if n <= 1 {
                plot.center().x
            } else {
                plot.min.x + plot.width() * i as f32 / (n - 1) as f32
            }
        };
        let hover_idx = ix.pointer.filter(|_| ix.hovered && n > 0).map(|p| {
            let plot = plot_of(ix.rect);
            let mut best = 0;
            let mut bd = f32::MAX;
            for i in 0..n {
                let d = (x_of(plot, i) - p.x).abs();
                if d < bd {
                    bd = d;
                    best = i;
                }
            }
            best
        });
        let hov_a = ui.fade(id, 90_000, hover_idx.is_some(), mo.hover_in, mo.hover_out, Ease::Linear);
        let last_idx = {
            #[derive(Default)]
            struct Last(usize);
            let l = ui.mem::<Last>(id);
            if let Some(h) = hover_idx {
                l.0 = h;
            }
            l.0
        };
        let cross_x = ui.spring(id, 90_001, last_idx as f32, mo.snappy);
        let bar_focus: Vec<f32> = (0..n).map(|i| ui.fade(id.with(("bar", i)), 0, hover_idx == Some(i), mo.hover_in, mo.hover_out, Ease::Linear)).collect();

        let grid = ui.color(Token::BorderSubtle);
        let axis = ui.color(Token::Border);
        let muted = ui.color(Token::TextMuted);
        let surface = ui.color(Token::Surface);
        let tip = Tip::new(ui);
        let labels = self.labels.clone();
        let names: Vec<String> = self.series.iter().map(|s| s.name.clone()).collect();
        let targets: Vec<Vec<f32>> = self.series.iter().map(|s| s.values.clone()).collect();
        let (smooth, points) = (self.smooth, self.points);
        let radius = ui.look().control.radius[0].min(4.0);

        ui.begin(Style { width: Size::FILL, gap: 12.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if self.legend && ns > 0 {
            let items: Vec<(Color, String)> = colors.iter().copied().zip(names.iter().cloned()).collect();
            legend(ui, &items);
        }
        ui.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(200.0, self.height),
            Some(id),
            SENSE_HOVER,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let plot = plot_of(r);
                let y_of = |v: f32| plot.max.y - (v - y0) / (y1 - y0).max(1e-6) * plot.height();
                // Grid and y labels.
                for (t, lbl) in ticks.iter().zip(&tick_labels) {
                    let y = y_of(*t);
                    if *t == 0.0 || *t == y0 {
                        cx.painter.rect(Rect::from_xywh(plot.min.x, cx.painter.snap(y), plot.width(), 1.0), axis);
                    } else {
                        cx.painter.dashed_hline(plot.min.x, plot.max.x, y, 4.0, 4.0, grid);
                    }
                    let g = cx.galley(lbl, &cap);
                    cx.painter.text(cx.text, &g, vec2(plot.min.x - 8.0 - g.width, (y - g.height / 2.0).round()), muted);
                }
                // X labels, thinned to fit.
                if n > 0 && !labels.is_empty() {
                    let widest = labels.iter().map(|l| cx.galley(l, &cap).width).fold(0.0, f32::max) + 12.0;
                    let every = ((widest * n as f32 / plot.width().max(1.0)).ceil() as usize).max(1);
                    for (i, l) in labels.iter().enumerate().take(n) {
                        if i % every != 0 {
                            continue;
                        }
                        let g = cx.galley(l, &cap);
                        let x = x_of(plot, i);
                        let gx = (x - g.width / 2.0).clamp(plot.min.x - axis_w + 2.0, r.max.x - g.width);
                        cx.painter.text(cx.text, &g, vec2(gx.round(), plot.max.y + 6.0), muted);
                    }
                }
                cx.painter.push_clip(r);
                match kind {
                    ChartKind::Line | ChartKind::Area => {
                        let base = y_of(0.0f32.clamp(y0, y1));
                        for (si, v) in vals.iter().enumerate().rev() {
                            let pts: Vec<Vec2> = v.iter().enumerate().map(|(i, &y)| vec2(x_of(plot, i), y_of(y))).collect();
                            let line = if smooth { monotone(&pts, 10) } else { pts.clone() };
                            let col = colors[si];
                            if kind == ChartKind::Area {
                                let lower: Vec<Vec2> = line.iter().map(|p| vec2(p.x, base)).collect();
                                let top_a = if vals.len() > 1 { 0.18 } else { 0.28 };
                                cx.painter.fill_between(&line, &lower, col.multiply_alpha(top_a), col.multiply_alpha(0.0));
                            }
                            cx.painter.polyline(&line, 2.0, col);
                            if points {
                                for p in &pts {
                                    cx.painter.circle(*p, 3.5, surface, 2.0, col);
                                }
                            }
                        }
                        if hov_a > 0.0 && n > 0 {
                            // Crosshair glides between points.
                            let i0 = cross_x.floor().clamp(0.0, (n - 1) as f32) as usize;
                            let i1 = (i0 + 1).min(n - 1);
                            let f = cross_x - i0 as f32;
                            let x = x_of(plot, i0) + (x_of(plot, i1) - x_of(plot, i0)) * f;
                            cx.painter.rect(Rect::from_xywh(cx.painter.snap(x), plot.min.y, 1.0, plot.height()), axis.multiply_alpha(hov_a));
                            for (si, v) in vals.iter().enumerate() {
                                let y = v.get(i0).copied().unwrap_or(0.0) + (v.get(i1).copied().unwrap_or(0.0) - v.get(i0).copied().unwrap_or(0.0)) * f;
                                let c = vec2(x, y_of(y));
                                cx.painter.circle(c, 5.0 * hov_a.max(0.6), surface.multiply_alpha(hov_a), 2.0, colors[si].multiply_alpha(hov_a));
                            }
                        }
                    }
                    ChartKind::Bar | ChartKind::StackedBar => {
                        let slot = plot.width() / n.max(1) as f32;
                        let group_w = (slot * 0.72).min(18.0 * ns.max(1) as f32 + 6.0 * (ns.max(1) - 1) as f32);
                        let base = y_of(0.0f32.clamp(y0, y1));
                        #[allow(clippy::needless_range_loop)]
                        for i in 0..n {
                            let cxm = x_of(plot, i);
                            let dim = if hov_a > 0.0 { 1.0 - 0.45 * hov_a * (1.0 - bar_focus[i]) } else { 1.0 };
                            if stacked {
                                let w = group_w.clamp(4.0, 28.0);
                                let (mut pos, mut neg) = (0.0f32, 0.0f32);
                                let top_si = (0..vals.len()).rev().find(|&s| vals[s].get(i).copied().unwrap_or(0.0) > 0.0);
                                for (si, v) in vals.iter().enumerate() {
                                    let val = v.get(i).copied().unwrap_or(0.0);
                                    let (a, b) = if val >= 0.0 {
                                        let a = pos;
                                        pos += val;
                                        (a, pos)
                                    } else {
                                        let a = neg;
                                        neg += val;
                                        (neg, a)
                                    };
                                    let (ya, yb) = (y_of(b), y_of(a));
                                    let br = Rect { min: vec2(cxm - w / 2.0, ya.min(yb)), max: vec2(cxm + w / 2.0, ya.max(yb)) };
                                    let rr = if Some(si) == top_si { Corners([radius, radius, 0.0, 0.0]) } else { Corners::all(0.0) };
                                    // 1px gap between stacked segments.
                                    let br = Rect { min: br.min, max: vec2(br.max.x, (br.max.y - 1.0).max(br.min.y)) };
                                    cx.painter.rrect_unsnapped(br, rr, [colors[si].multiply_alpha(dim); 2], 0.0, Color::TRANSPARENT);
                                }
                            } else {
                                let k = vals.len().max(1) as f32;
                                let gap = if k > 1.0 { 3.0 } else { 0.0 };
                                let bw = ((group_w - gap * (k - 1.0)) / k).max(2.0);
                                for (si, v) in vals.iter().enumerate() {
                                    let val = v.get(i).copied().unwrap_or(0.0);
                                    let x = cxm - group_w / 2.0 + si as f32 * (bw + gap);
                                    let yv = y_of(val);
                                    let br = Rect { min: vec2(x, yv.min(base)), max: vec2(x + bw, yv.max(base)) };
                                    let rr = if val >= 0.0 { Corners([radius, radius, 0.0, 0.0]) } else { Corners([0.0, 0.0, radius, radius]) };
                                    cx.painter.rrect_unsnapped(br, rr, [colors[si].multiply_alpha(dim); 2], 0.0, Color::TRANSPARENT);
                                }
                            }
                        }
                    }
                }
                cx.painter.pop_clip();
                if hov_a > 0.5 {
                    if let Some(i) = hover_idx {
                        let title = labels.get(i).cloned().unwrap_or_else(|| format!("#{}", i + 1));
                        let rows: Vec<(Color, String, String)> = names
                            .iter()
                            .enumerate()
                            .map(|(si, nm)| (colors[si], nm.clone(), fmt(targets[si].get(i).copied().unwrap_or(0.0))))
                            .collect();
                        let ax = x_of(plot, i);
                        let ay = plot.center().y;
                        let tip = tip.clone();
                        let bounds = r;
                        cx.defer(r, Box::new(move |cx, _| tooltip_panel(cx, vec2(ax, ay), bounds.expand(8.0), &title, &rows, &tip)));
                    }
                }
            })),
        );
        ui.end();
        Response { id, rect: ix.rect, hovered: ix.hovered, ..Default::default() }
    }
}

show_on_drop!(Chart);

impl Ui {
    /// A line chart. `labels` name the x positions; chain `.area()`,
    /// `.smooth(true)`, `.points(true)`, `.height(..)`.
    pub fn line_chart<'u>(&'u mut self, key: &str, labels: &[&str], series: Vec<Series>) -> Chart<'u> {
        self.chart(key, ChartKind::Line, labels, series)
    }
    /// A filled area chart.
    pub fn area_chart<'u>(&'u mut self, key: &str, labels: &[&str], series: Vec<Series>) -> Chart<'u> {
        self.chart(key, ChartKind::Area, labels, series)
    }
    /// A grouped bar chart; chain `.stacked()` to stack series.
    pub fn bar_chart<'u>(&'u mut self, key: &str, labels: &[&str], series: Vec<Series>) -> Chart<'u> {
        self.chart(key, ChartKind::Bar, labels, series)
    }

    pub fn chart<'u>(&'u mut self, key: &str, kind: ChartKind, labels: &[&str], series: Vec<Series>) -> Chart<'u> {
        let legend = series.len() > 1;
        Chart {
            ui: self,
            key: key.to_owned(),
            kind,
            labels: labels.iter().map(|s| s.to_string()).collect(),
            series,
            height: 220.0,
            smooth: false,
            legend,
            points: false,
            format: compact,
            shown: false,
        }
    }

    /// A tiny inline trend line (no axes).
    pub fn sparkline(&mut self, key: &str, values: &[f32], tone: Option<Color>) -> Response {
        self.sparkline_sized(key, values, tone, None)
    }

    /// A sparkline with an explicit width (`None` = 96 px, `Some(f32::INFINITY)` = fill).
    pub fn sparkline_sized(&mut self, key: &str, values: &[f32], tone: Option<Color>, width: Option<f32>) -> Response {
        let id = self.make_id(("sparkline", key));
        let mo = self.motion();
        let vals: Vec<f32> = values.iter().enumerate().map(|(i, &v)| self.anims.spring_from(id, i as u32, v, v, mo.smooth)).collect();
        let col = tone.unwrap_or_else(|| self.color(Token::Accent));
        let (lo, hi) = values.iter().fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
        let span = (hi - lo).max(1e-6);
        let grow = self.anims.spring_from(id, 10_000, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);
        let w = width.unwrap_or(96.0);
        let st = if w.is_finite() { Style::default() } else { Style { width: Size::FILL, ..Default::default() } };
        self.leaf(
            st,
            vec2(if w.is_finite() { w } else { 96.0 }, 32.0),
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let r = r.expand(-2.0);
                let n = vals.len();
                if n < 2 {
                    return;
                }
                let pts: Vec<Vec2> = vals
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| vec2(r.min.x + r.width() * i as f32 / (n - 1) as f32, r.max.y - ((v - lo) / span) * r.height() * grow))
                    .collect();
                let line = monotone(&pts, 6);
                let lower: Vec<Vec2> = line.iter().map(|p| vec2(p.x, r.max.y + 2.0)).collect();
                cx.painter.fill_between(&line, &lower, col.multiply_alpha(0.22), col.multiply_alpha(0.0));
                cx.painter.polyline(&line, 1.5, col);
                cx.painter.circle(*pts.last().unwrap(), 2.5, col, 0.0, Color::TRANSPARENT);
            })),
        );
        Response { id, ..Default::default() }
    }

    /// A donut chart of named parts, with a legend.
    pub fn donut(&mut self, key: &str, parts: &[(&str, f32)]) -> Response {
        self.donut_sized(key, parts, 176.0)
    }

    pub fn donut_sized(&mut self, key: &str, parts: &[(&str, f32)], size: f32) -> Response {
        let id = self.make_id(("donut", key));
        let ix = self.interact(id, SENSE_HOVER, false);
        let mo = self.motion();
        let total: f32 = parts.iter().map(|p| p.1.max(0.0)).sum::<f32>().max(1e-6);
        let grow = self.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);
        let n = parts.len();
        let colors: Vec<Color> = (0..n).map(|i| series_color(self, i, n.max(2), None)).collect();
        let fracs: Vec<f32> = parts.iter().enumerate().map(|(i, p)| self.anims.spring_from(id, 10 + i as u32, 0.0, p.1.max(0.0) / total, mo.smooth)).collect();
        // Hit test by angle.
        let hovered = ix.pointer.filter(|_| ix.hovered).and_then(|p| {
            let c = vec2(ix.rect.min.x + size / 2.0, ix.rect.min.y + size / 2.0);
            let d = p - c;
            let dist = d.length();
            if dist < size * 0.30 || dist > size / 2.0 + 6.0 {
                return None;
            }
            let ang = (d.x.atan2(-d.y)).rem_euclid(TAU);
            let mut acc = 0.0;
            for (i, f) in fracs.iter().enumerate() {
                acc += f * TAU;
                if ang <= acc {
                    return Some(i);
                }
            }
            None
        });
        let pops: Vec<f32> = (0..n).map(|i| self.spring(id.with(("seg", i)), 0, if hovered == Some(i) { 5.0 } else { 0.0 }, mo.bouncy)).collect();
        let center_t = self.fade(id, 1, hovered.is_some(), mo.hover_in, mo.hover_out, Ease::Linear);
        let big = {
            let mut s = self.text_style(TextRole::Heading2);
            s.tabular = true;
            s
        };
        let cap = self.text_style(TextRole::Caption);
        let fg = self.color(Token::Foreground);
        let muted = self.color(Token::TextMuted);
        let (center_v, center_l) = match hovered {
            Some(i) => (format!("{:.0}%", parts[i].1 / total * 100.0), parts[i].0.to_string()),
            None => (compact(total), "Total".to_string()),
        };
        let _ = center_t;
        let legend_colors = colors.clone();
        let gap = if size < 150.0 { 16.0 } else { 24.0 };
        self.begin(Style { dir: Dir::Row, gap, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.leaf(
            Style::default(),
            vec2(size, size),
            Some(id),
            SENSE_HOVER,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let c = r.center();
                let ro = size / 2.0 - 6.0;
                let ri = ro * 0.66;
                let gap = if n > 1 { 0.025 } else { 0.0 };
                let mut a = 0.0;
                for i in 0..n {
                    let sweep = fracs[i] * TAU * grow;
                    if sweep > gap * 2.0 {
                        cx.painter.arc_band(c, ri, ro + pops[i], a + gap, a + sweep - gap, colors[i]);
                    }
                    a += sweep;
                }
                let g = cx.galley(&center_v, &big);
                cx.painter.text(cx.text, &g, vec2((c.x - g.width / 2.0).round(), (c.y - g.height / 2.0 - 7.0).round()), fg);
                let g2 = cx.galley(&center_l, &cap);
                cx.painter.text(cx.text, &g2, vec2((c.x - g2.width / 2.0).round(), (c.y + 9.0).round()), muted);
            })),
        );
        // Legend with values.
        let st = self.text_style(TextRole::Body);
        let mut vs = self.text_style(TextRole::Body);
        vs.tabular = true;
        let sec = self.color(Token::TextSecondary);
        self.begin(Style { gap: 8.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for (i, (name, v)) in parts.iter().enumerate() {
            let col = legend_colors[i];
            let hl = hovered.is_none() || hovered == Some(i);
            let a = self.fade(id.with(("lg", i)), 0, hl, mo.hover_in, mo.hover_out, Ease::Linear);
            let alpha = 0.45 + 0.55 * a;
            self.begin(Style { dir: Dir::Row, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.leaf(Style::default(), vec2(10.0, 10.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(3.0), col.multiply_alpha(alpha), 0.0, Color::TRANSPARENT))));
            let name_w = if size < 150.0 { 72.0 } else { 110.0 };
            self.text_leaf(name.to_string(), st, fg.multiply_alpha(alpha), false, true, Style { width: Size::Fixed(name_w), ..Default::default() });
            self.text_leaf(format!("{:.0}%", v / total * 100.0), vs, sec.multiply_alpha(alpha), false, false, Style::default());
            self.end();
        }
        self.end();
        self.end();
        Response { id, rect: ix.rect, hovered: ix.hovered, ..Default::default() }
    }

    /// A semicircular gauge for a load-style `value` in `0..=1` (accent,
    /// then warning above 65% and danger above 85%), with a caption.
    pub fn gauge(&mut self, key: &str, value: f32, caption: &str) -> Response {
        let col = if value > 0.85 {
            self.color(Token::Danger)
        } else if value > 0.65 {
            self.color(Token::Warning)
        } else {
            self.color(Token::Accent)
        };
        self.gauge_colored(key, value, caption, col)
    }

    /// A gauge in a fixed color (e.g. a score where high is good).
    pub fn gauge_colored(&mut self, key: &str, value: f32, caption: &str, col: Color) -> Response {
        let id = self.make_id(("gauge", key));
        let mo = self.motion();
        let v = self.anims.spring_from(id, 0, 0.0, value.clamp(0.0, 1.0), mo.smooth);
        let track = self.color(Token::Track).multiply_alpha(0.6);
        let mut big = self.text_style(TextRole::Heading2);
        big.tabular = true;
        let cap = self.text_style(TextRole::Caption);
        let fg = self.color(Token::Foreground);
        let muted = self.color(Token::TextMuted);
        let caption = caption.to_string();
        let w = 160.0;
        self.leaf(
            Style::default(),
            vec2(w, w * 0.70),
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let c = vec2(r.center().x, r.min.y + w / 2.0);
                let ro = w / 2.0 - 4.0;
                let ri = ro - 12.0;
                let (a0, a1) = (-PI / 2.0 - 0.35, PI / 2.0 + 0.35);
                cx.painter.arc_band(c, ri, ro, a0, a1, track);
                let end = a0 + (a1 - a0) * v.clamp(0.0, 1.0);
                if end > a0 + 0.01 {
                    cx.painter.arc_band(c, ri, ro, a0, end, col);
                    let d = vec2(end.sin(), -end.cos());
                    cx.painter.circle(c + d * ((ri + ro) / 2.0), 6.0, col, 0.0, Color::TRANSPARENT);
                }
                let g = cx.galley(&format!("{:.0}%", v * 100.0), &big);
                cx.painter.text(cx.text, &g, vec2((c.x - g.width / 2.0).round(), (c.y - g.height + 4.0).round()), fg);
                let g2 = cx.galley(&caption, &cap);
                cx.painter.text(cx.text, &g2, vec2((c.x - g2.width / 2.0).round(), (c.y + 8.0).round()), muted);
            })),
        );
        Response { id, ..Default::default() }
    }

    /// A stat tile: label and delta badge, big value, and an optional
    /// full-width sparkline.
    pub fn stat(&mut self, label: &str, value: &str, delta: Option<f32>, trend: Option<&[f32]>) {
        let lab = self.text_style(TextRole::Label);
        let mut big = self.text_style(TextRole::Heading1);
        big.tabular = true;
        let sec = self.color(Token::TextSecondary);
        let fg = self.color(Token::Foreground);
        self.begin(Style { gap: 6.0, width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf(label, lab, sec, false, true, Style { width: Size::FILL, ..Default::default() });
        if let Some(d) = delta {
            let tone = if d >= 0.0 { super::toast::Tone::Success } else { super::toast::Tone::Danger };
            self.badge(format!("{}{:.1}%", if d >= 0.0 { "+" } else { "" }, d), tone);
        }
        self.end();
        self.text_leaf(value, big, fg, false, true, Style::default());
        if let Some(t) = trend {
            let col = match delta {
                Some(d) if d < 0.0 => Some(self.color(Token::Danger)),
                Some(_) => Some(self.color(Token::Success)),
                None => None,
            };
            self.sparkline_sized(label, t, col, Some(f32::INFINITY));
        }
        self.end();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_are_nice() {
        assert_eq!(nice_ticks(0.0, 97.0, 5), vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);
        let t = nice_ticks(-12.0, 30.0, 4);
        assert!(t.first().unwrap() <= &-12.0 && t.last().unwrap() >= &30.0);
        assert_eq!(compact(1250.0), "1.25k");
        assert_eq!(compact(3_400_000.0), "3.4M");
        assert_eq!(compact(12.0), "12");
    }

    #[test]
    fn monotone_never_overshoots() {
        let pts = [vec2(0.0, 0.0), vec2(1.0, 10.0), vec2(2.0, 10.0), vec2(3.0, 0.0)];
        for p in monotone(&pts, 12) {
            assert!(p.y <= 10.0 + 1e-3 && p.y >= -1e-3, "{p:?}");
        }
    }
}
