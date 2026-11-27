//! Accessibility tree: roles, names, state, and actions from a screen reader.

use lumen_core::*;

#[derive(Default)]
struct S {
    on: bool,
    agree: bool,
    vol: f32,
    name: String,
    clicks: u32,
}

fn view(ui: &mut Ui, s: &mut S) {
    ui.column().pad(16.0).gap(8.0).show(|ui| {
        ui.heading("Settings").level(2).show();
        ui.label("Change how the app behaves.").show();
        if ui.button("Save").show().clicked() {
            s.clicks += 1;
        }
        ui.icon_button("trash").tooltip("Delete").show();
        ui.switch("Wi-Fi", &mut s.on).show();
        ui.checkbox("I agree", &mut s.agree).show();
        ui.slider("Volume", &mut s.vol, 0.0..=10.0).step(1.0).show();
        ui.text_input(&mut s.name).label("Name").placeholder("Your name").show();
    });
}

struct H {
    ui: Ui,
    t: f64,
    s: S,
    last: Option<A11yTree>,
}

impl H {
    fn new() -> Self {
        let mut ui = Ui::new();
        ui.set_accessibility(true);
        let mut h = H { ui, t: 0.0, s: S::default(), last: None };
        h.frame(vec![]);
        h.frame(vec![]);
        h
    }
    fn frame(&mut self, events: Vec<Event>) -> &A11yTree {
        self.t += 1.0 / 60.0;
        let input = RawInput { screen_size: vec2(600.0, 600.0), time: self.t, events, ..Default::default() };
        let s = &mut self.s;
        let out = self.ui.run(input, |ui| view(ui, s));
        self.last = out.a11y;
        self.last.as_ref().unwrap()
    }
    fn tree(&self) -> &A11yTree {
        self.last.as_ref().unwrap()
    }
    fn act(&mut self, role: Role, label: &str, action: A11yAction) {
        let id = self.tree().find(role, label).unwrap_or_else(|| panic!("no {role:?} {label:?}\n{}", self.tree().dump())).id;
        self.frame(vec![Event::A11y { target: Id(id), action }]);
        self.frame(vec![]);
    }
}

#[test]
fn off_by_default() {
    let mut ui = Ui::new();
    let mut s = S::default();
    let out = ui.run(RawInput::default(), |ui| view(ui, &mut s));
    assert!(out.a11y.is_none());
}

#[test]
fn roles_names_and_state() {
    let h = H::new();
    let t = h.tree();
    println!("{}", t.dump());
    let heading = t.find(Role::Heading, "Settings").expect("heading");
    assert_eq!(heading.info.level, Some(2));
    assert!(t.find(Role::Label, "Change how the app behaves.").is_some());
    let save = t.find(Role::Button, "Save").expect("button named by its text");
    assert!(save.actions.click && save.actions.focus);
    assert!(t.find(Role::Button, "Delete").is_some(), "icon button named by its tooltip");
    assert_eq!(t.find(Role::Switch, "Wi-Fi").unwrap().info.toggled, Some(Toggled::False));
    assert_eq!(t.find(Role::CheckBox, "I agree").unwrap().info.toggled, Some(Toggled::False));
    let vol = t.find(Role::Slider, "Volume").unwrap();
    assert_eq!(vol.info.numeric.map(|n| (n.min, n.max)), Some((0.0, 10.0)));
    let name = t.find(Role::TextInput, "Name").unwrap();
    assert_eq!(name.info.placeholder.as_deref(), Some("Your name"));
    // No text is read twice: the button's caption is its name, not a label.
    assert!(t.find(Role::Label, "Save").is_none());
    assert!(t.find(Role::Label, "Wi-Fi").is_none());
    // Every child id resolves, and every node but the root has one parent.
    let mut parents = std::collections::HashMap::new();
    for n in &t.nodes {
        for c in &n.children {
            assert!(t.get(*c).is_some());
            assert!(parents.insert(*c, n.id).is_none(), "two parents");
        }
    }
    assert_eq!(parents.len(), t.nodes.len() - 1);
}

#[test]
fn actions_from_a_screen_reader() {
    let mut h = H::new();
    h.act(Role::Button, "Save", A11yAction::Click);
    assert_eq!(h.s.clicks, 1);
    let save = h.tree().find(Role::Button, "Save").unwrap().id;
    assert_eq!(h.tree().focus, save, "pressing focuses");

    h.act(Role::Switch, "Wi-Fi", A11yAction::Click);
    assert!(h.s.on);
    assert_eq!(h.tree().find(Role::Switch, "Wi-Fi").unwrap().info.toggled, Some(Toggled::True));

    h.act(Role::CheckBox, "I agree", A11yAction::Click);
    assert!(h.s.agree);

    h.act(Role::Slider, "Volume", A11yAction::Increment);
    h.act(Role::Slider, "Volume", A11yAction::Increment);
    assert_eq!(h.s.vol, 2.0);
    h.act(Role::Slider, "Volume", A11yAction::Decrement);
    assert_eq!(h.s.vol, 1.0);
    h.act(Role::Slider, "Volume", A11yAction::SetNumber(7.0));
    assert_eq!(h.s.vol, 7.0);

    h.act(Role::TextInput, "Name", A11yAction::SetValue("Ada".into()));
    assert_eq!(h.s.name, "Ada");
    assert_eq!(h.tree().find(Role::TextInput, "Name").unwrap().info.value.as_deref(), Some("Ada"));

    h.act(Role::TextInput, "Name", A11yAction::Focus);
    let name = h.tree().find(Role::TextInput, "Name").unwrap().id;
    assert_eq!(h.tree().focus, name);
}

#[derive(Default)]
struct S2 {
    fruit: u8,
    tab: u8,
    view: u8,
    dialog: bool,
    menu_picks: u32,
    deleted: bool,
    sel: Option<usize>,
}

fn view2(ui: &mut Ui, s: &mut S2) {
    ui.column().pad(16.0).gap(8.0).show(|ui| {
        ui.select(&mut s.fruit, &[(0, "Apple"), (1, "Pear"), (2, "Plum")]).label("Fruit").show();
        ui.tabs(&mut s.tab, &[(0, "General"), (1, "Advanced")]);
        ui.segmented(&mut s.view, &[(0, "List"), (1, "Grid")]);
        let b = ui.button("Actions").show();
        ui.menu(&b, |ui| {
            if ui.menu_item("copy", "Copy").shortcut("⌘C").show().clicked() {
                s.menu_picks += 1;
            }
            ui.menu_item("", "Paste").show();
        });
        if ui.button("Delete…").show().clicked() {
            s.dialog = true;
        }
        let rows: Vec<ListRow> = (0..1000).map(|i| ListRow::new(format!("Message {i}"))).collect();
        ui.scroll("inbox").height(200.0).show(|ui| {
            ui.list_view("inbox", &rows, &mut s.sel);
        });
        let mut del = false;
        ui.dialog("confirm", &mut s.dialog).title("Delete file?").description("This can't be undone.").show(|ui| {
            if ui.button("Delete").variant(Variant::Danger).show().clicked() {
                del = true;
                ui.close_layer();
            }
        });
        s.deleted |= del;
    });
}

struct H2 {
    ui: Ui,
    t: f64,
    s: S2,
    last: Option<A11yTree>,
}

impl H2 {
    fn new() -> Self {
        let mut ui = Ui::new();
        ui.set_accessibility(true);
        let mut h = H2 { ui, t: 0.0, s: S2::default(), last: None };
        h.frame(vec![]);
        h.frame(vec![]);
        h
    }
    fn frame(&mut self, events: Vec<Event>) {
        self.t += 1.0 / 60.0;
        let input = RawInput { screen_size: vec2(700.0, 800.0), time: self.t, events, ..Default::default() };
        let s = &mut self.s;
        self.last = self.ui.run(input, |ui| view2(ui, s)).a11y;
    }
    fn settle(&mut self) {
        for _ in 0..30 {
            self.frame(vec![]);
        }
    }
    fn tree(&self) -> &A11yTree {
        self.last.as_ref().unwrap()
    }
    fn id(&self, role: Role, label: &str) -> u64 {
        self.tree().find(role, label).unwrap_or_else(|| panic!("no {role:?} {label:?}\n{}", self.tree().dump())).id
    }
    fn act(&mut self, role: Role, label: &str, action: A11yAction) {
        let id = self.id(role, label);
        self.frame(vec![Event::A11y { target: Id(id), action }]);
        self.settle();
    }
}

#[test]
fn select_tabs_menu_dialog_list() {
    let mut h = H2::new();
    let t = h.tree();
    let combo = t.find(Role::ComboBox, "Fruit").unwrap();
    assert_eq!(combo.info.value.as_deref(), Some("Apple"));
    assert_eq!(combo.info.expanded, Some(false));
    assert!(t.find(Role::Label, "Fruit").is_none(), "field label not read twice");
    assert_eq!(t.find(Role::Tab, "General").unwrap().info.selected, Some(true));
    assert!(t.nodes.iter().any(|n| n.info.role == Role::TabList));
    assert_eq!(t.find(Role::RadioButton, "Grid").unwrap().info.toggled, Some(Toggled::False));
    let lb = t.nodes.iter().find(|n| n.info.role == Role::ListBox).unwrap();
    let first = t.find(Role::ListBoxOption, "Message 0").unwrap();
    assert_eq!(first.info.position, Some((1, 1000)));
    assert!(lb.children.len() < 100, "virtualized: only built rows, {}", lb.children.len());

    // Select opens on Expand, then an option click picks it.
    h.act(Role::ComboBox, "Fruit", A11yAction::Expand);
    assert_eq!(h.tree().find(Role::ComboBox, "Fruit").unwrap().info.expanded, Some(true));
    assert!(h.tree().nodes.iter().any(|n| n.info.role == Role::ListBox && n.children.len() == 3), "{}", h.tree().dump());
    h.act(Role::ListBoxOption, "Pear", A11yAction::Click);
    assert_eq!(h.s.fruit, 1);
    assert!(h.tree().find(Role::ListBoxOption, "Pear").is_none(), "closed");

    h.act(Role::Tab, "Advanced", A11yAction::Click);
    assert_eq!(h.s.tab, 1);
    h.act(Role::RadioButton, "Grid", A11yAction::Click);
    assert_eq!(h.s.view, 1);

    // Menu: the trigger reports expanded; items are menu items.
    h.act(Role::Button, "Actions", A11yAction::Click);
    assert_eq!(h.tree().find(Role::Button, "Actions").unwrap().info.expanded, Some(true));
    let copy = h.tree().find(Role::MenuItem, "Copy").unwrap();
    assert_eq!(copy.info.description.as_deref(), Some("⌘C"));
    assert!(h.tree().nodes.iter().any(|n| n.info.role == Role::Menu && n.children.contains(&copy.id)));
    h.act(Role::MenuItem, "Copy", A11yAction::Click);
    assert_eq!(h.s.menu_picks, 1);
    let actions = h.id(Role::Button, "Actions");
    assert_eq!(h.tree().focus, actions, "focus returns to the menu's trigger\n{}", h.tree().dump());
    assert!(h.tree().find(Role::MenuItem, "Copy").is_none());

    // Dialog: modal, named by its title, focus moves inside.
    h.act(Role::Button, "Delete…", A11yAction::Click);
    let d = h.tree().find(Role::Dialog, "Delete file?").unwrap_or_else(|| panic!("{}", h.tree().dump()));
    assert!(d.info.modal);
    assert_eq!(d.info.description.as_deref(), Some("This can't be undone."));
    let del = h.id(Role::Button, "Delete");
    assert_eq!(h.tree().focus, del, "dialog focuses its action");
    h.act(Role::Button, "Delete", A11yAction::Click);
    assert!(h.s.deleted && !h.s.dialog);

    // List: a click selects.
    h.act(Role::ListBoxOption, "Message 3", A11yAction::Click);
    assert_eq!(h.s.sel, Some(3));
    assert_eq!(h.tree().find(Role::ListBoxOption, "Message 3").unwrap().info.selected, Some(true));
}
