//! Bullet & watchlist charts, AI model & context-budget widgets, REST API
//! inspector, RBAC permission matrix, description lists, masonry grids,
//! collapsible cards and marquee tickers.

use std::f32::consts::PI;

use super::field::{Chrome, EditOpts};
use super::toast::Tone;
use crate::anim::Ease;
use crate::color::Color;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{ControlSize, TextRole, Token};
use crate::ui::{Response, Sense, Ui};

// ============================================================= BulletChart

/// Target-vs-actual entry in [`Ui::bullet_chart`].
#[derive(Clone, Debug)]
pub struct BulletMetric {
    pub label: String,
    pub actual: f32,
    pub target: f32,
    pub max: f32,
    pub unit: String,
}

impl BulletMetric {
    pub fn new(label: impl Into<String>, actual: f32, target: f32, max: f32, unit: impl Into<String>) -> Self {
        BulletMetric {
            label: label.into(),
            actual: actual.max(0.0),
            target: target.max(0.0),
            max: max.max(1e-3),
            unit: unit.into(),
        }
    }
}

// ============================================================= MetricTable

/// Watchlist row in [`Ui::metric_table`].
#[derive(Clone, Debug)]
pub struct MetricRow {
    pub name: String,
    pub sub: String,
    pub spark: Vec<f32>,
    pub value: String,
    pub delta_pct: f32,
}

impl MetricRow {
    pub fn new(name: impl Into<String>, sub: impl Into<String>, spark: Vec<f32>, value: impl Into<String>, delta_pct: f32) -> Self {
        MetricRow {
            name: name.into(),
            sub: sub.into(),
            spark,
            value: value.into(),
            delta_pct,
        }
    }
}

// =========================================================== AiModelPicker

/// Model descriptor in [`Ui::ai_model_picker`].
#[derive(Clone, Debug)]
pub struct AiModelSpec {
    pub name: String,
    pub provider: String,
    pub context_window: String,
    pub price_hint: String,
    pub speed_dots: u8,
    pub tags: Vec<String>,
}

impl AiModelSpec {
    pub fn new(
        name: impl Into<String>,
        provider: impl Into<String>,
        context_window: impl Into<String>,
        price_hint: impl Into<String>,
        speed_dots: u8,
        tags: &[&str],
    ) -> Self {
        AiModelSpec {
            name: name.into(),
            provider: provider.into(),
            context_window: context_window.into(),
            price_hint: price_hint.into(),
            speed_dots: speed_dots.min(5),
            tags: tags.iter().map(|s| (*s).to_owned()).collect(),
        }
    }
}

// ============================================================ ContextMeter

/// Token budget slice in [`Ui::context_meter`].
#[derive(Clone, Debug)]
pub struct ContextSlice {
    pub label: String,
    pub tokens_k: f32,
}

impl ContextSlice {
    pub fn new(label: impl Into<String>, tokens_k: f32) -> Self {
        ContextSlice { label: label.into(), tokens_k: tokens_k.max(0.0) }
    }
}

// ============================================================ ApiInspector

/// HTTP verb for [`Ui::api_inspector`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum HttpMethod {
    #[default]
    Get,
    Post,
    Put,
    Delete,
}

impl HttpMethod {
    pub const ALL: [HttpMethod; 4] = [HttpMethod::Get, HttpMethod::Post, HttpMethod::Put, HttpMethod::Delete];

    pub fn label(self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
        }
    }

    pub fn tone(self) -> Tone {
        match self {
            HttpMethod::Get => Tone::Success,
            HttpMethod::Post => Tone::Accent,
            HttpMethod::Put => Tone::Warning,
            HttpMethod::Delete => Tone::Danger,
        }
    }
}

// ========================================================= DescriptionList

/// Metadata entry in [`Ui::description_list`].
#[derive(Clone, Debug)]
pub struct DescPair {
    pub term: String,
    pub detail: String,
    pub mono: bool,
    pub copyable: bool,
}

impl DescPair {
    pub fn new(term: impl Into<String>, detail: impl Into<String>) -> Self {
        DescPair { term: term.into(), detail: detail.into(), mono: false, copyable: false }
    }
    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }
    pub fn copyable(mut self) -> Self {
        self.copyable = true;
        self
    }
}

// ============================================================= MasonryGrid

/// Card item in [`Ui::masonry_grid`].
#[derive(Clone, Debug)]
pub struct MasonryCard {
    pub tag: String,
    pub title: String,
    pub body: String,
    pub metric: Option<String>,
    pub tone: Tone,
}

impl MasonryCard {
    pub fn new(tag: impl Into<String>, title: impl Into<String>, body: impl Into<String>, tone: Tone) -> Self {
        MasonryCard {
            tag: tag.into(),
            title: title.into(),
            body: body.into(),
            metric: None,
            tone,
        }
    }
    pub fn metric(mut self, m: impl Into<String>) -> Self {
        self.metric = Some(m.into());
        self
    }
}

impl Ui {
    /// Stephen Few target-vs-actual bullet chart with qualitative background
    /// bands (`60% / 85% / 100%`), spring actual bar, and perpendicular target tick.
    pub fn bullet_chart(&mut self, key: impl std::hash::Hash, metrics: &[BulletMetric]) -> Response {
        let id = self.make_id(("bullet_chart", key));
        let mo = self.motion();
        let enter = self.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);

        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(16.0), gap: 14.0, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        for m in metrics {
            self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);

            self.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.text_leaf(m.label.clone(), self.text_style(TextRole::Label), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
            let mut st_r = self.text_style(TextRole::Caption);
            st_r.tabular = true;
            let beat = m.actual >= m.target;
            let status_c = self.color(if beat { Token::SuccessStrong } else { Token::WarningStrong });
            self.text_leaf(
                format!("{:.0}{} / target {:.0}{}", m.actual, m.unit, m.target, m.unit),
                st_r,
                status_c,
                false,
                false,
                Style::default(),
            );
            self.end();

            let act_frac = (m.actual / m.max).clamp(0.0, 1.0) * enter;
            let tgt_frac = (m.target / m.max).clamp(0.0, 1.0);
            let band3 = self.color(Token::SurfaceMuted);
            let band2 = self.color(Token::BorderSubtle);
            let band1 = self.color(Token::Border);
            let bar_c = self.color(if beat { Token::Success } else { Token::Accent });
            let tick_c = self.color(Token::Foreground);

            self.leaf(
                Style { width: Size::FILL, ..Default::default() },
                vec2(260.0, 22.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    // 3 qualitative range bands (100%, 80%, 55%).
                    cx.painter.rrect(r, Corners::all(6.0), band3, 0.0, Color::TRANSPARENT);
                    cx.painter.rrect(Rect::from_xywh(r.min.x, r.min.y, r.width() * 0.80, r.height()), Corners::all(6.0), band2, 0.0, Color::TRANSPARENT);
                    cx.painter.rrect(Rect::from_xywh(r.min.x, r.min.y, r.width() * 0.55, r.height()), Corners::all(6.0), band1, 0.0, Color::TRANSPARENT);

                    // Primary measure bar centered inside the band.
                    let bh = 10.0;
                    let bw = (r.width() * act_frac).max(3.0);
                    cx.painter.rrect(Rect::from_xywh(r.min.x, r.center().y - bh * 0.5, bw, bh), Corners::all(4.0), bar_c, 0.0, Color::TRANSPARENT);

                    // Crisp perpendicular target marker!
                    let tx = r.min.x + r.width() * tgt_frac;
                    cx.painter.rrect(Rect::from_xywh(tx - 1.5, r.min.y + 2.0, 3.0, r.height() - 4.0), Corners::all(1.5), tick_c, 0.0, Color::TRANSPARENT);
                })),
            );
            self.end();
        }
        self.end();

        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Compact observability / market watchlist table with inline sparklines,
    /// tabular values, and `%` delta badges.
    pub fn metric_table(&mut self, key: impl std::hash::Hash, rows: &[MetricRow]) -> Response {
        let id = self.make_id(("metric_table", key));
        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, pad: Sides::xy(0.0, 4.0), ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        for (ri, row) in rows.iter().enumerate() {
            let rid = id.with(("mr", ri));
            let r_ix = self.interact(rid, Sense::HOVER, false);
            let hov = self.fade(rid, 0, r_ix.hovered, self.motion().hover_in, self.motion().hover_out, Ease::Out);
            let h_bg = self.color(Token::Hover).multiply_alpha(hov);
            let not_last = ri + 1 < rows.len();

            self.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(14.0, 10.0), gap: 12.0, align: Align::Center, ..Default::default() },
                Some(rid),
                Sense::HOVER,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| {
                    if hov > 0.005 {
                        cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.min.y, (r.width() - 2.0).max(0.0), r.height()), h_bg);
                    }
                    if not_last {
                        cx.painter.rect(Rect::from_xywh(r.min.x + 14.0, r.max.y - 1.0, (r.width() - 28.0).max(0.0), 1.0), bd_c);
                    }
                })),
            );

            self.begin(Style { dir: Dir::Column, width: Size::Fill(1.2), gap: 1.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.text_leaf(row.name.clone(), self.text_style(TextRole::BodyStrong), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
            self.text_leaf(row.sub.clone(), self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, true, Style { width: Size::FILL, ..Default::default() });
            self.end();

            // Inline mini sparkline.
            let up = row.delta_pct >= 0.0;
            let sp_col = self.color(if up { Token::Success } else { Token::Danger });
            let pts_cl = row.spark.clone();
            self.leaf(
                Style::default(),
                vec2(96.0, 26.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    if pts_cl.len() >= 2 {
                        let lo = pts_cl.iter().copied().fold(f32::MAX, f32::min);
                        let hi = pts_cl.iter().copied().fold(f32::MIN, f32::max);
                        let span = (hi - lo).max(1e-4);
                        let n = pts_cl.len();
                        let mut upper = Vec::with_capacity(n);
                        let mut lower = Vec::with_capacity(n);
                        for (i, &v) in pts_cl.iter().enumerate() {
                            let x = r.min.x + (i as f32 / (n - 1) as f32) * r.width();
                            let y = r.max.y - 2.0 - ((v - lo) / span) * (r.height() - 4.0);
                            upper.push(vec2(x, y));
                            lower.push(vec2(x, r.max.y));
                        }
                        cx.painter.fill_between(&upper, &lower, sp_col.with_alpha(0.18), sp_col.with_alpha(0.0));
                        cx.painter.polyline(&upper, 1.6, sp_col);
                    }
                })),
            );

            let mut st_v = self.text_style(TextRole::Mono);
            st_v.tabular = true;
            st_v.weight = 600;
            self.begin(Style { width: Size::Fixed(80.0), dir: Dir::Row, justify: Justify::End, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.text_leaf(row.value.clone(), st_v, self.color(Token::Foreground), false, false, Style::default());
            self.end();

            let sign = if up { "+" } else { "" };
            self.begin(Style { width: Size::Fixed(64.0), dir: Dir::Row, justify: Justify::End, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.badge(format!("{sign}{:.1}%", row.delta_pct), if up { Tone::Success } else { Tone::Danger });
            self.end();
            self.end();
        }
        self.end();

        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Rich LLM model selector cards showing context window, speed dots,
    /// pricing hint, and capability tags.
    pub fn ai_model_picker(&mut self, key: impl std::hash::Hash, models: &[AiModelSpec], selected: &mut usize) -> bool {
        let id = self.make_id(("ai_model_picker", key));
        let n = models.len().max(1);
        *selected = (*selected).min(n - 1);
        let mo = self.motion();
        let mut changed = false;

        self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 8.0, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);
        for (mi, m) in models.iter().enumerate() {
            let mid = id.with(("mdl", mi));
            let ix = self.interact(mid, Sense::CLICK, false);
            if ix.focused {
                if self.key_pressed_for(mid, Key::ArrowDown) {
                    *selected = (*selected + 1) % n;
                    changed = true;
                } else if self.key_pressed_for(mid, Key::ArrowUp) {
                    *selected = (*selected + n - 1) % n;
                    changed = true;
                }
            }
            let is_sel = mi == *selected;
            let sel_t = self.spring(mid, 0, if is_sel { 1.0 } else { 0.0 }, mo.snappy).clamp(0.0, 1.0);
            let hov = self.fade(mid, 1, ix.hovered && !is_sel, mo.hover_in, mo.hover_out, Ease::Out);

            let r_card = (self.look().card.radius - 2.0).max(10.0);
            let bg_c = self
                .color(Token::Surface)
                .mix(self.color(Token::Hover), hov)
                .mix(self.color(Token::AccentSubtle).multiply_alpha(0.42), sel_t);
            let bd_c = self.color(Token::BorderSubtle).mix(self.color(Token::Accent), sel_t);

            self.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(14.0, 11.0), gap: 12.0, align: Align::Center, ..Default::default() },
                Some(mid),
                Sense::CLICK,
                CursorIcon::Pointer,
                false,
                Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0 + 0.5 * sel_t, bd_c))),
            );

            // Radio dot indicator.
            let dot_acc = self.color(Token::Accent);
            let dot_bd = self.color(Token::BorderStrong).mix(dot_acc, sel_t);
            self.leaf(
                Style::default(),
                vec2(18.0, 18.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    cx.painter.circle(r.center(), 8.0, Color::TRANSPARENT, 1.5, dot_bd);
                    if sel_t > 0.02 {
                        cx.painter.circle(r.center(), 4.5 * sel_t, dot_acc, 0.0, Color::TRANSPARENT);
                    }
                })),
            );

            self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 4.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            // Name, provider and tags wrap onto a second line in a narrow card.
            self.begin(Style { dir: Dir::Row, width: Size::FILL, wrap: true, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.text_leaf(m.name.clone(), self.text_style(TextRole::BodyStrong), self.color(Token::Foreground), false, false, Style::default());
            self.text_leaf(m.provider.clone(), self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, false, Style::default());
            for tag in &m.tags {
                self.badge(tag, if is_sel { Tone::Accent } else { Tone::Neutral });
            }
            self.end();

            self.begin(Style { dir: Dir::Row, gap: 10.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let mut st_m = self.text_style(TextRole::Mono);
            st_m.size = 11.0;
            self.text_leaf(m.context_window.clone(), st_m, self.color(Token::TextSecondary), false, false, Style::default());
            self.text_leaf("·", self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, false, Style::default());
            self.text_leaf(m.price_hint.clone(), st_m, self.color(Token::TextMuted), false, false, Style::default());
            self.end();
            self.end();

            // Speed rating dots on the right.
            let spd = m.speed_dots;
            let on_c = self.color(Token::Success);
            let off_c = self.color(Token::BorderStrong);
            self.leaf(
                Style::default(),
                vec2(46.0, 14.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    for k in 0..5u8 {
                        let cx_k = r.min.x + 5.0 + k as f32 * 9.0;
                        cx.painter.circle(vec2(cx_k, r.center().y), 2.8, if k < spd { on_c } else { off_c }, 0.0, Color::TRANSPARENT);
                    }
                })),
            );
            self.end();

            if ix.clicked && *selected != mi {
                *selected = mi;
                changed = true;
            }
        }
        self.end();
        changed
    }

    /// AI context-window token usage meter with a multi-slice circular ring and
    /// itemized token breakdown.
    pub fn context_meter(&mut self, key: impl std::hash::Hash, slices: &[ContextSlice], limit_k: f32) -> Response {
        let id = self.make_id(("context_meter", key));
        let mo = self.motion();
        let enter = self.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);

        let lim = limit_k.max(1.0);
        let used_k: f32 = slices.iter().map(|s| s.tokens_k).sum();
        let used_pct = (used_k / lim * 100.0).clamp(0.0, 100.0);

        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let track_c = self.color(Token::Track);
        let palette = [
            self.color(Token::Series0),
            self.color(Token::Series2),
            self.color(Token::Series1),
            self.color(Token::Series3),
        ];

        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::all(16.0), gap: 18.0, align: Align::Center, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        let sweeps: Vec<(f32, Color)> = slices
            .iter()
            .enumerate()
            .map(|(i, s)| ((s.tokens_k / lim).clamp(0.0, 1.0) * 2.0 * PI * enter, palette[i % palette.len()]))
            .collect();
        let pct_lbl = format!("{used_pct:.0}%");
        let mut st_p = self.text_style(TextRole::BodyStrong);
        st_p.tabular = true;
        let fg_c = self.color(Token::Foreground);

        self.leaf(
            Style::default(),
            vec2(84.0, 84.0),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let c = r.center();
                cx.painter.arc_band(c, 30.0, 39.0, 0.0, 2.0 * PI, track_c);
                let mut a0 = 0.0;
                for &(sw, col) in &sweeps {
                    if sw > 0.01 {
                        cx.painter.arc_band(c, 30.0, 39.0, a0, (a0 + sw - 0.025).max(a0 + 0.005), col);
                        a0 += sw;
                    }
                }
                let gp = cx.galley(&pct_lbl, &st_p);
                cx.painter.text(cx.text, &gp, c - vec2(gp.width * 0.5, gp.height * 0.5), fg_c);
            })),
        );

        self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf("Context window", self.text_style(TextRole::BodyStrong), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
        let mut st_tot = self.text_style(TextRole::Mono);
        st_tot.size = 11.5;
        st_tot.tabular = true;
        self.text_leaf(format!("{used_k:.1}k / {lim:.0}k tok"), st_tot, self.color(Token::TextMuted), false, false, Style::default());
        self.end();

        for (i, sl) in slices.iter().enumerate() {
            let col = palette[i % palette.len()];
            self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.leaf(Style::default(), vec2(8.0, 8.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.circle(r.center(), 4.0, col, 0.0, Color::TRANSPARENT))));
            self.text_leaf(sl.label.clone(), self.text_style(TextRole::Caption), self.color(Token::TextSecondary), false, true, Style { width: Size::FILL, ..Default::default() });
            let mut st_v = self.text_style(TextRole::Mono);
            st_v.size = 11.5;
            st_v.tabular = true;
            self.text_leaf(format!("{:.1}k", sl.tokens_k), st_v, self.color(Token::Foreground), false, false, Style::default());
            self.end();
        }
        self.end();
        self.end();

        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// REST / RPC endpoint request bar & response inspector with cycleable HTTP
    /// verb pill, URL editor, `"Send"` trigger, status/latency pills, and JSON body.
    pub fn api_inspector(
        &mut self,
        key: impl std::hash::Hash,
        method: &mut HttpMethod,
        url: &mut String,
        status_label: &str,
        latency_ms: f32,
        response_json: &str,
    ) -> bool {
        let id = self.make_id(("api_inspector", key));
        let edit_id = id.with("url_ed");
        let focused = self.focused() == Some(edit_id);
        let hovered = self.pointer().is_some_and(|p| self.rect_of(id.with("bar")).is_some_and(|r| r.contains(p)));
        let cm = self.control_metrics(ControlSize::Md);

        self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 10.0, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);

        // Request bar: Method pill + URL inline edit + Send button.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.field_begin(
            id.with("bar"),
            Chrome { hovered, focused, error: false, disabled: false },
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                height: Size::Fixed(cm.height),
                pad: Sides::xy(6.0, 0.0),
                gap: 8.0,
                align: Align::Center,
                ..Default::default()
            },
        );
        let m_id = id.with("verb");
        let m_ix = self.interact(m_id, Sense::CLICK, false);
        self.begin(Style::default(), Some(m_id), Sense::CLICK, CursorIcon::Pointer, false, None);
        self.badge(method.label(), method.tone());
        self.end();
        if m_ix.clicked {
            let pos = HttpMethod::ALL.iter().position(|x| *x == *method).unwrap_or(0);
            *method = HttpMethod::ALL[(pos + 1) % HttpMethod::ALL.len()];
        }

        let mut opts = EditOpts::new(self);
        opts.style = self.text_style(TextRole::Mono);
        let _ = self.inline_edit(edit_id, url, opts);
        self.end();

        let sent = self.scope("send_req", |ui| ui.button("Send").primary().icon("send").clicked());
        self.end();

        // Response card with status & latency header + syntax colored JSON.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.badge(status_label, Tone::Success);
        let mut st_l = self.text_style(TextRole::Mono);
        st_l.size = 11.5;
        st_l.tabular = true;
        self.text_leaf(format!("{latency_ms:.1} ms · application/json"), st_l, self.color(Token::TextMuted), false, true, Style { width: Size::FILL, ..Default::default() });
        self.end();

        self.scope(id, |ui| {
            ui.code_block("api_body", response_json).language("json");
        });
        self.end();
        sent
    }

    /// Interactive RBAC role × capability checkbox matrix (`roles` columns ×
    /// `capabilities` rows).
    pub fn permission_matrix(
        &mut self,
        key: impl std::hash::Hash,
        roles: &[&str],
        capabilities: &[&str],
        granted: &mut [Vec<bool>],
    ) -> bool {
        let id = self.make_id(("perm_matrix", key));
        let r_card = self.look().card.radius;
        let top_rad = (r_card - 1.0).max(0.0);
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let hdr_bg = self.color(Token::SurfaceMuted).multiply_alpha(0.6);
        let mut changed = false;

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        // Header row.
        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(14.0, 9.0), align: Align::Center, ..Default::default() },
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
        let mut st_h = self.text_style(TextRole::Caption);
        st_h.weight = 600;
        self.text_leaf("Capability", st_h, self.color(Token::TextSecondary), false, true, Style { width: Size::Fill(1.6), ..Default::default() });
        for &role in roles {
            self.begin(Style { width: Size::FILL, align: Align::Center, justify: Justify::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.text_leaf(role, st_h, self.color(Token::TextSecondary), false, false, Style::default());
            self.end();
        }
        self.end();

        // Capability rows.
        for (ci, &cap) in capabilities.iter().enumerate() {
            let not_last = ci + 1 < capabilities.len();
            self.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(14.0, 8.0), align: Align::Center, ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                if not_last {
                    Some(Box::new(move |cx, r| cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.max.y - 1.0, (r.width() - 2.0).max(0.0), 1.0), bd_c)))
                } else {
                    None
                },
            );
            self.text_leaf(cap, self.text_style(TextRole::Mono), self.color(Token::Foreground), false, true, Style { width: Size::Fill(1.6), ..Default::default() });
            for ri in 0..roles.len() {
                self.begin(Style { width: Size::FILL, justify: Justify::Center, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                if let Some(val) = granted.get_mut(ci).and_then(|r| r.get_mut(ri)) {
                    if self.scope(("pm", ci, ri), |ui| ui.checkbox("", val).changed()) {
                        changed = true;
                    }
                }
                self.end();
            }
            self.end();
        }
        self.end();
        changed
    }

    /// Two-column structured metadata / resource inspector description list
    /// with optional monospace & one-click copy icons.
    pub fn description_list(&mut self, key: impl std::hash::Hash, title: &str, pairs: &[DescPair]) -> Response {
        let id = self.make_id(("desc_list", key));
        let r_card = self.look().card.radius;
        let top_rad = (r_card - 1.0).max(0.0);
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let hdr_bg = self.color(Token::SurfaceMuted).multiply_alpha(0.6);

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        if !title.is_empty() {
            self.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(14.0, 9.0), align: Align::Center, ..Default::default() },
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
            self.text_leaf(title.to_owned(), self.text_style(TextRole::BodyStrong), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
            self.end();
        }

        for (pi, p) in pairs.iter().enumerate() {
            let not_last = pi + 1 < pairs.len();
            self.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(14.0, 9.0), gap: 12.0, align: Align::Center, ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                if not_last {
                    Some(Box::new(move |cx, r| cx.painter.rect(Rect::from_xywh(r.min.x + 14.0, r.max.y - 1.0, (r.width() - 28.0).max(0.0), 1.0), bd_c)))
                } else {
                    None
                },
            );
            self.text_leaf(p.term.clone(), self.text_style(TextRole::Label), self.color(Token::TextMuted), false, true, Style { width: Size::Fixed(148.0), ..Default::default() });
            let st_v = self.text_style(if p.mono { TextRole::Mono } else { TextRole::Body });
            self.text_leaf(p.detail.clone(), st_v, self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
            if p.copyable {
                let v_copy = p.detail.clone();
                if self.scope(("dl_cp", pi), |ui| ui.icon_button("copy").small().tooltip("Copy value").clicked()) {
                    self.set_copied_text(v_copy);
                }
            }
            self.end();
        }
        self.end();

        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Multi-column staggered masonry layout distributing variable-height cards
    /// automatically into balanced columns.
    pub fn masonry_grid(&mut self, key: impl std::hash::Hash, columns: usize, cards: &[MasonryCard]) -> Response {
        let id = self.make_id(("masonry_grid", key));
        // `columns` is the most it shows; narrow widths (last frame's) drop
        // columns rather than squeeze cards below a readable width.
        let min_col = self.look().control.text_size[1] * 11.0;
        let fit = self.rect_of(id).map(|r| ((r.width() + 12.0) / (min_col + 12.0)).floor() as usize).unwrap_or(columns);
        let n_cols = columns.clamp(1, 4).min(fit.max(1));

        // Estimate each card's relative height from body length + optional metric so columns balance naturally!
        let mut col_heights = vec![0usize; n_cols];
        let mut buckets: Vec<Vec<&MasonryCard>> = vec![Vec::new(); n_cols];
        for c in cards {
            let shortest = col_heights
                .iter()
                .enumerate()
                .min_by_key(|&(_, h)| *h)
                .map(|(i, _)| i)
                .unwrap_or(0);
            let weight = 50 + c.body.len() + if c.metric.is_some() { 35 } else { 0 };
            col_heights[shortest] += weight;
            buckets[shortest].push(c);
        }

        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, gap: 12.0, align: Align::Start, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );
        for col_cards in buckets {
            self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 12.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            for card in col_cards {
                let r_card = self.look().card.radius;
                let bg_c = self.color(Token::Surface);
                let bd_c = self.color(Token::BorderSubtle);
                self.begin(
                    Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(14.0), gap: 6.0, ..Default::default() },
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
                );
                self.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                self.badge(&card.tag, card.tone);
                if let Some(ref m) = card.metric {
                    self.begin(Style { width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                    self.end();
                    let mut st_m = self.text_style(TextRole::Mono);
                    st_m.tabular = true;
                    st_m.weight = 600;
                    self.text_leaf(m.clone(), st_m, self.color(Token::Foreground), false, false, Style::default());
                }
                self.end();

                self.text_leaf(card.title.clone(), self.text_style(TextRole::BodyStrong), self.color(Token::Foreground), true, false, Style { width: Size::FILL, ..Default::default() });
                self.text_leaf(card.body.clone(), self.text_style(TextRole::Caption), self.color(Token::TextSecondary), true, false, Style { width: Size::FILL, ..Default::default() });
                self.end();
            }
            self.end();
        }
        self.end();

        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Surface card with an interactive header and spring-rotating chevron that
    /// expands or collapses its body cleanly.
    pub fn collapsible_card(
        &mut self,
        key: impl std::hash::Hash,
        title: &str,
        badge_text: Option<&str>,
        open: &mut bool,
        body: impl FnOnce(&mut Ui),
    ) -> Response {
        let id = self.make_id(("collapsible_card", key));
        let hdr_id = id.with("hdr");
        let ix = self.interact(hdr_id, Sense::CLICK, false);
        if ix.clicked {
            *open = !*open;
        }
        let mo = self.motion();
        let t = self.spring(id, 0, if *open { 1.0 } else { 0.0 }, mo.snappy).clamp(0.0, 1.0);
        let hov = self.fade(id, 1, ix.hovered, mo.hover_in, mo.hover_out, Ease::Out);

        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let hov_bg = self.color(Token::Hover).multiply_alpha(hov);
        let is_open = *open;

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        let top_rad = (r_card - 1.0).max(0.0);
        let hdr_corners = if is_open { Corners([top_rad, top_rad, 0.0, 0.0]) } else { Corners::all(top_rad) };
        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(16.0, 12.0), gap: 10.0, align: Align::Center, ..Default::default() },
            Some(hdr_id),
            Sense::CLICK,
            CursorIcon::Pointer,
            false,
            Some(Box::new(move |cx, r| {
                if hov > 0.005 {
                    cx.painter.rrect(r.expand(-1.0), hdr_corners, hov_bg, 0.0, Color::TRANSPARENT);
                }
                if is_open {
                    cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.max.y - 1.0, (r.width() - 2.0).max(0.0), 1.0), bd_c);
                }
            })),
        );

        // Rotating chevron drawn via polyline so `0° → 90°` interpolates smoothly!
        let chev_c = self.color(Token::TextSecondary);
        self.leaf(
            Style::default(),
            vec2(16.0, 16.0),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let c = r.center();
                let ang = t * std::f32::consts::FRAC_PI_2;
                let rot = |p: crate::math::Vec2| vec2(p.x * ang.cos() - p.y * ang.sin(), p.x * ang.sin() + p.y * ang.cos());
                let p0 = c + rot(vec2(-2.5, -4.5));
                let p1 = c + rot(vec2(2.5, 0.0));
                let p2 = c + rot(vec2(-2.5, 4.5));
                cx.painter.polyline(&[p0, p1, p2], 1.8, chev_c);
            })),
        );

        self.text_leaf(title.to_owned(), self.text_style(TextRole::BodyStrong), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
        if let Some(b) = badge_text {
            self.badge(b, Tone::Neutral);
        }
        self.end();

        if *open {
            self.begin(Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(16.0), gap: 10.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.scope("cc_body", body);
            self.end();
        }
        self.end();

        let rect = self.rect_of(id).unwrap_or(ix.rect);
        Response { id, rect, hovered: ix.hovered, clicked: ix.clicked, changed: ix.clicked, focused: ix.focused }
    }

    /// Seamless horizontal capability / status marquee ticker that pauses on
    /// hover and sleeps with zero wakeups when `playing == false` or reduced
    /// motion is enabled.
    pub fn marquee_strip(&mut self, key: impl std::hash::Hash, items: &[(&str, &str)], playing: bool) -> Response {
        let id = self.make_id(("marquee", key));
        let ix = self.interact(id, Sense::HOVER, false);
        let active_anim = playing && !ix.hovered && !self.reduced_motion();

        #[derive(Default, Clone, Copy)]
        struct MarqueeMem {
            offset: f32,
            last_t: Option<f64>,
        }
        let now = self.time();
        let mut mem = *self.mem::<MarqueeMem>(id);
        if active_anim {
            if let Some(prev) = mem.last_t {
                let dt = (now - prev).clamp(0.0, 0.1) as f32;
                mem.offset += dt * 42.0;
            }
            mem.last_t = Some(now);
            self.request_repaint_after(0.016);
        } else {
            mem.last_t = None;
        }
        *self.mem::<MarqueeMem>(id) = mem;

        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let chip_bg = self.color(Token::SurfaceMuted);
        let chip_bd = self.color(Token::BorderSubtle);
        let ic_c = self.color(Token::AccentStrong);
        let fg_c = self.color(Token::Foreground);
        let st_lbl = self.text_style(TextRole::Label);
        let items_vec: Vec<(String, String)> = items.iter().map(|&(ic, t)| (ic.to_owned(), t.to_owned())).collect();
        let off = mem.offset;

        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(320.0, 54.0),
            Some(id),
            Sense::HOVER,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_card), bg_c, 0.0, Color::TRANSPARENT);
                cx.painter.push_clip(r.expand(-1.0));

                if !items_vec.is_empty() {
                    let gap = 10.0;
                    let mut widths = Vec::with_capacity(items_vec.len());
                    let mut cycle_w = 0.0f32;
                    for (_, txt) in &items_vec {
                        let g = cx.galley(txt, &st_lbl);
                        let cw = 34.0 + g.width + 14.0;
                        widths.push((cw, g));
                        cycle_w += cw + gap;
                    }
                    let cycle = cycle_w.max(1.0);
                    let shift = off.rem_euclid(cycle);
                    let mut start_x = r.min.x + 12.0 - shift;
                    while start_x < r.max.x {
                        let mut x = start_x;
                        for ((ic, _), (cw, g)) in items_vec.iter().zip(widths.iter()) {
                            let cr = Rect::from_xywh(x, r.center().y - 16.0, *cw, 32.0);
                            if cr.max.x >= r.min.x && cr.min.x <= r.max.x {
                                cx.painter.rrect(cr, Corners::all(16.0), chip_bg, 1.0, chip_bd);
                                cx.icon(ic, vec2(cr.min.x + 17.0, cr.center().y), 14.0, ic_c);
                                cx.painter.text(cx.text, g, vec2(cr.min.x + 31.0, cr.center().y - g.height * 0.5), fg_c);
                            }
                            x += *cw + gap;
                        }
                        start_x += cycle;
                    }

                    // Smooth 16-slice non-overlapping edge fade on left and right, respecting rounded corners!
                    let slices = 16usize;
                    let slice_w = 2.0f32;
                    for s in 0..slices {
                        let t = (s as f32 + 0.5) / slices as f32;
                        let alpha = (1.0 - t).powi(2) * 0.96;
                        let d = s as f32 * slice_w;
                        let inset_y = if d < r_card {
                            let dx = r_card - (d + 0.5).min(r_card);
                            (r_card - (r_card * r_card - dx * dx).max(0.0).sqrt()).max(1.0)
                        } else {
                            1.0
                        };
                        let h = (r.height() - inset_y * 2.0).max(0.0);
                        cx.painter.rect(Rect::from_xywh(r.min.x + d, r.min.y + inset_y, slice_w, h), bg_c.with_alpha(alpha));
                        cx.painter.rect(Rect::from_xywh(r.max.x - d - slice_w, r.min.y + inset_y, slice_w, h), bg_c.with_alpha(alpha));
                    }
                }

                cx.painter.pop_clip();
                cx.painter.ring(r, Corners::all(r_card), 1.0, bd_c);
            })),
        );
        Response { id, rect: ix.rect, hovered: ix.hovered, ..Default::default() }
    }
}
