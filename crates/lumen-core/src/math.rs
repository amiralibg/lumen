//! Small geometry types. All coordinates are logical pixels, y down.

use std::ops::{Add, AddAssign, Mul, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[inline]
pub const fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}

impl Vec2 {
    pub const ZERO: Vec2 = vec2(0.0, 0.0);
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, o: Vec2) -> Vec2 {
        vec2(self.x + o.x, self.y + o.y)
    }
}
impl AddAssign for Vec2 {
    fn add_assign(&mut self, o: Vec2) {
        self.x += o.x;
        self.y += o.y;
    }
}
impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, o: Vec2) -> Vec2 {
        vec2(self.x - o.x, self.y - o.y)
    }
}
impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, s: f32) -> Vec2 {
        vec2(self.x * s, self.y * s)
    }
}

/// Axis-aligned rectangle (min inclusive, max exclusive).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub const ZERO: Rect = Rect { min: Vec2::ZERO, max: Vec2::ZERO };
    pub const EVERYTHING: Rect = Rect { min: vec2(-1e9, -1e9), max: vec2(1e9, 1e9) };

    pub fn from_min_size(min: Vec2, size: Vec2) -> Rect {
        Rect { min, max: min + size }
    }
    pub fn from_xywh(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { min: vec2(x, y), max: vec2(x + w, y + h) }
    }
    pub fn from_center_size(c: Vec2, size: Vec2) -> Rect {
        Rect { min: c - size * 0.5, max: c + size * 0.5 }
    }
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }
    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.min.x && p.x < self.max.x && p.y >= self.min.y && p.y < self.max.y
    }
    /// Grow by `d` on every side (negative shrinks).
    /// True if `o` lies entirely inside this rect.
    pub fn contains_rect(&self, o: &Rect) -> bool {
        o.min.x >= self.min.x && o.min.y >= self.min.y && o.max.x <= self.max.x && o.max.y <= self.max.y
    }
    pub fn expand(&self, d: f32) -> Rect {
        Rect { min: self.min - vec2(d, d), max: self.max + vec2(d, d) }
    }
    pub fn expand2(&self, dx: f32, dy: f32) -> Rect {
        Rect { min: self.min - vec2(dx, dy), max: self.max + vec2(dx, dy) }
    }
    pub fn translate(&self, d: Vec2) -> Rect {
        Rect { min: self.min + d, max: self.max + d }
    }
    pub fn intersect(&self, o: &Rect) -> Rect {
        Rect {
            min: vec2(self.min.x.max(o.min.x), self.min.y.max(o.min.y)),
            max: vec2(self.max.x.min(o.max.x), self.max.y.min(o.max.y)),
        }
    }
    pub fn union(&self, o: &Rect) -> Rect {
        Rect {
            min: vec2(self.min.x.min(o.min.x), self.min.y.min(o.min.y)),
            max: vec2(self.max.x.max(o.max.x), self.max.y.max(o.max.y)),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.max.x <= self.min.x || self.max.y <= self.min.y
    }
}

/// Per-side insets (padding).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sides {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Sides {
    pub const fn all(v: f32) -> Sides {
        Sides { left: v, right: v, top: v, bottom: v }
    }
    pub const fn xy(x: f32, y: f32) -> Sides {
        Sides { left: x, right: x, top: y, bottom: y }
    }
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

impl From<f32> for Sides {
    fn from(v: f32) -> Self {
        Sides::all(v)
    }
}
impl From<i32> for Sides {
    fn from(v: i32) -> Self {
        Sides::all(v as f32)
    }
}
impl From<(f32, f32)> for Sides {
    fn from((x, y): (f32, f32)) -> Self {
        Sides::xy(x, y)
    }
}
impl From<(i32, i32)> for Sides {
    fn from((x, y): (i32, i32)) -> Self {
        Sides::xy(x as f32, y as f32)
    }
}

/// Per-corner radii: top-left, top-right, bottom-right, bottom-left.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Corners(pub [f32; 4]);

impl Corners {
    pub const fn all(r: f32) -> Corners {
        Corners([r; 4])
    }
    pub fn shrink(self, d: f32) -> Corners {
        Corners(self.0.map(|r| (r - d).max(0.0)))
    }
    pub fn grow(self, d: f32) -> Corners {
        Corners(self.0.map(|r| (r + d).max(0.0)))
    }
}

impl From<f32> for Corners {
    fn from(r: f32) -> Self {
        Corners::all(r)
    }
}

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
