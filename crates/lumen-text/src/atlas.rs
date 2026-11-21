//! Single-channel (coverage) glyph atlas with incremental upload deltas.

/// Shelf packer: glyphs of similar height share a shelf. Growing the atlas
/// only extends shelves and adds room below, so nothing ever moves.
struct Shelves {
    width: u32,
    height: u32,
    shelves: Vec<Shelf>,
    next_y: u32,
}

struct Shelf {
    y: u32,
    h: u32,
    x: u32,
}

impl Shelves {
    fn new(size: u32) -> Self {
        Self { width: size, height: size, shelves: Vec::new(), next_y: 0 }
    }

    fn allocate(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        if w > self.width {
            return None;
        }
        // Best fit: the shortest shelf that is tall enough and wastes < 50%.
        let mut best: Option<usize> = None;
        for (i, s) in self.shelves.iter().enumerate() {
            if s.h >= h && s.h <= h + h / 2 + 2 && self.width - s.x >= w {
                if best.is_none_or(|b| self.shelves[b].h > s.h) {
                    best = Some(i);
                }
            }
        }
        if let Some(i) = best {
            let s = &mut self.shelves[i];
            let pos = (s.x, s.y);
            s.x += w;
            return Some(pos);
        }
        // Round shelf heights up to a multiple of 4 so similar sizes share.
        let sh = (h + 3) & !3;
        if self.next_y + sh > self.height {
            return None;
        }
        self.shelves.push(Shelf { y: self.next_y, h: sh, x: w });
        let pos = (0, self.next_y);
        self.next_y += sh;
        Some(pos)
    }

    fn grow(&mut self, size: u32) {
        self.width = size;
        self.height = size;
    }
}

/// A glyph's location in the atlas plus its placement relative to the pen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtlasGlyph {
    /// Top-left texel in the atlas.
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    /// Offset from pen position to the bitmap's left edge (physical px).
    pub left: i32,
    /// Offset from baseline up to the bitmap's top edge (physical px).
    pub top: i32,
    /// A color glyph (emoji): the texels are its colors, not a coverage
    /// mask to tint.
    pub color: bool,
}

/// Region of the atlas the renderer must (re)upload.
#[derive(Clone, Debug)]
pub struct AtlasDelta {
    /// Full atlas size; if it differs from the GPU texture, recreate it.
    pub atlas_size: u32,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    /// `w * h` premultiplied RGBA texels, row-major (mask glyphs are white
    /// with alpha = coverage).
    pub data: Vec<u8>,
}

pub struct Atlas {
    size: u32,
    data: Vec<u8>,
    alloc: Shelves,
    dirty: Option<[u32; 4]>,
}

impl Atlas {
    pub fn new(size: u32) -> Self {
        let mut a = Self {
            size,
            data: vec![0; (size * size * 4) as usize],
            alloc: Shelves::new(size),
            dirty: None,
        };
        // Reserve a fully-opaque 2x2 texel block at the origin so solid
        // geometry can sample "white" if a renderer wants it.
        if let Some((x, y)) = a.insert(2, 2, &[255; 4]) {
            debug_assert_eq!((x, y), (1, 1));
        }
        a
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Copy a `w*h` coverage bitmap into the atlas, growing it if needed.
    /// Existing glyph positions never move, so UVs stay valid.
    pub fn insert(&mut self, w: u32, h: u32, coverage: &[u8]) -> Option<(u32, u32)> {
        // Masks are stored as white with alpha = coverage.
        let rgba: Vec<u8> = coverage.iter().flat_map(|&c| [255, 255, 255, c]).collect();
        self.insert_rgba(w, h, &rgba)
    }

    /// Insert premultiplied RGBA texels (color glyphs).
    pub fn insert_rgba(&mut self, w: u32, h: u32, rgba: &[u8]) -> Option<(u32, u32)> {
        // 1px padding on every side avoids bleeding under bilinear sampling.
        let (x, y) = loop {
            if let Some(a) = self.alloc.allocate(w + 2, h + 2) {
                break a;
            }
            if self.size >= 8192 {
                return None;
            }
            self.grow();
        };
        let (x, y) = (x + 1, y + 1);
        for row in 0..h {
            let dst = (((y + row) * self.size + x) * 4) as usize;
            let src = (row * w * 4) as usize;
            self.data[dst..dst + (w * 4) as usize].copy_from_slice(&rgba[src..src + (w * 4) as usize]);
        }
        self.mark_dirty(x, y, w, h);
        Some((x, y))
    }

    fn grow(&mut self) {
        let old = self.size;
        let new = old * 2;
        let mut data = vec![0u8; (new * new * 4) as usize];
        for row in 0..old {
            let s = (row * old * 4) as usize;
            let d = (row * new * 4) as usize;
            data[d..d + (old * 4) as usize].copy_from_slice(&self.data[s..s + (old * 4) as usize]);
        }
        self.data = data;
        self.size = new;
        self.alloc.grow(new);
        self.dirty = Some([0, 0, new, new]);
    }

    fn mark_dirty(&mut self, x: u32, y: u32, w: u32, h: u32) {
        self.dirty = Some(match self.dirty {
            None => [x, y, x + w, y + h],
            Some([a, b, c, d]) => [a.min(x), b.min(y), c.max(x + w), d.max(y + h)],
        });
    }

    /// Mark the whole atlas dirty (e.g. after the GPU context was lost).
    pub fn invalidate(&mut self) {
        self.dirty = Some([0, 0, self.size, self.size]);
    }

    pub fn take_delta(&mut self) -> Option<AtlasDelta> {
        let [x0, y0, x1, y1] = self.dirty.take()?;
        let (w, h) = (x1 - x0, y1 - y0);
        let mut data = Vec::with_capacity((w * h * 4) as usize);
        for row in y0..y1 {
            let s = ((row * self.size + x0) * 4) as usize;
            data.extend_from_slice(&self.data[s..s + (w * 4) as usize]);
        }
        Some(AtlasDelta { atlas_size: self.size, x: x0, y: y0, w, h, data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grows_without_moving_glyphs() {
        let mut a = Atlas::new(64);
        let first = a.insert(10, 10, &[7; 100]).unwrap();
        for _ in 0..40 {
            a.insert(20, 20, &[1; 400]).unwrap();
        }
        assert!(a.size() > 64);
        let s = a.size();
        assert_eq!(a.data()[((first.1 * s + first.0) * 4 + 3) as usize], 7, "coverage lives in alpha");
        let d = a.take_delta().unwrap();
        assert_eq!(d.atlas_size, s);
        assert_eq!((d.w, d.h), (s, s));
    }
}
