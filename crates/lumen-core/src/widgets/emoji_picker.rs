//! Emoji picker: search, a category bar with a sliding pill, a grid whose
//! cells pop on hover, recents, and a preview of the emoji under the pointer.

use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{TextRole, Token};
use crate::ui::{Sense, Ui};
use crate::Color;

const COLS: usize = 8;
const CELL: f32 = 36.0;
const VISIBLE_ROWS: usize = 5;

/// Category: Lucide icon and title.
const CATS: [(&str, &str); 8] = [
    ("smile", "Smileys"),
    ("hand", "People"),
    ("leaf", "Nature"),
    ("utensils", "Food"),
    ("trophy", "Activity"),
    ("plane", "Travel"),
    ("lightbulb", "Objects"),
    ("heart", "Symbols"),
];

/// (emoji, name, category index)
const EMOJI: &[(&str, &str, u8)] = &[
    ("😀", "grinning face", 0), ("😃", "big eyes", 0), ("😄", "smiling eyes", 0), ("😁", "beaming", 0), ("😆", "laughing", 0),
    ("😅", "sweat smile", 0), ("😂", "tears of joy", 0), ("🙂", "slight smile", 0), ("😉", "wink", 0), ("😊", "blush", 0),
    ("😍", "heart eyes", 0), ("🤩", "star struck", 0), ("😘", "kiss", 0), ("😋", "yum", 0), ("😎", "sunglasses", 0),
    ("🤔", "thinking", 0), ("😴", "sleeping", 0), ("😮", "open mouth", 0), ("🥳", "party face", 0), ("😇", "halo", 0),
    ("👋", "wave", 1), ("👍", "thumbs up", 1), ("👎", "thumbs down", 1), ("👏", "clap", 1), ("🙌", "raised hands", 1),
    ("🙏", "folded hands", 1), ("💪", "flexed biceps", 1), ("✌️", "victory", 1), ("🤝", "handshake", 1), ("👀", "eyes", 1),
    ("🧑‍💻", "technologist", 1), ("👩‍🎨", "artist", 1), ("🧑‍🚀", "astronaut", 1), ("🕺", "dancing", 1), ("🤷", "shrug", 1), ("🫶", "heart hands", 1),
    ("🌱", "seedling", 2), ("🌿", "herb", 2), ("🌸", "blossom", 2), ("🌻", "sunflower", 2), ("🍀", "four leaf clover", 2),
    ("🌲", "evergreen", 2), ("🌊", "wave water", 2), ("🔥", "fire", 2), ("⭐", "star", 2), ("🌙", "moon", 2),
    ("☀️", "sun", 2), ("🌈", "rainbow", 2), ("❄️", "snowflake", 2), ("🐶", "dog", 2), ("🐱", "cat", 2), ("🦊", "fox", 2),
    ("🍎", "apple", 3), ("🍋", "lemon", 3), ("🍓", "strawberry", 3), ("🥑", "avocado", 3), ("🍕", "pizza", 3),
    ("🍔", "burger", 3), ("🌮", "taco", 3), ("🍜", "ramen", 3), ("🍣", "sushi", 3), ("🍩", "doughnut", 3),
    ("🍪", "cookie", 3), ("☕", "coffee", 3), ("🍵", "tea", 3), ("🧋", "bubble tea", 3), ("🥐", "croissant", 3), ("🧁", "cupcake", 3),
    ("⚽", "soccer", 4), ("🏀", "basketball", 4), ("🎾", "tennis", 4), ("🏆", "trophy", 4), ("🥇", "gold medal", 4),
    ("🎮", "video game", 4), ("🎲", "dice", 4), ("🎯", "bullseye", 4), ("🎨", "palette", 4), ("🎸", "guitar", 4),
    ("🎧", "headphones", 4), ("🎬", "clapper", 4), ("🧩", "puzzle", 4), ("🏄", "surfing", 4), ("🚴", "cycling", 4), ("🧗", "climbing", 4),
    ("🚀", "rocket", 5), ("✈️", "airplane", 5), ("🚗", "car", 5), ("🚲", "bicycle", 5), ("🚆", "train", 5),
    ("⛵", "sailboat", 5), ("🗺️", "map", 5), ("🏔️", "mountain", 5), ("🏝️", "island", 5), ("🏙️", "city", 5),
    ("🌍", "globe", 5), ("🗽", "statue", 5), ("🏕️", "camping", 5), ("🛸", "ufo", 5), ("🌋", "volcano", 5), ("🎡", "ferris wheel", 5),
    ("💡", "light bulb", 6), ("💻", "laptop", 6), ("⌨️", "keyboard", 6), ("🖥️", "desktop", 6), ("📱", "phone", 6),
    ("📷", "camera", 6), ("🔒", "lock", 6), ("🔑", "key", 6), ("📦", "package", 6), ("🧪", "test tube", 6),
    ("🔧", "wrench", 6), ("🧭", "compass", 6), ("📌", "pin", 6), ("📎", "paperclip", 6), ("📚", "books", 6), ("✏️", "pencil", 6),
    ("❤️", "red heart", 7), ("🧡", "orange heart", 7), ("💛", "yellow heart", 7), ("💚", "green heart", 7), ("💙", "blue heart", 7),
    ("💜", "purple heart", 7), ("✨", "sparkles", 7), ("✅", "check", 7), ("❌", "cross", 7), ("⚠️", "warning", 7),
    ("💯", "hundred", 7), ("🎉", "party popper", 7), ("💬", "speech", 7), ("🔔", "bell", 7), ("➕", "plus", 7), ("🆕", "new", 7),
];

#[derive(Default)]
struct EmojiMem {
    query: String,
    cat: usize,
    /// Highlighted cell (keyboard), in the shown list.
    hot: Option<usize>,
    recent: Vec<&'static str>,
    scroll_row: usize,
}

impl Ui {
    /// An emoji picker. Returns the emoji picked this frame. Search by
    /// name, switch categories, or use the arrow keys and Enter on the grid.
    /// Recently picked emoji appear first.
    pub fn emoji_picker(&mut self, key: impl std::hash::Hash) -> Option<&'static str> {
        // Emoji come from the system's color emoji font.
        self.text.set_system_fallback(true);
        let id = self.make_id(("emoji_picker", key));
        let grid_id = id.with("grid");
        let mo = self.motion();
        let reduced = self.reduced_motion();
        let mut picked = None;

        self.begin(Style { dir: Dir::Column, width: Size::Fixed(COLS as f32 * CELL), gap: 10.0, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);

        // Search.
        let mut q = std::mem::take(&mut self.mem::<EmojiMem>(id).query);
        let search = self.scope(id, |ui| ui.text_input(&mut q).icon("search").placeholder("Search emoji").id("search").fill_width().show());
        let searching = !q.trim().is_empty();
        if search.changed {
            let m = self.mem::<EmojiMem>(id);
            m.hot = None;
            m.scroll_row = 0;
        }
        if search.focused && self.key_pressed_for(search.id, Key::ArrowDown) {
            self.request_focus(grid_id);
            self.mem::<EmojiMem>(id).hot = Some(0);
        }
        self.mem::<EmojiMem>(id).query = q.clone();

        // Category bar with a pill that slides to the chosen one.
        let cat = self.mem::<EmojiMem>(id).cat;
        let pill_x = if reduced { cat as f32 } else { self.spring(id, 0, cat as f32, mo.smooth) };
        let pill_a = self.fade(id, 1, !searching, mo.hover_in, mo.hover_out, Ease::Linear);
        let tab_w = (COLS as f32 * CELL) / CATS.len() as f32;
        let pill_c = self.color(Token::AccentSubtle);
        let r_pill = self.look().control.radius[0];
        self.begin(
            Style { dir: Dir::Row, width: Size::FILL, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                if pill_a > 0.0 {
                    let pr = Rect::from_xywh(r.min.x + pill_x * tab_w + 2.0, r.min.y, tab_w - 4.0, r.height());
                    cx.painter.rrect_unsnapped(pr, Corners::all(r_pill), [pill_c.multiply_alpha(pill_a); 2], 0.0, Color::TRANSPARENT);
                }
            })),
        );
        for (k, (icon, title)) in CATS.iter().enumerate() {
            let tid = id.with(("cat", k));
            let ix = self.interact(tid, Sense::CLICK, false);
            if ix.clicked || ix.focused && (self.key_pressed_for(tid, Key::Enter) || self.key_pressed_for(tid, Key::Space)) {
                let m = self.mem::<EmojiMem>(id);
                m.cat = k;
                m.hot = None;
                m.scroll_row = 0;
                m.query.clear();
            }
            if ix.focused {
                let go = if self.key_pressed_for(tid, Key::ArrowRight) { Some((k + 1) % CATS.len()) } else if self.key_pressed_for(tid, Key::ArrowLeft) { Some((k + CATS.len() - 1) % CATS.len()) } else { None };
                if let Some(g) = go {
                    self.request_focus(id.with(("cat", g)));
                }
            }
            let on = k == cat && !searching;
            let h = self.fade(tid, 0, ix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
            let (ring_a, ring_gap) = self.focus_anim(&ix);
            let ic = self.color(Token::TextMuted).mix(self.color(Token::Foreground), h).mix(self.color(Token::AccentStrong), if on { 1.0 } else { 0.0 });
            let ring = self.color(Token::FocusRing);
            let icon = *icon;
            self.tooltip_for(tid, crate::ui::TooltipSpec { text: (*title).into(), shortcut: None, placement: crate::ui::Placement::Bottom, gap: 6.0 });
            self.leaf(
                Style { width: Size::Fixed(tab_w), ..Default::default() },
                vec2(tab_w, 30.0),
                Some(tid),
                Sense::CLICK,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    cx.icon(icon, r.center(), 16.0, ic);
                    if ring_a > 0.0 {
                        cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr.expand(-2.0), r_pill, ring_a, ring_gap, ring)));
                    }
                })),
            );
        }
        self.end();

        // What the grid shows: matches, or recents then the category.
        let (cat, recent) = {
            let m = self.mem::<EmojiMem>(id);
            (m.cat, m.recent.clone())
        };
        let ql = q.trim().to_lowercase();
        let shown: Vec<(&'static str, &'static str)> = if searching {
            EMOJI.iter().filter(|e| e.1.contains(&ql)).map(|e| (e.0, e.1)).collect()
        } else {
            let mut v: Vec<(&str, &str)> = if cat == 0 { recent.iter().filter_map(|r| EMOJI.iter().find(|e| e.0 == *r).map(|e| (e.0, e.1))).collect() } else { vec![] };
            let rest: Vec<(&str, &str)> = EMOJI.iter().filter(|e| e.2 as usize == cat && !v.iter().any(|x| x.0 == e.0)).map(|e| (e.0, e.1)).collect();
            v.extend(rest);
            v
        };
        let n_recent = if !searching && cat == 0 { recent.len().min(shown.len()) } else { 0 };
        let section = if searching { format!("{} results", shown.len()) } else if n_recent > 0 { format!("Recent · {}", CATS[cat].1) } else { CATS[cat].1.to_owned() };
        let st = self.text_style(TextRole::Label);
        let muted = self.color(Token::TextMuted);
        self.text_leaf(section, st, muted, false, true, Style { width: Size::FILL, ..Default::default() });

        // Grid: one focusable leaf, hit-tested by cell.
        let rows_total = shown.len().div_ceil(COLS).max(1);
        let gix = self.interact(grid_id, Sense { click: true, drag: false, focusable: true, scroll: rows_total > VISIBLE_ROWS }, false);
        let grid_rect = gix.rect;
        let cell_at = |p: crate::Vec2, scroll_row: usize| -> Option<usize> {
            if !grid_rect.contains(p) {
                return None;
            }
            let c = ((p.x - grid_rect.min.x) / CELL).floor() as usize;
            let r = ((p.y - grid_rect.min.y) / CELL).floor() as usize + scroll_row;
            (c < COLS).then_some(r * COLS + c).filter(|i| *i < shown.len())
        };
        let wheel = self.take_scroll(grid_id).y;
        {
            let m = self.mem::<EmojiMem>(id);
            if wheel.abs() > 0.5 {
                let max = rows_total.saturating_sub(VISIBLE_ROWS);
                m.scroll_row = if wheel > 0.0 { (m.scroll_row + 1).min(max) } else { m.scroll_row.saturating_sub(1) };
            }
        }
        let scroll_row = self.mem::<EmojiMem>(id).scroll_row;
        let hover_cell = gix.pointer.and_then(|p| cell_at(p, scroll_row)).filter(|_| gix.hovered);
        if gix.clicked {
            if let Some(i) = gix.pointer.and_then(|p| cell_at(p, scroll_row)) {
                picked = Some(shown[i].0);
            }
        }
        if gix.focused && !shown.is_empty() {
            let m = self.mem::<EmojiMem>(id);
            let mut hot = m.hot.unwrap_or(0).min(shown.len() - 1);
            for (k, mods, _) in self.keys_for(grid_id) {
                if mods.command() {
                    continue;
                }
                hot = match k {
                    Key::ArrowRight => (hot + 1).min(shown.len() - 1),
                    Key::ArrowLeft => hot.saturating_sub(1),
                    Key::ArrowDown => (hot + COLS).min(shown.len() - 1),
                    Key::ArrowUp => hot.saturating_sub(COLS),
                    Key::Home => 0,
                    Key::End => shown.len() - 1,
                    Key::Enter | Key::Space => {
                        picked = Some(shown[hot].0);
                        hot
                    }
                    _ => hot,
                };
            }
            let m = self.mem::<EmojiMem>(id);
            m.hot = Some(hot);
            // Keep the highlighted row in view.
            let row = hot / COLS;
            if row < m.scroll_row {
                m.scroll_row = row;
            } else if row >= m.scroll_row + VISIBLE_ROWS {
                m.scroll_row = row + 1 - VISIBLE_ROWS;
            }
        }
        let scroll_row = self.mem::<EmojiMem>(id).scroll_row;
        let key_hot = if gix.focused { self.mem::<EmojiMem>(id).hot } else { None };
        let focus = hover_cell.or(key_hot);

        let scroll = if reduced { scroll_row as f32 } else { self.spring(grid_id, 0, scroll_row as f32, mo.smooth) };
        let first = scroll_row.saturating_sub(1) * COLS;
        let last = ((scroll_row + VISIBLE_ROWS + 1) * COLS).min(shown.len());
        let cells: Vec<(usize, &'static str, f32, f32)> = (first..last)
            .map(|i| {
                let on = focus == Some(i);
                // Hover: the emoji pops past full size and settles.
                let pop = if reduced { if on { 1.0 } else { 0.0 } } else { self.spring(grid_id.with(i), 0, if on { 1.0 } else { 0.0 }, mo.bouncy) };
                let bg = self.fade(grid_id.with(i), 1, on, mo.hover_in, mo.hover_out, Ease::Linear);
                (i, shown[i].0, pop, bg)
            })
            .collect();
        let hover_bg = self.color(Token::Hover);
        let (ring_a, ring_gap) = self.focus_anim(&gix);
        let ring = self.color(Token::FocusRing);
        let rule = self.color(Token::BorderSubtle);
        let r_cell = self.look().control.radius[0];
        let emoji_style = {
            let mut s = self.text_style(TextRole::Heading2);
            s.size = 22.0;
            s.line_height = 28.0;
            s
        };
        let grid_h = VISIBLE_ROWS as f32 * CELL;
        let empty = shown.is_empty();
        let empty_g = self.galley("No emoji match", &self.text_style(TextRole::Body));
        self.leaf(
            Style { width: Size::Fixed(COLS as f32 * CELL), height: Size::Fixed(grid_h), margin_top: -2.0, ..Default::default() },
            vec2(COLS as f32 * CELL, grid_h),
            Some(grid_id),
            Sense { click: true, drag: false, focusable: true, scroll: rows_total > VISIBLE_ROWS },
            CursorIcon::Pointer,
            Some(Box::new(move |cx, r| {
                cx.painter.push_clip(r.expand2(2.0, 0.0));
                if empty {
                    cx.painter.text(cx.text, &empty_g, vec2(r.center().x - empty_g.width * 0.5, r.center().y - empty_g.height * 0.5), muted);
                }
                for (i, e, pop, bg) in &cells {
                    let row = (*i / COLS) as f32 - scroll;
                    let col = (*i % COLS) as f32;
                    let cr = Rect::from_xywh(r.min.x + col * CELL, r.min.y + row * CELL, CELL, CELL);
                    if *bg > 0.0 {
                        cx.painter.rrect(cr.expand(-2.0), Corners::all(r_cell), hover_bg.multiply_alpha(*bg * 1.6), 0.0, Color::TRANSPARENT);
                    }
                    if n_recent > 0 && *i == n_recent {
                        // A hairline between recents and the category.
                        cx.painter.rect(Rect::from_xywh(r.min.x, cr.min.y - 0.5, r.width(), 1.0), rule);
                    }
                    let mut st = emoji_style;
                    st.size = (22.0 * (1.0 + 0.18 * pop)).round();
                    let g = cx.galley(e, &st);
                    cx.painter.text(cx.text, &g, vec2(cr.center().x - g.width * 0.5, cr.center().y - g.height * 0.5), Color::WHITE);
                }
                cx.painter.pop_clip();
                if ring_a > 0.0 {
                    cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr, r_cell, ring_a, ring_gap, ring)));
                }
            })),
        );

        // Preview of the emoji under the pointer (or highlighted).
        let preview = focus.and_then(|i| shown.get(i)).copied();
        let pa = self.fade(id, 2, preview.is_some(), mo.hover_in, mo.hover_out, Ease::Linear);
        if let Some((e, name)) = preview.or_else(|| shown.first().copied()) {
            let fg = self.color(Token::Foreground);
            self.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 10.0, align: Align::Center, pad: Sides { top: 4.0, ..Default::default() }, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let mut big = emoji_style;
            big.size = 28.0;
            big.line_height = 32.0;
            self.text_leaf(e.to_owned(), big, Color::WHITE.multiply_alpha(0.35 + 0.65 * pa), false, false, Style::default());
            let st = self.text_style(TextRole::BodyStrong);
            self.text_leaf(name.to_owned(), st, fg.multiply_alpha(0.35 + 0.65 * pa), false, true, Style { width: Size::FILL, ..Default::default() });
            self.end();
        }
        self.end();

        if let Some(e) = picked {
            let m = self.mem::<EmojiMem>(id);
            m.recent.retain(|r| *r != e);
            m.recent.insert(0, e);
            m.recent.truncate(COLS * 2);
        }
        picked
    }
}
