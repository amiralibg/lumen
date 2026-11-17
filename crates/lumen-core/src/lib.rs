//! lumen-core: the immediate-mode heart of lumen.
//!
//! Owns ids, input, layout, theme tokens and springs, and produces
//! [`paint::DrawData`] (triangles plus clip commands). It never touches a
//! window or GPU.

pub mod anim;
pub mod color;
pub mod date;
pub mod id;
pub mod input;
pub mod layout;
pub mod look;
pub mod math;
pub mod paint;
pub mod theme;

pub use color::Color;
pub use date::{Date, Time};
pub use id::Id;
pub use math::{vec2, Rect, Sides, Vec2};
pub use theme::{Accent, Mode, Token};
pub use look::{Density, Fonts, Look, Motion, Neutrals, Platform};

/// Hash map with a fast, deterministic hasher (no runtime RNG, works on wasm).
pub type FastMap<K, V> = std::collections::HashMap<K, V, std::hash::BuildHasherDefault<ahash::AHasher>>;

pub mod ui;
pub mod widgets;

pub use anim::{Ease, SpringParams};
pub use input::{CursorIcon, Event, Key, Modifiers, PointerButton, RawInput};
pub use layout::{Align, Justify, Size};
pub use theme::{springs, ControlSize, TextRole};
pub use ui::{FrameOutput, LayerAlign, LayerPlace, Placement, Response, ScrollInfo, Shown, SystemPrefs, Ui};
pub use widgets::{ChartKind, Command, Series, Toast, Tone};
pub use widgets::Variant;
