//! lumen-core: the immediate-mode heart of lumen.
//!
//! Owns ids, input, layout, theme tokens and springs, and produces
//! [`paint::DrawData`] (triangles plus clip commands). It never touches a
//! window or GPU.

pub mod anim;
pub mod color;
pub mod id;
pub mod input;
pub mod layout;
pub mod math;
pub mod paint;
pub mod theme;

pub use color::Color;
pub use id::Id;
pub use math::{vec2, Rect, Sides, Vec2};
pub use theme::{Accent, Mode, Token};

/// Hash map with a fast, deterministic hasher (no runtime RNG, works on wasm).
pub type FastMap<K, V> = std::collections::HashMap<K, V, std::hash::BuildHasherDefault<ahash::AHasher>>;
