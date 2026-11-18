//! Batch-1 widgets (docs/widgets-batch1.md).

pub mod button;
pub mod edit;
pub mod card;
pub mod checkbox;
pub mod nav;
pub mod slider;
pub mod switch;
pub mod text;
pub mod text_input;
pub mod picker;
pub mod segmented;
pub mod popover;
pub mod menu;
pub mod select;
pub mod toast;
pub mod dialog;
pub mod tabs;
pub mod misc;
pub mod calendar;
pub mod chart;
pub mod choice;
pub mod field;
pub mod combo;
pub mod time_picker;
pub mod color_picker;
pub mod overlays;
pub mod palette;
pub mod inputs;
pub mod number_field;
pub mod otp;
pub mod feedback;
pub mod navigation;
pub mod selection;
pub mod actions_chrome;
pub mod data;
pub mod code_data;
pub mod charts_extra;
pub mod planning;
pub mod ai_devtools;
pub mod motion_chrome;
pub mod workspace_pro;
pub mod rich_views;

pub use button::{Button, Variant};
pub use card::Card;
pub use checkbox::Checkbox;
pub use nav::{NavItem, Sidebar};
pub use slider::Slider;
pub use switch::Switch;
pub use text::{Heading, Label};
pub use text_input::TextInput;
pub use popover::Popover;
pub use menu::MenuItem;
pub use select::Select;
pub use toast::{Toast, Tone};
pub use dialog::Dialog;
pub use chart::{Chart, ChartKind, Series};
pub use combo::{Combobox, MultiSelect};
pub use time_picker::TimePicker;
pub use color_picker::ColorPicker;
pub use overlays::{ConfirmPopover, HoverCard, Sheet};
pub use palette::{Command, CommandPalette};
pub use inputs::{password_strength, PasswordInput, SearchField, TagInput, TextArea};
pub use number_field::NumberField;
pub use otp::OtpInput;
pub use feedback::{Alert, EmptyState, Keycap, Skeleton};
pub use navigation::{Breadcrumb, Pagination, Steps};
pub use selection::{RadioCard, RangeSlider, Rating, ShortcutRecorder};
pub use actions_chrome::{Dropzone, HoldButton, Meter, SplitButton, StatusBarScope, ToolbarScope};
pub use data::{Accordion, AccordionItem, Collapsible, SortDir, Table, TableCell, TableCol, TimelineItem, Tree, TreeNode};
pub use code_data::{CodeBlock, JsonViewer, KeyValueEditor, KvPair, LogEntry, LogLevel, LogViewer};
pub use charts_extra::{BarListItem, Candle, CandlestickChart, FunnelChart, Heatmap, ProgressRing, RadarChart, RingSpec, ScatterChart, ScatterPoint, ScatterSeries};
pub use planning::{KanbanCard, KanbanCol};
pub use ai_devtools::{ChatMessage, ChatRole, ComposerOut, DiffViewer, PromptComposer, ToolCall, ToolStatus, TraceSpan, UptimeStatus};
pub use motion_chrome::{DockItem, RollingNumber, Waveform, WindowChrome};
pub use workspace_pro::{FlameSpan, MenuBarScope, PropertyInspector, SplitPane, TermLine, TermTone, Terminal};
pub use rich_views::{GanttTask, NotificationItem, SpotlightCard, Treemap, TreemapItem};

use crate::color::Color;
use crate::theme::Token;
use crate::ui::{Interact, Ui};

/// Rest/hover/pressed blend of three colors with hover and press weights.
pub(crate) fn blend3(rest: Color, hover: Color, pressed: Color, h: f32, p: f32) -> Color {
    rest.mix(hover, h).mix(pressed, p)
}

/// Neutral state layer: `base` + HOVER / PRESSED overlays (pre-composited).
pub(crate) fn neutral_layers(ui: &Ui, base: Color) -> (Color, Color) {
    let hover = ui.color(Token::Hover);
    let pressed = ui.color(Token::Pressed);
    if base.a <= 0.0 {
        (hover, pressed)
    } else {
        (base.over(hover), base.over(pressed))
    }
}

/// Space/Enter activation for a focused widget. Returns (activated, held).
pub(crate) fn key_activate(ui: &Ui, ix: &Interact, enter: bool) -> (bool, bool) {
    use crate::input::Key;
    if !ix.focused || ix.disabled {
        return (false, false);
    }
    let act = ui.key_pressed_for(ix.id, Key::Space) || (enter && ui.key_pressed_for(ix.id, Key::Enter));
    let held = ui.key_down(Key::Space) || (enter && ui.key_down(Key::Enter));
    (act, held)
}

/// Implements "show on drop" for builder widgets.
macro_rules! show_on_drop {
    ($t:ident) => {
        impl Drop for $t<'_> {
            fn drop(&mut self) {
                if !self.shown {
                    self.shown = true;
                    self.show_inner();
                }
            }
        }
        impl $t<'_> {
            /// Add the widget and return its response.
            pub fn show(mut self) -> crate::ui::Response {
                self.shown = true;
                self.show_inner()
            }
            /// Add the widget; true if it was clicked this frame.
            pub fn clicked(self) -> bool {
                self.show().clicked
            }
            /// Add the widget; true if its value changed this frame.
            pub fn changed(self) -> bool {
                self.show().changed
            }
        }
    };
}
pub(crate) use show_on_drop;
