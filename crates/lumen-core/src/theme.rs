//! Design tokens (DESIGN.md) and the animated theme.
//!
//! Widgets only ever read colors through [`Token`]; the theme resolves each
//! token for the current mode and accent, and blends every token in OKLab
//! when either changes.

use crate::anim::{Ease, SpringParams};
use crate::color::Color;
use lumen_text::{FontFamily, TextStyle};

macro_rules! tokens {
    ($($name:ident),* $(,)?) => {
        /// A named theme color. See DESIGN.md section 2.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[repr(u8)]
        pub enum Token { $($name),* }
        impl Token {
            pub const ALL: &'static [Token] = &[$(Token::$name),*];
        }
        pub(crate) const TOKEN_COUNT: usize = [$(Token::$name),*].len();
    };
}

tokens!(
    Background, Surface, SurfaceRaised, SurfaceMuted, Track, InverseSurface, InverseForeground,
    Foreground, TextSecondary, TextMuted, TextDisabled,
    BorderSubtle, Border, BorderStrong, Knob,
    Hover, Pressed, Scrim, FocusRing, FocusHalo, Selection, ShadowInk,
    Accent, AccentStrong, AccentSubtle, AccentForeground, AccentHover, AccentPressed,
    Success, SuccessStrong, SuccessSubtle, SuccessForeground,
    Warning, WarningStrong, WarningSubtle, WarningForeground,
    Info, InfoStrong, InfoSubtle, InfoForeground,
    Danger, DangerStrong, DangerSubtle, DangerForeground, DangerHover, DangerPressed,
    Series0, Series1, Series2, Series3, Series4, Series5,
);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    #[default]
    Light,
    Dark,
}

impl Mode {
    pub fn toggled(self) -> Mode {
        match self {
            Mode::Light => Mode::Dark,
            Mode::Dark => Mode::Light,
        }
    }
}

/// The eight accent colors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Accent {
    Neutral,
    #[default]
    Violet,
    Blue,
    Green,
    Amber,
    Orange,
    Coral,
    Rose,
}

impl Accent {
    pub const ALL: [Accent; 8] =
        [Accent::Neutral, Accent::Violet, Accent::Blue, Accent::Green, Accent::Amber, Accent::Orange, Accent::Coral, Accent::Rose];

    pub fn name(self) -> &'static str {
        match self {
            Accent::Neutral => "Neutral",
            Accent::Violet => "Violet",
            Accent::Blue => "Blue",
            Accent::Green => "Green",
            Accent::Amber => "Amber",
            Accent::Orange => "Orange",
            Accent::Coral => "Coral",
            Accent::Rose => "Rose",
        }
    }

    /// `[ACCENT, ACCENT_STRONG, ACCENT_SUBTLE, ACCENT_FOREGROUND]` (DESIGN.md 2.2).
    pub fn colors(self, mode: Mode) -> [Color; 4] {
        let h = |a, b, c, d| [Color::hex(a), Color::hex(b), Color::hex(c), Color::hex(d)];
        match (mode, self) {
            (Mode::Light, Accent::Neutral) => h(0x14171D, 0x14171D, 0xE8E9EC, 0xFFFFFF),
            (Mode::Light, Accent::Violet) => h(0x7D55EC, 0x6849C4, 0xF0EFFE, 0xFFFFFF),
            (Mode::Light, Accent::Blue) => h(0x0E6EE6, 0x125DC0, 0xE9F2FF, 0xFFFFFF),
            (Mode::Light, Accent::Green) => h(0x02844B, 0x01703F, 0xE3F7E9, 0xFFFFFF),
            (Mode::Light, Accent::Amber) => h(0xF5AF20, 0x865600, 0xFCF0D3, 0x261700),
            (Mode::Light, Accent::Orange) => h(0xF77C10, 0xA04500, 0xFFEDE2, 0x2B1405),
            (Mode::Light, Accent::Coral) => h(0xF46C5F, 0xAA3838, 0xFFECE9, 0x2C120E),
            (Mode::Light, Accent::Rose) => h(0xDB1E69, 0xB61F58, 0xFFEBEF, 0xFFFFFF),
            (Mode::Dark, Accent::Neutral) => h(0xEFF0F3, 0xEFF0F3, 0x282A2F, 0x121418),
            (Mode::Dark, Accent::Violet) => h(0x7F58EE, 0xB0A3FB, 0x2A2543, 0xFFFFFF),
            (Mode::Dark, Accent::Blue) => h(0x1270E9, 0x7CB0FD, 0x182A45, 0xFFFFFF),
            (Mode::Dark, Accent::Green) => h(0x00864B, 0x64C489, 0x0F321E, 0xFFFFFF),
            (Mode::Dark, Accent::Amber) => h(0xFCB52C, 0xDBA549, 0x36270E, 0x261700),
            (Mode::Dark, Accent::Orange) => h(0xFE821E, 0xEF9963, 0x3C2314, 0x2B1405),
            (Mode::Dark, Accent::Coral) => h(0xFB7365, 0xF79285, 0x3E211D, 0x2C120E),
            (Mode::Dark, Accent::Rose) => h(0xDD226C, 0xF48FA7, 0x401E26, 0xFFFFFF),
        }
    }
}

/// Hover and pressed variants of a solid fill (DESIGN.md 2.3).
pub fn filled_states(fill: Color) -> (Color, Color) {
    if fill.lightness() < 0.30 {
        (fill.shift_lightness(0.07), fill.shift_lightness(0.12))
    } else {
        (fill.shift_lightness(-0.035), fill.shift_lightness(-0.07))
    }
}

/// A full set of resolved token colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette(pub [Color; TOKEN_COUNT]);

impl Palette {
    pub fn get(&self, t: Token) -> Color {
        self.0[t as usize]
    }

    pub fn new(mode: Mode, accent: Accent) -> Palette {
        use Token::*;
        let mut p = [Color::TRANSPARENT; TOKEN_COUNT];
        let mut set = |t: Token, c: Color| p[t as usize] = c;
        let light = mode == Mode::Light;
        let pick = |l: u32, d: u32| Color::hex(if light { l } else { d });
        set(Background, pick(0xF9FAFB, 0x0B0C0F));
        set(Surface, pick(0xFFFFFF, 0x121417));
        set(SurfaceRaised, pick(0xFFFFFF, 0x1A1C20));
        set(SurfaceMuted, pick(0xF3F4F6, 0x1F2024));
        set(Track, pick(0xD4D6DA, 0x35383D));
        set(InverseSurface, pick(0x191C21, 0xE6E8EB));
        set(InverseForeground, pick(0xF4F5F7, 0x14171D));
        set(Foreground, pick(0x14171D, 0xEFF0F3));
        set(TextSecondary, pick(0x51555D, 0xA8ABB1));
        set(TextMuted, pick(0x686C74, 0x888C94));
        set(TextDisabled, pick(0xA1A5AB, 0x4D5055));
        set(BorderSubtle, pick(0xEAEBEE, 0x212327));
        set(Border, pick(0xDEE0E3, 0x2A2D31));
        set(BorderStrong, pick(0xC2C4C9, 0x3E4147));
        set(Knob, Color::WHITE);
        let ink = pick(0x0F121C, 0x000000);
        set(ShadowInk, ink);
        set(Hover, if light { ink.with_alpha(0.05) } else { Color::WHITE.with_alpha(0.06) });
        set(Pressed, if light { ink.with_alpha(0.09) } else { Color::WHITE.with_alpha(0.10) });
        set(Scrim, if light { ink.with_alpha(0.40) } else { Color::BLACK.with_alpha(0.60) });

        let [a, a_strong, a_subtle, a_fg] = accent.colors(mode);
        set(Accent, a);
        set(AccentStrong, a_strong);
        set(AccentSubtle, a_subtle);
        set(AccentForeground, a_fg);
        let (ah, ap) = filled_states(a);
        set(AccentHover, ah);
        set(AccentPressed, ap);

        let focus = if accent == self::Accent::Neutral { self::Accent::Blue.colors(mode)[1] } else { a_strong };
        set(FocusRing, focus);
        set(FocusHalo, focus.with_alpha(if light { 0.24 } else { 0.32 }));
        set(Selection, focus.with_alpha(if light { 0.22 } else { 0.36 }));

        let status = |acc: self::Accent| acc.colors(mode);
        for (base, acc) in [(Success, self::Accent::Green), (Warning, self::Accent::Amber), (Info, self::Accent::Blue)] {
            let c = status(acc);
            for i in 0..4 {
                p[base as usize + i] = c[i];
            }
        }
        let d = if light {
            [0xDD2729, 0xB82524, 0xFFECE9, 0xFFFFFF]
        } else {
            [0xE02B2B, 0xF69186, 0x411E1B, 0xFFFFFF]
        };
        for (i, v) in d.iter().enumerate() {
            p[Danger as usize + i] = Color::hex(*v);
        }
        let (dh, dp) = filled_states(Color::hex(d[0]));
        p[DangerHover as usize] = dh;
        p[DangerPressed as usize] = dp;

        let series = if light {
            [0x297CEF, 0xC56001, 0x009863, 0x8B61E3, 0xDA3870, 0x0090A8]
        } else {
            [0x68A5FF, 0xEA8846, 0x58BB8B, 0xAD8DFE, 0xF97096, 0x4AB5CC]
        };
        for (i, v) in series.iter().enumerate() {
            p[Series0 as usize + i] = Color::hex(*v);
        }
        Palette(p)
    }

    /// Per-token OKLab blend.
    pub fn mix(&self, other: &Palette, t: f32) -> Palette {
        let mut out = self.0;
        for (i, c) in out.iter_mut().enumerate() {
            *c = c.mix(other.0[i], t);
        }
        Palette(out)
    }
}

/// Text roles (DESIGN.md 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextRole {
    Caption,
    Label,
    Body,
    BodyStrong,
    Heading4,
    Heading3,
    Heading2,
    Heading1,
    Display,
    Mono,
}

impl TextRole {
    pub fn style(self) -> TextStyle {
        let s = |family, weight, size, line_height, tracking| TextStyle { family, weight, size, line_height, tracking, tabular: false };
        use FontFamily::*;
        match self {
            TextRole::Caption => s(Sans, 400, 11.0, 16.0, 0.005),
            TextRole::Label => s(Sans, 500, 12.0, 16.0, 0.0),
            TextRole::Body => s(Sans, 400, 13.0, 20.0, 0.0),
            TextRole::BodyStrong => s(Sans, 500, 13.0, 20.0, 0.0),
            TextRole::Heading4 => s(Sans, 600, 14.0, 20.0, -0.006),
            TextRole::Heading3 => s(Sans, 600, 16.0, 24.0, -0.011),
            TextRole::Heading2 => s(Sans, 600, 20.0, 28.0, -0.017),
            TextRole::Heading1 => s(Display, 600, 24.0, 32.0, -0.020),
            TextRole::Display => s(Display, 600, 32.0, 40.0, -0.025),
            TextRole::Mono => s(Mono, 400, 12.0, 20.0, 0.0),
        }
    }
}

/// One shadow layer: offset, CSS blur radius, spread, color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

/// Elevation levels for floating layers (DESIGN.md 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Elevation {
    Knob,
    Level1,
    Level2,
    Level3,
}

impl Elevation {
    /// Shadow layers back to front for the given mode, using the (blended)
    /// shadow ink color.
    pub fn shadows(self, mode_dark_t: f32, ink: Color) -> Vec<Shadow> {
        // Alphas are interpolated between modes so theme switching blends
        // shadows too.
        let sh = |x, y, blur, spread, la: f32, da: f32| Shadow {
            x,
            y,
            blur,
            spread,
            color: ink.with_alpha(la + (da - la) * mode_dark_t),
        };
        match self {
            Elevation::Knob => {
                // Light has two layers, dark one; blend by fading the extra layer.
                vec![sh(0.0, 1.0, 1.0, 0.0, 0.10, 0.0), sh(0.0, 2.0, 4.0, 0.0, 0.14, 0.0), sh(0.0, 1.0, 3.0, 0.0, 0.0, 0.45)]
            }
            Elevation::Level1 => vec![sh(0.0, 1.0, 2.0, 0.0, 0.06, 0.40), sh(0.0, 4.0, 10.0, -2.0, 0.10, 0.40)],
            Elevation::Level2 => vec![sh(0.0, 2.0, 4.0, -1.0, 0.06, 0.40), sh(0.0, 10.0 + 2.0 * mode_dark_t, 24.0 + 4.0 * mode_dark_t, -4.0, 0.12, 0.50)],
            Elevation::Level3 => vec![sh(0.0, 4.0, 8.0, -2.0, 0.06, 0.40), sh(0.0, 24.0 + 4.0 * mode_dark_t, 56.0 + 8.0 * mode_dark_t, -12.0, 0.22, 0.64)],
        }
    }
}

/// Named springs (DESIGN.md 7.1).
pub mod springs {
    use super::SpringParams;
    pub const SNAPPY: SpringParams = SpringParams { stiffness: 987.0, damping: 62.83 };
    pub const SMOOTH: SpringParams = SpringParams { stiffness: 322.3, damping: 35.90 };
    pub const BOUNCY: SpringParams = SpringParams { stiffness: 438.6, damping: 29.32 };
    pub const GENTLE: SpringParams = SpringParams { stiffness: 130.5, damping: 22.85 };
}

/// Durations in seconds (DESIGN.md 7.2).
pub mod durations {
    pub const THEME_BLEND: f32 = 0.300;
    pub const ACCENT_BLEND: f32 = 0.200;
    pub const HOVER_IN: f32 = 0.060;
    pub const HOVER_OUT: f32 = 0.160;
    pub const PRESSED_OUT: f32 = 0.160;
    pub const SELECTED: f32 = 0.120;
    pub const FADE_OUT: f32 = 0.100;
    pub const LAYER_IN: f32 = 0.120;
    pub const MARK_OUT: f32 = 0.080;
    pub const TOOLTIP_DELAY: f64 = 0.500;
    pub const TOOLTIP_OUT: f64 = 0.100;
    pub const TOOLTIP_WARM: f64 = 0.400;
}

/// Size class for controls (DESIGN.md 3.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ControlSize {
    Sm,
    #[default]
    Md,
    Lg,
}

pub struct ControlMetrics {
    pub height: f32,
    pub pad_x: f32,
    pub pad_icon: f32,
    pub icon: f32,
    pub gap: f32,
    pub radius: f32,
    pub text: TextStyle,
    pub inset: f32,
}

impl ControlSize {
    pub fn metrics(self) -> ControlMetrics {
        let mut t = TextRole::BodyStrong.style();
        match self {
            ControlSize::Sm => {
                t.size = 12.0;
                t.line_height = 16.0;
                ControlMetrics { height: 28.0, pad_x: 10.0, pad_icon: 8.0, icon: 14.0, gap: 6.0, radius: 6.0, text: t, inset: 1.0 }
            }
            ControlSize::Md => ControlMetrics { height: 32.0, pad_x: 12.0, pad_icon: 10.0, icon: 16.0, gap: 6.0, radius: 8.0, text: t, inset: 1.0 },
            ControlSize::Lg => {
                t.size = 14.0;
                ControlMetrics { height: 40.0, pad_x: 16.0, pad_icon: 14.0, icon: 18.0, gap: 8.0, radius: 10.0, text: t, inset: 1.5 }
            }
        }
    }
}

/// Live theme: the current palette, plus an in-flight blend.
pub struct Theme {
    mode: Mode,
    accent: Accent,
    from: Palette,
    to: Palette,
    /// Blend progress for the palette, 0..1.
    start: f64,
    duration: f32,
    current: Palette,
    /// 0 = light, 1 = dark, blended (used for shadow alpha).
    dark_from: f32,
    dark_t: f32,
    pub reduced_motion: bool,
    /// Global animation speed multiplier (2.0 = slow motion).
    pub motion_scale: f32,
}

impl Theme {
    pub fn new(mode: Mode, accent: Accent) -> Self {
        let p = Palette::new(mode, accent);
        let d = if mode == Mode::Dark { 1.0 } else { 0.0 };
        Self {
            mode,
            accent,
            from: p,
            to: p,
            start: 0.0,
            duration: 0.0,
            current: p,
            dark_from: d,
            dark_t: d,
            reduced_motion: false,
            motion_scale: 1.0,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }
    pub fn accent(&self) -> Accent {
        self.accent
    }

    /// Blended color for a token.
    pub fn color(&self, t: Token) -> Color {
        self.current.get(t)
    }

    pub fn palette(&self) -> &Palette {
        &self.current
    }

    /// 0 in light mode, 1 in dark mode, in between while blending.
    pub fn darkness(&self) -> f32 {
        self.dark_t
    }

    pub fn is_animating(&self) -> bool {
        self.from != self.to
    }

    pub(crate) fn set(&mut self, mode: Mode, accent: Accent, now: f64) {
        if mode == self.mode && accent == self.accent {
            return;
        }
        let dur = if mode != self.mode { durations::THEME_BLEND } else { durations::ACCENT_BLEND };
        self.mode = mode;
        self.accent = accent;
        // Start from what is on screen right now.
        self.from = self.current;
        self.dark_from = self.dark_t;
        self.to = Palette::new(mode, accent);
        self.start = now;
        self.duration = if self.reduced_motion { 0.1 } else { dur * self.motion_scale };
    }

    pub(crate) fn update(&mut self, now: f64) {
        if self.from == self.to {
            return;
        }
        let t = if self.duration <= 0.0 { 1.0 } else { ((now - self.start) as f32 / self.duration).clamp(0.0, 1.0) };
        let e = if self.reduced_motion { t } else { Ease::Standard.apply(t) };
        let target_dark = if self.mode == Mode::Dark { 1.0 } else { 0.0 };
        self.dark_t = self.dark_from + (target_dark - self.dark_from) * e;
        if t >= 1.0 {
            self.current = self.to;
            self.from = self.to;
            self.dark_t = target_dark;
        } else {
            self.current = self.from.mix(&self.to, e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Color, hex: u32) -> bool {
        let b = Color::hex(hex);
        let d = |x: f32, y: f32| ((x - y) * 255.0).abs();
        d(a.r, b.r) <= 3.0 && d(a.g, b.g) <= 3.0 && d(a.b, b.b) <= 3.0
    }

    #[test]
    fn hover_pressed_match_reference_table() {
        // DESIGN.md 2.3 reference results.
        let cases = [
            (Mode::Light, Accent::Neutral, 0x24272E, 0x30343B),
            (Mode::Light, Accent::Violet, 0x7349E0, 0x6A3DD4),
            (Mode::Light, Accent::Amber, 0xE9A403, 0xDB9901),
            (Mode::Dark, Accent::Neutral, 0xE3E4E7, 0xD8D9DC),
            (Mode::Dark, Accent::Blue, 0x0065DB, 0x005CC8),
            (Mode::Dark, Accent::Rose, 0xD00761, 0xC00058),
        ];
        for (m, a, h, p) in cases {
            let pal = Palette::new(m, a);
            assert!(close(pal.get(Token::AccentHover), h), "{m:?} {a:?} hover {:06X}", pal.get(Token::AccentHover).to_hex());
            assert!(close(pal.get(Token::AccentPressed), p), "{m:?} {a:?} pressed {:06X}", pal.get(Token::AccentPressed).to_hex());
        }
        let l = Palette::new(Mode::Light, Accent::Violet);
        assert!(close(l.get(Token::DangerHover), 0xD0121C));
    }

    #[test]
    fn neutral_focus_ring_falls_back_to_blue() {
        let p = Palette::new(Mode::Light, Accent::Neutral);
        assert_eq!(p.get(Token::FocusRing).to_hex(), 0x125DC0);
        let p = Palette::new(Mode::Dark, Accent::Violet);
        assert_eq!(p.get(Token::FocusRing).to_hex(), 0xB0A3FB);
    }

    #[test]
    fn theme_blend_animates_and_settles() {
        let mut th = Theme::new(Mode::Light, Accent::Violet);
        th.set(Mode::Dark, Accent::Violet, 0.0);
        th.update(0.15);
        let mid = th.color(Token::Background);
        assert!(th.is_animating());
        assert_ne!(mid.to_hex(), 0xF9FAFB);
        assert_ne!(mid.to_hex(), 0x0B0C0F);
        th.update(0.31);
        assert!(!th.is_animating());
        assert_eq!(th.color(Token::Background).to_hex(), 0x0B0C0F);
        assert_eq!(th.darkness(), 1.0);
    }

    #[test]
    fn retarget_mid_blend_starts_from_screen() {
        let mut th = Theme::new(Mode::Light, Accent::Violet);
        th.set(Mode::Dark, Accent::Violet, 0.0);
        th.update(0.15);
        let shown = th.color(Token::Background);
        th.set(Mode::Light, Accent::Violet, 0.15);
        th.update(0.15);
        assert_eq!(th.color(Token::Background).to_hex(), shown.to_hex());
    }
}
