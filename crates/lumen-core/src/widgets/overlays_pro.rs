//! Overlays and feedback: a veil over sensitive content, a getting-started
//! checklist that lives in a corner, rows with swipe actions, and docking
//! panels you drag between the sides of a window.
//!
//! The veil and an open swipe row cover their content with a leaf pulled
//! up over it (`margin_top` = minus the content's height), so it sits on
//! top for painting and for hit-testing.

use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides, Vec2};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

use super::Tone;

fn hash01(a: u32, b: u32) -> f32 {
    let mut h = a.wrapping_mul(0x9E37_79B9) ^ b.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xFFFF) as f32 / 65535.0
}

fn tone_token(t: Tone) -> Token {
    match t {
        Tone::Neutral => Token::TextSecondary,
        Tone::Accent => Token::Accent,
        Tone::Success => Token::Success,
        Tone::Warning => Token::Warning,
        Tone::Danger => Token::Danger,
        Tone::Info => Token::Info,
    }
}

// ============================================================ veil

#[derive(Default)]
struct VeilMem {
    /// Where the reveal started (in the content, 0..1) and when.
    origin: Vec2,
    changed_at: f64,
    was: Option<bool>,
}

// ============================================================ onboarding

/// A step of a getting-started checklist.
#[derive(Clone, Debug, Default)]
pub struct OnboardingStep {
    pub title: String,
    pub detail: String,
    /// The current step's button.
    pub action: String,
    pub done: bool,
}

impl OnboardingStep {
    pub fn new(title: &str, detail: &str, action: &str, done: bool) -> Self {
        OnboardingStep { title: title.into(), detail: detail.into(), action: action.into(), done }
    }
}

/// What the person did in the checklist this frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct OnboardingOut {
    /// The step whose action button was pressed.
    pub action: Option<usize>,
    /// Dismiss was pressed after everything was done.
    pub dismissed: bool,
}

#[derive(Default)]
struct OnboardMem {
    done: Vec<bool>,
    popped: Vec<(usize, f64)>,
    all_at: f64,
}

// ============================================================ swipe actions

/// A button revealed by swiping a row.
#[derive(Clone, Debug)]
pub struct SwipeAction {
    pub label: String,
    pub icon: String,
    pub tone: Tone,
}

impl SwipeAction {
    pub fn new(label: &str, icon: &str, tone: Tone) -> Self {
        SwipeAction { label: label.into(), icon: icon.into(), tone }
    }
}

/// An action that fired: from the leading (left) or trailing (right) side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwipeHit {
    pub trailing: bool,
    pub index: usize,
}

#[derive(Default)]
struct SwipeMem {
    /// Resting offset: 0 closed, + leading open, - trailing open.
    open: f32,
    drag: Option<(f32, f32)>,
    live: f32,
    scroll_at: f64,
    /// A full swipe fired: the row flies out, then comes back.
    fired: Option<(SwipeHit, f64)>,
    content_h: f32,
}

// ============================================================ docking

/// Where a panel is docked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DockZone {
    Left,
    Center,
    Right,
    Bottom,
}

impl DockZone {
    pub const ALL: [DockZone; 4] = [DockZone::Left, DockZone::Center, DockZone::Right, DockZone::Bottom];
    fn ix(self) -> usize {
        self as usize
    }
    fn name(self) -> &'static str {
        match self {
            DockZone::Left => "Left dock",
            DockZone::Center => "Editor area",
            DockZone::Right => "Right dock",
            DockZone::Bottom => "Bottom dock",
        }
    }
}

/// A panel that can be docked.
#[derive(Clone, Debug)]
pub struct DockPanel {
    pub id: String,
    pub title: String,
    pub icon: String,
}

impl DockPanel {
    pub fn new(id: &str, title: &str, icon: &str) -> Self {
        DockPanel { id: id.into(), title: title.into(), icon: icon.into() }
    }
}

/// Which panels sit where, which tab is showing in each zone, and the dock
/// sizes in points.
#[derive(Clone, Debug, PartialEq)]
pub struct DockLayout {
    pub zones: [Vec<String>; 4],
    pub active: [usize; 4],
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Default for DockLayout {
    fn default() -> Self {
        DockLayout { zones: Default::default(), active: [0; 4], left: 200.0, right: 200.0, bottom: 140.0 }
    }
}

impl DockLayout {
    pub fn zone_of(&self, id: &str) -> Option<DockZone> {
        DockZone::ALL.into_iter().find(|z| self.zones[z.ix()].iter().any(|p| p == id))
    }
    /// Move a panel to `zone` and show it there.
    pub fn move_to(&mut self, id: &str, zone: DockZone) {
        for z in DockZone::ALL {
            let v = &mut self.zones[z.ix()];
            if let Some(k) = v.iter().position(|p| p == id) {
                v.remove(k);
                self.active[z.ix()] = self.active[z.ix()].min(v.len().saturating_sub(1));
            }
        }
        let v = &mut self.zones[zone.ix()];
        v.push(id.to_owned());
        self.active[zone.ix()] = v.len() - 1;
    }
}

#[derive(Default)]
struct DockMem {
    /// The tab being dragged: panel id, press point, and whether it moved.
    drag: Option<(String, Vec2, bool)>,
    target: Option<DockZone>,
    resize: Option<(usize, f32, f32)>,
}

impl Ui {
    /// Cover `content` with a frosted veil until it's revealed: a spoiler,
    /// an API key, a sensitive image. A click (or Enter) dissolves the veil
    /// outward from where you clicked; setting `revealed` to false brings
    /// it back. While veiled, the content can't be clicked or read by a
    /// screen reader.
    pub fn veil<R>(&mut self, key: impl std::hash::Hash, revealed: &mut bool, label: &str, content: impl FnOnce(&mut Ui) -> R) -> R {
        let id = self.make_id(("veil", key));
        let mo = self.motion();
        let reduced = self.reduced_motion();
        let lk = self.look().clone();
        let now = self.time();
        let cover_id = id.with("cover");
        let cix = self.interact(cover_id, Sense { click: true, drag: false, focusable: true, scroll: false }, false);
        let (act, _) = super::key_activate(self, &cix, true);
        if !*revealed && (cix.clicked || act) {
            *revealed = true;
            let r = cix.rect;
            let o = match cix.pointer.filter(|_| cix.clicked) {
                Some(p) if r.width() > 0.0 => vec2((p.x - r.min.x) / r.width(), (p.y - r.min.y) / r.height()),
                _ => vec2(0.5, 0.5),
            };
            self.mem::<VeilMem>(id).origin = o;
        }
        {
            let m = self.mem::<VeilMem>(id);
            if m.was != Some(*revealed) {
                if m.was.is_some() {
                    m.changed_at = now;
                }
                m.was = Some(*revealed);
            }
        }
        let (origin, changed_at) = {
            let m = self.mem::<VeilMem>(id);
            (m.origin, m.changed_at)
        };
        let dur = (mo.layer_in * 5.0).max(0.45);
        let t = if reduced || changed_at == 0.0 { 1.0 } else { ((now - changed_at) as f32 / dur).clamp(0.0, 1.0) };
        if t < 1.0 {
            self.anims.keep_alive();
        }
        // 0 = fully veiled, 1 = fully revealed.
        let shown = if *revealed { Ease::Out.apply(t) } else { 1.0 - Ease::Out.apply(t) };
        let veiled = shown < 0.999;

        let content_id = id.with("content");
        let h = self.rect_of(content_id).map(|r| r.height()).unwrap_or(0.0);
        // Fully veiled (or not yet measured), the content isn't drawn at
        // all: nothing to peek at between the frost cells.
        let hide_first = (h <= 0.0 && !*revealed) || shown <= 0.001;
        let rad = lk.control.radius[2];
        self.begin(Style { dir: Dir::Column, width: Size::FILL, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);
        let node = self.begin(Style { dir: Dir::Column, width: Size::FILL, ..Default::default() }, Some(content_id), Sense::NONE, CursorIcon::Default, false, None);
        if hide_first {
            self.set_paint(node, Box::new(|cx, _| cx.painter.push_layer(0.0, Vec2::ZERO)));
            self.set_paint_after(node, Box::new(|cx, _| cx.painter.pop_layer()));
        }
        let out = content(self);
        self.end();
        if !*revealed {
            self.a11y_hide(node);
        }
        if veiled && h > 0.0 {
            let frost_a = self.color(Token::SurfaceMuted);
            let frost_b = self.color(Token::SurfaceRaised).mix(self.color(Token::BorderStrong), 0.45);
            let fg = self.color(Token::Foreground);
            let sec = self.color(Token::TextSecondary);
            let pill = self.color(Token::SurfaceRaised);
            let pill_bd = self.color(Token::Border);
            let ring = self.color(Token::FocusRing);
            let (ra, rg) = self.focus_anim(&cix);
            let hov = self.fade(cover_id, 0, cix.hovered, mo.hover_in, mo.hover_out, Ease::Linear);
            let cell = (lk.control.gap[0] * 1.5).max(5.0);
            let st = self.text_style(TextRole::BodyStrong);
            let st_c = self.text_style(TextRole::Caption);
            let title = label.to_owned();
            let hint = "Click to reveal".to_owned();
            let pad_x = lk.control.pad_x[1];
            let pad_y = lk.control.gap[1];
            let inner = lk.control.gap[1];
            self.leaf(
                Style { width: Size::FILL, height: Size::Fixed(h), margin_top: -h, ..Default::default() },
                vec2(10.0, h),
                Some(cover_id),
                Sense { click: true, drag: false, focusable: true, scroll: false },
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| {
                    // A dither of frosted cells that dissolve from the origin.
                    cx.painter.push_clip(r);
                    let cols = (r.width() / cell).ceil() as u32;
                    let rows = (r.height() / cell).ceil() as u32;
                    let o = vec2(r.min.x + origin.x * r.width(), r.min.y + origin.y * r.height());
                    let far = [r.min, vec2(r.max.x, r.min.y), r.max, vec2(r.min.x, r.max.y)].iter().map(|c| (*c - o).length()).fold(1.0f32, f32::max);
                    for y in 0..rows {
                        for x in 0..cols {
                            let n = hash01(x, y);
                            let c = vec2(r.min.x + (x as f32 + 0.5) * cell, r.min.y + (y as f32 + 0.5) * cell);
                            let threshold = (c - o).length() / far * 0.7 + n * 0.3;
                            if shown > threshold {
                                continue;
                            }
                            let col = frost_a.mix(frost_b, n).mix(frost_b, hov * 0.3);
                            cx.painter.rect(Rect::from_xywh(r.min.x + x as f32 * cell, r.min.y + y as f32 * cell, cell + 0.5, cell + 0.5), col);
                        }
                    }
                    cx.painter.pop_clip();
                    // The label, fading as the veil goes.
                    let a = (1.0 - shown * 2.0).clamp(0.0, 1.0);
                    if a > 0.0 {
                        let g = cx.galley(&title, &st);
                        let gh = cx.galley(&hint, &st_c);
                        let isz = st.size + 4.0;
                        let w = isz + pad_x * 2.0 + inner + g.width.max(gh.width);
                        let ph = g.height + gh.height + pad_y * 2.0;
                        let p = Rect::from_center_size(r.center(), vec2(w, ph));
                        cx.painter.rrect(p, Corners::all(rad), pill.multiply_alpha(a), 1.0, pill_bd.multiply_alpha(a));
                        cx.icon("eye-off", vec2(p.min.x + pad_x + isz * 0.5, p.center().y), isz, sec.multiply_alpha(a));
                        let tx = p.min.x + pad_x + isz + inner;
                        cx.painter.text(cx.text, &g, vec2(tx, p.min.y + pad_y), fg.multiply_alpha(a));
                        cx.painter.text(cx.text, &gh, vec2(tx, p.min.y + pad_y + g.height), sec.multiply_alpha(a));
                    }
                    if ra > 0.0 {
                        cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr, rad, ra, rg, ring)));
                    }
                })),
            );
            if !*revealed {
                let l = label.to_owned();
                self.a11y(cover_id, || A11y::new(Role::Button).label(format!("Reveal {l}")).description("Hidden until you reveal it"));
            } else {
                self.a11y(cover_id, A11y::hidden);
            }
        }
        self.end();
        out
    }

    /// A getting-started checklist that lives in a corner: a pill with
    /// progress that opens into the list. The next step shows its detail
    /// and button; finished steps check off with a pop; when everything is
    /// done it celebrates and offers Dismiss.
    pub fn onboarding_checklist(&mut self, key: impl std::hash::Hash, title: &str, steps: &[OnboardingStep], open: &mut bool) -> OnboardingOut {
        let id = self.make_id(("onboarding", key));
        let lk = self.look().clone();
        let mo = self.motion();
        let reduced = self.reduced_motion();
        let now = self.time();
        let gap = lk.control.gap[1];
        let n = steps.len();
        let done_n = steps.iter().filter(|s| s.done).count();
        let all = n > 0 && done_n == n;
        let current = steps.iter().position(|s| !s.done);
        let mut out = OnboardingOut::default();
        // Steps that just got done pop their check; finishing celebrates.
        {
            let m = self.mem::<OnboardMem>(id);
            if m.done.len() == n {
                for (i, s) in steps.iter().enumerate() {
                    if s.done && !m.done[i] {
                        m.popped.push((i, now));
                    }
                }
                if all && m.done.iter().any(|d| !d) {
                    m.all_at = now;
                }
            }
            m.done = steps.iter().map(|s| s.done).collect();
            m.popped.retain(|(_, t)| now - t < 1.0);
        }
        let prog = if reduced { done_n as f32 / n.max(1) as f32 } else { self.spring(id, 0, done_n as f32 / n.max(1) as f32, mo.smooth) };
        let open_t = if reduced { if *open { 1.0 } else { 0.0 } } else { self.spring(id, 1, if *open { 1.0 } else { 0.0 }, mo.smooth) };
        let accent = self.color(Token::Accent);
        let ok = self.color(Token::Success);
        let track = self.color(Token::Track);
        let fg = self.color(Token::Foreground);
        let sec = self.color(Token::TextSecondary);
        let muted = self.color(Token::TextMuted);
        let ring_c = self.color(Token::FocusRing);
        let rad = lk.card.radius;
        let width = (lk.control.height[1] * 9.0).round();

        let ring = move |cx: &mut crate::ui::PaintCx, c: Vec2, r: f32, w: f32, p: f32, col: Color| {
            cx.painter.arc_band(c, r - w, r, 0.0, std::f32::consts::TAU, track);
            if p > 0.002 {
                cx.painter.arc_band(c, r - w, r, 0.0, std::f32::consts::TAU * p.min(1.0), col);
            }
        };

        self.begin(Style { dir: Dir::Column, gap, align: Align::End, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);
        // The panel.
        if open_t > 0.01 {
            let bg = self.color(Token::SurfaceRaised);
            let bd = self.color(Token::Border);
            let shadows = crate::theme::Elevation::Level2.shadows(self.theme.darkness(), self.color(Token::ShadowInk));
            let a = open_t.clamp(0.0, 1.0);
            self.column().layer(a, vec2(0.0, (1.0 - a) * 16.0)).show(|ui| {
                ui.begin(Style { dir: Dir::Column, width: Size::Fixed(width), pad: Sides::all(gap * 1.5), gap, ..Default::default() }, Some(id.with("panel")), Sense::NONE, CursorIcon::Default, false, Some(Box::new(move |cx, r| {
                    cx.painter.shadows(r, Corners::all(rad), &shadows);
                    cx.painter.rrect(r, Corners::all(rad), bg, 1.0, bd);
                })));
                // Header: progress ring, title and count, collapse.
                ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                let rs = lk.control.height[1];
                let col = if all { ok } else { accent };
                ui.leaf(Style::default(), vec2(rs, rs), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| {
                    ring(cx, r.center(), rs * 0.5, rs * 0.14, prog, col);
                    if all {
                        cx.icon("check", r.center(), rs * 0.5, ok);
                    }
                })));
                ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 1.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                let sth = ui.text_style(TextRole::Heading4);
                ui.text_leaf(if all { "You're all set".to_owned() } else { title.to_owned() }, sth, fg, false, true, Style::default());
                let stc = ui.text_style(TextRole::Caption);
                ui.text_leaf(format!("{done_n} of {n} done"), stc, sec, false, false, Style::default());
                ui.end();
                if ui.scope(id.with("collapse"), |ui| ui.icon_button("chevron-down").ghost().small().tooltip("Minimize").show().clicked) {
                    *open = false;
                }
                ui.end();
                // A burst of dots when everything's done.
                let all_at = ui.mem::<OnboardMem>(id).all_at;
                let burst = if all_at > 0.0 && !reduced { ((now - all_at) as f32 / 0.9).clamp(0.0, 1.0) } else { 1.0 };
                if burst < 1.0 {
                    ui.anims.keep_alive();
                    let series: Vec<Color> = [Token::Series0, Token::Series1, Token::Series2, Token::Series3, Token::Series4].iter().map(|t| ui.color(*t)).collect();
                    let n_r = ui.last_node();
                    let _ = n_r;
                    ui.leaf(Style { width: Size::FILL, height: Size::Fixed(0.0), ..Default::default() }, vec2(1.0, 0.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| {
                        let c = vec2(r.min.x + rs * 0.5, r.min.y - rs * 0.5 - gap);
                        let e = Ease::Out.apply(burst);
                        for k in 0..14 {
                            let a = k as f32 / 14.0 * std::f32::consts::TAU + hash01(k, 3) * 0.4;
                            let d = rs * (0.6 + 1.6 * e) * (0.7 + 0.5 * hash01(k, 9));
                            let p = c + vec2(a.cos(), a.sin()) * d;
                            cx.painter.circle(p, 2.5 * (1.0 - burst * 0.6), series[k as usize % series.len()].multiply_alpha(1.0 - burst), 0.0, Color::TRANSPARENT);
                        }
                    })));
                }
                // Steps.
                for (i, s) in steps.iter().enumerate() {
                    let sid = id.with(("step", i));
                    let is_cur = Some(i) == current;
                    let pop_at = ui.mem::<OnboardMem>(id).popped.iter().find(|p| p.0 == i).map(|p| p.1);
                    let pop = match pop_at {
                        Some(t0) if !reduced => {
                            let t = ((now - t0) as f32 / 0.5).clamp(0.0, 1.0);
                            ui.anims.keep_alive();
                            1.0 + 0.35 * (t * std::f32::consts::PI).sin()
                        }
                        _ => 1.0,
                    };
                    let cur_t = ui.fade(sid, 0, is_cur, mo.selected, mo.selected, Ease::Out);
                    let isz = lk.control.height[0] * 0.7;
                    let done = s.done;
                    let row_bg = ui.color(Token::AccentSubtle).multiply_alpha(cur_t * 0.6);
                    let r0 = lk.control.radius[1];
                    ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: gap * 0.75, pad: Sides::xy(gap * 0.5, gap * 0.5), align: Align::Start, ..Default::default() }, Some(sid), Sense::NONE, CursorIcon::Default, false, Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(r0), row_bg, 0.0, Color::TRANSPARENT))));
                    ui.leaf(Style::default(), vec2(isz, isz), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| {
                        let c = r.center();
                        if done {
                            cx.painter.circle(c, isz * 0.5 * pop, ok, 0.0, Color::TRANSPARENT);
                            cx.icon("check", c, isz * 0.7 * pop, Color::WHITE);
                        } else if is_cur {
                            cx.painter.circle(c, isz * 0.5, Color::TRANSPARENT, 1.5, accent);
                            cx.painter.circle(c, isz * 0.18, accent, 0.0, Color::TRANSPARENT);
                        } else {
                            cx.painter.circle(c, isz * 0.5, Color::TRANSPARENT, 1.5, muted);
                        }
                    })));
                    ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 4.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                    let st = ui.text_style(if is_cur { TextRole::BodyStrong } else { TextRole::Body });
                    ui.text_leaf(s.title.clone(), st, if done { muted } else { fg }, true, false, Style { width: Size::FILL, ..Default::default() });
                    if is_cur {
                        let stc = ui.text_style(TextRole::Caption);
                        ui.text_leaf(s.detail.clone(), stc, sec, true, false, Style { width: Size::FILL, ..Default::default() });
                        if !s.action.is_empty() && ui.scope(("act", i), |ui| ui.button(s.action.as_str()).primary().small().show().clicked) {
                            out.action = Some(i);
                        }
                    }
                    ui.end();
                    ui.end();
                    let state = if done { "Done" } else if is_cur { "Next" } else { "To do" };
                    let t = s.title.clone();
                    ui.a11y(sid, || A11y::new(Role::ListItem).label(t).value(state).position(i, n));
                }
                if all && ui.scope(id.with("dismiss"), |ui| ui.button("Dismiss").small().show().clicked) {
                    out.dismissed = true;
                }
                ui.end();
            });
        }
        // The launcher pill.
        let lid = id.with("launcher");
        let lix = self.interact(lid, Sense::CLICK, false);
        let (act, held) = super::key_activate(self, &lix, true);
        if lix.clicked || act {
            *open = !*open;
        }
        let (hov, press) = self.state_weights(&lix, held);
        let squash = if reduced { 1.0 } else { self.spring(lid, 0, if lix.pressed || held { 0.95 } else { 1.0 }, mo.bouncy) };
        let (fa, fgp) = self.focus_anim(&lix);
        let pill_bg = self.color(Token::SurfaceRaised).over(self.color(Token::Hover).multiply_alpha(hov * 0.7 + press * 0.3));
        let pill_bd = self.color(Token::Border);
        let shadows = crate::theme::Elevation::Level1.shadows(self.theme.darkness(), self.color(Token::ShadowInk));
        let st = self.text_style(TextRole::BodyStrong);
        let stc = self.text_style(TextRole::Caption);
        let label = if all { "All set".to_owned() } else { title.to_owned() };
        let count = format!("{done_n}/{n}");
        let g = self.galley(&label, &st);
        let gc = self.galley(&count, &stc);
        let h = lk.control.height[1];
        let w = h * 0.7 + gap + g.width + gap + gc.width + gap * 2.0;
        let col = if all { ok } else { accent };
        self.leaf(Style::default(), vec2(w, h), Some(lid), Sense::CLICK, CursorIcon::Pointer, Some(Box::new(move |cx, r| {
            let rr = Rect::from_center_size(r.center(), r.size() * squash);
            cx.painter.shadows(rr, Corners::all(rr.height() * 0.5), &shadows);
            cx.painter.rrect(rr, Corners::all(rr.height() * 0.5), pill_bg, 1.0, pill_bd);
            let rs = rr.height() * 0.35;
            let c = vec2(rr.min.x + gap + rs, rr.center().y);
            ring(cx, c, rs, rs * 0.3, prog, col);
            let x = c.x + rs + gap;
            let gg = cx.galley(&label, &st);
            cx.painter.text(cx.text, &gg, vec2(x, (rr.center().y - gg.height * 0.5).round()), fg);
            let g2 = cx.galley(&count, &stc);
            cx.painter.text(cx.text, &g2, vec2(x + gg.width + gap, (rr.center().y - g2.height * 0.5).round()), sec);
            if fa > 0.0 {
                cx.defer(r, Box::new(move |cx, r2| cx.focus_ring(r2, r2.height() * 0.5, fa, fgp, ring_c)));
            }
        })));
        let o = *open;
        let t = title.to_owned();
        self.a11y(lid, || A11y::new(Role::Button).label(t).value(format!("{done_n} of {n} done")).expanded(o));
        self.end();
        out
    }

    /// A row you swipe sideways to reveal actions: drag it (or scroll
    /// sideways on a trackpad) and it snaps open; swipe far enough and the
    /// outermost action fires. Arrow keys open it from the keyboard.
    /// `label` names the row for screen readers.
    pub fn swipe_row(&mut self, key: impl std::hash::Hash, label: &str, leading: &[SwipeAction], trailing: &[SwipeAction], content: impl FnOnce(&mut Ui)) -> Option<SwipeHit> {
        let id = self.make_id(("swipe", key));
        let lk = self.look().clone();
        let mo = self.motion();
        let reduced = self.reduced_motion();
        let now = self.time();
        let ix = self.interact(id, Sense { click: true, drag: true, focusable: true, scroll: true }, false);
        let w = ix.rect.width().max(1.0);
        let act_w = lk.control.height[1] * 2.0;
        let max_lead = act_w * leading.len() as f32;
        let max_trail = act_w * trailing.len() as f32;
        let full = w * 0.6;
        let mut hit = None;
        let resolved = {
            let m = self.mem::<SwipeMem>(id);
            if m.open.is_infinite() {
                m.open = if m.open > 0.0 { max_lead } else { -max_trail };
                m.live = m.open;
                Some(m.open)
            } else {
                None
            }
        };
        if let Some(o) = resolved {
            self.anims.set_spring(id, 0, o);
        }
        let clamp = |x: f32| -> f32 {
            // Past the actions it rubber-bands, up to the full-swipe point.
            let rb = |over: f32| over * 0.5;
            if x > 0.0 {
                if leading.is_empty() { rb(x).min(act_w * 0.3) } else if x > max_lead { max_lead + rb(x - max_lead) } else { x }
            } else if trailing.is_empty() {
                -rb(-x).min(act_w * 0.3)
            } else if -x > max_trail {
                -(max_trail + rb(-x - max_trail))
            } else {
                x
            }
        };
        let fire = |off: f32| -> Option<SwipeHit> {
            if off > full.min(max_lead + act_w) && !leading.is_empty() {
                Some(SwipeHit { trailing: false, index: 0 })
            } else if -off > full.min(max_trail + act_w) && !trailing.is_empty() {
                Some(SwipeHit { trailing: true, index: trailing.len() - 1 })
            } else {
                None
            }
        };
        {
            let m = self.mem::<SwipeMem>(id);
            if ix.press_started {
                if let Some(p) = ix.pointer {
                    m.drag = Some((p.x, m.open));
                    m.live = m.open;
                }
            }
            if let (true, Some((x0, o0)), Some(p)) = (ix.held, m.drag, ix.pointer) {
                m.live = clamp(o0 + p.x - x0);
            } else if let Some((x0, o0)) = m.drag.take() {
                let moved = ix.pointer.map_or(0.0, |p| (p.x - x0).abs()) > 4.0;
                if moved {
                    if let Some(h) = fire(m.live) {
                        hit = Some(h);
                        m.fired = Some((h, now));
                        m.open = 0.0;
                    } else if m.live > max_lead * 0.5 && max_lead > 0.0 {
                        m.open = max_lead;
                    } else if -m.live > max_trail * 0.5 && max_trail > 0.0 {
                        m.open = -max_trail;
                    } else {
                        m.open = 0.0;
                    }
                } else if o0 != 0.0 {
                    // A tap while open: on an action fires it, else closes.
                    if let Some(p) = ix.pointer {
                        let x = p.x - ix.rect.min.x;
                        if o0 > 0.0 && x < o0 {
                            hit = Some(SwipeHit { trailing: false, index: ((x / act_w) as usize).min(leading.len().saturating_sub(1)) });
                        } else if o0 < 0.0 && x > w + o0 {
                            hit = Some(SwipeHit { trailing: true, index: (((x - (w + o0)) / act_w) as usize).min(trailing.len().saturating_sub(1)) });
                        }
                    }
                    m.open = 0.0;
                }
                m.live = m.open;
            }
        }
        // Trackpad: sideways scroll moves it; it snaps once the scroll stops.
        let sc = self.take_scroll(id);
        if sc.x != 0.0 && sc.x.abs() > sc.y.abs() {
            let m = self.mem::<SwipeMem>(id);
            m.live = clamp(m.live - sc.x);
            m.scroll_at = now;
        }
        {
            let m = self.mem::<SwipeMem>(id);
            if m.scroll_at > 0.0 && m.drag.is_none() {
                if now - m.scroll_at > 0.15 {
                    m.scroll_at = 0.0;
                    if let Some(h) = fire(m.live) {
                        hit = Some(h);
                        m.fired = Some((h, now));
                        m.open = 0.0;
                    } else {
                        m.open = if m.live > max_lead * 0.5 && max_lead > 0.0 { max_lead } else if -m.live > max_trail * 0.5 && max_trail > 0.0 { -max_trail } else { 0.0 };
                    }
                    m.live = m.open;
                } else {
                    self.anims.keep_alive();
                }
            }
        }
        if ix.focused {
            for (k, _, _) in self.keys_for(id) {
                let m = self.mem::<SwipeMem>(id);
                match k {
                    Key::ArrowLeft if m.open > 0.0 => m.open = 0.0,
                    Key::ArrowLeft if max_trail > 0.0 => m.open = -max_trail,
                    Key::ArrowRight if m.open < 0.0 => m.open = 0.0,
                    Key::ArrowRight if max_lead > 0.0 => m.open = max_lead,
                    Key::Escape => m.open = 0.0,
                    Key::Enter if m.open > 0.0 => {
                        hit = Some(SwipeHit { trailing: false, index: 0 });
                        m.open = 0.0;
                    }
                    Key::Enter if m.open < 0.0 => {
                        hit = Some(SwipeHit { trailing: true, index: trailing.len() - 1 });
                        m.open = 0.0;
                    }
                    _ => continue,
                }
                m.live = m.open;
            }
        }
        let (live, dragging, open, fired, content_h) = {
            let m = self.mem::<SwipeMem>(id);
            if m.fired.is_some_and(|(_, t)| now - t > 0.6) {
                m.fired = None;
            }
            (m.live, m.drag.is_some() || m.scroll_at > 0.0, m.open, m.fired, m.content_h)
        };
        // Offset shown: the finger while dragging, else a spring; a fired
        // row flies off its side first, then springs back.
        let shown = if dragging {
            self.anims.set_spring(id, 0, live);
            live
        } else if reduced {
            open
        } else if let Some((h, t0)) = fired {
            let t = ((now - t0) as f32 / 0.25).min(1.0);
            self.anims.keep_alive();
            if t < 1.0 {
                let fly = if h.trailing { -w } else { w };
                self.anims.set_spring(id, 0, fly * Ease::Out.apply(t));
                fly * Ease::Out.apply(t)
            } else {
                self.spring(id, 0, 0.0, mo.smooth)
            }
        } else {
            self.spring(id, 0, open, mo.smooth)
        };
        let (ra, rg) = self.focus_anim(&ix);
        let ring = self.color(Token::FocusRing);
        let lead_cols: Vec<Color> = leading.iter().map(|a| self.color(tone_token(a.tone))).collect();
        let trail_cols: Vec<Color> = trailing.iter().map(|a| self.color(tone_token(a.tone))).collect();
        let lead: Vec<(String, String)> = leading.iter().map(|a| (a.label.clone(), a.icon.clone())).collect();
        let trail: Vec<(String, String)> = trailing.iter().map(|a| (a.label.clone(), a.icon.clone())).collect();
        let st = self.text_style(TextRole::Caption);
        let bg = self.color(Token::Surface);
        let rad = lk.control.radius[1];

        let node = self.begin(Style { dir: Dir::Column, width: Size::FILL, ..Default::default() }, Some(id), Sense { click: true, drag: true, focusable: true, scroll: true }, CursorIcon::Default, true, None);
        self.set_paint(node, Box::new(move |cx, r| {
            // Actions behind the row; each grows into its share of the gap.
            let draw = |cx: &mut crate::ui::PaintCx, acts: &[(String, String)], cols: &[Color], from_left: bool, span: f32| {
                if span <= 0.5 || acts.is_empty() {
                    return;
                }
                let n = acts.len() as f32;
                let full_one = span > act_w * n + act_w * 0.5;
                for (k, (l, icon)) in acts.iter().enumerate() {
                    let share = if full_one { if (from_left && k == 0) || (!from_left && k + 1 == acts.len()) { span } else { 0.0 } } else { span / n };
                    if share <= 0.0 {
                        continue;
                    }
                    let x = if full_one {
                        if from_left { r.min.x } else { r.max.x - span }
                    } else if from_left {
                        r.min.x + share * k as f32
                    } else {
                        r.max.x - span + share * k as f32
                    };
                    let rr = Rect::from_xywh(x, r.min.y, share + 0.5, r.height());
                    cx.painter.rect(rr, cols[k]);
                    let reveal = (share / act_w).clamp(0.0, 1.0);
                    let c = if full_one { vec2(if from_left { r.min.x + act_w * 0.5 } else { r.max.x - act_w * 0.5 }, rr.center().y) } else { rr.center() };
                    let isz = st.size + 6.0;
                    cx.icon(icon, c - vec2(0.0, st.size * 0.6), isz * (0.6 + 0.4 * reveal), Color::WHITE.multiply_alpha(reveal));
                    let g = cx.galley(l, &st);
                    cx.painter.text(cx.text, &g, vec2((c.x - g.width * 0.5).round(), c.y + st.size * 0.3), Color::WHITE.multiply_alpha(reveal));
                }
            };
            if shown > 0.0 {
                draw(cx, &lead, &lead_cols, true, shown);
            } else if shown < 0.0 {
                draw(cx, &trail, &trail_cols, false, -shown);
            }
            if ra > 0.0 {
                cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr, rad, ra, rg, ring)));
            }
        }));
        let content_id = id.with("content");
        self.column().fill_width().layer(1.0, vec2(shown, 0.0)).show(|ui| {
            ui.begin(Style { dir: Dir::Column, width: Size::FILL, ..Default::default() }, Some(content_id), Sense::NONE, CursorIcon::Default, false, Some(Box::new(move |cx, r| cx.painter.rect(r, bg))));
            content(ui);
            ui.end();
        });
        let ch = self.rect_of(content_id).map(|r| r.height()).unwrap_or(0.0);
        self.mem::<SwipeMem>(id).content_h = ch;
        // While open, a cover takes the taps (they close it or hit an action).
        if (open != 0.0 || dragging) && content_h > 0.0 {
            self.leaf(Style { width: Size::FILL, height: Size::Fixed(content_h), margin_top: -content_h, ..Default::default() }, vec2(1.0, content_h), Some(id.with("cover")), Sense::NONE, CursorIcon::Grab, None);
        }
        // Screen readers get the actions as buttons.
        for (k, a) in leading.iter().enumerate().map(|(k, a)| (k, (a, false))).chain(trailing.iter().enumerate().map(|(k, a)| (k, (a, true)))) {
            let aid = id.with(("a11y_act", a.1, k));
            self.leaf(Style { width: Size::Fixed(1.0), height: Size::Fixed(0.0), ..Default::default() }, vec2(1.0, 0.0), Some(aid), Sense::NONE, CursorIcon::Default, None);
            if self.clicked_id(aid) {
                hit = Some(SwipeHit { trailing: a.1, index: k });
            }
            let l = a.0.label.clone();
            self.a11y(aid, || A11y::new(Role::Button).label(l));
        }
        self.end();
        let l = label.to_owned();
        let state = if open > 0.0 { "Leading actions shown" } else if open < 0.0 { "Trailing actions shown" } else { "" };
        self.a11y(id, || A11y::new(Role::Group).label(l).description(format!("Swipe or use the arrow keys for actions. {state}")));
        hit
    }

    /// Open (or close, with `None`) the swipe row `key` in the current
    /// scope, as if it had been swiped: `Some(true)` shows the trailing
    /// actions. Call it before the row.
    pub fn swipe_row_reveal(&mut self, key: impl std::hash::Hash, trailing: Option<bool>) {
        let id = self.peek_id(("swipe", key));
        let m = self.mem::<SwipeMem>(id);
        m.open = match trailing {
            Some(true) => f32::NEG_INFINITY,
            Some(false) => f32::INFINITY,
            None => 0.0,
        };
        m.live = m.open;
    }

    /// Panels docked left, right, bottom and in the middle, each zone with
    /// tabs. Drag a tab to another zone (a preview shows where it lands),
    /// drag the splitters to resize, or move the focused tab with
    /// Cmd/Ctrl+Shift+arrows. `content` draws a panel's body by id.
    pub fn dock_area(&mut self, key: impl std::hash::Hash, panels: &[DockPanel], layout: &mut DockLayout, height: f32, mut content: impl FnMut(&mut Ui, &str)) -> Response {
        let id = self.make_id(("dock", key));
        let lk = self.look().clone();
        let mo = self.motion();
        let reduced = self.reduced_motion();
        let gap = lk.control.gap[1];
        let tab_h = lk.control.height[0];
        let split = (lk.control.gap[0] * 1.5).max(5.0);
        let area = self.rect_of(id).unwrap_or(Rect::ZERO);
        let before = layout.clone();
        let title_of = |pid: &str| panels.iter().find(|p| p.id == pid).map(|p| (p.title.clone(), p.icon.clone())).unwrap_or((pid.to_owned(), "panel-top".into()));
        // Sizes stay sane on small areas.
        if area.width() > 0.0 {
            // The middle keeps at least 40% of the width; the side docks
            // share the rest.
            let sides = [DockZone::Left, DockZone::Right].iter().filter(|z| !layout.zones[z.ix()].is_empty()).count().max(1) as f32;
            let max_side = (area.width() * 0.6 / sides).max(60.0);
            let min_side = max_side.min(100.0);
            layout.left = layout.left.clamp(min_side, max_side);
            layout.right = layout.right.clamp(min_side, max_side);
            layout.bottom = layout.bottom.clamp(60.0, (height * 0.6).max(80.0));
        }
        let has = |z: DockZone, l: &DockLayout| !l.zones[z.ix()].is_empty();
        // Zone sizes spring when a zone appears or empties.
        let sz = |ui: &mut Ui, slot: u32, on: bool, v: f32| if reduced { if on { v } else { 0.0 } } else { ui.spring(id, slot, if on { v } else { 0.0 }, mo.smooth).max(0.0) };
        let lw = sz(self, 0, has(DockZone::Left, layout), layout.left);
        let rw = sz(self, 1, has(DockZone::Right, layout), layout.right);
        let bh = sz(self, 2, has(DockZone::Bottom, layout), layout.bottom);

        // Drag in progress: which zone is under the pointer.
        let ptr = self.pointer();
        let drag = self.mem::<DockMem>(id).drag.clone();
        let target = match (&drag, ptr) {
            (Some((_, _, true)), Some(p)) if area.contains(p) => {
                let u = vec2((p.x - area.min.x) / area.width(), (p.y - area.min.y) / area.height());
                Some(if u.y > 0.72 { DockZone::Bottom } else if u.x < 0.22 { DockZone::Left } else if u.x > 0.78 { DockZone::Right } else { DockZone::Center })
            }
            _ => None,
        };
        self.mem::<DockMem>(id).target = target;
        let preview_for = |z: DockZone| -> Rect {
            let top_h = area.height() - if has(DockZone::Bottom, layout) { bh + split } else { 0.0 };
            match z {
                DockZone::Left => Rect::from_xywh(area.min.x, area.min.y, layout.left.max(area.width() * 0.22), top_h),
                DockZone::Right => {
                    let w = layout.right.max(area.width() * 0.22);
                    Rect::from_xywh(area.max.x - w, area.min.y, w, top_h)
                }
                DockZone::Bottom => {
                    let h = layout.bottom.max(area.height() * 0.28);
                    Rect::from_xywh(area.min.x, area.max.y - h, area.width(), h)
                }
                DockZone::Center => {
                    let x0 = area.min.x + if has(DockZone::Left, layout) { lw + split } else { 0.0 };
                    let x1 = area.max.x - if has(DockZone::Right, layout) { rw + split } else { 0.0 };
                    Rect::from_xywh(x0, area.min.y, (x1 - x0).max(40.0), top_h)
                }
            }
        };
        let preview = target.map(preview_for);

        let bg = self.color(Token::Surface);
        let edge = self.color(Token::BorderSubtle);
        let rad = lk.card.radius;
        let accent = self.color(Token::Accent);
        let acc_sub = self.color(Token::AccentSubtle);
        let pv_key = id.with("preview");
        let pv_on = self.fade(id, 10, preview.is_some(), mo.hover_in, mo.hover_out, Ease::Linear);
        let ghost = drag.as_ref().filter(|d| d.2).map(|d| title_of(&d.0));
        let ghost_bg = self.color(Token::SurfaceRaised);
        let ghost_bd = self.color(Token::Border);
        let st_tab = self.text_style(TextRole::Label);
        let fg = self.color(Token::Foreground);
        let shadows = crate::theme::Elevation::Level2.shadows(self.theme.darkness(), self.color(Token::ShadowInk));
        let ghost_r = lk.control.radius[1];
        let pv_r = lk.control.radius[2];

        let root = self.begin(Style { dir: Dir::Column, width: Size::FILL, height: Size::Fixed(height), ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, true, Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(rad), bg, 0.0, Color::TRANSPARENT))));
        self.set_paint_after(root, Box::new(move |cx, r| {
            cx.painter.rrect(r, Corners::all(rad), Color::TRANSPARENT, 1.0, edge);
            // Where the dragged panel will land, springing between zones.
            if let Some(p) = preview {
                let first = cx.anims.spring_state(pv_key, 0).is_none();
                let sp = mo.snappy;
                let mut v = [p.min.x, p.min.y, p.max.x, p.max.y];
                if !reduced && !first {
                    for (k, x) in v.iter_mut().enumerate() {
                        *x = cx.anims.spring_from(pv_key, k as u32, *x, *x, sp);
                    }
                } else {
                    for (k, x) in v.iter().enumerate() {
                        cx.anims.spring_from(pv_key, k as u32, *x, *x, sp);
                    }
                }
                let pr = Rect { min: vec2(v[0], v[1]), max: vec2(v[2], v[3]) }.expand(-gap * 0.5);
                cx.painter.rrect_unsnapped(pr, Corners::all(pv_r), [acc_sub.multiply_alpha(0.85 * pv_on); 2], 2.0, accent.multiply_alpha(pv_on));
            }
            if let (Some((t, icon)), Some(p)) = (&ghost, ptr) {
                let g = cx.galley(t, &st_tab);
                let isz = st_tab.size + 3.0;
                let gr = Rect::from_xywh(p.x + gap, p.y + gap, g.width + isz + gap * 3.0, tab_h);
                cx.painter.shadows(gr, Corners::all(ghost_r), &shadows);
                cx.painter.rrect(gr, Corners::all(ghost_r), ghost_bg, 1.0, ghost_bd);
                cx.icon(icon, vec2(gr.min.x + gap + isz * 0.5, gr.center().y), isz, accent);
                cx.painter.text(cx.text, &g, vec2(gr.min.x + gap * 2.0 + isz, (gr.center().y - g.height * 0.5).round()), fg);
            }
        }));
        let top_h = height - if has(DockZone::Bottom, layout) { bh + split } else { 0.0 };
        self.begin(Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(top_h.max(0.0)), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let mut moves: Vec<(String, DockZone)> = vec![];
        for z in [DockZone::Left, DockZone::Center, DockZone::Right] {
            let present = has(z, layout) || z == DockZone::Center;
            let w = match z {
                DockZone::Left => lw,
                DockZone::Right => rw,
                _ => 0.0,
            };
            if z == DockZone::Right && rw > 0.5 {
                splitter(self, id, 1, false, split, &mut layout.right, -1.0);
            }
            if present && (z == DockZone::Center || w > 0.5) {
                let style = if z == DockZone::Center { Size::FILL } else { Size::Fixed(w) };
                dock_zone(self, id, z, style, panels, layout, &mut moves, &mut content);
            }
            if z == DockZone::Left && lw > 0.5 {
                splitter(self, id, 0, false, split, &mut layout.left, 1.0);
            }
        }
        self.end();
        if bh > 0.5 {
            splitter(self, id, 2, true, split, &mut layout.bottom, -1.0);
            dock_zone(self, id, DockZone::Bottom, Size::FILL, panels, layout, &mut moves, &mut content);
        }
        self.end();

        // Finish a drag: drop on the target zone.
        let held = self.pointer_down();
        {
            let m = self.mem::<DockMem>(id);
            if let Some((pid, p0, moved)) = m.drag.clone() {
                if !held {
                    if moved {
                        if let Some(z) = m.target {
                            moves.push((pid, z));
                        }
                    }
                    m.drag = None;
                } else if !moved && ptr.is_some_and(|p| (p - p0).length() > 6.0) {
                    m.drag = Some((pid, p0, true));
                }
            }
        }
        for (pid, z) in moves {
            layout.move_to(&pid, z);
        }
        Response { id, rect: area, changed: *layout != before, ..Default::default() }
    }
}

/// A splitter between dock zones: drag to resize `size` (`sign` +1 when
/// moving right/down grows it).
fn splitter(ui: &mut Ui, id: crate::Id, k: usize, horizontal: bool, thick: f32, size: &mut f32, sign: f32) {
    let sid = id.with(("split", k));
    let six = ui.interact(sid, Sense::DRAG, false);
    let mo = ui.motion();
    let p = ui.pointer();
    {
        let m = ui.mem::<DockMem>(id);
        if six.press_started {
            if let Some(p) = p {
                m.resize = Some((k, if horizontal { p.y } else { p.x }, *size));
            }
        }
        if let (true, Some((kk, p0, s0)), Some(p)) = (six.held, m.resize, p) {
            if kk == k {
                let d = if horizontal { p.y - p0 } else { p.x - p0 };
                *size = (s0 + d * sign).max(60.0);
            }
        } else if !six.held && m.resize.is_some_and(|r| r.0 == k) {
            m.resize = None;
        }
    }
    let hot = ui.fade(sid, 0, six.hovered || six.held, mo.hover_in, mo.hover_out, Ease::Linear);
    let line = ui.color(Token::BorderSubtle).mix(ui.color(Token::Accent), hot);
    let style = if horizontal { Style { width: Size::FILL, height: Size::Fixed(thick), ..Default::default() } } else { Style { width: Size::Fixed(thick), height: Size::FILL, ..Default::default() } };
    ui.leaf(style, vec2(thick, thick), Some(sid), Sense::DRAG, if horizontal { CursorIcon::ResizeVertical } else { CursorIcon::ResizeHorizontal }, Some(Box::new(move |cx, r| {
        let w = 1.0 + hot;
        let lr = if horizontal { Rect::from_xywh(r.min.x, r.center().y - w * 0.5, r.width(), w) } else { Rect::from_xywh(r.center().x - w * 0.5, r.min.y, w, r.height()) };
        cx.painter.rect(lr, line);
    })));
    let v = *size as f64;
    ui.a11y(sid, || A11y::new(Role::Separator).label(if horizontal { "Resize bottom dock" } else { "Resize side dock" }).numeric(v, 60.0, 2000.0, Some(10.0)));
}

#[allow(clippy::too_many_arguments)]
fn dock_zone(ui: &mut Ui, id: crate::Id, z: DockZone, width: Size, panels: &[DockPanel], layout: &mut DockLayout, moves: &mut Vec<(String, DockZone)>, content: &mut impl FnMut(&mut Ui, &str)) {
    let lk = ui.look().clone();
    let mo = ui.motion();
    let reduced = ui.reduced_motion();
    let gap = lk.control.gap[1];
    let tab_h = lk.control.height[0];
    let zid = id.with(("zone", z.ix()));
    let tabs = layout.zones[z.ix()].clone();
    let active = layout.active[z.ix()].min(tabs.len().saturating_sub(1));
    let strip_bg = ui.color(Token::SurfaceMuted);
    let edge = ui.color(Token::BorderSubtle);
    let accent = ui.color(Token::Accent);
    let fg = ui.color(Token::Foreground);
    let sec = ui.color(Token::TextSecondary);
    let ring = ui.color(Token::FocusRing);
    let st = ui.text_style(TextRole::Label);
    let tab_ids: Vec<crate::Id> = tabs.iter().map(|t| id.with(("tab", t))).collect();
    let sel_id = tab_ids.get(active).copied();
    let bar_key = zid.with("bar");
    let dragging = ui.mem::<DockMem>(id).drag.as_ref().filter(|d| d.2).map(|d| d.0.clone());

    ui.begin(Style { dir: Dir::Column, width, height: Size::FILL, ..Default::default() }, Some(zid), Sense::NONE, CursorIcon::Default, true, None);
    let strip = ui.begin(Style { dir: Dir::Row, width: Size::FILL, height: Size::Fixed(tab_h), pad: Sides::xy(gap * 0.5, 0.0), gap: 2.0, align: Align::Center, scroll_x: true, ..Default::default() }, Some(zid.with("strip")), Sense::NONE, CursorIcon::Default, true, None);
    ui.set_paint(strip, Box::new(move |cx, r| {
        cx.painter.rect(r, strip_bg);
        cx.painter.rect(Rect { min: vec2(r.min.x, r.max.y - 1.0), max: r.max }, edge);
        // The active tab's underline glides.
        if let Some(s) = sel_id.and_then(|i| cx.rects.get(&i).copied()) {
            let (x, w) = if reduced {
                (s.min.x, s.width())
            } else {
                let first = cx.anims.spring_state(bar_key, 0).is_none();
                let x = cx.anims.spring_from(bar_key, 0, s.min.x, s.min.x, mo.snappy);
                let w = cx.anims.spring_from(bar_key, 1, s.width(), s.width(), mo.snappy);
                if first { (s.min.x, s.width()) } else { (x, w) }
            };
            cx.painter.rect(Rect::from_xywh(x + 6.0, r.max.y - 2.0, (w - 12.0).max(0.0), 2.0), accent);
        }
    }));
    for (k, pid) in tabs.iter().enumerate() {
        let tid = tab_ids[k];
        let tix = ui.interact(tid, Sense { click: true, drag: true, focusable: true, scroll: false }, false);
        if tix.press_started {
            if let Some(p) = tix.pointer {
                ui.mem::<DockMem>(id).drag = Some((pid.clone(), p, false));
            }
            layout.active[z.ix()] = k;
        }
        if tix.focused {
            for (key, mods, _) in ui.keys_for(tid) {
                if mods.command() && mods.shift {
                    let to = match key {
                        Key::ArrowLeft => Some(DockZone::Left),
                        Key::ArrowRight => Some(DockZone::Right),
                        Key::ArrowDown => Some(DockZone::Bottom),
                        Key::ArrowUp => Some(DockZone::Center),
                        _ => None,
                    };
                    if let Some(to) = to {
                        moves.push((pid.clone(), to));
                    }
                } else {
                    let j = match key {
                        Key::ArrowRight => Some((k + 1).min(tabs.len() - 1)),
                        Key::ArrowLeft => Some(k.saturating_sub(1)),
                        _ => None,
                    };
                    if let Some(j) = j {
                        layout.active[z.ix()] = j;
                        ui.request_focus(tab_ids[j]);
                    }
                }
            }
        }
        let on = k == active;
        let on_t = ui.fade(tid, 0, on, mo.selected, mo.selected, Ease::Out);
        let (hov, _) = ui.state_weights(&tix, false);
        let (fa, fgp) = ui.focus_anim(&tix);
        let lifted = dragging.as_deref() == Some(pid.as_str());
        let alpha = if lifted { 0.4 } else { 1.0 };
        let hover_c = ui.color(Token::Hover).multiply_alpha(hov * 0.8);
        let tc = sec.mix(fg, on_t.max(hov)).multiply_alpha(alpha);
        let (title, icon) = panels.iter().find(|p| p.id == *pid).map(|p| (p.title.clone(), p.icon.clone())).unwrap_or((pid.clone(), "panel-top".into()));
        let ic = sec.mix(accent, on_t).multiply_alpha(alpha);
        let rr = lk.control.radius[0];
        ui.begin(Style { dir: Dir::Row, height: Size::Fixed(tab_h - 6.0), pad: Sides::xy(gap * 0.75, 0.0), gap: gap * 0.5, align: Align::Center, justify: Justify::Center, ..Default::default() }, Some(tid), Sense { click: true, drag: true, focusable: true, scroll: false }, if lifted { CursorIcon::Grabbing } else { CursorIcon::Pointer }, false, Some(Box::new(move |cx, r| {
            cx.painter.rrect(r, Corners::all(rr), hover_c, 0.0, Color::TRANSPARENT);
            if fa > 0.0 {
                cx.defer(r, Box::new(move |cx, r2| cx.focus_ring(r2, rr, fa, fgp, ring)));
            }
        })));
        ui.icon_leaf(&icon, st.size + 2.0, ic);
        ui.text_leaf(title.clone(), st, tc, false, false, Style::default());
        ui.end();
        let n = tabs.len();
        let zn = z.name();
        ui.a11y(tid, || A11y::new(Role::Tab).label(title).selected(on).position(k, n).description(format!("In the {}. Command Shift arrows move it.", zn.to_lowercase())));
    }
    ui.end();
    let zn = z.name();
    ui.a11y(zid.with("strip"), || A11y::new(Role::TabList).label(zn));
    // Body, fading in when the tab changes.
    let body_id = zid.with("body");
    let shown_tab = tabs.get(active).cloned();
    let fresh = ui.anims.spring(body_id.with(shown_tab.as_deref().unwrap_or("")), 0, 1.0, mo.smooth);
    let _ = fresh;
    ui.begin(Style { dir: Dir::Column, width: Size::FILL, height: Size::FILL, pad: Sides::all(gap), gap, ..Default::default() }, Some(body_id), Sense::NONE, CursorIcon::Default, true, None);
    match shown_tab {
        Some(pid) => ui.scope(("panel", &pid), |ui| content(ui, &pid)),
        None => {
            ui.begin(Style { width: Size::FILL, height: Size::FILL, align: Align::Center, justify: Justify::Center, gap: 6.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            let ic = ui.color(Token::TextMuted);
            ui.icon_leaf("layout-panel-left", 22.0, ic);
            let stc = ui.text_style(TextRole::Caption);
            ui.text_leaf("Drop a panel here".to_owned(), stc, sec, false, false, Style::default());
            ui.end();
        }
    }
    ui.end();
    ui.end();
}
