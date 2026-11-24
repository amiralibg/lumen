//! Flex-style layout: rows, columns and stacks with padding, gap,
//! alignment, justification, hug/fixed/fill sizing and min/max constraints.
//!
//! The UI records a tree during the frame; this module sizes and places it
//! after the user's closure returns. Passes:
//! 1. intrinsic (max-content) widths, bottom-up;
//! 2. final widths, top-down (fill and shrink distribution);
//! 3. heights for those widths, bottom-up (text wraps here);
//! 4. placement, top-down (fill heights, justify, align, scroll offsets).

use crate::math::{vec2, Rect, Sides, Vec2};

pub type NodeId = usize;

/// How a node sizes itself along one axis.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Size {
    /// Fit content.
    #[default]
    Hug,
    /// Exact size in logical px.
    Fixed(f32),
    /// Take a share (weight) of the parent's free space.
    Fill(f32),
}

impl Size {
    pub const FILL: Size = Size::Fill(1.0);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Dir {
    Row,
    #[default]
    Column,
    /// Children overlap, each aligned within the node.
    Stack,
}

/// Cross-axis alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
    Stretch,
}

/// Main-axis distribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Justify {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    pub dir: Dir,
    pub width: Size,
    pub height: Size,
    pub min: Vec2,
    pub max: Vec2,
    pub pad: Sides,
    pub gap: f32,
    /// Alignment of children on the cross axis (both axes for stacks).
    pub align: Align,
    pub justify: Justify,
    /// Override of the parent's `align` for this node.
    pub self_align: Option<Align>,
    /// Space before this node in a column, skipped for the first child.
    pub margin_top: f32,
    /// Vertical scrolling: children may overflow; offset applied on place.
    pub scroll: bool,
    /// Rows only: horizontal scrolling. Children keep their natural widths
    /// (nothing shrinks) and are shifted by the node's `scroll_offset_x`.
    pub scroll_x: bool,
    /// Rows only: children that don't fit move to a new line. Lines are
    /// `gap` apart; `Fill` children share the free space of their line and
    /// break by their `min` width.
    pub wrap: bool,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            dir: Dir::Column,
            width: Size::Hug,
            height: Size::Hug,
            min: Vec2::ZERO,
            max: vec2(f32::INFINITY, f32::INFINITY),
            pad: Sides::default(),
            gap: 0.0,
            align: Align::Start,
            justify: Justify::Start,
            self_align: None,
            margin_top: 0.0,
            scroll: false,
            scroll_x: false,
            wrap: false,
        }
    }
}

/// Leaf content.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Content {
    None,
    /// Fixed intrinsic size.
    Fixed(Vec2),
    /// Measured by the [`Measure`] callback (text). The key is opaque.
    Measured(u32),
}

/// Measures [`Content::Measured`] leaves.
pub trait Measure {
    /// Size of content `key` when wrapped to `max_width` (None = unbounded).
    fn measure(&mut self, key: u32, max_width: Option<f32>) -> Vec2;
}

impl<F: FnMut(u32, Option<f32>) -> Vec2> Measure for F {
    fn measure(&mut self, key: u32, max_width: Option<f32>) -> Vec2 {
        self(key, max_width)
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub style: Style,
    pub content: Content,
    pub children: Vec<NodeId>,
    /// Final rect (after layout).
    pub rect: Rect,
    /// Size of the children's bounding box (for scroll containers).
    pub content_size: Vec2,
    /// Scroll offset applied when placing children.
    pub scroll_offset: f32,
    /// Horizontal scroll offset ([`Style::scroll_x`] rows).
    pub scroll_offset_x: f32,
    /// Width of a [`Style::scroll_x`] row's content, padding included.
    pub content_w: f32,
    pref_w: f32,
    /// Narrowest this node can get without squeezing a descendant below
    /// its fixed width or `min` (text and fixed-size content count as 0:
    /// they wrap, ellipsize or scale).
    min_w: f32,
    content_h: f32,
}

#[derive(Default, Debug)]
pub struct Tree {
    pub nodes: Vec<Node>,
}

impl Tree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    pub fn add(&mut self, style: Style, content: Content) -> NodeId {
        self.nodes.push(Node {
            style,
            content,
            children: vec![],
            rect: Rect::ZERO,
            content_size: Vec2::ZERO,
            scroll_offset: 0.0,
            scroll_offset_x: 0.0,
            content_w: 0.0,
            pref_w: 0.0,
            min_w: 0.0,
            content_h: 0.0,
        });
        self.nodes.len() - 1
    }

    pub fn add_child(&mut self, parent: NodeId, style: Style, content: Content) -> NodeId {
        let id = self.add(style, content);
        self.nodes[parent].children.push(id);
        id
    }

    pub fn rect(&self, n: NodeId) -> Rect {
        self.nodes[n].rect
    }

    /// Lay out the subtree at `root` inside `space`. A Hug root takes its
    /// content size (capped to the space); Fill/Fixed behave as usual.
    pub fn layout(&mut self, root: NodeId, space: Rect, m: &mut dyn Measure) {
        self.intrinsic(root, m);
        let st = self.nodes[root].style;
        let w = resolve(st.width, self.nodes[root].pref_w, space.width(), true, st.min.x, st.max.x);
        self.widths(root, w, m);
        let ch = self.heights(root, m);
        let h = resolve(st.height, ch, space.height(), true, st.min.y, st.max.y);
        self.place(root, Rect::from_min_size(space.min, vec2(w, h)));
    }

    /// Size of a Hug root laid out with at most `max_width`.
    pub fn measure_root(&mut self, root: NodeId, max_width: f32, m: &mut dyn Measure) -> Vec2 {
        self.intrinsic(root, m);
        let st = self.nodes[root].style;
        let w = resolve(st.width, self.nodes[root].pref_w, max_width, true, st.min.x, st.max.x);
        self.widths(root, w, m);
        let h = self.heights(root, m);
        vec2(w, clamp(h, st.min.y, st.max.y))
    }

    fn intrinsic(&mut self, n: NodeId, m: &mut dyn Measure) -> f32 {
        let st = self.nodes[n].style;
        let children = std::mem::take(&mut self.nodes[n].children);
        let mut sum = 0.0f32;
        let mut max = 0.0f32;
        let (mut min_sum, mut min_max) = (0.0f32, 0.0f32);
        for &c in &children {
            let w = self.intrinsic(c, m);
            sum += w;
            max = max.max(w);
            min_sum += self.nodes[c].min_w;
            min_max = min_max.max(self.nodes[c].min_w);
        }
        let gaps = st.gap * children.len().saturating_sub(1) as f32;
        let content = match self.nodes[n].content {
            Content::None => 0.0,
            Content::Fixed(v) => v.x,
            Content::Measured(k) => m.measure(k, None).x,
        };
        let inner = match st.dir {
            Dir::Row => sum + gaps,
            Dir::Column | Dir::Stack => max,
        }
        .max(content);
        let pref = match st.width {
            Size::Fixed(w) => w,
            _ => inner + st.pad.horizontal(),
        };
        let pref = clamp(pref, st.min.x, st.max.x);
        // Fixed-size content (a gauge, a chart) is a preference: widgets
        // draw it scaled to their rect, so it isn't a floor.
        let inner_min = match st.dir {
            Dir::Row if st.wrap => min_max,
            Dir::Row => min_sum + gaps,
            Dir::Column | Dir::Stack => min_max,
        };
        let min_w = match st.width {
            Size::Fixed(w) => w,
            // A scroll container scrolls its content instead.
            _ if st.scroll || st.scroll_x => 0.0,
            _ => inner_min + st.pad.horizontal(),
        };
        self.nodes[n].min_w = clamp(min_w.max(st.min.x), st.min.x, st.max.x).min(pref.max(st.min.x));
        self.nodes[n].children = children;
        self.nodes[n].pref_w = pref;
        pref
    }

    fn widths(&mut self, n: NodeId, w: f32, m: &mut dyn Measure) {
        self.nodes[n].rect.max.x = self.nodes[n].rect.min.x + w;
        self.nodes[n].content_size.x = w;
        let st = self.nodes[n].style;
        let inner = (w - st.pad.horizontal()).max(0.0);
        let children = std::mem::take(&mut self.nodes[n].children);
        let mut cw = vec![0.0f32; children.len()];
        match st.dir {
            Dir::Row if st.wrap => {
                for (i, &c) in children.iter().enumerate() {
                    let cs = self.nodes[c].style;
                    cw[i] = match cs.width {
                        Size::Fixed(f) => clamp(f, cs.min.x, cs.max.x),
                        Size::Hug => self.nodes[c].pref_w,
                        Size::Fill(_) => cs.min.x,
                    }
                    .min(inner);
                }
                for line in wrap_lines(&cw, inner, st.gap) {
                    let used: f32 = cw[line.clone()].iter().sum::<f32>() + st.gap * (line.len().saturating_sub(1)) as f32;
                    let free = inner - used;
                    let weights: f32 = line.clone().map(|i| fill_weight(&self.nodes[children[i]].style)).sum();
                    if free > 0.0 && weights > 0.0 {
                        for i in line {
                            let cs = self.nodes[children[i]].style;
                            let wt = fill_weight(&cs);
                            if wt > 0.0 {
                                cw[i] = (cw[i] + free * wt / weights).min(cs.max.x);
                            }
                        }
                    }
                }
            }
            Dir::Row => {
                let gaps = st.gap * children.len().saturating_sub(1) as f32;
                let mut used = gaps;
                let mut weights = 0.0;
                for (i, &c) in children.iter().enumerate() {
                    let cs = self.nodes[c].style;
                    match cs.width {
                        Size::Fixed(f) => cw[i] = clamp(f, cs.min.x, cs.max.x),
                        Size::Hug => cw[i] = self.nodes[c].pref_w,
                        Size::Fill(wt) => {
                            weights += wt;
                            cw[i] = cs.min.x;
                        }
                    }
                    used += cw[i];
                }
                let free = inner - used;
                if free > 0.0 && weights > 0.0 {
                    // Distribute among fill children by weight, respecting max;
                    // leftover from capped children is redistributed.
                    let mut remaining = free;
                    let mut open: Vec<usize> =
                        (0..children.len()).filter(|&i| matches!(self.nodes[children[i]].style.width, Size::Fill(_))).collect();
                    for _ in 0..4 {
                        if remaining <= 0.01 || open.is_empty() {
                            break;
                        }
                        let tw: f32 = open.iter().map(|&i| fill_weight(&self.nodes[children[i]].style)).sum();
                        let mut next = vec![];
                        let mut spent = 0.0;
                        for &i in &open {
                            let cs = self.nodes[children[i]].style;
                            let want = cw[i] + remaining * fill_weight(&cs) / tw;
                            let got = want.min(cs.max.x);
                            spent += got - cw[i];
                            cw[i] = got;
                            if got < want {
                                continue;
                            }
                            next.push(i);
                        }
                        remaining -= spent;
                        if next.len() == open.len() {
                            break;
                        }
                        open = next;
                    }
                } else if free < 0.0 && !st.scroll_x {
                    // Shrink hug children proportionally (text then wraps).
                    let shrinkable: f32 = children
                        .iter()
                        .enumerate()
                        .filter(|(_, &c)| self.nodes[c].style.width == Size::Hug)
                        .map(|(i, &c)| (cw[i] - self.nodes[c].min_w).max(0.0))
                        .sum();
                    if shrinkable > 0.0 {
                        let k = ((-free) / shrinkable).min(1.0);
                        for (i, &c) in children.iter().enumerate() {
                            if self.nodes[c].style.width == Size::Hug {
                                let floor = self.nodes[c].min_w;
                                cw[i] -= (cw[i] - floor).max(0.0) * k;
                            }
                        }
                    }
                }
                // Fill children share space as usual, but never end up
                // narrower than their content's fixed widths.
                for (i, &c) in children.iter().enumerate() {
                    if matches!(self.nodes[c].style.width, Size::Fill(_)) {
                        cw[i] = cw[i].max(self.nodes[c].min_w);
                    }
                }
            }
            Dir::Column | Dir::Stack => {
                for (i, &c) in children.iter().enumerate() {
                    let cs = self.nodes[c].style;
                    let align = cs.self_align.unwrap_or(st.align);
                    cw[i] = match cs.width {
                        Size::Fixed(f) => f,
                        Size::Fill(_) => inner,
                        Size::Hug if align == Align::Stretch => inner,
                        Size::Hug => self.nodes[c].pref_w.min(inner).max(self.nodes[c].min_w),
                    };
                    cw[i] = clamp(cw[i], cs.min.x, cs.max.x);
                }
            }
        }
        for (i, &c) in children.iter().enumerate() {
            self.widths(c, cw[i], m);
        }
        self.nodes[n].children = children;
    }

    fn heights(&mut self, n: NodeId, m: &mut dyn Measure) -> f32 {
        let st = self.nodes[n].style;
        let w = self.nodes[n].content_size.x;
        let children = std::mem::take(&mut self.nodes[n].children);
        let mut sum = 0.0f32;
        let mut max = 0.0f32;
        for (i, &c) in children.iter().enumerate() {
            let h = self.heights(c, m);
            let mt = if i > 0 && st.dir == Dir::Column { self.nodes[c].style.margin_top } else { 0.0 };
            sum += h + mt;
            max = max.max(h);
        }
        let gaps = st.gap * children.len().saturating_sub(1) as f32;
        let inner_w = (w - st.pad.horizontal()).max(0.0);
        let content = match self.nodes[n].content {
            Content::None => 0.0,
            Content::Fixed(v) => v.y,
            Content::Measured(k) => m.measure(k, Some(inner_w)).y,
        };
        let inner = match st.dir {
            Dir::Column => sum + gaps,
            Dir::Row if st.wrap => {
                let ws: Vec<f32> = children.iter().map(|&c| self.nodes[c].content_size.x).collect();
                let lines = wrap_lines(&ws, inner_w, st.gap);
                let n = lines.len();
                lines.into_iter().map(|l| l.map(|i| self.nodes[children[i]].content_h).fold(0.0, f32::max)).sum::<f32>() + st.gap * n.saturating_sub(1) as f32
            }
            Dir::Row | Dir::Stack => max,
        }
        .max(content);
        let content_h = inner + st.pad.vertical();
        self.nodes[n].content_size.y = content_h;
        let h = match st.height {
            Size::Fixed(f) => f,
            _ => content_h,
        };
        let h = clamp(h, st.min.y, st.max.y);
        self.nodes[n].content_h = h;
        self.nodes[n].children = children;
        h
    }

    fn place(&mut self, n: NodeId, rect: Rect) {
        self.nodes[n].rect = rect;
        let st = self.nodes[n].style;
        let inner = Rect {
            min: rect.min + vec2(st.pad.left, st.pad.top),
            max: rect.max - vec2(st.pad.right, st.pad.bottom),
        };
        let iw = inner.width().max(0.0);
        let ih = inner.height().max(0.0);
        let children = std::mem::take(&mut self.nodes[n].children);
        let count = children.len();
        let scroll = if st.scroll { self.nodes[n].scroll_offset } else { 0.0 };
        match st.dir {
            Dir::Row if st.wrap => {
                let ws: Vec<f32> = children.iter().map(|&c| self.nodes[c].content_size.x).collect();
                let mut y = inner.min.y - scroll;
                for line in wrap_lines(&ws, iw, st.gap) {
                    let lh = line.clone().map(|i| self.nodes[children[i]].content_h).fold(0.0, f32::max);
                    let total: f32 = ws[line.clone()].iter().sum::<f32>() + st.gap * line.len().saturating_sub(1) as f32;
                    let (mut x, extra_gap) = justify(st.justify, iw - total, line.len(), false);
                    x += inner.min.x;
                    for i in line {
                        let c = children[i];
                        let cs = self.nodes[c].style;
                        let align = cs.self_align.unwrap_or(st.align);
                        let h = match (cs.height, align) {
                            (Size::Fill(_), _) | (Size::Hug, Align::Stretch) => clamp(lh, cs.min.y, cs.max.y),
                            _ => self.nodes[c].content_h,
                        };
                        self.place(c, Rect::from_min_size(vec2(x, y + cross(align, lh, h)), vec2(ws[i], h)));
                        x += ws[i] + st.gap + extra_gap;
                    }
                    y += lh + st.gap;
                }
            }
            Dir::Column => {
                let mut hs: Vec<f32> = children.iter().map(|&c| self.nodes[c].content_h).collect();
                let margins: Vec<f32> =
                    children.iter().enumerate().map(|(i, &c)| if i > 0 { self.nodes[c].style.margin_top } else { 0.0 }).collect();
                let gaps = st.gap * count.saturating_sub(1) as f32;
                if !st.scroll {
                    let mut weights = 0.0;
                    let mut used = gaps + margins.iter().sum::<f32>();
                    for (i, &c) in children.iter().enumerate() {
                        match self.nodes[c].style.height {
                            Size::Fill(wt) => {
                                weights += wt;
                                hs[i] = self.nodes[c].style.min.y;
                            }
                            _ => {}
                        }
                        used += hs[i];
                    }
                    let free = ih - used;
                    if free > 0.0 && weights > 0.0 {
                        for (i, &c) in children.iter().enumerate() {
                            if let Size::Fill(wt) = self.nodes[c].style.height {
                                let cs = self.nodes[c].style;
                                hs[i] = clamp(hs[i] + free * wt / weights, cs.min.y, cs.max.y);
                            }
                        }
                    }
                }
                let total: f32 = hs.iter().sum::<f32>() + gaps + margins.iter().sum::<f32>();
                let (mut y, extra_gap) = justify(st.justify, ih - total, count, st.scroll);
                y += inner.min.y - scroll;
                for (i, &c) in children.iter().enumerate() {
                    y += margins[i];
                    let cw = self.nodes[c].content_size.x;
                    let align = self.nodes[c].style.self_align.unwrap_or(st.align);
                    let x = inner.min.x + cross(align, iw, cw);
                    self.place(c, Rect::from_min_size(vec2(x, y), vec2(cw, hs[i])));
                    y += hs[i] + st.gap + extra_gap;
                }
                self.nodes[n].content_size.y = total + st.pad.vertical();
            }
            Dir::Row => {
                let ws: Vec<f32> = children.iter().map(|&c| self.nodes[c].content_size.x).collect();
                let total: f32 = ws.iter().sum::<f32>() + st.gap * count.saturating_sub(1) as f32;
                let (mut x, extra_gap) = justify(st.justify, iw - total, count, st.scroll_x);
                x += inner.min.x;
                if st.scroll_x {
                    x -= self.nodes[n].scroll_offset_x;
                    self.nodes[n].content_w = total + st.pad.horizontal();
                }
                for (i, &c) in children.iter().enumerate() {
                    let cs = self.nodes[c].style;
                    let align = cs.self_align.unwrap_or(st.align);
                    let h = match (cs.height, align) {
                        (Size::Fill(_), _) | (Size::Hug, Align::Stretch) => clamp(ih, cs.min.y, cs.max.y),
                        _ => self.nodes[c].content_h,
                    };
                    let y = inner.min.y + cross(align, ih, h) - scroll;
                    self.place(c, Rect::from_min_size(vec2(x, y), vec2(ws[i], h)));
                    x += ws[i] + st.gap + extra_gap;
                }
            }
            Dir::Stack => {
                for &c in &children {
                    let cs = self.nodes[c].style;
                    let align = cs.self_align.unwrap_or(st.align);
                    let w = self.nodes[c].content_size.x;
                    let h = match (cs.height, align) {
                        (Size::Fill(_), _) | (Size::Hug, Align::Stretch) => clamp(ih, cs.min.y, cs.max.y),
                        _ => self.nodes[c].content_h,
                    };
                    let x = inner.min.x + cross(align, iw, w);
                    let y = inner.min.y + cross(align, ih, h) - scroll;
                    self.place(c, Rect::from_min_size(vec2(x, y), vec2(w, h)));
                }
            }
        }
        self.nodes[n].children = children;
    }
}

/// Greedy line breaking for wrapping rows: index ranges of each line.
fn wrap_lines(ws: &[f32], inner: f32, gap: f32) -> Vec<std::ops::Range<usize>> {
    let mut lines = vec![];
    let mut start = 0;
    let mut x = 0.0f32;
    for (i, &w) in ws.iter().enumerate() {
        if i > start && x + gap + w > inner + 0.5 {
            lines.push(start..i);
            start = i;
            x = w;
        } else {
            x += if i > start { gap + w } else { w };
        }
    }
    if start < ws.len() {
        lines.push(start..ws.len());
    }
    lines
}

fn fill_weight(s: &Style) -> f32 {
    match s.width {
        Size::Fill(w) => w.max(1e-6),
        _ => 0.0,
    }
}

fn clamp(v: f32, min: f32, max: f32) -> f32 {
    v.max(min).min(max.max(min))
}

fn resolve(size: Size, content: f32, space: f32, cap: bool, min: f32, max: f32) -> f32 {
    let v = match size {
        Size::Fixed(f) => f,
        Size::Fill(_) => space,
        Size::Hug => {
            if cap {
                content.min(space)
            } else {
                content
            }
        }
    };
    clamp(v, min, max)
}

fn cross(align: Align, space: f32, size: f32) -> f32 {
    match align {
        Align::Start | Align::Stretch => 0.0,
        Align::Center => ((space - size) / 2.0).max(0.0),
        Align::End => (space - size).max(0.0),
    }
}

/// Returns (start offset, extra gap between items).
fn justify(j: Justify, free: f32, count: usize, scroll: bool) -> (f32, f32) {
    if free <= 0.0 || scroll {
        return (0.0, 0.0);
    }
    match j {
        Justify::Start => (0.0, 0.0),
        Justify::Center => (free / 2.0, 0.0),
        Justify::End => (free, 0.0),
        Justify::SpaceBetween if count > 1 => (0.0, free / (count - 1) as f32),
        Justify::SpaceBetween => (0.0, 0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(w: f32, h: f32) -> (Style, Content) {
        (Style::default(), Content::Fixed(vec2(w, h)))
    }

    fn no_measure() -> impl FnMut(u32, Option<f32>) -> Vec2 {
        |_, _| Vec2::ZERO
    }

    /// Fake text: 10 units of width per "char", 20 line height, word-wraps
    /// at 50 per word.
    fn text_measure(chars: u32) -> impl FnMut(u32, Option<f32>) -> Vec2 {
        move |_, max| {
            let full = chars as f32 * 10.0;
            match max {
                None => vec2(full, 20.0),
                Some(w) => {
                    let per_line = ((w / 50.0).floor() * 50.0).max(50.0);
                    let lines = (full / per_line).ceil().max(1.0);
                    vec2(full.min(per_line), lines * 20.0)
                }
            }
        }
    }

    /// Regression (TODO.md): a row narrower than its content used to shrink
    /// hugging children below their descendants' fixed widths, so siblings
    /// overlapped. They now stop at their min-content width.
    #[test]
    fn narrow_row_keeps_descendant_min_widths() {
        let mut t = Tree::new();
        let row = t.add(Style { dir: Dir::Row, width: Size::Fixed(200.0), gap: 10.0, ..Default::default() }, Content::None);
        let a = t.add_child(row, Style::default(), Content::None);
        t.add_child(a, Style { width: Size::Fixed(150.0), ..Default::default() }, Content::Fixed(vec2(150.0, 20.0)));
        let b = t.add_child(row, Style::default(), Content::None);
        t.add_child(b, Style { min: vec2(120.0, 0.0), ..Default::default() }, Content::Fixed(vec2(120.0, 20.0)));
        // A text child can still shrink (it wraps).
        let txt = t.add_child(row, Style::default(), Content::Measured(0));
        t.layout(row, Rect::from_xywh(0.0, 0.0, 200.0, 100.0), &mut text_measure(30));
        assert_eq!(t.rect(a).width(), 150.0);
        assert_eq!(t.rect(b).width(), 120.0);
        assert!(t.rect(b).min.x >= t.rect(a).max.x, "siblings don't overlap");
        assert!(t.rect(txt).width() < 300.0, "text still shrinks");
    }

    /// A horizontally scrolling row keeps its children at full width,
    /// shifts them by the offset and reports its content width.
    #[test]
    fn scroll_x_row_keeps_widths_and_offsets() {
        let mut t = Tree::new();
        let row = t.add(Style { dir: Dir::Row, width: Size::Fixed(100.0), gap: 10.0, scroll_x: true, ..Default::default() }, Content::None);
        let a = t.add_child(row, Style::default(), Content::Measured(0));
        let b = t.add_child(row, Style::default(), Content::Measured(0));
        t.nodes[row].scroll_offset_x = 30.0;
        t.layout(row, Rect::from_xywh(0.0, 0.0, 100.0, 50.0), &mut text_measure(8));
        assert_eq!(t.rect(a).width(), 80.0, "text isn't squeezed");
        assert_eq!(t.rect(a).min.x, -30.0);
        assert_eq!(t.rect(b).min.x, 60.0);
        assert_eq!(t.nodes[row].content_w, 170.0);
        assert_eq!(t.rect(row).width(), 100.0);
    }

    /// Fill children start from their min-content too.
    #[test]
    fn fill_child_keeps_fixed_descendant() {
        let mut t = Tree::new();
        let row = t.add(Style { dir: Dir::Row, width: Size::Fixed(100.0), ..Default::default() }, Content::None);
        let a = t.add_child(row, Style { width: Size::FILL, ..Default::default() }, Content::None);
        t.add_child(a, Style { width: Size::Fixed(80.0), ..Default::default() }, Content::None);
        let b = t.add_child(row, Style { width: Size::FILL, ..Default::default() }, Content::None);
        t.add_child(b, Style { width: Size::Fixed(80.0), ..Default::default() }, Content::None);
        t.layout(row, Rect::from_xywh(0.0, 0.0, 100.0, 100.0), &mut no_measure());
        assert_eq!((t.rect(a).width(), t.rect(b).width()), (80.0, 80.0));
        assert!(t.rect(b).min.x >= t.rect(a).max.x);
    }

    #[test]
    fn column_pad_gap_hug() {
        let mut t = Tree::new();
        let root = t.add(Style { pad: Sides::all(24.0), gap: 14.0, ..Default::default() }, Content::None);
        let (s, c) = fixed(100.0, 20.0);
        let a = t.add_child(root, s, c);
        let (s, c) = fixed(60.0, 30.0);
        let b = t.add_child(root, s, c);
        t.layout(root, Rect::from_xywh(0.0, 0.0, 900.0, 600.0), &mut no_measure());
        assert_eq!(t.rect(root), Rect::from_xywh(0.0, 0.0, 148.0, 112.0));
        assert_eq!(t.rect(a), Rect::from_xywh(24.0, 24.0, 100.0, 20.0));
        assert_eq!(t.rect(b), Rect::from_xywh(24.0, 58.0, 60.0, 30.0));
    }

    #[test]
    fn wrapping_row_breaks_lines_and_fills_the_last() {
        let mut t = Tree::new();
        let root = t.add(Style { dir: Dir::Row, wrap: true, width: Size::Fixed(200.0), gap: 10.0, ..Default::default() }, Content::None);
        let chips: Vec<_> = (0..3).map(|_| t.add_child(root, Style::default(), Content::Fixed(vec2(80.0, 20.0)))).collect();
        let input = t.add_child(root, Style { width: Size::FILL, min: vec2(60.0, 0.0), ..Default::default() }, Content::Fixed(vec2(0.0, 30.0)));
        t.layout(root, Rect::from_xywh(0.0, 0.0, 1000.0, 1000.0), &mut no_measure());
        assert_eq!(t.rect(chips[0]), Rect::from_xywh(0.0, 0.0, 80.0, 20.0));
        assert_eq!(t.rect(chips[1]), Rect::from_xywh(90.0, 0.0, 80.0, 20.0));
        // Line two: a chip, then the fill child takes the rest of the line.
        assert_eq!(t.rect(chips[2]), Rect::from_xywh(0.0, 30.0, 80.0, 20.0));
        assert_eq!(t.rect(input), Rect::from_xywh(90.0, 30.0, 110.0, 30.0));
        assert_eq!(t.rect(root).height(), 60.0);
    }

    #[test]
    fn row_fill_distributes_by_weight() {
        let mut t = Tree::new();
        let root = t.add(Style { dir: Dir::Row, width: Size::Fixed(300.0), gap: 10.0, ..Default::default() }, Content::None);
        let (s, c) = fixed(50.0, 10.0);
        let a = t.add_child(root, s, c);
        let b = t.add_child(root, Style { width: Size::Fill(1.0), ..Default::default() }, Content::Fixed(vec2(0.0, 10.0)));
        let c2 = t.add_child(root, Style { width: Size::Fill(3.0), ..Default::default() }, Content::Fixed(vec2(0.0, 10.0)));
        t.layout(root, Rect::from_xywh(0.0, 0.0, 1000.0, 100.0), &mut no_measure());
        assert_eq!(t.rect(a).width(), 50.0);
        // 300 - 50 - 20 gaps = 230 free: 57.5 and 172.5.
        assert_eq!(t.rect(b).width(), 57.5);
        assert_eq!(t.rect(c2).width(), 172.5);
        assert_eq!(t.rect(c2).max.x, 300.0);
    }

    #[test]
    fn fill_respects_max_and_redistributes() {
        let mut t = Tree::new();
        let root = t.add(Style { dir: Dir::Row, width: Size::Fixed(300.0), ..Default::default() }, Content::None);
        let a = t.add_child(root, Style { width: Size::FILL, max: vec2(50.0, f32::INFINITY), ..Default::default() }, Content::None);
        let b = t.add_child(root, Style { width: Size::FILL, ..Default::default() }, Content::None);
        t.layout(root, Rect::from_xywh(0.0, 0.0, 300.0, 100.0), &mut no_measure());
        assert_eq!(t.rect(a).width(), 50.0);
        assert_eq!(t.rect(b).width(), 250.0);
    }

    #[test]
    fn row_cross_alignment_and_stretch() {
        let mut t = Tree::new();
        let root = t.add(
            Style { dir: Dir::Row, height: Size::Fixed(100.0), align: Align::Center, ..Default::default() },
            Content::None,
        );
        let (s, c) = fixed(10.0, 20.0);
        let a = t.add_child(root, s, c);
        let b = t.add_child(root, Style { self_align: Some(Align::End), ..Default::default() }, Content::Fixed(vec2(10.0, 20.0)));
        let c3 = t.add_child(root, Style { self_align: Some(Align::Stretch), ..Default::default() }, Content::Fixed(vec2(10.0, 20.0)));
        t.layout(root, Rect::from_xywh(0.0, 0.0, 500.0, 500.0), &mut no_measure());
        assert_eq!(t.rect(a).min.y, 40.0);
        assert_eq!(t.rect(b).min.y, 80.0);
        assert_eq!(t.rect(c3).height(), 100.0);
    }

    #[test]
    fn justify_modes() {
        for (j, expect) in [(Justify::Start, 0.0), (Justify::Center, 40.0), (Justify::End, 80.0)] {
            let mut t = Tree::new();
            let root = t.add(Style { dir: Dir::Row, width: Size::Fixed(100.0), justify: j, ..Default::default() }, Content::None);
            let (s, c) = fixed(20.0, 10.0);
            let a = t.add_child(root, s, c);
            t.layout(root, Rect::from_xywh(0.0, 0.0, 100.0, 100.0), &mut no_measure());
            assert_eq!(t.rect(a).min.x, expect, "{j:?}");
        }
        let mut t = Tree::new();
        let root =
            t.add(Style { dir: Dir::Row, width: Size::Fixed(100.0), justify: Justify::SpaceBetween, ..Default::default() }, Content::None);
        let (s, c) = fixed(20.0, 10.0);
        t.add_child(root, s, c);
        let (s, c) = fixed(20.0, 10.0);
        let b = t.add_child(root, s, c);
        t.layout(root, Rect::from_xywh(0.0, 0.0, 100.0, 100.0), &mut no_measure());
        assert_eq!(t.rect(b).max.x, 100.0);
    }

    #[test]
    fn column_fill_height_takes_rest() {
        let mut t = Tree::new();
        let root = t.add(Style { height: Size::Fixed(200.0), width: Size::FILL, pad: Sides::all(10.0), gap: 10.0, ..Default::default() }, Content::None);
        let (s, c) = fixed(10.0, 30.0);
        t.add_child(root, s, c);
        let f = t.add_child(root, Style { height: Size::FILL, width: Size::FILL, ..Default::default() }, Content::None);
        t.layout(root, Rect::from_xywh(0.0, 0.0, 300.0, 600.0), &mut no_measure());
        assert_eq!(t.rect(f), Rect::from_xywh(10.0, 50.0, 280.0, 140.0));
    }

    #[test]
    fn text_wraps_to_fill_width() {
        let mut t = Tree::new();
        let root = t.add(Style { width: Size::Fixed(120.0), ..Default::default() }, Content::None);
        let txt = t.add_child(root, Style { self_align: Some(Align::Stretch), ..Default::default() }, Content::Measured(0));
        t.layout(root, Rect::from_xywh(0.0, 0.0, 1000.0, 1000.0), &mut text_measure(30));
        // 300 wide text in a 120 box: wraps in 100-wide lines -> 3 lines.
        assert_eq!(t.rect(txt).width(), 120.0);
        assert_eq!(t.rect(txt).height(), 60.0);
        assert_eq!(t.rect(root).height(), 60.0);
    }

    #[test]
    fn row_shrinks_hug_text_when_overflowing() {
        let mut t = Tree::new();
        let root = t.add(Style { dir: Dir::Row, width: Size::Fixed(200.0), gap: 0.0, ..Default::default() }, Content::None);
        let txt = t.add_child(root, Style::default(), Content::Measured(0));
        let (s, c) = fixed(50.0, 20.0);
        let fixed_child = t.add_child(root, Style { width: Size::Fixed(50.0), ..s }, c);
        t.layout(root, Rect::from_xywh(0.0, 0.0, 1000.0, 1000.0), &mut text_measure(30));
        assert_eq!(t.rect(txt).width(), 150.0);
        assert_eq!(t.rect(fixed_child).min.x, 150.0);
        assert!(t.rect(txt).height() > 20.0);
    }

    #[test]
    fn min_max_constraints() {
        let mut t = Tree::new();
        let root = t.add(Style { width: Size::Fixed(500.0), ..Default::default() }, Content::None);
        let a = t.add_child(root, Style { min: vec2(64.0, 32.0), ..Default::default() }, Content::Fixed(vec2(20.0, 10.0)));
        let b = t.add_child(root, Style { width: Size::FILL, max: vec2(200.0, f32::INFINITY), ..Default::default() }, Content::None);
        t.layout(root, Rect::from_xywh(0.0, 0.0, 1000.0, 1000.0), &mut no_measure());
        assert_eq!(t.rect(a).size(), vec2(64.0, 32.0));
        assert_eq!(t.rect(b).width(), 200.0);
    }

    #[test]
    fn margin_top_skipped_for_first_child() {
        let mut t = Tree::new();
        let root = t.add(Style::default(), Content::None);
        let a = t.add_child(root, Style { margin_top: 32.0, ..Default::default() }, Content::Fixed(vec2(10.0, 10.0)));
        let b = t.add_child(root, Style { margin_top: 32.0, ..Default::default() }, Content::Fixed(vec2(10.0, 10.0)));
        t.layout(root, Rect::from_xywh(0.0, 0.0, 100.0, 100.0), &mut no_measure());
        assert_eq!(t.rect(a).min.y, 0.0);
        assert_eq!(t.rect(b).min.y, 42.0);
    }

    #[test]
    fn scroll_offsets_children_and_reports_content() {
        let mut t = Tree::new();
        let root = t.add(Style { height: Size::Fixed(100.0), width: Size::Fixed(100.0), scroll: true, ..Default::default() }, Content::None);
        let mut kids = vec![];
        for _ in 0..5 {
            kids.push(t.add_child(root, Style::default(), Content::Fixed(vec2(50.0, 50.0))));
        }
        t.nodes[root].scroll_offset = 30.0;
        t.layout(root, Rect::from_xywh(0.0, 0.0, 100.0, 100.0), &mut no_measure());
        assert_eq!(t.rect(root).height(), 100.0);
        assert_eq!(t.nodes[root].content_size.y, 250.0);
        assert_eq!(t.rect(kids[0]).min.y, -30.0);
        assert_eq!(t.rect(kids[4]).min.y, 170.0);
    }

    #[test]
    fn stack_centers_children() {
        let mut t = Tree::new();
        let root = t.add(Style { dir: Dir::Stack, width: Size::Fixed(100.0), height: Size::Fixed(50.0), align: Align::Center, ..Default::default() }, Content::None);
        let a = t.add_child(root, Style::default(), Content::Fixed(vec2(20.0, 10.0)));
        t.layout(root, Rect::from_xywh(0.0, 0.0, 100.0, 50.0), &mut no_measure());
        assert_eq!(t.rect(a), Rect::from_xywh(40.0, 20.0, 20.0, 10.0));
    }

    #[test]
    fn measure_root_for_overlays() {
        let mut t = Tree::new();
        let root = t.add(Style { pad: Sides::xy(8.0, 4.0), ..Default::default() }, Content::None);
        t.add_child(root, Style { self_align: Some(Align::Stretch), ..Default::default() }, Content::Measured(0));
        let s = t.measure_root(root, 120.0, &mut text_measure(30));
        assert_eq!(s.x, 120.0);
        assert_eq!(s.y, 20.0 * 3.0 + 8.0);
    }
}
