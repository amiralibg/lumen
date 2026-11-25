//! Virtualized rows: build only the rows in view of the enclosing scroll
//! area, for lists of any length.
//!
//! [`Ui::virtual_rows`] lays out `count` rows of varying height as one
//! column: a spacer for the rows above the viewport, the visible rows (plus
//! some overscan), and a spacer for the rows below. Heights are measured as
//! rows are shown and kept per row key in a Fenwick tree, so mapping a scroll
//! offset to a row and back is O(log n); rows never seen use an estimate that
//! follows the average of the measured ones.
//!
//! With [`keys`](VirtualRows::keys), inserting or removing rows above the
//! viewport keeps what you're looking at in place: the first visible row is
//! the anchor, and the enclosing scroll offset moves with it. Measuring a row
//! above the viewport that was taller or shorter than estimated is
//! compensated the same way, so the content never jumps.

use crate::id::Id;
use crate::input::CursorIcon;
use crate::layout::{Dir, Size, Style};
use crate::math::{vec2, Sides};
use crate::ui::{ScrollState, Sense, Ui};
use crate::FastMap;

/// Prefix sums over row heights (1-based Fenwick tree, f64 for precision
/// across 100k+ rows).
#[derive(Default, Clone)]
pub(crate) struct Fenwick {
    t: Vec<f64>,
}

impl Fenwick {
    fn build(vals: &[f32]) -> Self {
        let n = vals.len();
        let mut t = vec![0.0f64; n + 1];
        for i in 1..=n {
            t[i] += vals[i - 1] as f64;
            let j = i + (i & i.wrapping_neg());
            if j <= n {
                let v = t[i];
                t[j] += v;
            }
        }
        Fenwick { t }
    }
    fn add(&mut self, i: usize, d: f64) {
        let mut i = i + 1;
        while i < self.t.len() {
            self.t[i] += d;
            i += i & i.wrapping_neg();
        }
    }
    /// Sum of rows `0..i`.
    fn prefix(&self, i: usize) -> f64 {
        let mut i = i.min(self.t.len().saturating_sub(1));
        let mut s = 0.0;
        while i > 0 {
            s += self.t[i];
            i -= i & i.wrapping_neg();
        }
        s
    }
    fn total(&self) -> f64 {
        self.prefix(self.t.len().saturating_sub(1))
    }
    /// The row containing offset `y` (the last row if past the end).
    fn find(&self, y: f64) -> usize {
        let n = self.t.len().saturating_sub(1);
        if n == 0 {
            return 0;
        }
        let mut pos = 0usize;
        let mut rem = y;
        let mut step = n.next_power_of_two();
        while step > 0 {
            let nxt = pos + step;
            if nxt <= n && self.t[nxt] <= rem {
                pos = nxt;
                rem -= self.t[nxt];
            }
            step >>= 1;
        }
        pos.min(n - 1)
    }
}

#[derive(Default)]
struct VState {
    count: usize,
    keyed: bool,
    keys: Vec<u64>,
    vals: Vec<f32>,
    tree: Fenwick,
    /// Measured heights (row plus gap), by key.
    heights: FastMap<u64, f32>,
    estimate: f32,
    sum: f64,
    measured: u32,
    /// First visible row: key, index and its offset when last shown.
    anchor: Option<(u64, usize, f64)>,
    /// Rows built last frame.
    built: Vec<(usize, u64)>,
    viewport: f32,
}

impl VState {
    fn key(&self, i: usize) -> u64 {
        if self.keyed {
            self.keys[i]
        } else {
            i as u64
        }
    }
    fn rebuild(&mut self, count: usize, key: Option<&dyn Fn(usize) -> u64>) {
        let old_keys = std::mem::take(&mut self.keys);
        let old_vals = std::mem::take(&mut self.vals);
        let was_keyed = self.keyed;
        self.count = count;
        self.keyed = key.is_some();
        if let Some(f) = key {
            self.keys.reserve(count);
            self.keys.extend((0..count).map(f));
        }
        // Inserts and removals at one end shift every row by the same
        // amount: find that shift from the anchor (or the first row) and
        // copy heights across, hashing only rows that don't line up.
        let shift: Option<isize> = if was_keyed && self.keyed && !old_keys.is_empty() {
            let probe = self.anchor.map(|(k, i, _)| (k, i)).unwrap_or((old_keys[0], 0));
            self.index_of(probe.0, probe.1).map(|ni| ni as isize - probe.1 as isize)
        } else {
            None
        };
        let est = self.estimate;
        let mut vals = Vec::with_capacity(count);
        for i in 0..count {
            let k = self.key(i);
            let j = shift.map(|d| i as isize - d).filter(|&j| j >= 0 && (j as usize) < old_keys.len()).map(|j| j as usize);
            let v = match j {
                Some(j) if old_keys[j] == k && old_vals.len() == old_keys.len() => old_vals[j],
                _ if self.heights.is_empty() => est,
                _ => *self.heights.get(&k).unwrap_or(&est),
            };
            vals.push(v);
        }
        self.vals = vals;
        self.tree = Fenwick::build(&self.vals);
    }
    fn index_of(&self, key: u64, near: usize) -> Option<usize> {
        if !self.keyed {
            return ((key as usize) < self.count).then_some(key as usize);
        }
        if self.keys.get(near) == Some(&key) {
            return Some(near);
        }
        // Inserts and removals usually move the anchor a little: look near
        // its old place first.
        for d in 1..64usize {
            if let Some(i) = near.checked_add(d).filter(|&i| self.keys.get(i) == Some(&key)) {
                return Some(i);
            }
            if let Some(i) = near.checked_sub(d).filter(|&i| self.keys.get(i) == Some(&key)) {
                return Some(i);
            }
        }
        self.keys.iter().position(|&k| k == key)
    }
}

/// What [`Ui::virtual_rows`] showed this frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct VirtualOut {
    /// Rows `first..end` were built (overscan included; a pinned row
    /// outside it is built too).
    pub first: usize,
    pub end: usize,
    /// Rows built this frame.
    pub built: usize,
    /// Total height of all rows (estimated for rows not yet shown).
    pub total_height: f32,
    /// Height of the viewport the rows were culled against (last frame).
    pub viewport: f32,
    /// The scroll area the rows live in, if any.
    pub scroll: Option<Id>,
}

/// Builder for [`Ui::virtual_rows`].
#[must_use = "call .show(|ui, i| ...)"]
pub struct VirtualRows<'u, 'k> {
    ui: &'u mut Ui,
    id: Id,
    count: usize,
    estimate: Option<f32>,
    gap: f32,
    key: Option<&'k dyn Fn(usize) -> u64>,
    pin: Option<usize>,
    reveal: Option<usize>,
    overscan: f32,
}

impl<'u, 'k> VirtualRows<'u, 'k> {
    /// Height of a row before it's measured (default 36). Rows shown so far
    /// refine it.
    pub fn estimate(mut self, h: f32) -> Self {
        self.estimate = Some(h.max(1.0));
        self
    }
    /// Space between rows.
    pub fn gap(mut self, g: f32) -> Self {
        self.gap = g.max(0.0);
        self
    }
    /// A stable key per row (an id from your data). With keys, measured
    /// heights and the scroll position follow rows across inserts, removals
    /// and re-sorts. Without, rows are identified by index.
    pub fn keys<'j>(self, f: &'j dyn Fn(usize) -> u64) -> VirtualRows<'u, 'j> {
        let VirtualRows { ui, id, count, estimate, gap, key: _, pin, reveal, overscan } = self;
        VirtualRows { ui, id, count, estimate, gap, key: Some(f), pin, reveal, overscan }
    }
    /// Always build this row, even scrolled out of view (the focused or
    /// selected row, so focus and selection survive scrolling).
    pub fn pin(mut self, i: Option<usize>) -> Self {
        self.pin = i;
        self
    }
    /// Scroll so this row is fully in view (keyboard navigation).
    pub fn reveal(mut self, i: Option<usize>) -> Self {
        self.reveal = i;
        self
    }
    /// Extra pixels built above and below the viewport (default: half a
    /// viewport, at least 200).
    pub fn overscan(mut self, px: f32) -> Self {
        self.overscan = px.max(0.0);
        self
    }

    /// Build the visible rows. `row(ui, index)` adds one row's content; it
    /// is wrapped in a full-width column whose id derives from the row key.
    pub fn show(self, mut row: impl FnMut(&mut Ui, usize)) -> VirtualOut {
        let VirtualRows { ui, id, count, estimate, gap, key, pin, reveal, overscan } = self;
        let row_id = |k: u64| id.with(("vrow", k));
        let mut st = std::mem::take(ui.mem::<VState>(id));
        if st.estimate <= 0.0 {
            st.estimate = estimate.unwrap_or(36.0) + gap;
        }

        // 1. Rows changed (count, or the anchor row moved): re-index.
        let moved = st.keyed && st.anchor.is_some_and(|(k, i, _)| key.is_some_and(|f| i >= count || f(i) != k));
        if count != st.count || st.keyed != key.is_some() || moved || st.vals.len() != count {
            st.rebuild(count, key);
        }

        // 2. Measure what was shown last frame.
        let built = std::mem::take(&mut st.built);
        for &(i, k) in &built {
            let Some(r) = ui.rect_of(row_id(k)) else { continue };
            let h = r.height();
            if h <= 0.0 {
                continue;
            }
            match st.heights.insert(k, h) {
                None => {
                    st.sum += h as f64;
                    st.measured += 1;
                }
                Some(old) => st.sum += (h - old) as f64,
            }
            if i < st.count && st.key(i) == k && (st.vals[i] - h).abs() > 0.01 {
                st.tree.add(i, (h - st.vals[i]) as f64);
                st.vals[i] = h;
            }
        }
        // Unseen rows follow the measured average once it's clearly off.
        if st.measured >= 8 {
            let avg = (st.sum / st.measured as f64) as f32;
            if (avg - st.estimate).abs() > st.estimate * 0.15 {
                st.estimate = avg;
                st.rebuild(count, key);
            }
        }

        // 3. Where the viewport is, in the rows' own coordinates.
        let scroll = ui.scroll_ctx.last().copied();
        let list_rect = ui.rect_of(id);
        let view = scroll.and_then(|(sid, _)| ui.rect_of(sid)).unwrap_or_else(|| ui.content_rect());
        let delta = scroll.map(|s| s.1).unwrap_or(0.0);
        // Keep the anchor row where it was on screen.
        let mut shift = 0.0f64;
        if let (Some((k, i, top)), Some(_)) = (st.anchor, scroll) {
            if let Some(ni) = st.index_of(k, i) {
                shift = st.tree.prefix(ni) - top;
            }
        }
        let list_top = list_rect.map(|r| r.min.y as f64 - delta as f64 - shift).unwrap_or(view.min.y as f64);
        let mut s = view.min.y as f64 - list_top;
        let mut e = view.max.y as f64 - list_top;
        st.viewport = view.height();

        // 4. Reveal a row (keyboard navigation).
        if let (Some(r), Some(_)) = (reveal.filter(|&r| r < count), scroll) {
            let (top, bot) = (st.tree.prefix(r), st.tree.prefix(r + 1) - gap as f64);
            let d = if top < s.max(0.0) {
                top - s.max(0.0)
            } else if bot > e {
                (bot - e).min(top - s)
            } else {
                0.0
            };
            shift += d;
            s += d;
            e += d;
        }
        if shift.abs() > 0.01 {
            if let Some((sid, _)) = scroll {
                let ss = ui.mem::<ScrollState>(sid);
                ss.offset = (ss.offset + shift as f32).max(0.0);
                if let Some(t) = ss.target.as_mut() {
                    *t = (*t + shift as f32).max(0.0);
                }
            }
        }

        // 5. Rows to build.
        let over = if overscan > 0.0 { overscan as f64 } else { (st.viewport as f64 * 0.5).max(200.0) };
        let total = st.tree.total();
        let (first, end) = if count == 0 || e < 0.0 || s > total {
            (0, 0)
        } else {
            let a = st.tree.find((s - over).max(0.0));
            let b = (st.tree.find((e + over).max(0.0)) + 1).min(count);
            (a, b)
        };
        let anchor_i = if count > 0 { st.tree.find(s.max(0.0)) } else { 0 };
        st.anchor = (count > 0 && scroll.is_some()).then(|| (st.key(anchor_i), anchor_i, st.tree.prefix(anchor_i)));

        let mut segs: Vec<(usize, usize)> = vec![];
        if let Some(p) = pin.filter(|&p| p < count && (p < first || p >= end)) {
            if p < first {
                segs.push((p, p + 1));
            }
            if first < end {
                segs.push((first, end));
            }
            if p >= end {
                segs.push((p, p + 1));
            }
        } else if first < end {
            segs.push((first, end));
        }

        // 6. Build: spacer, rows, spacer.
        ui.begin(Style { dir: Dir::Column, width: Size::FILL, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);
        let spacer = |ui: &mut Ui, h: f64| {
            if h > 0.5 {
                ui.leaf(Style { width: Size::FILL, ..Default::default() }, vec2(0.0, h as f32), None, Sense::NONE, CursorIcon::Default, None);
            }
        };
        let mut at = 0usize;
        let mut built_now = Vec::with_capacity(segs.iter().map(|s| s.1 - s.0).sum());
        for (a, b) in segs {
            spacer(ui, st.tree.prefix(a) - st.tree.prefix(at));
            for i in a..b {
                let k = st.key(i);
                ui.begin(
                    Style { dir: Dir::Column, width: Size::FILL, pad: Sides { bottom: gap, ..Default::default() }, ..Default::default() },
                    Some(row_id(k)),
                    Sense::NONE,
                    CursorIcon::Default,
                    false,
                    None,
                );
                ui.scope(("vrow", k), |ui| row(ui, i));
                ui.end();
                built_now.push((i, k));
            }
            at = b;
        }
        spacer(ui, total - st.tree.prefix(at));
        ui.end();

        let out = VirtualOut {
            first,
            end,
            built: built_now.len(),
            total_height: total as f32,
            viewport: st.viewport,
            scroll: scroll.map(|s| s.0),
        };
        st.built = built_now;
        *ui.mem::<VState>(id) = st;
        out
    }
}

impl Ui {
    /// Rows `0..count`, building only those in view of the enclosing
    /// [`scroll`](Ui::scroll) area (or the window). See [`VirtualRows`].
    ///
    /// ```ignore
    /// ui.scroll("log").show(|ui| {
    ///     ui.virtual_rows("lines", lines.len()).estimate(24.0).show(|ui, i| {
    ///         ui.label(&lines[i]);
    ///     });
    /// });
    /// ```
    pub fn virtual_rows(&mut self, key: impl std::hash::Hash, count: usize) -> VirtualRows<'_, 'static> {
        let id = self.make_id(("virtual_rows", key));
        VirtualRows { ui: self, id, count, estimate: None, gap: 0.0, key: None, pin: None, reveal: None, overscan: 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::Fenwick;

    #[test]
    fn fenwick_prefix_and_find() {
        let vals: Vec<f32> = (0..1000).map(|i| 10.0 + (i % 7) as f32).collect();
        let mut f = Fenwick::build(&vals);
        let mut acc = 0.0f64;
        for (i, v) in vals.iter().enumerate() {
            assert!((f.prefix(i) - acc).abs() < 1e-6);
            assert_eq!(f.find(acc), i, "row {i} starts at {acc}");
            assert_eq!(f.find(acc + *v as f64 - 0.01), i);
            acc += *v as f64;
        }
        assert_eq!(f.find(acc + 100.0), 999);
        f.add(10, 5.0);
        assert!((f.prefix(11) - (f.prefix(10) + vals[10] as f64 + 5.0)).abs() < 1e-6);
    }
}
