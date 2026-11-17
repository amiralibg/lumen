//! Design tokens (DESIGN.md) and the animated theme.
//!
//! Widgets only ever read colors through [`Token`]; the theme resolves each
//! token for the current mode and accent, and blends every token in OKLab
//! when either changes.

use crate::anim::{Ease, SpringParams};
use crate::color::Color;
use crate::look::{Lerp, Look, Neutrals};
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
    Control, ControlBorder, ControlEdge,
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

/// The accent color: eight tuned presets, any custom color, or the OS one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Accent {
    Neutral,
    Violet,
    #[default]
    Blue,
    Green,
    Amber,
    Orange,
    Coral,
    Rose,
    /// Any color as `0xRRGGBB`; hover, strong, subtle and foreground
    /// shades are derived from it.
    Custom(u32),
    /// The OS accent when the app layer reports one, else the look's own
    /// accent (system blue on macOS, Windows and GNOME; violet for lumen).
    Native,
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
            Accent::Custom(_) => "Custom",
            Accent::Native => "System",
        }
    }

    /// A custom accent from OKLCH hue (degrees), at a lightness and chroma
    /// that work as a button fill.
    pub fn from_hue(hue: f32) -> Accent {
        Accent::Custom(Color::oklch(0.60, 0.17, hue).to_hex())
    }

    /// `[ACCENT, ACCENT_STRONG, ACCENT_SUBTLE, ACCENT_FOREGROUND]` (DESIGN.md 2.2).
    ///
    /// Custom and native accents are derived against lumen's neutrals here;
    /// the theme derives them against the current look's.
    pub fn colors(self, mode: Mode) -> [Color; 4] {
        let h = |a, b, c, d| [Color::hex(a), Color::hex(b), Color::hex(c), Color::hex(d)];
        match (mode, self) {
            (_, Accent::Custom(v)) => derive_accent(Color::hex(v), mode, Look::lumen().neutrals(mode), 0.0),
            (_, Accent::Native) => Accent::Blue.colors(mode),
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

/// `[ACCENT, ACCENT_STRONG, ACCENT_SUBTLE, ACCENT_FOREGROUND]` for an
/// arbitrary color against a set of neutrals.
pub fn derive_accent(c: Color, mode: Mode, n: &Neutrals, dark_min_l: f32) -> [Color; 4] {
    let light = mode == Mode::Light;
    let c = c.with_alpha(1.0);
    let mut a = c;
    if !light {
        let min_l = dark_min_l.max(0.45);
        if a.lightness() < min_l {
            a = a.shift_lightness(min_l - a.lightness());
        }
    }
    // White text when it reads (3:1, like the platforms), else a dark tint.
    let fg = if Color::WHITE.contrast(a) >= 3.0 { Color::WHITE } else { a.shift_lightness(0.20 - a.lightness()) };
    // Text-safe shade against the surface.
    let surface = n.surface;
    let mut strong = a;
    for _ in 0..30 {
        if strong.contrast(surface) >= 4.5 {
            break;
        }
        strong = strong.shift_lightness(if light { -0.02 } else { 0.02 });
    }
    let subtle = surface.mix(a, if light { 0.12 } else { 0.22 });
    [a, strong, subtle, fg]
}

/// Per-token color overrides for each mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Overrides {
    light: [Option<Color>; TOKEN_COUNT],
    dark: [Option<Color>; TOKEN_COUNT],
}

impl Default for Overrides {
    fn default() -> Self {
        Overrides { light: [None; TOKEN_COUNT], dark: [None; TOKEN_COUNT] }
    }
}

impl Overrides {
    pub fn set(&mut self, mode: Mode, t: Token, c: Option<Color>) {
        match mode {
            Mode::Light => self.light[t as usize] = c,
            Mode::Dark => self.dark[t as usize] = c,
        }
    }
    pub fn get(&self, mode: Mode, t: Token) -> Option<Color> {
        match mode {
            Mode::Light => self.light[t as usize],
            Mode::Dark => self.dark[t as usize],
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

    /// lumen's own palette for a mode and accent.
    pub fn new(mode: Mode, accent: Accent) -> Palette {
        Palette::build(mode, accent, &Look::lumen(), None, &Overrides::default())
    }

    /// The palette for a look. `native` is the OS accent (for
    /// [`Accent::Native`]).
    pub fn build(mode: Mode, accent: Accent, look: &Look, native: Option<Color>, overrides: &Overrides) -> Palette {
        use Token::*;
        let mut p = [Color::TRANSPARENT; TOKEN_COUNT];
        let mut set = |t: Token, c: Color| p[t as usize] = c;
        let light = mode == Mode::Light;
        let n = look.neutrals(mode);
        set(Background, n.background);
        set(Surface, n.surface);
        set(SurfaceRaised, n.surface_raised);
        set(SurfaceMuted, n.surface_muted);
        set(Track, n.track);
        set(InverseSurface, n.inverse_surface);
        set(InverseForeground, n.inverse_foreground);
        set(Foreground, n.foreground);
        set(TextSecondary, n.text_secondary);
        set(TextMuted, n.text_muted);
        set(TextDisabled, n.text_disabled);
        set(BorderSubtle, n.border_subtle);
        set(Border, n.border);
        set(BorderStrong, n.border_strong);
        set(Control, n.control);
        set(ControlBorder, n.control_border);
        set(ControlEdge, n.control_edge);
        set(Knob, Color::WHITE);
        let ink = n.ink;
        set(ShadowInk, ink);
        set(Hover, if light { ink.with_alpha(0.05) } else { Color::WHITE.with_alpha(0.06) });
        set(Pressed, if light { ink.with_alpha(0.09) } else { Color::WHITE.with_alpha(0.10) });
        set(Scrim, if light { ink.with_alpha(0.40) } else { Color::BLACK.with_alpha(0.60) });

        let mi = if light { 0 } else { 1 };
        let derive = |c: Color| derive_accent(c, mode, n, look.dark_accent_min_lightness);
        let [a, a_strong, a_subtle, a_fg] = match accent {
            self::Accent::Custom(v) => derive(Color::hex(v)),
            self::Accent::Native => match native {
                Some(c) => derive(c),
                None if look.platform == crate::look::Platform::Lumen => self::Accent::Blue.colors(mode),
                None => derive(look.accent[mi]),
            },
            named if look.platform == crate::look::Platform::Lumen => named.colors(mode),
            // Named accents on platform looks keep their hue, re-derived
            // against the look's neutrals.
            named => {
                let tuned = named.colors(mode);
                if named == self::Accent::Neutral {
                    let fg = n.foreground;
                    [fg, fg, n.surface_muted, n.surface]
                } else {
                    let mut d = derive(tuned[0]);
                    d[3] = tuned[3];
                    d
                }
            }
        };
        set(Accent, a);
        set(AccentStrong, a_strong);
        set(AccentSubtle, a_subtle);
        set(AccentForeground, a_fg);
        let (ah, ap) = filled_states(a);
        set(AccentHover, ah);
        set(AccentPressed, ap);

        let focus = match n.focus {
            Some(f) => f,
            None if accent == self::Accent::Neutral => self::Accent::Blue.colors(mode)[1],
            None => a_strong,
        };
        let focus_alpha = look.focus.alpha;
        set(FocusRing, focus.with_alpha(focus_alpha));
        set(FocusHalo, a_strong.with_alpha(if light { 0.24 } else { 0.32 }));
        set(Selection, a_strong.with_alpha(if light { 0.22 } else { 0.36 }));

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
        if look.platform != crate::look::Platform::Lumen {
            p[DangerSubtle as usize] = n.surface.mix(Color::hex(d[0]), if light { 0.10 } else { 0.20 });
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
        for (i, c) in p.iter_mut().enumerate() {
            if let Some(o) = overrides.get(mode, Token::ALL[i]) {
                *c = o;
            }
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

/// Live theme: mode, accent and look, plus an in-flight blend between
/// what was on screen and the new target. Colors blend per token in OKLab;
/// the look's geometry and motion blend on the same timeline.
pub struct Theme {
    mode: Mode,
    accent: Accent,
    target: Look,
    native_accent: Option<Color>,
    overrides: Overrides,
    from: Palette,
    to: Palette,
    current: Palette,
    look_from: Look,
    look_current: Look,
    blending: bool,
    start: f64,
    duration: f32,
    /// 0 = light, 1 = dark, blended (used for shadow alpha).
    dark_from: f32,
    dark_t: f32,
    pub reduced_motion: bool,
    /// Global animation speed multiplier (2.0 = slow motion).
    pub motion_scale: f32,
}

impl Theme {
    pub fn new(mode: Mode, accent: Accent) -> Self {
        Self::with_look(mode, accent, Look::lumen())
    }

    pub fn with_look(mode: Mode, accent: Accent, look: Look) -> Self {
        let overrides = Overrides::default();
        let p = Palette::build(mode, accent, &look, None, &overrides);
        let d = if mode == Mode::Dark { 1.0 } else { 0.0 };
        Self {
            mode,
            accent,
            target: look,
            native_accent: None,
            overrides,
            from: p,
            to: p,
            current: p,
            look_from: look,
            look_current: look,
            blending: false,
            start: 0.0,
            duration: 0.0,
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
    /// The target look (what [`Ui::set_look`](crate::Ui::set_look) set).
    pub fn target_look(&self) -> &Look {
        &self.target
    }
    /// The look on screen right now, blended while a change animates.
    pub fn look(&self) -> &Look {
        &self.look_current
    }
    pub fn native_accent(&self) -> Option<Color> {
        self.native_accent
    }
    pub fn overrides(&self) -> &Overrides {
        &self.overrides
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
        self.blending
    }

    fn retarget(&mut self, now: f64, dur: f32) {
        // Start from what is on screen right now.
        self.from = self.current;
        self.look_from = self.look_current;
        self.dark_from = self.dark_t;
        self.to = Palette::build(self.mode, self.accent, &self.target, self.native_accent, &self.overrides);
        self.start = now;
        self.duration = if self.reduced_motion { 0.1 } else { dur * self.motion_scale };
        self.blending = true;
    }

    pub(crate) fn set(&mut self, mode: Mode, accent: Accent, now: f64) {
        if mode == self.mode && accent == self.accent {
            return;
        }
        let blend = self.target.motion.theme_blend;
        let dur = if mode != self.mode { blend } else { blend * (2.0 / 3.0) };
        self.mode = mode;
        self.accent = accent;
        self.retarget(now, dur);
    }

    pub(crate) fn set_look(&mut self, look: Look, now: f64) {
        if look == self.target {
            return;
        }
        self.target = look;
        self.retarget(now, look.motion.theme_blend * 1.5);
    }

    pub(crate) fn set_native_accent(&mut self, c: Option<Color>, now: f64) {
        if c == self.native_accent {
            return;
        }
        self.native_accent = c;
        if self.accent == Accent::Native {
            self.retarget(now, self.target.motion.theme_blend);
        }
    }

    pub(crate) fn set_override(&mut self, mode: Mode, t: Token, c: Option<Color>, now: f64) {
        if self.overrides.get(mode, t) == c {
            return;
        }
        self.overrides.set(mode, t, c);
        self.retarget(now, self.target.motion.theme_blend * (2.0 / 3.0));
    }

    /// Jump to the end of any blend in flight.
    pub fn settle(&mut self) {
        self.update(f64::INFINITY);
    }

    pub(crate) fn update(&mut self, now: f64) {
        if !self.blending {
            return;
        }
        let t = if self.duration <= 0.0 { 1.0 } else { ((now - self.start) as f32 / self.duration).clamp(0.0, 1.0) };
        let e = if self.reduced_motion { t } else { Ease::Standard.apply(t) };
        let target_dark = if self.mode == Mode::Dark { 1.0 } else { 0.0 };
        self.dark_t = self.dark_from + (target_dark - self.dark_from) * e;
        if t >= 1.0 {
            self.current = self.to;
            self.from = self.to;
            self.look_current = self.target;
            self.look_from = self.target;
            self.dark_t = target_dark;
            self.blending = false;
        } else {
            self.current = self.from.mix(&self.to, e);
            self.look_current = self.look_from.lerp(self.target, e);
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
    fn look_change_blends_geometry_and_colors() {
        let mut th = Theme::new(Mode::Light, Accent::Native);
        th.set_look(Look::windows(), 0.0);
        th.update(0.2);
        let r = th.look().card.radius;
        assert!(r < 12.0 && r > 8.0, "{r}");
        th.update(5.0);
        assert!(!th.is_animating());
        assert_eq!(th.look().card.radius, 8.0);
        assert_eq!(th.color(Token::Background).to_hex(), 0xF3F3F3);
        assert_eq!(th.color(Token::Accent).to_hex(), 0x005FB8);
    }

    #[test]
    fn native_accent_follows_os() {
        let mut th = Theme::with_look(Mode::Light, Accent::Native, Look::mac());
        assert_eq!(th.color(Token::Accent).to_hex(), 0x007AFF);
        th.set_native_accent(Some(Color::hex(0xE0383E)), 0.0);
        th.update(5.0);
        assert_eq!(th.color(Token::Accent).to_hex(), 0xE0383E);
    }

    #[test]
    fn derived_accents_are_readable() {
        for hue in (0..360).step_by(15) {
            for mode in [Mode::Light, Mode::Dark] {
                for look in [Look::lumen(), Look::mac(), Look::windows(), Look::gnome()] {
                    let p = Palette::build(mode, Accent::from_hue(hue as f32), &look, None, &Overrides::default());
                    let fg = p.get(Token::AccentForeground);
                    let a = p.get(Token::Accent);
                    assert!(fg.contrast(a) >= 3.0, "hue {hue} {mode:?} fg on accent {}", fg.contrast(a));
                    let s = p.get(Token::AccentStrong);
                    assert!(s.contrast(p.get(Token::Surface)) >= 4.4, "hue {hue} {mode:?} strong {}", s.contrast(p.get(Token::Surface)));
                }
            }
        }
    }

    #[test]
    fn overrides_apply() {
        let mut o = Overrides::default();
        o.set(Mode::Dark, Token::Background, Some(Color::hex(0x101820)));
        let p = Palette::build(Mode::Dark, Accent::Blue, &Look::lumen(), None, &o);
        assert_eq!(p.get(Token::Background).to_hex(), 0x101820);
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
