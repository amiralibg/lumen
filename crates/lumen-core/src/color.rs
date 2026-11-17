//! sRGB colors with straight alpha, plus OKLab/OKLCH math for blending.

/// A color in gamma-encoded sRGB with straight (non-premultiplied) alpha,
/// each channel in 0..=1. Blending of UI layers happens in sRGB space,
/// exactly like browsers and design tools, so hex values match the spec.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    /// `0xRRGGBB`, opaque.
    pub const fn hex(v: u32) -> Color {
        Color {
            r: ((v >> 16) & 0xff) as f32 / 255.0,
            g: ((v >> 8) & 0xff) as f32 / 255.0,
            b: (v & 0xff) as f32 / 255.0,
            a: 1.0,
        }
    }

    pub const fn with_alpha(self, a: f32) -> Color {
        Color { a, ..self }
    }

    pub fn multiply_alpha(self, f: f32) -> Color {
        Color { a: self.a * f, ..self }
    }

    /// Pack as `[r, g, b, a]` bytes (straight alpha).
    pub fn to_u8(self) -> [u8; 4] {
        let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        [q(self.r), q(self.g), q(self.b), q(self.a)]
    }

    pub fn to_hex(self) -> u32 {
        let [r, g, b, _] = self.to_u8();
        (r as u32) << 16 | (g as u32) << 8 | b as u32
    }

    /// Source-over composite of `top` onto `self` (both straight alpha).
    pub fn over(self, top: Color) -> Color {
        let a = top.a + self.a * (1.0 - top.a);
        if a <= 0.0 {
            return Color::TRANSPARENT;
        }
        let ch = |b: f32, t: f32| (t * top.a + b * self.a * (1.0 - top.a)) / a;
        Color { r: ch(self.r, top.r), g: ch(self.g, top.g), b: ch(self.b, top.b), a }
    }

    pub fn to_oklab(self) -> Oklab {
        let l = |c: f32| if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
        let (r, g, b) = (l(self.r), l(self.g), l(self.b));
        let lm = 0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b;
        let mm = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
        let sm = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;
        let (l_, m_, s_) = (lm.cbrt(), mm.cbrt(), sm.cbrt());
        Oklab {
            l: 0.210_454_26 * l_ + 0.793_617_8 * m_ - 0.004_072_047 * s_,
            a: 1.977_998_5 * l_ - 2.428_592_2 * m_ + 0.450_593_7 * s_,
            b: 0.025_904_037 * l_ + 0.782_771_77 * m_ - 0.808_675_77 * s_,
            alpha: self.a,
        }
    }

    /// Interpolate in OKLab (alpha linearly). This is how every lumen color
    /// transition is computed.
    pub fn mix(self, other: Color, t: f32) -> Color {
        if t <= 0.0 {
            return self;
        }
        if t >= 1.0 {
            return other;
        }
        // Mixing to/from fully transparent should only fade alpha.
        let (a, b) = match (self.a == 0.0, other.a == 0.0) {
            (true, false) => (other.with_alpha(0.0), other),
            (false, true) => (self, self.with_alpha(0.0)),
            _ => (self, other),
        };
        let (x, y) = (a.to_oklab(), b.to_oklab());
        Oklab {
            l: x.l + (y.l - x.l) * t,
            a: x.a + (y.a - x.a) * t,
            b: x.b + (y.b - x.b) * t,
            alpha: x.alpha + (y.alpha - x.alpha) * t,
        }
        .to_srgb()
    }

    /// Shift OKLCH lightness by `dl`, keeping hue and reducing chroma until
    /// the result fits in sRGB (DESIGN.md 2.3).
    pub fn shift_lightness(self, dl: f32) -> Color {
        let lab = self.to_oklab();
        let c = (lab.a * lab.a + lab.b * lab.b).sqrt();
        let h = lab.b.atan2(lab.a);
        let l = (lab.l + dl).clamp(0.0, 1.0);
        let make = |c: f32| Oklab { l, a: c * h.cos(), b: c * h.sin(), alpha: lab.alpha };
        if make(c).in_gamut() {
            return make(c).to_srgb();
        }
        let (mut lo, mut hi) = (0.0, c);
        for _ in 0..24 {
            let mid = (lo + hi) / 2.0;
            if make(mid).in_gamut() {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        make(lo).to_srgb()
    }

    /// OKLCH lightness.
    pub fn lightness(self) -> f32 {
        self.to_oklab().l
    }

    /// A color from OKLCH (`l` 0..1, chroma, hue in degrees), reduced in
    /// chroma until it fits in sRGB.
    pub fn oklch(l: f32, c: f32, hue_deg: f32) -> Color {
        let h = hue_deg.to_radians();
        let make = |c: f32| Oklab { l: l.clamp(0.0, 1.0), a: c * h.cos(), b: c * h.sin(), alpha: 1.0 };
        if make(c).in_gamut() {
            return make(c).to_srgb();
        }
        let (mut lo, mut hi) = (0.0, c);
        for _ in 0..24 {
            let mid = (lo + hi) / 2.0;
            if make(mid).in_gamut() {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        make(lo).to_srgb()
    }

    /// OKLCH hue in degrees (0..360).
    pub fn hue(self) -> f32 {
        let lab = self.to_oklab();
        lab.b.atan2(lab.a).to_degrees().rem_euclid(360.0)
    }

    /// WCAG relative luminance.
    pub fn luminance(self) -> f32 {
        let l = |c: f32| if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
        0.2126 * l(self.r) + 0.7152 * l(self.g) + 0.0722 * l(self.b)
    }

    /// WCAG contrast ratio between two opaque colors (1..21).
    pub fn contrast(self, other: Color) -> f32 {
        let (a, b) = (self.luminance(), other.luminance());
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Oklab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
    pub alpha: f32,
}

impl Oklab {
    fn to_linear(self) -> (f32, f32, f32) {
        let l_ = self.l + 0.396_337_78 * self.a + 0.215_803_76 * self.b;
        let m_ = self.l - 0.105_561_346 * self.a - 0.063_854_17 * self.b;
        let s_ = self.l - 0.089_484_18 * self.a - 1.291_485_5 * self.b;
        let (l, m, s) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
        (
            4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
            -1.268_438 * l + 2.609_757_4 * m - 0.341_319_4 * s,
            -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
        )
    }

    fn in_gamut(self) -> bool {
        let (r, g, b) = self.to_linear();
        let ok = |v: f32| (-1e-4..=1.0 + 1e-4).contains(&v);
        ok(r) && ok(g) && ok(b)
    }

    pub fn to_srgb(self) -> Color {
        let (r, g, b) = self.to_linear();
        let e = |c: f32| {
            let c = c.clamp(0.0, 1.0);
            if c <= 0.003_130_8 {
                c * 12.92
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            }
        };
        Color { r: e(r), g: e(g), b: e(b), a: self.alpha }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        for v in [0x14171D, 0xFFFFFF, 0x7D55EC, 0x000000] {
            assert_eq!(Color::hex(v).to_hex(), v);
        }
    }

    #[test]
    fn oklab_roundtrip() {
        for v in [0x14171D, 0xF9FAFB, 0x7D55EC, 0xF5AF20, 0x02844B] {
            let c = Color::hex(v);
            assert_eq!(c.to_oklab().to_srgb().to_hex(), v);
        }
    }

    #[test]
    fn oklch_lightness_matches_spec() {
        // DESIGN.md: FOREGROUND light is L 0.204, VIOLET accent L 0.575.
        assert!((Color::hex(0x14171D).lightness() - 0.204).abs() < 0.002);
        assert!((Color::hex(0x7D55EC).lightness() - 0.575).abs() < 0.002);
    }

    #[test]
    fn mix_endpoints_and_alpha() {
        let a = Color::hex(0x000000);
        let b = Color::hex(0xFFFFFF);
        assert_eq!(a.mix(b, 0.0), a);
        assert_eq!(a.mix(b, 1.0), b);
        let mid = a.mix(b, 0.5);
        // OKLab midpoint of black/white is perceptual mid-gray, not #808080.
        assert!(mid.r > 0.35 && mid.r < 0.42, "{mid:?}");
        let fade = Color::TRANSPARENT.mix(b, 0.5);
        assert_eq!(fade.to_hex(), 0xFFFFFF);
        assert!((fade.a - 0.5).abs() < 1e-6);
    }

    #[test]
    fn contrast_and_oklch() {
        assert!((Color::WHITE.contrast(Color::BLACK) - 21.0).abs() < 0.01);
        let c = Color::oklch(0.6, 0.15, 250.0);
        assert!((c.lightness() - 0.6).abs() < 0.01);
        assert!((c.hue() - 250.0).abs() < 1.0);
    }

    #[test]
    fn over_composites() {
        let base = Color::hex(0xFFFFFF);
        let c = base.over(Color::hex(0x000000).with_alpha(0.5));
        assert_eq!(c.to_hex(), 0x808080);
        assert_eq!(c.a, 1.0);
    }
}
