//! Platform-independent input events. The app layer translates window
//! events into these; tests construct them directly.

use crate::math::Vec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    /// The Command key on macOS, the Windows/Super key elsewhere.
    pub logo: bool,
    /// True on macOS: shortcuts use Command and word jumps use Alt.
    pub mac: bool,
}

impl Modifiers {
    /// The platform shortcut modifier (Cmd on macOS, Ctrl elsewhere).
    pub fn command(&self) -> bool {
        if self.mac {
            self.logo
        } else {
            self.ctrl
        }
    }
    /// The word-jump modifier (Alt on macOS, Ctrl elsewhere).
    pub fn word(&self) -> bool {
        if self.mac {
            self.alt
        } else {
            self.ctrl
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Tab,
    Enter,
    Space,
    Escape,
    Backspace,
    Delete,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Home,
    End,
    PageUp,
    PageDown,
    /// A character key, lowercase (for shortcuts such as Cmd+A).
    Char(char),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    PointerMoved(Vec2),
    PointerButton { pos: Vec2, button: PointerButton, pressed: bool },
    PointerLeft,
    /// Scroll delta in logical pixels (positive y scrolls content up).
    Scroll(Vec2),
    Key { key: Key, pressed: bool, repeat: bool, modifiers: Modifiers },
    /// Typed text.
    Text(String),
    /// Input-method composition: CJK input, dead keys, press-and-hold accents.
    Ime(Ime),
    /// Clipboard contents for a paste the app performed on Cmd/Ctrl+V.
    Paste(String),
    WindowFocused(bool),
    /// The window was maximized (zoomed) or restored. Hosts send it when the
    /// state changes so a custom title bar can show restore instead of
    /// maximize.
    WindowMaximized(bool),
    /// A screen reader asked for an action on a widget (see [`crate::a11y`]).
    A11y { target: crate::Id, action: crate::a11y::A11yAction },
}

/// An input-method event for the focused text field.
///
/// lumen keeps composition text inside the field's value, marked and
/// underlined, until it's committed, the way browsers do. Byte ranges refer
/// to the field's value as reported in [`crate::TextFieldState`].
#[derive(Clone, Debug, PartialEq)]
pub enum Ime {
    /// Show `text` as the composition. It replaces the current composition,
    /// else `replace`, else the selection. `cursor` is the selection inside
    /// `text` in bytes (`None` puts the caret at its end). Empty text
    /// cancels the composition.
    Preedit { text: String, cursor: Option<(usize, usize)>, replace: Option<std::ops::Range<usize>> },
    /// Final text. It replaces the composition, else `replace`, else the
    /// selection.
    Commit { text: String, replace: Option<std::ops::Range<usize>> },
}

/// Everything the UI needs for one frame.
#[derive(Clone, Debug)]
pub struct RawInput {
    /// Window size in logical pixels.
    pub screen_size: Vec2,
    /// Physical pixels per logical pixel.
    pub scale: f32,
    /// Seconds since an arbitrary epoch, monotonic.
    pub time: f64,
    pub events: Vec<Event>,
    pub modifiers: Modifiers,
}

impl Default for RawInput {
    fn default() -> Self {
        Self { screen_size: Vec2 { x: 800.0, y: 600.0 }, scale: 1.0, time: 0.0, events: vec![], modifiers: Modifiers::default() }
    }
}

/// Mouse cursor the app should show.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CursorIcon {
    #[default]
    Default,
    Pointer,
    Text,
    Grab,
    Grabbing,
    /// Left-right arrows: scrubbing a number.
    ResizeHorizontal,
    /// Drawing: signature pads and canvases.
    Crosshair,
}
