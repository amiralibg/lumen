//! Rich views, planning & Ethereal surface widgets:
//! - [`Ui::gantt_chart`] (roadmap / sprint timeline with completion fill, milestones and hover tooltips)
//! - [`Treemap`] (proportional slice-and-dice area breakdown with hover tooltips)
//! - [`Ui::markdown`] (zero-dep immediate-mode Markdown document view)
//! - [`Ui::notification_list`] (filterable All / Unread inbox feed with Mark all read and archive)
//! - [`SpotlightCard`] (Ethereal pointer-following radial accent spotlight card)

use crate::anim::Ease;
use crate::color::Color;
use crate::input::CursorIcon;
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides, Vec2};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};

use super::chart::{compact, tooltip_panel, Tip};
use super::toast::Tone;

const SENSE_HOVER: Sense = Sense { click: true, drag: false, focusable: false, scroll: false };

// ============================================================= GanttChart

/// Task or milestone row in [`Ui::gantt_chart`]. `start` and `duration` are in
/// column units (`0.0 .. cols.len() as f32`). Set `duration == 0.0` for a
/// diamond milestone marker.
#[derive(Clone, Debug)]
pub struct GanttTask {
    pub name: String,
    pub owner: Option<String>,
    pub start: f32,
    pub duration: f32,
    pub progress: f32,
    pub tone: Tone,
}

impl GanttTask {
    pub fn new(name: impl Into<String>, start: f32, duration: f32, progress: f32) -> Self {
        GanttTask {
            name: name.into(),
            owner: None,
            start: start.max(0.0),
            duration: duration.max(0.0),
            progress: progress.clamp(0.0, 1.0),
            tone: Tone::Accent,
        }
    }
    pub fn milestone(name: impl Into<String>, at: f32) -> Self {
        GanttTask { name: name.into(), owner: None, start: at.max(0.0), duration: 0.0, progress: 1.0, tone: Tone::Warning }
    }
    pub fn owner(mut self, initials: impl Into<String>) -> Self {
        self.owner = Some(initials.into());
        self
    }
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
}

fn tone_col(ui: &Ui, tone: Tone) -> Color {
    match tone {
        Tone::Neutral => ui.color(Token::Series4),
        Tone::Accent => ui.color(Token::Accent),
        Tone::Success => ui.color(Token::Success),
        Tone::Warning => ui.color(Token::Warning),
        Tone::Info => ui.color(Token::Info),
        Tone::Danger => ui.color(Token::Danger),
    }
}

// =============================================================== Treemap

/// Weighted item in [`Treemap`].
#[derive(Clone, Debug)]
pub struct TreemapItem {
    pub label: String,
    pub value: f32,
    pub detail: Option<String>,
}

impl TreemapItem {
    pub fn new(label: impl Into<String>, value: f32) -> Self {
        TreemapItem { label: label.into(), value: value.max(0.001), detail: None }
    }
    pub fn detail(mut self, d: impl Into<String>) -> Self {
        self.detail = Some(d.into());
        self
    }
}

fn layout_treemap(indices: &[usize], values: &[f32], rect: Rect, out: &mut [(usize, Rect)]) {
    if indices.is_empty() || out.is_empty() {
        return;
    }
    if indices.len() == 1 {
        out[0] = (indices[0], rect);
        return;
    }
    let total: f32 = indices.iter().map(|&i| values[i]).sum::<f32>().max(1e-6);
    let mut acc = 0.0f32;
    let mut split = 1usize;
    for (k, &idx) in indices.iter().enumerate() {
        if k > 0 && acc >= total * 0.45 {
            split = k;
            break;
        }
        acc += values[idx];
        split = k + 1;
    }
    split = split.clamp(1, indices.len() - 1);
    let left_sum: f32 = indices[..split].iter().map(|&i| values[i]).sum();
    let frac = (left_sum / total).clamp(0.08, 0.92);

    let (r_a, r_b) = if rect.width() >= rect.height() {
        let w_a = rect.width() * frac;
        (
            Rect::from_xywh(rect.min.x, rect.min.y, w_a, rect.height()),
            Rect::from_xywh(rect.min.x + w_a, rect.min.y, rect.width() - w_a, rect.height()),
        )
    } else {
        let h_a = rect.height() * frac;
        (
            Rect::from_xywh(rect.min.x, rect.min.y, rect.width(), h_a),
            Rect::from_xywh(rect.min.x, rect.min.y + h_a, rect.width(), rect.height() - h_a),
        )
    };
    layout_treemap(&indices[..split], values, r_a, &mut out[..split]);
    layout_treemap(&indices[split..], values, r_b, &mut out[split..]);
}

/// Proportional area treemap colored across `Series0..5` with hover tooltips.
pub struct Treemap<'u> {
    ui: &'u mut Ui,
    key: String,
    items: Vec<TreemapItem>,
    unit: String,
    height: f32,
    shown: bool,
}

impl<'u> Treemap<'u> {
    pub fn unit(mut self, u: impl Into<String>) -> Self {
        self.unit = u.into();
        self
    }
    pub fn height(mut self, px: f32) -> Self {
        self.height = px.max(140.0);
        self
    }
    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("treemap", &self.key));
        let ix = ui.interact(id, SENSE_HOVER, false);
        let mo = ui.motion();
        let enter = ui.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);

        let mut order: Vec<usize> = (0..self.items.len()).collect();
        order.sort_by(|&a, &b| self.items[b].value.partial_cmp(&self.items[a].value).unwrap_or(std::cmp::Ordering::Equal));
        let values: Vec<f32> = self.items.iter().map(|i| i.value).collect();
        let total: f32 = values.iter().sum::<f32>().max(1e-6);

        let palette = [
            ui.color(Token::Series0),
            ui.color(Token::Series1),
            ui.color(Token::Series2),
            ui.color(Token::Series3),
            ui.color(Token::Series4),
            ui.color(Token::Series5),
        ];
        let hover_bd = ui.color(Token::Foreground);
        let st_title = ui.text_style(TextRole::BodyStrong);
        let mut st_sub = ui.text_style(TextRole::Caption);
        st_sub.tabular = true;
        let tip = Tip::new(ui);

        let items_cl = self.items.clone();
        let unit = self.unit.clone();
        let ptr = ix.pointer.filter(|_| ix.hovered);

        ui.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(280.0, self.height),
            Some(id),
            SENSE_HOVER,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                if order.is_empty() {
                    return;
                }
                let mut placed = vec![(0usize, Rect::ZERO); order.len()];
                layout_treemap(&order, &values, r, &mut placed);

                let mut hov_tile: Option<(usize, Rect, Color)> = None;
                for (rank, &(idx, tr)) in placed.iter().enumerate() {
                    let inset = tr.expand(-2.0);
                    if inset.width() <= 1.0 || inset.height() <= 1.0 {
                        continue;
                    }
                    let scaled = Rect::from_center_size(inset.center(), inset.size() * (0.85 + 0.15 * enter));
                    let base_c = palette[rank % palette.len()];
                    let is_h = ptr.is_some_and(|p| inset.contains(p));
                    let fill_c = base_c.multiply_alpha(if is_h { 0.92 } else { 0.78 });
                    cx.painter.rrect(scaled, Corners::all(8.0), fill_c, if is_h { 1.5 } else { 0.0 }, hover_bd);

                    if scaled.width() > 44.0 && scaled.height() > 30.0 {
                        let max_w = scaled.width() - 16.0;
                        let it = &items_cl[idx];
                        let g_t = cx.galley(&it.label, &st_title);
                        let mut title_h = 0.0;
                        if g_t.width <= max_w {
                            title_h = g_t.height;
                            cx.painter.text(cx.text, &g_t, vec2(scaled.min.x + 9.0, scaled.min.y + 8.0), Color::WHITE);
                        } else {
                            let g_sm = cx.galley(&it.label, &st_sub);
                            if g_sm.width <= max_w {
                                title_h = g_sm.height;
                                cx.painter.text(cx.text, &g_sm, vec2(scaled.min.x + 8.0, scaled.min.y + 8.0), Color::WHITE);
                            } else {
                                let chars: Vec<char> = it.label.chars().collect();
                                let mut hi = chars.len();
                                while hi > 1 {
                                    hi -= 1;
                                    let cand: String = chars[..hi].iter().collect::<String>() + "…";
                                    let gc = cx.galley(&cand, &st_sub);
                                    if gc.width <= max_w {
                                        title_h = gc.height;
                                        cx.painter.text(cx.text, &gc, vec2(scaled.min.x + 8.0, scaled.min.y + 8.0), Color::WHITE);
                                        break;
                                    }
                                }
                            }
                        }
                        if title_h > 0.0 && scaled.height() > 50.0 {
                            let val_s = if unit.is_empty() { compact(it.value) } else { format!("{} {unit}", compact(it.value)) };
                            let pct = (it.value / total * 100.0).round();
                            let sub = format!("{val_s} · {pct:.0}%");
                            let g_s = cx.galley(&sub, &st_sub);
                            if g_s.width <= max_w {
                                cx.painter.text(cx.text, &g_s, vec2(scaled.min.x + 9.0, scaled.min.y + 10.0 + title_h), Color::WHITE.multiply_alpha(0.84));
                            } else {
                                let g_v = cx.galley(&val_s, &st_sub);
                                if g_v.width <= max_w {
                                    cx.painter.text(cx.text, &g_v, vec2(scaled.min.x + 8.0, scaled.min.y + 10.0 + title_h), Color::WHITE.multiply_alpha(0.84));
                                }
                            }
                        }
                    }
                    if is_h {
                        hov_tile = Some((idx, scaled, base_c));
                    }
                }

                if let Some((idx, tr, col)) = hov_tile {
                    let it = &items_cl[idx];
                    let val_s = if unit.is_empty() { compact(it.value) } else { format!("{} {unit}", compact(it.value)) };
                    let pct = it.value / total * 100.0;
                    let mut rows = vec![
                        (col, "Size".into(), val_s),
                        (col, "Share".into(), format!("{pct:.1}%")),
                    ];
                    if let Some(ref d) = it.detail {
                        rows.push((col, "Info".into(), d.clone()));
                    }
                    let title = it.label.clone();
                    let tip_cl = tip.clone();
                    let bounds = r.expand(12.0);
                    cx.defer(r, Box::new(move |cx, _| tooltip_panel(cx, tr.center(), bounds, &title, &rows, &tip_cl)));
                }
            })),
        );
        Response { id, rect: ix.rect, hovered: ix.hovered, ..Default::default() }
    }
}

impl Drop for Treemap<'_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

// ====================================================== NotificationList

/// Item in [`Ui::notification_list`].
#[derive(Clone, Debug)]
pub struct NotificationItem {
    pub id: String,
    pub author: String,
    pub title: String,
    pub body: String,
    pub time: String,
    pub unread: bool,
}

impl NotificationItem {
    pub fn new(id: impl Into<String>, author: impl Into<String>, title: impl Into<String>, body: impl Into<String>, time: impl Into<String>, unread: bool) -> Self {
        NotificationItem {
            id: id.into(),
            author: author.into(),
            title: title.into(),
            body: body.into(),
            time: time.into(),
            unread,
        }
    }
}

#[derive(Default)]
struct NotifMem {
    unread_only: bool,
}

// ========================================================= SpotlightCard

/// Ethereal card whose surface and border bloom with a soft radial accent
/// spotlight following the pointer (`site/DESIGN.md`).
pub struct SpotlightCard<'u> {
    ui: &'u mut Ui,
    key: String,
    icon: Option<String>,
    title: Option<String>,
}

impl<'u> SpotlightCard<'u> {
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = Some(name.to_owned());
        self
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }

    pub fn show<R>(self, body: impl FnOnce(&mut Ui) -> R) -> R {
        let SpotlightCard { ui, key, icon, title } = self;
        let id = ui.make_id(("spotlight_card", &key));
        let ix = ui.interact(id, SENSE_HOVER, false);
        let mo = ui.motion();
        let reduced = ui.reduced_motion();
        let hov = ui.fade(id, 0, ix.hovered, mo.hover_in, 0.24, Ease::Out);
        let rect_prev = ui.rect_of(id).unwrap_or_default();
        let (tx, ty) = match ix.pointer.filter(|_| ix.hovered) {
            Some(p) if rect_prev.width() > 1.0 => (p.x - rect_prev.min.x, p.y - rect_prev.min.y),
            _ => (
                ui.anims.spring_state(id, 1).map_or(rect_prev.width() * 0.5, |s| s.value),
                ui.anims.spring_state(id, 2).map_or(rect_prev.height() * 0.35, |s| s.value),
            ),
        };
        let gx = ui.spring(id, 1, tx, mo.smooth);
        let gy = ui.spring(id, 2, ty, mo.smooth);

        let dark = ui.theme.darkness();
        let r_card = ui.look().card.radius;
        let bg_c = ui.color(Token::Surface);
        let bd_c = ui.color(Token::BorderSubtle).mix(ui.color(Token::Accent).multiply_alpha(0.55), hov);
        let glow_c = ui.color(Token::Accent);
        let tile_bg = ui.color(Token::AccentSubtle);
        let tile_ic = ui.color(Token::AccentStrong);
        let fg_c = ui.color(Token::Foreground);

        ui.begin(
            Style {
                dir: Dir::Column,
                width: Size::FILL,
                pad: Sides::all(18.0),
                gap: 10.0,
                ..Default::default()
            },
            Some(id),
            SENSE_HOVER,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_card), bg_c, 0.0, Color::TRANSPARENT);
                if hov > 0.005 && !reduced {
                    let spot_pos = r.min + vec2(gx, gy);
                    cx.painter.push_clip(r.expand(-1.0));
                    cx.painter.shadow(
                        Rect::from_center_size(spot_pos, vec2(124.0, 124.0)),
                        Corners::all(62.0),
                        &crate::theme::Shadow {
                            x: 0.0,
                            y: 0.0,
                            blur: 170.0,
                            spread: 0.0,
                            color: glow_c.with_alpha((0.18 + 0.14 * dark) * hov),
                        },
                    );
                    cx.painter.pop_clip();
                }
                cx.painter.ring(r, Corners::all(r_card), 1.0, bd_c);
                // Top glass specular edge.
                cx.painter.rect(
                    Rect::from_xywh(r.min.x + r_card, r.min.y + 1.0, (r.width() - r_card * 2.0).max(0.0), 1.0),
                    glow_c.multiply_alpha(0.14 + 0.24 * hov),
                );
            })),
        );

        if icon.is_some() || title.is_some() {
            ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 10.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            if let Some(ic) = icon {
                ui.leaf(
                    Style::default(),
                    vec2(34.0, 34.0),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, r| {
                        cx.painter.rrect(r, Corners::all(9.0), tile_bg, 0.0, Color::TRANSPARENT);
                        cx.icon(&ic, r.center(), 17.0, tile_ic);
                    })),
                );
            }
            if let Some(t) = title {
                let st_t = ui.text_style(TextRole::Heading3);
                ui.text_leaf(t, st_t, fg_c, false, true, Style { width: Size::FILL, ..Default::default() });
            }
            ui.end();
        }

        let res = ui.scope("spot_body", body);
        ui.end();
        res
    }
}

// =========================================================== Markdown helpers

fn take_trailing_punct<'a>(s: &'a str) -> (&'a str, &'a str) {
    let n = s
        .chars()
        .take_while(|c| matches!(c, '.' | ',' | ':' | ';' | '!' | '?' | ')' | ']'))
        .map(char::len_utf8)
        .sum::<usize>();
    (&s[..n], &s[n..])
}

fn render_inline_md(ui: &mut Ui, text: &str, base_role: TextRole, base_col: Color) {
    // Fast check: if no ` or ** markers, emit one wrapping text leaf!
    if !text.contains('`') && !text.contains("**") {
        let st = ui.text_style(base_role);
        ui.text_leaf(text.to_owned(), st, base_col, true, false, Style { width: Size::FILL, ..Default::default() });
        return;
    }

    // Tokenize into wrapping inline words/code/bold pills with attached punctuation.
    ui.begin(
        Style { dir: Dir::Row, width: Size::FILL, gap: 4.0, wrap: true, align: Align::Center, ..Default::default() },
        None,
        Sense::NONE,
        CursorIcon::Default,
        false,
        None,
    );
    let mut rest = text;
    let mut pending_prefix = String::new();
    while !rest.is_empty() {
        if let Some(after_tick) = rest.strip_prefix('`') {
            if let Some(end) = after_tick.find('`') {
                let code_txt = &after_tick[..end];
                let (suffix, next_rest) = take_trailing_punct(&after_tick[end + 1..]);
                let prefix = std::mem::take(&mut pending_prefix);
                let code_bg = ui.color(Token::SurfaceMuted);
                let code_bd = ui.color(Token::BorderSubtle);
                let code_fg = ui.color(Token::AccentStrong);
                let mut mono = ui.text_style(TextRole::Mono);
                mono.size = (mono.size - 0.5).max(11.0);
                let st_plain = ui.text_style(base_role);

                if !prefix.is_empty() || !suffix.is_empty() {
                    ui.begin(
                        Style { dir: Dir::Row, gap: 1.5, align: Align::Center, ..Default::default() },
                        None,
                        Sense::NONE,
                        CursorIcon::Default,
                        false,
                        None,
                    );
                    if !prefix.is_empty() {
                        ui.text_leaf(prefix, st_plain, base_col, false, false, Style::default());
                    }
                    ui.begin(
                        Style { pad: Sides::xy(5.0, 1.0), ..Default::default() },
                        None,
                        Sense::NONE,
                        CursorIcon::Default,
                        false,
                        Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(4.0), code_bg, 1.0, code_bd))),
                    );
                    ui.text_leaf(code_txt.to_owned(), mono, code_fg, false, false, Style::default());
                    ui.end();
                    if !suffix.is_empty() {
                        ui.text_leaf(suffix.to_owned(), st_plain, base_col, false, false, Style::default());
                    }
                    ui.end();
                } else {
                    ui.begin(
                        Style { pad: Sides::xy(5.0, 1.0), ..Default::default() },
                        None,
                        Sense::NONE,
                        CursorIcon::Default,
                        false,
                        Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(4.0), code_bg, 1.0, code_bd))),
                    );
                    ui.text_leaf(code_txt.to_owned(), mono, code_fg, false, false, Style::default());
                    ui.end();
                }
                rest = next_rest.trim_start_matches(' ');
                continue;
            }
        }
        if let Some(after_b) = rest.strip_prefix("**") {
            if let Some(end) = after_b.find("**") {
                let bold_txt = &after_b[..end];
                let (suffix, next_rest) = take_trailing_punct(&after_b[end + 2..]);
                let prefix = std::mem::take(&mut pending_prefix);
                let st_b = ui.text_style(TextRole::BodyStrong);
                let fg_strong = ui.color(Token::Foreground);
                let words: Vec<&str> = bold_txt.split_whitespace().collect();
                for (i, w) in words.iter().enumerate() {
                    let mut s = String::new();
                    if i == 0 && !prefix.is_empty() {
                        s.push_str(&prefix);
                    }
                    s.push_str(w);
                    if i + 1 == words.len() && !suffix.is_empty() {
                        s.push_str(suffix);
                    }
                    ui.text_leaf(s, st_b, fg_strong, false, false, Style::default());
                }
                rest = next_rest.trim_start_matches(' ');
                continue;
            }
        }
        let next_special = [rest.find('`'), rest.find("**")].into_iter().flatten().min().unwrap_or(rest.len());
        let plain = if next_special == 0 {
            let ch_len = rest.chars().next().map(char::len_utf8).unwrap_or(1);
            let p = &rest[..ch_len];
            rest = &rest[ch_len..];
            p
        } else {
            let p = &rest[..next_special];
            rest = &rest[next_special..];
            p
        };
        let st = ui.text_style(base_role);
        let words: Vec<&str> = plain.split_whitespace().collect();
        let ends_no_space = !plain.ends_with(char::is_whitespace) && !rest.is_empty();
        for (i, w) in words.iter().enumerate() {
            let mut tok = if i == 0 && !pending_prefix.is_empty() {
                let mut p = std::mem::take(&mut pending_prefix);
                p.push_str(w);
                p
            } else {
                (*w).to_owned()
            };
            if i + 1 == words.len() && ends_no_space && matches!(tok.as_str(), "(" | "[") {
                pending_prefix = tok;
            } else {
                ui.text_leaf(std::mem::take(&mut tok), st, base_col, false, false, Style::default());
            }
        }
    }
    if !pending_prefix.is_empty() {
        let st = ui.text_style(base_role);
        ui.text_leaf(pending_prefix, st, base_col, false, false, Style::default());
    }
    ui.end();
}

impl Ui {
    /// Roadmap / sprint Gantt timeline with progress bars, milestones and hover tooltips.
    pub fn gantt_chart(&mut self, key: impl std::hash::Hash, columns: &[&str], tasks: &[GanttTask]) -> Response {
        let id = self.make_id(("gantt", key));
        let ix = self.interact(id, SENSE_HOVER, false);
        let mo = self.motion();
        let enter = self.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);

        let n_cols = columns.len().max(1);
        let r_card = self.look().card.radius;
        let top_rad = (r_card - 1.0).max(0.0);
        let card_bg = self.color(Token::Surface);
        let card_bd = self.color(Token::BorderSubtle);
        let hdr_bg = self.color(Token::SurfaceMuted).multiply_alpha(0.6);
        let div_c = self.color(Token::BorderSubtle);
        let grid_c = self.color(Token::BorderSubtle).multiply_alpha(0.65);
        let fg_c = self.color(Token::Foreground);
        let muted_c = self.color(Token::TextMuted);
        let hov_bg = self.color(Token::Hover);
        let tip = Tip::new(self);

        let task_cols: Vec<Color> = tasks.iter().map(|t| tone_col(self, t.tone)).collect();
        let st_hdr = self.text_style(TextRole::Caption);
        let st_name = self.text_style(TextRole::BodyStrong);

        let ruler_h = 30.0;
        let row_h = 34.0;
        let total_h = ruler_h + tasks.len() as f32 * row_h + 6.0;
        let cols_vec: Vec<String> = columns.iter().map(|s| (*s).to_owned()).collect();
        let tasks_vec = tasks.to_vec();
        let ptr = ix.pointer.filter(|_| ix.hovered);

        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(320.0, total_h),
            Some(id),
            SENSE_HOVER,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_card), card_bg, 1.0, card_bd);
                let hdr_r = Rect::from_xywh(r.min.x + 1.0, r.min.y + 1.0, (r.width() - 2.0).max(0.0), ruler_h - 1.0);
                cx.painter.rrect(hdr_r, Corners([top_rad, top_rad, 0.0, 0.0]), hdr_bg, 0.0, Color::TRANSPARENT);
                cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.min.y + ruler_h, (r.width() - 2.0).max(0.0), 1.0), div_c);

                let left_w = (r.width() * 0.38).clamp(140.0, 224.0);
                let tl_x0 = r.min.x + left_w;
                let tl_w = (r.max.x - 12.0 - tl_x0).max(60.0);
                let col_w = tl_w / n_cols as f32;

                let g_t = cx.galley("Deliverable", &st_hdr);
                cx.painter.text(cx.text, &g_t, vec2(r.min.x + 14.0, r.min.y + (ruler_h - g_t.height) * 0.5), muted_c);

                for (ci, cl) in cols_vec.iter().enumerate() {
                    let x = tl_x0 + ci as f32 * col_w;
                    cx.painter.rect(Rect::from_xywh(cx.painter.snap(x), r.min.y + ruler_h, 1.0, r.height() - ruler_h - 2.0), grid_c);
                    let g = cx.galley(cl, &st_hdr);
                    cx.painter.text(cx.text, &g, vec2((x + (col_w - g.width) * 0.5).round(), r.min.y + (ruler_h - g.height) * 0.5), muted_c);
                }

                let mut hov_task: Option<(usize, Vec2)> = None;
                for (ti, t) in tasks_vec.iter().enumerate() {
                    let ry = r.min.y + ruler_h + 2.0 + ti as f32 * row_h;
                    let rr = Rect::from_xywh(r.min.x + 2.0, ry, r.width() - 4.0, row_h);
                    let is_h = ptr.is_some_and(|p| rr.contains(p));
                    if is_h {
                        cx.painter.rrect(rr, Corners::all(5.0), hov_bg, 0.0, Color::TRANSPARENT);
                    }

                    cx.painter.push_clip(Rect::from_xywh(r.min.x + 12.0, ry, left_w - 20.0, row_h));
                    let g_nm = cx.galley(&t.name, &st_name);
                    cx.painter.text(cx.text, &g_nm, vec2(r.min.x + 14.0, ry + (row_h - g_nm.height) * 0.5), fg_c);
                    cx.painter.pop_clip();

                    let col = task_cols[ti];
                    let cy = ry + row_h * 0.5;
                    let sx = tl_x0 + (t.start / n_cols as f32) * tl_w;
                    if t.duration <= 0.01 {
                        // Diamond milestone marker!
                        let d = 7.0 * enter;
                        let c = vec2(sx, cy);
                        let pts = [c + vec2(0.0, -d), c + vec2(d, 0.0), c + vec2(0.0, d), c + vec2(-d, 0.0)];
                        cx.painter.mesh(&[(pts[0], col), (pts[1], col), (pts[2], col), (pts[3], col)], &[0, 1, 2, 0, 2, 3]);
                        if is_h {
                            hov_task = Some((ti, c));
                        }
                    } else {
                        let bw = ((t.duration / n_cols as f32) * tl_w * enter).max(8.0);
                        let bar = Rect::from_xywh(sx, cy - 8.0, bw, 16.0);
                        cx.painter.rrect(bar, Corners::all(5.0), col.multiply_alpha(0.28), 1.0, col.multiply_alpha(0.65));
                        if t.progress > 0.01 {
                            let pw = (bar.width() * t.progress).max(4.0);
                            cx.painter.rrect(Rect::from_xywh(bar.min.x, bar.min.y, pw, bar.height()), Corners::all(5.0), col, 0.0, Color::TRANSPARENT);
                        }
                        if let Some(ref own) = t.owner {
                            let g_o = cx.galley(own, &st_hdr);
                            if bar.max.x + 6.0 + g_o.width < r.max.x - 4.0 {
                                cx.painter.text(cx.text, &g_o, vec2(bar.max.x + 6.0, cy - g_o.height * 0.5), muted_c);
                            }
                        }
                        if is_h {
                            hov_task = Some((ti, bar.center()));
                        }
                    }
                }

                if let Some((ti, anchor)) = hov_task {
                    let t = &tasks_vec[ti];
                    let col = task_cols[ti];
                    let rows = if t.duration <= 0.01 {
                        vec![(col, "Type".into(), "Milestone".into())]
                    } else {
                        vec![(col, "Progress".into(), format!("{:.0}%", (t.progress * 100.0).round()))]
                    };
                    let title = t.name.clone();
                    let tip_cl = tip.clone();
                    let bounds = r.expand(12.0);
                    cx.defer(r, Box::new(move |cx, _| tooltip_panel(cx, anchor, bounds, &title, &rows, &tip_cl)));
                }
            })),
        );
        Response { id, rect: ix.rect, hovered: ix.hovered, ..Default::default() }
    }

    /// Proportional slice-and-dice area treemap.
    pub fn treemap(&mut self, key: impl Into<String>, items: &[TreemapItem]) -> Treemap<'_> {
        Treemap { ui: self, key: key.into(), items: items.to_vec(), unit: String::new(), height: 230.0, shown: false }
    }

    /// Zero-dependency immediate-mode Markdown renderer supporting headings,
    /// inline `**bold**` and `` `code` ``, bullet/numbered lists, blockquotes,
    /// horizontal rules and syntax-highlighted fenced code blocks.
    pub fn markdown(&mut self, key: impl std::hash::Hash, md: &str) -> Response {
        let id = self.make_id(("markdown", key));
        self.begin(Style { width: Size::FILL, dir: Dir::Column, gap: 10.0, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);

        let mut in_fence = false;
        let mut fence_lang = String::new();
        let mut fence_buf = String::new();
        let mut block_idx = 0usize;

        for raw in md.lines() {
            let trimmed = raw.trim();
            if let Some(after) = trimmed.strip_prefix("```") {
                if in_fence {
                    let code_s = fence_buf.trim_end_matches('\n').to_owned();
                    let lang_s = if fence_lang.is_empty() { "rust" } else { fence_lang.as_str() };
                    self.code_block(format!("md_cb_{block_idx}"), code_s).language(lang_s).line_numbers(false).show();
                    block_idx += 1;
                    fence_buf.clear();
                    fence_lang.clear();
                    in_fence = false;
                } else {
                    fence_lang = after.trim().to_owned();
                    in_fence = true;
                }
                continue;
            }
            if in_fence {
                fence_buf.push_str(raw);
                fence_buf.push('\n');
                continue;
            }
            if trimmed.is_empty() {
                continue;
            }
            if trimmed == "---" || trimmed == "***" {
                self.divider();
                continue;
            }
            if let Some(h1) = trimmed.strip_prefix("# ") {
                let c = self.color(Token::Foreground);
                render_inline_md(self, h1, TextRole::Heading2, c);
                continue;
            }
            if let Some(h2) = trimmed.strip_prefix("## ") {
                let c = self.color(Token::Foreground);
                render_inline_md(self, h2, TextRole::Heading3, c);
                continue;
            }
            if let Some(h3) = trimmed.strip_prefix("### ") {
                let c = self.color(Token::Foreground);
                render_inline_md(self, h3, TextRole::BodyStrong, c);
                continue;
            }
            if let Some(q) = trimmed.strip_prefix("> ") {
                let bar_c = self.color(Token::Accent);
                let bg_c = self.color(Token::AccentSubtle).multiply_alpha(0.35);
                let txt_c = self.color(Token::TextSecondary);
                self.begin(
                    Style { dir: Dir::Row, width: Size::FILL, pad: Sides { left: 14.0, right: 10.0, top: 7.0, bottom: 7.0 }, ..Default::default() },
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    Some(Box::new(move |cx, r| {
                        cx.painter.rrect(r, Corners::all(6.0), bg_c, 0.0, Color::TRANSPARENT);
                        cx.painter.rrect(Rect::from_xywh(r.min.x, r.min.y + 3.0, 3.0, (r.height() - 6.0).max(0.0)), Corners::all(1.5), bar_c, 0.0, Color::TRANSPARENT);
                    })),
                );
                render_inline_md(self, q, TextRole::Body, txt_c);
                self.end();
                continue;
            }
            if let Some(li) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
                let dot_c = self.color(Token::Accent);
                let txt_c = self.color(Token::Foreground);
                self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, align: Align::Start, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                self.leaf(
                    Style::default(),
                    vec2(10.0, 20.0),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, r| cx.painter.circle(r.center(), 2.6, dot_c, 0.0, Color::TRANSPARENT))),
                );
                self.begin(Style { width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                render_inline_md(self, li, TextRole::Body, txt_c);
                self.end();
                self.end();
                continue;
            }
            let txt_c = self.color(Token::TextSecondary);
            render_inline_md(self, trimmed, TextRole::Body, txt_c);
        }

        self.end();
        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Filterable `All` / `Unread` notification inbox with `"Mark all read"` and archive actions.
    pub fn notification_list(&mut self, key: impl std::hash::Hash, items: &mut Vec<NotificationItem>) -> Response {
        let id = self.make_id(("notif_list", key));
        let mut unread_only = self.mem::<NotifMem>(id).unread_only;
        let unread_count = items.iter().filter(|i| i.unread).count();

        let r_card = self.look().card.radius;
        let top_rad = (r_card - 1.0).max(0.0);
        let card_bg = self.color(Token::Surface);
        let card_bd = self.color(Token::BorderSubtle);
        let hdr_bg = self.color(Token::SurfaceMuted).multiply_alpha(0.6);
        let div_c = self.color(Token::BorderSubtle);

        self.begin(
            Style { width: Size::FILL, dir: Dir::Column, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), card_bg, 1.0, card_bd))),
        );

        // Header bar.
        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(12.0, 8.0), gap: 6.0, align: Align::Center, ..Default::default() },
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
        let mut all_on = !unread_only;
        if self.scope("nf_all", |ui| ui.chip("All", &mut all_on).clicked) {
            unread_only = false;
        }
        let mut un_on = unread_only;
        let un_lbl = format!("Unread {unread_count}");
        if self.scope("nf_un", |ui| ui.chip(&un_lbl, &mut un_on).clicked) {
            unread_only = true;
        }
        self.spacer();
        if unread_count > 0 && self.scope("mk_read", |ui| ui.button("Mark all read").ghost().small().icon("check-check").clicked()) {
            for it in items.iter_mut() {
                it.unread = false;
            }
        }
        self.end();

        self.mem::<NotifMem>(id).unread_only = unread_only;

        let mut archive_idx: Option<usize> = None;
        let mo = self.motion();
        let mut shown_any = false;

        for (idx, item) in items.iter_mut().enumerate() {
            if unread_only && !item.unread {
                continue;
            }
            let draw_sep = shown_any;
            shown_any = true;
            let rid = id.with(("nrow", &item.id));
            let rix = self.interact(rid, Sense::CLICK, false);
            if rix.clicked {
                item.unread = !item.unread;
            }
            let h = self.fade(rid, 0, rix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
            let hov_c = self.color(Token::Hover).multiply_alpha(h);
            let un_tint = if item.unread { self.color(Token::AccentSubtle).multiply_alpha(0.28) } else { Color::TRANSPARENT };
            let sep_c = self.color(Token::BorderSubtle);
            let dot_c = self.color(Token::Accent);
            let fg_c = self.color(Token::Foreground);
            let sec_c = self.color(Token::TextSecondary);
            let tm_c = self.color(Token::TextMuted);
            let is_unread = item.unread;

            self.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(14.0, 11.0), gap: 11.0, align: Align::Start, ..Default::default() },
                Some(rid),
                Sense::CLICK,
                CursorIcon::Pointer,
                false,
                Some(Box::new(move |cx, r| {
                    let bg = un_tint.over(hov_c);
                    if bg.a > 0.0 {
                        cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.min.y, (r.width() - 2.0).max(0.0), r.height()), bg);
                    }
                    if draw_sep {
                        cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.min.y, (r.width() - 2.0).max(0.0), 1.0), sep_c.multiply_alpha(0.7));
                    }
                })),
            );

            self.avatar(&item.author, idx, 30.0, None);

            self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 2.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            if is_unread {
                self.leaf(
                    Style::default(),
                    vec2(8.0, 8.0),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, r| cx.painter.circle(r.center(), 3.8, dot_c, 0.0, Color::TRANSPARENT))),
                );
            }
            let st_t = if is_unread { self.text_style(TextRole::BodyStrong) } else { self.text_style(TextRole::Body) };
            self.text_leaf(item.title.clone(), st_t, fg_c, false, true, Style { width: Size::FILL, ..Default::default() });
            let st_tm = self.text_style(TextRole::Caption);
            self.text_leaf(item.time.clone(), st_tm, tm_c, false, false, Style::default());
            self.end();

            let st_b = self.text_style(TextRole::Body);
            self.text_leaf(item.body.clone(), st_b, sec_c, true, false, Style { width: Size::FILL, ..Default::default() });
            self.end();

            if self.field_icon_button(id.with(("arch", &item.id)), "x", 1.0, false) {
                archive_idx = Some(idx);
            }
            self.end();
        }

        if !shown_any {
            let mc = self.color(Token::TextMuted);
            let st = self.text_style(TextRole::Body);
            self.begin(Style { width: Size::FILL, pad: Sides::xy(16.0, 28.0), align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.text_leaf("Inbox zero — all caught up.", st, mc, false, false, Style::default());
            self.end();
        }

        if let Some(ai) = archive_idx {
            items.remove(ai);
        }

        self.end();
        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Ethereal card whose border and surface follow the pointer with a soft radial accent glow.
    pub fn spotlight_card(&mut self, key: impl Into<String>) -> SpotlightCard<'_> {
        SpotlightCard { ui: self, key: key.into(), icon: None, title: None }
    }
}
