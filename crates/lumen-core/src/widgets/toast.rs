//! Toasts: short notices that stack in a corner and leave on their own.

use crate::id::Id;
use crate::input::CursorIcon;
use crate::layout::{Align, Dir, Justify, Size, Style};
use crate::math::{vec2, Corners, Sides};
use crate::theme::{Elevation, TextRole, Token};
use crate::ui::{LayerPlace, Sense, Ui};
use crate::Color;

/// Tone of a toast, badge or alert.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub enum Tone {
    #[default]
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
    Info,
}

impl Tone {
    /// (subtle background, strong foreground, solid) tokens.
    pub fn tokens(self) -> (Token, Token, Token) {
        match self {
            Tone::Neutral => (Token::SurfaceMuted, Token::TextSecondary, Token::TextSecondary),
            Tone::Accent => (Token::AccentSubtle, Token::AccentStrong, Token::Accent),
            Tone::Success => (Token::SuccessSubtle, Token::SuccessStrong, Token::Success),
            Tone::Warning => (Token::WarningSubtle, Token::WarningStrong, Token::Warning),
            Tone::Danger => (Token::DangerSubtle, Token::DangerStrong, Token::Danger),
            Tone::Info => (Token::InfoSubtle, Token::InfoStrong, Token::Info),
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            Tone::Neutral | Tone::Accent => "bell",
            Tone::Success => "circle-check",
            Tone::Warning => "triangle-alert",
            Tone::Danger => "circle-x",
            Tone::Info => "info",
        }
    }
}

/// A toast notice. `ui.toast(Toast::new("Saved").tone(Tone::Success))`.
#[derive(Clone, Debug)]
pub struct Toast {
    pub title: String,
    pub description: Option<String>,
    pub tone: Tone,
    /// Seconds on screen (0 = until dismissed).
    pub duration: f32,
}

impl Toast {
    pub fn new(title: impl Into<String>) -> Self {
        Toast { title: title.into(), description: None, tone: Tone::Neutral, duration: 4.0 }
    }
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    pub fn tone(mut self, t: Tone) -> Self {
        self.tone = t;
        self
    }
    pub fn success(self) -> Self {
        self.tone(Tone::Success)
    }
    pub fn danger(self) -> Self {
        self.tone(Tone::Danger)
    }
    pub fn duration(mut self, s: f32) -> Self {
        self.duration = s;
        self
    }
}

pub(crate) struct ToastState {
    id: Id,
    toast: Toast,
    created: f64,
    closing: Option<f64>,
}

impl Ui {
    /// Show a toast in the bottom-right corner.
    pub fn toast(&mut self, t: Toast) {
        let n = self.time();
        let id = Id::new(("lumen-toast", n.to_bits(), self.toasts.len()));
        self.toasts.push(ToastState { id, toast: t, created: n, closing: None });
    }

    pub(crate) fn build_toasts(&mut self) {
        let now = self.time();
        let fade = self.motion().fade_out as f64 * 2.0;
        self.toasts.retain(|t| t.closing.is_none_or(|c| now - c < fade));
        if self.toasts.is_empty() {
            return;
        }
        for t in &mut self.toasts {
            if t.closing.is_none() && t.toast.duration > 0.0 && now - t.created > t.toast.duration as f64 {
                t.closing = Some(now);
            }
        }
        let toasts: Vec<(Id, Toast, Option<f64>)> = self.toasts.iter().map(|t| (t.id, t.toast.clone(), t.closing)).collect();
        let mut close = vec![];
        self.layer(LayerPlace::Screen, vec2(0.0, 0.0), 1.0, None, |ui| {
            let root = ui.layer_root_building();
            if let Some(r) = root {
                let st = ui.node_style(r);
                st.justify = Justify::End;
                st.align = Align::End;
                st.pad = Sides::all(20.0);
                st.gap = 10.0;
            }
            let mo = ui.motion();
            for (id, t, closing) in toasts {
                let on = closing.is_none();
                let a = ui.fade(id, 0, on, mo.layer_in * 1.5, mo.fade_out * 2.0, crate::anim::Ease::Out);
                let x = ui.anims.spring_from(id, 1, 32.0, if on { 0.0 } else { 16.0 }, mo.bouncy);
                let (_, strong, _) = t.tone.tokens();
                let ic = ui.color(strong);
                let fill = ui.color(Token::SurfaceRaised);
                let border = ui.color(Token::Border);
                let shadows = Elevation::Level3.shadows(ui.theme.darkness(), ui.color(Token::ShadowInk));
                let radius = ui.look().card.radius * 0.85;
                let fill_a = fill.multiply_alpha(a);
                let border_a = border.multiply_alpha(a);
                let shadows: Vec<_> = shadows.into_iter().map(|mut s| { s.color = s.color.multiply_alpha(a * 0.8); s }).collect();
                ui.begin(
                    Style { dir: Dir::Row, width: Size::Fixed(340.0), pad: Sides::all(14.0), gap: 12.0, align: Align::Start, ..Default::default() },
                    Some(id),
                    Sense::HOVER,
                    CursorIcon::Default,
                    false,
                    Some(Box::new(move |cx, r| {
                        let r = r.translate(vec2(x, 0.0));
                        cx.painter.shadows(r, Corners::all(radius), &shadows);
                        cx.painter.rrect_unsnapped(r, Corners::all(radius), [fill_a; 2], 1.0, border_a);
                    })),
                );
                let ic_a = ic.multiply_alpha(a);
                ui.leaf(Style::default(), vec2(18.0, 20.0), None, Sense::NONE, CursorIcon::Default, Some(Box::new(move |cx, r| {
                    cx.icon(t.tone.icon(), r.center() + vec2(x, 0.0), 18.0, ic_a);
                })));
                ui.begin(Style { width: Size::FILL, gap: 2.0, ..Default::default() }, None, Sense::NONE, CursorIcon::Default, false, None);
                let st = ui.text_style(TextRole::BodyStrong);
                let fg = ui.color(Token::Foreground).multiply_alpha(a);
                let n = ui.text_leaf(t.title.clone(), st, fg, true, false, Style::default());
                ui.node_style(n).margin_top = 0.0;
                if let Some(d) = &t.description {
                    let st = ui.text_style(TextRole::Body);
                    let c = ui.color(Token::TextSecondary).multiply_alpha(a);
                    ui.text_leaf(d.clone(), st, c, true, false, Style::default());
                }
                ui.end();
                let cid = id.with("close");
                let cix = ui.interact(cid, Sense::CLICK, false);
                if cix.clicked {
                    close.push(id);
                }
                let hc = ui.fade(cid, 0, cix.hovered, 0.0, 0.1, crate::anim::Ease::Linear);
                let xc = ui.color(Token::TextMuted).mix(ui.color(Token::Foreground), hc).multiply_alpha(a);
                let hov = ui.color(Token::Hover).multiply_alpha(hc * a);
                ui.leaf(Style::default(), vec2(20.0, 20.0), Some(cid), Sense::CLICK, CursorIcon::Pointer, Some(Box::new(move |cx, r| {
                    let r = r.translate(vec2(x, 0.0));
                    cx.painter.rrect_unsnapped(r, Corners::all(5.0), [hov; 2], 0.0, Color::TRANSPARENT);
                    cx.icon("x", r.center(), 14.0, xc);
                })));
                ui.end();
                if a < 1.0 || x.abs() > 0.01 {
                    ui.request_repaint_after(0.0);
                }
            }
        });
        let mut next: Option<f64> = None;
        for t in &mut self.toasts {
            if close.contains(&t.id) && t.closing.is_none() {
                t.closing = Some(now);
            }
            if t.closing.is_none() && t.toast.duration > 0.0 {
                let left = t.toast.duration as f64 - (now - t.created);
                if left > 0.0 {
                    next = Some(next.map_or(left, |n: f64| n.min(left)));
                }
            }
        }
        if let Some(n) = next {
            self.repaint_after_secs(n);
        }
    }
}
