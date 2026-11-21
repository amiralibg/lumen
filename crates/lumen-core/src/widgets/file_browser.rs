//! File browser and path bar: a breadcrumb of folders, sortable columns,
//! folders first, a selection that glides, rows that rise in when the
//! folder changes, and keyboard navigation with type-to-select.

use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{TextRole, Token};
use crate::ui::{Sense, Ui};
use crate::Color;

/// An item in a [`Ui::file_browser`].
#[derive(Clone, Debug, Default)]
pub struct FileEntry {
    pub name: String,
    pub folder: bool,
    /// Bytes, for files.
    pub size: Option<u64>,
    /// Shown as is, such as "Today, 09:12".
    pub modified: String,
    /// Sorts the Modified column (larger is newer).
    pub modified_key: i64,
    /// Lucide icon; defaults by kind and extension.
    pub icon: Option<String>,
}

impl FileEntry {
    pub fn folder(name: impl Into<String>, modified: impl Into<String>, key: i64) -> Self {
        FileEntry { name: name.into(), folder: true, modified: modified.into(), modified_key: key, ..Default::default() }
    }
    pub fn file(name: impl Into<String>, size: u64, modified: impl Into<String>, key: i64) -> Self {
        FileEntry { name: name.into(), size: Some(size), modified: modified.into(), modified_key: key, ..Default::default() }
    }
}

/// What happened in a [`Ui::file_browser`] this frame. Indices refer to
/// the `entries` slice, whatever the sort order.
#[derive(Clone, Copy, Debug, Default)]
pub struct FileOut {
    pub changed: bool,
    /// Double click or Enter: open this entry (enter a folder).
    pub opened: Option<usize>,
    /// A path bar segment was clicked: go to `path[..=k]`.
    pub navigate: Option<usize>,
    /// Back, Backspace or Cmd+↑: go to the parent folder.
    pub up: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum SortBy {
    #[default]
    Name,
    Size,
    Modified,
}

#[derive(Default)]
struct BrowseMem {
    sort: SortBy,
    desc: bool,
    path_key: u64,
    born: f64,
    typed: String,
    typed_at: f64,
}

fn icon_for(e: &FileEntry) -> &str {
    if let Some(i) = &e.icon {
        return i;
    }
    if e.folder {
        return "folder";
    }
    match e.name.rsplit('.').next().unwrap_or("") {
        "rs" | "ts" | "js" | "py" | "wgsl" | "toml" | "json" | "sh" => "file-code",
        "png" | "jpg" | "jpeg" | "svg" | "webp" => "image",
        "mp3" | "m4a" | "wav" | "flac" => "audio-lines",
        "zip" | "tar" | "gz" => "archive",
        "md" | "txt" | "pdf" => "file-text",
        _ => "file",
    }
}

fn fmt_size(b: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut v = b as f64;
    let mut u = 0;
    while v >= 1000.0 && u + 1 < units.len() {
        v /= 1000.0;
        u += 1;
    }
    if u == 0 {
        format!("{b} B")
    } else if v < 10.0 {
        format!("{v:.1} {}", units[u])
    } else {
        format!("{v:.0} {}", units[u])
    }
}

impl Ui {
    /// A breadcrumb of folders. Returns the clicked segment (go to
    /// `segments[..=k]`). The last segment is the current folder. Long
    /// paths collapse their middle into "…".
    pub fn path_bar(&mut self, key: impl std::hash::Hash, segments: &[&str]) -> Option<usize> {
        let id = self.make_id(("path_bar", key));
        let mo = self.motion();
        let mut out = None;
        let avail = self.rect_of(id).map(|r| r.width()).unwrap_or(f32::INFINITY);
        let st = self.text_style(TextRole::Body);
        let st_cur = self.text_style(TextRole::BodyStrong);
        let widths: Vec<f32> = segments.iter().enumerate().map(|(k, s)| self.galley(s, if k + 1 == segments.len() { &st_cur } else { &st }).width + 16.0 + 18.0).collect();
        // Collapse from the second segment until it fits.
        let mut hidden = 0usize;
        while segments.len() > 2 + hidden && widths.iter().enumerate().filter(|(k, _)| *k == 0 || *k > hidden).map(|(_, w)| *w).sum::<f32>() + 40.0 > avail {
            hidden += 1;
        }
        let radius = self.look().control.radius[0];
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 0.0, align: Align::Center, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);
        for (k, seg) in segments.iter().enumerate() {
            if k > 0 && k <= hidden {
                if k == 1 {
                    self.icon_leaf("chevron-right", 14.0, self.color(Token::TextMuted));
                    let st = self.text_style(TextRole::Body);
                    let c = self.color(Token::TextMuted);
                    self.text_leaf("…".to_owned(), st, c, false, false, Style { pad: Sides::xy(6.0, 0.0), ..Default::default() });
                }
                continue;
            }
            if k > 0 {
                self.icon_leaf("chevron-right", 14.0, self.color(Token::TextMuted));
            }
            let last = k + 1 == segments.len();
            let sid = id.with(("seg", k));
            let ix = self.interact(sid, if last { Sense::NONE } else { Sense::CLICK }, false);
            if ix.clicked || ix.focused && self.key_pressed_for(sid, Key::Enter) {
                out = Some(k);
            }
            let h = self.fade(sid, 0, ix.hovered && !last, mo.hover_in, mo.hover_out, Ease::Linear);
            let (ring_a, ring_gap) = self.focus_anim(&ix);
            let hover = self.color(Token::Hover).multiply_alpha(h);
            let ring = self.color(Token::FocusRing);
            let c = if last { self.color(Token::Foreground) } else { self.color(Token::TextSecondary).mix(self.color(Token::Foreground), h) };
            let style = if last { st_cur } else { st };
            let g = self.galley(seg, &style);
            let icon = if k == 0 { Some("hard-drive") } else { None };
            let w = g.width + 16.0 + if icon.is_some() { 20.0 } else { 0.0 };
            self.leaf(
                Style::default(),
                vec2(w, 28.0),
                Some(sid),
                if last { Sense::NONE } else { Sense::CLICK },
                if last { CursorIcon::Default } else { CursorIcon::Pointer },
                Some(Box::new(move |cx, r| {
                    if h > 0.0 {
                        cx.painter.rrect(r, Corners::all(radius), hover, 0.0, Color::TRANSPARENT);
                    }
                    let mut x = r.min.x + 8.0;
                    if let Some(i) = icon {
                        cx.icon(i, vec2(x + 7.0, r.center().y), 14.0, c);
                        x += 20.0;
                    }
                    cx.painter.text(cx.text, &g, vec2(x, r.center().y - g.height * 0.5), c);
                    if ring_a > 0.0 {
                        cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr, radius, ring_a, ring_gap, ring)));
                    }
                })),
            );
        }
        self.end();
        out
    }

    /// A file browser for the folder `path`, listing `entries`. Click a
    /// header to sort; folders stay first. ↑ ↓ select, Enter opens,
    /// Backspace or Cmd+↑ goes up, and typing jumps to a matching name.
    pub fn file_browser(&mut self, key: impl std::hash::Hash, path: &[&str], entries: &[FileEntry], selected: &mut Option<usize>) -> FileOut {
        let id = self.make_id(("file_browser", key));
        let mo = self.motion();
        let reduced = self.reduced_motion();
        let now = self.time();
        let mut out = FileOut::default();

        // A new folder: rows rise in again, typed prefix resets.
        let pk = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            path.hash(&mut h);
            h.finish()
        };
        {
            let m = self.mem::<BrowseMem>(id);
            if m.path_key != pk {
                m.path_key = pk;
                m.born = now;
                m.typed.clear();
            }
        }
        let (sort, desc, born) = {
            let m = self.mem::<BrowseMem>(id);
            (m.sort, m.desc, m.born)
        };
        let mut order: Vec<usize> = (0..entries.len()).collect();
        order.sort_by(|&a, &b| {
            let (x, y) = (&entries[a], &entries[b]);
            y.folder.cmp(&x.folder).then_with(|| {
                let o = match sort {
                    SortBy::Name => x.name.to_lowercase().cmp(&y.name.to_lowercase()),
                    SortBy::Size => x.size.unwrap_or(0).cmp(&y.size.unwrap_or(0)),
                    SortBy::Modified => x.modified_key.cmp(&y.modified_key),
                };
                if desc { o.reverse() } else { o }
            })
        });

        let frame_r = self.look().card.radius;
        let bg = self.color(Token::Surface);
        let bd = self.color(Token::Border);
        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, ..Default::default() },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(frame_r), bg, 1.0, bd))),
        );

        // Toolbar: up, path.
        self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, align: Align::Center, pad: Sides::xy(8.0, 6.0), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let can_up = path.len() > 1;
        if self.scope(id, |ui| ui.icon_button("arrow-left").tooltip("Back").disabled(!can_up).show()).clicked {
            out.up = true;
        }
        if let Some(k) = self.scope(id, |ui| ui.path_bar("path", path)) {
            out.navigate = Some(k);
        }
        let count = format!("{} items", entries.len());
        let st = self.text_style(TextRole::Caption);
        let mc = self.color(Token::TextMuted);
        self.text_leaf(count, st, mc, false, false, Style { pad: Sides::xy(6.0, 0.0), ..Default::default() });
        self.end();

        // Header: sortable columns; the arrow flips on a spring.
        let rule = self.color(Token::BorderSubtle);
        let head_bg = self.color(Token::SurfaceMuted);
        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, gap: 0.0, align: Align::Center, pad: Sides::xy(12.0, 0.0), ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                cx.painter.rect(r, head_bg);
                cx.painter.rect(Rect::from_xywh(r.min.x, r.min.y, r.width(), 1.0), rule);
                cx.painter.rect(Rect::from_xywh(r.min.x, r.max.y - 1.0, r.width(), 1.0), rule);
            })),
        );
        for (col, label, width, right) in [(SortBy::Name, "Name", Size::FILL, false), (SortBy::Size, "Size", Size::Fixed(90.0), true), (SortBy::Modified, "Modified", Size::Fixed(150.0), true)] {
            let hid = id.with(("head", label));
            let ix = self.interact(hid, Sense::CLICK, false);
            if ix.clicked || ix.focused && self.key_pressed_for(hid, Key::Enter) {
                let m = self.mem::<BrowseMem>(id);
                if m.sort == col {
                    m.desc = !m.desc;
                } else {
                    m.sort = col;
                    m.desc = false;
                }
            }
            let on = sort == col;
            let flip = if reduced { if desc { 1.0 } else { 0.0 } } else { self.spring(hid, 0, if desc { 1.0 } else { 0.0 }, mo.snappy) };
            let show = self.fade(hid, 1, on, mo.hover_in, mo.hover_out, Ease::Linear);
            let h = self.fade(hid, 2, ix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
            let c = self.color(Token::TextMuted).mix(self.color(Token::Foreground), h.max(if on { 0.7 } else { 0.0 }));
            let st = self.text_style(TextRole::Label);
            let g = self.galley(label, &st);
            self.leaf(
                Style { width, ..Default::default() },
                vec2(g.width + 20.0, 30.0),
                Some(hid),
                Sense::CLICK,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    let x = if right { r.max.x - g.width - 16.0 } else { r.min.x };
                    cx.painter.text(cx.text, &g, vec2(x, r.center().y - g.height * 0.5), c);
                    if show > 0.0 {
                        // A chevron that turns from up (ascending) to down.
                        let at = vec2(x + g.width + 9.0, r.center().y);
                        let angle = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * flip;
                        super::data::draw_chevron(cx.painter, at, 4.0, angle, 1.5, c.multiply_alpha(show));
                    }
                })),
            );
        }
        self.end();

        // Rows.
        let row_ids: Vec<_> = (0..entries.len()).map(|i| id.with(("row", i))).collect();
        let mut focus_within = false;
        let mut pending = false;
        let nl = self.look().nav;
        let rows_node = self.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 1.0, pad: Sides::all(4.0), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for (pos, &i) in order.iter().enumerate() {
            let e = &entries[i];
            let rid = row_ids[i];
            let ix = self.interact(rid, Sense::CLICK, false);
            focus_within |= ix.focused;
            if ix.clicked {
                out.changed |= *selected != Some(i);
                *selected = Some(i);
                if ix.click_count >= 2 {
                    out.opened = Some(i);
                }
            }
            if ix.focused {
                let step = |d: isize| -> Option<usize> { order.get((pos as isize + d).clamp(0, order.len() as isize - 1) as usize).copied() };
                let mut go = None;
                for (k, m, _) in self.keys_for(rid) {
                    match k {
                        Key::ArrowDown if !m.command() => go = step(1),
                        Key::ArrowUp if m.command() => out.up = true,
                        Key::ArrowUp => go = step(-1),
                        Key::Home => go = order.first().copied(),
                        Key::End => go = order.last().copied(),
                        Key::Enter => out.opened = Some(i),
                        Key::Backspace => out.up = true,
                        Key::Char(c) if !m.command() && !m.ctrl => {
                            // Type-to-select: letters typed close together
                            // build a prefix.
                            let mm = self.mem::<BrowseMem>(id);
                            if now - mm.typed_at > 0.8 {
                                mm.typed.clear();
                            }
                            mm.typed.push(c);
                            mm.typed_at = now;
                            let pre = mm.typed.clone();
                            go = order.iter().copied().find(|&j| entries[j].name.to_lowercase().starts_with(&pre));
                        }
                        _ => {}
                    }
                }
                if let Some(g) = go {
                    self.request_focus(row_ids[g]);
                    out.changed |= *selected != Some(g);
                    *selected = Some(g);
                }
            }
            let enter = if reduced {
                1.0
            } else {
                let t = ((now - born - pos.min(12) as f64 * 0.03) / 0.26).clamp(0.0, 1.0) as f32;
                pending |= t < 1.0;
                Ease::Out.apply(t)
            };
            let hov = self.fade(rid, 0, ix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
            let (ring_a, ring_gap) = self.focus_anim(&ix);
            let hover_c = self.color(Token::Hover).multiply_alpha(hov);
            let ring = self.color(Token::FocusRing);
            let radius = nl.radius;
            let is_sel = *selected == Some(i);
            let row = self.begin(
                Style { dir: Dir::Row, width: Size::FILL, gap: 10.0, align: Align::Center, pad: Sides::xy(8.0, 6.0), ..Default::default() },
                Some(rid),
                Sense::CLICK,
                CursorIcon::Default,
                false,
                Some(Box::new(move |cx, r| {
                    if hov > 0.0 {
                        cx.painter.rrect(r, Corners::all(radius), hover_c, 0.0, Color::TRANSPARENT);
                    }
                    if ring_a > 0.0 {
                        cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr, radius, ring_a, ring_gap, ring)));
                    }
                    cx.painter.push_layer(enter, vec2(0.0, (1.0 - enter) * 6.0));
                })),
            );
            self.set_paint_after(row, Box::new(|cx, _| cx.painter.pop_layer()));
            let ic = if e.folder { self.color(Token::AccentStrong) } else if is_sel { self.color(Token::Foreground) } else { self.color(Token::TextSecondary) };
            self.icon_leaf(icon_for(e), 16.0, ic);
            let st = self.text_style(if e.folder { TextRole::BodyStrong } else { TextRole::Body });
            let fg = self.color(Token::Foreground);
            self.text_leaf(e.name.clone(), st, fg, false, true, Style { width: Size::FILL, ..Default::default() });
            let st = self.text_style(TextRole::Caption);
            let mc = self.color(Token::TextMuted);
            let size = if e.folder { "—".to_owned() } else { e.size.map(fmt_size).unwrap_or_default() };
            let n = self.text_leaf(size, st, mc, false, false, Style { width: Size::Fixed(90.0), ..Default::default() });
            self.set_text_align(n, lumen_text::TextAlign::End);
            let n = self.text_leaf(e.modified.clone(), st, mc, false, true, Style { width: Size::Fixed(150.0), pad: Sides { right: 8.0, ..Default::default() }, ..Default::default() });
            self.set_text_align(n, lumen_text::TextAlign::End);
            self.end();
        }
        if entries.is_empty() {
            let st = self.text_style(TextRole::Body);
            let mc = self.color(Token::TextMuted);
            self.text_leaf("This folder is empty".to_owned(), st, mc, false, false, Style { pad: Sides::xy(12.0, 16.0), ..Default::default() });
        }
        self.end();
        if pending {
            self.anims.keep_alive();
        }

        // The selection glides between rows (drawn under them).
        let accent_sel = self.color(Token::AccentSubtle);
        let neutral = self.color(Token::Surface).over(self.color(Token::Pressed));
        let sel_fill = if focus_within || nl.accent_selection > 0.5 { accent_sel } else { neutral.mix(accent_sel, 0.35) };
        let sel_id = selected.and_then(|s| row_ids.get(s).copied());
        let anim_key = id.with("pill");
        let smooth = mo.smooth;
        let radius = nl.radius;
        self.set_paint(
            rows_node,
            Box::new(move |cx, _| {
                let Some(sel) = sel_id.and_then(|s| cx.rects.get(&s).copied()) else { return };
                // First show: start at the row instead of gliding from 0.
                let fresh = reduced || cx.anims.spring_state(anim_key, 0).is_none();
                let y = cx.anims.spring_from(anim_key, 0, sel.min.y, sel.min.y, smooth);
                let y = if fresh { sel.min.y } else { y };
                cx.painter.rrect(Rect::from_xywh(sel.min.x, y, sel.width(), sel.height()), Corners::all(radius), sel_fill, 0.0, Color::TRANSPARENT);
            }),
        );
        self.end();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_read_like_finder() {
        assert_eq!(fmt_size(812), "812 B");
        assert_eq!(fmt_size(6_200_000), "6.2 MB");
        assert_eq!(fmt_size(48_000_000), "48 MB");
    }
}
