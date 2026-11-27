//! Avatar upload: a round photo slot with its actions beside it. Empty, it
//! shows a dashed ring and a camera that turn toward the accent on hover;
//! uploading, a progress ring sweeps around the dimmed photo; done, a check
//! pops in at the corner; failed, the ring turns red and Retry appears.
//! lumen doesn't open file dialogs: `choose` tells the app to open its own.

use std::f32::consts::TAU;

use crate::a11y::{A11y, Role};
use crate::input::CursorIcon;
use crate::layout::{Align, Dir, Size, Style};
use crate::math::{vec2, Corners, Rect};
use crate::theme::{TextRole, Token};
use crate::ui::{Response, Sense, Ui};
use crate::Color;

/// Where an avatar upload stands. The app drives it.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum UploadState {
    /// No photo yet.
    #[default]
    Empty,
    /// Uploading, 0..1.
    Uploading(f32),
    /// The photo is set.
    Done,
    /// The last upload failed, with a reason.
    Failed(String),
}

/// What the person asked for this frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct AvatarOut {
    pub response: Response,
    /// Open a file picker (the photo or Upload / Change was pressed).
    pub choose: bool,
    /// Remove the photo, or cancel the upload in progress.
    pub remove: bool,
    pub retry: bool,
}

#[derive(Default)]
struct AvatarMem {
    /// The state kind last frame, to pop the check when an upload lands.
    was: u8,
}

pub struct AvatarUpload<'u> {
    ui: &'u mut Ui,
    key: String,
    state: &'u UploadState,
    initials: String,
    tone: usize,
    title: String,
    hint: String,
    size: Option<f32>,
}

impl AvatarUpload<'_> {
    /// Initials drawn on the photo.
    pub fn initials(mut self, s: impl Into<String>) -> Self {
        self.initials = s.into();
        self
    }
    /// Photo color, from the chart series.
    pub fn tone(mut self, t: usize) -> Self {
        self.tone = t;
        self
    }
    pub fn title(mut self, s: impl Into<String>) -> Self {
        self.title = s.into();
        self
    }
    /// Shown under the title (the failure reason replaces it).
    pub fn hint(mut self, s: impl Into<String>) -> Self {
        self.hint = s.into();
        self
    }
    /// Diameter of the photo.
    pub fn size(mut self, d: f32) -> Self {
        self.size = Some(d);
        self
    }

    pub fn show(self) -> AvatarOut {
        let ui = self.ui;
        let id = ui.make_id(("avatar_upload", &self.key));
        let ix = ui.interact(id, Sense::CLICK, false);
        let (act, key_held) = super::key_activate(ui, &ix, true);
        let mo = ui.motion();
        let reduced = ui.reduced_motion();
        let lk = ui.look().clone();
        let state = self.state.clone();
        let kind = match state {
            UploadState::Empty => 0u8,
            UploadState::Uploading(_) => 1,
            UploadState::Done => 2,
            UploadState::Failed(_) => 3,
        };
        let was = std::mem::replace(&mut ui.mem::<AvatarMem>(id).was, kind);
        let d = self.size.unwrap_or((lk.control.height[2] * 2.0).round());
        let mut out = AvatarOut::default();
        if ix.clicked || act {
            match kind {
                1 => {}
                3 => out.retry = true,
                _ => out.choose = true,
            }
        }

        let (hov, press) = ui.state_weights(&ix, key_held);
        let squash = if reduced { 1.0 } else { ui.spring(id, 0, if ix.pressed || key_held { 0.96 } else { 1.0 }, mo.bouncy) };
        let (ra, rg) = ui.focus_anim(&ix);
        // The check pops in when an upload lands; it's simply there otherwise.
        let badge_key = id.with("badge");
        let badge = if kind != 2 {
            ui.anims.set_spring(badge_key, 0, 0.0);
            0.0
        } else if reduced || was == 2 && ui.anims.spring_state(badge_key, 0).is_none() {
            1.0
        } else if was != 2 {
            ui.anims.spring_from(badge_key, 0, 0.0, 1.0, mo.bouncy)
        } else {
            ui.spring(badge_key, 0, 1.0, mo.bouncy)
        };
        let progress = match state {
            UploadState::Uploading(p) => {
                let p = p.clamp(0.0, 1.0);
                if reduced {
                    p
                } else {
                    ui.spring(id, 1, p, mo.smooth)
                }
            }
            _ => {
                ui.anims.set_spring(id, 1, 0.0);
                0.0
            }
        };
        let spin = if reduced { 0.0 } else { ui.spring(id, 2, if ix.hovered { 1.0 } else { 0.0 }, mo.smooth) };
        let lift = if reduced { 0.0 } else { ui.spring(id, 3, if ix.hovered { 1.0 } else { 0.0 }, mo.bouncy) };
        let show_photo = matches!(kind, 1 | 2);
        let ring_t = if reduced { if kind == 1 { 1.0 } else { 0.0 } } else { ui.spring(id, 4, if kind == 1 { 1.0 } else { 0.0 }, mo.smooth).clamp(0.0, 1.0) };
        let shrink = 1.0 - 0.16 * ring_t;
        let scrim_a = if kind == 2 { hov } else if kind == 1 { 1.0 } else { 0.0 };

        let tk = [Token::Series0, Token::Series1, Token::Series2, Token::Series3, Token::Series4, Token::Series5][self.tone % 6];
        let photo = ui.color(tk);
        let photo_top = photo.mix(Color::WHITE, 0.25);
        let accent = ui.color(Token::Accent);
        let danger = ui.color(Token::Danger);
        let ok = ui.color(Token::Success);
        let surface = ui.color(Token::Surface);
        let muted_bg = ui.color(Token::SurfaceMuted).over(ui.color(Token::Hover).multiply_alpha(hov * 0.6 + press * 0.4));
        let dash_c = if kind == 3 { danger } else { ui.color(Token::BorderStrong).mix(accent, hov) };
        let icon_c = if kind == 3 { danger } else { ui.color(Token::TextSecondary).mix(accent, hov) };
        let track = ui.color(Token::Track);
        let scrim = ui.color(Token::Scrim);
        let ring = ui.color(Token::FocusRing);
        let mut st_init = ui.text_style(TextRole::Heading2);
        st_init.size = (d * 0.34).round();
        st_init.line_height = st_init.size * 1.2;
        let mut st_pct = ui.text_style(TextRole::Label);
        st_pct.tabular = true;
        let initials = self.initials.clone();
        let pct = format!("{:.0}%", progress * 100.0);
        let band = (d * 0.06).max(lk.focus.width + 1.0);
        let icon_sz = (d * 0.3).round();
        let edge_w = lk.focus.width.max(1.5);
        let lift_px = d * 0.03;

        let gap = lk.control.gap[2].max(10.0);
        ui.begin(Style { dir: Dir::Row, width: Size::FILL, gap, align: Align::Center, wrap: false, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        ui.leaf(
            Style::default(),
            vec2(d, d),
            Some(id),
            Sense::CLICK,
            if kind == 1 { CursorIcon::Default } else { CursorIcon::Pointer },
            Some(Box::new(move |cx, r| {
                let c = r.center();
                let outer = d * 0.5 * squash;
                // The photo shrinks inside the progress ring while uploading.
                let rad = outer * shrink;
                if show_photo {
                    let pr = Rect::from_center_size(c, vec2(rad * 2.0, rad * 2.0));
                    cx.painter.rrect_gradient(pr, Corners::all(rad), photo_top, photo);
                    let g = cx.galley(&initials, &st_init);
                    // Initials step back while the progress reads on top.
                    let ia = if kind == 1 { 0.3 } else { 1.0 - 0.6 * scrim_a };
                    cx.painter.text(cx.text, &g, vec2((c.x - g.width * 0.5).round(), (c.y - g.height * 0.5).round()), Color::WHITE.multiply_alpha(ia));
                    if scrim_a > 0.01 {
                        cx.painter.circle(c, rad, scrim.multiply_alpha(0.55 * scrim_a), 0.0, Color::TRANSPARENT);
                    }
                    if kind == 2 && scrim_a > 0.01 {
                        cx.icon("camera", c - vec2(0.0, lift * lift_px), icon_sz, Color::WHITE.multiply_alpha(scrim_a));
                    }
                } else {
                    cx.painter.circle(c, rad, muted_bg, 0.0, Color::TRANSPARENT);
                    // A dashed ring that turns a little on hover.
                    let dashes = 20;
                    for k in 0..dashes {
                        let a0 = (k as f32 + spin * 0.5) / dashes as f32 * TAU;
                        cx.painter.arc_band(c, rad - band * 0.4, rad, a0, a0 + TAU / dashes as f32 * 0.55, dash_c);
                    }
                    let icon = if kind == 3 { "image-plus" } else { "camera" };
                    cx.icon(icon, c - vec2(0.0, lift * lift_px), icon_sz, icon_c);
                }
                if kind == 1 || ring_t > 0.01 {
                    cx.painter.arc_band(c, outer - band, outer, 0.0, TAU, track.multiply_alpha(ring_t));
                    if progress > 0.002 {
                        cx.painter.arc_band(c, outer - band, outer, 0.0, TAU * progress, accent.multiply_alpha(ring_t));
                    }
                }
                if kind == 1 {
                    let g = cx.galley(&pct, &st_pct);
                    cx.painter.text(cx.text, &g, vec2((c.x - g.width * 0.5).round(), (c.y - g.height * 0.5).round()), Color::WHITE);
                }
                let corner = c + vec2(outer, outer) * std::f32::consts::FRAC_1_SQRT_2;
                let br = d * 0.14;
                if badge > 0.01 {
                    let s = badge.max(0.0);
                    cx.painter.circle(corner, (br + edge_w) * s, surface, 0.0, Color::TRANSPARENT);
                    cx.painter.circle(corner, br * s, ok, 0.0, Color::TRANSPARENT);
                    cx.icon("check", corner, br * 1.3 * s, Color::WHITE);
                } else if kind == 3 {
                    cx.painter.circle(corner, br + edge_w, surface, 0.0, Color::TRANSPARENT);
                    cx.painter.circle(corner, br, danger, 0.0, Color::TRANSPARENT);
                    cx.icon("x", corner, br * 1.3, Color::WHITE);
                }
                if ra > 0.0 {
                    cx.defer(r, Box::new(move |cx, rr| cx.focus_ring(rr, rr.width() * 0.5, ra, rg, ring)));
                }
            })),
        );
        let (role_label, value) = match &state {
            UploadState::Empty => ("Upload photo".to_owned(), "No photo".to_owned()),
            UploadState::Uploading(p) => ("Photo".to_owned(), format!("Uploading, {:.0}%", p * 100.0)),
            UploadState::Done => ("Change photo".to_owned(), "Photo set".to_owned()),
            UploadState::Failed(e) => ("Retry upload".to_owned(), format!("Upload failed: {e}")),
        };
        if kind == 1 {
            let p = match state {
                UploadState::Uploading(p) => p.clamp(0.0, 1.0) as f64,
                _ => 0.0,
            };
            ui.a11y(id, || A11y::new(Role::ProgressIndicator).label(role_label).numeric(p * 100.0, 0.0, 100.0, None).value(value));
        } else {
            ui.a11y(id, || A11y::new(Role::Button).label(role_label).value(value));
        }

        // Title, hint (or error) and actions.
        ui.begin(Style { dir: Dir::Column, width: Size::FILL, gap: lk.control.gap[0].max(4.0), ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
        if !self.title.is_empty() {
            let st = ui.text_style(TextRole::BodyStrong);
            let fg = ui.color(Token::Foreground);
            ui.text_leaf(self.title.clone(), st, fg, false, true, Style { width: Size::FILL, ..Default::default() });
        }
        let (hint, hint_c) = match &state {
            UploadState::Failed(e) => (e.clone(), danger),
            UploadState::Uploading(_) => ("Uploading…".to_owned(), ui.color(Token::TextSecondary)),
            _ => (self.hint.clone(), ui.color(Token::TextSecondary)),
        };
        if !hint.is_empty() {
            let st = ui.text_style(TextRole::Caption);
            let n = ui.text_leaf(hint, st, hint_c, true, false, Style { width: Size::FILL, ..Default::default() });
            if kind == 3 {
                ui.a11y_node(n, || A11y::new(Role::Alert));
            }
        }
        let sp = ui.make_id(("acts", id));
        ui.scope(sp, |ui| {
            ui.begin(Style { dir: Dir::Row, gap: lk.control.gap[0].max(4.0), wrap: true, align: Align::Center, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
            match kind {
                0 => out.choose |= ui.button("Upload photo").icon("upload").small().show().clicked,
                1 => out.remove |= ui.button("Cancel").ghost().small().show().clicked,
                2 => {
                    out.choose |= ui.button("Change").small().show().clicked;
                    out.remove |= ui.button("Remove").ghost().small().show().clicked;
                }
                _ => {
                    out.retry |= ui.button("Retry").icon("refresh-cw").small().show().clicked;
                    out.remove |= ui.button("Remove").ghost().small().show().clicked;
                }
            }
            ui.end();
        });
        ui.end();
        ui.end();
        out.response = Response { id, rect: ui.rect_of(id).unwrap_or(ix.rect), hovered: ix.hovered, clicked: out.choose || out.retry, changed: out.choose || out.remove || out.retry, focused: ix.focused };
        out
    }
}

impl Ui {
    /// A round photo slot with Upload, Change and Remove. Call `.show()`.
    pub fn avatar_upload<'u>(&'u mut self, key: impl Into<String>, state: &'u UploadState) -> AvatarUpload<'u> {
        AvatarUpload { ui: self, key: key.into(), state, initials: String::new(), tone: 0, title: String::new(), hint: String::new(), size: None }
    }
}
