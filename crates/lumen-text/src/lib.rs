//! Text shaping, layout and glyph rasterization for lumen.
//!
//! `TextSystem` owns the font database (with Geist, Geist Mono and
//! Lucide bundled), lays text out into [`Galley`]s in logical pixels, and
//! rasterizes glyphs into a single-channel [`Atlas`] at physical resolution.
//! It knows nothing about windows or GPUs: the renderer uploads
//! [`AtlasDelta`]s.

mod atlas;
mod icons;

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use ahash::{AHashMap, AHasher};
pub use atlas::{Atlas, AtlasDelta, AtlasGlyph};
pub use cosmic_text;
use cosmic_text::{
    fontdb, Attrs, Buffer, CacheKey, Family, FontSystem, LayoutGlyph, Metrics, Shaping,
    SwashCache, SwashContent, Weight, Wrap,
};

/// Font family role. Each role maps to a bundled family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum FontFamily {
    /// Geist: body text, labels, controls.
    #[default]
    Sans,
    /// Geist: large display headings.
    Display,
    /// Geist Mono: code and tabular values.
    Mono,
    /// Lucide icon font.
    Icons,
}

impl FontFamily {
    fn name(self) -> &'static str {
        match self {
            FontFamily::Sans => "Geist",
            FontFamily::Display => "Geist",
            FontFamily::Mono => "Geist Mono",
            FontFamily::Icons => "lucide",
        }
    }
}

/// Horizontal alignment of lines inside a galley's wrap width.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TextAlign {
    #[default]
    Start,
    Center,
    End,
}

/// Everything that affects how a run of text is shaped.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
    pub family: FontFamily,
    /// CSS-style weight (400 regular, 500 medium, 600 semibold).
    pub weight: u16,
    /// Font size in logical pixels.
    pub size: f32,
    /// Line height in logical pixels.
    pub line_height: f32,
    /// Letter spacing in em.
    pub tracking: f32,
    /// Enable tabular figures (`tnum`), useful for changing numbers.
    pub tabular: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self { family: FontFamily::Sans, weight: 400, size: 14.0, line_height: 20.0, tracking: 0.0, tabular: false }
    }
}

impl TextStyle {
    fn hash_into(&self, h: &mut impl Hasher) {
        self.family.hash(h);
        self.weight.hash(h);
        self.size.to_bits().hash(h);
        self.line_height.to_bits().hash(h);
        self.tracking.to_bits().hash(h);
        self.tabular.hash(h);
    }
}

/// One shaped glyph, positioned relative to the galley origin (logical px).
#[derive(Clone, Debug)]
pub struct GalleyGlyph {
    pub(crate) glyph: LayoutGlyph,
    /// Baseline y of the glyph's line.
    pub line_y: f32,
    /// Line index.
    pub line: usize,
}

impl GalleyGlyph {
    /// Byte range in the source text covered by this glyph's cluster.
    pub fn range(&self) -> std::ops::Range<usize> {
        self.glyph.start..self.glyph.end
    }
    /// Left edge of the glyph's advance box.
    pub fn x(&self) -> f32 {
        self.glyph.x
    }
    /// Advance width.
    pub fn width(&self) -> f32 {
        self.glyph.w
    }
    pub fn is_rtl(&self) -> bool {
        self.glyph.level.is_rtl()
    }
}

/// Metrics of one laid-out line.
#[derive(Clone, Copy, Debug)]
pub struct GalleyLine {
    pub top: f32,
    pub height: f32,
    pub baseline: f32,
    pub width: f32,
    /// Byte range of the source text this line covers.
    pub start: usize,
    pub end: usize,
}

/// Laid-out text, ready to paint. All units are logical pixels.
#[derive(Clone, Debug)]
pub struct Galley {
    pub text: String,
    pub style: TextStyle,
    /// Width of the widest line.
    pub width: f32,
    /// Total height of all lines.
    pub height: f32,
    pub lines: Vec<GalleyLine>,
    pub glyphs: Vec<GalleyGlyph>,
}

impl Galley {
    pub fn size(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    /// X position (logical, relative to galley origin) of the caret placed
    /// before byte `index` on a single-line galley.
    pub fn x_for_index(&self, index: usize) -> f32 {
        if self.glyphs.is_empty() {
            return 0.0;
        }
        for g in &self.glyphs {
            let r = g.range();
            if index <= r.start {
                return if g.is_rtl() { g.x() + g.width() } else { g.x() };
            }
            if index < r.end {
                // Inside a multi-codepoint cluster: interpolate.
                let t = (index - r.start) as f32 / (r.end - r.start).max(1) as f32;
                return if g.is_rtl() { g.x() + g.width() * (1.0 - t) } else { g.x() + g.width() * t };
            }
        }
        let last = self.glyphs.last().unwrap();
        if last.is_rtl() {
            last.x()
        } else {
            last.x() + last.width()
        }
    }

    /// Byte index of the caret boundary nearest to x on a single-line galley.
    pub fn index_for_x(&self, x: f32) -> usize {
        let mut best = (f32::INFINITY, 0usize);
        let mut consider = |pos: f32, idx: usize| {
            let d = (pos - x).abs();
            if d < best.0 {
                best = (d, idx);
            }
        };
        consider(self.x_for_index(0), 0);
        for g in &self.glyphs {
            let r = g.range();
            let (l, rr) = (g.x(), g.x() + g.width());
            if g.is_rtl() {
                consider(rr, r.start);
                consider(l, r.end);
            } else {
                consider(l, r.start);
                consider(rr, r.end);
            }
        }
        best.1
    }
}

/// A glyph quad ready for drawing, in physical pixels.
#[derive(Clone, Copy, Debug)]
pub struct GlyphQuad {
    /// Top-left in physical pixels.
    pub x: i32,
    pub y: i32,
    pub glyph: AtlasGlyph,
}

static GEIST_REGULAR: &[u8] = include_bytes!("../fonts/Geist-Regular.ttf");
static GEIST_MEDIUM: &[u8] = include_bytes!("../fonts/Geist-Medium.ttf");
static GEIST_SEMIBOLD: &[u8] = include_bytes!("../fonts/Geist-SemiBold.ttf");
static GEIST_MONO: &[u8] = include_bytes!("../fonts/GeistMono-Regular.ttf");
/// ⌘ ⌥ ⌃ ⌫ ⏎ ✓ and other keyboard symbols Geist lacks: a 14 KB subset of
/// Inter, renamed, used only as a fallback.
static SYMBOLS: &[u8] = include_bytes!("../fonts/LumenSymbols.ttf");
static LUCIDE: &[u8] = include_bytes!("../fonts/Lucide.ttf");

struct CachedGalley {
    galley: Arc<Galley>,
    last_used: u64,
}

/// Font database, shaper, galley cache and glyph atlas.
pub struct TextSystem {
    fonts: FontSystem,
    swash: SwashCache,
    atlas: Atlas,
    glyphs: AHashMap<CacheKey, Option<AtlasGlyph>>,
    galleys: AHashMap<u64, CachedGalley>,
    icons: icons::IconTable,
    frame: u64,
    /// Family overrides for Sans, Display and Mono.
    families: [Option<String>; 3],
    system_loaded: bool,
    /// Load system fonts the first time text needs a script the bundled
    /// fonts lack (see [`TextSystem::set_system_fallback`]).
    auto_fallback: bool,
}

/// True if `c` is outside what the bundled fonts cover: Latin, Greek and
/// Cyrillic, punctuation and symbols, the keyboard symbols and the icons.
fn needs_system_font(c: char) -> bool {
    let u = c as u32;
    u >= 0x0530 && !(0x2000..=0x2BFF).contains(&u) && !(0xE000..=0xF8FF).contains(&u) && !(0xFE00..=0xFE0F).contains(&u)
}

impl Default for TextSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl TextSystem {
    /// Create a text system with only the bundled fonts (deterministic on
    /// every platform, including the web).
    pub fn new() -> Self {
        let mut db = fontdb::Database::new();
        for data in [GEIST_REGULAR, GEIST_MEDIUM, GEIST_SEMIBOLD, GEIST_MONO, SYMBOLS, LUCIDE] {
            db.load_font_source(fontdb::Source::Binary(Arc::new(data)));
        }
        db.set_sans_serif_family("Geist");
        db.set_monospace_family("Geist Mono");
        let fonts = FontSystem::new_with_locale_and_db("en-US".into(), db);
        Self {
            fonts,
            swash: SwashCache::new(),
            atlas: Atlas::new(1024),
            glyphs: AHashMap::new(),
            galleys: AHashMap::new(),
            icons: icons::IconTable::load(),
            frame: 0,
            families: [None, None, None],
            system_loaded: false,
            auto_fallback: false,
        }
    }

    /// When on, the first text that needs a script the bundled fonts don't
    /// cover (CJK, Arabic, Hebrew, Thai, emoji...) loads the system fonts
    /// as fallbacks, once. Native runners turn this on; it stays off for
    /// headless rendering so screenshots don't depend on the machine.
    pub fn set_system_fallback(&mut self, on: bool) {
        self.auto_fallback = on;
    }

    /// Add a font (e.g. a CJK or Arabic fallback). Fonts added here
    /// participate in fallback for every role.
    pub fn add_font(&mut self, data: Vec<u8>) {
        self.fonts.db_mut().load_font_data(data);
        self.galleys.clear();
    }

    /// Load the platform's system fonts as fallbacks (native only; slow,
    /// done once).
    pub fn load_system_fonts(&mut self) {
        if self.system_loaded {
            return;
        }
        self.system_loaded = true;
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.fonts.db_mut().load_system_fonts();
            self.galleys.clear();
        }
    }

    fn role_index(role: FontFamily) -> Option<usize> {
        match role {
            FontFamily::Sans => Some(0),
            FontFamily::Display => Some(1),
            FontFamily::Mono => Some(2),
            FontFamily::Icons => None,
        }
    }

    /// Whether a family with this name is loaded.
    pub fn has_family(&self, name: &str) -> bool {
        self.fonts.db().faces().any(|f| f.families.iter().any(|(n, _)| n.eq_ignore_ascii_case(name)))
    }

    /// All loaded family names (sorted, deduplicated).
    pub fn family_names(&self) -> Vec<String> {
        let mut v: Vec<String> = self.fonts.db().faces().flat_map(|f| f.families.iter().map(|(n, _)| n.clone())).collect();
        v.sort();
        v.dedup();
        v
    }

    /// Use `name` for a role instead of the bundled family (`None` resets).
    /// Icons cannot be overridden.
    pub fn set_family(&mut self, role: FontFamily, name: Option<&str>) {
        let Some(i) = Self::role_index(role) else { return };
        let new = name.map(str::to_owned);
        if self.families[i] != new {
            self.families[i] = new;
            self.galleys.clear();
        }
    }

    /// The family name a role currently resolves to.
    pub fn family_name(&self, role: FontFamily) -> &str {
        Self::role_index(role).and_then(|i| self.families[i].as_deref()).unwrap_or(role.name())
    }

    /// Use the first installed family from `candidates` for `role`, loading
    /// system fonts if needed. Falls back to the bundled family and returns
    /// the chosen name, if any.
    pub fn choose_family(&mut self, role: FontFamily, candidates: &[&str]) -> Option<String> {
        if candidates.is_empty() {
            self.set_family(role, None);
            return None;
        }
        self.load_system_fonts();
        let found = candidates.iter().find(|c| self.has_family(c)).map(|c| c.to_string());
        self.set_family(role, found.as_deref());
        found
    }

    /// The Lucide codepoint for an icon name such as `"settings"`.
    pub fn icon_char(&self, name: &str) -> Option<char> {
        self.icons.get(name)
    }

    /// Lay text out. `max_width` enables word wrapping. Results are cached
    /// and reused across frames while they keep being requested.
    pub fn layout(&mut self, text: &str, style: &TextStyle, max_width: Option<f32>, align: TextAlign) -> Arc<Galley> {
        if self.auto_fallback && !self.system_loaded && text.chars().any(needs_system_font) {
            self.load_system_fonts();
        }
        let mut h = AHasher::default();
        text.hash(&mut h);
        style.hash_into(&mut h);
        max_width.map(f32::to_bits).hash(&mut h);
        align.hash(&mut h);
        let key = h.finish();
        if let Some(c) = self.galleys.get_mut(&key) {
            c.last_used = self.frame;
            return c.galley.clone();
        }
        let galley = Arc::new(self.layout_uncached(text, style, max_width, align));
        self.galleys.insert(key, CachedGalley { galley: galley.clone(), last_used: self.frame });
        galley
    }

    fn layout_uncached(&mut self, text: &str, style: &TextStyle, max_width: Option<f32>, align: TextAlign) -> Galley {
        let metrics = Metrics::new(style.size, style.line_height);
        let mut buffer = Buffer::new(&mut self.fonts, metrics);
        buffer.set_wrap(if max_width.is_some() { Wrap::WordOrGlyph } else { Wrap::None });
        // Round the wrap width up a hair so text measured unconstrained and
        // then laid out at exactly its own width doesn't re-wrap.
        buffer.set_size(max_width.map(|w| w + 0.01), None);
        let family = self.family_name(style.family).to_owned();
        let mut attrs = Attrs::new().family(Family::Name(&family)).weight(Weight(style.weight));
        if style.tracking != 0.0 {
            attrs = attrs.letter_spacing(style.tracking);
        }
        if style.tabular {
            let mut f = cosmic_text::FontFeatures::new();
            f.enable(cosmic_text::FeatureTag::new(b"tnum"));
            attrs = attrs.font_features(f);
        }
        let ct_align = match align {
            TextAlign::Start => None,
            TextAlign::Center => Some(cosmic_text::Align::Center),
            TextAlign::End => Some(cosmic_text::Align::End),
        };
        buffer.set_text(text, &attrs, Shaping::Advanced, ct_align);
        buffer.shape_until_scroll(&mut self.fonts, false);

        // Map buffer line index (split on '\n') to byte offset in `text`.
        let mut line_starts = vec![0usize];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }

        let mut lines = Vec::new();
        let mut glyphs = Vec::new();
        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;
        for run in buffer.layout_runs() {
            let base = line_starts.get(run.line_i).copied().unwrap_or(0);
            let (mut s, mut e) = (usize::MAX, 0usize);
            for g in run.glyphs {
                let mut g2 = g.clone();
                g2.start += base;
                g2.end += base;
                s = s.min(g2.start);
                e = e.max(g2.end);
                glyphs.push(GalleyGlyph { glyph: g2, line_y: run.line_y, line: lines.len() });
            }
            if s == usize::MAX {
                s = base;
                e = base;
            }
            lines.push(GalleyLine { top: run.line_top, height: run.line_height, baseline: run.line_y, width: run.line_w, start: s, end: e });
            width = width.max(run.line_w);
            height = height.max(run.line_top + run.line_height);
        }
        if lines.is_empty() {
            // Empty text still occupies one line, so inputs and labels keep height.
            let ascent = style.size * 0.8;
            let baseline = (style.line_height - style.size) / 2.0 + ascent;
            lines.push(GalleyLine { top: 0.0, height: style.line_height, baseline, width: 0.0, start: 0, end: 0 });
            height = style.line_height;
        }
        Galley { text: text.to_owned(), style: *style, width, height, lines, glyphs }
    }

    /// Rasterize (if needed) every glyph of `galley` placed with its origin at
    /// `origin` (logical px) for a display with `scale` physical px per
    /// logical px, calling `f` with each quad.
    pub fn glyph_quads(&mut self, galley: &Galley, origin: (f32, f32), scale: f32, mut f: impl FnMut(GlyphQuad)) {
        let ox = origin.0 * scale;
        // Snap each baseline to the physical pixel grid for crisp text.
        for g in &galley.glyphs {
            let baseline = ((origin.1 + g.line_y) * scale).round();
            let phys = g.glyph.physical((ox, 0.0), scale);
            let Some(ag) = self.raster(phys.cache_key) else { continue };
            f(GlyphQuad { x: phys.x + ag.left, y: baseline as i32 + phys.y - ag.top, glyph: ag });
        }
    }

    fn raster(&mut self, key: CacheKey) -> Option<AtlasGlyph> {
        if let Some(g) = self.glyphs.get(&key) {
            return *g;
        }
        let img = self.swash.get_image_uncached(&mut self.fonts, key);
        let out = img.and_then(|img| {
            let (w, h) = (img.placement.width, img.placement.height);
            if w == 0 || h == 0 {
                return None;
            }
            let color = img.content == SwashContent::Color;
            let at = match img.content {
                SwashContent::Mask => self.atlas.insert(w, h, &img.data),
                SwashContent::SubpixelMask => self.atlas.insert(w, h, &img.data.chunks(4).map(|p| p[1]).collect::<Vec<_>>()),
                // Color glyphs (emoji) keep their colors, premultiplied.
                SwashContent::Color => {
                    let pm: Vec<u8> = img
                        .data
                        .chunks(4)
                        .flat_map(|p| {
                            let a = p[3] as u16;
                            [(p[0] as u16 * a / 255) as u8, (p[1] as u16 * a / 255) as u8, (p[2] as u16 * a / 255) as u8, p[3]]
                        })
                        .collect();
                    self.atlas.insert_rgba(w, h, &pm)
                }
            };
            at.map(|(x, y)| AtlasGlyph { x, y, w, h, left: img.placement.left, top: img.placement.top, color })
        });
        self.glyphs.insert(key, out);
        out
    }

    pub fn atlas(&self) -> &Atlas {
        &self.atlas
    }

    /// Changes to the atlas since the last call, for the renderer to upload.
    pub fn take_atlas_delta(&mut self) -> Option<AtlasDelta> {
        self.atlas.take_delta()
    }

    /// Call once per frame: evicts galleys unused for a while.
    pub fn end_frame(&mut self) {
        self.frame += 1;
        if self.galleys.len() > 2048 {
            let f = self.frame;
            self.galleys.retain(|_, c| f - c.last_used < 120);
        }
    }

    /// Number of cached galleys (for tests and diagnostics).
    pub fn cached_galleys(&self) -> usize {
        self.galleys.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body() -> TextStyle {
        TextStyle { size: 14.0, line_height: 20.0, ..Default::default() }
    }

    #[test]
    fn measures_single_line() {
        let mut ts = TextSystem::new();
        let g = ts.layout("Hello, lumen", &body(), None, TextAlign::Start);
        assert_eq!(g.lines.len(), 1);
        assert!(g.width > 60.0 && g.width < 110.0, "width {}", g.width);
        assert_eq!(g.height, 20.0);
    }

    #[test]
    fn wraps_to_width() {
        let mut ts = TextSystem::new();
        let text = "The quick brown fox jumps over the lazy dog again and again";
        let g = ts.layout(text, &body(), Some(120.0), TextAlign::Start);
        assert!(g.lines.len() >= 3, "lines {}", g.lines.len());
        assert!(g.width <= 120.5);
        assert_eq!(g.height, g.lines.len() as f32 * 20.0);
    }

    #[test]
    fn own_width_does_not_rewrap() {
        let mut ts = TextSystem::new();
        let g = ts.layout("Save changes", &body(), None, TextAlign::Start);
        let g2 = ts.layout("Save changes", &body(), Some(g.width), TextAlign::Start);
        assert_eq!(g2.lines.len(), 1);
    }

    #[test]
    fn empty_text_has_line_height() {
        let mut ts = TextSystem::new();
        let g = ts.layout("", &body(), None, TextAlign::Start);
        assert_eq!(g.height, 20.0);
        assert_eq!(g.x_for_index(0), 0.0);
    }

    #[test]
    fn caret_positions_monotonic_ltr() {
        let mut ts = TextSystem::new();
        let g = ts.layout("abc def", &body(), None, TextAlign::Start);
        let xs: Vec<f32> = (0..=7).map(|i| g.x_for_index(i)).collect();
        for w in xs.windows(2) {
            assert!(w[1] > w[0], "{xs:?}");
        }
        assert_eq!(g.index_for_x(-5.0), 0);
        assert_eq!(g.index_for_x(1000.0), 7);
        assert_eq!(g.index_for_x(g.x_for_index(4) + 0.5), 4);
    }

    #[test]
    fn galleys_are_cached() {
        let mut ts = TextSystem::new();
        let a = ts.layout("cache me", &body(), None, TextAlign::Start);
        let b = ts.layout("cache me", &body(), None, TextAlign::Start);
        assert!(Arc::ptr_eq(&a, &b));
        assert_eq!(ts.cached_galleys(), 1);
    }

    #[test]
    fn rasterizes_into_atlas() {
        let mut ts = TextSystem::new();
        let g = ts.layout("Ag", &body(), None, TextAlign::Start);
        let mut quads = vec![];
        ts.glyph_quads(&g, (10.0, 10.0), 2.0, |q| quads.push(q));
        assert_eq!(quads.len(), 2);
        assert!(quads.iter().all(|q| q.glyph.w > 0 && q.glyph.h > 0));
        let d = ts.take_atlas_delta().expect("atlas changed");
        assert!(d.w > 0 && d.h > 0);
        assert!(ts.take_atlas_delta().is_none());
    }

    #[test]
    fn icons_resolve_and_render() {
        let mut ts = TextSystem::new();
        let c = ts.icon_char("settings").expect("settings icon");
        let style = TextStyle { family: FontFamily::Icons, size: 16.0, line_height: 16.0, ..Default::default() };
        let g = ts.layout(&c.to_string(), &style, None, TextAlign::Start);
        let mut n = 0;
        ts.glyph_quads(&g, (0.0, 0.0), 1.0, |_| n += 1);
        assert_eq!(n, 1);
        assert!((g.width - 16.0).abs() < 1.0, "lucide glyphs are 1em wide: {}", g.width);
    }

    #[test]
    fn keyboard_symbols_have_glyphs() {
        // Geist has no ⌘ ⌥ ⌃; the bundled symbol fallback must cover them.
        let mut ts = TextSystem::new();
        let g = ts.layout("⌘⌥⌃⇧K", &body(), None, TextAlign::Start);
        assert_eq!(g.glyphs.len(), 5);
        for gl in &g.glyphs {
            assert_ne!(gl.glyph.glyph_id, 0, "missing glyph in {:?}", g.text);
        }
    }

    #[test]
    fn system_fallback_covers_cjk_when_enabled() {
        assert!(!needs_system_font('é') && !needs_system_font('⌘') && !needs_system_font('Ж'));
        assert!(needs_system_font('日') && needs_system_font('😀'));
        let mut ts = TextSystem::new();
        ts.set_system_fallback(true);
        let g = ts.layout("日本", &body(), None, TextAlign::Start);
        if cfg!(target_os = "macos") {
            assert!(g.glyphs.iter().all(|gl| gl.glyph.glyph_id != 0), "macOS ships CJK fonts");
        }
    }

    #[test]
    fn family_override_falls_back() {
        let mut ts = TextSystem::new();
        assert!(ts.has_family("Geist"));
        ts.set_family(FontFamily::Sans, Some("Geist Mono"));
        assert_eq!(ts.family_name(FontFamily::Sans), "Geist Mono");
        let g = ts.layout("Hello", &body(), None, TextAlign::Start);
        assert!(g.width > 0.0);
        ts.set_family(FontFamily::Sans, None);
        assert_eq!(ts.family_name(FontFamily::Sans), "Geist");
    }

    #[test]
    #[ignore = "lists system fonts; run with --ignored --nocapture"]
    fn print_system_families() {
        let mut ts = TextSystem::new();
        ts.load_system_fonts();
        for f in ts.family_names() {
            println!("{f}");
        }
    }

    #[test]
    fn rtl_text_shapes() {
        let mut ts = TextSystem::new();
        // Geist covers Latin only; Hebrew falls back to notdef but must not panic.
        let g = ts.layout("abc שלום", &body(), None, TextAlign::Start);
        assert!(!g.glyphs.is_empty());
    }
}
