//! Looks: the visual language the theme renders with.
//!
//! A [`Look`] holds everything about how lumen looks apart from the mode
//! and accent: the neutral colors, control sizes and radii, per-widget
//! shape parameters, the type scale, fonts and motion. Every widget reads
//! its geometry from the current look.
//!
//! Four presets ship: [`Look::lumen`] (the default), [`Look::mac`],
//! [`Look::windows`] and [`Look::gnome`], which follow those platforms'
//! own control conventions. [`Look::native`] picks the one for the OS the
//! app runs on. Every field is public, so a look can be tweaked or built
//! from scratch, and the builder helpers ([`rounded`](Look::rounded),
//! [`density`](Look::density), [`motion`](Look::motion), ...) cover the
//! common changes.
//!
//! Switching looks animates: numeric fields blend over the same timeline
//! as the colors, so corners, sizes and spacing morph from one look to the
//! next.

use crate::anim::SpringParams;
use crate::color::Color;

/// Linear blend, used to animate between looks.
pub trait Lerp: Copy {
    fn lerp(self, other: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(self, o: f32, t: f32) -> f32 {
        self + (o - self) * t
    }
}

impl<const N: usize> Lerp for [f32; N] {
    fn lerp(self, o: Self, t: f32) -> Self {
        let mut out = self;
        for (i, v) in out.iter_mut().enumerate() {
            *v = v.lerp(o[i], t);
        }
        out
    }
}

impl Lerp for SpringParams {
    fn lerp(self, o: Self, t: f32) -> Self {
        SpringParams { stiffness: self.stiffness.lerp(o.stiffness, t), damping: self.damping.lerp(o.damping, t) }
    }
}

macro_rules! lerp_struct {
    ($t:ident { $($f:ident),* $(,)? }) => {
        impl Lerp for $t {
            fn lerp(self, o: Self, t: f32) -> Self {
                $t { $($f: self.$f.lerp(o.$f, t)),* }
            }
        }
    };
}

/// Which platform's conventions a look follows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Platform {
    #[default]
    Lumen,
    Mac,
    Windows,
    Gnome,
}

impl Platform {
    pub const ALL: [Platform; 4] = [Platform::Lumen, Platform::Mac, Platform::Windows, Platform::Gnome];

    pub fn name(self) -> &'static str {
        match self {
            Platform::Lumen => "Lumen",
            Platform::Mac => "macOS",
            Platform::Windows => "Windows",
            Platform::Gnome => "GNOME",
        }
    }

    /// The platform this binary runs on (Lumen on the web and unknown OSes).
    pub fn current() -> Platform {
        if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
            Platform::Mac
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd")) {
            Platform::Gnome
        } else {
            Platform::Lumen
        }
    }

    pub fn look(self) -> Look {
        match self {
            Platform::Lumen => Look::lumen(),
            Platform::Mac => Look::mac(),
            Platform::Windows => Look::windows(),
            Platform::Gnome => Look::gnome(),
        }
    }
}

/// Neutral colors for one mode. Accent and status colors are derived
/// separately; see [`crate::theme::Palette`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Neutrals {
    pub background: Color,
    pub surface: Color,
    pub surface_raised: Color,
    pub surface_muted: Color,
    pub track: Color,
    pub inverse_surface: Color,
    pub inverse_foreground: Color,
    pub foreground: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_disabled: Color,
    pub border_subtle: Color,
    pub border: Color,
    pub border_strong: Color,
    /// Resting fill of secondary buttons and other neutral controls.
    pub control: Color,
    /// Outline of neutral controls (transparent for flat looks).
    pub control_border: Color,
    /// Bottom edge of neutral controls (Windows draws a darker one).
    pub control_edge: Color,
    /// Ink for shadows and hover/press overlays.
    pub ink: Color,
    /// Focus ring color; `None` uses the accent.
    pub focus: Option<Color>,
}

const fn h(v: u32) -> Color {
    Color::hex(v)
}

/// Size, radius, padding and text size for the three control sizes
/// (`[sm, md, lg]`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlLook {
    pub height: [f32; 3],
    pub radius: [f32; 3],
    pub pad_x: [f32; 3],
    pub text_size: [f32; 3],
    /// Gap between icon and label.
    pub gap: [f32; 3],
}
lerp_struct!(ControlLook { height, radius, pad_x, text_size, gap });

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonLook {
    /// How far the button shrinks while pressed (px).
    pub press_inset: f32,
    /// 0..1: dim the label while pressed (Windows).
    pub press_dim: f32,
    /// 0..1: vertical highlight gradient on filled buttons (macOS).
    pub bevel: f32,
    /// 0..1: small drop shadow under buttons (macOS).
    pub shadow: f32,
    /// 0..1: darker bottom edge on neutral buttons (Windows).
    pub bottom_edge: f32,
}
lerp_struct!(ButtonLook { press_inset, press_dim, bevel, shadow, bottom_edge });

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchLook {
    pub width: f32,
    pub height: f32,
    /// Knob diameter.
    pub knob: f32,
    /// Knob growth on hover (px).
    pub hover_grow: f32,
    /// Knob stretch while pressed (px).
    pub press_widen: f32,
    /// 0..1: the off track is an outline with a dark knob (Windows).
    pub outline_off: f32,
    /// 0..1: drop shadow under the knob.
    pub knob_shadow: f32,
    /// 0..1: knob takes the accent foreground when on (else stays white).
    pub knob_tint_on: f32,
}
lerp_struct!(SwitchLook { width, height, knob, hover_grow, press_widen, outline_off, knob_shadow, knob_tint_on });

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CheckboxLook {
    pub size: f32,
    pub radius: f32,
    pub border: f32,
}
lerp_struct!(CheckboxLook { size, radius, border });

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderLook {
    pub track: f32,
    pub thumb: f32,
    /// Thumb growth on hover (px).
    pub hover_grow: f32,
    /// 0..1: the thumb is a ring around an accent dot (Windows).
    pub inner_dot: f32,
    /// 0..1: translucent halo while dragging.
    pub halo: f32,
    /// 0..1: thin outline on the thumb (GNOME).
    pub thumb_border: f32,
}
lerp_struct!(SliderLook { track, thumb, hover_grow, inner_dot, halo, thumb_border });

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputLook {
    /// 0..1: filled with the control color instead of outlined (GNOME).
    pub filled: f32,
    /// 0..1: accent underline when focused (Windows).
    pub underline: f32,
    /// 0..1: soft accent halo when focused.
    pub halo: f32,
}
lerp_struct!(InputLook { filled, underline, halo });

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FocusLook {
    pub width: f32,
    /// Gap between the control and the ring at rest.
    pub gap: f32,
    /// Gap the ring animates in from.
    pub gap_from: f32,
    /// Ring opacity.
    pub alpha: f32,
}
lerp_struct!(FocusLook { width, gap, gap_from, alpha });

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardLook {
    pub radius: f32,
    /// 0..1: outline opacity.
    pub border: f32,
    /// 0..1: soft drop shadow (GNOME).
    pub shadow: f32,
    pub pad: f32,
    /// A 1px light edge along the top inside the card, like glass catching light.
    pub highlight: f32,
    /// A soft accent glow that follows the pointer across the card.
    pub glow: f32,
}
lerp_struct!(CardLook { radius, border, shadow, pad, highlight, glow });

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavLook {
    pub item_height: f32,
    pub radius: f32,
    /// 0..1: accent bar at the selected item's leading edge (Windows).
    pub indicator: f32,
    /// 0..1: accent-tinted selection (else a neutral gray one).
    pub accent_selection: f32,
}
lerp_struct!(NavLook { item_height, radius, indicator, accent_selection });

/// Springs and durations (seconds) every widget animates with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion {
    /// Fast, no overshoot: toggles, presses, focus.
    pub snappy: SpringParams,
    /// Slower, no overshoot: layout, sliding indicators.
    pub smooth: SpringParams,
    /// Slight overshoot: release, hover grow.
    pub bouncy: SpringParams,
    /// Slow and soft: programmatic scrolling.
    pub gentle: SpringParams,
    pub hover_in: f32,
    pub hover_out: f32,
    pub press_out: f32,
    pub selected: f32,
    pub fade_out: f32,
    pub layer_in: f32,
    pub mark_out: f32,
    /// Theme mode, accent and look changes.
    pub theme_blend: f32,
}
lerp_struct!(Motion { snappy, smooth, bouncy, gentle, hover_in, hover_out, press_out, selected, fade_out, layer_in, mark_out, theme_blend });

impl Motion {
    /// lumen's own motion (DESIGN.md 7).
    pub const fn lumen() -> Motion {
        Motion {
            snappy: SpringParams { stiffness: 987.0, damping: 62.83 },
            smooth: SpringParams { stiffness: 322.3, damping: 35.90 },
            bouncy: SpringParams { stiffness: 438.6, damping: 29.32 },
            gentle: SpringParams { stiffness: 130.5, damping: 22.85 },
            hover_in: 0.060,
            hover_out: 0.160,
            press_out: 0.160,
            selected: 0.120,
            fade_out: 0.100,
            layer_in: 0.120,
            mark_out: 0.080,
            theme_blend: 0.300,
        }
    }

    /// Soft springs with a little life, like AppKit and SwiftUI defaults.
    pub fn mac() -> Motion {
        Motion {
            snappy: SpringParams::from_response(0.24, 0.92),
            smooth: SpringParams::from_response(0.42, 0.95),
            bouncy: SpringParams::from_response(0.32, 0.72),
            gentle: SpringParams::from_response(0.60, 1.0),
            hover_in: 0.0,
            hover_out: 0.12,
            press_out: 0.12,
            selected: 0.15,
            theme_blend: 0.35,
            ..Motion::lumen()
        }
    }

    /// Quick, decisive and without overshoot, like Fluent's fast curves.
    pub fn windows() -> Motion {
        Motion {
            snappy: SpringParams::from_response(0.18, 1.0),
            smooth: SpringParams::from_response(0.33, 1.0),
            bouncy: SpringParams::from_response(0.22, 0.9),
            gentle: SpringParams::from_response(0.50, 1.0),
            hover_in: 0.083,
            hover_out: 0.083,
            press_out: 0.083,
            selected: 0.167,
            fade_out: 0.083,
            layer_in: 0.167,
            theme_blend: 0.25,
            ..Motion::lumen()
        }
    }

    /// Calm ease-out motion, like libadwaita.
    pub fn gnome() -> Motion {
        Motion {
            snappy: SpringParams::from_response(0.22, 1.0),
            smooth: SpringParams::from_response(0.40, 1.0),
            bouncy: SpringParams::from_response(0.28, 0.85),
            gentle: SpringParams::from_response(0.55, 1.0),
            hover_in: 0.10,
            hover_out: 0.20,
            press_out: 0.20,
            selected: 0.20,
            theme_blend: 0.30,
            ..Motion::lumen()
        }
    }

    /// Bouncier and a touch slower: playful products, onboarding.
    pub fn playful() -> Motion {
        Motion {
            snappy: SpringParams::from_response(0.26, 0.75),
            smooth: SpringParams::from_response(0.45, 0.78),
            bouncy: SpringParams::from_response(0.38, 0.55),
            gentle: SpringParams::from_response(0.65, 0.9),
            selected: 0.18,
            theme_blend: 0.45,
            ..Motion::lumen()
        }
    }

    /// No overshoot anywhere and slower fades: dense, focused tools.
    pub fn calm() -> Motion {
        Motion {
            snappy: SpringParams::from_response(0.28, 1.0),
            smooth: SpringParams::from_response(0.50, 1.0),
            bouncy: SpringParams::from_response(0.30, 1.0),
            gentle: SpringParams::from_response(0.70, 1.0),
            hover_in: 0.10,
            hover_out: 0.24,
            press_out: 0.24,
            selected: 0.20,
            theme_blend: 0.45,
            ..Motion::lumen()
        }
    }

    /// Every spring and duration scaled: `2.0` is twice as slow.
    pub fn slower(mut self, f: f32) -> Motion {
        let f = f.max(0.05);
        for s in [&mut self.snappy, &mut self.smooth, &mut self.bouncy, &mut self.gentle] {
            // Same damping ratio, response scaled by f.
            s.stiffness /= f * f;
            s.damping /= f;
        }
        for d in [&mut self.hover_in, &mut self.hover_out, &mut self.press_out, &mut self.selected, &mut self.fade_out, &mut self.layer_in, &mut self.mark_out, &mut self.theme_blend] {
            *d *= f;
        }
        self
    }
}

/// Font families to try for each text role, in order. Empty uses the
/// bundled fonts (Geist, Geist Mono).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Fonts {
    pub sans: &'static [&'static str],
    pub display: &'static [&'static str],
    pub mono: &'static [&'static str],
}

impl Fonts {
    pub const BUNDLED: Fonts = Fonts { sans: &[], display: &[], mono: &[] };

    pub fn is_bundled(&self) -> bool {
        self.sans.is_empty() && self.display.is_empty() && self.mono.is_empty()
    }
}

/// Control density.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Density {
    Compact,
    #[default]
    Regular,
    Comfortable,
}

/// A complete visual language. See the [module docs](self).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    pub platform: Platform,
    pub light: Neutrals,
    pub dark: Neutrals,
    /// The look's own accent `[light, dark]`, used by [`crate::Accent::Native`]
    /// when the OS reports none.
    pub accent: [Color; 2],
    /// Dark mode: lift accents below this OKLCH lightness (Windows uses
    /// light accent shades on dark backgrounds).
    pub dark_accent_min_lightness: f32,
    /// Preferred system fonts (native builds; falls back to bundled ones).
    pub fonts: Fonts,
    /// Scales every text role's size and line height.
    pub text_scale: f32,
    pub control: ControlLook,
    pub button: ButtonLook,
    pub switch: SwitchLook,
    pub checkbox: CheckboxLook,
    pub slider: SliderLook,
    pub input: InputLook,
    pub focus: FocusLook,
    pub card: CardLook,
    pub nav: NavLook,
    pub motion: Motion,
}

impl Default for Look {
    fn default() -> Self {
        Look::lumen()
    }
}

impl Lerp for Look {
    fn lerp(self, o: Self, t: f32) -> Self {
        // Discrete fields switch at the start: colors come from the palette
        // blend and fonts cannot be interpolated.
        let d = if t > 0.0 { o } else { self };
        Look {
            platform: d.platform,
            light: o.light,
            dark: o.dark,
            accent: o.accent,
            dark_accent_min_lightness: o.dark_accent_min_lightness,
            fonts: d.fonts,
            text_scale: self.text_scale.lerp(o.text_scale, t),
            control: self.control.lerp(o.control, t),
            button: self.button.lerp(o.button, t),
            switch: self.switch.lerp(o.switch, t),
            checkbox: self.checkbox.lerp(o.checkbox, t),
            slider: self.slider.lerp(o.slider, t),
            input: self.input.lerp(o.input, t),
            focus: self.focus.lerp(o.focus, t),
            card: self.card.lerp(o.card, t),
            nav: self.nav.lerp(o.nav, t),
            motion: self.motion.lerp(o.motion, t),
        }
    }
}

impl Look {
    /// lumen's own look (DESIGN.md).
    pub const fn lumen() -> Look {
        Look {
            platform: Platform::Lumen,
            // Ethereal: a cool night-sky canvas, glassy surfaces and light as
            // the only material (site/DESIGN.md).
            light: Neutrals {
                background: h(0xEEF0F5),
                surface: h(0xF9FAFC),
                surface_raised: h(0xFFFFFF),
                surface_muted: h(0xF0F2F6),
                track: h(0xD5D8E1),
                inverse_surface: h(0x1A1D28),
                inverse_foreground: h(0xF3F4F8),
                foreground: h(0x11131C),
                text_secondary: h(0x5B6072),
                text_muted: h(0x666B7C),
                text_disabled: h(0xA5A9B6),
                border_subtle: h(0xE6E8EE),
                border: h(0xDDE0E8),
                border_strong: h(0xC3C7D2),
                control: h(0xFFFFFF),
                control_border: h(0xDADDE5),
                control_edge: h(0xDADDE5),
                ink: h(0x101428),
                focus: None,
            },
            dark: Neutrals {
                background: h(0x07080C),
                surface: h(0x10121A),
                surface_raised: h(0x151821),
                surface_muted: h(0x1C1E26),
                track: h(0x262832),
                inverse_surface: h(0xE6E9F2),
                inverse_foreground: h(0x11131C),
                foreground: h(0xEEF0F6),
                text_secondary: h(0x9AA0B2),
                text_muted: h(0x80869B),
                text_disabled: h(0x444857),
                border_subtle: h(0x1C1E25),
                border: h(0x24262E),
                border_strong: h(0x363844),
                control: h(0x161821),
                control_border: h(0x2A2C35),
                control_edge: h(0x2A2C35),
                ink: h(0x000000),
                focus: None,
            },
            accent: [h(0x0E6EE6), h(0x1270E9)],
            dark_accent_min_lightness: 0.0,
            fonts: Fonts::BUNDLED,
            text_scale: 1.0,
            control: ControlLook {
                height: [30.0, 36.0, 42.0],
                radius: [8.0, 10.0, 12.0],
                pad_x: [10.0, 12.0, 16.0],
                text_size: [12.0, 13.0, 14.0],
                gap: [6.0, 6.0, 8.0],
            },
            button: ButtonLook { press_inset: 1.0, press_dim: 0.0, bevel: 0.0, shadow: 0.0, bottom_edge: 0.0 },
            switch: SwitchLook {
                width: 40.0,
                height: 24.0,
                knob: 20.0,
                hover_grow: 0.0,
                press_widen: 5.0,
                outline_off: 0.0,
                knob_shadow: 1.0,
                knob_tint_on: 1.0,
            },
            checkbox: CheckboxLook { size: 16.0, radius: 4.0, border: 1.0 },
            slider: SliderLook { track: 4.0, thumb: 18.0, hover_grow: 3.0, inner_dot: 0.0, halo: 1.0, thumb_border: 0.0 },
            input: InputLook { filled: 0.0, underline: 0.0, halo: 1.0 },
            focus: FocusLook { width: 2.0, gap: 2.0, gap_from: 4.0, alpha: 1.0 },
            card: CardLook { radius: 16.0, border: 1.0, shadow: 0.0, pad: 20.0, highlight: 1.0, glow: 1.0 },
            nav: NavLook { item_height: 32.0, radius: 8.0, indicator: 0.0, accent_selection: 1.0 },
            motion: Motion::lumen(),
        }
    }

    /// macOS (Sonoma and later): compact controls, bevelled buttons, wide
    /// soft focus rings, SF Pro where available.
    pub fn mac() -> Look {
        Look {
            platform: Platform::Mac,
            light: Neutrals {
                background: h(0xF0F0F0),
                surface: h(0xFFFFFF),
                surface_raised: h(0xFFFFFF),
                surface_muted: h(0xF5F5F5),
                track: h(0xE1E1E1),
                inverse_surface: h(0x2A2A2C),
                inverse_foreground: h(0xF5F5F5),
                foreground: h(0x242424),
                text_secondary: h(0x6E6E6E),
                text_muted: h(0x8A8A8A),
                text_disabled: h(0xB5B5B5),
                border_subtle: h(0xE6E6E6),
                border: h(0xDCDCDC),
                border_strong: h(0xC4C4C4),
                control: h(0xFFFFFF),
                control_border: h(0xD3D3D3),
                control_edge: h(0xC6C6C6),
                ink: h(0x000000),
                focus: None,
            },
            dark: Neutrals {
                background: h(0x1E1E1E),
                surface: h(0x282828),
                surface_raised: h(0x323232),
                surface_muted: h(0x2C2C2C),
                track: h(0x3C3C3C),
                inverse_surface: h(0xE8E8E8),
                inverse_foreground: h(0x1E1E1E),
                foreground: h(0xDFDFDF),
                text_secondary: h(0x9B9B9B),
                text_muted: h(0x7C7C7C),
                text_disabled: h(0x565656),
                border_subtle: h(0x2F2F2F),
                border: h(0x393939),
                border_strong: h(0x4C4C4C),
                control: h(0x5A5A5C),
                control_border: h(0x5A5A5C),
                control_edge: h(0x4E4E50),
                ink: h(0x000000),
                focus: None,
            },
            accent: [h(0x007AFF), h(0x0A84FF)],
            dark_accent_min_lightness: 0.0,
            fonts: Fonts {
                sans: &["SF Pro Text", "SF Pro", "System Font", ".SF NS", "Helvetica Neue"],
                display: &["SF Pro Display", "SF Pro", "System Font", ".SF NS", "Helvetica Neue"],
                mono: &["SF Mono", ".SF NS Mono", "Menlo"],
            },
            text_scale: 1.0,
            control: ControlLook {
                height: [20.0, 24.0, 30.0],
                radius: [5.0, 6.0, 7.0],
                pad_x: [8.0, 10.0, 14.0],
                text_size: [11.0, 13.0, 13.0],
                gap: [4.0, 5.0, 6.0],
            },
            button: ButtonLook { press_inset: 0.0, press_dim: 0.0, bevel: 1.0, shadow: 1.0, bottom_edge: 0.0 },
            switch: SwitchLook {
                width: 38.0,
                height: 22.0,
                knob: 20.0,
                hover_grow: 0.0,
                press_widen: 5.0,
                outline_off: 0.0,
                knob_shadow: 1.0,
                knob_tint_on: 0.0,
            },
            checkbox: CheckboxLook { size: 14.0, radius: 3.5, border: 1.0 },
            slider: SliderLook { track: 4.0, thumb: 20.0, hover_grow: 0.0, inner_dot: 0.0, halo: 0.0, thumb_border: 0.0 },
            input: InputLook { filled: 0.0, underline: 0.0, halo: 1.0 },
            focus: FocusLook { width: 3.5, gap: 0.0, gap_from: 6.0, alpha: 0.6 },
            card: CardLook { radius: 10.0, border: 0.7, shadow: 0.0, pad: 16.0, highlight: 0.0, glow: 0.0 },
            nav: NavLook { item_height: 28.0, radius: 6.0, indicator: 0.0, accent_selection: 0.0 },
            motion: Motion::mac(),
        }
    }

    /// Windows 11 (Fluent): Mica backgrounds, small radii, outlined off
    /// switches, accent underlines, Segoe UI Variable where available.
    pub fn windows() -> Look {
        Look {
            platform: Platform::Windows,
            light: Neutrals {
                background: h(0xF3F3F3),
                surface: h(0xFBFBFB),
                surface_raised: h(0xFFFFFF),
                surface_muted: h(0xF6F6F6),
                track: h(0x8A8A8A),
                inverse_surface: h(0x2B2B2B),
                inverse_foreground: h(0xFFFFFF),
                foreground: h(0x1B1B1B),
                text_secondary: h(0x5F5F5F),
                text_muted: h(0x8B8B8B),
                text_disabled: h(0xA0A0A0),
                border_subtle: h(0xEBEBEB),
                border: h(0xE5E5E5),
                border_strong: h(0x8A8A8A),
                control: h(0xFBFBFB),
                control_border: h(0xE5E5E5),
                control_edge: h(0xCCCCCC),
                ink: h(0x000000),
                focus: Some(h(0x1B1B1B)),
            },
            dark: Neutrals {
                background: h(0x202020),
                surface: h(0x2B2B2B),
                surface_raised: h(0x2D2D2D),
                surface_muted: h(0x282828),
                track: h(0x9A9A9A),
                inverse_surface: h(0x2C2C2C),
                inverse_foreground: h(0xFFFFFF),
                foreground: h(0xFFFFFF),
                text_secondary: h(0xC5C5C5),
                text_muted: h(0x9E9E9E),
                text_disabled: h(0x5D5D5D),
                border_subtle: h(0x272727),
                border: h(0x1D1D1D),
                border_strong: h(0x9A9A9A),
                control: h(0x2D2D2D),
                control_border: h(0x353535),
                control_edge: h(0x262626),
                ink: h(0x000000),
                focus: Some(h(0xFFFFFF)),
            },
            accent: [h(0x005FB8), h(0x60CDFF)],
            dark_accent_min_lightness: 0.78,
            fonts: Fonts {
                sans: &["Segoe UI Variable Text", "Segoe UI Variable", "Segoe UI"],
                display: &["Segoe UI Variable Display", "Segoe UI Variable", "Segoe UI"],
                mono: &["Cascadia Mono", "Consolas"],
            },
            text_scale: 14.0 / 13.0,
            control: ControlLook {
                height: [24.0, 32.0, 40.0],
                radius: [4.0, 4.0, 5.0],
                pad_x: [8.0, 12.0, 16.0],
                text_size: [12.0, 14.0, 14.0],
                gap: [6.0, 8.0, 8.0],
            },
            button: ButtonLook { press_inset: 0.0, press_dim: 1.0, bevel: 0.0, shadow: 0.0, bottom_edge: 1.0 },
            switch: SwitchLook {
                width: 40.0,
                height: 20.0,
                knob: 12.0,
                hover_grow: 2.0,
                press_widen: 3.0,
                outline_off: 1.0,
                knob_shadow: 0.0,
                knob_tint_on: 1.0,
            },
            checkbox: CheckboxLook { size: 20.0, radius: 4.0, border: 1.0 },
            slider: SliderLook { track: 4.0, thumb: 20.0, hover_grow: 0.0, inner_dot: 1.0, halo: 0.0, thumb_border: 1.0 },
            input: InputLook { filled: 0.0, underline: 1.0, halo: 0.0 },
            focus: FocusLook { width: 2.0, gap: 1.0, gap_from: 1.0, alpha: 1.0 },
            card: CardLook { radius: 8.0, border: 1.0, shadow: 0.0, pad: 16.0, highlight: 0.0, glow: 0.0 },
            nav: NavLook { item_height: 36.0, radius: 4.0, indicator: 1.0, accent_selection: 0.0 },
            motion: Motion::windows(),
        }
    }

    /// GNOME (libadwaita): flat filled buttons, roomy controls, cards with
    /// soft shadows, Adwaita Sans or Cantarell where available.
    pub fn gnome() -> Look {
        Look {
            platform: Platform::Gnome,
            light: Neutrals {
                background: h(0xFAFAFB),
                surface: h(0xFFFFFF),
                surface_raised: h(0xFFFFFF),
                surface_muted: h(0xF2F2F3),
                track: h(0xD8D8DA),
                inverse_surface: h(0x323234),
                inverse_foreground: h(0xFFFFFF),
                foreground: h(0x323234),
                text_secondary: h(0x707073),
                text_muted: h(0x8E8E91),
                text_disabled: h(0xB8B8BA),
                border_subtle: h(0xEDEDEE),
                border: h(0xE0E0E2),
                border_strong: h(0xBDBDC0),
                control: h(0xE4E4E6),
                control_border: Color::TRANSPARENT,
                control_edge: Color::TRANSPARENT,
                ink: h(0x000006),
                focus: None,
            },
            dark: Neutrals {
                background: h(0x222226),
                surface: h(0x323236),
                surface_raised: h(0x3A3A3E),
                surface_muted: h(0x2B2B2F),
                track: h(0x4A4A4E),
                inverse_surface: h(0xE8E8EA),
                inverse_foreground: h(0x222226),
                foreground: h(0xFFFFFF),
                text_secondary: h(0xA2A2A5),
                text_muted: h(0x88888B),
                text_disabled: h(0x58585C),
                border_subtle: h(0x2D2D31),
                border: h(0x3A3A3E),
                border_strong: h(0x5A5A5E),
                control: h(0x3A3A3E),
                control_border: Color::TRANSPARENT,
                control_edge: Color::TRANSPARENT,
                ink: h(0x000000),
                focus: None,
            },
            accent: [h(0x3584E4), h(0x3584E4)],
            dark_accent_min_lightness: 0.0,
            fonts: Fonts {
                // GNOME 48+, older GNOME, Ubuntu's GNOME, then what most
                // distributions ship.
                sans: &["Adwaita Sans", "Cantarell", "Ubuntu Sans", "Ubuntu", "Noto Sans", "DejaVu Sans"],
                display: &["Adwaita Sans", "Cantarell", "Ubuntu Sans", "Ubuntu", "Noto Sans", "DejaVu Sans"],
                mono: &["Adwaita Mono", "Ubuntu Sans Mono", "Ubuntu Mono", "Source Code Pro", "DejaVu Sans Mono"],
            },
            text_scale: 14.0 / 13.0,
            control: ControlLook {
                height: [28.0, 34.0, 42.0],
                radius: [6.0, 6.0, 8.0],
                pad_x: [10.0, 14.0, 18.0],
                text_size: [12.0, 13.0, 14.0],
                gap: [6.0, 6.0, 8.0],
            },
            button: ButtonLook { press_inset: 0.0, press_dim: 0.0, bevel: 0.0, shadow: 0.0, bottom_edge: 0.0 },
            switch: SwitchLook {
                width: 46.0,
                height: 26.0,
                knob: 20.0,
                hover_grow: 0.0,
                press_widen: 0.0,
                outline_off: 0.0,
                knob_shadow: 1.0,
                knob_tint_on: 0.0,
            },
            checkbox: CheckboxLook { size: 16.0, radius: 4.0, border: 2.0 },
            slider: SliderLook { track: 4.0, thumb: 20.0, hover_grow: 0.0, inner_dot: 0.0, halo: 0.0, thumb_border: 1.0 },
            input: InputLook { filled: 1.0, underline: 0.0, halo: 0.0 },
            focus: FocusLook { width: 2.0, gap: 0.0, gap_from: 3.0, alpha: 0.5 },
            card: CardLook { radius: 12.0, border: 0.0, shadow: 1.0, pad: 18.0, highlight: 0.0, glow: 0.0 },
            nav: NavLook { item_height: 36.0, radius: 6.0, indicator: 0.0, accent_selection: 0.0 },
            motion: Motion::gnome(),
        }
    }

    /// The look for the OS this binary runs on (lumen's own on the web).
    pub fn native() -> Look {
        Platform::current().look()
    }

    /// Neutrals for a mode.
    pub fn neutrals(&self, mode: crate::theme::Mode) -> &Neutrals {
        match mode {
            crate::theme::Mode::Light => &self.light,
            crate::theme::Mode::Dark => &self.dark,
        }
    }

    pub fn neutrals_mut(&mut self, mode: crate::theme::Mode) -> &mut Neutrals {
        match mode {
            crate::theme::Mode::Light => &mut self.light,
            crate::theme::Mode::Dark => &mut self.dark,
        }
    }

    // ------------------------------------------------------------ builders

    /// Scale every corner radius (`0.0` square, `1.0` as designed, `2.0`
    /// twice as round). Pill shapes (switches, knobs) stay pills.
    pub fn rounded(mut self, f: f32) -> Look {
        let f = f.max(0.0);
        for r in &mut self.control.radius {
            *r *= f;
        }
        self.checkbox.radius = (self.checkbox.radius * f).min(self.checkbox.size / 2.0);
        self.card.radius *= f;
        self.nav.radius *= f;
        self
    }

    /// Shift control heights for a denser or roomier UI.
    pub fn density(mut self, d: Density) -> Look {
        let (dh, dp) = match d {
            Density::Compact => (-4.0, -2.0),
            Density::Regular => (0.0, 0.0),
            Density::Comfortable => (4.0, 2.0),
        };
        for v in &mut self.control.height {
            *v += dh;
        }
        for v in &mut self.control.pad_x {
            *v += dp;
        }
        self.nav.item_height += dh;
        self.card.pad += dp * 2.0;
        self
    }

    /// Replace the motion.
    pub fn motion(mut self, m: Motion) -> Look {
        self.motion = m;
        self
    }

    /// Scale all text.
    /// Control labels scale along with it.
    pub fn text_scale(mut self, s: f32) -> Look {
        let s = s.clamp(0.5, 3.0);
        let ratio = s / self.text_scale;
        for v in &mut self.control.text_size {
            *v *= ratio;
        }
        self.text_scale = s;
        self
    }

    /// Prefer these font families (system fonts) for the three roles.
    pub fn fonts(mut self, f: Fonts) -> Look {
        self.fonts = f;
        self
    }

    /// Use the bundled Geist fonts even for platform looks.
    pub fn bundled_fonts(mut self) -> Look {
        self.fonts = Fonts::BUNDLED;
        self
    }

    /// Set the look's own accent (used by `Accent::Native` when the OS
    /// reports none).
    pub fn accent(mut self, light: Color, dark: Color) -> Look {
        self.accent = [light, dark];
        self
    }

    /// Edit the neutrals of one mode.
    pub fn with_neutrals(mut self, mode: crate::theme::Mode, f: impl FnOnce(&mut Neutrals)) -> Look {
        f(self.neutrals_mut(mode));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_endpoints() {
        let a = Look::lumen();
        let b = Look::windows();
        assert_eq!(a.lerp(b, 1.0), b);
        let mid = a.lerp(b, 0.5);
        assert_eq!(mid.card.radius, 12.0);
        assert_eq!(mid.platform, Platform::Windows);
        assert_eq!(a.lerp(b, 0.0).platform, Platform::Lumen);
    }

    #[test]
    fn rounded_and_density() {
        let l = Look::lumen().rounded(0.0).density(Density::Compact);
        assert_eq!(l.control.radius, [0.0; 3]);
        assert_eq!(l.control.height[1], 32.0);
        let r = Look::lumen().rounded(10.0);
        assert_eq!(r.checkbox.radius, 8.0, "capped at half the box");
    }

    #[test]
    fn slower_keeps_damping_ratio() {
        let m = Motion::lumen();
        let s = m.slower(2.0);
        assert!((m.bouncy.damping_ratio() - s.bouncy.damping_ratio()).abs() < 1e-4);
        assert_eq!(s.hover_out, m.hover_out * 2.0);
    }
}
