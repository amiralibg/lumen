//! A bubble chart that animates between frames (a Gapminder-style
//! timeline), and a dot-matrix display with a 5×7 font.

use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Dir, Size, Style};
use crate::math::{vec2, Corners, Rect, Vec2};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

use super::chart::{compact, nice_ticks};

const SERIES: [Token; 6] = [Token::Series0, Token::Series1, Token::Series2, Token::Series3, Token::Series4, Token::Series5];

// ============================================================ bubble chart

/// One bubble: position, a size encoded as area, and a group for color.
#[derive(Clone, Debug, Default)]
pub struct Bubble {
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub group: usize,
}

impl Bubble {
    pub fn new(label: &str, x: f32, y: f32, size: f32, group: usize) -> Self {
        Bubble { label: label.into(), x, y, size, group }
    }
}

pub struct BubbleChart<'u, 'a> {
    ui: &'u mut Ui,
    key: String,
    bubbles: &'a [Bubble],
    groups: Vec<String>,
    x_label: String,
    y_label: String,
    size_label: String,
    x_range: Option<(f32, f32)>,
    y_range: Option<(f32, f32)>,
    max_size: Option<f32>,
    log_x: bool,
    frame: String,
    height: f32,
    format_x: fn(f32) -> String,
    format_y: fn(f32) -> String,
}

#[derive(Default)]
struct BubbleMem {
    selected: Option<String>,
    hover: Option<String>,
}

impl<'a> BubbleChart<'_, 'a> {
    pub fn groups(mut self, g: &[&str]) -> Self {
        self.groups = g.iter().map(|s| s.to_string()).collect();
        self
    }
    pub fn axes(mut self, x: &str, y: &str) -> Self {
        self.x_label = x.into();
        self.y_label = y.into();
        self
    }
    /// What the bubble size means, for the size legend.
    pub fn size_label(mut self, s: &str) -> Self {
        self.size_label = s.into();
        self
    }
    /// Fixed ranges keep the axes still while a timeline plays.
    pub fn x_range(mut self, lo: f32, hi: f32) -> Self {
        self.x_range = Some((lo, hi));
        self
    }
    pub fn y_range(mut self, lo: f32, hi: f32) -> Self {
        self.y_range = Some((lo, hi));
        self
    }
    /// The size that gets the largest bubble (keeps sizes comparable
    /// across frames).
    pub fn max_size(mut self, s: f32) -> Self {
        self.max_size = Some(s);
        self
    }
    /// Logarithmic x axis.
    pub fn log_x(mut self, on: bool) -> Self {
        self.log_x = on;
        self
    }
    /// A large faint label behind the plot (the timeline's year).
    pub fn frame(mut self, label: &str) -> Self {
        self.frame = label.into();
        self
    }
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    pub fn format(mut self, x: fn(f32) -> String, y: fn(f32) -> String) -> Self {
        self.format_x = x;
        self.format_y = y;
        self
    }

    /// Click or arrow keys select a bubble (`changed`); its details show
    /// in a tooltip and are read by screen readers.
    pub fn show(self) -> Response {
        let BubbleChart { ui, key, bubbles, groups, x_label, y_label, size_label, x_range, y_range, max_size, log_x, frame, height, format_x, format_y } = self;
        let id = ui.make_id(("bubbles", &key));
        let ix = ui.interact(id, Sense { click: true, drag: false, focusable: true, scroll: false }, false);
        let mo = ui.motion();
        let reduced = ui.reduced_motion();
        let lk = ui.look().clone();
        let n = bubbles.len();
        let tx = |x: f32| if log_x { x.max(1e-6).log10() } else { x };
        let (xl, xh) = x_range.map(|(a, b)| (tx(a), tx(b))).unwrap_or_else(|| {
            let lo = bubbles.iter().map(|b| tx(b.x)).fold(f32::MAX, f32::min);
            let hi = bubbles.iter().map(|b| tx(b.x)).fold(f32::MIN, f32::max);
            (lo, hi.max(lo + 1e-3))
        });
        let (yl, yh) = y_range.unwrap_or_else(|| {
            let lo = bubbles.iter().map(|b| b.y).fold(f32::MAX, f32::min);
            let hi = bubbles.iter().map(|b| b.y).fold(f32::MIN, f32::max);
            (lo, hi.max(lo + 1e-3))
        });
        let yt = nice_ticks(yl, yh, (height / 56.0).round().max(2.0) as usize);
        let (y0, y1) = (yt.first().copied().unwrap_or(yl), yt.last().copied().unwrap_or(yh));
        let xt: Vec<f32> = if log_x { ((xl.floor() as i32)..=(xh.ceil() as i32)).map(|e| e as f32).collect() } else { nice_ticks(xl, xh, 5) };
        let (x0, x1) = if log_x { (xl, xh) } else { (xt.first().copied().unwrap_or(xl), xt.last().copied().unwrap_or(xh)) };
        let smax = max_size.unwrap_or_else(|| bubbles.iter().map(|b| b.size).fold(0.0, f32::max)).max(1e-6);
        let r_max = (height * 0.11).max(lk.control.height[0] * 0.8);
        let cap = ui.text_style(TextRole::Caption);
        let y_labels: Vec<String> = yt.iter().map(|v| format_y(*v)).collect();
        let axis_w = y_labels.iter().map(|t| ui.galley(t, &cap).width).fold(0.0, f32::max).ceil() + 12.0;
        let plot_of = move |r: Rect| Rect { min: vec2(r.min.x + axis_w, r.min.y + 8.0), max: vec2(r.max.x - 8.0, r.max.y - 38.0) };
        let place = move |plot: Rect, x: f32, y: f32| vec2(plot.min.x + (tx(x) - x0) / (x1 - x0).max(1e-6) * plot.width(), plot.max.y - (y - y0) / (y1 - y0).max(1e-6) * plot.height());

        // Each bubble springs to its new place, keyed by label, so frames animate.
        let mut shown: Vec<(Vec2, f32)> = Vec::with_capacity(n);
        let plot_last = plot_of(ix.rect);
        for b in bubbles {
            let bid = id.with(("b", &b.label));
            let p = place(plot_last, b.x, b.y);
            let rel = vec2((p.x - plot_last.min.x) / plot_last.width().max(1.0), (p.y - plot_last.min.y) / plot_last.height().max(1.0));
            let rr = (b.size / smax).max(0.0).sqrt();
            let (sx, sy, sr) = if reduced {
                (rel.x, rel.y, rr)
            } else {
                (ui.spring(bid, 0, rel.x, mo.smooth), ui.spring(bid, 1, rel.y, mo.smooth), ui.anims.spring_from(bid, 2, 0.0, rr, mo.smooth))
            };
            shown.push((vec2(sx, sy), sr));
        }
        // Hover: the smallest bubble under the pointer (small ones sit on top).
        let hit = |p: Vec2| -> Option<usize> {
            let plot = plot_of(ix.rect);
            let mut best: Option<(usize, f32)> = None;
            for (i, (rel, r)) in shown.iter().enumerate() {
                let c = vec2(plot.min.x + rel.x * plot.width(), plot.min.y + rel.y * plot.height());
                let rad = (r * r_max).max(4.0);
                if (p - c).length() <= rad && best.is_none_or(|(_, br)| rad < br) {
                    best = Some((i, rad));
                }
            }
            best.map(|b| b.0)
        };
        let hover = if ix.hovered { ix.pointer.and_then(hit) } else { None };
        let mut changed = false;
        {
            let m = ui.mem::<BubbleMem>(id);
            m.hover = hover.map(|i| bubbles[i].label.clone());
            if ix.clicked {
                let h = ix.pointer.and_then(hit).map(|i| bubbles[i].label.clone());
                m.selected = if h == m.selected { None } else { h };
                changed = true;
            }
        }
        if ix.focused && n > 0 {
            let mut order: Vec<usize> = (0..n).collect();
            order.sort_by(|a, b| bubbles[*a].x.partial_cmp(&bubbles[*b].x).unwrap_or(std::cmp::Ordering::Equal));
            for (k, _, _) in ui.keys_for(id) {
                let m = ui.mem::<BubbleMem>(id);
                let cur = m.selected.as_ref().and_then(|l| order.iter().position(|i| bubbles[*i].label == *l));
                let next = match k {
                    Key::ArrowRight | Key::ArrowDown => Some(cur.map_or(0, |c| (c + 1).min(n - 1))),
                    Key::ArrowLeft | Key::ArrowUp => Some(cur.map_or(0, |c| c.saturating_sub(1))),
                    Key::Escape => {
                        m.selected = None;
                        changed = true;
                        None
                    }
                    _ => None,
                };
                if let Some(j) = next {
                    m.selected = Some(bubbles[order[j]].label.clone());
                    changed = true;
                }
            }
        }
        let selected = ui.mem::<BubbleMem>(id).selected.clone();
        let focus_label = hover.map(|i| bubbles[i].label.clone()).or(selected.clone());
        let dim = ui.fade(id, 0, focus_label.is_some(), mo.hover_in, mo.hover_out, Ease::Linear);
        let (ra, rg) = ui.focus_anim(&ix);

        let colors: Vec<Color> = SERIES.iter().map(|t| ui.color(*t)).collect();
        let grid = ui.color(Token::BorderSubtle);
        let axis = ui.color(Token::Border);
        let muted = ui.color(Token::TextMuted);
        let sec = ui.color(Token::TextSecondary);
        let fg = ui.color(Token::Foreground);
        let surface = ui.color(Token::Surface);
        let ring = ui.color(Token::FocusRing);
        let tip = super::chart::Tip::new(ui);
        let mut st_frame = ui.text_style(TextRole::Display);
        st_frame.size = (height * 0.28).round();
        st_frame.line_height = st_frame.size * 1.1;
        st_frame.tabular = true;
        let frame_c = fg.multiply_alpha(0.07);
        let data: Vec<Bubble> = bubbles.to_vec();
        let x_labels: Vec<(f32, String)> = xt.iter().map(|v| (*v, if log_x { format_x(10f32.powf(*v)) } else { format_x(*v) })).collect();
        let fl = focus_label.clone();
        let fr = frame.clone();
        let sel = selected.clone();
        let size_lbl = size_label.clone();
        let (xl_t, yl_t) = (x_label.clone(), y_label.clone());

        ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: lk.control.gap[1], ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        // Legend: groups, then two sizes.
        if !groups.is_empty() || !size_label.is_empty() {
            let items: Vec<(Color, String)> = groups.iter().enumerate().map(|(i, g)| (colors[i % colors.len()], g.clone())).collect();
            super::chart::legend(ui, &items);
        }
        ui.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(240.0, height),
            Some(id),
            Sense { click: true, drag: false, focusable: true, scroll: false },
            if hover.is_some() { CursorIcon::Pointer } else { CursorIcon::Default },
            Some(Box::new(move |cx, r| {
                let plot = plot_of(r);
                // The frame label behind everything.
                if !frame.is_empty() {
                    let g = cx.galley(&frame, &st_frame);
                    cx.painter.text(cx.text, &g, vec2(plot.max.x - g.width - 8.0, plot.max.y - g.height - 4.0), frame_c);
                }
                for (v, l) in yt.iter().zip(&y_labels) {
                    let y = plot.max.y - (v - y0) / (y1 - y0).max(1e-6) * plot.height();
                    cx.painter.dashed_hline(plot.min.x, plot.max.x, y, 4.0, 4.0, grid);
                    let g = cx.galley(l, &cap);
                    cx.painter.text(cx.text, &g, vec2(plot.min.x - 8.0 - g.width, (y - g.height * 0.5).round()), muted);
                }
                cx.painter.rect(Rect::from_xywh(plot.min.x, plot.max.y, plot.width(), 1.0), axis);
                for (v, l) in &x_labels {
                    let x = plot.min.x + (v - x0) / (x1 - x0).max(1e-6) * plot.width();
                    if x < plot.min.x - 1.0 || x > plot.max.x + 1.0 {
                        continue;
                    }
                    cx.painter.rect(Rect::from_xywh(x.round(), plot.max.y, 1.0, 4.0), axis);
                    let g = cx.galley(l, &cap);
                    cx.painter.text(cx.text, &g, vec2((x - g.width * 0.5).clamp(r.min.x, r.max.x - g.width).round(), plot.max.y + 6.0), muted);
                }
                let gx = cx.galley(&xl_t, &cap);
                cx.painter.text(cx.text, &gx, vec2(plot.max.x - gx.width, plot.max.y + 20.0), sec);
                let gy = cx.galley(&yl_t, &cap);
                cx.painter.text(cx.text, &gy, vec2(plot.min.x + 6.0, plot.min.y), sec);
                // Bubbles: biggest first, so small ones stay visible on top.
                let mut order: Vec<usize> = (0..data.len()).collect();
                order.sort_by(|a, b| shown[*b].1.partial_cmp(&shown[*a].1).unwrap_or(std::cmp::Ordering::Equal));
                cx.painter.push_clip(r);
                let mut focus_c = None;
                for &i in &order {
                    let b = &data[i];
                    let (rel, rr) = shown[i];
                    let c = vec2(plot.min.x + rel.x * plot.width(), plot.min.y + rel.y * plot.height());
                    let rad = (rr * r_max).max(3.0);
                    let col = colors[b.group % colors.len()];
                    let is_focus = fl.as_deref() == Some(b.label.as_str());
                    let a = if is_focus { 1.0 } else { 1.0 - 0.6 * dim };
                    cx.painter.circle(c, rad, col.multiply_alpha(0.5 * a), 1.5, col.multiply_alpha(a));
                    if sel.as_deref() == Some(b.label.as_str()) {
                        cx.painter.circle(c, rad + 3.0, Color::TRANSPARENT, 2.0, fg);
                    }
                    let g = cx.galley(&b.label, &cap);
                    if g.width + 6.0 < rad * 2.0 && g.height < rad * 1.6 {
                        cx.painter.text(cx.text, &g, vec2((c.x - g.width * 0.5).round(), (c.y - g.height * 0.5).round()), fg.multiply_alpha(a));
                    }
                    if is_focus {
                        focus_c = Some((i, c, rad));
                    }
                }
                cx.painter.pop_clip();
                // Size legend: two reference circles in the top right.
                if !size_lbl.is_empty() {
                    let big = r_max * 0.6;
                    let small = big * 0.5f32.sqrt() * 0.7;
                    let base = vec2(plot.max.x - big - 4.0, plot.min.y + big * 2.0 + 4.0);
                    cx.painter.circle(base - vec2(0.0, big), big, Color::TRANSPARENT, 1.0, muted);
                    cx.painter.circle(base - vec2(0.0, small), small, Color::TRANSPARENT, 1.0, muted);
                    let g = cx.galley(&size_lbl, &cap);
                    cx.painter.text(cx.text, &g, vec2(base.x - big - g.width - 6.0, base.y - g.height), muted);
                }
                if let Some((i, c, rad)) = focus_c {
                    let b = &data[i];
                    let rows = vec![
                        (colors[b.group % colors.len()], xl_t.clone(), format_x(b.x)),
                        (colors[b.group % colors.len()], yl_t.clone(), format_y(b.y)),
                        (colors[b.group % colors.len()], size_lbl.clone(), compact(b.size)),
                    ];
                    let title = b.label.clone();
                    let tip = tip.clone();
                    cx.defer(r, Box::new(move |cx, _| super::chart::tooltip_panel(cx, c + vec2(rad, 0.0), r.expand(8.0), &title, &rows, &tip)));
                }
                let _ = surface;
                if ra > 0.0 {
                    cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr, 6.0, ra, rg, ring)));
                }
            })),
        );
        ui.end();
        let value = selected.as_ref().and_then(|l| bubbles.iter().find(|b| b.label == *l)).map(|b| format!("{}: {} {}, {} {}, {} {}", b.label, x_label, format_x(b.x), y_label, format_y(b.y), size_label, compact(b.size))).unwrap_or_else(|| format!("{n} bubbles"));
        let lbl = format!("Bubble chart: {y_label} by {x_label}");
        ui.a11y(id, || {
            let mut a = A11y::new(Role::Image).label(lbl).value(value).description("Arrow keys move between bubbles.");
            if !fr.is_empty() {
                a = a.description(format!("Showing {fr}. Arrow keys move between bubbles."));
            }
            a
        });
        Response { id, rect: ix.rect, hovered: ix.hovered, clicked: ix.clicked, changed, focused: ix.focused }
    }
}

// ============================================================ dot matrix

/// 5×7 glyphs, rows top to bottom, '#' lit. Lowercase is drawn as capitals.
fn glyph(c: char) -> [&'static str; 7] {
    match c.to_ascii_uppercase() {
        '0' => [" ### ", "#   #", "#  ##", "# # #", "##  #", "#   #", " ### "],
        '1' => ["  #  ", " ##  ", "  #  ", "  #  ", "  #  ", "  #  ", " ### "],
        '2' => [" ### ", "#   #", "    #", "   # ", "  #  ", " #   ", "#####"],
        '3' => ["#####", "   # ", "  #  ", "   # ", "    #", "#   #", " ### "],
        '4' => ["   # ", "  ## ", " # # ", "#  # ", "#####", "   # ", "   # "],
        '5' => ["#####", "#    ", "#### ", "    #", "    #", "#   #", " ### "],
        '6' => ["  ## ", " #   ", "#    ", "#### ", "#   #", "#   #", " ### "],
        '7' => ["#####", "    #", "   # ", "  #  ", " #   ", " #   ", " #   "],
        '8' => [" ### ", "#   #", "#   #", " ### ", "#   #", "#   #", " ### "],
        '9' => [" ### ", "#   #", "#   #", " ####", "    #", "   # ", " ##  "],
        'A' => [" ### ", "#   #", "#   #", "#####", "#   #", "#   #", "#   #"],
        'B' => ["#### ", "#   #", "#   #", "#### ", "#   #", "#   #", "#### "],
        'C' => [" ### ", "#   #", "#    ", "#    ", "#    ", "#   #", " ### "],
        'D' => ["###  ", "#  # ", "#   #", "#   #", "#   #", "#  # ", "###  "],
        'E' => ["#####", "#    ", "#    ", "#### ", "#    ", "#    ", "#####"],
        'F' => ["#####", "#    ", "#    ", "#### ", "#    ", "#    ", "#    "],
        'G' => [" ### ", "#   #", "#    ", "# ###", "#   #", "#   #", " ####"],
        'H' => ["#   #", "#   #", "#   #", "#####", "#   #", "#   #", "#   #"],
        'I' => [" ### ", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", " ### "],
        'J' => ["  ###", "   # ", "   # ", "   # ", "   # ", "#  # ", " ##  "],
        'K' => ["#   #", "#  # ", "# #  ", "##   ", "# #  ", "#  # ", "#   #"],
        'L' => ["#    ", "#    ", "#    ", "#    ", "#    ", "#    ", "#####"],
        'M' => ["#   #", "## ##", "# # #", "# # #", "#   #", "#   #", "#   #"],
        'N' => ["#   #", "#   #", "##  #", "# # #", "#  ##", "#   #", "#   #"],
        'O' => [" ### ", "#   #", "#   #", "#   #", "#   #", "#   #", " ### "],
        'P' => ["#### ", "#   #", "#   #", "#### ", "#    ", "#    ", "#    "],
        'Q' => [" ### ", "#   #", "#   #", "#   #", "# # #", "#  # ", " ## #"],
        'R' => ["#### ", "#   #", "#   #", "#### ", "# #  ", "#  # ", "#   #"],
        'S' => [" ####", "#    ", "#    ", " ### ", "    #", "    #", "#### "],
        'T' => ["#####", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "  #  "],
        'U' => ["#   #", "#   #", "#   #", "#   #", "#   #", "#   #", " ### "],
        'V' => ["#   #", "#   #", "#   #", "#   #", "#   #", " # # ", "  #  "],
        'W' => ["#   #", "#   #", "#   #", "# # #", "# # #", "# # #", " # # "],
        'X' => ["#   #", "#   #", " # # ", "  #  ", " # # ", "#   #", "#   #"],
        'Y' => ["#   #", "#   #", " # # ", "  #  ", "  #  ", "  #  ", "  #  "],
        'Z' => ["#####", "    #", "   # ", "  #  ", " #   ", "#    ", "#####"],
        ':' => ["     ", "  #  ", "  #  ", "     ", "  #  ", "  #  ", "     "],
        '.' => ["     ", "     ", "     ", "     ", "     ", " ##  ", " ##  "],
        ',' => ["     ", "     ", "     ", "     ", " ##  ", "  #  ", " #   "],
        '-' => ["     ", "     ", "     ", "#####", "     ", "     ", "     "],
        '+' => ["     ", "  #  ", "  #  ", "#####", "  #  ", "  #  ", "     "],
        '/' => ["     ", "    #", "   # ", "  #  ", " #   ", "#    ", "     "],
        '!' => ["  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "     ", "  #  "],
        '?' => [" ### ", "#   #", "    #", "   # ", "  #  ", "     ", "  #  "],
        '\'' => ["  #  ", "  #  ", " #   ", "     ", "     ", "     ", "     "],
        '%' => ["##   ", "##  #", "   # ", "  #  ", " #   ", "#  ##", "   ##"],
        '>' => [" #   ", "  #  ", "   # ", "    #", "   # ", "  #  ", " #   "],
        '<' => ["   # ", "  #  ", " #   ", "#    ", " #   ", "  #  ", "   # "],
        '(' => ["   # ", "  #  ", " #   ", " #   ", " #   ", "  #  ", "   # "],
        ')' => [" #   ", "  #  ", "   # ", "   # ", "   # ", "  #  ", " #   "],
        '#' => [" # # ", " # # ", "#####", " # # ", "#####", " # # ", " # # "],
        '=' => ["     ", "     ", "#####", "     ", "#####", "     ", "     "],
        '&' => [" ##  ", "#  # ", "# #  ", " #   ", "# # #", "#  # ", " ## #"],
        '°' => [" ##  ", "#  # ", " ##  ", "     ", "     ", "     ", "     "],
        _ => ["     ", "     ", "     ", "     ", "     ", "     ", "     "],
    }
}

/// The lit dots of `text`: one column per dot, a blank column between
/// letters. Returns columns of 7 bits.
fn render_dots(text: &str) -> Vec<u8> {
    let mut cols = vec![];
    for (k, c) in text.chars().enumerate() {
        if k > 0 {
            cols.push(0u8);
        }
        let g = glyph(c);
        for x in 0..5 {
            let mut bits = 0u8;
            for (y, row) in g.iter().enumerate() {
                if row.as_bytes().get(x) == Some(&b'#') {
                    bits |= 1 << y;
                }
            }
            cols.push(bits);
        }
    }
    cols
}

#[derive(Default)]
struct DotMem {
    text: String,
    prev: Vec<u8>,
    changed_at: f64,
    born: f64,
}

pub struct DotMatrix<'u> {
    ui: &'u mut Ui,
    key: String,
    text: String,
    cols: usize,
    dot: Option<f32>,
    tone: Token,
    scroll: bool,
    align_right: bool,
}

impl DotMatrix<'_> {
    /// Width in dots (default: just enough for the text).
    pub fn cols(mut self, n: usize) -> Self {
        self.cols = n;
        self
    }
    /// Dot pitch in points.
    pub fn dot(mut self, d: f32) -> Self {
        self.dot = Some(d);
        self
    }
    /// The lit color (default amber: `Token::Warning`).
    pub fn tone(mut self, t: Token) -> Self {
        self.tone = t;
        self
    }
    /// Scroll text that doesn't fit, like a marquee.
    pub fn scroll(mut self, on: bool) -> Self {
        self.scroll = on;
        self
    }
    /// Right-align text that fits (clocks, numbers).
    pub fn align_right(mut self) -> Self {
        self.align_right = true;
        self
    }

    pub fn show(self) -> Response {
        let DotMatrix { ui, key, text, cols, dot, tone, scroll, align_right } = self;
        let id = ui.make_id(("dotmatrix", &key));
        let mo = ui.motion();
        let reduced = ui.reduced_motion();
        let lk = ui.look().clone();
        let now = ui.time();
        let dots = render_dots(&text);
        let cols = if cols == 0 { dots.len().max(1) } else { cols };
        let pitch = dot.unwrap_or((lk.control.height[0] / 7.0).max(3.0));
        // A text change flips the dots column by column.
        {
            let m = ui.mem::<DotMem>(id);
            if m.born == 0.0 {
                m.born = now;
                m.text = text.clone();
                m.prev = vec![0; dots.len()];
                m.changed_at = now;
            } else if m.text != text {
                let old = render_dots(&m.text);
                m.prev = old;
                m.text = text.clone();
                m.changed_at = now;
            }
        }
        let (prev, changed_at, born) = {
            let m = ui.mem::<DotMem>(id);
            (m.prev.clone(), m.changed_at, m.born)
        };
        let per_col = (mo.layer_in * 0.15).max(0.008);
        let flip = (mo.layer_in * 1.5).max(0.12);
        let fits = dots.len() <= cols;
        let scrolling = scroll && !fits && !reduced;
        let span = cols as f32 * per_col + flip;
        let settling = !reduced && (now - changed_at) < span as f64;
        if settling || scrolling {
            ui.anims.keep_alive();
        }
        // Scrolling: the text plus a gap goes by at a steady pace.
        let offset = if scrolling {
            let speed = 1.0 / (mo.layer_in * 0.6).max(0.06);
            let period = dots.len() + cols / 2;
            (((now - born) as f32 * speed) as usize) % period.max(1)
        } else {
            0
        };
        let start = if fits && align_right { cols - dots.len() } else { 0 };
        let lit = ui.color(tone);
        let off = lit.multiply_alpha(0.09);
        // An LED panel is dark in both modes.
        let dk = ui.theme.darkness();
        let bg = ui.color(Token::InverseSurface).mix(ui.color(Token::Background), dk).mix(ui.color(Token::ShadowInk), 0.35);
        let rad = lk.control.radius[1];
        let pad = pitch;
        let w = cols as f32 * pitch + pad * 2.0;
        let h = 7.0 * pitch + pad * 2.0;
        let t_since = (now - changed_at) as f32;
        let period = dots.len() + cols / 2;
        let node = ui.leaf(
            Style { width: Size::Fixed(w), height: Size::Fixed(h), ..Default::default() },
            vec2(w, h),
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(rad), bg, 0.0, Color::TRANSPARENT);
                let dr = pitch * 0.36;
                for x in 0..cols {
                    // Which text column shows here.
                    let src = |d: &[u8]| -> u8 {
                        if scrolling {
                            let k = (x + offset) % period.max(1);
                            if k >= cols / 2 { d.get(k - cols / 2).copied().unwrap_or(0) } else { 0 }
                        } else if x >= start {
                            d.get(x - start).copied().unwrap_or(0)
                        } else {
                            0
                        }
                    };
                    let now_bits = src(&dots);
                    let old_bits = src(&prev);
                    // This column's flip progress.
                    let t = if reduced { 1.0 } else { ((t_since - x as f32 * per_col) / flip).clamp(0.0, 1.0) };
                    let e = Ease::Out.apply(t);
                    for y in 0..7 {
                        let on_now = now_bits & (1 << y) != 0;
                        let on_old = old_bits & (1 << y) != 0;
                        let a = if on_now == on_old { if on_now { 1.0 } else { 0.0 } } else if on_now { e } else { 1.0 - e };
                        let c = vec2(r.min.x + pad + (x as f32 + 0.5) * pitch, r.min.y + pad + (y as f32 + 0.5) * pitch);
                        cx.painter.circle(c, dr, off, 0.0, Color::TRANSPARENT);
                        if a > 0.01 {
                            cx.painter.circle(c, dr * 1.9, lit.multiply_alpha(0.16 * a), 0.0, Color::TRANSPARENT);
                            cx.painter.circle(c, dr, lit.multiply_alpha(a), 0.0, Color::TRANSPARENT);
                        }
                    }
                }
            })),
        );
        let t = text.clone();
        ui.a11y_node(node, || A11y::new(Role::Label).label(t));
        Response { id, rect: ui.rect_of(id).unwrap_or_default(), ..Default::default() }
    }
}

impl Ui {
    /// Bubbles placed by x and y, sized by area, colored by group. Give it
    /// a new frame of data and the bubbles glide there; keep the ranges
    /// fixed with `.x_range`/`.y_range`/`.max_size` while a timeline plays.
    pub fn bubble_chart<'u, 'a>(&'u mut self, key: impl Into<String>, bubbles: &'a [Bubble]) -> BubbleChart<'u, 'a> {
        BubbleChart {
            ui: self,
            key: key.into(),
            bubbles,
            groups: vec![],
            x_label: String::new(),
            y_label: String::new(),
            size_label: String::new(),
            x_range: None,
            y_range: None,
            max_size: None,
            log_x: false,
            frame: String::new(),
            height: 300.0,
            format_x: compact,
            format_y: compact,
        }
    }

    /// Text on a grid of dots, like a departure board or an LED sign. A new
    /// text flips the dots column by column; `.scroll(true)` runs text that
    /// doesn't fit as a marquee.
    pub fn dot_matrix(&mut self, key: impl Into<String>, text: &str) -> DotMatrix<'_> {
        DotMatrix { ui: self, key: key.into(), text: text.into(), cols: 0, dot: None, tone: Token::Warning, scroll: false, align_right: false }
    }
}
