//! Accessibility: what each widget is, for screen readers.
//!
//! Widgets describe themselves with [`Ui::a11y`](crate::Ui::a11y): a
//! [`Role`], a label, a value and state, attached to their layout node.
//! After layout, [`Ui::end_frame`](crate::Ui::end_frame) walks the tree and
//! returns an [`A11yTree`] in [`FrameOutput::a11y`](crate::FrameOutput):
//!
//! - text inside a control (a button's caption) becomes the control's
//!   name when it has no label of its own;
//! - any other text becomes a [`Role::Label`] node;
//! - interactive nodes that never described themselves show up as
//!   generic buttons, named by their text, so nothing clickable is
//!   invisible to a screen reader.
//!
//! Building the tree costs time, so it only happens after a host calls
//! [`Ui::set_accessibility`](crate::Ui::set_accessibility) (when a screen
//! reader asks for it). Hosts turn the tree into AccessKit updates
//! (the `accesskit` feature adds [`A11yTree::to_accesskit`]) and send
//! actions back as [`Event::A11y`](crate::Event::A11y).

use crate::math::Rect;

/// What a node is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Role {
    Window,
    #[default]
    Group,
    /// Static text.
    Label,
    /// A section title; see [`A11y::level`].
    Heading,
    Paragraph,
    Button,
    Link,
    CheckBox,
    Switch,
    RadioButton,
    RadioGroup,
    TextInput,
    MultilineTextInput,
    SearchInput,
    PasswordInput,
    Slider,
    /// A number field with steppers.
    SpinButton,
    ProgressIndicator,
    Meter,
    /// A select or combo box: shows a value, opens a list.
    ComboBox,
    ListBox,
    ListBoxOption,
    List,
    ListItem,
    Menu,
    MenuBar,
    MenuItem,
    MenuItemCheckBox,
    MenuItemRadio,
    Dialog,
    AlertDialog,
    Alert,
    Status,
    TabList,
    Tab,
    TabPanel,
    Table,
    Row,
    Cell,
    ColumnHeader,
    Tree,
    TreeItem,
    Toolbar,
    Tooltip,
    Image,
    ScrollView,
    Separator,
    Navigation,
    Grid,
    Canvas,
}

impl Role {
    /// Controls whose text children name them instead of being read on
    /// their own.
    pub fn named_by_contents(self) -> bool {
        use Role::*;
        matches!(
            self,
            Button
                | Link
                | CheckBox
                | Switch
                | RadioButton
                | ListBoxOption
                | ListItem
                | MenuItem
                | MenuItemCheckBox
                | MenuItemRadio
                | Tab
                | TreeItem
                | Cell
                | ColumnHeader
                | Heading
                | Label
                | Tooltip
                | ComboBox
                | Row
                | Status
                | Alert
        )
    }

    /// Text inputs and value controls: their own text is the value, not
    /// separate labels.
    pub fn owns_text(self) -> bool {
        use Role::*;
        matches!(self, TextInput | MultilineTextInput | SearchInput | PasswordInput | Slider | SpinButton | ProgressIndicator | Meter)
    }

    fn clickable(self) -> bool {
        use Role::*;
        matches!(
            self,
            Button | Link | CheckBox | Switch | RadioButton | ListBoxOption | ListItem | MenuItem | MenuItemCheckBox | MenuItemRadio | Tab | TreeItem | ComboBox | Row
        )
    }
}

/// Tri-state for checkboxes and switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Toggled {
    False,
    True,
    Mixed,
}

impl From<bool> for Toggled {
    fn from(b: bool) -> Self {
        if b {
            Toggled::True
        } else {
            Toggled::False
        }
    }
}

/// A number a slider, stepper or progress bar reports.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Numeric {
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: Option<f64>,
}

/// Actions assistive technology can request.
#[derive(Clone, Debug, PartialEq)]
pub enum A11yAction {
    /// Activate (press a button, toggle a switch, pick an option).
    Click,
    Focus,
    Blur,
    Increment,
    Decrement,
    /// Replace a text field's value, or set a slider's number.
    SetValue(String),
    SetNumber(f64),
    Expand,
    Collapse,
    ScrollIntoView,
}

/// Which actions a node supports.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Actions {
    pub click: bool,
    pub focus: bool,
    pub increment: bool,
    pub set_value: bool,
    pub expand: bool,
}

/// What a widget says about itself. Build with [`A11y::new`] and the
/// chained setters; pass to [`Ui::a11y`](crate::Ui::a11y).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct A11y {
    pub role: Role,
    pub label: Option<String>,
    pub value: Option<String>,
    pub description: Option<String>,
    pub numeric: Option<Numeric>,
    pub toggled: Option<Toggled>,
    pub selected: Option<bool>,
    pub expanded: Option<bool>,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub invalid: bool,
    pub modal: bool,
    /// Heading level, 1–6.
    pub level: Option<u8>,
    /// Position in its set (1-based) and the set's size: a virtualized
    /// list builds only visible rows but still reports "row 40 of 100000".
    pub position: Option<(usize, usize)>,
    /// Hide this node and its subtree from assistive technology
    /// (decoration, or a duplicate of something already exposed).
    pub hidden: bool,
    /// Extra actions beyond what the role implies.
    pub actions: Actions,
    /// Placeholder text for empty inputs.
    pub placeholder: Option<String>,
}

impl A11y {
    pub fn new(role: Role) -> Self {
        A11y { role, ..Default::default() }
    }
    pub fn hidden() -> Self {
        A11y { hidden: true, ..Default::default() }
    }
    pub fn label(mut self, s: impl Into<String>) -> Self {
        let s = s.into();
        if !s.is_empty() {
            self.label = Some(s);
        }
        self
    }
    pub fn value(mut self, s: impl Into<String>) -> Self {
        self.value = Some(s.into());
        self
    }
    pub fn description(mut self, s: impl Into<String>) -> Self {
        let s = s.into();
        if !s.is_empty() {
            self.description = Some(s);
        }
        self
    }
    pub fn numeric(mut self, value: f64, min: f64, max: f64, step: Option<f64>) -> Self {
        self.numeric = Some(Numeric { value, min, max, step });
        self.actions.increment = true;
        self
    }
    pub fn toggled(mut self, t: impl Into<Toggled>) -> Self {
        self.toggled = Some(t.into());
        self
    }
    pub fn selected(mut self, b: bool) -> Self {
        self.selected = Some(b);
        self
    }
    pub fn expanded(mut self, b: bool) -> Self {
        self.expanded = Some(b);
        self.actions.expand = true;
        self
    }
    pub fn disabled(mut self, b: bool) -> Self {
        self.disabled = b;
        self
    }
    pub fn read_only(mut self, b: bool) -> Self {
        self.read_only = b;
        self
    }
    pub fn required(mut self, b: bool) -> Self {
        self.required = b;
        self
    }
    pub fn invalid(mut self, b: bool) -> Self {
        self.invalid = b;
        self
    }
    pub fn modal(mut self, b: bool) -> Self {
        self.modal = b;
        self
    }
    pub fn level(mut self, l: u8) -> Self {
        self.level = Some(l.clamp(1, 6));
        self
    }
    pub fn position(mut self, index: usize, size: usize) -> Self {
        self.position = Some((index + 1, size));
        self
    }
    pub fn placeholder(mut self, s: impl Into<String>) -> Self {
        let s = s.into();
        if !s.is_empty() {
            self.placeholder = Some(s);
        }
        self
    }
    pub fn settable(mut self) -> Self {
        self.actions.set_value = true;
        self
    }
}

/// One node of the exported tree.
#[derive(Clone, Debug, PartialEq)]
pub struct A11yNode {
    /// Stable across frames (the widget's [`Id`](crate::Id) when it has one).
    pub id: u64,
    pub info: A11y,
    /// Logical pixels, window space.
    pub rect: Rect,
    pub children: Vec<u64>,
    pub actions: Actions,
}

/// The whole window, rebuilt each frame while accessibility is on.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct A11yTree {
    /// Root first; every node appears once.
    pub nodes: Vec<A11yNode>,
    pub root: u64,
    /// The focused node, or the root.
    pub focus: u64,
}

impl A11yTree {
    pub fn get(&self, id: u64) -> Option<&A11yNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Find the first node with this role and label (tests, debugging).
    pub fn find(&self, role: Role, label: &str) -> Option<&A11yNode> {
        self.nodes.iter().find(|n| n.info.role == role && n.info.label.as_deref() == Some(label))
    }

    /// Indented text dump: `role "label" = value [state]`.
    pub fn dump(&self) -> String {
        let mut out = String::new();
        let map: std::collections::HashMap<u64, &A11yNode> = self.nodes.iter().map(|n| (n.id, n)).collect();
        fn walk(map: &std::collections::HashMap<u64, &A11yNode>, id: u64, depth: usize, focus: u64, out: &mut String) {
            let Some(n) = map.get(&id) else { return };
            let i = &n.info;
            out.push_str(&"  ".repeat(depth));
            out.push_str(&format!("{:?}", i.role));
            if let Some(l) = &i.label {
                out.push_str(&format!(" {l:?}"));
            }
            if let Some(v) = &i.value {
                out.push_str(&format!(" = {v:?}"));
            }
            if let Some(nm) = &i.numeric {
                out.push_str(&format!(" = {}", nm.value));
            }
            let mut st = vec![];
            if let Some(t) = i.toggled {
                st.push(format!("{t:?}").to_lowercase());
            }
            if i.selected == Some(true) {
                st.push("selected".into());
            }
            if let Some(e) = i.expanded {
                st.push(if e { "expanded".into() } else { "collapsed".into() });
            }
            if i.disabled {
                st.push("disabled".into());
            }
            if i.modal {
                st.push("modal".into());
            }
            if let Some((p, s)) = i.position {
                st.push(format!("{p} of {s}"));
            }
            if id == focus {
                st.push("focused".into());
            }
            if !st.is_empty() {
                out.push_str(&format!(" [{}]", st.join(", ")));
            }
            out.push('\n');
            for c in &n.children {
                walk(map, *c, depth + 1, focus, out);
            }
        }
        walk(&map, self.root, 0, self.focus, &mut out);
        out
    }

    /// Controls a screen reader can't name: something you can press or
    /// focus with no label, value or placeholder.
    pub fn problems(&self) -> Vec<String> {
        self.nodes
            .iter()
            .filter(|n| (n.actions.click || n.actions.focus) && n.info.label.is_none() && n.info.placeholder.is_none() && !n.info.hidden)
            .map(|n| format!("unnamed {:?} at {:.0},{:.0} {:.0}x{:.0}", n.info.role, n.rect.min.x, n.rect.min.y, n.rect.width(), n.rect.height()))
            .collect()
    }

    pub(crate) fn default_actions(info: &A11y, clickable: bool, focusable: bool) -> Actions {
        let mut a = info.actions;
        a.click |= clickable && (info.role.clickable() || info.role == Role::Group) && !info.disabled;
        a.focus |= focusable && !info.disabled;
        if matches!(info.role, Role::TextInput | Role::MultilineTextInput | Role::SearchInput | Role::PasswordInput) && !info.read_only {
            a.set_value = true;
        }
        if info.numeric.is_some() && matches!(info.role, Role::Slider | Role::SpinButton) {
            a.increment = true;
            a.set_value = true;
        }
        a
    }
}

#[cfg(feature = "accesskit")]
mod ak {
    use super::*;
    use accesskit as k;

    fn role(r: Role) -> k::Role {
        match r {
            Role::Window => k::Role::Window,
            Role::Group => k::Role::Group,
            Role::Label => k::Role::Label,
            Role::Heading => k::Role::Heading,
            Role::Paragraph => k::Role::Paragraph,
            Role::Button => k::Role::Button,
            Role::Link => k::Role::Link,
            Role::CheckBox => k::Role::CheckBox,
            Role::Switch => k::Role::Switch,
            Role::RadioButton => k::Role::RadioButton,
            Role::RadioGroup => k::Role::RadioGroup,
            Role::TextInput => k::Role::TextInput,
            Role::MultilineTextInput => k::Role::MultilineTextInput,
            Role::SearchInput => k::Role::SearchInput,
            Role::PasswordInput => k::Role::PasswordInput,
            Role::Slider => k::Role::Slider,
            Role::SpinButton => k::Role::SpinButton,
            Role::ProgressIndicator => k::Role::ProgressIndicator,
            Role::Meter => k::Role::Meter,
            Role::ComboBox => k::Role::ComboBox,
            Role::ListBox => k::Role::ListBox,
            Role::ListBoxOption => k::Role::ListBoxOption,
            Role::List => k::Role::List,
            Role::ListItem => k::Role::ListItem,
            Role::Menu => k::Role::Menu,
            Role::MenuBar => k::Role::MenuBar,
            Role::MenuItem => k::Role::MenuItem,
            Role::MenuItemCheckBox => k::Role::MenuItemCheckBox,
            Role::MenuItemRadio => k::Role::MenuItemRadio,
            Role::Dialog => k::Role::Dialog,
            Role::AlertDialog => k::Role::AlertDialog,
            Role::Alert => k::Role::Alert,
            Role::Status => k::Role::Status,
            Role::TabList => k::Role::TabList,
            Role::Tab => k::Role::Tab,
            Role::TabPanel => k::Role::TabPanel,
            Role::Table => k::Role::Table,
            Role::Row => k::Role::Row,
            Role::Cell => k::Role::Cell,
            Role::ColumnHeader => k::Role::ColumnHeader,
            Role::Tree => k::Role::Tree,
            Role::TreeItem => k::Role::TreeItem,
            Role::Toolbar => k::Role::Toolbar,
            Role::Tooltip => k::Role::Tooltip,
            Role::Image => k::Role::Image,
            Role::ScrollView => k::Role::ScrollView,
            Role::Separator => k::Role::Splitter,
            Role::Navigation => k::Role::Navigation,
            Role::Grid => k::Role::Grid,
            Role::Canvas => k::Role::Canvas,
        }
    }

    impl A11yNode {
        /// This node as an AccessKit node. `scale` turns logical pixels
        /// into the physical ones AccessKit expects.
        pub fn to_accesskit(&self, scale: f32) -> k::Node {
            let i = &self.info;
            let mut n = k::Node::new(role(i.role));
            let s = scale as f64;
            n.set_bounds(k::Rect {
                x0: self.rect.min.x as f64 * s,
                y0: self.rect.min.y as f64 * s,
                x1: self.rect.max.x as f64 * s,
                y1: self.rect.max.y as f64 * s,
            });
            if !self.children.is_empty() {
                n.set_children(self.children.iter().map(|c| k::NodeId(*c)).collect::<Vec<_>>());
            }
            if let Some(l) = &i.label {
                n.set_label(l.as_str());
            }
            if let Some(v) = &i.value {
                n.set_value(v.as_str());
            }
            if let Some(d) = &i.description {
                n.set_description(d.as_str());
            }
            if let Some(p) = &i.placeholder {
                n.set_placeholder(p.as_str());
            }
            if let Some(m) = &i.numeric {
                n.set_numeric_value(m.value);
                n.set_min_numeric_value(m.min);
                n.set_max_numeric_value(m.max);
                if let Some(st) = m.step {
                    n.set_numeric_value_step(st);
                }
            }
            if let Some(t) = i.toggled {
                n.set_toggled(match t {
                    Toggled::False => k::Toggled::False,
                    Toggled::True => k::Toggled::True,
                    Toggled::Mixed => k::Toggled::Mixed,
                });
            }
            if let Some(sel) = i.selected {
                n.set_selected(sel);
            }
            if let Some(e) = i.expanded {
                n.set_expanded(e);
            }
            if i.disabled {
                n.set_disabled();
            }
            if i.read_only {
                n.set_read_only();
            }
            if i.required {
                n.set_required();
            }
            if i.invalid {
                n.set_invalid(k::Invalid::True);
            }
            if i.modal {
                n.set_modal();
            }
            if i.hidden {
                n.set_hidden();
            }
            if let Some(l) = i.level {
                n.set_level(l as usize);
            }
            if let Some((p, sz)) = i.position {
                n.set_position_in_set(p);
                n.set_size_of_set(sz);
            }
            let a = self.actions;
            if a.click {
                n.add_action(k::Action::Click);
            }
            if a.focus {
                n.add_action(k::Action::Focus);
            }
            if a.increment {
                n.add_action(k::Action::Increment);
                n.add_action(k::Action::Decrement);
            }
            if a.set_value {
                n.add_action(k::Action::SetValue);
            }
            if a.expand {
                n.add_action(k::Action::Expand);
                n.add_action(k::Action::Collapse);
            }
            n.add_action(k::Action::ScrollIntoView);
            n
        }
    }

    impl A11yTree {
        /// A full AccessKit update: every node, the tree and the focus.
        /// `title` labels the window node.
        pub fn to_accesskit(&self, scale: f32, title: &str) -> k::TreeUpdate {
            self.update_since(None, scale, title)
        }

        /// An update with only the nodes that changed since `prev` (all of
        /// them when `prev` is `None`, which also sends the tree itself).
        pub fn update_since(&self, prev: Option<&A11yTree>, scale: f32, title: &str) -> k::TreeUpdate {
            let old: std::collections::HashMap<u64, &A11yNode> =
                prev.map(|p| p.nodes.iter().map(|n| (n.id, n)).collect()).unwrap_or_default();
            let nodes = self
                .nodes
                .iter()
                .filter(|n| prev.is_none() || old.get(&n.id).is_none_or(|o| *o != *n))
                .map(|n| {
                    let mut node = n.to_accesskit(scale);
                    if n.id == self.root && n.info.label.is_none() && !title.is_empty() {
                        node.set_label(title);
                    }
                    (k::NodeId(n.id), node)
                })
                .collect();
            k::TreeUpdate {
                nodes,
                tree: prev.is_none().then(|| {
                    let mut t = k::Tree::new(k::NodeId(self.root));
                    t.toolkit_name = Some("lumen".into());
                    t.toolkit_version = Some(env!("CARGO_PKG_VERSION").into());
                    t
                }),
                tree_id: k::TreeId::ROOT,
                focus: k::NodeId(self.focus),
            }
        }
    }

    impl A11yAction {
        /// An AccessKit request as a lumen action. `None` for actions
        /// lumen doesn't handle.
        pub fn from_accesskit(req: &k::ActionRequest) -> Option<(crate::Id, A11yAction)> {
            let a = match req.action {
                k::Action::Click => A11yAction::Click,
                k::Action::Focus => A11yAction::Focus,
                k::Action::Blur => A11yAction::Blur,
                k::Action::Increment => A11yAction::Increment,
                k::Action::Decrement => A11yAction::Decrement,
                k::Action::Expand => A11yAction::Expand,
                k::Action::Collapse => A11yAction::Collapse,
                k::Action::ScrollIntoView => A11yAction::ScrollIntoView,
                k::Action::SetValue => match &req.data {
                    Some(k::ActionData::Value(v)) => A11yAction::SetValue(v.to_string()),
                    Some(k::ActionData::NumericValue(v)) => A11yAction::SetNumber(*v),
                    _ => return None,
                },
                _ => return None,
            };
            Some((crate::Id(req.target_node.0), a))
        }
    }
}
