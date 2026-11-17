//! Project planning & board widgets:
//! - [`Kanban`] (multi-column status board with tags, avatars, and pointer/keyboard column moves)

use std::f32::consts::PI;

use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{Elevation, TextRole, Token};
use crate::ui::{Sense, Ui};
use crate::Color;

use super::data::draw_chevron;
use super::toast::Tone;

/// Column definition for [`Ui::kanban`].
#[derive(Clone, Debug)]
pub struct KanbanCol {
    pub title: String,
    pub tone: Tone,
}

impl KanbanCol {
    pub fn new(title: impl Into<String>, tone: Tone) -> Self {
        KanbanCol { title: title.into(), tone }
    }
}

/// Task card placed inside one column of [`Ui::kanban`].
#[derive(Clone, Debug)]
pub struct KanbanCard {
    pub id: String,
    pub col: usize,
    pub title: String,
    pub tag: Option<(String, Tone)>,
    pub assignee: Option<String>,
    pub meta: Option<String>,
}

impl KanbanCard {
    pub fn new(id: impl Into<String>, col: usize, title: impl Into<String>) -> Self {
        KanbanCard {
            id: id.into(),
            col,
            title: title.into(),
            tag: None,
            assignee: None,
            meta: None,
        }
    }
    pub fn tag(mut self, label: impl Into<String>, tone: Tone) -> Self {
        self.tag = Some((label.into(), tone));
        self
    }
    pub fn assignee(mut self, initials: impl Into<String>) -> Self {
        self.assignee = Some(initials.into());
        self
    }
    pub fn meta(mut self, text: impl Into<String>) -> Self {
        self.meta = Some(text.into());
        self
    }
}

fn col_dot_token(tone: Tone) -> Token {
    match tone {
        Tone::Neutral => Token::TextMuted,
        Tone::Accent => Token::Accent,
        Tone::Success => Token::Success,
        Tone::Warning => Token::Warning,
        Tone::Info => Token::Info,
        Tone::Danger => Token::Danger,
    }
}

impl Ui {
    /// Multi-column Kanban board. Cards can be moved across columns via the
    /// hover `<` / `>` triggers or `←` / `→` when focused. Returns
    /// `Some((card_index, new_col))` when a card moves.
    pub fn kanban(&mut self, key: impl std::hash::Hash, cols: &[KanbanCol], cards: &mut [KanbanCard]) -> Option<(usize, usize)> {
        let id = self.make_id(("kanban", key));
        let n_cols = cols.len().max(1);
        let mo = self.motion();
        let mut moved: Option<(usize, usize)> = None;

        // Keyboard Left/Right move on any focused card.
        for (ci, card) in cards.iter_mut().enumerate() {
            let cid = id.with(("card", &card.id));
            if self.focused() == Some(cid) {
                if self.key_pressed_for(cid, Key::ArrowRight) && card.col + 1 < n_cols {
                    card.col += 1;
                    moved = Some((ci, card.col));
                } else if self.key_pressed_for(cid, Key::ArrowLeft) && card.col > 0 {
                    card.col -= 1;
                    moved = Some((ci, card.col));
                }
            }
        }

        let well_rad = self.look().card.radius;
        let card_rad = (well_rad - 2.0).max(6.0);

        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, gap: 12.0, align: Align::Stretch, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );

        for (col_idx, col_spec) in cols.iter().enumerate() {
            let count = cards.iter().filter(|c| c.col == col_idx).count();
            let well_bg = self.color(Token::SurfaceMuted).multiply_alpha(0.55);
            let well_bd = self.color(Token::BorderSubtle);
            let dot_c = self.color(col_dot_token(col_spec.tone));
            let hdr_fg = self.color(Token::Foreground);
            let cnt_bg = self.color(Token::Surface);
            let cnt_fg = self.color(Token::TextSecondary);

            self.begin(
                Style {
                    dir: Dir::Column,
                    width: Size::FILL,
                    pad: Sides::all(10.0),
                    gap: 8.0,
                    ..Default::default()
                },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| {
                    cx.painter.rrect(r, Corners::all(well_rad), well_bg, 1.0, well_bd);
                })),
            );

            // Column header.
            self.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(4.0, 2.0), gap: 7.0, align: Align::Center, ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                None,
            );
            self.leaf(
                Style::default(),
                vec2(8.0, 8.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| cx.painter.circle(r.center(), 4.0, dot_c, 0.0, Color::TRANSPARENT))),
            );
            let st_col = self.text_style(TextRole::BodyStrong);
            self.text_leaf(col_spec.title.clone(), st_col, hdr_fg, false, true, Style { width: Size::FILL, ..Default::default() });
            let mut st_cnt = self.text_style(TextRole::Caption);
            st_cnt.tabular = true;
            st_cnt.weight = 600;
            self.begin(
                Style { pad: Sides::xy(6.5, 1.5), ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r.height() * 0.5), cnt_bg, 1.0, well_bd))),
            );
            self.text_leaf(count.to_string(), st_cnt, cnt_fg, false, false, Style::default());
            self.end();
            self.end();

            // Cards belonging to this column.
            for (ci, card) in cards.iter_mut().enumerate() {
                if card.col != col_idx {
                    continue;
                }
                let cid = id.with(("card", &card.id));
                let prev_id = cid.with("mv_l");
                let next_id = cid.with("mv_r");

                let cix = self.interact(cid, Sense::CLICK, false);
                let can_left = col_idx > 0;
                let can_right = col_idx + 1 < n_cols;
                let l_ix = self.interact(prev_id, if can_left { Sense { click: true, drag: false, focusable: false, scroll: false } } else { Sense::NONE }, false);
                let r_ix = self.interact(next_id, if can_right { Sense { click: true, drag: false, focusable: false, scroll: false } } else { Sense::NONE }, false);

                if can_left && l_ix.clicked {
                    card.col = col_idx - 1;
                    moved = Some((ci, card.col));
                } else if can_right && r_ix.clicked {
                    card.col = col_idx + 1;
                    moved = Some((ci, card.col));
                }

                let any_hov = cix.hovered || l_ix.hovered || r_ix.hovered;
                let h = self.fade(cid, 0, any_hov, mo.hover_in, mo.hover_out, Ease::Linear);
                let lift = self.spring(cid, 1, if any_hov { 1.0 } else { 0.0 }, mo.snappy);
                let (ring_a, ring_gap) = self.focus_anim(&cix);

                let c_bg = self.color(Token::Surface);
                let c_bd = self.color(Token::BorderSubtle).mix(self.color(Token::BorderStrong), h);
                let ring_c = self.color(Token::FocusRing);
                let shadows = Elevation::Level1.shadows(self.theme.darkness(), self.color(Token::ShadowInk));
                let fg_c = self.color(Token::Foreground);
                let meta_c = self.color(Token::TextMuted);
                let btn_hov_l = self.color(Token::Hover).multiply_alpha(if l_ix.hovered { 1.0 } else { 0.0 });
                let btn_hov_r = self.color(Token::Hover).multiply_alpha(if r_ix.hovered { 1.0 } else { 0.0 });
                let arrow_l_c = if can_left { self.color(Token::TextSecondary) } else { self.color(Token::TextDisabled).multiply_alpha(0.45) };
                let arrow_r_c = if can_right { self.color(Token::TextSecondary) } else { self.color(Token::TextDisabled).multiply_alpha(0.45) };

                self.begin(
                    Style {
                        dir: Dir::Column,
                        width: Size::FILL,
                        pad: Sides::all(11.0),
                        gap: 8.0,
                        ..Default::default()
                    },
                    Some(cid),
                    Sense::CLICK,
                    CursorIcon::Pointer,
                    false,
                    Some(Box::new(move |cx, r| {
                        let lifted = Rect::from_xywh(r.min.x, r.min.y - lift * 1.0, r.width(), r.height());
                        cx.painter.shadows(lifted, Corners::all(card_rad), &shadows);
                        cx.painter.rrect(lifted, Corners::all(card_rad), c_bg, 1.0, c_bd);
                        if ring_a > 0.0 {
                            cx.defer(lifted, Box::new(move |cx, r| cx.focus_ring(r, card_rad, ring_a, ring_gap, ring_c)));
                        }
                    })),
                );

                // Top row: tag badge + move arrows.
                self.begin(
                    Style { dir: Dir::Row, width: Size::FILL, gap: 4.0, align: Align::Center, ..Default::default() },
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    None,
                );
                if let Some((ref t_lbl, t_tone)) = card.tag {
                    self.badge(t_lbl, t_tone);
                }
                self.spacer();
                self.leaf(
                    Style::default(),
                    vec2(20.0, 20.0),
                    Some(prev_id),
                    if can_left { Sense { click: true, drag: false, focusable: false, scroll: false } } else { Sense::NONE },
                    if can_left { CursorIcon::Pointer } else { CursorIcon::Default },
                    Some(Box::new(move |cx, r| {
                        if btn_hov_l.a > 0.0 {
                            cx.painter.rrect(r, Corners::all(5.0), btn_hov_l, 0.0, Color::TRANSPARENT);
                        }
                        draw_chevron(&mut cx.painter, r.center(), 3.6, PI, 1.5, arrow_l_c);
                    })),
                );
                self.leaf(
                    Style::default(),
                    vec2(20.0, 20.0),
                    Some(next_id),
                    if can_right { Sense { click: true, drag: false, focusable: false, scroll: false } } else { Sense::NONE },
                    if can_right { CursorIcon::Pointer } else { CursorIcon::Default },
                    Some(Box::new(move |cx, r| {
                        if btn_hov_r.a > 0.0 {
                            cx.painter.rrect(r, Corners::all(5.0), btn_hov_r, 0.0, Color::TRANSPARENT);
                        }
                        draw_chevron(&mut cx.painter, r.center(), 3.6, 0.0, 1.5, arrow_r_c);
                    })),
                );
                self.end();

                // Card title.
                let st_t = self.text_style(TextRole::BodyStrong);
                self.text_leaf(card.title.clone(), st_t, fg_c, true, false, Style { width: Size::FILL, ..Default::default() });

                // Footer: ID/meta + assignee avatar.
                if card.meta.is_some() || card.assignee.is_some() {
                    self.begin(
                        Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() },
                        None,
                        Sense::NONE,
                        CursorIcon::Default,
                        false,
                        None,
                    );
                    if let Some(ref m) = card.meta {
                        let st_m = self.text_style(TextRole::Mono);
                        self.text_leaf(m.clone(), st_m, meta_c, false, false, Style::default());
                    }
                    self.spacer();
                    if let Some(ref ini) = card.assignee {
                        self.avatar(ini, ci, 22.0, None);
                    }
                    self.end();
                }

                self.end(); // card
            }

            self.end(); // column well
        }

        self.end(); // board
        moved
    }
}
