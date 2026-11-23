//! Collaboration, communication, linear walkthroughs, shortcut bars, view
//! switchers, multi-stop OKLab gradient sliders, recurring schedule builders
//! and fiscal quarter pickers.

use std::ops::RangeInclusive;

use super::toast::Tone;
use crate::anim::Ease;
use crate::color::Color;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{ControlSize, Shadow, TextRole, Token};
use crate::ui::{Response, Sense, Ui};

// ============================================================= PresencePile

/// Collaborator peer in [`Ui::presence_pile`].
#[derive(Clone, Debug)]
pub struct PresencePeer {
    pub initials: String,
    pub name: String,
    pub status: String,
    pub tone: usize,
    pub editing: bool,
}

impl PresencePeer {
    pub fn new(initials: &str, name: impl Into<String>, status: impl Into<String>, tone: usize) -> Self {
        PresencePeer {
            initials: initials.to_owned(),
            name: name.into(),
            status: status.into(),
            tone,
            editing: false,
        }
    }

    pub fn editing(mut self, editing: bool) -> Self {
        self.editing = editing;
        self
    }
}

// =========================================================== CommentThread

/// Threaded review comment entry in [`Ui::comment_thread`].
#[derive(Clone, Debug)]
pub struct CommentEntry {
    pub author: String,
    pub initials: String,
    pub time: String,
    pub body: String,
    pub likes: u32,
    pub liked: bool,
    pub is_reply: bool,
}

impl CommentEntry {
    pub fn new(author: impl Into<String>, initials: &str, time: impl Into<String>, body: impl Into<String>) -> Self {
        CommentEntry {
            author: author.into(),
            initials: initials.to_owned(),
            time: time.into(),
            body: body.into(),
            likes: 0,
            liked: false,
            is_reply: false,
        }
    }

    pub fn reply(mut self) -> Self {
        self.is_reply = true;
        self
    }

    pub fn with_likes(mut self, likes: u32, liked: bool) -> Self {
        self.likes = likes;
        self.liked = liked;
        self
    }
}

// ============================================================= ReactionBar

/// Interactive reaction pill in [`Ui::reaction_bar`].
#[derive(Clone, Debug)]
pub struct ReactionChip {
    pub icon: String,
    pub label: String,
    pub count: u32,
    pub active: bool,
}

impl ReactionChip {
    pub fn new(icon: &str, label: impl Into<String>, count: u32, active: bool) -> Self {
        ReactionChip {
            icon: icon.to_owned(),
            label: label.into(),
            count,
            active,
        }
    }
}

// ========================================================== AttachmentList

/// File attachment entry in [`Ui::attachment_list`].
#[derive(Clone, Debug)]
pub struct AttachmentItem {
    pub name: String,
    pub ext: String,
    pub size_label: String,
    /// `None` when upload is complete; `Some(0.0..=1.0)` while uploading.
    pub progress: Option<f32>,
}

impl AttachmentItem {
    pub fn uploaded(name: impl Into<String>, ext: &str, size_label: impl Into<String>) -> Self {
        AttachmentItem {
            name: name.into(),
            ext: ext.to_owned(),
            size_label: size_label.into(),
            progress: None,
        }
    }

    pub fn uploading(name: impl Into<String>, ext: &str, size_label: impl Into<String>, progress: f32) -> Self {
        AttachmentItem {
            name: name.into(),
            ext: ext.to_owned(),
            size_label: size_label.into(),
            progress: Some(progress.clamp(0.0, 1.0)),
        }
    }
}

// ============================================================= ShortcutBar

/// Keyboard chord + action hint in [`Ui::shortcut_bar`].
#[derive(Clone, Debug)]
pub struct ShortcutHint {
    pub keys: Vec<String>,
    pub action: String,
}

impl ShortcutHint {
    pub fn new(keys: &[&str], action: impl Into<String>) -> Self {
        ShortcutHint {
            keys: keys.iter().map(|&k| k.to_owned()).collect(),
            action: action.into(),
        }
    }
}

// ============================================================ ViewSwitcher

/// Workspace view mode option in [`Ui::view_switcher`].
#[derive(Clone, Debug)]
pub struct ViewMode {
    pub icon: String,
    pub label: String,
}

impl ViewMode {
    pub fn new(icon: &str, label: impl Into<String>) -> Self {
        ViewMode { icon: icon.to_owned(), label: label.into() }
    }
}

// ========================================================== SchedulePicker

/// Recurring schedule configuration for [`Ui::schedule_picker`].
#[derive(Clone, Debug)]
pub struct ScheduleRule {
    /// `0 = Hourly`, `1 = Daily`, `2 = Weekly`.
    pub freq: usize,
    /// Monday (`0`) through Sunday (`6`).
    pub weekdays: [bool; 7],
    pub hour: u8,
    pub minute: u8,
}

impl Default for ScheduleRule {
    fn default() -> Self {
        ScheduleRule {
            freq: 2,
            weekdays: [true, false, true, false, true, false, false],
            hour: 9,
            minute: 30,
        }
    }
}

impl ScheduleRule {
    /// Formats a standard 5-field cron expression for this rule.
    pub fn cron_expr(&self) -> String {
        let m = self.minute.min(59);
        let h = self.hour.min(23);
        match self.freq {
            0 => format!("{m} * * * *"),
            1 => format!("{m} {h} * * *"),
            _ => {
                let mut days = Vec::new();
                for (i, &on) in self.weekdays.iter().enumerate() {
                    if on {
                        days.push(((i + 1) % 7).to_string());
                    }
                }
                let day_part = if days.is_empty() { "*".to_owned() } else { days.join(",") };
                format!("{m} {h} * * {day_part}")
            }
        }
    }
}

impl Ui {
    /// Overlapping multiplayer collaborator avatar stack with live presence
    /// rings, `+N` overflow counter, hover lift, and active peer status readout.
    pub fn presence_pile(
        &mut self,
        key: impl std::hash::Hash,
        peers: &[PresencePeer],
        max_visible: usize,
        following: &mut bool,
    ) -> Option<usize> {
        let id = self.make_id(("presence_pile", key));
        let mo = self.motion();
        let vis_cap = max_visible.max(1);
        let shown = peers.len().min(vis_cap);
        let overflow = peers.len().saturating_sub(shown);
        let editing_cnt = peers.iter().filter(|p| p.editing).count();
        let mut clicked_peer = None;
        let mut hovered_peer: Option<usize> = None;

        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);

        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(14.0, 12.0), gap: 14.0, align: Align::Center, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        // Overlapping avatar pile rendered on one crisp leaf so cutout rings overlap cleanly!
        let av_d = 34.0f32;
        let step_x = 29.0f32;
        let tot_items = shown + usize::from(overflow > 0);
        let pile_w = if tot_items == 0 { 0.0 } else { av_d + (tot_items - 1) as f32 * step_x + 6.0 };
        let pile_id = id.with("pile");
        let pile_ix = self.interact(pile_id, Sense::CLICK, false);

        if let Some(ptr) = pile_ix.pointer {
            if pile_ix.hovered {
                if let Some(pr) = self.rect_of(pile_id) {
                    let rel_x = ptr.x - pr.min.x;
                    for idx in (0..shown).rev() {
                        let cx_i = av_d * 0.5 + idx as f32 * step_x;
                        let dx = rel_x - cx_i;
                        let dy = ptr.y - pr.center().y;
                        if dx * dx + dy * dy <= (av_d * 0.52).powi(2) {
                            hovered_peer = Some(idx);
                            break;
                        }
                    }
                }
            }
        }
        if pile_ix.clicked {
            clicked_peer = hovered_peer.or(if shown > 0 { Some(0) } else { None });
        }

        let series = [
            self.color(Token::Series0),
            self.color(Token::Series2),
            self.color(Token::Series1),
            self.color(Token::Series3),
        ];
        let mut lifts = Vec::with_capacity(shown);
        for idx in 0..shown {
            let l = self.spring(id, idx as u32, if hovered_peer == Some(idx) { 1.0 } else { 0.0 }, mo.snappy);
            lifts.push(l);
        }

        let peers_snap: Vec<(String, Color, bool, f32)> = peers
            .iter()
            .take(shown)
            .enumerate()
            .map(|(i, p)| (p.initials.clone(), series[p.tone % series.len()], p.editing, lifts[i]))
            .collect();
        let surf_c = self.color(Token::Surface);
        let mut st_in = self.text_style(TextRole::Caption);
        st_in.weight = 600;
        let over_bg = self.color(Token::SurfaceMuted);
        let over_fg = self.color(Token::TextSecondary);
        let succ_c = self.color(Token::Success);

        self.leaf(
            Style::default(),
            vec2(pile_w.max(36.0), 40.0),
            Some(pile_id),
            Sense::CLICK,
            CursorIcon::Pointer,
            Some(Box::new(move |cx, r| {
                for (i, (init, col, _, lift)) in peers_snap.iter().enumerate() {
                    let c = vec2(r.min.x + av_d * 0.5 + i as f32 * step_x, r.center().y - lift * 3.5);
                    cx.painter.circle(c, av_d * 0.5 + 2.2, surf_c, 0.0, Color::TRANSPARENT);
                    cx.painter.circle(c, av_d * 0.5, col.with_alpha(0.20), 1.8, *col);
                    let g = cx.galley(init, &st_in);
                    cx.painter.text(cx.text, &g, c - vec2(g.width * 0.5, g.height * 0.5), *col);
                }
                if overflow > 0 {
                    let c = vec2(r.min.x + av_d * 0.5 + shown as f32 * step_x, r.center().y);
                    cx.painter.circle(c, av_d * 0.5 + 2.2, surf_c, 0.0, Color::TRANSPARENT);
                    cx.painter.circle(c, av_d * 0.5, over_bg, 1.2, bd_c);
                    let lbl = format!("+{overflow}");
                    let g = cx.galley(&lbl, &st_in);
                    cx.painter.text(cx.text, &g, c - vec2(g.width * 0.5, g.height * 0.5), over_fg);
                }
                for (i, (_, _, ed, lift)) in peers_snap.iter().enumerate() {
                    if *ed {
                        let c = vec2(r.min.x + av_d * 0.5 + i as f32 * step_x, r.center().y - lift * 3.5);
                        let dot = c + vec2(av_d * 0.34, av_d * 0.34);
                        cx.painter.circle(dot, 5.2, surf_c, 0.0, Color::TRANSPARENT);
                        cx.painter.circle(dot, 3.6, succ_c, 0.0, Color::TRANSPARENT);
                    }
                }
            })),
        );

        // Status column in the middle.
        self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 2.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let headline = if let Some(hi) = hovered_peer.and_then(|i| peers.get(i)) {
            hi.name.clone()
        } else {
            format!("{} collaborators online", peers.len())
        };
        let subline = if let Some(hi) = hovered_peer.and_then(|i| peers.get(i)) {
            hi.status.clone()
        } else {
            format!("{editing_cnt} editing canvas live")
        };
        self.text_leaf(headline, self.text_style(TextRole::BodyStrong), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
        self.text_leaf(subline, self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, true, Style { width: Size::FILL, ..Default::default() });
        self.end();

        // Follow mode toggle chip on the right.
        let _ = self.scope("follow_chip", |ui| ui.chip("Follow", following));
        self.end();

        clicked_peer
    }

    /// Threaded code / design review comment card with reply connector rails,
    /// interactive reaction pills, and a thread resolution toggle.
    pub fn comment_thread(
        &mut self,
        key: impl std::hash::Hash,
        title: &str,
        comments: &mut [CommentEntry],
        resolved: &mut bool,
    ) -> bool {
        let id = self.make_id(("comment_thread", key));
        let mo = self.motion();
        let r_card = self.look().card.radius;
        let top_rad = (r_card - 1.0).max(0.0);
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let hdr_bg = self.color(Token::SurfaceMuted).multiply_alpha(0.55);
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
        self.badge(if *resolved { "Resolved" } else { "Open" }, if *resolved { Tone::Success } else { Tone::Accent });
        self.text_leaf(title.to_owned(), self.text_style(TextRole::BodyStrong), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
        let btn_lbl = if *resolved { "Reopen" } else { "Resolve" };
        if self.scope("res_btn", |ui| ui.button(btn_lbl).small().ghost().clicked()) {
            *resolved = !*resolved;
            changed = true;
        }
        self.end();

        // Comment entries.
        self.begin(Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(14.0), gap: 12.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for (ci, c) in comments.iter_mut().enumerate() {
            let rail_c = self.color(Token::BorderStrong);
            let is_rep = c.is_reply;
            self.begin(
                Style {
                    dir: Dir::Row,
                    width: Size::FILL,
                    pad: Sides { left: if is_rep { 22.0 } else { 0.0 }, top: 0.0, right: 0.0, bottom: 0.0 },
                    gap: 10.0,
                    align: Align::Start,
                    ..Default::default()
                },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                if is_rep {
                    Some(Box::new(move |cx, r| {
                        cx.painter.rrect(Rect::from_xywh(r.min.x + 9.0, r.min.y + 2.0, 2.0, (r.height() - 4.0).max(4.0)), Corners::all(1.0), rail_c, 0.0, Color::TRANSPARENT);
                    }))
                } else {
                    None
                },
            );

            self.avatar(&c.initials, ci, 28.0, None);

            self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 4.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.text_leaf(c.author.clone(), self.text_style(TextRole::Label), self.color(Token::Foreground), false, false, Style::default());
            self.text_leaf(c.time.clone(), self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, true, Style { width: Size::FILL, ..Default::default() });

            // Like pill on the right of the comment header.
            let lk_id = id.with(("like", ci));
            let lk_ix = self.interact(lk_id, Sense::CLICK, false);
            if lk_ix.clicked {
                c.liked = !c.liked;
                if c.liked {
                    c.likes = c.likes.saturating_add(1);
                } else {
                    c.likes = c.likes.saturating_sub(1);
                }
                changed = true;
            }
            let act_t = self.spring(lk_id, 0, if c.liked { 1.0 } else { 0.0 }, mo.bouncy).clamp(0.0, 1.2);
            let hov_t = self.fade(lk_id, 1, lk_ix.hovered, mo.hover_in, mo.hover_out, Ease::Out);
            let pill_bg = self.color(Token::SurfaceMuted).mix(self.color(Token::Hover), hov_t).mix(self.color(Token::AccentSubtle), act_t.min(1.0));
            let pill_bd = self.color(Token::BorderSubtle).mix(self.color(Token::Accent), act_t.min(1.0));
            let pill_fg = self.color(Token::TextSecondary).mix(self.color(Token::AccentStrong), act_t.min(1.0));
            let cnt_str = format!("{}", c.likes);
            let mut st_cnt = self.text_style(TextRole::Caption);
            st_cnt.tabular = true;
            st_cnt.weight = 600;

            self.leaf(
                Style::default(),
                vec2(46.0, 22.0),
                Some(lk_id),
                Sense::CLICK,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    cx.painter.rrect(r, Corners::all(11.0), pill_bg, 1.0, pill_bd);
                    cx.icon("heart", vec2(r.min.x + 13.0, r.center().y), 11.5, pill_fg);
                    let g = cx.galley(&cnt_str, &st_cnt);
                    cx.painter.text(cx.text, &g, vec2(r.min.x + 24.0, r.center().y - g.height * 0.5), pill_fg);
                })),
            );
            self.end();

            self.text_leaf(c.body.clone(), self.text_style(TextRole::Body), self.color(Token::TextSecondary), true, false, Style { width: Size::FILL, ..Default::default() });
            self.end();
            self.end();
        }
        self.end();
        self.end();

        changed
    }

    /// Interactive Lucide reaction chip strip with bouncy toggle states,
    /// live tabular counters, and a `+` quick-reaction trigger.
    pub fn reaction_bar(&mut self, key: impl std::hash::Hash, chips: &mut [ReactionChip]) -> Option<usize> {
        let id = self.make_id(("reaction_bar", key));
        let mo = self.motion();
        let mut toggled = None;

        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, wrap: true, align: Align::Center, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );

        for (ri, chip) in chips.iter_mut().enumerate() {
            let cid = id.with(("rx", ri));
            let ix = self.interact(cid, Sense::CLICK, false);
            if ix.clicked {
                chip.active = !chip.active;
                if chip.active {
                    chip.count = chip.count.saturating_add(1);
                } else {
                    chip.count = chip.count.saturating_sub(1);
                }
                toggled = Some(ri);
            }
            let sq = self.spring(cid, 0, if ix.pressed { 0.95 } else { 1.0 }, mo.bouncy);
            let on_t = self.spring(cid, 1, if chip.active { 1.0 } else { 0.0 }, mo.snappy).clamp(0.0, 1.0);
            let hov = self.fade(cid, 2, ix.hovered, mo.hover_in, mo.hover_out, Ease::Out);

            let bg_c = self
                .color(Token::Surface)
                .mix(self.color(Token::Hover), hov)
                .mix(self.color(Token::AccentSubtle), on_t);
            let bd_c = self.color(Token::BorderSubtle).mix(self.color(Token::Accent), on_t);
            let ic_c = self.color(Token::TextSecondary).mix(self.color(Token::AccentStrong), on_t);
            let fg_c = self.color(Token::Foreground).mix(self.color(Token::AccentStrong), on_t);
            let foc_c = if ix.focus_visible { self.color(Token::FocusRing) } else { Color::TRANSPARENT };

            let mut st_l = self.text_style(TextRole::Label);
            st_l.tabular = true;
            let txt = format!("{}  {}", chip.label, chip.count);
            let g_w = self.galley(&txt, &st_l).width;
            let ic_name = chip.icon.clone();

            self.leaf(
                Style::default(),
                vec2(32.0 + g_w + 12.0, 34.0),
                Some(cid),
                Sense::CLICK,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    let dr = Rect::from_center_size(r.center(), r.size() * sq);
                    if foc_c.a > 0.0 {
                        cx.painter.ring(dr.expand(2.0), Corners::all(19.0), 2.0, foc_c);
                    }
                    cx.painter.rrect(dr, Corners::all(17.0), bg_c, 1.0 + 0.4 * on_t, bd_c);
                    cx.icon(&ic_name, vec2(dr.min.x + 16.0, dr.center().y), 14.5, ic_c);
                    let g = cx.galley(&txt, &st_l);
                    cx.painter.text(cx.text, &g, vec2(dr.min.x + 28.0, dr.center().y - g.height * 0.5), fg_c);
                })),
            );
        }

        let _ = self.scope("rx_add", |ui| ui.icon_button("smile-plus").small().tooltip("Add reaction").show());
        self.end();

        toggled
    }

    /// File attachment cards with extension badges, upload progress bars or
    /// completion checks, file sizes, and interactive remove (`×`) buttons.
    pub fn attachment_list(&mut self, key: impl std::hash::Hash, items: &mut Vec<AttachmentItem>) -> Option<usize> {
        let id = self.make_id(("attachment_list", key));
        let mut removed = None;

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, gap: 8.0, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );

        for (ai, item) in items.iter().enumerate() {
            let r_card = (self.look().card.radius - 2.0).max(10.0);
            let bg_c = self.color(Token::Surface);
            let bd_c = self.color(Token::BorderSubtle);
            let tone = match item.ext.as_str() {
                "WASM" | "RS" => Tone::Accent,
                "WGSL" | "JSON" => Tone::Info,
                "PNG" | "SVG" => Tone::Success,
                _ => Tone::Neutral,
            };

            self.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(12.0, 10.0), gap: 12.0, align: Align::Center, ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
            );

            self.badge(&item.ext, tone);

            self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 4.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.text_leaf(item.name.clone(), self.text_style(TextRole::Label), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
            let mut st_sz = self.text_style(TextRole::Mono);
            st_sz.size = 11.0;
            st_sz.tabular = true;
            let right_meta = if let Some(p) = item.progress {
                format!("{} · {:.0}%", item.size_label, p * 100.0)
            } else {
                format!("{} · Ready", item.size_label)
            };
            self.text_leaf(right_meta, st_sz, self.color(Token::TextMuted), false, false, Style::default());
            self.end();

            if let Some(p) = item.progress {
                let frac = self.spring(id, ai as u32, p.clamp(0.0, 1.0), self.motion().smooth).clamp(0.0, 1.0);
                let tr_c = self.color(Token::Track);
                let fl_c = self.color(Token::Accent);
                self.leaf(
                    Style { width: Size::FILL, ..Default::default() },
                    vec2(160.0, 5.0),
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    Some(Box::new(move |cx, r| {
                        cx.painter.rrect(r, Corners::all(2.5), tr_c, 0.0, Color::TRANSPARENT);
                        cx.painter.rrect(Rect::from_xywh(r.min.x, r.min.y, (r.width() * frac).max(4.0), r.height()), Corners::all(2.5), fl_c, 0.0, Color::TRANSPARENT);
                    })),
                );
            }
            self.end();

            if self.scope(("att_rm", ai), |ui| ui.icon_button("x").small().tooltip("Remove attachment").clicked()) {
                removed = Some(ai);
            }
            self.end();
        }
        self.end();

        if let Some(idx) = removed {
            if idx < items.len() {
                items.remove(idx);
            }
        }
        removed
    }

    /// Segmented linear onboarding / walkthrough story progress bars with
    /// clickable segments, spring fill transitions, and Prev / Next controls.
    pub fn story_bars(&mut self, key: impl std::hash::Hash, steps: &[(&str, &str)], current: &mut usize) -> bool {
        let id = self.make_id(("story_bars", key));
        let n = steps.len().max(1);
        *current = (*current).min(n - 1);
        let mo = self.motion();
        let mut changed = false;

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

        // Segmented top progress bars!
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for si in 0..n {
            let sid = id.with(("seg", si));
            let ix = self.interact(sid, Sense::CLICK, false);
            if ix.clicked && *current != si {
                *current = si;
                changed = true;
            }
            let target = if si < *current {
                1.0
            } else if si == *current {
                0.68
            } else {
                0.0
            };
            let fill = self.spring(sid, 0, target, mo.snappy).clamp(0.0, 1.0);
            let tr_c = self.color(Token::Track);
            let fl_c = self.color(if si < *current { Token::Accent } else { Token::AccentStrong });
            self.leaf(
                Style { width: Size::FILL, ..Default::default() },
                vec2(48.0, 6.0),
                Some(sid),
                Sense::CLICK,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    cx.painter.rrect(r, Corners::all(3.0), tr_c, 0.0, Color::TRANSPARENT);
                    if fill > 0.01 {
                        cx.painter.rrect(Rect::from_xywh(r.min.x, r.min.y, (r.width() * fill).max(4.0), r.height()), Corners::all(3.0), fl_c, 0.0, Color::TRANSPARENT);
                    }
                })),
            );
        }
        self.end();

        // Active step copy.
        if let Some(&(title, body)) = steps.get(*current) {
            self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 4.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let mut st_idx = self.text_style(TextRole::Mono);
            st_idx.size = 11.5;
            st_idx.tabular = true;
            self.text_leaf(format!("STEP {:02} / {:02}", *current + 1, n), st_idx, self.color(Token::AccentStrong), false, false, Style::default());
            self.text_leaf(title.to_owned(), self.text_style(TextRole::Heading2), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
            self.text_leaf(body.to_owned(), self.text_style(TextRole::Body), self.color(Token::TextSecondary), true, false, Style { width: Size::FILL, ..Default::default() });
            self.end();
        }

        // Footer navigation controls.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let at_start = *current == 0;
        let at_end = *current + 1 >= n;
        if self.scope("sb_prev", |ui| ui.button("Previous").ghost().small().disabled(at_start).clicked()) && !at_start {
            *current -= 1;
            changed = true;
        }
        self.begin(Style { width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.end();
        let next_lbl = if at_end { "Restart" } else { "Continue" };
        if self.scope("sb_next", |ui| ui.button(next_lbl).primary().small().clicked()) {
            *current = (*current + 1) % n;
            changed = true;
        }
        self.end();

        self.end();
        changed
    }

    /// Raycast / Linear-style bottom command hint footer pairing keyboard
    /// chords (`⌘ Enter`) with clickable actions.
    pub fn shortcut_bar(
        &mut self,
        key: impl std::hash::Hash,
        context_label: &str,
        hints: &[ShortcutHint],
    ) -> Option<usize> {
        let id = self.make_id(("shortcut_bar", key));
        let mo = self.motion();
        let r_bar = (self.look().card.radius - 2.0).max(10.0);
        let bg_c = self.color(Token::SurfaceRaised);
        let bd_c = self.color(Token::Border);
        let sh_c = self.color(Token::ShadowInk);
        let mut triggered = None;

        // Hints that don't fit drop off the end (last frame's widths), so a
        // narrow bar keeps its context label and the first shortcuts.
        let bar_w = self.rect_of(id).map(|r| r.width());
        let mut shown = hints.len();
        if let Some(bar_w) = bar_w {
            let widths = &self.mem::<ShortcutBarMem>(id).widths;
            let avail = bar_w - 24.0 - 96.0;
            let mut used = 0.0;
            shown = 0;
            for i in 0..hints.len() {
                used += widths.get(i).copied().unwrap_or(0.0) + 10.0;
                if used > avail {
                    break;
                }
                shown = i + 1;
            }
        }

        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(12.0, 7.0), gap: 10.0, align: Align::Center, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                cx.painter.shadow(r, Corners::all(r_bar), &Shadow { x: 0.0, y: 4.0, blur: 12.0, spread: -2.0, color: sh_c.multiply_alpha(0.18) });
                cx.painter.rrect(r, Corners::all(r_bar), bg_c, 1.0, bd_c);
            })),
        );

        self.text_leaf(context_label.to_owned(), self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, true, Style { width: Size::FILL, ..Default::default() });

        for (hi, h) in hints.iter().enumerate().take(shown) {
            let hid = id.with(("sh", hi));
            if let Some(w) = self.rect_of(hid).map(|r| r.width()) {
                let m = self.mem::<ShortcutBarMem>(id);
                if m.widths.len() <= hi {
                    m.widths.resize(hi + 1, 0.0);
                }
                m.widths[hi] = w;
            }
            let ix = self.interact(hid, Sense::CLICK, false);
            if ix.clicked {
                triggered = Some(hi);
            }
            let hov = self.fade(hid, 0, ix.hovered, mo.hover_in, mo.hover_out, Ease::Out);
            let h_bg = self.color(Token::Hover).multiply_alpha(hov);

            self.begin(
                Style { dir: Dir::Row, pad: Sides::xy(6.0, 3.0), gap: 6.0, align: Align::Center, ..Default::default() },
                Some(hid),
                Sense::CLICK,
                CursorIcon::Pointer,
                false,
                Some(Box::new(move |cx, r| {
                    if hov > 0.005 {
                        cx.painter.rrect(r, Corners::all(6.0), h_bg, 0.0, Color::TRANSPARENT);
                    }
                })),
            );
            self.text_leaf(h.action.clone(), self.text_style(TextRole::Caption), self.color(Token::TextSecondary), false, false, Style::default());
            let key_refs: Vec<&str> = h.keys.iter().map(String::as_str).collect();
            self.scope(("sh_k", hi), |ui| ui.shortcut_keys(&key_refs));
            self.end();
        }

        self.end();
        triggered
    }

    /// Icon + label view-mode switcher (`List | Board | Calendar | Graph`) with
    /// a single spring-gliding elevated pill indicator and arrow-key navigation.
    pub fn view_switcher(&mut self, key: impl std::hash::Hash, modes: &[ViewMode], selected: &mut usize) -> bool {
        let id = self.make_id(("view_switcher", key));
        let n = modes.len().max(1);
        *selected = (*selected).min(n - 1);
        let mo = self.motion();
        let mut changed = false;

        // Compute segment widths up front so a single indicator pill glides across segments!
        let st_lbl = self.text_style(TextRole::Label);
        let mut seg_w = Vec::with_capacity(n);
        let mut seg_x = Vec::with_capacity(n);
        let pad_outer = 4.0f32;
        let gap = 4.0f32;
        let mut acc_x = pad_outer;
        for m in modes {
            let gw = self.galley(&m.label, &st_lbl).width;
            let w = 28.0 + gw + 14.0;
            seg_x.push(acc_x);
            seg_w.push(w);
            acc_x += w + gap;
        }
        let total_w = (acc_x - gap + pad_outer).max(80.0);
        let pill_x = self.spring(id, 0, seg_x.get(*selected).copied().unwrap_or(pad_outer), mo.snappy);
        let pill_w = self.spring(id, 1, seg_w.get(*selected).copied().unwrap_or(64.0), mo.snappy);

        let r_outer = self.look().control.radius[1] + 2.0;
        let r_pill = self.look().control.radius[1].max(6.0);
        let tr_bg = self.color(Token::SurfaceMuted);
        let tr_bd = self.color(Token::BorderSubtle);
        let pill_bg = self.color(Token::SurfaceRaised);
        let pill_bd = self.color(Token::Border);
        let sh_c = self.color(Token::ShadowInk);

        self.begin(
            Style { dir: Dir::Row, pad: Sides::all(pad_outer), gap, align: Align::Center, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                let _ = total_w;
                cx.painter.rrect(r, Corners::all(r_outer), tr_bg, 1.0, tr_bd);
                let pr = Rect::from_xywh(r.min.x + pill_x, r.min.y + pad_outer, pill_w, (r.height() - pad_outer * 2.0).max(20.0));
                cx.painter.shadow(pr, Corners::all(r_pill), &Shadow { x: 0.0, y: 1.5, blur: 5.0, spread: -1.0, color: sh_c.multiply_alpha(0.16) });
                cx.painter.rrect(pr, Corners::all(r_pill), pill_bg, 1.0, pill_bd);
            })),
        );

        for (mi, m) in modes.iter().enumerate() {
            let mid = id.with(("vm", mi));
            let ix = self.interact(mid, Sense::CLICK, false);
            if ix.focused {
                if self.key_pressed_for(mid, Key::ArrowRight) {
                    *selected = (*selected + 1) % n;
                    changed = true;
                } else if self.key_pressed_for(mid, Key::ArrowLeft) {
                    *selected = (*selected + n - 1) % n;
                    changed = true;
                }
            }
            if ix.clicked && *selected != mi {
                *selected = mi;
                changed = true;
            }
            let is_sel = mi == *selected;
            let sel_t = self.spring(mid, 0, if is_sel { 1.0 } else { 0.0 }, mo.snappy).clamp(0.0, 1.0);
            let sq = self.spring(mid, 1, if ix.pressed { 0.96 } else { 1.0 }, mo.snappy);
            let fg_c = self.color(Token::TextSecondary).mix(self.color(Token::Foreground), sel_t);
            let ic_c = self.color(Token::TextMuted).mix(self.color(Token::AccentStrong), sel_t);
            let ic_n = m.icon.clone();
            let lbl_n = m.label.clone();
            let w_i = seg_w[mi];

            self.leaf(
                Style::default(),
                vec2(w_i, 30.0),
                Some(mid),
                Sense::CLICK,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    let dr = Rect::from_center_size(r.center(), r.size() * sq);
                    cx.icon(&ic_n, vec2(dr.min.x + 15.0, dr.center().y), 14.0, ic_c);
                    let g = cx.galley(&lbl_n, &st_lbl);
                    cx.painter.text(cx.text, &g, vec2(dr.min.x + 27.0, dr.center().y - g.height * 0.5), fg_c);
                })),
            );
        }
        self.end();

        changed
    }

    /// Continuous multi-stop perceptual OKLab color/temperature track slider
    /// with a spring-scaled elevated thumb tinted with the sampled stop color.
    pub fn gradient_slider(
        &mut self,
        key: impl std::hash::Hash,
        label: &str,
        value: &mut f32,
        range: RangeInclusive<f32>,
        stops: &[Color],
        unit: &str,
    ) -> Response {
        let id = self.make_id(("gradient_slider", key));
        let lo = *range.start();
        let hi = (*range.end()).max(lo + 1e-5);
        *value = value.clamp(lo, hi);
        let mo = self.motion();

        let sample_stop = |t: f32, palette: &[Color]| -> Color {
            if palette.is_empty() {
                return Color::WHITE;
            }
            if palette.len() == 1 {
                return palette[0];
            }
            let pos = t.clamp(0.0, 1.0) * (palette.len() - 1) as f32;
            let i0 = (pos.floor() as usize).min(palette.len() - 2);
            let frac = pos - i0 as f32;
            palette[i0].mix(palette[i0 + 1], frac)
        };

        let palette: Vec<Color> = if stops.len() >= 2 {
            stops.to_vec()
        } else {
            vec![self.color(Token::Series0), self.color(Token::Series2), self.color(Token::Warning), self.color(Token::Danger)]
        };

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, gap: 8.0, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );

        self.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf(label.to_owned(), self.text_style(TextRole::Label), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
        let mut st_v = self.text_style(TextRole::Mono);
        st_v.tabular = true;
        st_v.weight = 600;
        let readout = if (hi - lo) >= 100.0 {
            format!("{:.0}{unit}", *value)
        } else {
            format!("{:.2}{unit}", *value)
        };
        self.text_leaf(readout, st_v, self.color(Token::Foreground), false, false, Style::default());
        self.end();

        let ix = self.interact(id, Sense::DRAG, false);
        let mut changed = false;
        if ix.held {
            if let (Some(ptr), Some(r)) = (ix.pointer, self.rect_of(id)) {
                let pad_x = 10.0;
                let usable = (r.width() - pad_x * 2.0).max(1.0);
                let t = ((ptr.x - (r.min.x + pad_x)) / usable).clamp(0.0, 1.0);
                let nv = lo + t * (hi - lo);
                if (nv - *value).abs() > 1e-4 {
                    *value = nv;
                    changed = true;
                }
            }
        }
        if ix.focused {
            let step = (hi - lo) * 0.05;
            if self.key_pressed_for(id, Key::ArrowRight) {
                *value = (*value + step).min(hi);
                changed = true;
            } else if self.key_pressed_for(id, Key::ArrowLeft) {
                *value = (*value - step).max(lo);
                changed = true;
            } else if self.key_pressed_for(id, Key::Home) {
                *value = lo;
                changed = true;
            } else if self.key_pressed_for(id, Key::End) {
                *value = hi;
                changed = true;
            }
        }

        let frac = ((*value - lo) / (hi - lo)).clamp(0.0, 1.0);
        let anim_f = self.spring(id, 0, frac, mo.snappy).clamp(0.0, 1.0);
        let th_scale = self.spring(id, 1, if ix.held { 1.12 } else if ix.hovered { 1.05 } else { 1.0 }, mo.snappy);
        let sampled = sample_stop(anim_f, &palette);
        let bd_c = self.color(Token::Border);
        let thumb_bg = self.color(Token::Knob);
        let sh_c = self.color(Token::ShadowInk);
        let foc_c = if ix.focus_visible { self.color(Token::FocusRing) } else { Color::TRANSPARENT };

        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(260.0, 28.0),
            Some(id),
            Sense::DRAG,
            if ix.held { CursorIcon::Grabbing } else { CursorIcon::Grab },
            Some(Box::new(move |cx, r| {
                let pad_x = 10.0;
                let tr_h = 12.0;
                let tr = Rect::from_xywh(r.min.x + pad_x, r.center().y - tr_h * 0.5, (r.width() - pad_x * 2.0).max(20.0), tr_h);
                let rad = tr_h * 0.5;

                // Multi-stop horizontal gradient segments using GPU `rrect_horiz`.
                let segs = (palette.len() - 1).max(1);
                let seg_w = tr.width() / segs as f32;
                for s in 0..segs {
                    let sr = Rect::from_xywh(tr.min.x + s as f32 * seg_w, tr.min.y, seg_w + if s + 1 < segs { 0.5 } else { 0.0 }, tr.height());
                    let corners = match (s == 0, s + 1 == segs) {
                        (true, true) => Corners::all(rad),
                        (true, false) => Corners([rad, 0.0, 0.0, rad]),
                        (false, true) => Corners([0.0, rad, rad, 0.0]),
                        (false, false) => Corners::all(0.0),
                    };
                    cx.painter.rrect_horiz(sr, corners, palette[s], palette[(s + 1).min(palette.len() - 1)]);
                }
                cx.painter.ring(tr, Corners::all(rad), 1.0, bd_c);

                let tc = vec2(tr.min.x + anim_f * tr.width(), tr.center().y);
                let th_r = 10.0 * th_scale;
                let th_rect = Rect::from_center_size(tc, vec2(th_r * 2.0, th_r * 2.0));
                if foc_c.a > 0.0 {
                    cx.painter.ring(th_rect.expand(2.5), Corners::all(th_r + 2.5), 2.0, foc_c);
                }
                cx.painter.shadow(th_rect, Corners::all(th_r), &Shadow { x: 0.0, y: 2.0, blur: 6.0, spread: -1.0, color: sh_c.multiply_alpha(0.22) });
                cx.painter.circle(tc, th_r, thumb_bg, 1.5, bd_c);
                cx.painter.circle(tc, th_r * 0.48, sampled, 0.0, Color::TRANSPARENT);
            })),
        );
        self.end();

        Response { id, rect: ix.rect, hovered: ix.hovered, clicked: ix.clicked, changed, focused: ix.focused }
    }

    /// Recurring cron / job schedule builder with `Hourly | Daily | Weekly`
    /// cadence, `M..S` weekday pills, time stepper, and live cron expression.
    pub fn schedule_picker(&mut self, key: impl std::hash::Hash, rule: &mut ScheduleRule) -> bool {
        let id = self.make_id(("schedule_picker", key));
        let mo = self.motion();
        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let mut changed = false;

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(16.0), gap: 12.0, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        // Top row: Cadence segmented + Cron badge.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 10.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if self.scope("sp_freq", |ui| ui.segmented(&mut rule.freq, &[(0usize, "Hourly"), (1, "Daily"), (2, "Weekly")]).changed()) {
            changed = true;
        }
        self.begin(Style { width: Size::FILL, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.end();
        self.badge(rule.cron_expr(), Tone::Accent);
        self.end();

        // Weekday pills row (`M T W T F S S`).
        let day_labels = ["M", "T", "W", "T", "F", "S", "S"];
        let weekly = rule.freq == 2;
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf("Days", self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, true, Style { width: Size::FILL, ..Default::default() });
        for (di, &dl) in day_labels.iter().enumerate() {
            let did = id.with(("wd", di));
            let ix = self.interact(did, Sense::CLICK, !weekly);
            if ix.clicked && weekly {
                rule.weekdays[di] = !rule.weekdays[di];
                changed = true;
            }
            let on = weekly && rule.weekdays[di];
            let t_on = self.spring(did, 0, if on { 1.0 } else { 0.0 }, mo.snappy).clamp(0.0, 1.0);
            let sq = self.spring(did, 1, if ix.pressed { 0.94 } else { 1.0 }, mo.snappy);
            let c_bg = self.color(Token::SurfaceMuted).mix(self.color(Token::Accent), t_on);
            let c_bd = self.color(Token::BorderSubtle).mix(self.color(Token::Accent), t_on);
            let c_fg = if !weekly {
                self.color(Token::TextDisabled)
            } else {
                self.color(Token::TextSecondary).mix(self.color(Token::AccentForeground), t_on)
            };
            let mut st_d = self.text_style(TextRole::Caption);
            st_d.weight = 600;
            self.leaf(
                Style::default(),
                vec2(30.0, 30.0),
                Some(did),
                Sense::CLICK,
                if weekly { CursorIcon::Pointer } else { CursorIcon::Default },
                Some(Box::new(move |cx, r| {
                    let dr = Rect::from_center_size(r.center(), r.size() * sq);
                    cx.painter.rrect(dr, Corners::all(15.0), c_bg, 1.0, c_bd);
                    let g = cx.galley(dl, &st_d);
                    cx.painter.text(cx.text, &g, dr.center() - vec2(g.width * 0.5, g.height * 0.5), c_fg);
                })),
            );
        }
        self.end();

        // Time row (`At HH:MM UTC` with `-30m` / `+30m` steppers).
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf("Run window (UTC)", self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, true, Style { width: Size::FILL, ..Default::default() });
        if self.scope("sp_m", |ui| ui.icon_button("minus").small().clicked()) {
            let tot = (rule.hour as i32 * 60 + rule.minute as i32 - 30).rem_euclid(24 * 60);
            rule.hour = (tot / 60) as u8;
            rule.minute = (tot % 60) as u8;
            changed = true;
        }
        let mut st_tm = self.text_style(TextRole::Mono);
        st_tm.tabular = true;
        st_tm.weight = 600;
        let tm_txt = if rule.freq == 0 {
            format!(":{:02} past hour", rule.minute)
        } else {
            format!("{:02}:{:02} UTC", rule.hour, rule.minute)
        };
        self.text_leaf(tm_txt, st_tm, self.color(Token::Foreground), false, false, Style::default());
        if self.scope("sp_p", |ui| ui.icon_button("plus").small().clicked()) {
            let tot = (rule.hour as i32 * 60 + rule.minute as i32 + 30).rem_euclid(24 * 60);
            rule.hour = (tot / 60) as u8;
            rule.minute = (tot % 60) as u8;
            changed = true;
        }
        self.end();

        self.end();
        changed
    }

    /// Fiscal year stepper & `Q1..Q4` quarter selector cards with month-range
    /// subtitles and spring selection states.
    pub fn quarter_picker(&mut self, key: impl std::hash::Hash, year: &mut i32, quarter: &mut u8) -> bool {
        let id = self.make_id(("quarter_picker", key));
        *quarter = (*quarter).clamp(1, 4);
        let mo = self.motion();
        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let mut changed = false;

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(16.0), gap: 12.0, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        // Year navigation header.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if self.scope("qp_prev", |ui| ui.icon_button("chevron-left").small().clicked()) {
            *year -= 1;
            changed = true;
        }
        let mut st_yr = self.text_style(TextRole::BodyStrong);
        st_yr.tabular = true;
        self.begin(Style { width: Size::FILL, align: Align::Center, justify: Justify::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf(format!("FY {year}"), st_yr, self.color(Token::Foreground), false, false, Style::default());
        self.end();
        if self.scope("qp_next", |ui| ui.icon_button("chevron-right").small().clicked()) {
            *year += 1;
            changed = true;
        }
        self.end();

        // 4 quarter cards across, or 2x2 when that doesn't fit: each card is
        // at least as wide as its month range.
        let quarters = [("Q1", "Jan – Mar"), ("Q2", "Apr – Jun"), ("Q3", "Jul – Sep"), ("Q4", "Oct – Dec")];
        let (st_q, st_m) = (self.text_style(TextRole::BodyStrong), self.text_style(TextRole::Caption));
        let cell_min = quarters.iter().map(|(q, m)| self.galley(m, &st_m).width.max(self.galley(q, &st_q).width)).fold(0.0f32, f32::max).ceil() + 16.0;
        // Last frame's inner width picks the grid (the first frame assumes 4).
        let inner_w = self.rect_of(id).map(|r| r.width() - 32.0);
        let cols = if inner_w.is_some_and(|w| w < cell_min * 4.0 + 24.0) { 2 } else { 4 };
        self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 8.0, min: vec2(cell_min * 2.0 + 8.0, 0.0), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for (qi, &(q_code, months)) in quarters.iter().enumerate() {
            if qi % cols == 0 {
                self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 8.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            }
            let q_num = (qi + 1) as u8;
            let qid = id.with(("q", q_num));
            let ix = self.interact(qid, Sense::CLICK, false);
            if ix.focused {
                if self.key_pressed_for(qid, Key::ArrowRight) {
                    *quarter = if *quarter >= 4 { 1 } else { *quarter + 1 };
                    changed = true;
                } else if self.key_pressed_for(qid, Key::ArrowLeft) {
                    *quarter = if *quarter <= 1 { 4 } else { *quarter - 1 };
                    changed = true;
                }
            }
            if ix.clicked && *quarter != q_num {
                *quarter = q_num;
                changed = true;
            }
            let is_sel = *quarter == q_num;
            let sel_t = self.spring(qid, 0, if is_sel { 1.0 } else { 0.0 }, mo.snappy).clamp(0.0, 1.0);
            let sq = self.spring(qid, 1, if ix.pressed { 0.96 } else { 1.0 }, mo.snappy);
            let hov = self.fade(qid, 2, ix.hovered && !is_sel, mo.hover_in, mo.hover_out, Ease::Out);

            let c_bg = self
                .color(Token::SurfaceMuted)
                .mix(self.color(Token::Hover), hov)
                .mix(self.color(Token::AccentSubtle), sel_t);
            let c_bd = self.color(Token::BorderSubtle).mix(self.color(Token::Accent), sel_t);
            let fg_q = self.color(Token::Foreground).mix(self.color(Token::AccentStrong), sel_t);
            let fg_m = self.color(Token::TextMuted).mix(self.color(Token::AccentStrong), sel_t * 0.7);
            let st_q = self.text_style(TextRole::BodyStrong);
            let st_m = self.text_style(TextRole::Caption);
            let r_q = (self.look().control.radius[1] + 2.0).max(8.0);

            self.leaf(
                // In 4-across mode the cells may shrink (the next frame drops to
                // 2x2); a floor there would pin the card wide.
                Style { width: Size::FILL, min: vec2(if cols == 2 { cell_min } else { 0.0 }, 0.0), ..Default::default() },
                vec2(78.0, 62.0),
                Some(qid),
                Sense::CLICK,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    let dr = Rect::from_center_size(r.center(), r.size() * sq);
                    cx.painter.rrect(dr, Corners::all(r_q), c_bg, 1.0 + 0.5 * sel_t, c_bd);
                    let gq = cx.galley(q_code, &st_q);
                    let gm = cx.galley(months, &st_m);
                    cx.painter.text(cx.text, &gq, vec2(dr.center().x - gq.width * 0.5, dr.center().y - gq.height + 1.0), fg_q);
                    cx.painter.text(cx.text, &gm, vec2(dr.center().x - gm.width * 0.5, dr.center().y + 3.0), fg_m);
                })),
            );
            if qi % cols == cols - 1 {
                self.end();
            }
        }
        self.end();

        let _ = ControlSize::Md;
        self.end();
        changed
    }
}

#[derive(Default)]
struct ShortcutBarMem {
    /// Each hint's width when last shown.
    widths: Vec<f32>,
}
