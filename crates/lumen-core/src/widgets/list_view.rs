//! Selectable list view: the master list of a mail, notes, music or files
//! app. One highlight glides between rows, rows arrive in sequence, unread
//! dots pop in and out, and the arrow keys walk the selection.

use super::toast::Tone;
use crate::anim::Ease;
use crate::color::Color;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{TextRole, Token};
use crate::ui::{Sense, Ui};

/// A row in [`Ui::list_view`]. Every field but the title is optional.
#[derive(Clone, Debug, Default)]
pub struct ListRow {
    pub title: String,
    /// Second line, such as a subject.
    pub subtitle: Option<String>,
    /// Muted third line, such as a message preview. Cut to one line.
    pub preview: Option<String>,
    /// Trailing text on the title line, such as a time.
    pub meta: Option<String>,
    /// Leading Lucide icon.
    pub icon: Option<String>,
    /// Leading avatar: initials and a tone index.
    pub avatar: Option<(String, usize)>,
    pub unread: bool,
    pub starred: bool,
    pub badge: Option<(String, Tone)>,
    /// A stable id from your data. With keys, rows inserted or removed
    /// above the viewport don't move what you're looking at.
    pub key: Option<u64>,
}

impl ListRow {
    pub fn new(title: impl Into<String>) -> Self {
        ListRow { title: title.into(), ..Default::default() }
    }
    /// A stable id for the row (see [`ListRow::key`](ListRow#structfield.key)).
    pub fn key(mut self, k: u64) -> Self {
        self.key = Some(k);
        self
    }
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.subtitle = Some(s.into());
        self
    }
    pub fn preview(mut self, s: impl Into<String>) -> Self {
        self.preview = Some(s.into());
        self
    }
    pub fn meta(mut self, s: impl Into<String>) -> Self {
        self.meta = Some(s.into());
        self
    }
    pub fn icon(mut self, name: impl Into<String>) -> Self {
        self.icon = Some(name.into());
        self
    }
    pub fn avatar(mut self, initials: impl Into<String>, tone: usize) -> Self {
        self.avatar = Some((initials.into(), tone));
        self
    }
    pub fn unread(mut self, on: bool) -> Self {
        self.unread = on;
        self
    }
    pub fn starred(mut self, on: bool) -> Self {
        self.starred = on;
        self
    }
    pub fn badge(mut self, text: impl Into<String>, tone: Tone) -> Self {
        self.badge = Some((text.into(), tone));
        self
    }
}

/// What happened in a [`Ui::list_view`] this frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct ListOut {
    /// The selection moved (click or arrow keys).
    pub changed: bool,
    /// A row was opened: double click, or Enter on the selection.
    pub activated: Option<usize>,
    /// A row was clicked, even the one already selected (a phone layout
    /// opens on a single tap).
    pub clicked: Option<usize>,
    /// A row's star was toggled.
    pub starred: Option<usize>,
    /// Rows built this frame (only those in view; see [`Ui::virtual_rows`]).
    pub built: usize,
}

#[derive(Default)]
struct ListMem {
    born: Option<f64>,
    count: usize,
    /// Row that had keyboard focus last frame.
    focus: Option<usize>,
    /// Rows per page for Page Up / Page Down.
    page: usize,
    /// Type-ahead buffer and when it was last typed into.
    typed: String,
    typed_at: f64,
}

impl Ui {
    /// A selectable list. `selected` is the chosen row (if any); click a
    /// row, or use ↑ ↓ Page Up/Down Home End, or type the start of a title,
    /// to move it. Inside a [`Ui::scroll`] only the rows in view are built
    /// (see [`Ui::virtual_rows`]), so it handles 100k rows.
    pub fn list_view(&mut self, key: impl std::hash::Hash, rows: &[ListRow], selected: &mut Option<usize>) -> ListOut {
        let id = self.make_id(("list_view", key));
        let mo = self.motion();
        let nl = self.look().nav;
        let reduced = self.reduced_motion();
        let now = self.time();

        // Rows arrive in sequence the first time the list shows, and again
        // when its contents are replaced (a new folder, a new search).
        let born = {
            let m = self.mem::<ListMem>(id);
            if m.born.is_none() || (m.count == 0) != rows.is_empty() || m.count.abs_diff(rows.len()) > 1 {
                m.born = Some(now);
            }
            m.count = rows.len();
            m.born.unwrap()
        };

        let mut out = ListOut::default();
        let n = rows.len();
        let row_id = |i: usize| id.with(("row", i));

        // Keyboard, on the row focused last frame, before the rows are
        // built: the row it moves to is scrolled into view and built now.
        let focused_i = self.mem::<ListMem>(id).focus.filter(|&i| i < n && self.focused() == Some(row_id(i)));
        let mut reveal = None;
        if let Some(i) = focused_i {
            let rid = row_id(i);
            let page = self.mem::<ListMem>(id).page.max(1);
            let mut go = None;
            if self.key_pressed_for(rid, Key::ArrowDown) && i + 1 < n {
                go = Some(i + 1);
            }
            if self.key_pressed_for(rid, Key::ArrowUp) && i > 0 {
                go = Some(i - 1);
            }
            if self.key_pressed_for(rid, Key::PageDown) {
                go = Some((i + page).min(n - 1));
            }
            if self.key_pressed_for(rid, Key::PageUp) {
                go = Some(i.saturating_sub(page));
            }
            if self.key_pressed_for(rid, Key::Home) {
                go = Some(0);
            }
            if self.key_pressed_for(rid, Key::End) {
                go = Some(n - 1);
            }
            // Type-ahead: jump to the next title starting with what was
            // typed (a pause of 0.8 s starts a new search).
            let typed: String = self.take_text_input(rid).concat().to_lowercase();
            if !typed.trim().is_empty() {
                let m = self.mem::<ListMem>(id);
                if now - m.typed_at > 0.8 {
                    m.typed.clear();
                }
                m.typed.push_str(&typed);
                m.typed_at = now;
                let q = m.typed.clone();
                let repeat = q.chars().all(|c| q.starts_with(c));
                let probe: String = if repeat { q.chars().take(1).collect() } else { q.clone() };
                let start = if repeat { i + 1 } else { i };
                go = (0..n).map(|k| (start + k) % n).find(|&j| rows[j].title.to_lowercase().starts_with(&probe)).or(go);
            }
            if let Some(g) = go {
                self.request_focus(row_id(g));
                if *selected != Some(g) {
                    out.changed = true;
                }
                *selected = Some(g);
                reveal = Some(g);
            }
            if self.key_pressed_for(rid, Key::Enter) {
                *selected = Some(i);
                out.activated = Some(i);
            }
        }

        // A guess at a row's height until rows are measured.
        let est = rows.first().map_or(36.0, |r| {
            let lines = 20.0 + if r.subtitle.is_some() { 22.0 } else { 0.0 } + if r.preview.is_some() { 18.0 } else { 0.0 };
            let lead: f32 = if r.avatar.is_some() { 32.0 } else { 18.0 };
            lead.max(lines) + 16.0
        });

        let node = self.begin(Style { dir: Dir::Column, width: Size::FILL, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);

        let mut focus_within = false;
        let mut focus_row = None;
        let mut stagger_pending = false;
        let mut built = 0usize;
        let pin = selected.or(focused_i);
        let key_of = |i: usize| rows[i].key.unwrap_or(i as u64);
        let vout = self.virtual_rows("rows", n).keys(&key_of).gap(2.0).estimate(est).pin(pin).reveal(reveal).show(|ui, i| {
            let row = &rows[i];
            let rid = row_id(i);
            let ix = ui.interact(rid, Sense::CLICK, false);
            focus_within |= ix.focused;
            if ix.focused {
                focus_row = Some(i);
            }
            if ix.clicked {
                if *selected != Some(i) {
                    out.changed = true;
                }
                out.clicked = Some(i);
                *selected = Some(i);
                if ix.click_count >= 2 {
                    out.activated = Some(i);
                }
            }

            // Entrance: fade and rise, 35 ms apart, capped so long lists
            // don't make you wait.
            let enter = if reduced {
                1.0
            } else {
                let delay = (built.min(12) as f64) * 0.035;
                let t = ((now - born - delay) / 0.28).clamp(0.0, 1.0) as f32;
                if t < 1.0 {
                    stagger_pending = true;
                }
                Ease::Out.apply(t)
            };
            built += 1;

            let is_sel = *selected == Some(i);
            let hov = ui.fade(rid, 0, ix.hovered && !is_sel, mo.hover_in, mo.hover_out, Ease::Linear);
            let press = ui.fade(rid, 1, ix.pressed, 0.0, mo.press_out, Ease::Out);
            let dot = ui.spring(rid, 2, if row.unread { 1.0 } else { 0.0 }, mo.bouncy).max(0.0);
            let (ring_a, ring_gap) = ui.focus_anim(&ix);

            let hover_c = ui.color(Token::Hover).multiply_alpha(hov);
            let press_c = ui.color(Token::Pressed).multiply_alpha(press * 0.8);
            let ring_c = ui.color(Token::FocusRing);
            let radius = nl.radius;
            let rise = (1.0 - enter) * 8.0;

            let row_node = ui.begin(
                Style { dir: Dir::Row, width: Size::FILL, pad: Sides::xy(10.0, 8.0), gap: 10.0, align: Align::Start, ..Default::default() },
                Some(rid),
                Sense::CLICK,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| {
                    let bg = hover_c.over(press_c);
                    if bg.a > 0.0 {
                        cx.painter.rrect(r, Corners::all(radius), bg, 0.0, Color::TRANSPARENT);
                    }
                    if ring_a > 0.0 {
                        cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr, radius, ring_a, ring_gap, ring_c)));
                    }
                    // Everything in the row (text, avatar, badge, star)
                    // fades and rises together during the entrance.
                    cx.painter.push_layer(enter, vec2(0.0, rise));
                })),
            );
            ui.set_paint_after(row_node, Box::new(|cx, _| cx.painter.pop_layer()));

            let fg = ui.color(Token::Foreground);
            let sec = ui.color(Token::TextSecondary);
            let muted = ui.color(Token::TextMuted);
            let accent = ui.color(Token::Accent);

            // Unread dot gutter: keeps titles aligned whether or not a row is unread.
            ui.leaf(
                Style::default(),
                vec2(6.0, 18.0),
                None,
                Sense::NONE,
                CursorIcon::Default,
                Some(Box::new(move |cx, r| {
                    if dot > 0.01 {
                        cx.painter.circle(vec2(r.center().x, r.min.y + 9.0), 3.5 * dot, accent, 0.0, Color::TRANSPARENT);
                    }
                })),
            );

            if let Some((ini, tone)) = &row.avatar {
                ui.avatar(ini, *tone, 32.0, None);
            } else if let Some(icon) = &row.icon {
                let ic = if is_sel { ui.color(Token::AccentStrong) } else { ui.color(Token::TextSecondary) };
                ui.begin(Style { pad: Sides::xy(0.0, 1.0), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                ui.icon_leaf(icon, 16.0, ic);
                ui.end();
            }

            ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 2.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let title_role = if row.unread { TextRole::BodyStrong } else { TextRole::Body };
            let st = ui.text_style(title_role);
            ui.text_leaf(row.title.clone(), st, fg, false, true, Style { width: Size::FILL, ..Default::default() });
            if let Some((b, tone)) = &row.badge {
                ui.badge(b.clone(), *tone);
            }
            if let Some(m) = &row.meta {
                let st = ui.text_style(TextRole::Caption);
                let c = if row.unread { ui.color(Token::AccentStrong) } else { muted };
                ui.text_leaf(m.clone(), st, c, false, false, Style::default());
            }
            ui.end();
            if let Some(s) = &row.subtitle {
                let role = if row.unread { TextRole::BodyStrong } else { TextRole::Body };
                let st = ui.text_style(role);
                let c = if row.unread { fg } else { sec };
                ui.text_leaf(s.clone(), st, c, false, true, Style { width: Size::FILL, ..Default::default() });
            }
            if let Some(p) = &row.preview {
                let st = ui.text_style(TextRole::Caption);
                ui.text_leaf(p.clone(), st, muted, false, true, Style { width: Size::FILL, ..Default::default() });
            }
            ui.end();

            if row.starred || ix.hovered || is_sel {
                let sid = rid.with("star");
                let six = ui.interact(sid, Sense::CLICK, false);
                let pop = ui.spring(sid, 0, if row.starred { 1.0 } else { 0.0 }, mo.bouncy);
                let sh = ui.fade(sid, 1, six.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
                let on_c = ui.color(Token::Warning);
                let off_c = ui.color(Token::TextMuted).mix(ui.color(Token::Foreground), sh);
                let starred = row.starred;
                if six.clicked {
                    out.starred = Some(i);
                }
                ui.leaf(
                    Style::default(),
                    vec2(18.0, 18.0),
                    Some(sid),
                    Sense::CLICK,
                    CursorIcon::Pointer,
                    Some(Box::new(move |cx, r| {
                        let c = r.center();
                        if starred {
                            // A filled star that pops past full size and settles.
                            star(cx, c, 7.0 * pop.max(0.0), on_c);
                        } else {
                            cx.icon("star", c, 14.0, off_c);
                        }
                    })),
                );
            }
            ui.end();
        });
        self.end();
        out.built = vout.built;
        {
            let m = self.mem::<ListMem>(id);
            m.focus = focus_row;
            if n > 0 && vout.total_height > 0.0 {
                m.page = ((vout.viewport / (vout.total_height / n as f32)).floor() as usize).saturating_sub(1).max(1);
            }
        }

        if stagger_pending {
            self.anims.keep_alive();
        }

        // The selection: one rect that glides between rows.
        // PRESSED is an overlay: composite it onto the surface first, or
        // mixing it with an opaque color gives a heavy translucent gray.
        let neutral = self.color(Token::Surface).over(self.color(Token::Pressed));
        let accent_sel = self.color(Token::AccentSubtle);
        let sel_fill = if focus_within || nl.accent_selection > 0.5 { accent_sel } else { neutral.mix(accent_sel, 0.35) };
        let bar = self.color(Token::Accent).multiply_alpha(nl.indicator.max(0.0));
        let sel_id = selected.filter(|&s| s < n).map(row_id);
        let anim_key = id.with("pill");
        let smooth = mo.smooth;
        let radius = nl.radius;
        self.set_paint(
            node,
            Box::new(move |cx, _| {
                let Some(sel) = sel_id.and_then(|s| cx.rects.get(&s).copied()) else { return };
                let (y, h) = if reduced {
                    (sel.min.y, sel.height())
                } else {
                    let first = cx.anims.spring_state(anim_key, 0).is_none();
                    let y = cx.anims.spring_from(anim_key, 0, sel.min.y, sel.min.y, smooth);
                    let h = cx.anims.spring_from(anim_key, 1, sel.height(), sel.height(), smooth);
                    if first {
                        (sel.min.y, sel.height())
                    } else {
                        (y, h)
                    }
                };
                let pr = Rect::from_xywh(sel.min.x, y, sel.width(), h);
                cx.painter.rrect(pr, Corners::all(radius), sel_fill, 0.0, Color::TRANSPARENT);
                if bar.a > 0.0 {
                    let bh = (pr.height() * 0.45).round();
                    let br = Rect::from_xywh(pr.min.x, pr.center().y - bh / 2.0, 3.0, bh);
                    cx.painter.rrect_unsnapped(br, Corners::all(1.5), [bar; 2], 0.0, Color::TRANSPARENT);
                }
            }),
        );
        out
    }
}

/// A filled five-point star of outer radius `r` centered at `c`.
fn star(cx: &mut crate::ui::PaintCx<'_>, c: crate::math::Vec2, r: f32, color: Color) {
    if r <= 0.1 {
        return;
    }
    let mut verts = vec![(c, color)];
    for k in 0..10 {
        let a = -std::f32::consts::FRAC_PI_2 + k as f32 * std::f32::consts::PI / 5.0;
        let rr = if k % 2 == 0 { r } else { r * 0.45 };
        verts.push((c + vec2(a.cos() * rr, a.sin() * rr), color));
    }
    let mut idx = Vec::with_capacity(30);
    for k in 0..10u32 {
        idx.extend_from_slice(&[0, 1 + k, 1 + (k + 1) % 10]);
    }
    cx.painter.mesh(&verts, &idx);
}
