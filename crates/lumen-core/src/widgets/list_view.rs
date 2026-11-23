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
}

impl ListRow {
    pub fn new(title: impl Into<String>) -> Self {
        ListRow { title: title.into(), ..Default::default() }
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
}

#[derive(Default)]
struct ListMem {
    born: Option<f64>,
    count: usize,
}

impl Ui {
    /// A selectable list. `selected` is the chosen row (if any); click a
    /// row or use ↑ ↓ Home End to move it. Put it inside [`Ui::scroll`] for
    /// long lists.
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
        let mut focus_within = false;
        let row_ids: Vec<_> = (0..rows.len()).map(|i| id.with(("row", i))).collect();

        let node = self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 2.0, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);

        let mut stagger_pending = false;
        for (i, row) in rows.iter().enumerate() {
            let rid = row_ids[i];
            let ix = self.interact(rid, Sense::CLICK, false);
            focus_within |= ix.focused;
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
            if ix.focused {
                let mut go = None;
                if self.key_pressed_for(rid, Key::ArrowDown) && i + 1 < rows.len() {
                    go = Some(i + 1);
                }
                if self.key_pressed_for(rid, Key::ArrowUp) && i > 0 {
                    go = Some(i - 1);
                }
                if self.key_pressed_for(rid, Key::Home) && !rows.is_empty() {
                    go = Some(0);
                }
                if self.key_pressed_for(rid, Key::End) && !rows.is_empty() {
                    go = Some(rows.len() - 1);
                }
                if let Some(g) = go {
                    self.request_focus(row_ids[g]);
                    if *selected != Some(g) {
                        out.changed = true;
                    }
                    *selected = Some(g);
                }
                if self.key_pressed_for(rid, Key::Enter) {
                    *selected = Some(i);
                    out.activated = Some(i);
                }
            }

            // Entrance: fade and rise, 35 ms apart, capped so long lists
            // don't make you wait.
            let enter = if reduced {
                1.0
            } else {
                let delay = (i.min(12) as f64) * 0.035;
                let t = ((now - born - delay) / 0.28).clamp(0.0, 1.0) as f32;
                if t < 1.0 {
                    stagger_pending = true;
                }
                Ease::Out.apply(t)
            };

            let is_sel = *selected == Some(i);
            let hov = self.fade(rid, 0, ix.hovered && !is_sel, mo.hover_in, mo.hover_out, Ease::Linear);
            let press = self.fade(rid, 1, ix.pressed, 0.0, mo.press_out, Ease::Out);
            let dot = self.spring(rid, 2, if row.unread { 1.0 } else { 0.0 }, mo.bouncy).max(0.0);
            let (ring_a, ring_gap) = self.focus_anim(&ix);

            let hover_c = self.color(Token::Hover).multiply_alpha(hov);
            let press_c = self.color(Token::Pressed).multiply_alpha(press * 0.8);
            let ring_c = self.color(Token::FocusRing);
            let radius = nl.radius;
            let rise = (1.0 - enter) * 8.0;

            let row_node = self.begin(
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
            self.set_paint_after(row_node, Box::new(|cx, _| cx.painter.pop_layer()));

            let fg = self.color(Token::Foreground);
            let sec = self.color(Token::TextSecondary);
            let muted = self.color(Token::TextMuted);
            let accent = self.color(Token::Accent);

            // Unread dot gutter: keeps titles aligned whether or not a row is unread.
            self.leaf(
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
                self.avatar(ini, *tone, 32.0, None);
            } else if let Some(icon) = &row.icon {
                let ic = if is_sel { self.color(Token::AccentStrong) } else { self.color(Token::TextSecondary) };
                self.begin(Style { pad: Sides::xy(0.0, 1.0), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                self.icon_leaf(icon, 16.0, ic);
                self.end();
            }

            self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 2.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let title_role = if row.unread { TextRole::BodyStrong } else { TextRole::Body };
            let st = self.text_style(title_role);
            self.text_leaf(row.title.clone(), st, fg, false, true, Style { width: Size::FILL, ..Default::default() });
            if let Some((b, tone)) = &row.badge {
                self.badge(b.clone(), *tone);
            }
            if let Some(m) = &row.meta {
                let st = self.text_style(TextRole::Caption);
                let c = if row.unread { self.color(Token::AccentStrong) } else { muted };
                self.text_leaf(m.clone(), st, c, false, false, Style::default());
            }
            self.end();
            if let Some(s) = &row.subtitle {
                let role = if row.unread { TextRole::BodyStrong } else { TextRole::Body };
                let st = self.text_style(role);
                let c = if row.unread { fg } else { sec };
                self.text_leaf(s.clone(), st, c, false, true, Style { width: Size::FILL, ..Default::default() });
            }
            if let Some(p) = &row.preview {
                let st = self.text_style(TextRole::Caption);
                self.text_leaf(p.clone(), st, muted, false, true, Style { width: Size::FILL, ..Default::default() });
            }
            self.end();

            if row.starred || ix.hovered || is_sel {
                let sid = rid.with("star");
                let six = self.interact(sid, Sense::CLICK, false);
                let pop = self.spring(sid, 0, if row.starred { 1.0 } else { 0.0 }, mo.bouncy);
                let sh = self.fade(sid, 1, six.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
                let on_c = self.color(Token::Warning);
                let off_c = self.color(Token::TextMuted).mix(self.color(Token::Foreground), sh);
                let starred = row.starred;
                if six.clicked {
                    out.starred = Some(i);
                }
                self.leaf(
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
            self.end();
        }
        self.end();

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
        let sel_id = selected.and_then(|s| row_ids.get(s).copied());
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
