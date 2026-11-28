//! Batch action bars, speed-dial FABs, numeric PIN/amount pads, CSS/CAD
//! dimension inputs, stacked category meters, announcement bars, quantity
//! pickers, OKLab swatch pickers, week date strips and world-clock bars.

use super::field::{Chrome, EditOpts};
use super::toast::Tone;
use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::color::Color;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{ControlSize, TextRole, Token};
use crate::ui::{Response, Sense, Ui};

// ====================================================== FloatingActionBar

/// Output from [`Ui::floating_action_bar`].
#[derive(Clone, Copy, Debug, Default)]
pub struct ActionBarOut {
    pub clicked_action: Option<usize>,
    pub cleared: bool,
}

// ============================================================= SpeedDial

/// Quick-action entry in [`Ui::speed_dial`].
#[derive(Clone, Debug)]
pub struct SpeedDialItem {
    pub icon: String,
    pub label: String,
    pub shortcut: Option<String>,
}

impl SpeedDialItem {
    pub fn new(icon: &str, label: impl Into<String>) -> Self {
        SpeedDialItem { icon: icon.to_owned(), label: label.into(), shortcut: None }
    }
    pub fn shortcut(mut self, s: impl Into<String>) -> Self {
        self.shortcut = Some(s.into());
        self
    }
}

// ========================================================= DimensionInput

/// Unit for [`Ui::dimension_input`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DimensionUnit {
    #[default]
    Px,
    Percent,
    Rem,
    Auto,
}

impl DimensionUnit {
    pub const ALL: [DimensionUnit; 4] = [DimensionUnit::Px, DimensionUnit::Percent, DimensionUnit::Rem, DimensionUnit::Auto];

    pub fn label(self) -> &'static str {
        match self {
            DimensionUnit::Px => "px",
            DimensionUnit::Percent => "%",
            DimensionUnit::Rem => "rem",
            DimensionUnit::Auto => "auto",
        }
    }
}

#[derive(Default)]
struct DimMem {
    draft: String,
    was_focused: bool,
}

// ========================================================== StackedMeter

/// Category slice in [`Ui::stacked_meter`].
#[derive(Clone, Debug)]
pub struct MeterSegment {
    pub label: String,
    pub value: f32,
    pub unit_hint: String,
}

impl MeterSegment {
    pub fn new(label: impl Into<String>, value: f32, unit_hint: impl Into<String>) -> Self {
        MeterSegment { label: label.into(), value: value.max(0.0), unit_hint: unit_hint.into() }
    }
}

// ============================================================= DateStrip

/// Day tile in [`Ui::date_strip`].
#[derive(Clone, Debug)]
pub struct DateStripDay {
    pub weekday: String,
    pub day_num: u8,
    pub events: u8,
}

impl DateStripDay {
    pub fn new(weekday: impl Into<String>, day_num: u8, events: u8) -> Self {
        DateStripDay { weekday: weekday.into(), day_num, events }
    }
}

#[derive(Default, Clone, Copy)]
struct StripSlide {
    x: f32,
    w: f32,
    ready: bool,
}

// =========================================================== WorldClocks

/// Timezone entry in [`Ui::world_clocks`].
#[derive(Clone, Debug)]
pub struct WorldClock {
    pub city: String,
    pub code: String,
    pub time_str: String,
    pub offset: String,
    pub is_daytime: bool,
}

impl WorldClock {
    pub fn new(city: impl Into<String>, code: impl Into<String>, time_str: impl Into<String>, offset: impl Into<String>, is_daytime: bool) -> Self {
        WorldClock {
            city: city.into(),
            code: code.into(),
            time_str: time_str.into(),
            offset: offset.into(),
            is_daytime,
        }
    }
}

impl Ui {
    /// Elevated floating batch-selection bar (`"3 selected"`, action buttons,
    /// danger button, and `×` clear).
    pub fn floating_action_bar(
        &mut self,
        key: impl std::hash::Hash,
        selected_count: usize,
        actions: &[(&str, &str, bool)],
    ) -> ActionBarOut {
        let id = self.make_id(("action_bar", key));
        let mut out = ActionBarOut::default();
        if selected_count == 0 {
            return out;
        }
        let mo = self.motion();
        let enter = self.anims.spring_from(id, 0, 0.0, 1.0, mo.bouncy).clamp(0.0, 1.08);

        let r_pill = (self.look().card.radius).max(12.0);
        let bg_c = self.color(Token::SurfaceRaised);
        let bd_c = self.color(Token::Border);
        let div_c = self.color(Token::BorderSubtle);
        let ink = self.color(Token::ShadowInk);

        // A full-width slot measures the room the bar has; when last frame's
        // labelled bar didn't fit, actions collapse to icons with tooltips.
        let slot = id.with("slot");
        let room = self.rect_of(slot).map(|r| r.width());
        let full_w = self.rect_of(id).map(|r| r.width()).unwrap_or(0.0);
        let mut compact = *self.mem::<bool>(id);
        if !compact && room.is_some_and(|w| full_w > w + 0.5) {
            compact = true;
            *self.mem::<f32>(slot) = full_w;
        } else if compact && room.is_some_and(|w| *self.mem::<f32>(slot) <= w) {
            compact = false;
        }
        *self.mem::<bool>(id) = compact;
        self.begin(Style { dir: Dir::Row, width: Size::FILL, justify: crate::layout::Justify::Center, ..Default::default() }, Some(slot), Sense::NONE, CursorIcon::Default, false, None);

        self.begin(
            Style {
                dir: Dir::Row,
                pad: Sides::xy(10.0, 8.0),
                gap: 8.0,
                align: Align::Center,
                ..Default::default()
            },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                let dy = (1.0 - enter.min(1.0)) * 10.0;
                let sr = r.translate(vec2(0.0, dy));
                cx.painter.shadow(
                    sr,
                    Corners::all(r_pill),
                    &crate::theme::Shadow { x: 0.0, y: 8.0, blur: 24.0, spread: 0.0, color: ink.with_alpha(0.22) },
                );
                cx.painter.rrect(sr, Corners::all(r_pill), bg_c, 1.0, bd_c);
            })),
        );

        self.badge(format!("{selected_count} selected"), Tone::Accent);
        self.leaf(Style::default(), vec2(1.0, 18.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.rect(r, div_c))));

        for (ai, &(icon, label, danger)) in actions.iter().enumerate() {
            let clicked = self.scope(("ab_act", ai), |ui| {
                let b = if compact { ui.icon_button(icon).small().tooltip(label) } else { ui.button(label).small().icon(icon) };
                if danger {
                    b.danger().clicked()
                } else {
                    b.ghost().clicked()
                }
            });
            if clicked {
                out.clicked_action = Some(ai);
            }
        }

        self.leaf(Style::default(), vec2(1.0, 18.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.rect(r, div_c))));
        if self.scope("ab_clr", |ui| ui.icon_button("x").small().tooltip("Clear selection").clicked()) {
            out.cleared = true;
        }
        self.end();
        self.end(); // slot
        out
    }

    /// Expandable floating action button (`+` / `×`) fanning out quick-action
    /// pill items upward with staggered spring entrances.
    pub fn speed_dial(&mut self, key: impl std::hash::Hash, open: &mut bool, items: &[SpeedDialItem]) -> Option<usize> {
        let id = self.make_id(("speed_dial", key));
        let mo = self.motion();
        let reduced = self.reduced_motion();
        let mut chosen = None;

        if *open && self.key_pressed(Key::Escape) {
            *open = false;
        }

        self.begin(
            Style { dir: Dir::Column, gap: 8.0, align: Align::End, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );

        if *open {
            for (i, item) in items.iter().enumerate() {
                let iid = id.with(("sd_it", i));
                let ix = self.interact(iid, Sense::CLICK, false);
                let enter = if reduced { 1.0 } else { self.anims.spring_from(iid, 0, 0.0, 1.0, mo.bouncy).clamp(0.0, 1.08) };
                let hov = self.fade(iid, 1, ix.hovered, mo.hover_in, mo.hover_out, Ease::Out);
                let bg_c = self.color(Token::SurfaceRaised).mix(self.color(Token::Hover), hov);
                let bd_c = self.color(Token::Border).mix(self.color(Token::Accent), hov * 0.5);
                let ink = self.color(Token::ShadowInk);

                self.begin(
                    Style { dir: Dir::Row, pad: Sides::xy(12.0, 7.0), gap: 8.0, align: Align::Center, ..Default::default() },
                    Some(iid),
                    Sense::CLICK,
                    CursorIcon::Pointer,
                    false,
                    Some(Box::new(move |cx, r| {
                        let dy = (1.0 - enter.min(1.0)) * 8.0;
                        let sr = r.translate(vec2(0.0, dy));
                        cx.painter.shadow(sr, Corners::all(99.0), &crate::theme::Shadow { x: 0.0, y: 4.0, blur: 12.0, spread: 0.0, color: ink.with_alpha(0.16) });
                        cx.painter.rrect(sr, Corners::all(99.0), bg_c, 1.0, bd_c);
                    })),
                );
                self.icon_leaf(&item.icon, 14.0, self.color(Token::AccentStrong));
                self.text_leaf(item.label.clone(), self.text_style(TextRole::Label), self.color(Token::Foreground), false, false, Style::default());
                if let Some(ref sc) = item.shortcut {
                    let mut st_k = self.text_style(TextRole::Mono);
                    st_k.size = 11.0;
                    self.text_leaf(sc.clone(), st_k, self.color(Token::TextMuted), false, false, Style::default());
                }
                self.end();

                if ix.clicked {
                    chosen = Some(i);
                    *open = false;
                }
            }
        }

        // Primary circular FAB trigger.
        let fab_id = id.with("fab");
        let f_ix = self.interact(fab_id, Sense::CLICK, false);
        let is_open = *open;
        self.a11y(fab_id, || A11y::new(Role::Button).label("Actions").expanded(is_open));
        let press = self.spring(fab_id, 0, if f_ix.pressed { 1.0 } else { 0.0 }, mo.snappy);
        let rot_t = self.spring(fab_id, 1, if *open { 1.0 } else { 0.0 }, mo.bouncy);
        let hov = self.fade(fab_id, 2, f_ix.hovered, mo.hover_in, mo.hover_out, Ease::Out);
        let fab_bg = self.color(Token::Accent).mix(self.color(Token::AccentHover), hov);
        let fab_fg = self.color(Token::AccentForeground);
        let ink = self.color(Token::ShadowInk);

        self.leaf(
            Style::default(),
            vec2(46.0, 46.0),
            Some(fab_id),
            Sense::CLICK,
            CursorIcon::Pointer,
            Some(Box::new(move |cx, r| {
                let rad = 22.0 * (1.0 - 0.05 * press);
                let c = r.center();
                cx.painter.shadow(
                    Rect::from_center_size(c, vec2(rad * 2.0, rad * 2.0)),
                    Corners::all(rad),
                    &crate::theme::Shadow { x: 0.0, y: 6.0, blur: 16.0, spread: 0.0, color: ink.with_alpha(0.26) },
                );
                cx.painter.circle(c, rad, fab_bg, 0.0, Color::TRANSPARENT);
                // Draw rotating + / × arms analytically so the 45° rotation springs smoothly!
                let ang = rot_t * std::f32::consts::FRAC_PI_4;
                let d1 = vec2(ang.cos(), ang.sin()) * 7.0;
                let d2 = vec2(-ang.sin(), ang.cos()) * 7.0;
                cx.painter.polyline(&[c - d1, c + d1], 2.2, fab_fg);
                cx.painter.polyline(&[c - d2, c + d2], 2.2, fab_fg);
            })),
        );
        if f_ix.clicked {
            *open = !*open;
        }
        self.end();
        chosen
    }

    /// Tactile 3×4 numeric PIN / amount keypad with press-squash keys, Clear,
    /// Backspace, and masked or plain tabular readout.
    pub fn numeric_pad(&mut self, key: impl std::hash::Hash, value: &mut String, max_len: usize, masked: bool) -> bool {
        let id = self.make_id(("numeric_pad", key));
        let max_n = max_len.max(1);
        let mut submitted = false;

        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let disp_bg = self.color(Token::SurfaceMuted);

        self.begin(
            Style { dir: Dir::Column, width: Size::Fixed(248.0), pad: Sides::all(14.0), gap: 10.0, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        // Readout display window.
        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(42.0), pad: Sides::xy(12.0, 0.0), align: Align::Center, justify: Justify::Center, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(9.0), disp_bg, 1.0, bd_c))),
        );
        let mut st_d = self.text_style(TextRole::Heading3);
        st_d.tabular = true;
        if value.is_empty() {
            self.text_leaf("Enter PIN", self.text_style(TextRole::Body), self.color(Token::TextMuted), false, false, Style::default());
        } else {
            let shown = if masked { "• ".repeat(value.chars().count()).trim().to_owned() } else { value.clone() };
            self.text_leaf(shown, st_d, self.color(Token::Foreground), false, false, Style::default());
        }
        self.end();

        // 4 rows × 3 columns of tactile keys.
        const ROWS: [[&str; 3]; 4] = [
            ["1", "2", "3"],
            ["4", "5", "6"],
            ["7", "8", "9"],
            ["C", "0", "⌫"],
        ];
        let mo = self.motion();
        for (ri, row) in ROWS.iter().enumerate() {
            self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            for (ci, &k_lbl) in row.iter().enumerate() {
                let kid = id.with(("k", ri, ci));
                let k_ix = self.interact(kid, Sense::CLICK, false);
                self.a11y(kid, || A11y::new(Role::Button).label(match k_lbl {
                    "C" => "Clear",
                    "⌫" => "Delete",
                    d => d,
                }));
                let press = self.spring(kid, 0, if k_ix.pressed { 1.0 } else { 0.0 }, mo.snappy);
                let hov = self.fade(kid, 1, k_ix.hovered, mo.hover_in, mo.hover_out, Ease::Out);
                let k_bg = self.color(Token::SurfaceMuted).mix(self.color(Token::Hover), hov);
                let k_bd = self.color(Token::BorderSubtle);
                let k_fg = self.color(if matches!(k_lbl, "C" | "⌫") { Token::TextSecondary } else { Token::Foreground });
                let mut st_k = self.text_style(TextRole::BodyStrong);
                st_k.tabular = true;

                self.leaf(
                    Style { width: Size::FILL, ..Default::default() },
                    vec2(60.0, 40.0),
                    Some(kid),
                    Sense::CLICK,
                    CursorIcon::Pointer,
                    Some(Box::new(move |cx, r| {
                        let sq = 1.0 - 0.05 * press;
                        let sr = Rect::from_center_size(r.center(), r.size() * sq);
                        cx.painter.rrect(sr, Corners::all(9.0), k_bg, 1.0, k_bd);
                        if k_lbl == "⌫" {
                            cx.icon("delete", sr.center(), 15.0, k_fg);
                        } else {
                            let gk = cx.galley(k_lbl, &st_k);
                            cx.painter.text(cx.text, &gk, sr.center() - vec2(gk.width * 0.5, gk.height * 0.5), k_fg);
                        }
                    })),
                );
                if k_ix.clicked {
                    match k_lbl {
                        "C" => value.clear(),
                        "⌫" => {
                            value.pop();
                        }
                        d => {
                            if value.chars().count() < max_n {
                                value.push_str(d);
                                if value.chars().count() == max_n {
                                    submitted = true;
                                }
                            }
                        }
                    }
                }
            }
            self.end();
        }
        self.end();
        submitted
    }

    /// CSS / CAD dimension input pairing a tabular number with a segmented
    /// `px | % | rem | auto` unit selector.
    pub fn dimension_input(
        &mut self,
        key: impl std::hash::Hash,
        label: &str,
        value: &mut f32,
        unit: &mut DimensionUnit,
    ) -> bool {
        let id = self.make_id(("dim_input", key));
        let edit_id = id.with("ed");
        let focused = self.focused() == Some(edit_id);
        let hovered = self.pointer().is_some_and(|p| self.rect_of(id).is_some_and(|r| r.contains(p)));
        let cm = self.control_metrics(ControlSize::Md);
        let mut changed = false;

        let was_focused = self.mem::<DimMem>(id).was_focused;
        if !focused {
            self.mem::<DimMem>(id).draft = format!("{:.0}", *value);
        } else if !was_focused {
            let d = self.mem::<DimMem>(id).draft.clone();
            self.inline_edit_select_all(edit_id, &d);
        }

        self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if !label.is_empty() {
            self.field_label(label, false);
        }

        self.field_begin(
            id,
            Chrome { hovered, focused, error: false, disabled: false },
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                height: Size::Fixed(cm.height),
                pad: Sides { left: 10.0, right: 4.0, top: 0.0, bottom: 0.0 },
                gap: 6.0,
                align: Align::Center,
                ..Default::default()
            },
        );

        if *unit == DimensionUnit::Auto {
            self.text_leaf("Automatic sizing", self.text_style(TextRole::Body), self.color(Token::TextMuted), false, true, Style { width: Size::FILL, ..Default::default() });
        } else {
            let mut draft = self.mem::<DimMem>(id).draft.clone();
            let mut opts = EditOpts::new(self);
            opts.style = self.text_style(TextRole::Mono);
            opts.style.tabular = true;
            opts.accept = Some(|c| c.is_ascii_digit() || matches!(c, '.' | '-'));
            let ed = self.inline_edit(edit_id, &mut draft, opts);
            if ed.changed {
                if let Ok(v) = draft.parse::<f32>() {
                    *value = v;
                    changed = true;
                }
            }
            if ed.key(Key::ArrowUp) {
                *value += 1.0;
                draft = format!("{:.0}", *value);
                changed = true;
            } else if ed.key(Key::ArrowDown) {
                *value -= 1.0;
                draft = format!("{:.0}", *value);
                changed = true;
            }
            self.mem::<DimMem>(id).draft = draft;
        }

        // Compact unit pill strip on the right.
        let strip_bg = self.color(Token::SurfaceMuted);
        self.begin(
            Style { dir: Dir::Row, pad: Sides::all(2.0), gap: 2.0, align: Align::Center, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(6.0), strip_bg, 0.0, Color::TRANSPARENT))),
        );
        for u in DimensionUnit::ALL {
            let uid = id.with(("u", u.label()));
            let u_ix = self.interact(uid, Sense::CLICK, false);
            let is_act = *unit == u;
            let u_bg = if is_act { self.color(Token::Surface) } else { Color::TRANSPARENT };
            let u_bd = if is_act { self.color(Token::BorderSubtle) } else { Color::TRANSPARENT };
            let u_fg = if is_act { self.color(Token::AccentStrong) } else { self.color(Token::TextMuted) };
            self.begin(
                Style { pad: Sides::xy(6.0, 3.0), ..Default::default() },
                Some(uid),
                Sense::CLICK,
                CursorIcon::Pointer,
                false,
                Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(4.5), u_bg, 1.0, u_bd))),
            );
            let mut st_u = self.text_style(TextRole::Mono);
            st_u.size = 11.0;
            if is_act {
                st_u.weight = 600;
            }
            self.text_leaf(u.label(), st_u, u_fg, false, false, Style::default());
            self.end();
            if u_ix.clicked && *unit != u {
                *unit = u;
                changed = true;
            }
        }
        self.end();
        self.end();

        self.mem::<DimMem>(id).was_focused = self.focused() == Some(edit_id);
        self.end();
        changed
    }

    /// Multi-category stacked proportion bar (macOS storage / GitHub languages)
    /// with hairline gaps, rounded outer caps, and itemized legend.
    pub fn stacked_meter(&mut self, key: impl std::hash::Hash, title: &str, total_label: &str, segments: &[MeterSegment]) -> Response {
        let id = self.make_id(("stacked_meter", key));
        let mo = self.motion();
        let enter = self.anims.spring_from(id, 0, 0.0, 1.0, mo.smooth).clamp(0.0, 1.0);
        let sum: f32 = segments.iter().map(|s| s.value).sum::<f32>().max(1e-5);

        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let track_c = self.color(Token::SurfaceMuted);
        let palette = [
            self.color(Token::Series0),
            self.color(Token::Series1),
            self.color(Token::Series2),
            self.color(Token::Series3),
            self.color(Token::Series4),
        ];

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(16.0), gap: 12.0, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        self.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf(title.to_owned(), self.text_style(TextRole::BodyStrong), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
        let mut st_tot = self.text_style(TextRole::Caption);
        st_tot.tabular = true;
        self.text_leaf(total_label.to_owned(), st_tot, self.color(Token::TextMuted), false, false, Style::default());
        self.end();

        let fracs: Vec<(f32, Color)> = segments
            .iter()
            .enumerate()
            .map(|(i, s)| ((s.value / sum) * enter, palette[i % palette.len()]))
            .collect();

        self.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(260.0, 12.0),
            None,
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(6.0), track_c, 0.0, Color::TRANSPARENT);
                let gap = 2.0;
                let avail = (r.width() - gap * (fracs.len().saturating_sub(1)) as f32).max(10.0);
                let mut x = r.min.x;
                for (i, &(f, col)) in fracs.iter().enumerate() {
                    let w = (avail * f).max(2.0);
                    let corners = if fracs.len() == 1 {
                        Corners::all(6.0)
                    } else if i == 0 {
                        Corners([6.0, 2.0, 2.0, 6.0])
                    } else if i + 1 == fracs.len() {
                        Corners([2.0, 6.0, 6.0, 2.0])
                    } else {
                        Corners::all(2.0)
                    };
                    cx.painter.rrect(Rect::from_xywh(x, r.min.y, w, r.height()), corners, col, 0.0, Color::TRANSPARENT);
                    x += w + gap;
                }
            })),
        );

        // Wrapping legend items with colored dot, label and unit hint.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 14.0, wrap: true, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for (i, seg) in segments.iter().enumerate() {
            let col = palette[i % palette.len()];
            self.begin(Style { dir: Dir::Row, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.leaf(Style::default(), vec2(8.0, 8.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| cx.painter.circle(r.center(), 4.0, col, 0.0, Color::TRANSPARENT))));
            self.text_leaf(seg.label.clone(), self.text_style(TextRole::Caption), self.color(Token::Foreground), false, false, Style::default());
            let mut st_u = self.text_style(TextRole::Caption);
            st_u.tabular = true;
            self.text_leaf(seg.unit_hint.clone(), st_u, self.color(Token::TextMuted), false, false, Style::default());
            self.end();
        }
        self.end();
        self.end();

        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }

    /// Pill-framed top announcement / release ribbon with accent badge, message,
    /// CTA button, and dismiss `×`. Returns `(cta_clicked, dismissed)`.
    pub fn announcement_bar(&mut self, key: impl std::hash::Hash, tag: &str, message: &str, cta: &str) -> (bool, bool) {
        let id = self.make_id(("announce_bar", key));
        let bg_c = self.color(Token::AccentSubtle).multiply_alpha(0.5);
        let bd_c = self.color(Token::Accent).multiply_alpha(0.35);

        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(10.0, 7.0), gap: 10.0, align: Align::Center, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(99.0), bg_c, 1.0, bd_c))),
        );
        self.badge(tag, Tone::Accent);
        self.text_leaf(message.to_owned(), self.text_style(TextRole::Label), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
        let cta_clicked = self.scope("ann_cta", |ui| ui.button(cta).ghost().small().trailing_icon("arrow-right").clicked());
        let dismissed = self.scope("ann_x", |ui| ui.icon_button("x").small().tooltip("Dismiss").clicked());
        self.end();
        (cta_clicked, dismissed)
    }

    /// Quantity selector card with `−` / `+` buttons, rolling tabular readout,
    /// and quick preset chips (`1`, `5`, `10`, …).
    pub fn quantity_picker(
        &mut self,
        key: impl std::hash::Hash,
        label: &str,
        value: &mut i64,
        min: i64,
        max: i64,
        presets: &[i64],
    ) -> bool {
        let id = self.make_id(("qty_picker", key));
        *value = (*value).clamp(min, max);
        let mut changed = false;

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

        self.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf(label.to_owned(), self.text_style(TextRole::BodyStrong), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });

        let dec = self.scope("q_dec", |ui| ui.icon_button("minus").small().disabled(*value <= min).clicked());
        self.begin(Style { width: Size::Fixed(68.0), justify: Justify::Center, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.scope(id, |ui| {
            ui.rolling_number("q_roll", *value).role(TextRole::Heading3);
        });
        self.end();
        let inc = self.scope("q_inc", |ui| ui.icon_button("plus").small().disabled(*value >= max).clicked());
        self.end();

        if dec && *value > min {
            *value -= 1;
            changed = true;
        } else if inc && *value < max {
            *value += 1;
            changed = true;
        }

        if !presets.is_empty() {
            self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.text_leaf("Quick set:", self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, false, Style::default());
            for &p in presets {
                let clamped = p.clamp(min, max);
                let mut is_cur = *value == clamped;
                let chip_lbl = format!("{clamped}×");
                if self.scope(("qp", clamped), |ui| ui.chip(&chip_lbl, &mut is_cur)).changed && *value != clamped {
                    *value = clamped;
                    changed = true;
                }
            }
            self.end();
        }
        self.end();
        changed
    }

    /// Perceptual OKLab color swatch picker with spring-popping selection ring,
    /// luminance-contrasted checkmark, and active swatch readout.
    pub fn swatch_picker(&mut self, key: impl std::hash::Hash, label: &str, selected: &mut usize, swatches: &[(&str, Color)]) -> bool {
        let id = self.make_id(("swatch_picker", key));
        let n = swatches.len().max(1);
        *selected = (*selected).min(n - 1);
        let mo = self.motion();
        let mut changed = false;

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

        self.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.text_leaf(label.to_owned(), self.text_style(TextRole::BodyStrong), self.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
        if let Some(&(name, col)) = swatches.get(*selected) {
            let [r, g, b, _] = col.to_u8();
            let mut st_hex = self.text_style(TextRole::Mono);
            st_hex.size = 11.5;
            self.text_leaf(format!("{name} · #{r:02X}{g:02X}{b:02X}"), st_hex, self.color(Token::TextMuted), false, false, Style::default());
        }
        self.end();

        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 10.0, wrap: true, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for (si, &(sw_name, col)) in swatches.iter().enumerate() {
            let sid = id.with(("sw", si));
            let ix = self.interact(sid, Sense::CLICK, false);
            let sel_now = si == *selected;
            self.a11y(sid, || A11y::new(Role::RadioButton).label(sw_name).toggled(sel_now).position(si, n));
            if ix.focused {
                if self.key_pressed_for(sid, Key::ArrowRight) {
                    *selected = (*selected + 1) % n;
                    changed = true;
                } else if self.key_pressed_for(sid, Key::ArrowLeft) {
                    *selected = (*selected + n - 1) % n;
                    changed = true;
                }
            }
            let is_sel = si == *selected;
            let sel_t = self.spring(sid, 0, if is_sel { 1.0 } else { 0.0 }, mo.bouncy).clamp(0.0, 1.15);
            let press = self.spring(sid, 1, if ix.pressed { 1.0 } else { 0.0 }, mo.snappy);
            let check_c = if col.to_oklab().l > 0.72 { Color::BLACK } else { Color::WHITE };
            let ring_c = col;

            self.leaf(
                Style::default(),
                vec2(34.0, 34.0),
                Some(sid),
                Sense::CLICK,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    let c = r.center();
                    let rad = 13.5 * (1.0 - 0.06 * press);
                    cx.painter.circle(c, rad, col, 0.0, Color::TRANSPARENT);
                    if sel_t > 0.02 {
                        cx.painter.circle(c, rad + 3.5 * sel_t.min(1.0), Color::TRANSPARENT, 2.0, ring_c.with_alpha(sel_t.min(1.0)));
                        cx.icon("check", c, 14.0 * sel_t.min(1.0), check_c.with_alpha(sel_t.min(1.0)));
                    }
                })),
            );
            if ix.clicked && *selected != si {
                *selected = si;
                changed = true;
            }
        }
        self.end();
        self.end();
        changed
    }

    /// Horizontal 7-day calendar date strip with event count dots and a
    /// spring-gliding active day card.
    pub fn date_strip(&mut self, key: impl std::hash::Hash, days: &[DateStripDay], selected: &mut usize) -> Response {
        let id = self.make_id(("date_strip", key));
        let n = days.len().max(1);
        *selected = (*selected).min(n - 1);
        let mo = self.motion();
        let mut changed = false;

        let bar_prev = self.rect_of(id).unwrap_or(Rect::ZERO);
        let slide = *self.mem::<StripSlide>(id);
        let ind_x = if slide.ready { self.spring(id, 0, slide.x, mo.snappy) } else { slide.x };
        let ind_w = if slide.ready { self.spring(id, 1, slide.w, mo.snappy) } else { slide.w };

        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let act_bg = self.color(Token::Accent);
        let ready = slide.ready;

        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, pad: Sides::all(6.0), gap: 6.0, align: Align::Center, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c);
                if ready && ind_w > 4.0 {
                    let pr = Rect::from_xywh(r.min.x + ind_x, r.min.y + 6.0, ind_w, (r.height() - 12.0).max(20.0));
                    cx.painter.rrect(pr, Corners::all((r_card - 4.0).max(8.0)), act_bg, 0.0, Color::TRANSPARENT);
                }
            })),
        );

        let mut target_rel: Option<(f32, f32)> = None;
        for (di, day) in days.iter().enumerate() {
            let did = id.with(("ds_d", di));
            let ix = self.interact(did, Sense::CLICK, false);
            if ix.focused {
                if self.key_pressed_for(did, Key::ArrowRight) {
                    *selected = (*selected + 1) % n;
                    changed = true;
                } else if self.key_pressed_for(did, Key::ArrowLeft) {
                    *selected = (*selected + n - 1) % n;
                    changed = true;
                }
            }
            let is_act = di == *selected;
            if is_act && ix.rect.width() > 1.0 && bar_prev.width() > 1.0 {
                target_rel = Some((ix.rect.min.x - bar_prev.min.x, ix.rect.width()));
            }
            let hov = self.fade(did, 0, ix.hovered && !is_act, mo.hover_in, mo.hover_out, Ease::Out);
            let h_bg = self.color(Token::Hover).multiply_alpha(hov);

            self.begin(
                Style { dir: Dir::Column, width: Size::FILL, pad: Sides::xy(6.0, 8.0), gap: 3.0, align: Align::Center, ..Default::default() },
                Some(did),
                Sense::CLICK,
                CursorIcon::Pointer,
                false,
                Some(Box::new(move |cx, r| {
                    if hov > 0.005 {
                        cx.painter.rrect(r, Corners::all(8.0), h_bg, 0.0, Color::TRANSPARENT);
                    }
                })),
            );
            let w_fg = if is_act { self.color(Token::AccentForeground).multiply_alpha(0.85) } else { self.color(Token::TextMuted) };
            let n_fg = if is_act { self.color(Token::AccentForeground) } else { self.color(Token::Foreground) };
            self.text_leaf(day.weekday.clone(), self.text_style(TextRole::Caption), w_fg, false, false, Style::default());
            let mut st_n = self.text_style(TextRole::Heading4);
            st_n.tabular = true;
            self.text_leaf(format!("{}", day.day_num), st_n, n_fg, false, false, Style::default());

            // Event dots row (`0..=3` dots).
            let dots = day.events.min(3);
            let dot_c = if is_act { self.color(Token::AccentForeground) } else { self.color(Token::Accent) };
            self.leaf(
                Style::default(),
                vec2(24.0, 6.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    if dots > 0 {
                        let span = (dots - 1) as f32 * 6.0;
                        let x0 = r.center().x - span * 0.5;
                        for k in 0..dots {
                            cx.painter.circle(vec2(x0 + k as f32 * 6.0, r.center().y), 2.0, dot_c, 0.0, Color::TRANSPARENT);
                        }
                    }
                })),
            );
            self.end();

            if ix.clicked && *selected != di {
                *selected = di;
                changed = true;
            }
        }
        self.end();

        if let Some((rx, rw)) = target_rel {
            let mem = self.mem::<StripSlide>(id);
            mem.x = rx;
            mem.w = rw;
            mem.ready = true;
        }
        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, changed, ..Default::default() }
    }

    /// Multi-timezone distributed team clock strip with day/night icons and
    /// UTC offset badges.
    pub fn world_clocks(&mut self, key: impl std::hash::Hash, clocks: &[WorldClock]) -> Response {
        let id = self.make_id(("world_clocks", key));
        let r_card = self.look().card.radius;
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c))),
        );

        // As many clocks per row as fit (last frame's width), balanced so
        // four become 2x2 rather than 3 + 1.
        const CELL: f32 = 132.0;
        let n = clocks.len().max(1);
        let fit = self.rect_of(id).map_or(n, |r| ((r.width() / CELL).floor() as usize).clamp(1, n));
        let cols = n.div_ceil(n.div_ceil(fit));

        for (i, ck) in clocks.iter().enumerate() {
            if i % cols == 0 {
                let not_first_row = i > 0;
                self.begin(
                    Style { dir: Dir::Row, width: Size::FILL, align: Align::Stretch, ..Default::default() },
                    None,
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    if not_first_row {
                        Some(Box::new(move |cx, r| cx.painter.rect(Rect::from_xywh(r.min.x + 10.0, r.min.y, (r.width() - 20.0).max(0.0), 1.0), bd_c)))
                    } else {
                        None
                    },
                );
            }
            let not_first = i % cols > 0;
            self.begin(
                Style { dir: Dir::Column, width: Size::FILL, pad: Sides::all(14.0), gap: 6.0, ..Default::default() },
                None,
                Sense::NONE,
                CursorIcon::Default,
                false,
                if not_first {
                    Some(Box::new(move |cx, r| cx.painter.rect(Rect::from_xywh(r.min.x, r.min.y + 10.0, 1.0, (r.height() - 20.0).max(0.0)), bd_c)))
                } else {
                    None
                },
            );
            self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let ic = if ck.is_daytime { "sun" } else { "moon" };
            let ic_c = self.color(if ck.is_daytime { Token::WarningStrong } else { Token::AccentStrong });
            self.icon_leaf(ic, 13.5, ic_c);
            self.text_leaf(ck.code.clone(), self.text_style(TextRole::Caption), self.color(Token::TextMuted), false, true, Style { width: Size::FILL, ..Default::default() });
            self.badge(&ck.offset, Tone::Neutral);
            self.end();

            let mut st_t = self.text_style(TextRole::Heading3);
            st_t.tabular = true;
            self.text_leaf(ck.time_str.clone(), st_t, self.color(Token::Foreground), false, false, Style::default());
            self.text_leaf(ck.city.clone(), self.text_style(TextRole::Caption), self.color(Token::TextSecondary), false, true, Style { width: Size::FILL, ..Default::default() });
            self.end();
            if i % cols == cols - 1 || i + 1 == clocks.len() {
                self.end();
            }
        }
        self.end();

        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, ..Default::default() }
    }
}
