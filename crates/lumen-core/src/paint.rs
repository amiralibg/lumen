//! Tessellation into a renderer-agnostic triangle list.
//!
//! Every primitive is a quad whose vertices carry enough data for a single
//! fragment shader to draw it: signed-distance rounded rects with borders,
//! Gaussian-blurred rounded-rect shadows, and atlas-sampled glyphs.

use crate::color::Color;
use crate::math::{vec2, Corners, Rect, Vec2};
use crate::theme::Shadow;
use lumen_text::{Galley, TextSystem};

/// Fragment shader mode stored per vertex.
pub mod mode {
    /// Plain fill (color interpolated).
    pub const SOLID: u32 = 0;
    /// SDF rounded rect with optional inside border.
    pub const RRECT: u32 = 1;
    /// Gaussian-blurred rounded rect.
    pub const SHADOW: u32 = 2;
    /// Coverage sampled from the glyph atlas red channel.
    pub const TEXT: u32 = 3;
}

/// 64-byte vertex. Positions are logical pixels; the renderer multiplies by
/// the scale factor.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vertex {
    pub pos: [f32; 2],
    /// Atlas texel coordinates (TEXT mode).
    pub uv: [f32; 2],
    /// Position relative to the shape's center (RRECT/SHADOW).
    pub local: [f32; 2],
    /// Half size of the shape.
    pub half: [f32; 2],
    /// Corner radii: top-left, top-right, bottom-right, bottom-left.
    pub radii: [f32; 4],
    /// RRECT: border width. SHADOW: sigma.
    pub param: f32,
    pub mode: u32,
    /// Straight-alpha sRGB fill color.
    pub color: [u8; 4],
    /// Straight-alpha sRGB border color.
    pub border_color: [u8; 4],
}

/// A batch of indices sharing a clip rect.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawCmd {
    /// Scissor rect in physical pixels: x, y, w, h.
    pub clip: [u32; 4],
    pub index_start: u32,
    pub index_count: u32,
}

/// Output of one frame: the only thing a renderer consumes.
#[derive(Clone, Debug, Default)]
pub struct DrawData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub cmds: Vec<DrawCmd>,
    /// Physical framebuffer size.
    pub size_px: [u32; 2],
    pub scale: f32,
    pub clear: Color,
    /// Glyph atlas update, if any.
    pub atlas: Option<lumen_text::AtlasDelta>,
}

/// Records primitives into [`DrawData`].
pub struct Painter {
    pub(crate) data: DrawData,
    clip_stack: Vec<Rect>,
    clip: Rect,
    scale: f32,
    screen: Vec2,
}

impl Painter {
    pub fn new(screen: Vec2, scale: f32, clear: Color) -> Self {
        let full = Rect::from_min_size(Vec2::ZERO, screen);
        let size_px = [(screen.x * scale).round() as u32, (screen.y * scale).round() as u32];
        Painter {
            data: DrawData { size_px, scale, clear, ..Default::default() },
            clip_stack: vec![],
            clip: full,
            scale,
            screen,
        }
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn screen(&self) -> Vec2 {
        self.screen
    }

    /// Round a coordinate to the physical pixel grid.
    pub fn snap(&self, v: f32) -> f32 {
        (v * self.scale).round() / self.scale
    }

    pub fn snap_rect(&self, r: Rect) -> Rect {
        Rect { min: vec2(self.snap(r.min.x), self.snap(r.min.y)), max: vec2(self.snap(r.max.x), self.snap(r.max.y)) }
    }

    pub fn clip_rect(&self) -> Rect {
        self.clip
    }

    /// Intersect the clip with `r` until the matching [`pop_clip`](Self::pop_clip).
    pub fn push_clip(&mut self, r: Rect) {
        self.clip_stack.push(self.clip);
        self.clip = self.clip.intersect(&r);
    }

    /// Replace the clip entirely (for top layers that escape parent clips).
    pub fn push_clip_replace(&mut self, r: Rect) {
        self.clip_stack.push(self.clip);
        self.clip = r.intersect(&Rect::from_min_size(Vec2::ZERO, self.screen));
    }

    pub fn pop_clip(&mut self) {
        self.clip = self.clip_stack.pop().unwrap_or(Rect::from_min_size(Vec2::ZERO, self.screen));
    }

    fn clip_px(&self) -> [u32; 4] {
        let s = self.scale;
        let c = self.clip;
        let x0 = (c.min.x * s).round().max(0.0);
        let y0 = (c.min.y * s).round().max(0.0);
        let x1 = (c.max.x * s).round().min(self.data.size_px[0] as f32).max(x0);
        let y1 = (c.max.y * s).round().min(self.data.size_px[1] as f32).max(y0);
        [x0 as u32, y0 as u32, (x1 - x0) as u32, (y1 - y0) as u32]
    }

    fn push_quad(&mut self, v: [Vertex; 4]) {
        if self.clip.is_empty() {
            return;
        }
        let clip = self.clip_px();
        let base = self.data.vertices.len() as u32;
        self.data.vertices.extend_from_slice(&v);
        let start = self.data.indices.len() as u32;
        self.data.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        match self.data.cmds.last_mut() {
            Some(c) if c.clip == clip => c.index_count += 6,
            _ => self.data.cmds.push(DrawCmd { clip, index_start: start, index_count: 6 }),
        }
    }

    /// Axis-aligned solid rectangle, snapped to pixels.
    pub fn rect(&mut self, r: Rect, color: Color) {
        self.rrect(r, Corners::all(0.0), color, 0.0, Color::TRANSPARENT);
    }

    /// Rounded rect with optional inside border. Edges snap to physical px.
    pub fn rrect(&mut self, r: Rect, radii: Corners, fill: Color, border: f32, border_color: Color) {
        if fill.a <= 0.0 && (border <= 0.0 || border_color.a <= 0.0) {
            return;
        }
        let r = self.snap_rect(r);
        self.rrect_unsnapped(r, radii, [fill; 2], border, border_color);
    }

    /// Rounded rect with a vertical two-stop gradient fill.
    pub fn rrect_gradient(&mut self, r: Rect, radii: Corners, top: Color, bottom: Color) {
        let r = self.snap_rect(r);
        self.rrect_unsnapped(r, radii, [top, bottom], 0.0, Color::TRANSPARENT);
    }

    /// Rounded rect at exact (fractional) coordinates, e.g. a moving thumb.
    pub fn rrect_unsnapped(&mut self, r: Rect, radii: Corners, fill: [Color; 2], border: f32, border_color: Color) {
        if r.width() <= 0.0 || r.height() <= 0.0 {
            return;
        }
        let half = r.size() * 0.5;
        let maxr = half.x.min(half.y);
        let radii = radii.0.map(|v| v.clamp(0.0, maxr));
        // Expand the quad by one physical pixel for anti-aliasing.
        let pad = 1.0 / self.scale;
        let q = r.expand(pad);
        let c = r.center();
        let bc = border_color.to_u8();
        let corner = |p: Vec2, color: Color| Vertex {
            pos: [p.x, p.y],
            uv: [0.0, 0.0],
            local: [p.x - c.x, p.y - c.y],
            half: [half.x, half.y],
            radii,
            param: border,
            mode: mode::RRECT,
            color: color.to_u8(),
            border_color: bc,
        };
        self.push_quad([
            corner(q.min, fill[0]),
            corner(vec2(q.max.x, q.min.y), fill[0]),
            corner(q.max, fill[1]),
            corner(vec2(q.min.x, q.max.y), fill[1]),
        ]);
    }

    /// Circle centered at `c`.
    pub fn circle(&mut self, c: Vec2, radius: f32, fill: Color, border: f32, border_color: Color) {
        let r = Rect::from_center_size(c, vec2(radius * 2.0, radius * 2.0));
        self.rrect_unsnapped(r, Corners::all(radius), [fill; 2], border, border_color);
    }

    /// Unfilled ring of `width` drawn inside `r`.
    pub fn ring(&mut self, r: Rect, radii: Corners, width: f32, color: Color) {
        if color.a <= 0.0 {
            return;
        }
        let r = self.snap_rect(r);
        self.rrect_unsnapped(r, radii, [Color::TRANSPARENT; 2], width, color);
    }

    /// CSS-style box shadow of a rounded rect.
    pub fn shadow(&mut self, r: Rect, radii: Corners, s: &Shadow) {
        if s.color.a <= 0.0 {
            return;
        }
        let shape = r.translate(vec2(s.x, s.y)).expand(s.spread);
        if shape.width() <= 0.0 || shape.height() <= 0.0 {
            return;
        }
        let radii = radii.grow(s.spread);
        let sigma = (s.blur / 2.0).max(0.01);
        let q = shape.expand(sigma * 3.0 + 1.0);
        let half = shape.size() * 0.5;
        let maxr = half.x.min(half.y);
        let c = shape.center();
        let col = s.color.to_u8();
        let v = |p: Vec2| Vertex {
            pos: [p.x, p.y],
            uv: [0.0, 0.0],
            local: [p.x - c.x, p.y - c.y],
            half: [half.x, half.y],
            radii: radii.0.map(|v| v.min(maxr)),
            param: sigma,
            mode: mode::SHADOW,
            color: col,
            border_color: [0; 4],
        };
        self.push_quad([v(q.min), v(vec2(q.max.x, q.min.y)), v(q.max), v(vec2(q.min.x, q.max.y))]);
    }

    pub fn shadows(&mut self, r: Rect, radii: Corners, layers: &[Shadow]) {
        for s in layers {
            self.shadow(r, radii, s);
        }
    }

    /// Draw a galley with its top-left at `pos`.
    pub fn text(&mut self, text: &mut TextSystem, galley: &Galley, pos: Vec2, color: Color) {
        if color.a <= 0.0 || galley.glyphs.is_empty() {
            return;
        }
        let s = self.scale;
        let col = color.to_u8();
        let mut quads = Vec::with_capacity(galley.glyphs.len());
        text.glyph_quads(galley, (pos.x, pos.y), s, |q| quads.push(q));
        for q in quads {
            let (x0, y0) = (q.x as f32 / s, q.y as f32 / s);
            let (x1, y1) = (x0 + q.glyph.w as f32 / s, y0 + q.glyph.h as f32 / s);
            let (u0, v0) = (q.glyph.x as f32, q.glyph.y as f32);
            let (u1, v1) = (u0 + q.glyph.w as f32, v0 + q.glyph.h as f32);
            let v = |x: f32, y: f32, u: f32, vv: f32| Vertex {
                pos: [x, y],
                uv: [u, vv],
                mode: mode::TEXT,
                color: col,
                ..Default::default()
            };
            self.push_quad([v(x0, y0, u0, v0), v(x1, y0, u1, v0), v(x1, y1, u1, v1), v(x0, y1, u0, v1)]);
        }
    }

    // ------------------------------------------------------------- meshes

    /// Append solid-color triangles (`SOLID` mode, color interpolated).
    pub fn mesh(&mut self, verts: &[(Vec2, Color)], indices: &[u32]) {
        if self.clip.is_empty() || verts.is_empty() || indices.is_empty() {
            return;
        }
        let clip = self.clip_px();
        let base = self.data.vertices.len() as u32;
        self.data.vertices.extend(verts.iter().map(|(p, c)| Vertex { pos: [p.x, p.y], mode: mode::SOLID, color: c.to_u8(), ..Default::default() }));
        let start = self.data.indices.len() as u32;
        self.data.indices.extend(indices.iter().map(|i| base + i));
        let n = indices.len() as u32;
        match self.data.cmds.last_mut() {
            Some(c) if c.clip == clip && c.index_start + c.index_count == start => c.index_count += n,
            _ => self.data.cmds.push(DrawCmd { clip, index_start: start, index_count: n }),
        }
    }

    /// Anti-aliased polyline of `width` logical px with mitered joins.
    pub fn polyline(&mut self, pts: &[Vec2], width: f32, color: Color) {
        if pts.len() < 2 || color.a <= 0.0 {
            return;
        }
        let aa = 1.0 / self.scale;
        let hw = (width / 2.0).max(aa / 2.0);
        // Thin lines fade instead of going below one pixel.
        let color = if width < aa { color.multiply_alpha(width / aa) } else { color };
        let n = pts.len();
        let normals: Vec<Vec2> = (0..n)
            .map(|i| {
                let d = |a: Vec2, b: Vec2| {
                    let v = b - a;
                    let l = v.length().max(1e-6);
                    vec2(-v.y / l, v.x / l)
                };
                let n0 = if i > 0 { Some(d(pts[i - 1], pts[i])) } else { None };
                let n1 = if i + 1 < n { Some(d(pts[i], pts[i + 1])) } else { None };
                match (n0, n1) {
                    (Some(a), Some(b)) => {
                        let m = a + b;
                        let l = m.length();
                        if l < 1e-4 {
                            a
                        } else {
                            let m = m * (1.0 / l);
                            let cos = (m.x * a.x + m.y * a.y).max(0.25);
                            m * (1.0 / cos)
                        }
                    }
                    (Some(a), None) | (None, Some(a)) => a,
                    _ => vec2(0.0, 1.0),
                }
            })
            .collect();
        let clear = color.with_alpha(0.0);
        let mut verts = Vec::with_capacity(n * 4);
        for i in 0..n {
            let (p, nm) = (pts[i], normals[i]);
            let inner = (hw - aa / 2.0).max(0.0);
            let outer = hw + aa / 2.0;
            verts.push((p + nm * outer, clear));
            verts.push((p + nm * inner, color));
            verts.push((p - nm * inner, color));
            verts.push((p - nm * outer, clear));
        }
        let mut idx = Vec::with_capacity((n - 1) * 18);
        for i in 0..(n - 1) as u32 {
            let a = i * 4;
            let b = a + 4;
            for k in 0..3 {
                idx.extend_from_slice(&[a + k, a + k + 1, b + k + 1, a + k, b + k + 1, b + k]);
            }
        }
        self.mesh(&verts, &idx);
    }

    /// Fill the band between a top polyline and a bottom polyline sampled
    /// at the same xs (area charts). `top` and `bottom` colors make a
    /// vertical gradient.
    pub fn fill_between(&mut self, upper: &[Vec2], lower: &[Vec2], top: Color, bottom: Color) {
        let n = upper.len().min(lower.len());
        if n < 2 {
            return;
        }
        let (y0, y1) = upper.iter().chain(lower.iter()).fold((f32::MAX, f32::MIN), |(a, b), p| (a.min(p.y), b.max(p.y)));
        let span = (y1 - y0).max(1e-3);
        let col = |p: Vec2| top.mix(bottom, (p.y - y0) / span);
        let mut verts = Vec::with_capacity(n * 2);
        for i in 0..n {
            verts.push((upper[i], col(upper[i])));
            verts.push((lower[i], col(lower[i])));
        }
        let mut idx = Vec::with_capacity((n - 1) * 6);
        for i in 0..(n - 1) as u32 {
            let a = i * 2;
            idx.extend_from_slice(&[a, a + 1, a + 3, a, a + 3, a + 2]);
        }
        self.mesh(&verts, &idx);
    }

    /// Anti-aliased ring segment (donut slice) from angle `a0` to `a1`
    /// (radians, 0 = 12 o'clock, clockwise).
    pub fn arc_band(&mut self, c: Vec2, r_inner: f32, r_outer: f32, a0: f32, a1: f32, color: Color) {
        if a1 <= a0 || color.a <= 0.0 || r_outer <= r_inner {
            return;
        }
        let aa = 1.0 / self.scale;
        let steps = (((a1 - a0) * r_outer / 3.0).ceil() as usize).clamp(2, 256);
        let clear = color.with_alpha(0.0);
        let radii = [r_outer + aa / 2.0, r_outer - aa / 2.0, r_inner + aa / 2.0, (r_inner - aa / 2.0).max(0.0)];
        let mut verts = Vec::with_capacity((steps + 1) * 4);
        for i in 0..=steps {
            let a = a0 + (a1 - a0) * i as f32 / steps as f32;
            let d = vec2(a.sin(), -a.cos());
            for (k, r) in radii.iter().enumerate() {
                verts.push((c + d * *r, if k == 0 || k == 3 { clear } else { color }));
            }
        }
        let mut idx = Vec::with_capacity(steps * 18);
        for i in 0..steps as u32 {
            let a = i * 4;
            let b = a + 4;
            for k in 0..3 {
                idx.extend_from_slice(&[a + k, a + k + 1, b + k + 1, a + k, b + k + 1, b + k]);
            }
        }
        self.mesh(&verts, &idx);
    }

    /// Thin dashed horizontal line (grid lines).
    pub fn dashed_hline(&mut self, x0: f32, x1: f32, y: f32, dash: f32, gap: f32, color: Color) {
        let y = self.snap(y);
        let px = 1.0 / self.scale;
        let mut x = x0;
        while x < x1 {
            let w = dash.min(x1 - x);
            self.rect(Rect::from_xywh(x, y, w, px.max(1.0)), color);
            x += dash + gap;
        }
    }

    pub fn finish(self) -> DrawData {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quads_batch_by_clip() {
        let mut p = Painter::new(vec2(100.0, 100.0), 2.0, Color::WHITE);
        p.rect(Rect::from_xywh(0.0, 0.0, 10.0, 10.0), Color::BLACK);
        p.rect(Rect::from_xywh(10.0, 0.0, 10.0, 10.0), Color::BLACK);
        p.push_clip(Rect::from_xywh(0.0, 0.0, 50.0, 50.0));
        p.rect(Rect::from_xywh(0.0, 20.0, 10.0, 10.0), Color::BLACK);
        p.pop_clip();
        let d = p.finish();
        assert_eq!(d.vertices.len(), 12);
        assert_eq!(d.cmds.len(), 2);
        assert_eq!(d.cmds[0].clip, [0, 0, 200, 200]);
        assert_eq!(d.cmds[1].clip, [0, 0, 100, 100]);
        assert_eq!(d.cmds[1].index_start, 12);
    }

    #[test]
    fn transparent_skipped_and_clip_empty_skipped() {
        let mut p = Painter::new(vec2(100.0, 100.0), 1.0, Color::WHITE);
        p.rect(Rect::from_xywh(0.0, 0.0, 10.0, 10.0), Color::TRANSPARENT);
        p.push_clip(Rect::from_xywh(200.0, 200.0, 5.0, 5.0));
        p.rect(Rect::from_xywh(0.0, 0.0, 10.0, 10.0), Color::BLACK);
        assert!(p.finish().vertices.is_empty());
    }

    #[test]
    fn snapping_uses_physical_pixels() {
        let p = Painter::new(vec2(100.0, 100.0), 2.0, Color::WHITE);
        assert_eq!(p.snap(10.3), 10.5);
        assert_eq!(p.snap(10.2), 10.0);
    }

    #[test]
    fn meshes_append_triangles() {
        let mut p = Painter::new(vec2(100.0, 100.0), 2.0, Color::WHITE);
        p.polyline(&[vec2(0.0, 0.0), vec2(50.0, 10.0), vec2(90.0, 0.0)], 2.0, Color::BLACK);
        let d = p.finish();
        assert_eq!(d.vertices.len(), 12);
        assert_eq!(d.indices.len(), 2 * 18);
        assert!(d.vertices.iter().all(|v| v.mode == mode::SOLID));
        let mut p = Painter::new(vec2(100.0, 100.0), 1.0, Color::WHITE);
        p.arc_band(vec2(50.0, 50.0), 20.0, 30.0, 0.0, 1.0, Color::BLACK);
        assert!(!p.finish().indices.is_empty());
    }

    #[test]
    fn vertex_is_64_bytes() {
        assert_eq!(std::mem::size_of::<Vertex>(), 64);
    }
}
