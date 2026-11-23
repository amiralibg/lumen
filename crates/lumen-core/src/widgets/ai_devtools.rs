//! AI interfaces & developer-tools widgets:
//! - [`Ui::reasoning_block`] & [`Ui::tool_call`] (collapsible CoT trace + structured tool invocation row)
//! - [`PromptComposer`] (auto-growing AI prompt box with attachment pills, model menu, context ring and send/stop)
//! - [`Ui::chat_thread`] (assistant & user message feed with citation pills and actions)
//! - [`DiffViewer`] (unified diff card with dual line numbers and `+`/`-` stats)
//! - [`Ui::waterfall`] (distributed trace / network span timeline with ruler and hover tooltip)
//! - [`Ui::uptime_bars`] (60-bar service availability strip with hover readout)

use std::f32::consts::TAU;

use crate::anim::Ease;
use crate::color::Color;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides, Vec2};
use crate::theme::{ControlSize, TextRole, Token};
use crate::ui::{Response, Sense, Ui};

use super::chart::{tooltip_panel, Tip};
use super::data::draw_chevron;
use super::field::EditOpts;
use super::{blend3, key_activate};

// ================================================== Reasoning & Tool Calls

/// Status of an AI [`ToolCall`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolStatus {
    Running,
    Done,
    Error,
}

/// Structured tool invocation row for [`Ui::tool_call`].
#[derive(Clone, Debug)]
pub struct ToolCall {
    pub name: String,
    pub args: String,
    pub status: ToolStatus,
    pub duration: Option<String>,
}

impl ToolCall {
    pub fn new(name: impl Into<String>, args: impl Into<String>, status: ToolStatus) -> Self {
        ToolCall { name: name.into(), args: args.into(), status, duration: None }
    }
    pub fn duration(mut self, d: impl Into<String>) -> Self {
        self.duration = Some(d.into());
        self
    }
}

// ========================================================= PromptComposer

/// Output events emitted by [`PromptComposer`] on a frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct ComposerOut {
    pub submitted: bool,
    pub stopped: bool,
    pub attach_clicked: bool,
}

/// Rich AI prompt composer with auto-growing multiline input, removable
/// attachment chips, model picker menu, context-window ring, and Send / Stop button.
pub struct PromptComposer<'u> {
    ui: &'u mut Ui,
    key: String,
    prompt: &'u mut String,
    model_idx: &'u mut usize,
    models: Vec<String>,
    attachments: Option<&'u mut Vec<String>>,
    placeholder: String,
    context_used: f32,
    streaming: bool,
}

impl<'u> PromptComposer<'u> {
    pub fn models(mut self, models: &[&str]) -> Self {
        self.models = models.iter().map(|s| (*s).to_owned()).collect();
        self
    }
    pub fn attachments(mut self, files: &'u mut Vec<String>) -> Self {
        self.attachments = Some(files);
        self
    }
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }
    /// Fraction (`0.0..=1.0`) of context window consumed.
    pub fn context_used(mut self, frac: f32) -> Self {
        self.context_used = frac.clamp(0.0, 1.0);
        self
    }
    pub fn streaming(mut self, on: bool) -> Self {
        self.streaming = on;
        self
    }

    pub fn show(mut self) -> ComposerOut {
        let ui = &mut *self.ui;
        let id = ui.make_id(("prompt_composer", &self.key));
        let ed_id = id.with("ed");
        let frame_id = id.with("frame");
        let mo = ui.motion();

        let focused = ui.focused() == Some(ed_id);
        let hovered = ui.pointer().zip(ui.rect_of(frame_id)).is_some_and(|(p, r)| r.contains(p));
        if ui.interact(frame_id, Sense { click: true, drag: false, focusable: false, scroll: false }, false).clicked {
            ui.request_focus(ed_id);
        }

        let f_t = ui.fade(id, 0, focused, 0.08, 0.16, Ease::Linear);
        let h_t = ui.fade(id, 1, hovered, 0.08, 0.16, Ease::Linear);
        let halo_w = ui.spring(id, 2, if focused { 3.0 * ui.look().input.halo } else { 0.0 }, mo.snappy);

        let r_card = (ui.look().card.radius + 2.0).min(16.0);
        let bg_c = ui.color(Token::Surface);
        let bd_c = ui.color(Token::Border).mix(ui.color(Token::BorderStrong), h_t).mix(ui.color(Token::FocusRing), f_t);
        let halo_c = ui.color(Token::FocusHalo).multiply_alpha(f_t);

        let mut out = ComposerOut::default();

        // Check Cmd/Ctrl + Enter submission before inline_edit consumes keys.
        if focused && ui.modifiers().command() && ui.key_pressed_for(ed_id, Key::Enter) && !self.prompt.trim().is_empty() && !self.streaming {
            out.submitted = true;
        }

        ui.begin(
            Style {
                dir: Dir::Column,
                width: Size::FILL,
                pad: Sides::all(12.0),
                gap: 10.0,
                ..Default::default()
            },
            Some(frame_id),
            Sense { click: true, drag: false, focusable: false, scroll: false },
            CursorIcon::Text,
            false,
            Some(Box::new(move |cx, r| {
                if halo_w > 0.05 && halo_c.a > 0.0 {
                    cx.painter.ring(r.expand(halo_w), Corners::all(r_card + halo_w), halo_w, halo_c);
                }
                cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c);
            })),
        );

        // Attachment chips row (if any).
        if let Some(files) = self.attachments.as_deref_mut() {
            if !files.is_empty() {
                let mut rm_idx: Option<usize> = None;
                ui.begin(
                    Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, wrap: true, align: Align::Center, ..Default::default() },
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    None,
                );
                for (fi, name) in files.iter().enumerate() {
                    let chip_bg = ui.color(Token::SurfaceMuted);
                    let chip_bd = ui.color(Token::BorderSubtle);
                    let ic_c = ui.color(Token::AccentStrong);
                    let fg_c = ui.color(Token::Foreground);
                    ui.begin(
                        Style { dir: Dir::Row, pad: Sides::xy(8.0, 4.0), gap: 6.0, align: Align::Center, ..Default::default() },
                        None,
                        Sense::NONE,
                        CursorIcon::Default,
                        false,
                        Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(7.0), chip_bg, 1.0, chip_bd))),
                    );
                    ui.icon_leaf("paperclip", 12.0, ic_c);
                    let st_f = ui.text_style(TextRole::Caption);
                    ui.text_leaf(name.clone(), st_f, fg_c, false, false, Style::default());
                    if ui.field_icon_button(id.with(("rm_att", fi)), "x", 1.0, false) {
                        rm_idx = Some(fi);
                    }
                    ui.end();
                }
                ui.end();
                if let Some(ri) = rm_idx {
                    files.remove(ri);
                }
            }
        }

        // Multiline prompt editor.
        let mut opts = EditOpts::new(ui);
        opts.multiline = true;
        opts.rows = (2, 5);
        opts.placeholder = self.placeholder.clone();
        ui.inline_edit(ed_id, self.prompt, opts);

        // Bottom control bar: attach + model picker + spacer + context ring + send/stop.
        ui.begin(
            Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, align: Align::Center, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );

        if ui.scope("att_btn", |ui| ui.icon_button("paperclip").small().tooltip("Attach file").clicked()) {
            out.attach_clicked = true;
        }

        if !self.models.is_empty() {
            let cur_m = (*self.model_idx).min(self.models.len() - 1);
            let cur_name = self.models[cur_m].clone();
            let trig = ui.scope("mdl_btn", |ui| ui.button(&cur_name).ghost().small().icon("sparkles").trailing_icon("chevron-down").show());
            let models_cl = self.models.clone();
            let mut chosen = cur_m;
            ui.menu(&trig, |ui| {
                for (mi, m_lbl) in models_cl.iter().enumerate() {
                    if ui.menu_item(if mi == chosen { "check" } else { "" }, m_lbl).clicked() {
                        chosen = mi;
                    }
                }
            });
            *self.model_idx = chosen;
        }

        ui.spacer();

        // Context-window mini ring + percentage.
        let ctx_v = ui.anims.spring_from(id, 3, self.context_used, self.context_used, mo.smooth).clamp(0.0, 1.0);
        let ctx_col = if ctx_v > 0.85 {
            ui.color(Token::Warning)
        } else {
            ui.color(Token::Accent)
        };
        let ctx_track = ui.color(Token::Track);
        let ctx_txt = ui.color(Token::TextMuted);
        ui.begin(Style { dir: Dir::Row, gap: 5.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        ui.leaf(
            Style::default(),
            vec2(16.0, 16.0),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let c = r.center();
                cx.painter.arc_band(c, 5.0, 7.0, 0.0, TAU, ctx_track);
                if ctx_v > 0.01 {
                    cx.painter.arc_band(c, 5.0, 7.0, 0.0, ctx_v * TAU, ctx_col);
                }
            })),
        );
        let mut st_ctx = ui.text_style(TextRole::Caption);
        st_ctx.tabular = true;
        ui.text_leaf(format!("{:.0}% ctx", (ctx_v * 100.0).round()), st_ctx, ctx_txt, false, false, Style::default());
        ui.end();

        // Circular primary Send or Stop button.
        let can_send = self.streaming || !self.prompt.trim().is_empty();
        let send_id = id.with("send");
        let six = ui.interact(send_id, Sense::CLICK, !can_send);
        let (sk_act, sk_held) = key_activate(ui, &six, true);
        if (six.clicked || sk_act) && can_send {
            if self.streaming {
                out.stopped = true;
            } else {
                out.submitted = true;
            }
        }
        let (sh, sp) = ui.state_weights(&six, sk_held);
        let squash = ui.spring(send_id, 0, if (six.pressed || sk_held) && can_send { 0.92 } else { 1.0 }, mo.bouncy);
        let (ring_a, ring_gap) = ui.focus_anim(&six);
        let (s_fill, s_fg) = if !can_send {
            (ui.color(Token::SurfaceMuted), ui.color(Token::TextDisabled))
        } else if self.streaming {
            let f = blend3(ui.color(Token::Danger), ui.color(Token::DangerHover), ui.color(Token::DangerPressed), sh, sp);
            (f, ui.color(Token::DangerForeground))
        } else {
            let f = blend3(ui.color(Token::Accent), ui.color(Token::AccentHover), ui.color(Token::AccentPressed), sh, sp);
            (f, ui.color(Token::AccentForeground))
        };
        let ring_c = ui.color(Token::FocusRing);
        let is_stream = self.streaming;
        let btn_sz = ui.control_metrics(ControlSize::Sm).height.max(28.0);

        ui.leaf(
            Style::default(),
            vec2(btn_sz, btn_sz),
            Some(send_id),
            Sense::CLICK,
            if can_send { CursorIcon::Pointer } else { CursorIcon::Default },
            Some(Box::new(move |cx, r| {
                let rad = (btn_sz * 0.5) * squash;
                cx.painter.circle(r.center(), rad, s_fill, 0.0, Color::TRANSPARENT);
                if is_stream {
                    let sq = Rect::from_center_size(r.center(), vec2(8.5, 8.5));
                    cx.painter.rrect(sq, Corners::all(2.0), s_fg, 0.0, Color::TRANSPARENT);
                } else {
                    cx.icon("arrow-up", r.center(), 15.0, s_fg);
                }
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, btn_sz * 0.5, ring_a, ring_gap, ring_c)));
                }
            })),
        );

        ui.end(); // bottom bar
        ui.end(); // composer
        out
    }
}

// ============================================================== ChatThread

/// Role of a [`ChatMessage`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
}

/// Message in [`Ui::chat_thread`].
#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub author: String,
    pub body: String,
    pub citations: Vec<String>,
    pub time: Option<String>,
}

impl ChatMessage {
    pub fn user(body: impl Into<String>) -> Self {
        ChatMessage { role: ChatRole::User, author: "You".into(), body: body.into(), citations: vec![], time: None }
    }
    pub fn assistant(author: impl Into<String>, body: impl Into<String>) -> Self {
        ChatMessage { role: ChatRole::Assistant, author: author.into(), body: body.into(), citations: vec![], time: None }
    }
    pub fn citations(mut self, cites: &[&str]) -> Self {
        self.citations = cites.iter().map(|s| (*s).to_owned()).collect();
        self
    }
    pub fn time(mut self, t: impl Into<String>) -> Self {
        self.time = Some(t.into());
        self
    }
}

// ============================================================== DiffViewer

/// Unified diff viewer with `+added` / `-removed` header counters and dual line numbers.
pub struct DiffViewer<'u> {
    ui: &'u mut Ui,
    key: String,
    path: String,
    diff: String,
    shown: bool,
}

impl<'u> DiffViewer<'u> {
    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("diff_viewer", &self.key));
        let r_card = ui.look().card.radius;
        let top_rad = (r_card - 1.0).max(0.0);
        let card_bg = ui.color(Token::Surface);
        let card_bd = ui.color(Token::BorderSubtle);
        let hdr_bg = ui.color(Token::SurfaceMuted).multiply_alpha(0.65);
        let div_c = ui.color(Token::BorderSubtle);

        let lines: Vec<&str> = self.diff.lines().collect();
        let added = lines.iter().filter(|l| l.starts_with('+') && !l.starts_with("+++")).count();
        let removed = lines.iter().filter(|l| l.starts_with('-') && !l.starts_with("---")).count();

        ui.begin(
            Style { width: Size::FILL, dir: Dir::Column, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), card_bg, 1.0, card_bd))),
        );

        // Header bar.
        ui.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(12.0, 8.0), gap: 8.0, align: Align::Center, ..Default::default() },
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
        ui.icon_leaf("git-compare", 14.0, ic_c);
        let mut st_p = ui.text_style(TextRole::Mono);
        st_p.weight = 600;
        let fg_c = ui.color(Token::Foreground);
        ui.text_leaf(self.path.clone(), st_p, fg_c, false, true, Style { width: Size::FILL, ..Default::default() });
        let st_stat = ui.text_style(TextRole::Mono);
        let add_c = ui.color(Token::SuccessStrong);
        let rem_c = ui.color(Token::DangerStrong);
        ui.text_leaf(format!("+{added}"), st_stat, add_c, false, false, Style::default());
        ui.text_leaf(format!("-{removed}"), st_stat, rem_c, false, false, Style::default());
        ui.end();

        // Diff lines with old / new line numbers.
        let mut mono = ui.text_style(TextRole::Mono);
        mono.line_height = 19.0;
        let (mut old_ln, mut new_ln) = (1usize, 1usize);
        let total_lines = lines.len();

        ui.begin(Style { width: Size::FILL, dir: Dir::Column, pad: Sides::xy(0.0, 4.0), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, true, None);
        for (idx, &raw) in lines.iter().enumerate() {
            if raw.starts_with("@@") {
                // Parse `@@ -old,count +new,count @@` if present.
                if let Some(rest) = raw.strip_prefix("@@ -") {
                    let parts: Vec<&str> = rest.split_whitespace().collect();
                    if let Some(o) = parts.first() {
                        old_ln = o.split(',').next().and_then(|s| s.parse().ok()).unwrap_or(old_ln);
                    }
                    if let Some(n) = parts.get(1).and_then(|s| s.strip_prefix('+')) {
                        new_ln = n.split(',').next().and_then(|s| s.parse().ok()).unwrap_or(new_ln);
                    }
                }
                let hunk_bg = ui.color(Token::InfoSubtle).multiply_alpha(0.45);
                let hunk_fg = ui.color(Token::InfoStrong);
                let s = raw.to_owned();
                let m_cl = mono;
                ui.leaf(
                    Style { width: Size::FILL, ..Default::default() },
                    vec2(240.0, 21.0),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, r| {
                        cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.min.y, (r.width() - 2.0).max(0.0), r.height()), hunk_bg);
                        let g = cx.galley(&s, &m_cl);
                        cx.painter.text(cx.text, &g, vec2(r.min.x + 12.0, r.min.y + 1.0), hunk_fg);
                    })),
                );
                continue;
            }

            let is_add = raw.starts_with('+');
            let is_rem = raw.starts_with('-');
            let (o_str, n_str) = if is_add {
                let n = new_ln;
                new_ln += 1;
                (String::new(), n.to_string())
            } else if is_rem {
                let o = old_ln;
                old_ln += 1;
                (o.to_string(), String::new())
            } else {
                let (o, n) = (old_ln, new_ln);
                old_ln += 1;
                new_ln += 1;
                (o.to_string(), n.to_string())
            };

            let (row_bg, bar_c, txt_c, ln_c) = if is_add {
                (
                    ui.color(Token::SuccessSubtle).multiply_alpha(0.55),
                    ui.color(Token::Success),
                    ui.color(Token::SuccessStrong),
                    ui.color(Token::SuccessStrong).multiply_alpha(0.75),
                )
            } else if is_rem {
                (
                    ui.color(Token::DangerSubtle).multiply_alpha(0.55),
                    ui.color(Token::Danger),
                    ui.color(Token::DangerStrong),
                    ui.color(Token::DangerStrong).multiply_alpha(0.75),
                )
            } else {
                (Color::TRANSPARENT, Color::TRANSPARENT, ui.color(Token::Foreground), ui.color(Token::TextMuted))
            };

            let _ = (idx, total_lines);
            let s = raw.to_owned();
            let m_cl = mono;
            ui.leaf(
                Style { width: Size::FILL, ..Default::default() },
                vec2(240.0, 21.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    let inner = Rect::from_xywh(r.min.x + 1.0, r.min.y, (r.width() - 2.0).max(0.0), r.height());
                    if row_bg.a > 0.0 {
                        cx.painter.rect(inner, row_bg);
                        cx.painter.rect(Rect::from_xywh(inner.min.x, inner.min.y, 2.5, inner.height()), bar_c);
                    }
                    let y = r.min.y + (r.height() - m_cl.line_height) * 0.5;
                    if !o_str.is_empty() {
                        let go = cx.galley(&o_str, &m_cl);
                        cx.painter.text(cx.text, &go, vec2(r.min.x + 30.0 - go.width, y), ln_c);
                    }
                    if !n_str.is_empty() {
                        let gn = cx.galley(&n_str, &m_cl);
                        cx.painter.text(cx.text, &gn, vec2(r.min.x + 58.0 - gn.width, y), ln_c);
                    }
                    cx.painter.push_clip(Rect::from_xywh(r.min.x + 68.0, r.min.y, (r.width() - 74.0).max(0.0), r.height()));
                    let gs = cx.galley(&s, &m_cl);
                    cx.painter.text(cx.text, &gs, vec2(r.min.x + 68.0, y), txt_c);
                    cx.painter.pop_clip();
                })),
            );
        }
        ui.end();
        ui.end();

        let rect = ui.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }
}

impl Drop for DiffViewer<'_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

// ===================================================== Waterfall & Uptime

/// Distributed trace or HTTP network span in [`Ui::waterfall`].
#[derive(Clone, Debug)]
pub struct TraceSpan {
    pub name: String,
    pub service: String,
    pub start_ms: f32,
    pub duration_ms: f32,
    pub depth: usize,
    pub error: bool,
}

impl TraceSpan {
    pub fn new(name: impl Into<String>, service: impl Into<String>, start_ms: f32, duration_ms: f32, depth: usize) -> Self {
        TraceSpan { name: name.into(), service: service.into(), start_ms, duration_ms, depth, error: false }
    }
    pub fn error(mut self) -> Self {
        self.error = true;
        self
    }
}

/// Daily availability status in [`Ui::uptime_bars`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UptimeStatus {
    Operational,
    Degraded,
    Outage,
}

impl Ui {
    /// Collapsible AI reasoning / chain-of-thought disclosure with left accent rail.
    pub fn reasoning_block(
        &mut self,
        key: impl std::hash::Hash,
        summary: &str,
        duration: Option<&str>,
        open: &mut bool,
        body: impl FnOnce(&mut Ui),
    ) -> Response {
        let id = self.make_id(("reasoning", key));
        let hdr_id = id.with("hdr");
        let ix = self.interact(hdr_id, Sense::CLICK, false);
        let (k_act, k_held) = key_activate(self, &ix, true);
        let mut changed = false;
        if ix.clicked || k_act {
            *open = !*open;
            changed = true;
        }
        let is_open = *open;
        let mo = self.motion();
        let init_t = if is_open { 1.0 } else { 0.0 };
        let chev_t = self.anims.spring_from(id, 0, init_t, if is_open { 1.0 } else { 0.0 }, mo.snappy);
        let open_t = self.anims.spring_from(id, 1, init_t, if is_open { 1.0 } else { 0.0 }, mo.smooth).clamp(0.0, 1.0);
        let (h, p) = self.state_weights(&ix, k_held);
        let (ring_a, ring_gap) = self.focus_anim(&ix);

        let ic_c = self.color(Token::AccentStrong);
        let fg_c = self.color(Token::TextSecondary).mix(self.color(Token::Foreground), h.max(p));
        let muted_c = self.color(Token::TextMuted);
        let rail_c = self.color(Token::Accent).multiply_alpha(0.45);
        let ring_c = self.color(Token::FocusRing);

        self.begin(Style { width: Size::FILL, dir: Dir::Column, gap: 4.0, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);

        self.begin(
            Style { dir: Dir::Row, gap: 7.0, pad: Sides::xy(4.0, 4.0), align: Align::Center, ..Default::default() },
            Some(hdr_id),
            Sense::CLICK,
            CursorIcon::Pointer,
            false,
            Some(Box::new(move |cx, r| {
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, r| cx.focus_ring(r, 6.0, ring_a, ring_gap, ring_c)));
                }
            })),
        );
        self.icon_leaf("brain", 14.0, ic_c);
        let st_s = self.text_style(TextRole::BodyStrong);
        self.text_leaf(summary.to_owned(), st_s, fg_c, false, false, Style::default());
        if let Some(dur) = duration {
            let st_d = self.text_style(TextRole::Caption);
            self.text_leaf(dur.to_owned(), st_d, muted_c, false, false, Style::default());
        }
        let angle = chev_t * std::f32::consts::FRAC_PI_2;
        self.leaf(
            Style::default(),
            vec2(12.0, 12.0),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| draw_chevron(&mut cx.painter, r.center(), 3.6, angle, 1.5, muted_c))),
        );
        self.end();

        if is_open || open_t > 0.01 {
            self.begin(
                Style {
                    dir: Dir::Column,
                    width: Size::FILL,
                    pad: Sides { left: 16.0, right: 6.0, top: 2.0, bottom: 4.0 },
                    gap: 6.0,
                    ..Default::default()
                },
                None,
                Sense::NONE,
                CursorIcon::Default,
                true,
                Some(Box::new(move |cx, r| {
                    cx.painter.rrect(Rect::from_xywh(r.min.x + 6.0, r.min.y + 2.0, 2.0, (r.height() - 4.0).max(0.0)), Corners::all(1.0), rail_c.multiply_alpha(open_t), 0.0, Color::TRANSPARENT);
                })),
            );
            self.scope("cot_body", body);
            self.end();
        }

        self.end();
        let rect = self.rect_of(id).unwrap_or(ix.rect);
        Response { id, rect, hovered: ix.hovered, clicked: changed, changed, focused: ix.focused }
    }

    /// Compact structured AI tool invocation pill/row.
    pub fn tool_call(&mut self, key: impl std::hash::Hash, call: &ToolCall) -> Response {
        let id = self.make_id(("tool_call", key));
        let bg = self.color(Token::SurfaceMuted).multiply_alpha(0.65);
        let bd = self.color(Token::BorderSubtle);
        let (st_ic, st_col) = match call.status {
            ToolStatus::Running => ("zap", self.color(Token::AccentStrong)),
            ToolStatus::Done => ("circle-check", self.color(Token::SuccessStrong)),
            ToolStatus::Error => ("circle-alert", self.color(Token::DangerStrong)),
        };
        let fn_c = self.color(Token::Foreground);
        let arg_c = self.color(Token::TextSecondary);
        let dur_c = self.color(Token::TextMuted);

        self.begin(
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                pad: Sides::xy(10.0, 6.0),
                gap: 8.0,
                align: Align::Center,
                ..Default::default()
            },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(8.0), bg, 1.0, bd))),
        );
        self.icon_leaf(st_ic, 14.0, st_col);
        let mut st_fn = self.text_style(TextRole::Mono);
        st_fn.weight = 600;
        self.text_leaf(call.name.clone(), st_fn, fn_c, false, false, Style::default());
        let st_arg = self.text_style(TextRole::Mono);
        self.text_leaf(call.args.clone(), st_arg, arg_c, false, true, Style { width: Size::FILL, ..Default::default() });
        if let Some(ref d) = call.duration {
            let st_d = self.text_style(TextRole::Caption);
            self.text_leaf(d.clone(), st_d, dur_c, false, false, Style::default());
        }
        self.end();

        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Rich AI prompt composer with attachments, model selector and send/stop button.
    pub fn prompt_composer<'u>(
        &'u mut self,
        key: impl Into<String>,
        prompt: &'u mut String,
        model_idx: &'u mut usize,
    ) -> PromptComposer<'u> {
        PromptComposer {
            ui: self,
            key: key.into(),
            prompt,
            model_idx,
            models: vec!["Claude 3.7".into(), "Gemini 2.5 Pro".into(), "o3-mini".into()],
            attachments: None,
            placeholder: "Ask anything or @-mention a crate…".into(),
            context_used: 0.24,
            streaming: false,
        }
    }

    /// Structured AI / user conversation feed with citation chips and copy buttons.
    pub fn chat_thread(&mut self, key: impl std::hash::Hash, messages: &[ChatMessage]) -> Response {
        let id = self.make_id(("chat_thread", key));
        self.begin(Style { width: Size::FILL, dir: Dir::Column, gap: 14.0, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);

        for (i, msg) in messages.iter().enumerate() {
            let is_user = msg.role == ChatRole::User;
            if is_user {
                let bub_bg = self.color(Token::AccentSubtle);
                let bub_bd = self.color(Token::Accent).multiply_alpha(0.28);
                let fg_c = self.color(Token::Foreground);
                self.begin(Style { dir: Dir::Row, width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                self.spacer();
                self.begin(
                    Style {
                        dir: Dir::Column,
                        max: vec2(420.0, f32::INFINITY),
                        pad: Sides::xy(14.0, 9.0),
                        ..Default::default()
                    },
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners([14.0, 14.0, 4.0, 14.0]), bub_bg, 1.0, bub_bd))),
                );
                let st_b = self.text_style(TextRole::Body);
                self.text_leaf(msg.body.clone(), st_b, fg_c, true, false, Style::default());
                self.end();
                self.end();
            } else {
                let badge_bg = self.color(Token::AccentSubtle);
                let badge_ic = self.color(Token::AccentStrong);
                let author_c = self.color(Token::Foreground);
                let time_c = self.color(Token::TextMuted);
                let body_c = self.color(Token::Foreground);
                self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 10.0, align: Align::Start, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                self.leaf(
                    Style::default(),
                    vec2(26.0, 26.0),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, r| {
                        cx.painter.circle(r.center(), 13.0, badge_bg, 0.0, Color::TRANSPARENT);
                        cx.icon("sparkles", r.center(), 13.0, badge_ic);
                    })),
                );
                self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                self.begin(Style { dir: Dir::Row, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                let st_a = self.text_style(TextRole::BodyStrong);
                self.text_leaf(msg.author.clone(), st_a, author_c, false, false, Style::default());
                if let Some(ref t) = msg.time {
                    let st_t = self.text_style(TextRole::Caption);
                    self.text_leaf(t.clone(), st_t, time_c, false, false, Style::default());
                }
                self.end();

                let st_b = self.text_style(TextRole::Body);
                self.text_leaf(msg.body.clone(), st_b, body_c, true, false, Style { width: Size::FILL, ..Default::default() });

                if !msg.citations.is_empty() {
                    self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, wrap: true, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                    for (ci, cite) in msg.citations.iter().enumerate() {
                        let c_bg = self.color(Token::SurfaceMuted);
                        let c_bd = self.color(Token::BorderSubtle);
                        let num_c = self.color(Token::AccentStrong);
                        let lbl_c = self.color(Token::TextSecondary);
                        self.begin(
                            Style { dir: Dir::Row, pad: Sides::xy(7.0, 2.5), gap: 5.0, align: Align::Center, ..Default::default() },
                            None,
                            Sense::NONE,
                            CursorIcon::Default,
                            false,
                            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(6.0), c_bg, 1.0, c_bd))),
                        );
                        let mut st_n = self.text_style(TextRole::Mono);
                        st_n.size = (st_n.size - 1.0).max(10.0);
                        st_n.weight = 600;
                        self.text_leaf(format!("[{}]", ci + 1), st_n, num_c, false, false, Style::default());
                        let st_c = self.text_style(TextRole::Caption);
                        self.text_leaf(cite.clone(), st_c, lbl_c, false, false, Style::default());
                        self.end();
                    }
                    self.end();
                }
                let _ = i;
                self.end();
                self.end();
            }
        }

        self.end();
        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Unified code diff viewer with dual line numbers and `+` / `-` stats.
    pub fn diff_viewer(&mut self, key: impl Into<String>, path: impl Into<String>, diff: impl Into<String>) -> DiffViewer<'_> {
        DiffViewer { ui: self, key: key.into(), path: path.into(), diff: diff.into(), shown: false }
    }

    /// Network / distributed-trace span waterfall timeline with ruler and hover tooltips.
    pub fn waterfall(&mut self, key: impl std::hash::Hash, spans: &[TraceSpan]) -> Response {
        let id = self.make_id(("waterfall", key));
        let ix = self.interact(id, Sense { click: true, drag: false, focusable: false, scroll: false }, false);
        let mo = self.motion();
        let enter = self.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);

        let total_ms = spans.iter().map(|s| s.start_ms + s.duration_ms).fold(1.0f32, f32::max);
        let r_card = self.look().card.radius;
        let top_rad = (r_card - 1.0).max(0.0);
        let card_bg = self.color(Token::Surface);
        let card_bd = self.color(Token::BorderSubtle);
        let hdr_bg = self.color(Token::SurfaceMuted).multiply_alpha(0.6);
        let div_c = self.color(Token::BorderSubtle);
        let grid_c = self.color(Token::BorderSubtle).multiply_alpha(0.65);
        let fg_c = self.color(Token::Foreground);
        let muted_c = self.color(Token::TextMuted);
        let on_bar_c = self.color(Token::AccentForeground);
        let hov_bg = self.color(Token::Hover);
        let series_cols = [
            self.color(Token::Series0),
            self.color(Token::Series1),
            self.color(Token::Series2),
            self.color(Token::Series3),
        ];
        let err_col = self.color(Token::Danger);
        let tip = Tip::new(self);

        let st_hdr = self.text_style(TextRole::Caption);
        let mut st_mono = self.text_style(TextRole::Mono);
        st_mono.size = (st_mono.size - 0.5).max(11.0);

        let row_h = 28.0;
        let ruler_h = 28.0;
        let total_h = ruler_h + spans.len() as f32 * row_h + 6.0;
        let spans_cl = spans.to_vec();
        let ptr = ix.pointer.filter(|_| ix.hovered);

        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(320.0, total_h),
            Some(id),
            Sense { click: true, drag: false, focusable: false, scroll: false },
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_card), card_bg, 1.0, card_bd);
                let hdr_r = Rect::from_xywh(r.min.x + 1.0, r.min.y + 1.0, (r.width() - 2.0).max(0.0), ruler_h - 1.0);
                cx.painter.rrect(hdr_r, Corners([top_rad, top_rad, 0.0, 0.0]), hdr_bg, 0.0, Color::TRANSPARENT);
                cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.min.y + ruler_h, (r.width() - 2.0).max(0.0), 1.0), div_c);

                let label_w = (r.width() * 0.38).clamp(130.0, 210.0);
                let tl_x0 = r.min.x + label_w;
                let tl_w = (r.max.x - 14.0 - tl_x0).max(60.0);

                let g_sp = cx.galley("Span", &st_hdr);
                cx.painter.text(cx.text, &g_sp, vec2(r.min.x + 12.0, r.min.y + (ruler_h - g_sp.height) * 0.5), muted_c);

                // 5 ruler ticks (0%, 25%, 50%, 75%, 100%). Labels thin out to
                // halves, then the ends, when a quarter is narrower than one.
                let widest = cx.galley(&format!("{:.0} ms", total_ms.round()), &st_hdr).width + 10.0;
                // The end label is right-aligned, so its neighbour needs 1.5 widths.
                let every = if tl_w / 4.0 >= widest * 1.5 { 1 } else if tl_w / 2.0 >= widest * 1.5 { 2 } else { 4 };
                for ti in 0..=4 {
                    let frac = ti as f32 / 4.0;
                    let x = tl_x0 + tl_w * frac;
                    cx.painter.rect(Rect::from_xywh(cx.painter.snap(x), r.min.y + ruler_h, 1.0, r.height() - ruler_h - 2.0), grid_c);
                    if ti % every != 0 {
                        continue;
                    }
                    let lbl = format!("{:.0} ms", (total_ms * frac).round());
                    let g = cx.galley(&lbl, &st_hdr);
                    let lx = if ti == 4 { x - g.width } else if ti == 0 { x + 3.0 } else { x - g.width * 0.5 };
                    cx.painter.text(cx.text, &g, vec2(lx.round(), r.min.y + (ruler_h - g.height) * 0.5), muted_c);
                }

                let mut hov_span: Option<(usize, Vec2)> = None;
                for (si, sp) in spans_cl.iter().enumerate() {
                    let ry = r.min.y + ruler_h + 2.0 + si as f32 * row_h;
                    let rr = Rect::from_xywh(r.min.x + 2.0, ry, r.width() - 4.0, row_h);
                    let is_hov = ptr.is_some_and(|p| rr.contains(p));
                    if is_hov {
                        cx.painter.rrect(rr, Corners::all(5.0), hov_bg, 0.0, Color::TRANSPARENT);
                    }

                    let indent = sp.depth as f32 * 12.0;
                    let g_nm = cx.galley_fit(&sp.name, &st_mono, label_w - 20.0 - indent);
                    cx.painter.text(cx.text, &g_nm, vec2(r.min.x + 12.0 + indent, ry + (row_h - g_nm.height) * 0.5), fg_c);

                    let bx = tl_x0 + (sp.start_ms / total_ms) * tl_w * enter;
                    let bw = ((sp.duration_ms / total_ms) * tl_w * enter).max(4.0);
                    let col = if sp.error { err_col } else { series_cols[sp.depth % series_cols.len()] };
                    let bar = Rect::from_xywh(bx, ry + 7.0, bw, row_h - 14.0);
                    cx.painter.rrect(bar, Corners::all(4.0), col.multiply_alpha(0.85), 0.0, Color::TRANSPARENT);

                    let dur_s = format!("{:.1} ms", sp.duration_ms);
                    let g_d = cx.galley(&dur_s, &st_hdr);
                    let (dx, d_col) = if bar.max.x + 6.0 + g_d.width < r.max.x - 6.0 {
                        (bar.max.x + 6.0, muted_c)
                    } else if bar.min.x - 6.0 - g_d.width >= tl_x0 + 2.0 {
                        (bar.min.x - 6.0 - g_d.width, muted_c)
                    } else {
                        (bar.max.x - 8.0 - g_d.width, on_bar_c)
                    };
                    cx.painter.text(cx.text, &g_d, vec2(dx.round(), ry + (row_h - g_d.height) * 0.5), d_col);

                    if is_hov {
                        hov_span = Some((si, bar.center()));
                    }
                }

                if let Some((si, anchor)) = hov_span {
                    let sp = &spans_cl[si];
                    let col = if sp.error { err_col } else { series_cols[sp.depth % series_cols.len()] };
                    let rows = vec![
                        (col, "Service".into(), sp.service.clone()),
                        (col, "Start / Duration".into(), format!("+{:.1} ms · {:.1} ms", sp.start_ms, sp.duration_ms)),
                    ];
                    let title = sp.name.clone();
                    let tip_cl = tip.clone();
                    let bounds = r.expand(10.0);
                    cx.defer(r, Box::new(move |cx, _| tooltip_panel(cx, anchor, bounds, &title, &rows, &tip_cl)));
                }
            })),
        );
        Response { id, rect: ix.rect, hovered: ix.hovered, ..Default::default() }
    }

    /// Service availability strip (e.g. 60 days) with status header, uptime `%`, and hover tooltip.
    pub fn uptime_bars(&mut self, key: impl std::hash::Hash, service: &str, statuses: &[UptimeStatus]) -> Response {
        let id = self.make_id(("uptime_bars", key));
        let bars_id = id.with("strip");
        let ix = self.interact(bars_id, Sense { click: true, drag: false, focusable: false, scroll: false }, false);
        let mo = self.motion();
        let enter = self.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);

        let n = statuses.len().max(1);
        let score: f32 = statuses
            .iter()
            .map(|s| match s {
                UptimeStatus::Operational => 1.0,
                UptimeStatus::Degraded => 0.96,
                UptimeStatus::Outage => 0.5,
            })
            .sum::<f32>()
            / n as f32;
        let pct = score * 100.0;

        let ok_c = self.color(Token::Success);
        let warn_c = self.color(Token::Warning);
        let err_c = self.color(Token::Danger);
        let fg_c = self.color(Token::Foreground);
        let muted_c = self.color(Token::TextMuted);
        let tip = Tip::new(self);

        let all_ok = statuses.iter().all(|s| *s == UptimeStatus::Operational);
        let hdr_dot = if all_ok { ok_c } else { warn_c };

        self.begin(Style { width: Size::FILL, dir: Dir::Column, gap: 8.0, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);

        // Header: dot + service name + spacer + uptime percentage.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.leaf(
            Style::default(),
            vec2(8.0, 8.0),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| cx.painter.circle(r.center(), 4.0, hdr_dot, 0.0, Color::TRANSPARENT))),
        );
        let st_s = self.text_style(TextRole::BodyStrong);
        self.text_leaf(service.to_owned(), st_s, fg_c, false, true, Style { width: Size::FILL, ..Default::default() });
        let mut st_p = self.text_style(TextRole::Caption);
        st_p.tabular = true;
        st_p.weight = 600;
        self.text_leaf(format!("{pct:.2}% uptime"), st_p, ok_c, false, false, Style::default());
        self.end();

        // Bar strip.
        let st_vec = statuses.to_vec();
        let ptr = ix.pointer.filter(|_| ix.hovered);
        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(240.0, 30.0),
            Some(bars_id),
            Sense { click: true, drag: false, focusable: false, scroll: false },
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                let gap = 2.5;
                let bw = ((r.width() - gap * (n.saturating_sub(1)) as f32) / n as f32).max(2.0);
                let mut hov_bar: Option<(usize, Rect)> = None;
                for (i, st) in st_vec.iter().enumerate() {
                    let x = r.min.x + i as f32 * (bw + gap);
                    let col = match st {
                        UptimeStatus::Operational => ok_c,
                        UptimeStatus::Degraded => warn_c,
                        UptimeStatus::Outage => err_c,
                    };
                    let bh = r.height() * enter;
                    let br = Rect::from_xywh(x, r.max.y - bh, bw, bh);
                    let is_h = ptr.is_some_and(|p| Rect::from_xywh(x - gap * 0.5, r.min.y, bw + gap, r.height()).contains(p));
                    let draw_r = if is_h { br.expand2(0.0, 2.0) } else { br };
                    cx.painter.rrect(draw_r, Corners::all((bw * 0.45).min(3.0)), col.multiply_alpha(if is_h { 1.0 } else { 0.86 }), 0.0, Color::TRANSPARENT);
                    if is_h {
                        hov_bar = Some((i, draw_r));
                    }
                }
                if let Some((bi, br)) = hov_bar {
                    let days_ago = n.saturating_sub(1 + bi);
                    let title = if days_ago == 0 { "Today".into() } else { format!("{days_ago}d ago") };
                    let (col, lbl) = match st_vec[bi] {
                        UptimeStatus::Operational => (ok_c, "100% operational"),
                        UptimeStatus::Degraded => (warn_c, "Elevated latency"),
                        UptimeStatus::Outage => (err_c, "Partial incident"),
                    };
                    let tip_cl = tip.clone();
                    let bounds = r.expand(20.0);
                    cx.defer(r, Box::new(move |cx, _| tooltip_panel(cx, br.center(), bounds, &title, &[(col, "Status".into(), lbl.into())], &tip_cl)));
                }
            })),
        );

        // Footer legend.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let st_f = self.text_style(TextRole::Caption);
        self.text_leaf(format!("{n} days ago"), st_f, muted_c, false, false, Style::default());
        self.spacer();
        let st_f2 = self.text_style(TextRole::Caption);
        self.text_leaf("Today", st_f2, muted_c, false, false, Style::default());
        self.end();

        self.end();
        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, hovered: ix.hovered, ..Default::default() }
    }
}
