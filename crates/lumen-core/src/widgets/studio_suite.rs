//! Diverging win/loss bar charts, 10×10 waffle proportion grids, database
//! schema cards, guarded environment banners, GPU memory arena maps, AI patch
//! review cards, starter prompt templates, pricing cards, A/B experiment cards
//! and interactive 8-band studio graphic equalizers.

use super::toast::Tone;
use crate::anim::Ease;
use crate::color::Color;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{Shadow, TextRole, Token};
use crate::ui::{Response, Sense, Ui};

// =========================================================== DivergingBars

/// Signed delta entry in [`Ui::diverging_bars`].
#[derive(Clone, Debug)]
pub struct DivergingBar {
    pub label: String,
    pub delta: f32,
    pub unit: String,
}

impl DivergingBar {
    pub fn new(label: impl Into<String>, delta: f32, unit: &str) -> Self {
        DivergingBar { label: label.into(), delta, unit: unit.to_owned() }
    }
}

// ============================================================= WaffleChart

/// Proportion slice in [`Ui::waffle_chart`] (100 cells total across a 10×10 grid).
#[derive(Clone, Debug)]
pub struct WaffleSlice {
    pub label: String,
    pub cells: u8,
    pub tone: Tone,
}

impl WaffleSlice {
    pub fn new(label: impl Into<String>, cells: u8, tone: Tone) -> Self {
        WaffleSlice { label: label.into(), cells, tone }
    }
}

// ============================================================== SchemaCard

/// Column / field row in [`Ui::schema_card`].
#[derive(Clone, Debug)]
pub struct SchemaField {
    pub name: String,
    pub ty: String,
    pub key_kind: Option<String>,
    pub nullable: bool,
}

impl SchemaField {
    pub fn new(name: impl Into<String>, ty: impl Into<String>) -> Self {
        SchemaField { name: name.into(), ty: ty.into(), key_kind: None, nullable: false }
    }

    pub fn pk(mut self) -> Self {
        self.key_kind = Some("PK".to_owned());
        self
    }

    pub fn fk(mut self) -> Self {
        self.key_kind = Some("FK".to_owned());
        self
    }

    pub fn indexed(mut self) -> Self {
        self.key_kind = Some("IDX".to_owned());
        self
    }

    pub fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }
}

// ============================================================= MemoryArena

/// Allocator / GPU slab entry in [`Ui::memory_arena`].
#[derive(Clone, Debug)]
pub struct ArenaSlab {
    pub name: String,
    pub used_mb: f32,
    pub cap_mb: f32,
}

impl ArenaSlab {
    pub fn new(name: impl Into<String>, used_mb: f32, cap_mb: f32) -> Self {
        ArenaSlab { name: name.into(), used_mb: used_mb.max(0.0), cap_mb: cap_mb.max(0.1) }
    }
}

// ========================================================= CodeSuggestion

/// Review decision state for [`Ui::code_suggestion`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SuggestionDecision {
    #[default]
    Pending,
    Accepted,
    Dismissed,
}

// ======================================================== PromptTemplates

/// Starter prompt card in [`Ui::prompt_templates`].
#[derive(Clone, Debug)]
pub struct PromptTemplate {
    pub icon: String,
    pub title: String,
    pub prompt: String,
    pub category: String,
}

impl PromptTemplate {
    pub fn new(icon: &str, title: impl Into<String>, category: impl Into<String>, prompt: impl Into<String>) -> Self {
        PromptTemplate {
            icon: icon.to_owned(),
            title: title.into(),
            prompt: prompt.into(),
            category: category.into(),
        }
    }
}

// ============================================================ PricingCards

/// Plan tier specification for [`Ui::pricing_cards`].
#[derive(Clone, Debug)]
pub struct PlanSpec {
    pub name: String,
    pub monthly_usd: u32,
    pub annual_usd: u32,
    pub blurb: String,
    pub features: Vec<String>,
    pub popular: bool,
}

impl PlanSpec {
    pub fn new(name: impl Into<String>, monthly_usd: u32, annual_usd: u32, blurb: impl Into<String>, features: &[&str]) -> Self {
        PlanSpec {
            name: name.into(),
            monthly_usd,
            annual_usd,
            blurb: blurb.into(),
            features: features.iter().map(|&f| f.to_owned()).collect(),
            popular: false,
        }
    }

    pub fn popular(mut self) -> Self {
        self.popular = true;
        self
    }
}

impl Ui {
    /// Zero-baseline diverging positive/negative horizontal bar chart for
    /// benchmark speedups vs regressions or win/loss deltas.
    pub fn diverging_bars(
        &mut self,
        key: impl std::hash::Hash,
        title: &str,
        items: &[DivergingBar],
        max_abs: f32,
    ) -> Response {
        let id = self.make_id(("diverging_bars", key));
        let mo = self.motion();
        let enter = self.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);
        let bound = max_abs.abs().max(1.0);

        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(16.0), gap: 10.0, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        if !title.is_empty() {
            self.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.text_leaf(title.to_owned(), self.text_style(TextRole::BodyStrong), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
            self.text_leaf("0 baseline", self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, false, Style::default());
            self.end();
        }

        for (bi, item) in items.iter().enumerate() {
            let rid = id.with(("db", bi));
            let ix = self.interact(rid, Sense::HOVER, false);
            let hov = self.fade(rid, 0, ix.hovered, mo.hover_in, mo.hover_out, Ease::Out);
            let h_bg = self.color(Token::Hover).multiply_alpha(hov);

            self.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(6.0, 4.0), gap: 10.0, align: Align::Center, ..Default::default() },
                Some(rid),
                Sense::HOVER,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| {
                    if hov > 0.005 {
                        cx.painter.rrect(r, Corners::all(6.0), h_bg, 0.0, Color::TRANSPARENT);
                    }
                })),
            );

            self.text_leaf(item.label.clone(), self.text_style(TextRole::Mono), self.color(Token::Foreground), false, true, Style { width: Size::Fixed(128.0), ..Default::default() });

            let frac = (item.delta / bound).clamp(-1.0, 1.0) * enter;
            let tr_c = self.color(Token::SurfaceMuted);
            let ax_c = self.color(Token::BorderStrong);
            let bar_c = self.color(if item.delta >= 0.0 { Token::Success } else { Token::Danger });

            self.leaf(
                Style { width: Size::FILL, ..Default::default() },
                vec2(180.0, 18.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    cx.painter.rrect(r, Corners::all(5.0), tr_c, 0.0, Color::TRANSPARENT);
                    let mid_x = r.center().x;
                    let half = r.width() * 0.5;
                    if frac >= 0.0 {
                        let w = (frac * half).max(2.0);
                        cx.painter.rrect(Rect::from_xywh(mid_x, r.min.y + 3.0, w, r.height() - 6.0), Corners([0.0, 4.0, 4.0, 0.0]), bar_c, 0.0, Color::TRANSPARENT);
                    } else {
                        let w = ((-frac) * half).max(2.0);
                        cx.painter.rrect(Rect::from_xywh(mid_x - w, r.min.y + 3.0, w, r.height() - 6.0), Corners([4.0, 0.0, 0.0, 4.0]), bar_c, 0.0, Color::TRANSPARENT);
                    }
                    cx.painter.rect(Rect::from_xywh(mid_x - 0.75, r.min.y, 1.5, r.height()), ax_c);
                })),
            );

            let mut st_v = self.text_style(TextRole::Mono);
            st_v.tabular = true;
            st_v.weight = 600;
            let sign = if item.delta >= 0.0 { "+" } else { "" };
            let val_c = self.color(if item.delta >= 0.0 { Token::SuccessStrong } else { Token::DangerStrong });
            self.begin(Style { width: Size::Fixed(72.0), dir: Dir::Row, justify: Justify::End, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.text_leaf(format!("{sign}{:.1}{}", item.delta, item.unit), st_v, val_c, false, false, Style::default());
            self.end();
            self.end();
        }
        self.end();

        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// 10×10 100-cell rounded-square proportion waffle chart with staggered
    /// entrance and right-hand breakdown legend.
    pub fn waffle_chart(&mut self, key: impl std::hash::Hash, title: &str, slices: &[WaffleSlice]) -> Response {
        let id = self.make_id(("waffle_chart", key));
        let mo = self.motion();
        let enter = self.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);

        let tone_col = |ui: &Ui, t: Tone| -> Color {
            match t {
                Tone::Accent => ui.color(Token::Series0),
                Tone::Success => ui.color(Token::Success),
                Tone::Warning => ui.color(Token::Warning),
                Tone::Danger => ui.color(Token::Danger),
                Tone::Info => ui.color(Token::Series3),
                Tone::Neutral => ui.color(Token::BorderStrong),
            }
        };

        let mut cells = [self.color(Token::Track); 100];
        let mut cursor = 0usize;
        for sl in slices {
            let col = tone_col(self, sl.tone);
            for _ in 0..sl.cells {
                if cursor < 100 {
                    cells[cursor] = col;
                    cursor += 1;
                }
            }
        }

        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);

        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::all(16.0), gap: 20.0, align: Align::Center, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        // 10x10 waffle matrix (138x138 px).
        self.leaf(
            Style::default(),
            vec2(138.0, 138.0),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let cell_sz = 11.0f32;
                let gap = 3.0f32;
                for row in 0..10usize {
                    for col in 0..10usize {
                        let idx = (9 - row) * 10 + col;
                        let cr = Rect::from_xywh(r.min.x + col as f32 * (cell_sz + gap), r.min.y + row as f32 * (cell_sz + gap), cell_sz, cell_sz);
                        let sc = enter.clamp(0.2, 1.0);
                        cx.painter.rrect(Rect::from_center_size(cr.center(), cr.size() * sc), Corners::all(3.0), cells[idx], 0.0, Color::TRANSPARENT);
                    }
                }
            })),
        );

        // Breakdown legend column.
        self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 8.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf(title.to_owned(), self.text_style(TextRole::BodyStrong), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
        for sl in slices {
            let sw_c = tone_col(self, sl.tone);
            self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.leaf(
                Style::default(),
                vec2(10.0, 10.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(3.0), sw_c, 0.0, Color::TRANSPARENT))),
            );
            self.text_leaf(sl.label.clone(), self.text_style(TextRole::Caption), self.color(Token::TextSecondary), false, true, Style { width: Size::FILL, ..Default::default() });
            let mut st_p = self.text_style(TextRole::Mono);
            st_p.size = 12.0;
            st_p.tabular = true;
            st_p.weight = 600;
            self.text_leaf(format!("{}%", sl.cells), st_p, self.color(Token::Foreground), false, false, Style::default());
            self.end();
        }
        self.end();
        self.end();

        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Database table / Rust struct schema inspector card with key badges
    /// (`PK`, `FK`, `IDX`), monospace field names, and column types.
    pub fn schema_card(&mut self, key: impl std::hash::Hash, table_name: &str, fields: &[SchemaField]) -> Response {
        let id = self.make_id(("schema_card", key));
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

        // Header with database icon + table name + column count.
        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(14.0, 9.0), gap: 8.0, align: Align::Center, ..Default::default() },
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
        let ic_c = self.color(Token::AccentStrong);
        self.leaf(Style::default(), vec2(16.0, 16.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.icon("database", r.center(), 14.0, ic_c))));
        let mut st_tbl = self.text_style(TextRole::Mono);
        st_tbl.weight = 600;
        self.text_leaf(table_name.to_owned(), st_tbl, self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
        self.badge(format!("{} cols", fields.len()), Tone::Neutral);
        self.end();

        for (fi, f) in fields.iter().enumerate() {
            let not_last = fi + 1 < fields.len();
            self.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(14.0, 7.5), gap: 8.0, align: Align::Center, ..Default::default() },
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

            self.begin(Style { width: Size::Fixed(42.0), dir: Dir::Row, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            if let Some(ref k) = f.key_kind {
                let kt = match k.as_str() {
                    "PK" => Tone::Warning,
                    "FK" => Tone::Info,
                    _ => Tone::Neutral,
                };
                self.badge(k, kt);
            }
            self.end();

            self.text_leaf(f.name.clone(), self.text_style(TextRole::Mono), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
            if f.nullable {
                self.text_leaf("nullable", self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, false, Style::default());
            }
            self.text_leaf(f.ty.clone(), self.text_style(TextRole::Mono), self.color(Token::AccentStrong), false, false, Style::default());
            self.end();
        }
        self.end();

        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Environment selector & safety guard (`Development | Staging | Production`)
    /// that morphs its status tone and exposes a write-protection switch.
    pub fn env_banner(
        &mut self,
        key: impl std::hash::Hash,
        env_idx: &mut usize,
        region: &str,
        write_locked: &mut bool,
    ) -> bool {
        let id = self.make_id(("env_banner", key));
        *env_idx = (*env_idx).min(2);
        let mo = self.motion();
        let mut changed = false;

        let prod_t = self.spring(id, 0, if *env_idx == 2 { 1.0 } else { 0.0 }, mo.snappy).clamp(0.0, 1.0);
        let stg_t = self.spring(id, 1, if *env_idx == 1 { 1.0 } else { 0.0 }, mo.snappy).clamp(0.0, 1.0);

        let r_card = self.look().card.radius;
        let bg_c = self
            .color(Token::Surface)
            .mix(self.color(Token::WarningSubtle).multiply_alpha(0.35), stg_t)
            .mix(self.color(Token::DangerSubtle).multiply_alpha(0.38), prod_t);
        let bd_c = self
            .color(Token::BorderSubtle)
            .mix(self.color(Token::Warning), stg_t)
            .mix(self.color(Token::Danger), prod_t);

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(16.0), gap: 12.0, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0 + 0.5 * prod_t.max(stg_t), bd_c))),
        );

        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 10.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if self.scope("eb_seg", |ui| ui.segmented(env_idx, &[(0usize, "Development"), (1, "Staging"), (2, "Production")]).changed()) {
            if *env_idx == 2 {
                *write_locked = true;
            }
            changed = true;
        }
        self.begin(Style { width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.end();
        let (badge_txt, tone) = match *env_idx {
            0 => ("Sandbox", Tone::Success),
            1 => ("Pre-flight", Tone::Warning),
            _ => ("Live traffic", Tone::Danger),
        };
        self.badge(badge_txt, tone);
        self.end();

        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 10.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let desc = match *env_idx {
            0 => format!("{region} · Ephemeral local branch database"),
            1 => format!("{region} · Mirrors production schema (synthetic traffic)"),
            _ => format!("{region} · Customer-facing cluster — mutations audited"),
        };
        self.text_leaf(desc, self.text_style(TextRole::Caption), self.color(Token::TextSecondary), false, true, Style { width: Size::FILL, ..Default::default() });
        if self.scope("eb_lock", |ui| ui.switch("Write lock", write_locked).changed()) {
            changed = true;
        }
        self.end();

        self.end();
        changed
    }

    /// GPU / allocator memory arena utilization view showing unified slab blocks,
    /// peak watermark, and per-slab capacity bars.
    pub fn memory_arena(&mut self, key: impl std::hash::Hash, slabs: &[ArenaSlab], peak_mb: f32) -> Response {
        let id = self.make_id(("memory_arena", key));
        let mo = self.motion();
        let enter = self.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);

        let tot_used: f32 = slabs.iter().map(|s| s.used_mb).sum();
        let tot_cap: f32 = slabs.iter().map(|s| s.cap_mb).sum::<f32>().max(1.0);
        let palette = [
            self.color(Token::Series0),
            self.color(Token::Series2),
            self.color(Token::Series1),
            self.color(Token::Series3),
        ];

        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(16.0), gap: 12.0, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        // Header summary.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf("GPU Frame Arena", self.text_style(TextRole::BodyStrong), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
        let mut st_m = self.text_style(TextRole::Mono);
        st_m.size = 11.5;
        st_m.tabular = true;
        self.text_leaf(
            format!("{tot_used:.1} / {tot_cap:.0} MiB  (peak {peak_mb:.1} MiB)"),
            st_m,
            self.color(Token::TextSecondary),
            false,
            false,
            Style::default(),
        );
        self.end();

        // Unified slab bar with peak watermark tick.
        let segs: Vec<(f32, Color)> = slabs
            .iter()
            .enumerate()
            .map(|(i, s)| ((s.used_mb / tot_cap).clamp(0.0, 1.0) * enter, palette[i % palette.len()]))
            .collect();
        let pk_frac = (peak_mb / tot_cap).clamp(0.0, 1.0);
        let tr_c = self.color(Token::Track);
        let pk_c = self.color(Token::WarningStrong);

        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(260.0, 18.0),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(6.0), tr_c, 0.0, Color::TRANSPARENT);
                let mut x = r.min.x;
                for &(f, col) in &segs {
                    let w = r.width() * f;
                    if w > 1.5 {
                        cx.painter.rrect(Rect::from_xywh(x, r.min.y, (w - 1.5).max(1.5), r.height()), Corners::all(4.0), col, 0.0, Color::TRANSPARENT);
                        x += w;
                    }
                }
                let px = r.min.x + r.width() * pk_frac;
                cx.painter.rrect(Rect::from_xywh(px - 1.0, r.min.y - 2.0, 2.5, r.height() + 4.0), Corners::all(1.0), pk_c, 0.0, Color::TRANSPARENT);
            })),
        );

        // Per-slab breakdown rows.
        for (si, slab) in slabs.iter().enumerate() {
            let col = palette[si % palette.len()];
            let frac = (slab.used_mb / slab.cap_mb).clamp(0.0, 1.0) * enter;
            let tr_s = self.color(Token::SurfaceMuted);
            self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 10.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.leaf(Style::default(), vec2(8.0, 8.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.circle(r.center(), 4.0, col, 0.0, Color::TRANSPARENT))));
            self.text_leaf(slab.name.clone(), self.text_style(TextRole::Caption), self.color(Token::Foreground), false, true, Style { width: Size::Fixed(112.0), ..Default::default() });
            self.leaf(
                Style { width: Size::FILL, ..Default::default() },
                vec2(100.0, 6.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    cx.painter.rrect(r, Corners::all(3.0), tr_s, 0.0, Color::TRANSPARENT);
                    cx.painter.rrect(Rect::from_xywh(r.min.x, r.min.y, (r.width() * frac).max(3.0), r.height()), Corners::all(3.0), col, 0.0, Color::TRANSPARENT);
                })),
            );
            self.begin(Style { width: Size::Fixed(86.0), dir: Dir::Row, justify: Justify::End, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.text_leaf(format!("{:.1}/{:.0}M", slab.used_mb, slab.cap_mb), st_m, self.color(Token::TextMuted), false, false, Style::default());
            self.end();
            self.end();
        }
        self.end();

        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Inline AI patch review card showing target file, rationale, `-` / `+`
    /// diff lines, and `"Accept patch"` / `"Dismiss"` review actions.
    pub fn code_suggestion(
        &mut self,
        key: impl std::hash::Hash,
        file_path: &str,
        rationale: &str,
        removed: &[&str],
        added: &[&str],
        decision: &mut SuggestionDecision,
    ) -> bool {
        let id = self.make_id(("code_suggestion", key));
        let r_card = self.look().card.radius;
        let top_rad = (r_card - 1.0).max(0.0);
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(match *decision {
            SuggestionDecision::Pending => Token::Border,
            SuggestionDecision::Accepted => Token::Success,
            SuggestionDecision::Dismissed => Token::BorderSubtle,
        });
        let hdr_bg = self.color(Token::AccentSubtle).multiply_alpha(0.35);
        let mut changed = false;

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
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(14.0, 9.0), gap: 8.0, align: Align::Center, ..Default::default() },
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
        let sp_c = self.color(Token::AccentStrong);
        self.leaf(Style::default(), vec2(16.0, 16.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.icon("sparkles", r.center(), 14.0, sp_c))));
        self.text_leaf(file_path.to_owned(), self.text_style(TextRole::Mono), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
        match *decision {
            SuggestionDecision::Pending => {
                self.badge("AI Patch", Tone::Accent);
            }
            SuggestionDecision::Accepted => {
                self.badge("Applied", Tone::Success);
            }
            SuggestionDecision::Dismissed => {
                self.badge("Dismissed", Tone::Neutral);
            }
        }
        self.end();

        // Body: rationale + diff lines + actions.
        self.begin(Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(14.0), gap: 10.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf(rationale.to_owned(), self.text_style(TextRole::Caption), self.color(Token::TextSecondary), true, false, Style { width: Size::FILL, ..Default::default() });

        // Compact inline diff block.
        let code_bg = self.color(Token::SurfaceMuted);
        let code_bd = self.color(Token::BorderSubtle);
        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, pad: Sides::xy(10.0, 8.0), gap: 3.0, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(8.0), code_bg, 1.0, code_bd))),
        );
        let mut st_code = self.text_style(TextRole::Mono);
        st_code.size = 12.0;
        let del_bg = self.color(Token::DangerSubtle).multiply_alpha(0.55);
        let del_fg = self.color(Token::DangerStrong);
        for &line in removed {
            self.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(6.0, 2.0), ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(4.0), del_bg, 0.0, Color::TRANSPARENT))),
            );
            self.text_leaf(format!("- {line}"), st_code, del_fg, false, true, Style { width: Size::FILL, ..Default::default() });
            self.end();
        }
        let add_bg = self.color(Token::SuccessSubtle).multiply_alpha(0.55);
        let add_fg = self.color(Token::SuccessStrong);
        for &line in added {
            self.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(6.0, 2.0), ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(4.0), add_bg, 0.0, Color::TRANSPARENT))),
            );
            self.text_leaf(format!("+ {line}"), st_code, add_fg, false, true, Style { width: Size::FILL, ..Default::default() });
            self.end();
        }
        self.end();

        // Action row.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, justify: Justify::End, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if *decision == SuggestionDecision::Pending {
            if self.scope("cs_dis", |ui| ui.button("Dismiss").ghost().small().clicked()) {
                *decision = SuggestionDecision::Dismissed;
                changed = true;
            }
            if self.scope("cs_acc", |ui| ui.button("Accept patch").primary().small().icon("check").clicked()) {
                *decision = SuggestionDecision::Accepted;
                changed = true;
            }
        } else if self.scope("cs_undo", |ui| ui.button("Reset review").ghost().small().clicked()) {
            *decision = SuggestionDecision::Pending;
            changed = true;
        }
        self.end();

        self.end();
        self.end();
        changed
    }

    /// 2-column grid of clickable AI starter prompt cards highlighting
    /// `{{variable}}` placeholders and category pills.
    pub fn prompt_templates(&mut self, key: impl std::hash::Hash, templates: &[PromptTemplate]) -> Option<usize> {
        let id = self.make_id(("prompt_templates", key));
        let mo = self.motion();
        let mut chosen = None;

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, gap: 10.0, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );

        let mut idx = 0usize;
        while idx < templates.len() {
            let row_end = (idx + 2).min(templates.len());
            self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 10.0, align: Align::Stretch, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            for ti in idx..row_end {
                let tpl = &templates[ti];
                let cid = id.with(("pt", ti));
                let ix = self.interact(cid, Sense::CLICK, false);
                if ix.clicked {
                    chosen = Some(ti);
                }
                let sq = self.spring(cid, 0, if ix.pressed { 0.97 } else { 1.0 }, mo.snappy);
                let hov = self.fade(cid, 1, ix.hovered, mo.hover_in, mo.hover_out, Ease::Out);
                let r_card = (self.look().card.radius - 2.0).max(10.0);
                let bg_c = self.color(Token::Surface).mix(self.color(Token::Hover), hov);
                let bd_c = self.color(Token::BorderSubtle).mix(self.color(Token::Accent), hov * 0.65);

                self.begin(
                    Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(13.0), gap: 7.0, ..Default::default() },
                    Some(cid),
                    Sense::CLICK,
                    CursorIcon::Pointer,
                    false,
                    Some(Box::new(move |cx, r| {
                        let dr = Rect::from_center_size(r.center(), r.size() * sq);
                        cx.painter.rrect(dr, Corners::all(r_card), bg_c, 1.0, bd_c);
                    })),
                );

                self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                let ic_c = self.color(Token::AccentStrong);
                let ic_n = tpl.icon.clone();
                self.leaf(Style::default(), vec2(16.0, 16.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.icon(&ic_n, r.center(), 14.5, ic_c))));
                self.text_leaf(tpl.title.clone(), self.text_style(TextRole::BodyStrong), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
                self.badge(&tpl.category, Tone::Neutral);
                self.end();

                self.text_leaf(tpl.prompt.clone(), self.text_style(TextRole::Caption), self.color(Token::TextSecondary), true, false, Style { width: Size::FILL, ..Default::default() });
                self.end();
            }
            self.end();
            idx = row_end;
        }
        self.end();

        chosen
    }

    /// SaaS / compute plan comparison cards with Monthly / Annual (`-20%`)
    /// billing toggle, rolling dollar figures, `Popular` highlight, and CTAs.
    pub fn pricing_cards(
        &mut self,
        key: impl std::hash::Hash,
        plans: &[PlanSpec],
        annual: &mut bool,
        selected_plan: &mut usize,
    ) -> bool {
        let id = self.make_id(("pricing_cards", key));
        if !plans.is_empty() {
            *selected_plan = (*selected_plan).min(plans.len() - 1);
        }
        let mut changed = false;

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, gap: 12.0, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );

        // Billing cycle bar.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 10.0, justify: Justify::Center, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if self.scope("pc_bill", |ui| ui.switch("Annual billing", annual).changed()) {
            changed = true;
        }
        self.badge("Save 20%", Tone::Success);
        self.end();

        // Side-by-side plan cards. They share the width, or keep 200 px
        // each and swipe sideways when that doesn't fit (a phone).
        self.scroll_row("plans", Style { width: Size::FILL, gap: 10.0, align: Align::Stretch, ..Default::default() }, |ui| {
            for (pi, plan) in plans.iter().enumerate() {
                let is_sel = pi == *selected_plan;
                let r_card = ui.look().card.radius;
                let bg_c = if plan.popular {
                    ui.color(Token::Surface).mix(ui.color(Token::AccentSubtle), 0.22)
                } else {
                    ui.color(Token::Surface)
                };
                let bd_c = if is_sel || plan.popular { ui.color(Token::Accent) } else { ui.color(Token::BorderSubtle) };
                let bw = if is_sel { 1.6 } else { 1.0 };

                ui.begin(
                    Style { dir: Dir::Column, width: Size::FILL, min: vec2(200.0, 0.0), pad: Sides::all(14.0), gap: 8.0, ..Default::default() },
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, bw, bd_c))),
                );

                ui.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                ui.text_leaf(plan.name.clone(), ui.text_style(TextRole::BodyStrong), ui.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
                if plan.popular {
                    ui.badge("Popular", Tone::Accent);
                }
                ui.end();

                // Rolling price figure!
                let price = if *annual { plan.annual_usd } else { plan.monthly_usd } as i64;
                ui.begin(Style { dir: Dir::Row, gap: 4.0, align: Align::End, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                ui.scope(id.with(("pr_num", pi)), |ui| {
                    ui.rolling_number("usd", price).prefix("$").role(TextRole::Heading1).show();
                });
                ui.text_leaf("/ mo", ui.text_style(TextRole::Caption), ui.color(Token::TextMuted), false, false, Style::default());
                ui.end();

                ui.text_leaf(plan.blurb.clone(), ui.text_style(TextRole::Caption), ui.color(Token::TextSecondary), true, false, Style { width: Size::FILL, ..Default::default() });

                for feat in &plan.features {
                    let ck_c = ui.color(Token::Success);
                    ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                    ui.leaf(Style::default(), vec2(13.0, 13.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.icon("check", r.center(), 12.0, ck_c))));
                    ui.text_leaf(feat.clone(), ui.text_style(TextRole::Caption), ui.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
                    ui.end();
                }

                ui.begin(Style { width: Size::FILL, height: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                ui.end();

                let btn_lbl = if is_sel { "Current plan" } else { "Select plan" };
                let clicked = ui.scope(("pc_cta", pi), |ui| {
                    let b = ui.button(btn_lbl).small().full_width();
                    let b = if plan.popular && !is_sel { b.primary() } else { b };
                    b.clicked()
                });
                if clicked && *selected_plan != pi {
                    *selected_plan = pi;
                    changed = true;
                }
                ui.end();
            }
        });
        self.end();

        changed
    }

    /// Variant A vs Variant B statistical experiment comparison card with
    /// relative lift badge, conversion bars, and confidence meter.
    pub fn ab_comparison(
        &mut self,
        key: impl std::hash::Hash,
        metric_name: &str,
        baseline_label: &str,
        baseline_rate: f32,
        variant_label: &str,
        variant_rate: f32,
        confidence_pct: f32,
    ) -> Response {
        let id = self.make_id(("ab_comparison", key));
        let mo = self.motion();
        let enter = self.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);

        let lift = if baseline_rate.abs() > 1e-4 {
            (variant_rate - baseline_rate) / baseline_rate * 100.0
        } else {
            0.0
        };
        let b_wins = variant_rate >= baseline_rate;
        let max_r = baseline_rate.max(variant_rate).max(1.0) * 1.15;

        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(16.0), gap: 12.0, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        // Header row with lift badge.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf(metric_name.to_owned(), self.text_style(TextRole::BodyStrong), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
        let sign = if lift >= 0.0 { "+" } else { "" };
        self.badge(format!("{sign}{lift:.1}% lift"), if b_wins { Tone::Success } else { Tone::Warning });
        self.end();

        // Variant rows.
        for (lbl, rate, win) in [(baseline_label, baseline_rate, !b_wins), (variant_label, variant_rate, b_wins)] {
            self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 5.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.text_leaf(lbl.to_owned(), self.text_style(TextRole::Label), self.color(Token::Foreground), false, false, Style::default());
            if win {
                self.badge("Leader", Tone::Accent);
            }
            self.begin(Style { width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.end();
            let mut st_r = self.text_style(TextRole::Mono);
            st_r.tabular = true;
            st_r.weight = 600;
            self.text_leaf(format!("{rate:.2}%"), st_r, self.color(Token::Foreground), false, false, Style::default());
            self.end();

            let frac = (rate / max_r).clamp(0.0, 1.0) * enter;
            let tr_c = self.color(Token::Track);
            let fl_c = self.color(if win { Token::Accent } else { Token::BorderStrong });
            self.leaf(
                Style { width: Size::FILL, ..Default::default() },
                vec2(240.0, 10.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    cx.painter.rrect(r, Corners::all(5.0), tr_c, 0.0, Color::TRANSPARENT);
                    cx.painter.rrect(Rect::from_xywh(r.min.x, r.min.y, (r.width() * frac).max(4.0), r.height()), Corners::all(5.0), fl_c, 0.0, Color::TRANSPARENT);
                })),
            );
            self.end();
        }

        // Statistical significance footer.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf("Bayesian confidence", self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, true, Style { width: Size::FILL, ..Default::default() });
        let mut st_c = self.text_style(TextRole::Mono);
        st_c.size = 12.0;
        st_c.tabular = true;
        let sig = confidence_pct >= 95.0;
        self.text_leaf(
            format!("{confidence_pct:.1}% ({})", if sig { "significant" } else { "collecting" }),
            st_c,
            self.color(if sig { Token::SuccessStrong } else { Token::TextSecondary }),
            false,
            false,
            Style::default(),
        );
        self.end();
        self.end();

        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Interactive 8-band studio dB graphic equalizer (`-12..+12 dB`) with
    /// preset curves (`Flat`, `Vocal`, `Warm`, `Air`), smooth frequency
    /// response curve, and draggable vertical faders.
    pub fn graphic_eq(
        &mut self,
        key: impl std::hash::Hash,
        bands_db: &mut [f32; 8],
        preset: &mut usize,
    ) -> bool {
        let id = self.make_id(("graphic_eq", key));
        *preset = (*preset).min(3);
        let mo = self.motion();
        let mut changed = false;

        let presets: [[f32; 8]; 4] = [
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [-2.5, -1.0, 1.5, 4.5, 5.2, 3.0, 1.0, -0.5],
            [5.5, 4.2, 2.0, -0.8, -1.5, 0.5, 2.2, 3.0],
            [-1.5, -0.5, 0.5, 1.8, 3.2, 5.0, 6.8, 7.5],
        ];

        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(16.0), gap: 12.0, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        // Header + Preset selector. Narrow cards (last frame's width) stack
        // the title above the presets instead of cutting it short.
        let narrow = self.rect_of(id).is_some_and(|r| r.width() < 440.0);
        let dir = if narrow { Dir::Column } else { Dir::Row };
        let align = if narrow { Align::Start } else { Align::Center };
        self.begin(Style { dir, width: Size::FILL, gap: 10.0, align, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf("8-Band Parametric EQ", self.text_style(TextRole::BodyStrong), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
        if self.scope("eq_pre", |ui| ui.segmented(preset, &[(0usize, "Flat"), (1, "Vocal"), (2, "Warm"), (3, "Air")]).changed()) {
            *bands_db = presets[*preset];
            changed = true;
        }
        self.end();

        // Spring-smoothed band values (`-12..=+12 dB`).
        let mut anim_db = [0.0f32; 8];
        for i in 0..8 {
            bands_db[i] = bands_db[i].clamp(-12.0, 12.0);
            anim_db[i] = self.spring(id, i as u32, bands_db[i], mo.snappy).clamp(-12.0, 12.0);
        }

        // Smooth frequency response curve preview strip!
        let crv_bg = self.color(Token::SurfaceMuted);
        let zero_c = self.color(Token::Border);
        let crv_c = self.color(Token::Accent);
        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(320.0, 52.0),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(8.0), crv_bg, 0.0, Color::TRANSPARENT);
                let mid_y = r.center().y;
                cx.painter.rect(Rect::from_xywh(r.min.x + 8.0, mid_y - 0.5, (r.width() - 16.0).max(0.0), 1.0), zero_c);

                let samples = 36usize;
                let mut upper = Vec::with_capacity(samples);
                let mut lower = Vec::with_capacity(samples);
                for s in 0..samples {
                    let u = s as f32 / (samples - 1) as f32;
                    let pos = u * 7.0;
                    let i0 = (pos.floor() as usize).min(6);
                    let f = pos - i0 as f32;
                    let smooth_f = f * f * (3.0 - 2.0 * f);
                    let db = anim_db[i0] + (anim_db[i0 + 1] - anim_db[i0]) * smooth_f;
                    let x = r.min.x + 16.0 + u * (r.width() - 32.0);
                    let y = mid_y - (db / 12.0) * (r.height() * 0.40);
                    upper.push(vec2(x, y));
                    lower.push(vec2(x, mid_y));
                }
                cx.painter.fill_between(&upper, &lower, crv_c.with_alpha(0.22), crv_c.with_alpha(0.02));
                cx.painter.polyline(&upper, 1.8, crv_c);
            })),
        );

        // 8 draggable vertical faders!
        let freqs = ["60", "150", "400", "1k", "2.4k", "6k", "12k", "16k"];
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for bi in 0..8usize {
            let fid = id.with(("fader", bi));
            let ix = self.interact(fid, Sense::DRAG, false);
            if ix.held {
                if let (Some(ptr), Some(fr)) = (ix.pointer, self.rect_of(fid)) {
                    let top = fr.min.y + 10.0;
                    let bot = fr.max.y - 22.0;
                    let t = ((bot - ptr.y) / (bot - top).max(1.0)).clamp(0.0, 1.0);
                    let ndb = -12.0 + t * 24.0;
                    if (ndb - bands_db[bi]).abs() > 0.05 {
                        bands_db[bi] = ndb;
                        changed = true;
                    }
                }
            }
            if ix.focused {
                if self.key_pressed_for(fid, Key::ArrowUp) {
                    bands_db[bi] = (bands_db[bi] + 1.5).min(12.0);
                    changed = true;
                } else if self.key_pressed_for(fid, Key::ArrowDown) {
                    bands_db[bi] = (bands_db[bi] - 1.5).max(-12.0);
                    changed = true;
                }
            }

            let db_v = anim_db[bi];
            let tr_c = self.color(Token::Track);
            let act_c = self.color(Token::Accent);
            let th_bg = self.color(Token::Knob);
            let th_bd = self.color(Token::Border);
            let sh_c = self.color(Token::ShadowInk);
            let mut st_f = self.text_style(TextRole::Mono);
            st_f.size = 10.5;
            let f_lbl = freqs[bi];
            let fg_m = self.color(Token::TextMuted);

            self.leaf(
                Style { width: Size::FILL, ..Default::default() },
                vec2(34.0, 118.0),
                Some(fid),
                Sense::DRAG,
                if ix.held { CursorIcon::Grabbing } else { CursorIcon::Grab },
                Some(Box::new(move |cx, r| {
                    let cx_x = r.center().x;
                    let top = r.min.y + 10.0;
                    let bot = r.max.y - 22.0;
                    let mid = (top + bot) * 0.5;
                    let tr_r = Rect::from_xywh(cx_x - 2.5, top, 5.0, bot - top);
                    cx.painter.rrect(tr_r, Corners::all(2.5), tr_c, 0.0, Color::TRANSPARENT);

                    let knob_y = mid - (db_v / 12.0) * ((bot - top) * 0.5);
                    let y0 = mid.min(knob_y);
                    let y1 = mid.max(knob_y);
                    if y1 - y0 > 1.0 {
                        cx.painter.rrect(Rect::from_xywh(cx_x - 2.5, y0, 5.0, y1 - y0), Corners::all(2.5), act_c, 0.0, Color::TRANSPARENT);
                    }

                    let kr = Rect::from_center_size(vec2(cx_x, knob_y), vec2(18.0, 11.0));
                    cx.painter.shadow(kr, Corners::all(4.0), &Shadow { x: 0.0, y: 1.5, blur: 4.0, spread: -1.0, color: sh_c.multiply_alpha(0.20) });
                    cx.painter.rrect(kr, Corners::all(4.0), th_bg, 1.0, th_bd);
                    cx.painter.rect(Rect::from_xywh(kr.min.x + 4.0, kr.center().y - 0.5, kr.width() - 8.0, 1.0), act_c);

                    let g = cx.galley(f_lbl, &st_f);
                    cx.painter.text(cx.text, &g, vec2(cx_x - g.width * 0.5, r.max.y - g.height), fg_m);
                })),
            );
        }
        self.end();

        self.end();
        changed
    }
}
