//! Navigation chrome, carousels, callouts, undo bars and media frames:
//! [`EditorTab`], [`TocItem`], [`NavBarItem`], [`Carousel`], [`Callout`],
//! [`UndoBar`], and [`ImageFrame`].

use super::toast::Tone;
use crate::a11y::{A11y, Role};
use crate::anim::Ease;
use crate::input::{CursorIcon, Key};
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Rect, Sides};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

// =============================================================== EditorTabs

/// Closable tab descriptor for [`Ui::editor_tabs`].
#[derive(Clone, Debug)]
pub struct EditorTab {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub dirty: bool,
}

impl EditorTab {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        EditorTab { id: id.into(), label: label.into(), icon: None, dirty: false }
    }
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = Some(name.to_owned());
        self
    }
    pub fn dirty(mut self, d: bool) -> Self {
        self.dirty = d;
        self
    }
}

/// Output from [`Ui::editor_tabs`].
#[derive(Clone, Debug, Default)]
pub struct EditorTabsOut {
    pub changed: bool,
    pub closed: Option<usize>,
    pub add_clicked: bool,
}

#[derive(Default, Clone, Copy)]
struct TabSlide {
    /// Pill position in scrolled-content coordinates.
    x: f32,
    w: f32,
    ready: bool,
    /// The scroll offset the last frame laid out with.
    last_off: f32,
    /// The active tab last scrolled into view.
    shown: Option<usize>,
}

// =========================================================== TableOfContents

/// Entry in [`Ui::table_of_contents`].
#[derive(Clone, Debug)]
pub struct TocItem {
    pub label: String,
    pub depth: u8,
}

impl TocItem {
    pub fn new(label: impl Into<String>, depth: u8) -> Self {
        TocItem { label: label.into(), depth }
    }
}

#[derive(Default, Clone, Copy)]
struct TocSlide {
    y: f32,
    h: f32,
    ready: bool,
}

// =================================================================== NavBar

/// Item in [`Ui::nav_bar`].
#[derive(Clone, Debug)]
pub struct NavBarItem {
    pub label: String,
    pub icon: Option<String>,
    pub badge: Option<String>,
}

impl NavBarItem {
    pub fn new(label: impl Into<String>) -> Self {
        NavBarItem { label: label.into(), icon: None, badge: None }
    }
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = Some(name.to_owned());
        self
    }
    pub fn badge(mut self, b: impl Into<String>) -> Self {
        self.badge = Some(b.into());
        self
    }
}

// ================================================================= Carousel

/// Slide in [`Carousel`].
#[derive(Clone, Debug)]
pub struct CarouselSlide {
    pub tag: String,
    pub title: String,
    pub body: String,
    pub icon: String,
    pub tone: Tone,
}

impl CarouselSlide {
    pub fn new(tag: impl Into<String>, title: impl Into<String>, body: impl Into<String>, icon: &str, tone: Tone) -> Self {
        CarouselSlide {
            tag: tag.into(),
            title: title.into(),
            body: body.into(),
            icon: icon.to_owned(),
            tone,
        }
    }
}

/// Horizontal spring-sliding feature/card carousel with prev/next buttons,
/// stretching pill dots, and `ArrowLeft` / `ArrowRight` support.
pub struct Carousel<'u> {
    ui: &'u mut Ui,
    key: String,
    active: &'u mut usize,
    slides: &'u [CarouselSlide],
    height: f32,
    shown: bool,
}

impl<'u> Carousel<'u> {
    pub fn height(mut self, px: f32) -> Self {
        self.height = px.max(150.0);
        self
    }
    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("carousel", &self.key));
        let n = self.slides.len().max(1);
        *self.active = (*self.active).min(n - 1);

        let stage_id = id.with("stage");
        let ix = ui.interact(stage_id, Sense::CLICK, false);
        let mo = ui.motion();
        let mut changed = false;

        if ix.focused {
            if ui.key_pressed_for(stage_id, Key::ArrowRight) {
                *self.active = (*self.active + 1) % n;
                changed = true;
            } else if ui.key_pressed_for(stage_id, Key::ArrowLeft) {
                *self.active = (*self.active + n - 1) % n;
                changed = true;
            }
        }

        let pos = ui.spring(id, 0, *self.active as f32, mo.smooth);
        let r_card = ui.look().card.radius;
        let bg_c = ui.color(Token::Surface);
        let bd_c = ui.color(Token::BorderSubtle);

        ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 12.0, ..Default::default() }, Some(id), Sense::NONE, CursorIcon::Default, false, None);

        // Slide viewport painted directly so adjacent slides glide horizontally across the frame!
        let slides_vec = self.slides.to_vec();
        let pal: Vec<(Color, Color)> = slides_vec
            .iter()
            .map(|s| match s.tone {
                Tone::Neutral => (ui.color(Token::SurfaceMuted), ui.color(Token::Foreground)),
                Tone::Accent | Tone::Info => (ui.color(Token::AccentSubtle), ui.color(Token::AccentStrong)),
                Tone::Success => (ui.color(Token::SuccessSubtle), ui.color(Token::SuccessStrong)),
                Tone::Warning => (ui.color(Token::WarningSubtle), ui.color(Token::WarningStrong)),
                Tone::Danger => (ui.color(Token::DangerSubtle), ui.color(Token::DangerStrong)),
            })
            .collect();
        let fg_c = ui.color(Token::Foreground);
        let sec_c = ui.color(Token::TextSecondary);
        let muted_c = ui.color(Token::TextMuted);
        let st_tag = ui.text_style(TextRole::Caption);
        let st_title = ui.text_style(TextRole::Heading2);
        let st_body = ui.text_style(TextRole::Body);
        let f_ring = ui.fade(id, 1, ix.focus_visible, mo.hover_in, mo.hover_out, Ease::Out);
        let ring_c = ui.color(Token::FocusRing).multiply_alpha(f_ring);

        ui.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(300.0, self.height),
            Some(stage_id),
            Sense::CLICK,
            CursorIcon::Default,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_card), bg_c, 0.0, Color::TRANSPARENT);
                cx.painter.push_clip(r.expand(-1.0));
                let w = r.width();
                for (si, sl) in slides_vec.iter().enumerate() {
                    let dx = (si as f32 - pos) * w;
                    if dx.abs() > w * 1.05 {
                        continue;
                    }
                    let sr = r.translate(vec2(dx, 0.0));
                    let (sub_bg, strong_c) = pal[si];

                    // Soft interior ambient glow kept clear of outer corner arcs.
                    cx.painter.shadow(
                        Rect::from_center_size(vec2(sr.max.x - 110.0, sr.min.y + 68.0), vec2(96.0, 96.0)),
                        Corners::all(48.0),
                        &crate::theme::Shadow {
                            x: 0.0,
                            y: 0.0,
                            blur: 96.0,
                            spread: 0.0,
                            color: strong_c.with_alpha(0.14),
                        },
                    );

                    // Icon badge + tag pill.
                    let tile = Rect::from_xywh(sr.min.x + 22.0, sr.min.y + 20.0, 38.0, 38.0);
                    cx.painter.rrect(tile, Corners::all(10.0), sub_bg, 0.0, Color::TRANSPARENT);
                    cx.icon(&sl.icon, tile.center(), 19.0, strong_c);

                    let g_tag = cx.galley(&sl.tag.to_uppercase(), &st_tag);
                    let pill_r = Rect::from_xywh(tile.max.x + 12.0, tile.center().y - 11.0, g_tag.width + 16.0, 22.0);
                    cx.painter.rrect(pill_r, Corners::all(11.0), sub_bg, 0.0, Color::TRANSPARENT);
                    cx.painter.text(cx.text, &g_tag, vec2(pill_r.min.x + 8.0, pill_r.min.y + (22.0 - g_tag.height) * 0.5), strong_c);

                    // Counter top-right.
                    let cnt = format!("{} / {}", si + 1, slides_vec.len());
                    let g_cnt = cx.galley(&cnt, &st_tag);
                    cx.painter.text(cx.text, &g_cnt, vec2(sr.max.x - 22.0 - g_cnt.width, tile.center().y - g_cnt.height * 0.5), muted_c);

                    // Title + wrapping body.
                    let text_w = (w - 44.0).max(80.0);
                    let g_t = cx.galley(&sl.title, &st_title);
                    cx.painter.text(cx.text, &g_t, vec2(sr.min.x + 22.0, tile.max.y + 16.0), fg_c);

                    let g_b = cx.text.layout(&sl.body, &st_body, Some(text_w), lumen_text::TextAlign::Start);
                    cx.painter.text(cx.text, &g_b, vec2(sr.min.x + 22.0, tile.max.y + 22.0 + g_t.height), sec_c);
                }
                cx.painter.pop_clip();
                cx.painter.ring(r, Corners::all(r_card), 1.0, bd_c);
                if f_ring > 0.005 {
                    cx.painter.ring(r.expand(2.0), Corners::all(r_card + 2.0), 2.0, ring_c);
                }
            })),
        );

        // Controls footer: Prev, stretching dots, Next.
        ui.begin(Style { dir: Dir::Row, width: Size::FILL, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        let prev_c = ui.scope("prev", |ui| ui.button("Previous").ghost().small().icon("chevron-left").clicked());

        // Stretching dot indicators centered.
        ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap: 6.0, align: Align::Center, justify: Justify::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        for di in 0..n {
            let did = id.with(("dot", di));
            let d_ix = ui.interact(did, Sense::CLICK, false);
            let is_act = di == *self.active;
            ui.a11y(did, || A11y::new(Role::Tab).label(format!("Slide {}", di + 1)).selected(is_act).position(di, n));
            let t = ui.spring(did, 0, if is_act { 1.0 } else { 0.0 }, mo.snappy).clamp(0.0, 1.0);
            let dw = 7.0 + 15.0 * t;
            let dc = ui.color(Token::BorderStrong).mix(ui.color(Token::Accent), t);
            ui.leaf(
                Style::default(),
                vec2(dw, 7.0),
                Some(did),
                Sense::CLICK,
                CursorIcon::Pointer,
                Some(Box::new(move |cx, r| cx.painter.rrect(r, Corners::all(3.5), dc, 0.0, Color::TRANSPARENT))),
            );
            if d_ix.clicked && *self.active != di {
                *self.active = di;
                changed = true;
            }
        }
        ui.end();

        let next_c = ui.scope("next", |ui| ui.button("Next").secondary().small().trailing_icon("chevron-right").clicked());
        ui.end();
        ui.end();

        if prev_c {
            *self.active = (*self.active + n - 1) % n;
            changed = true;
        } else if next_c {
            *self.active = (*self.active + 1) % n;
            changed = true;
        }

        let rect = ui.rect_of(id).unwrap_or(ix.rect);
        Response { id, rect, hovered: ix.hovered, clicked: ix.clicked, changed, focused: ix.focused }
    }
}

impl Drop for Carousel<'_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

// ================================================================== Callout

/// Documentation / inline callout card (`Note`, `Tip`, `Warning`, `Danger`)
/// with left accent bar, tone icon, title and arbitrary body content.
pub struct Callout<'u> {
    ui: &'u mut Ui,
    tone: Tone,
    title: String,
    icon: Option<String>,
}

impl<'u> Callout<'u> {
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = Some(name.to_owned());
        self
    }

    pub fn show<R>(self, body: impl FnOnce(&mut Ui) -> R) -> R {
        let Callout { ui, tone, title, icon } = self;
        let id = ui.make_id(("callout", &title));
        let (bg_tok, bar_tok, default_ic) = match tone {
            Tone::Neutral => (Token::SurfaceMuted, Token::BorderStrong, "bookmark"),
            Tone::Accent | Tone::Info => (Token::AccentSubtle, Token::Accent, "info"),
            Tone::Success => (Token::SuccessSubtle, Token::Success, "lightbulb"),
            Tone::Warning => (Token::WarningSubtle, Token::Warning, "triangle-alert"),
            Tone::Danger => (Token::DangerSubtle, Token::Danger, "octagon-alert"),
        };
        let strong_tok = match tone {
            Tone::Neutral => Token::Foreground,
            Tone::Accent | Tone::Info => Token::AccentStrong,
            Tone::Success => Token::SuccessStrong,
            Tone::Warning => Token::WarningStrong,
            Tone::Danger => Token::DangerStrong,
        };
        let bg_c = ui.color(bg_tok).multiply_alpha(0.55);
        let bar_c = ui.color(bar_tok);
        let bd_c = bar_c.multiply_alpha(0.28);
        let ic_c = ui.color(strong_tok);
        let r_card = (ui.look().card.radius - 3.0).max(8.0);
        let ic_name = icon.unwrap_or_else(|| default_ic.to_owned());

        ui.begin(
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                pad: Sides { left: 19.0, right: 14.0, top: 12.0, bottom: 12.0 },
                gap: 11.0,
                align: Align::Start,
                ..Default::default()
            },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_card), bg_c, 1.0, bd_c);
                cx.painter.rrect(
                    Rect::from_xywh(r.min.x + 6.0, r.min.y + 10.0, 3.5, (r.height() - 20.0).max(8.0)),
                    Corners::all(2.0),
                    bar_c,
                    0.0,
                    Color::TRANSPARENT,
                );
            })),
        );
        ui.begin(Style { pad: Sides { top: 2.0, ..Default::default() }, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        ui.icon_leaf(&ic_name, 16.0, ic_c);
        ui.end();

        ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: 4.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if !title.is_empty() {
            ui.text_leaf(title, ui.text_style(TextRole::BodyStrong), ic_c, false, true, Style { width: Size::FILL, ..Default::default() });
        }
        let out = ui.scope("callout_b", body);
        ui.end();
        ui.end();
        out
    }
}

// ================================================================== UndoBar

/// Output from [`UndoBar::show`].
#[derive(Clone, Copy, Debug, Default)]
pub struct UndoBarOut {
    pub undo_clicked: bool,
    pub dismissed: bool,
}

/// Elevated action / undo snackbar with a remaining-time progress bar along its
/// bottom edge, primary `"Undo"` button, and dismiss `×`.
pub struct UndoBar<'u> {
    ui: &'u mut Ui,
    key: String,
    message: String,
    action_label: String,
    remaining: f32,
}

impl<'u> UndoBar<'u> {
    pub fn action_label(mut self, l: impl Into<String>) -> Self {
        self.action_label = l.into();
        self
    }

    pub fn show(self) -> UndoBarOut {
        let UndoBar { ui, key, message, action_label, remaining } = self;
        let id = ui.make_id(("undo_bar", &key));
        let mo = ui.motion();
        let enter = ui.anims.spring_from(id, 0, 0.0, 1.0, mo.bouncy).clamp(0.0, 1.08);
        let prog = remaining.clamp(0.0, 1.0);

        let r_bar = (ui.look().card.radius - 2.0).max(10.0);
        let bg_c = ui.color(Token::SurfaceRaised);
        let bd_c = ui.color(Token::Border);
        let bar_c = ui.color(Token::Accent);
        let ink = ui.color(Token::ShadowInk);

        ui.begin(
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                pad: Sides { left: 14.0, right: 10.0, top: 10.0, bottom: 12.0 },
                gap: 10.0,
                align: Align::Center,
                ..Default::default()
            },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| {
                let lift_y = (1.0 - enter.min(1.0)) * 8.0;
                let sr = r.translate(vec2(0.0, lift_y));
                cx.painter.shadow(
                    sr,
                    Corners::all(r_bar),
                    &crate::theme::Shadow { x: 0.0, y: 6.0, blur: 18.0, spread: 0.0, color: ink.with_alpha(0.18) },
                );
                cx.painter.rrect(sr, Corners::all(r_bar), bg_c, 1.0, bd_c);
                // Bottom progress strip.
                let pw = ((sr.width() - r_bar * 1.4).max(0.0)) * prog;
                if pw > 1.0 {
                    cx.painter.rrect(
                        Rect::from_xywh(sr.min.x + r_bar * 0.7, sr.max.y - 3.0, pw, 2.0),
                        Corners::all(1.0),
                        bar_c,
                        0.0,
                        Color::TRANSPARENT,
                    );
                }
            })),
        );
        ui.icon_leaf("rotate-ccw", 15.0, ui.color(Token::AccentStrong));
        ui.text_leaf(message, ui.text_style(TextRole::Body), ui.color(Token::Foreground), false, true, Style { width: Size::FILL, ..Default::default() });
        let undo_clicked = ui.scope("undo_btn", |ui| ui.button(&action_label).primary().small().clicked());
        let dismissed = ui.scope("dismiss_btn", |ui| ui.icon_button("x").small().tooltip("Dismiss").clicked());
        ui.end();

        UndoBarOut { undo_clicked, dismissed }
    }
}

// =============================================================== ImageFrame

/// Media load state for [`ImageFrame`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ImageStatus {
    #[default]
    Loaded,
    Loading,
    Fallback,
}

/// Aspect-ratio media frame with procedural Ethereal artwork when `Loaded`,
/// skeleton shimmer when `Loading`, graceful broken-media state when
/// `Fallback`, caption pill, and click-to-zoom scale!
pub struct ImageFrame<'u> {
    ui: &'u mut Ui,
    key: String,
    title: String,
    caption: Option<String>,
    status: ImageStatus,
    height: f32,
    shown: bool,
}

impl<'u> ImageFrame<'u> {
    pub fn caption(mut self, c: impl Into<String>) -> Self {
        self.caption = Some(c.into());
        self
    }
    pub fn status(mut self, s: ImageStatus) -> Self {
        self.status = s;
        self
    }
    pub fn height(mut self, px: f32) -> Self {
        self.height = px.max(110.0);
        self
    }
    pub fn show(mut self) -> Response {
        self.shown = true;
        self.show_inner()
    }

    fn show_inner(&mut self) -> Response {
        let ui = &mut *self.ui;
        let id = ui.make_id(("image_frame", &self.key));
        let ix = ui.interact(id, Sense::CLICK, false);
        let mo = ui.motion();

        #[derive(Default)]
        struct ZoomMem(bool);
        if ix.clicked && self.status == ImageStatus::Loaded {
            let z = &mut ui.mem::<ZoomMem>(id).0;
            *z = !*z;
        }
        let zoomed = ui.mem::<ZoomMem>(id).0 && self.status == ImageStatus::Loaded;
        let zoom_t = ui.spring(id, 0, if zoomed { 1.0 } else { 0.0 }, mo.bouncy);
        let hov = ui.fade(id, 1, ix.hovered, mo.hover_in, mo.hover_out, Ease::Out);

        let r_card = ui.look().card.radius;
        let bg_c = ui.color(Token::SurfaceMuted);
        let bd_c = ui.color(Token::BorderSubtle);
        let c0 = ui.color(Token::Accent);
        let c1 = ui.color(Token::Series3);
        let c2 = ui.color(Token::Series2);
        let fg_c = ui.color(Token::Foreground);
        let muted_c = ui.color(Token::TextMuted);
        let badge_bg = ui.color(Token::Surface).multiply_alpha(0.84);
        let st_title = ui.text_style(TextRole::BodyStrong);
        let st_cap = ui.text_style(TextRole::Caption);

        let status = self.status;
        let title_cl = self.title.clone();
        let cap_cl = self.caption.clone();
        let time = ui.time() as f32;
        if status == ImageStatus::Loading && !ui.reduced_motion() {
            ui.request_repaint_after(0.016);
        }

        ui.leaf(
            Style { width: Size::FILL, ..Default::default() },
            vec2(260.0, self.height),
            Some(id),
            Sense::CLICK,
            if status == ImageStatus::Loaded { CursorIcon::Pointer } else { CursorIcon::Default },
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_card), bg_c, 0.0, Color::TRANSPARENT);
                cx.painter.push_clip(r.expand(-1.0));
                match status {
                    ImageStatus::Loaded => {
                        let scale = 1.0 + 0.04 * hov + 0.14 * zoom_t;
                        let center = r.center();
                        // Ethereal aurora procedural scene: layered Gaussian orbs + crisp geometric horizon!
                        cx.painter.shadow(
                            Rect::from_center_size(center + vec2(-44.0 * scale, -18.0 * scale), vec2(118.0 * scale, 118.0 * scale)),
                            Corners::all(59.0 * scale),
                            &crate::theme::Shadow { x: 0.0, y: 0.0, blur: 100.0, spread: 0.0, color: c0.with_alpha(0.42) },
                        );
                        cx.painter.shadow(
                            Rect::from_center_size(center + vec2(48.0 * scale, -8.0 * scale), vec2(106.0 * scale, 106.0 * scale)),
                            Corners::all(53.0 * scale),
                            &crate::theme::Shadow { x: 0.0, y: 0.0, blur: 96.0, spread: 0.0, color: c1.with_alpha(0.36) },
                        );
                        cx.painter.shadow(
                            Rect::from_center_size(center + vec2(4.0, 32.0 * scale), vec2(130.0 * scale, 70.0 * scale)),
                            Corners::all(35.0),
                            &crate::theme::Shadow { x: 0.0, y: 0.0, blur: 90.0, spread: 0.0, color: c2.with_alpha(0.26) },
                        );
                        // Sun / lens disc.
                        cx.painter.circle(center + vec2(0.0, -14.0 * scale), 22.0 * scale, Color::WHITE.with_alpha(0.85), 0.0, Color::TRANSPARENT);

                        // Caption bar bottom-left + zoom icon bottom-right.
                        let bar_h = 32.0;
                        let bar = Rect::from_xywh(r.min.x + 10.0, r.max.y - bar_h - 10.0, (r.width() - 20.0).max(40.0), bar_h);
                        cx.painter.rrect(bar, Corners::all(9.0), badge_bg, 1.0, bd_c);
                        let lbl = match cap_cl {
                            Some(ref c) => format!("{title_cl} · {c}"),
                            None => title_cl.clone(),
                        };
                        // Shortened to stop short of the zoom icon.
                        let g = cx.galley_fit(&lbl, &st_cap, bar.width() - 10.0 - 34.0);
                        cx.painter.text(cx.text, &g, vec2(bar.min.x + 10.0, bar.center().y - g.height * 0.5), fg_c);
                        cx.icon(if zoomed { "zoom-out" } else { "zoom-in" }, vec2(bar.max.x - 16.0, bar.center().y), 14.0, muted_c);
                    }
                    ImageStatus::Loading => {
                        let pulse = 0.45 + 0.35 * (time * 3.2).sin().abs();
                        cx.painter.rrect(r.expand(-12.0), Corners::all(10.0), c0.with_alpha(0.08 * pulse), 0.0, Color::TRANSPARENT);
                        cx.icon("loader", r.center() - vec2(0.0, 10.0), 22.0, muted_c.multiply_alpha(pulse));
                        let g = cx.galley("Decoding high-DPI texture…", &st_cap);
                        cx.painter.text(cx.text, &g, vec2(r.center().x - g.width * 0.5, r.center().y + 10.0), muted_c);
                    }
                    ImageStatus::Fallback => {
                        cx.icon("image-off", r.center() - vec2(0.0, 16.0), 24.0, muted_c);
                        let gt = cx.galley(&title_cl, &st_title);
                        cx.painter.text(cx.text, &gt, vec2(r.center().x - gt.width * 0.5, r.center().y + 4.0), fg_c);
                        let gs = cx.galley("Preview unavailable — fallback surface rendered", &st_cap);
                        cx.painter.text(cx.text, &gs, vec2(r.center().x - gs.width * 0.5, r.center().y + 6.0 + gt.height), muted_c);
                    }
                }
                cx.painter.pop_clip();
                cx.painter.ring(r, Corners::all(r_card), 1.0, bd_c);
            })),
        );

        Response { id, rect: ix.rect, hovered: ix.hovered, clicked: ix.clicked, changed: ix.clicked, focused: ix.focused }
    }
}

impl Drop for ImageFrame<'_> {
    fn drop(&mut self) {
        if !self.shown {
            self.shown = true;
            self.show_inner();
        }
    }
}

impl Ui {
    /// Closable IDE / browser tab strip with dirty indicator dot (`•`), `×`
    /// close button, `+` add button, and a spring-sliding active underline.
    pub fn editor_tabs(&mut self, key: impl std::hash::Hash, tabs: &[EditorTab], active: &mut usize) -> EditorTabsOut {
        let id = self.make_id(("editor_tabs", key));
        let mut out = EditorTabsOut::default();
        if !tabs.is_empty() {
            *active = (*active).min(tabs.len() - 1);
        }

        let r_bar = (self.look().card.radius - 2.0).max(8.0);
        let bar_bg = self.color(Token::SurfaceMuted).multiply_alpha(0.55);
        let bar_bd = self.color(Token::BorderSubtle);
        let bar_rect_prev = self.rect_of(id).unwrap_or(Rect::ZERO);
        let mo = self.motion();

        let slide = *self.mem::<TabSlide>(id);
        let ind_x = if slide.ready { self.spring(id, 0, slide.x, mo.snappy) } else { slide.x };
        let ind_w = if slide.ready { self.spring(id, 1, slide.w, mo.snappy) } else { slide.w };
        let accent_c = self.color(Token::Accent);
        let active_card_bg = self.color(Token::Surface);
        let has_tabs = !tabs.is_empty() && slide.ready;
        // The tabs scroll sideways when they don't fit. The pill lives in
        // content coordinates; paint subtracts this frame's offset so it
        // moves with the tabs instead of springing after them.
        let off = std::rc::Rc::new(std::cell::Cell::new(0.0f32));
        let off_paint = off.clone();
        let fade = std::rc::Rc::new(std::cell::Cell::new((false, false)));
        let fade_paint = fade.clone();
        // The bar fill as it shows over a card, so the fade blends in.
        let fade_c = self.color(Token::Surface).mix(self.color(Token::SurfaceMuted), 0.55);
        let last_off = slide.last_off;

        let bar = self.begin(
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                pad: Sides { left: 6.0, right: 6.0, top: 5.0, bottom: 5.0 },
                gap: 4.0,
                align: Align::Center,
                ..Default::default()
            },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            true,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_bar), bar_bg, 1.0, bar_bd);
                if has_tabs && ind_w > 4.0 {
                    cx.painter.push_clip(r.expand(-1.0));
                    let pill = Rect::from_xywh(r.min.x + ind_x - off_paint.get(), r.min.y + 4.0, ind_w, (r.height() - 8.0).max(16.0));
                    cx.painter.rrect(pill, Corners::all((r_bar - 3.0).max(5.0)), active_card_bg, 1.0, bar_bd);
                    cx.painter.rrect(
                        Rect::from_xywh(pill.min.x + 10.0, pill.max.y - 2.0, (pill.width() - 20.0).max(4.0), 2.0),
                        Corners::all(1.0),
                        accent_c,
                        0.0,
                        Color::TRANSPARENT,
                    );
                    cx.painter.pop_clip();
                }
            })),
        );

        let mut target_rel: Option<(f32, f32)> = None;
        let (_, sx) = self.scroll_row("tabs", Style { width: Size::FILL, gap: 4.0, align: Align::Center, ..Default::default() }, |ui| {
                    for (ti, tab) in tabs.iter().enumerate() {
                let tid = id.with(("tab", &tab.id));
                let close_id = tid.with("close");
                let c_ix = ui.interact(close_id, Sense::CLICK, false);
                let t_ix = ui.interact(tid, Sense::CLICK, false);

                if t_ix.focused {
                    if ui.key_pressed_for(tid, Key::ArrowRight) && !tabs.is_empty() {
                        *active = (*active + 1) % tabs.len();
                        out.changed = true;
                    } else if ui.key_pressed_for(tid, Key::ArrowLeft) && !tabs.is_empty() {
                        *active = (*active + tabs.len() - 1) % tabs.len();
                        out.changed = true;
                    }
                }

                let is_act = ti == *active;
                if is_act && t_ix.rect.width() > 1.0 && bar_rect_prev.width() > 1.0 {
                    target_rel = Some((t_ix.rect.min.x - bar_rect_prev.min.x, t_ix.rect.width()));
                }
                let hov = ui.fade(tid, 0, t_ix.hovered && !is_act, mo.hover_in, mo.hover_out, Ease::Out);
                let hov_bg = ui.color(Token::Hover).multiply_alpha(hov);

                ui.begin(
                    Style { dir: Dir::Row, pad: Sides::xy(10.0, 6.0), gap: 7.0, align: Align::Center, ..Default::default() },
                    Some(tid),
                    Sense::CLICK,
                    CursorIcon::Pointer,
                    false,
                    Some(Box::new(move |cx, r| {
                        if hov > 0.005 {
                            cx.painter.rrect(r, Corners::all(6.0), hov_bg, 0.0, Color::TRANSPARENT);
                        }
                    })),
                );
                if let Some(ref ic) = tab.icon {
                    let ic_c = if is_act { ui.color(Token::AccentStrong) } else { ui.color(Token::TextMuted) };
                    ui.icon_leaf(ic, 13.5, ic_c);
                }
                let mut st = ui.text_style(TextRole::Label);
                if is_act {
                    st.weight = 600;
                }
                let fg = if is_act { ui.color(Token::Foreground) } else { ui.color(Token::TextSecondary) };
                // Tabs keep their width; the bar clips the overflow instead of
                // squeezing labels into each other.
                let label_w = ui.galley(&tab.label, &st).width.ceil();
                ui.text_leaf(tab.label.clone(), st, fg, false, false, Style { min: vec2(label_w, 0.0), ..Default::default() });

                if tab.dirty && !t_ix.hovered && !c_ix.hovered {
                    let dot_c = ui.color(Token::WarningStrong);
                    ui.leaf(
                        Style { min: vec2(14.0, 14.0), ..Default::default() },
                        vec2(14.0, 14.0),
                        Some(close_id),
                        Sense::CLICK,
                        CursorIcon::Pointer,
                        Some(Box::new(move |cx, r| cx.painter.circle(r.center(), 3.5, dot_c, 0.0, Color::TRANSPARENT))),
                    );
                } else {
                    let c_hov = ui.fade(close_id, 0, c_ix.hovered, mo.hover_in, mo.hover_out, Ease::Out);
                    let x_bg = ui.color(Token::Hover).multiply_alpha(c_hov);
                    let x_fg = ui.color(if c_ix.hovered { Token::Foreground } else { Token::TextMuted });
                    ui.leaf(
                        Style { min: vec2(16.0, 16.0), ..Default::default() },
                        vec2(16.0, 16.0),
                        Some(close_id),
                        Sense::CLICK,
                        CursorIcon::Pointer,
                        Some(Box::new(move |cx, r| {
                            if c_hov > 0.01 {
                                cx.painter.rrect(r, Corners::all(4.0), x_bg, 0.0, Color::TRANSPARENT);
                            }
                            cx.icon("x", r.center(), 11.0, x_fg);
                        })),
                    );
                }
                ui.end();

                if c_ix.clicked {
                    out.closed = Some(ti);
                } else if t_ix.clicked && *active != ti {
                    *active = ti;
                    out.changed = true;
                }
            }
        });
        off.set(sx.offset);
        fade.set((sx.offset > 0.5, sx.offset < sx.max - 0.5));

        // Soft edges where more tabs hide, then the `+` add tab button.
        let row_id = sx.id;
        self.set_paint_after(
            bar,
            Box::new(move |cx, _| {
                let Some(&rr) = cx.rects.get(&row_id) else { return };
                let (left, right) = fade_paint.get();
                let w = 22.0;
                if left {
                    cx.painter.rrect_horiz(Rect::from_xywh(rr.min.x, rr.min.y, w, rr.height()), Corners::all(0.0), fade_c, fade_c.with_alpha(0.0));
                }
                if right {
                    cx.painter.rrect_horiz(Rect::from_xywh(rr.max.x - w, rr.min.y, w, rr.height()), Corners::all(0.0), fade_c.with_alpha(0.0), fade_c);
                }
            }),
        );
        out.add_clicked = self.scope("add_tab", |ui| ui.icon_button("plus").small().tooltip("New tab").clicked());
        self.end();

        if let Some((rx, rw)) = target_rel {
            let mem = self.mem::<TabSlide>(id);
            mem.x = rx + last_off;
            mem.w = rw;
            mem.ready = true;
        }
        self.mem::<TabSlide>(id).last_off = sx.offset;
        // A newly chosen tab scrolls into view.
        if self.mem::<TabSlide>(id).shown != Some(*active) {
            if let Some(r) = tabs.get(*active).and_then(|t| self.rect_of(id.with(("tab", &t.id)))) {
                self.scroll_row_reveal(sx.id, r);
            }
            self.mem::<TabSlide>(id).shown = Some(*active);
        }
        out
    }

    /// Vertical `"On this page"` scroll-spy rail with nested sections and a
    /// spring-sliding vertical accent bar.
    pub fn table_of_contents(&mut self, key: impl std::hash::Hash, items: &[TocItem], active: &mut usize) -> Response {
        let id = self.make_id(("toc", key));
        let n = items.len().max(1);
        *active = (*active).min(n - 1);
        let mo = self.motion();
        let mut changed = false;

        let rail_prev = self.rect_of(id).unwrap_or(Rect::ZERO);
        let slide = *self.mem::<TocSlide>(id);
        let ind_y = if slide.ready { self.spring(id, 0, slide.y, mo.snappy) } else { slide.y };
        let ind_h = if slide.ready { self.spring(id, 1, slide.h, mo.snappy) } else { 22.0 };
        let track_c = self.color(Token::BorderSubtle);
        let act_c = self.color(Token::Accent);
        let show_ind = slide.ready;

        self.begin(
            Style { dir: Dir::Column, width: Size::FILL, gap: 6.0, ..Default::default() },
            None,
            Sense::NONE,
            CursorIcon::Default,
            false,
            None,
        );
        let mut st_hdr = self.text_style(TextRole::Caption);
        st_hdr.weight = 600;
        self.begin(Style { dir: Dir::Row, gap: 6.0, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        self.icon_leaf("text", 13.0, self.color(Token::TextMuted));
        self.text_leaf("ON THIS PAGE", st_hdr, self.color(Token::TextMuted), false, false, Style::default());
        self.end();

        self.begin(
            Style {
                dir: Dir::Column,
                width: Size::FILL,
                pad: Sides { left: 14.0, right: 4.0, top: 2.0, bottom: 2.0 },
                gap: 2.0,
                ..Default::default()
            },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                cx.painter.rect(Rect::from_xywh(r.min.x + 1.0, r.min.y, 1.5, r.height()), track_c);
                if show_ind {
                    cx.painter.rrect(
                        Rect::from_xywh(r.min.x + 0.5, r.min.y + ind_y + 3.0, 2.5, (ind_h - 6.0).max(10.0)),
                        Corners::all(1.5),
                        act_c,
                        0.0,
                        Color::TRANSPARENT,
                    );
                }
            })),
        );

        let mut target_y: Option<(f32, f32)> = None;
        for (i, item) in items.iter().enumerate() {
            let rid = id.with(("toc_i", i));
            let ix = self.interact(rid, Sense::CLICK, false);
            if ix.focused {
                if self.key_pressed_for(rid, Key::ArrowDown) {
                    *active = (*active + 1) % n;
                    changed = true;
                } else if self.key_pressed_for(rid, Key::ArrowUp) {
                    *active = (*active + n - 1) % n;
                    changed = true;
                }
            }
            let is_act = i == *active;
            if is_act && ix.rect.height() > 1.0 && rail_prev.height() > 1.0 {
                target_y = Some((ix.rect.min.y - rail_prev.min.y, ix.rect.height()));
            }
            let indent = item.depth as f32 * 14.0;
            let hov = self.fade(rid, 0, ix.hovered && !is_act, mo.hover_in, mo.hover_out, Ease::Out);
            let bg = self.color(Token::Hover).multiply_alpha(hov);

            self.begin(
                Style {
                    dir: Dir::Row,
                    width: Size::FILL,
                    pad: Sides { left: 8.0 + indent, right: 8.0, top: 5.0, bottom: 5.0 },
                    align: Align::Center,
                    ..Default::default()
                },
                Some(rid),
                Sense::CLICK,
                CursorIcon::Pointer,
                false,
                Some(Box::new(move |cx, r| {
                    if hov > 0.005 {
                        cx.painter.rrect(r, Corners::all(6.0), bg, 0.0, Color::TRANSPARENT);
                    }
                })),
            );
            let mut st = self.text_style(if item.depth == 0 { TextRole::Label } else { TextRole::Caption });
            if is_act {
                st.weight = 600;
            }
            let col = if is_act {
                self.color(Token::AccentStrong)
            } else if ix.hovered {
                self.color(Token::Foreground)
            } else {
                self.color(Token::TextSecondary)
            };
            self.text_leaf(item.label.clone(), st, col, false, true, Style { width: Size::FILL, ..Default::default() });
            self.end();

            if ix.clicked && *active != i {
                *active = i;
                changed = true;
            }
        }
        self.end();
        self.end();

        if let Some((ry, rh)) = target_y {
            let mem = self.mem::<TocSlide>(id);
            mem.y = ry;
            mem.h = rh;
            mem.ready = true;
        }
        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, changed, ..Default::default() }
    }

    /// Top horizontal navigation bar with a spring-gliding active pill, icons,
    /// and badge pills.
    pub fn nav_bar(&mut self, key: impl std::hash::Hash, items: &[NavBarItem], active: &mut usize) -> Response {
        let id = self.make_id(("nav_bar", key));
        let n = items.len().max(1);
        *active = (*active).min(n - 1);
        let mo = self.motion();
        let mut changed = false;

        let bar_prev = self.rect_of(id).unwrap_or(Rect::ZERO);
        let slide = *self.mem::<TabSlide>(id);
        let ind_x = if slide.ready { self.spring(id, 0, slide.x, mo.snappy) } else { slide.x };
        let ind_w = if slide.ready { self.spring(id, 1, slide.w, mo.snappy) } else { slide.w };

        let r_bar = ui_look_nav_radius(self);
        let bg_c = self.color(Token::Surface);
        let bd_c = self.color(Token::BorderSubtle);
        let pill_c = self.color(Token::AccentSubtle);
        let pill_bd = self.color(Token::Accent).multiply_alpha(0.32);
        let ready = slide.ready;

        self.begin(
            Style {
                dir: Dir::Row,
                width: Size::FILL,
                pad: Sides::all(5.0),
                gap: 4.0,
                align: Align::Center,
                ..Default::default()
            },
            Some(id),
            Sense::NONE,
            CursorIcon::Default,
            false,
            Some(Box::new(move |cx, r| {
                cx.painter.rrect(r, Corners::all(r_bar), bg_c, 1.0, bd_c);
                if ready && ind_w > 4.0 {
                    let pr = Rect::from_xywh(r.min.x + ind_x, r.min.y + 5.0, ind_w, (r.height() - 10.0).max(16.0));
                    cx.painter.rrect(pr, Corners::all((r_bar - 4.0).max(6.0)), pill_c, 1.0, pill_bd);
                }
            })),
        );

        let mut target_rel: Option<(f32, f32)> = None;
        for (i, item) in items.iter().enumerate() {
            let iid = id.with(("n_it", i));
            let ix = self.interact(iid, Sense::CLICK, false);
            if ix.focused {
                if self.key_pressed_for(iid, Key::ArrowRight) {
                    *active = (*active + 1) % n;
                    changed = true;
                } else if self.key_pressed_for(iid, Key::ArrowLeft) {
                    *active = (*active + n - 1) % n;
                    changed = true;
                }
            }
            let is_act = i == *active;
            if is_act && ix.rect.width() > 1.0 && bar_prev.width() > 1.0 {
                target_rel = Some((ix.rect.min.x - bar_prev.min.x, ix.rect.width()));
            }
            let hov = self.fade(iid, 0, ix.hovered && !is_act, mo.hover_in, mo.hover_out, Ease::Out);
            let h_bg = self.color(Token::Hover).multiply_alpha(hov);

            self.begin(
                Style { dir: Dir::Row, pad: Sides::xy(12.0, 7.0), gap: 7.0, align: Align::Center, ..Default::default() },
                Some(iid),
                Sense::CLICK,
                CursorIcon::Pointer,
                false,
                Some(Box::new(move |cx, r| {
                    if hov > 0.005 {
                        cx.painter.rrect(r, Corners::all(7.0), h_bg, 0.0, Color::TRANSPARENT);
                    }
                })),
            );
            let fg = if is_act { self.color(Token::AccentStrong) } else { self.color(Token::TextSecondary) };
            if let Some(ref ic) = item.icon {
                self.icon_leaf(ic, 14.0, fg);
            }
            let mut st = self.text_style(TextRole::Label);
            if is_act {
                st.weight = 600;
            }
            self.text_leaf(item.label.clone(), st, fg, false, false, Style::default());
            if let Some(ref b) = item.badge {
                self.badge(b, Tone::Accent);
            }
            self.end();

            if ix.clicked && *active != i {
                *active = i;
                changed = true;
            }
        }
        self.end();

        if let Some((rx, rw)) = target_rel {
            let mem = self.mem::<TabSlide>(id);
            mem.x = rx;
            mem.w = rw;
            mem.ready = true;
        }
        let rect = self.rect_of(id).unwrap_or(Rect::ZERO);
        Response { id, rect, changed, ..Default::default() }
    }

    /// Horizontal spring-sliding feature/card carousel.
    pub fn carousel<'u>(&'u mut self, key: impl Into<String>, active: &'u mut usize, slides: &'u [CarouselSlide]) -> Carousel<'u> {
        Carousel { ui: self, key: key.into(), active, slides, height: 184.0, shown: false }
    }

    /// Rich documentation / inline callout card (`Tone::Info`, `Success`, `Warning`, `Danger`).
    pub fn callout(&mut self, tone: Tone, title: impl Into<String>) -> Callout<'_> {
        Callout { ui: self, tone, title: title.into(), icon: None }
    }

    /// Elevated undo snackbar with remaining-time progress bar and `"Undo"` button.
    pub fn undo_bar(&mut self, key: impl Into<String>, message: impl Into<String>, remaining: f32) -> UndoBar<'_> {
        UndoBar {
            ui: self,
            key: key.into(),
            message: message.into(),
            action_label: "Undo".into(),
            remaining,
        }
    }

    /// Aspect-locked media frame with procedural Ethereal art, loading shimmer, and fallback state.
    pub fn image_frame(&mut self, key: impl Into<String>, title: impl Into<String>) -> ImageFrame<'_> {
        ImageFrame {
            ui: self,
            key: key.into(),
            title: title.into(),
            caption: None,
            status: ImageStatus::Loaded,
            height: 176.0,
            shown: false,
        }
    }
}

fn ui_look_nav_radius(ui: &Ui) -> f32 {
    (ui.look().card.radius - 2.0).max(8.0)
}
