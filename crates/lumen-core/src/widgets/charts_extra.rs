//! Extended chart and data-visualization widgets:
//! - [`ProgressRing`] and [`ActivityRings`] (sweeping concentric arc gauges with rounded caps)
//! - [`BarList`] (ranked horizontal proportion bars)
//! - [`Heatmap`] (2D contribution / latency density matrix with OKLab color ramp and tooltip)
//! - [`RadarChart`] (multi-axis spider polygon chart with translucent mesh fills)
//! - [`ScatterChart`] (2D scatter / bubble plot with nice axes and nearest-point tooltip)
//! - [`FunnelChart`] (conversion trapezoid funnel with stage drop-off percentages)
//! - [`CandlestickChart`] (OHLC wicks and bodies with crosshair readout)

use std::f32::consts::{FRAC_PI_2, TAU};

use crate::anim::Ease;
use crate::color::Color;
use crate::input::CursorIcon;
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides, Vec2};
use crate::paint::Painter;
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};

use super::chart::{compact, legend, nice_ticks, series_color, tooltip_panel, Series, Tip};

const SENSE_HOVER: Sense = Sense { click: true, drag: false, focusable: false, scroll: false };

/// Draw an anti-aliased ring arc with rounded endcaps from angle `0` (12 o'clock)
/// to `sweep` radians clockwise.
fn draw_ring_arc(painter: &mut Painter, c: Vec2, r_mid: f32, thickness: f32, sweep: f32, track_c: Color, fill_c: Color) {
    let half_t = thickness * 0.5;
    let r_in = (r_mid - half_t).max(1.0);
    let r_out = r_mid + half_t;
    painter.arc_band(c, r_in, r_out, 0.0, TAU, track_c);
    let sw = sweep.clamp(0.0, TAU);
    if sw > 0.005 {
        painter.arc_band(c, r_in, r_out, 0.0, sw, fill_c);
        if sw < TAU - 0.02 {
            let p0 = c + vec2(0.0, -r_mid);
            let p1 = c + vec2(sw.sin() * r_mid, -sw.cos() * r_mid);
            painter.circle(p0, half_t, fill_c, 0.0, Color::TRANSPARENT);
            painter.circle(p1, half_t, fill_c, 0.0, Color::TRANSPARENT);
        }
    }
}

// =========================================== ProgressRing & ActivityRings

/// Specification for one ring in [`Ui::activity_rings`].
#[derive(Clone, Debug)]
pub struct RingSpec {
    pub label: String,
    pub progress: f32,
    pub detail: String,
    pub color: Option<Color>,
}

impl RingSpec {
    pub fn new(label: impl Into<String>, progress: f32, detail: impl Into<String>) -> Self {
        RingSpec { label: label.into(), progress, detail: detail.into(), color: None }
    }
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

/// Circular progress ring with rounded caps and optional center label.
pub struct ProgressRing<'u> {
    ui: &'u mut Ui,
    key: String,
    value: f32,
    size: f32,
    thickness: f32,
    label: Option<String>,
    color: Option<Color>,
    shown: bool,
}

impl<'u> ProgressRing<'u> {
    pub fn size(mut self, px: f32) -> Self {
        self.size = px.max(36.0);
        self
    }
    pub fn thickness(mut self, px: f32) -> Self {
        self.thickness = px.clamp(3.0, 24.0);
        self
    }
    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("progress_ring", &self.key));
        let mo = ui.motion();
        let target = self.value.clamp(0.0, 1.0);
        let v = ui.anims.spring_from(id, 0, 0.0, target, mo.smooth).clamp(0.0, 1.0);
        let fill_c = self.color.unwrap_or_else(|| ui.color(Token::Accent));
        let track_c = ui.color(Token::Track).multiply_alpha(0.75);
        let fg_c = ui.color(Token::Foreground);
        let sub_c = ui.color(Token::TextMuted);
        let mut st_pct = ui.text_style(TextRole::BodyStrong);
        st_pct.tabular = true;
        if self.size >= 96.0 {
            st_pct.size = (st_pct.size + 3.0).round();
        }
        let st_sub = ui.text_style(TextRole::Caption);
        let sz = self.size;
        let th = self.thickness;
        let sub_lbl = self.label.clone();

        ui.leaf(
            Style::default(),
            vec2(sz, sz),
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let c = r.center();
                let r_mid = (sz * 0.5 - th * 0.5 - 2.0).max(6.0);
                draw_ring_arc(&mut cx.painter, c, r_mid, th, v * TAU, track_c, fill_c);
                let pct = format!("{:.0}%", (v * 100.0).round());
                let g_p = cx.galley(&pct, &st_pct);
                if let Some(lbl) = &sub_lbl {
                    let g_l = cx.galley(lbl, &st_sub);
                    let tot_h = g_p.height + g_l.height;
                    let y0 = c.y - tot_h * 0.5;
                    cx.painter.text(cx.text, &g_p, vec2(c.x - g_p.width * 0.5, y0), fg_c);
                    cx.painter.text(cx.text, &g_l, vec2(c.x - g_l.width * 0.5, y0 + g_p.height - 1.0), sub_c);
                } else {
                    cx.painter.text(cx.text, &g_p, vec2(c.x - g_p.width * 0.5, c.y - g_p.height * 0.5), fg_c);
                }
            })),
        );
        let rect = ui.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }
}

impl Drop for ProgressRing<'_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

// ================================================================= BarList

/// Ranked horizontal bar row for [`Ui::bar_list`].
#[derive(Clone, Debug)]
pub struct BarListItem {
    pub label: String,
    pub value: f32,
    pub formatted: Option<String>,
}

impl BarListItem {
    pub fn new(label: impl Into<String>, value: f32) -> Self {
        BarListItem { label: label.into(), value, formatted: None }
    }
    pub fn formatted(mut self, text: impl Into<String>) -> Self {
        self.formatted = Some(text.into());
        self
    }
}

// ================================================================= Heatmap

/// 2D density / contribution heatmap with perceptual OKLab color ramp and hover tooltip.
pub struct Heatmap<'u> {
    ui: &'u mut Ui,
    key: String,
    rows: usize,
    cols: usize,
    values: Vec<f32>,
    row_labels: Vec<String>,
    col_labels: Vec<String>,
    unit: String,
    cell_gap: f32,
    shown: bool,
}

impl<'u> Heatmap<'u> {
    pub fn row_labels(mut self, labels: &[&str]) -> Self {
        self.row_labels = labels.iter().map(|s| (*s).to_owned()).collect();
        self
    }
    pub fn col_labels(mut self, labels: &[&str]) -> Self {
        self.col_labels = labels.iter().map(|s| (*s).to_owned()).collect();
        self
    }
    pub fn unit(mut self, u: impl Into<String>) -> Self {
        self.unit = u.into();
        self
    }
    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("heatmap", &self.key));
        let ix = ui.interact(id, SENSE_HOVER, false);
        let mo = ui.motion();
        let enter = ui.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);

        let rows = self.rows.max(1);
        let cols = self.cols.max(1);
        let max_v = self.values.iter().copied().fold(0.0f32, f32::max).max(1e-6);

        let cap = ui.text_style(TextRole::Caption);
        let left_w = if self.row_labels.is_empty() {
            0.0
        } else {
            self.row_labels.iter().map(|l| ui.galley(l, &cap).width).fold(0.0, f32::max) + 10.0
        };
        let top_h = if self.col_labels.is_empty() { 0.0 } else { 20.0 };
        let cell_h = 20.0;
        let gap = self.cell_gap;
        let total_h = top_h + rows as f32 * cell_h + (rows.saturating_sub(1)) as f32 * gap;

        let empty_c = ui.color(Token::SurfaceMuted);
        let low_c = ui.color(Token::AccentSubtle);
        let high_c = ui.color(Token::Accent);
        let ring_c = ui.color(Token::Foreground);
        let muted_c = ui.color(Token::TextMuted);
        let tip = Tip::new(ui);

        let vals = self.values.clone();
        let r_lbls = self.row_labels.clone();
        let c_lbls = self.col_labels.clone();
        let unit = self.unit.clone();
        let ptr = ix.pointer.filter(|_| ix.hovered);

        ui.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(240.0, total_h),
            Some(id),
            SENSE_HOVER,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let grid_x = r.min.x + left_w;
                let grid_y = r.min.y + top_h;
                let grid_w = (r.max.x - grid_x).max(20.0);
                let cell_w = ((grid_w - (cols.saturating_sub(1)) as f32 * gap) / cols as f32).max(4.0);

                for (ri, rl) in r_lbls.iter().enumerate().take(rows) {
                    if !rl.is_empty() {
                        let g = cx.galley(rl, &cap);
                        let y = grid_y + ri as f32 * (cell_h + gap) + (cell_h - g.height) * 0.5;
                        cx.painter.text(cx.text, &g, vec2(grid_x - 8.0 - g.width, y.round()), muted_c);
                    }
                }
                if !c_lbls.is_empty() {
                    let widest = c_lbls.iter().map(|l| cx.galley(l, &cap).width).fold(0.0, f32::max) + 8.0;
                    let every = ((widest / (cell_w + gap)).ceil() as usize).max(1);
                    for (ci, cl) in c_lbls.iter().enumerate().take(cols) {
                        if ci % every == 0 && !cl.is_empty() {
                            let g = cx.galley(cl, &cap);
                            let x = grid_x + ci as f32 * (cell_w + gap) + (cell_w - g.width) * 0.5;
                            cx.painter.text(cx.text, &g, vec2(x.round(), r.min.y), muted_c);
                        }
                    }
                }

                let mut hovered_cell: Option<(usize, usize, Rect, f32)> = None;
                let rad = (cell_w.min(cell_h) * 0.22).clamp(2.0, 5.0);
                for ri in 0..rows {
                    for ci in 0..cols {
                        let v = vals.get(ri * cols + ci).copied().unwrap_or(0.0);
                        let t = (v / max_v).clamp(0.0, 1.0) * enter;
                        let col = if t <= 0.02 {
                            empty_c
                        } else if t < 0.35 {
                            empty_c.mix(low_c, t / 0.35)
                        } else {
                            low_c.mix(high_c, (t - 0.35) / 0.65)
                        };
                        let cr = Rect::from_xywh(
                            grid_x + ci as f32 * (cell_w + gap),
                            grid_y + ri as f32 * (cell_h + gap),
                            cell_w,
                            cell_h,
                        );
                        cx.painter.rrect(cr, Corners::all(rad), col, 0.0, Color::TRANSPARENT);
                        if let Some(p) = ptr {
                            if cr.expand(gap * 0.5).contains(p) {
                                hovered_cell = Some((ri, ci, cr, v));
                            }
                        }
                    }
                }

                if let Some((ri, ci, cr, v)) = hovered_cell {
                    cx.painter.rrect(cr.expand(1.0), Corners::all(rad + 1.0), Color::TRANSPARENT, 1.5, ring_c);
                    let r_name = r_lbls.get(ri).filter(|s| !s.is_empty()).cloned().unwrap_or_else(|| format!("Row {}", ri + 1));
                    let c_name = c_lbls.get(ci).filter(|s| !s.is_empty()).cloned().unwrap_or_else(|| format!("Col {}", ci + 1));
                    let title = format!("{r_name} · {c_name}");
                    let val_s = if unit.is_empty() { compact(v) } else { format!("{} {}", compact(v), unit) };
                    let tip_cl = tip.clone();
                    let bounds = r.expand(12.0);
                    cx.defer(
                        r,
                        Box::new(move |cx, _| {
                            tooltip_panel(cx, cr.center(), bounds, &title, &[(high_c, "Value".into(), val_s)], &tip_cl);
                        }),
                    );
                }
            })),
        );
        Response { id, rect: ix.rect, hovered: ix.hovered, ..Default::default() }
    }
}

impl Drop for Heatmap<'_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

// ============================================================== RadarChart

/// Polygon spider / radar chart over `0.0..=max_value` axes.
pub struct RadarChart<'u> {
    ui: &'u mut Ui,
    key: String,
    axes: Vec<String>,
    series: Vec<Series>,
    max_val: f32,
    height: f32,
    shown: bool,
}

impl<'u> RadarChart<'u> {
    pub fn max_value(mut self, m: f32) -> Self {
        self.max_val = m.max(1e-3);
        self
    }
    pub fn height(mut self, px: f32) -> Self {
        self.height = px.max(160.0);
        self
    }
    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("radar", &self.key));
        let ix = ui.interact(id, SENSE_HOVER, false);
        let mo = ui.motion();
        let k = self.axes.len().max(3);
        let ns = self.series.len();
        let colors: Vec<Color> = (0..ns).map(|i| series_color(ui, i, ns, self.series[i].color)).collect();
        let names: Vec<String> = self.series.iter().map(|s| s.name.clone()).collect();

        let mut anim_vals: Vec<Vec<f32>> = Vec::with_capacity(ns);
        for (si, s) in self.series.iter().enumerate() {
            let mut row = Vec::with_capacity(k);
            for ai in 0..k {
                let raw = s.values.get(ai).copied().unwrap_or(0.0);
                let slot = (si * 32 + ai) as u32;
                row.push(ui.anims.spring_from(id, slot, 0.0, raw, mo.smooth));
            }
            anim_vals.push(row);
        }

        let grid_c = ui.color(Token::BorderSubtle);
        let axis_c = ui.color(Token::Border);
        let lbl_c = ui.color(Token::TextSecondary);
        let surf_c = ui.color(Token::Surface);
        let cap = ui.text_style(TextRole::Label);
        let tip = Tip::new(ui);
        let axes = self.axes.clone();
        let max_v = self.max_val;
        let ptr = ix.pointer.filter(|_| ix.hovered);

        ui.begin(Style { width: Size::FILL, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if ns > 1 {
            let items: Vec<(Color, String)> = colors.iter().copied().zip(names.iter().cloned()).collect();
            legend(ui, &items);
        }

        ui.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(240.0, self.height),
            Some(id),
            SENSE_HOVER,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let c = r.center();
                let radius = (r.height().min(r.width()) * 0.5 - 30.0).max(36.0);
                let dir_of = |i: usize| {
                    let a = -FRAC_PI_2 + TAU * i as f32 / k as f32;
                    vec2(a.cos(), a.sin())
                };

                // 4 concentric polygon rings + radial spokes.
                for ring in 1..=4 {
                    let rr = radius * (ring as f32 / 4.0);
                    let mut pts: Vec<Vec2> = (0..k).map(|i| c + dir_of(i) * rr).collect();
                    pts.push(pts[0]);
                    cx.painter.polyline(&pts, 1.0, if ring == 4 { axis_c } else { grid_c });
                }
                for i in 0..k {
                    let p_end = c + dir_of(i) * radius;
                    cx.painter.polyline(&[c, p_end], 1.0, grid_c);
                    if let Some(ax) = axes.get(i) {
                        let d = dir_of(i);
                        let g = cx.galley(ax, &cap);
                        let lp = c + d * (radius + 14.0);
                        let x = if d.x.abs() < 0.2 {
                            lp.x - g.width * 0.5
                        } else if d.x > 0.0 {
                            lp.x
                        } else {
                            lp.x - g.width
                        };
                        let y = lp.y - g.height * 0.5;
                        cx.painter.text(cx.text, &g, vec2(x.round(), y.round()), lbl_c);
                    }
                }

                // Series polygons (mesh fan + anti-aliased outline + vertex dots).
                for (si, vals) in anim_vals.iter().enumerate() {
                    let col = colors[si];
                    let pts: Vec<Vec2> = (0..k)
                        .map(|i| {
                            let t = (vals[i] / max_v).clamp(0.0, 1.0);
                            c + dir_of(i) * (radius * t)
                        })
                        .collect();
                    let fill_c = col.multiply_alpha(if ns > 1 { 0.20 } else { 0.26 });
                    let mut verts = Vec::with_capacity(k + 1);
                    verts.push((c, fill_c.multiply_alpha(0.55)));
                    for &p in &pts {
                        verts.push((p, fill_c));
                    }
                    let mut idx = Vec::with_capacity(k * 3);
                    for i in 0..k as u32 {
                        idx.extend_from_slice(&[0, 1 + i, 1 + (i + 1) % k as u32]);
                    }
                    cx.painter.mesh(&verts, &idx);

                    let mut closed = pts.clone();
                    closed.push(pts[0]);
                    cx.painter.polyline(&closed, 2.0, col);
                    for &p in &pts {
                        cx.painter.circle(p, 3.5, surf_c, 2.0, col);
                    }
                }

                // Nearest axis hover tooltip.
                if let Some(p) = ptr {
                    let v = p - c;
                    if v.length() <= radius + 24.0 {
                        let mut ang = v.y.atan2(v.x) + FRAC_PI_2;
                        if ang < 0.0 {
                            ang += TAU;
                        }
                        let best = ((ang / TAU * k as f32).round() as usize) % k;
                        let spoke = c + dir_of(best) * radius;
                        cx.painter.polyline(&[c, spoke], 1.5, axis_c);
                        let title = axes.get(best).cloned().unwrap_or_else(|| format!("Axis {}", best + 1));
                        let rows: Vec<(Color, String, String)> = (0..ns)
                            .map(|si| (colors[si], names[si].clone(), compact(anim_vals[si][best])))
                            .collect();
                        let tip_cl = tip.clone();
                        let bounds = r.expand(8.0);
                        cx.defer(r, Box::new(move |cx, _| tooltip_panel(cx, spoke, bounds, &title, &rows, &tip_cl)));
                    }
                }
            })),
        );
        ui.end();
        Response { id, rect: ix.rect, hovered: ix.hovered, ..Default::default() }
    }
}

impl Drop for RadarChart<'_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

// ============================================================ ScatterChart

/// Point in a [`ScatterSeries`].
#[derive(Clone, Debug)]
pub struct ScatterPoint {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub label: Option<String>,
}

impl ScatterPoint {
    pub fn new(x: f32, y: f32) -> Self {
        ScatterPoint { x, y, radius: 5.5, label: None }
    }
    pub fn bubble(x: f32, y: f32, radius: f32, label: impl Into<String>) -> Self {
        ScatterPoint { x, y, radius: radius.clamp(3.0, 22.0), label: Some(label.into()) }
    }
}

/// Named group of points in [`ScatterChart`].
#[derive(Clone, Debug)]
pub struct ScatterSeries {
    pub name: String,
    pub points: Vec<ScatterPoint>,
    pub color: Option<Color>,
}

impl ScatterSeries {
    pub fn new(name: impl Into<String>, points: impl Into<Vec<ScatterPoint>>) -> Self {
        ScatterSeries { name: name.into(), points: points.into(), color: None }
    }
}

/// 2D scatter and bubble chart with nice X/Y axes and hover crosshair readout.
pub struct ScatterChart<'u> {
    ui: &'u mut Ui,
    key: String,
    series: Vec<ScatterSeries>,
    height: f32,
    shown: bool,
}

impl<'u> ScatterChart<'u> {
    pub fn height(mut self, px: f32) -> Self {
        self.height = px.max(160.0);
        self
    }
    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("scatter", &self.key));
        let ix = ui.interact(id, SENSE_HOVER, false);
        let mo = ui.motion();
        let enter = ui.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);

        let ns = self.series.len();
        let colors: Vec<Color> = (0..ns).map(|i| series_color(ui, i, ns, self.series[i].color)).collect();
        let names: Vec<String> = self.series.iter().map(|s| s.name.clone()).collect();

        let (mut x_lo, mut x_hi, mut y_lo, mut y_hi) = (f32::MAX, f32::MIN, 0.0f32, f32::MIN);
        for s in &self.series {
            for p in &s.points {
                x_lo = x_lo.min(p.x);
                x_hi = x_hi.max(p.x);
                y_lo = y_lo.min(p.y);
                y_hi = y_hi.max(p.y);
            }
        }
        if x_lo >= x_hi {
            x_lo = 0.0;
            x_hi = 10.0;
        }
        if y_lo >= y_hi {
            y_lo = 0.0;
            y_hi = 10.0;
        }
        let x_ticks = nice_ticks(x_lo, x_hi, 5);
        let y_ticks = nice_ticks(y_lo, y_hi, 4);
        let (x0, x1) = (*x_ticks.first().unwrap_or(&0.0), *x_ticks.last().unwrap_or(&10.0));
        let (y0, y1) = (*y_ticks.first().unwrap_or(&0.0), *y_ticks.last().unwrap_or(&10.0));

        let cap = ui.text_style(TextRole::Caption);
        let y_lbls: Vec<String> = y_ticks.iter().map(|t| compact(*t)).collect();
        let x_lbls: Vec<String> = x_ticks.iter().map(|t| compact(*t)).collect();
        let axis_w = y_lbls.iter().map(|l| ui.galley(l, &cap).width).fold(0.0, f32::max) + 10.0;

        let grid_c = ui.color(Token::BorderSubtle);
        let axis_c = ui.color(Token::Border);
        let muted_c = ui.color(Token::TextMuted);
        let tip = Tip::new(ui);
        let ser_cl = self.series.clone();
        let ptr = ix.pointer.filter(|_| ix.hovered);

        ui.begin(Style { width: Size::FILL, gap: 10.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if ns > 1 {
            let items: Vec<(Color, String)> = colors.iter().copied().zip(names.iter().cloned()).collect();
            legend(ui, &items);
        }

        ui.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(240.0, self.height),
            Some(id),
            SENSE_HOVER,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let plot = Rect {
                    min: vec2(r.min.x + axis_w, r.min.y + 8.0),
                    max: vec2(r.max.x - 10.0, r.max.y - 22.0),
                };
                let x_of = |v: f32| plot.min.x + (v - x0) / (x1 - x0).max(1e-6) * plot.width();
                let y_of = |v: f32| plot.max.y - (v - y0) / (y1 - y0).max(1e-6) * plot.height();

                for (t, lbl) in y_ticks.iter().zip(&y_lbls) {
                    let y = y_of(*t);
                    if *t == y0 {
                        cx.painter.rect(Rect::from_xywh(plot.min.x, cx.painter.snap(y), plot.width(), 1.0), axis_c);
                    } else {
                        cx.painter.dashed_hline(plot.min.x, plot.max.x, y, 4.0, 4.0, grid_c);
                    }
                    let g = cx.galley(lbl, &cap);
                    cx.painter.text(cx.text, &g, vec2(plot.min.x - 8.0 - g.width, (y - g.height * 0.5).round()), muted_c);
                }
                for (t, lbl) in x_ticks.iter().zip(&x_lbls) {
                    let x = x_of(*t);
                    let g = cx.galley(lbl, &cap);
                    let gx = (x - g.width * 0.5).clamp(plot.min.x, r.max.x - g.width);
                    cx.painter.text(cx.text, &g, vec2(gx.round(), plot.max.y + 6.0), muted_c);
                }

                let mut nearest: Option<(usize, usize, Vec2, f32)> = None;
                for (si, s) in ser_cl.iter().enumerate() {
                    let col = colors[si];
                    for (pi, pt) in s.points.iter().enumerate() {
                        let sc = vec2(x_of(pt.x), y_of(y0 + (pt.y - y0) * enter));
                        let rad = pt.radius * (0.4 + 0.6 * enter);
                        cx.painter.circle(sc, rad, col.multiply_alpha(0.32), 1.75, col);
                        if let Some(p) = ptr {
                            let d = (p - sc).length();
                            if d < 28.0 && nearest.is_none_or(|(_, _, _, best_d)| d < best_d) {
                                nearest = Some((si, pi, sc, d));
                            }
                        }
                    }
                }

                if let Some((si, pi, sc, _)) = nearest {
                    let pt = &ser_cl[si].points[pi];
                    let col = colors[si];
                    cx.painter.rect(Rect::from_xywh(cx.painter.snap(sc.x), plot.min.y, 1.0, plot.height()), axis_c.multiply_alpha(0.65));
                    cx.painter.circle(sc, pt.radius + 3.0, Color::TRANSPARENT, 1.5, col);
                    let title = pt.label.clone().unwrap_or_else(|| ser_cl[si].name.clone());
                    let rows = vec![(col, "X / Y".into(), format!("{} · {}", compact(pt.x), compact(pt.y)))];
                    let tip_cl = tip.clone();
                    let bounds = r.expand(8.0);
                    cx.defer(r, Box::new(move |cx, _| tooltip_panel(cx, sc, bounds, &title, &rows, &tip_cl)));
                }
            })),
        );
        ui.end();
        Response { id, rect: ix.rect, hovered: ix.hovered, ..Default::default() }
    }
}

impl Drop for ScatterChart<'_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

// ============================================================= FunnelChart

/// Conversion funnel chart with centered tapering trapezoids and drop-off percentages.
pub struct FunnelChart<'u> {
    ui: &'u mut Ui,
    key: String,
    stages: Vec<(String, f32)>,
    height: f32,
    shown: bool,
}

impl<'u> FunnelChart<'u> {
    pub fn height(mut self, px: f32) -> Self {
        self.height = px.max(150.0);
        self
    }
    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("funnel", &self.key));
        let ix = ui.interact(id, SENSE_HOVER, false);
        let mo = ui.motion();
        let enter = ui.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);

        let n = self.stages.len().max(1);
        let top_v = self.stages.first().map(|(_, v)| *v).unwrap_or(1.0).max(1e-6);
        let base_c = ui.color(Token::Accent);
        let end_c = ui.color(Token::Series3);
        let fg_c = ui.color(Token::Foreground);
        let muted_c = ui.color(Token::TextSecondary);
        let st_lbl = ui.text_style(TextRole::BodyStrong);
        let mut st_val = ui.text_style(TextRole::Caption);
        st_val.tabular = true;
        let tip = Tip::new(ui);
        let stages = self.stages.clone();
        let ptr = ix.pointer.filter(|_| ix.hovered);

        ui.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(260.0, self.height),
            Some(id),
            SENSE_HOVER,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let left_col = 116.0f32.min(r.width() * 0.28);
                let right_col = 112.0f32.min(r.width() * 0.28);
                let funnel_x0 = r.min.x + left_col;
                let funnel_w = (r.width() - left_col - right_col).max(60.0);
                let cx_mid = funnel_x0 + funnel_w * 0.5;
                let gap = 5.0;
                let band_h = ((r.height() - gap * (n.saturating_sub(1)) as f32) / n as f32).max(18.0);

                let mut hov_stage: Option<(usize, Vec2)> = None;
                for i in 0..n {
                    let (ref name, val) = stages[i];
                    let next_val = stages.get(i + 1).map(|(_, v)| *v).unwrap_or(val * 0.86);
                    let w_top = (val / top_v).clamp(0.08, 1.0) * funnel_w * enter;
                    let w_bot = (next_val / top_v).clamp(0.06, 1.0) * funnel_w * enter;
                    let y0 = r.min.y + i as f32 * (band_h + gap);
                    let y1 = y0 + band_h;

                    let row_rect = Rect::from_xywh(r.min.x, y0, r.width(), band_h);
                    let is_hov = ptr.is_some_and(|p| row_rect.contains(p));
                    if is_hov {
                        hov_stage = Some((i, vec2(cx_mid, (y0 + y1) * 0.5)));
                    }

                    let mix_t = if n > 1 { i as f32 / (n - 1) as f32 } else { 0.0 };
                    let col = base_c.mix(end_c, mix_t * 0.55).multiply_alpha(if is_hov { 0.96 } else { 0.82 });

                    let tl = vec2(cx_mid - w_top * 0.5, y0);
                    let tr = vec2(cx_mid + w_top * 0.5, y0);
                    let br = vec2(cx_mid + w_bot * 0.5, y1);
                    let bl = vec2(cx_mid - w_bot * 0.5, y1);
                    cx.painter.mesh(&[(tl, col), (tr, col), (br, col.multiply_alpha(0.88)), (bl, col.multiply_alpha(0.88))], &[0, 1, 2, 0, 2, 3]);
                    cx.painter.polyline(&[tl, bl], 1.2, col);
                    cx.painter.polyline(&[tr, br], 1.2, col);

                    let g_n = cx.galley(name, &st_lbl);
                    cx.painter.text(cx.text, &g_n, vec2(r.min.x + 2.0, (y0 + (band_h - g_n.height) * 0.5).round()), fg_c);

                    let pct = (val / top_v * 100.0).round();
                    let stat = format!("{}  ({pct:.0}%)", compact(val));
                    let g_v = cx.galley(&stat, &st_val);
                    cx.painter.text(cx.text, &g_v, vec2(r.max.x - 2.0 - g_v.width, (y0 + (band_h - g_v.height) * 0.5).round()), muted_c);
                }

                if let Some((si, anchor)) = hov_stage {
                    let (ref nm, val) = stages[si];
                    let prev_v = if si > 0 { stages[si - 1].1.max(1e-6) } else { val.max(1e-6) };
                    let step_pct = (val / prev_v * 100.0).round();
                    let rows = vec![
                        (base_c, "Volume".into(), compact(val)),
                        (end_c, "Step keep".into(), format!("{step_pct:.0}%")),
                    ];
                    let title = nm.clone();
                    let tip_cl = tip.clone();
                    let bounds = r.expand(10.0);
                    cx.defer(r, Box::new(move |cx, _| tooltip_panel(cx, anchor, bounds, &title, &rows, &tip_cl)));
                }
            })),
        );
        Response { id, rect: ix.rect, hovered: ix.hovered, ..Default::default() }
    }
}

impl Drop for FunnelChart<'_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

// ======================================================== CandlestickChart

/// Single OHLC bar in a [`CandlestickChart`].
#[derive(Clone, Debug)]
pub struct Candle {
    pub label: String,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
}

impl Candle {
    pub fn new(label: impl Into<String>, open: f32, high: f32, low: f32, close: f32) -> Self {
        Candle { label: label.into(), open, high, low, close }
    }
}

/// OHLC candlestick chart with `Success` / `Danger` wicks and bodies and a crosshair tooltip.
pub struct CandlestickChart<'u> {
    ui: &'u mut Ui,
    key: String,
    candles: Vec<Candle>,
    height: f32,
    shown: bool,
}

impl<'u> CandlestickChart<'u> {
    pub fn height(mut self, px: f32) -> Self {
        self.height = px.max(160.0);
        self
    }
    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("candlestick", &self.key));
        let ix = ui.interact(id, SENSE_HOVER, false);
        let mo = ui.motion();
        let enter = ui.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);

        let n = self.candles.len();
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for c in &self.candles {
            lo = lo.min(c.low);
            hi = hi.max(c.high);
        }
        if lo >= hi {
            lo = 0.0;
            hi = 100.0;
        }
        let ticks = nice_ticks(lo, hi, 4);
        let (y0, y1) = (*ticks.first().unwrap_or(&lo), *ticks.last().unwrap_or(&hi));

        let cap = ui.text_style(TextRole::Caption);
        let tick_lbls: Vec<String> = ticks.iter().map(|t| compact(*t)).collect();
        let axis_w = tick_lbls.iter().map(|l| ui.galley(l, &cap).width).fold(0.0, f32::max) + 10.0;

        let up_c = ui.color(Token::Success);
        let down_c = ui.color(Token::Danger);
        let grid_c = ui.color(Token::BorderSubtle);
        let axis_c = ui.color(Token::Border);
        let muted_c = ui.color(Token::TextMuted);
        let tip = Tip::new(ui);
        let candles = self.candles.clone();
        let ptr = ix.pointer.filter(|_| ix.hovered);

        ui.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(260.0, self.height),
            Some(id),
            SENSE_HOVER,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let plot = Rect {
                    min: vec2(r.min.x + axis_w, r.min.y + 8.0),
                    max: vec2(r.max.x - 8.0, r.max.y - 22.0),
                };
                let y_of = |v: f32| plot.max.y - (v - y0) / (y1 - y0).max(1e-6) * plot.height();
                let mid_y = (y0 + y1) * 0.5;
                let anim_y = |v: f32| y_of(mid_y + (v - mid_y) * enter);

                for (t, lbl) in ticks.iter().zip(&tick_lbls) {
                    let y = y_of(*t);
                    if *t == y0 {
                        cx.painter.rect(Rect::from_xywh(plot.min.x, cx.painter.snap(y), plot.width(), 1.0), axis_c);
                    } else {
                        cx.painter.dashed_hline(plot.min.x, plot.max.x, y, 4.0, 4.0, grid_c);
                    }
                    let g = cx.galley(lbl, &cap);
                    cx.painter.text(cx.text, &g, vec2(plot.min.x - 8.0 - g.width, (y - g.height * 0.5).round()), muted_c);
                }

                if n == 0 {
                    return;
                }
                let slot = plot.width() / n as f32;
                let body_w = (slot * 0.58).clamp(4.0, 18.0);

                let widest = candles.iter().map(|c| cx.galley(&c.label, &cap).width).fold(0.0, f32::max) + 12.0;
                let every = ((widest / slot).ceil() as usize).max(1);

                let mut hov_idx: Option<usize> = None;
                for (i, c) in candles.iter().enumerate() {
                    let x_mid = plot.min.x + (i as f32 + 0.5) * slot;
                    if i % every == 0 {
                        let g = cx.galley(&c.label, &cap);
                        cx.painter.text(cx.text, &g, vec2((x_mid - g.width * 0.5).round(), plot.max.y + 6.0), muted_c);
                    }
                    if let Some(p) = ptr {
                        if (p.x - x_mid).abs() <= slot * 0.5 && p.y >= plot.min.y && p.y <= plot.max.y {
                            hov_idx = Some(i);
                        }
                    }

                    let bull = c.close >= c.open;
                    let col = if bull { up_c } else { down_c };
                    let yh = anim_y(c.high);
                    let yl = anim_y(c.low);
                    let yo = anim_y(c.open);
                    let yc = anim_y(c.close);

                    // Wick.
                    cx.painter.rect(Rect::from_xywh(cx.painter.snap(x_mid - 0.75), yh.min(yl), 1.5, (yh - yl).abs().max(1.0)), col);
                    // Body.
                    let top = yo.min(yc);
                    let bh = (yo - yc).abs().max(2.5);
                    let br = Rect::from_xywh(x_mid - body_w * 0.5, top, body_w, bh);
                    cx.painter.rrect(br, Corners::all(2.0), col, 0.0, Color::TRANSPARENT);
                }

                if let Some(hi_i) = hov_idx {
                    let c = &candles[hi_i];
                    let x_mid = plot.min.x + (hi_i as f32 + 0.5) * slot;
                    cx.painter.rect(Rect::from_xywh(cx.painter.snap(x_mid), plot.min.y, 1.0, plot.height()), axis_c.multiply_alpha(0.65));
                    let col = if c.close >= c.open { up_c } else { down_c };
                    let rows = vec![
                        (col, "Open / Close".into(), format!("{:.2} / {:.2}", c.open, c.close)),
                        (muted_c, "High / Low".into(), format!("{:.2} / {:.2}", c.high, c.low)),
                    ];
                    let title = c.label.clone();
                    let tip_cl = tip.clone();
                    let bounds = r.expand(8.0);
                    cx.defer(r, Box::new(move |cx, _| tooltip_panel(cx, vec2(x_mid, plot.center().y), bounds, &title, &rows, &tip_cl)));
                }
            })),
        );
        Response { id, rect: ix.rect, hovered: ix.hovered, ..Default::default() }
    }
}

impl Drop for CandlestickChart<'_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

impl Ui {
    /// Circular progress ring (`0.0..=1.0`) with rounded caps.
    pub fn progress_ring(&mut self, key: impl Into<String>, value: f32) -> ProgressRing<'_> {
        ProgressRing { ui: self, key: key.into(), value, size: 96.0, thickness: 8.0, label: None, color: None, shown: false }
    }

    /// Concentric multi-ring activity summary with rounded caps and an inline legend.
    pub fn activity_rings(&mut self, key: impl std::hash::Hash, rings: &[RingSpec], size: f32) -> Response {
        let id = self.make_id(("activity_rings", key));
        let mo = self.motion();
        let n = rings.len();
        let series_toks = [Token::Series0, Token::Series2, Token::Series4, Token::Series1];
        let colors: Vec<Color> = rings
            .iter()
            .enumerate()
            .map(|(i, r)| r.color.unwrap_or_else(|| self.color(series_toks[i % series_toks.len()])))
            .collect();
        let vals: Vec<f32> = rings
            .iter()
            .enumerate()
            .map(|(i, r)| self.anims.spring_from(id, i as u32, 0.0, r.progress.clamp(0.0, 1.0), mo.smooth).clamp(0.0, 1.0))
            .collect();

        let sz = size.max(88.0);
        let th = ((sz * 0.5 - 12.0) / (n.max(1) as f32 * 1.45)).clamp(6.0, 12.0);
        let ring_gap = (th * 0.36).max(3.0);
        let cols_cl = colors.clone();
        let vals_cl = vals.clone();

        self.begin(Style { dir: Dir::Row, gap: 22.0, align: Align::Center, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);
        self.leaf(
            Style::default(),
            vec2(sz, sz),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let c = r.center();
                for (i, &v) in vals_cl.iter().enumerate() {
                    let r_mid = sz * 0.5 - th * 0.5 - 2.0 - i as f32 * (th + ring_gap);
                    if r_mid > th {
                        let col = cols_cl[i];
                        draw_ring_arc(&mut cx.painter, c, r_mid, th, v * TAU, col.multiply_alpha(0.16), col);
                    }
                }
            })),
        );

        self.begin(Style { dir: Dir::Column, gap: 10.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for (i, spec) in rings.iter().enumerate() {
            let col = colors[i];
            let fg_c = self.color(Token::Foreground);
            let sub_c = self.color(Token::TextSecondary);
            self.begin(Style { dir: Dir::Row, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.leaf(
                Style::default(),
                vec2(10.0, 10.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| cx.painter.circle(r.center(), 4.5, col, 0.0, Color::TRANSPARENT))),
            );
            self.begin(Style { dir: Dir::Column, gap: 1.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let st_l = self.text_style(TextRole::BodyStrong);
            self.text_leaf(spec.label.clone(), st_l, fg_c, false, false, Style::default());
            let mut st_d = self.text_style(TextRole::Caption);
            st_d.tabular = true;
            self.text_leaf(format!("{} · {:.0}%", spec.detail, (vals[i] * 100.0).round()), st_d, sub_c, false, false, Style::default());
            self.end();
            self.end();
        }
        self.end();
        self.end();

        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Ranked horizontal proportion bars (top pages, endpoints or referrers).
    pub fn bar_list(&mut self, key: impl std::hash::Hash, items: &[BarListItem]) -> Response {
        let id = self.make_id(("bar_list", key));
        let mo = self.motion();
        let max_v = items.iter().map(|i| i.value).fold(0.0f32, f32::max).max(1e-6);
        let total_v: f32 = items.iter().map(|i| i.value.max(0.0)).sum::<f32>().max(1e-6);

        self.begin(Style { width: Size::FILL, dir: Dir::Column, gap: 6.0, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);
        for (i, item) in items.iter().enumerate() {
            let rid = id.with(("row", i));
            let rix = self.interact(rid, SENSE_HOVER, false);
            let ratio_target = (item.value / max_v).clamp(0.0, 1.0);
            let ratio = self.anims.spring_from(rid, 0, 0.0, ratio_target, mo.smooth).clamp(0.0, 1.0);
            let h = self.fade(rid, 1, rix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);

            let bar_c = self.color(Token::AccentSubtle).mix(self.color(Token::Accent).multiply_alpha(0.28), h);
            let track_c = self.color(Token::SurfaceMuted).multiply_alpha(0.45);
            let fg_c = self.color(Token::Foreground);
            let val_c = self.color(Token::TextSecondary);
            let pct_c = self.color(Token::TextMuted);

            self.begin(
                Style {
                    dir: Dir::Row,
                    width: Size::FILL,
                    height: Size::Fixed(32.0),
                    pad: Sides::xy(10.0, 0.0),
                    gap: 8.0,
                    align: Align::Center,
                    ..Default::default()
                },
                Some(rid),
                SENSE_HOVER,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| {
                    cx.painter.rrect(r, Corners::all(7.0), track_c, 0.0, Color::TRANSPARENT);
                    let bw = (r.width() * ratio).max(0.0);
                    if bw > 1.0 {
                        let br = Rect::from_xywh(r.min.x, r.min.y, bw, r.height());
                        cx.painter.rrect(br, Corners::all(7.0), bar_c, 0.0, Color::TRANSPARENT);
                    }
                })),
            );
            let st_lbl = self.text_style(TextRole::Mono);
            self.text_leaf(item.label.clone(), st_lbl, fg_c, false, true, Style { width: Size::FILL, ..Default::default() });
            let val_str = item.formatted.clone().unwrap_or_else(|| compact(item.value));
            let mut st_v = self.text_style(TextRole::BodyStrong);
            st_v.tabular = true;
            self.text_leaf(val_str, st_v, val_c, false, false, Style::default());
            let mut st_p = self.text_style(TextRole::Caption);
            st_p.tabular = true;
            let pct = (item.value / total_v * 100.0).round();
            let n_pct = self.text_leaf(format!("{pct:.0}%"), st_p, pct_c, false, false, Style { width: Size::Fixed(34.0), ..Default::default() });
            self.set_text_align(n_pct, lumen_text::TextAlign::End);
            self.end();
        }
        self.end();

        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// 2D density / contribution heatmap (`rows * cols` values in row-major order).
    pub fn heatmap(&mut self, key: impl Into<String>, rows: usize, cols: usize, values: &[f32]) -> Heatmap<'_> {
        Heatmap {
            ui: self,
            key: key.into(),
            rows,
            cols,
            values: values.to_vec(),
            row_labels: vec![],
            col_labels: vec![],
            unit: String::new(),
            cell_gap: 4.0,
            shown: false,
        }
    }

    /// Polygon spider / radar chart.
    pub fn radar_chart(&mut self, key: impl Into<String>, axes: &[&str], series: Vec<Series>) -> RadarChart<'_> {
        RadarChart {
            ui: self,
            key: key.into(),
            axes: axes.iter().map(|s| (*s).to_owned()).collect(),
            series,
            max_val: 100.0,
            height: 250.0,
            shown: false,
        }
    }

    /// 2D scatter / bubble chart.
    pub fn scatter_chart(&mut self, key: impl Into<String>, series: Vec<ScatterSeries>) -> ScatterChart<'_> {
        ScatterChart { ui: self, key: key.into(), series, height: 240.0, shown: false }
    }

    /// Conversion funnel chart (`(stage_name, volume)`).
    pub fn funnel_chart(&mut self, key: impl Into<String>, stages: &[(&str, f32)]) -> FunnelChart<'_> {
        FunnelChart {
            ui: self,
            key: key.into(),
            stages: stages.iter().map(|(n, v)| ((*n).to_owned(), *v)).collect(),
            height: 220.0,
            shown: false,
        }
    }

    /// OHLC candlestick chart.
    pub fn candlestick_chart(&mut self, key: impl Into<String>, candles: &[Candle]) -> CandlestickChart<'_> {
        CandlestickChart { ui: self, key: key.into(), candles: candles.to_vec(), height: 240.0, shown: false }
    }
}
