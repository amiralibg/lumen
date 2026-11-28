//! Query builder, regex tester, feature flags and dependency graph,
//! driven through a headless `Ui` and found through the accessibility tree.

use std::cell::Cell;

use lumen_core::*;

struct H<S> {
    ui: Ui,
    t: f64,
    s: S,
    view: fn(&mut Ui, &mut S),
    tree: Option<A11yTree>,
}

impl<S> H<S> {
    fn new(s: S, view: fn(&mut Ui, &mut S)) -> Self {
        let mut ui = Ui::new();
        ui.set_accessibility(true);
        let mut h = H { ui, t: 1.0, s, view, tree: None };
        h.idle(3);
        h
    }
    fn frame(&mut self, events: Vec<Event>) {
        self.t += 1.0 / 60.0;
        let input = RawInput { screen_size: vec2(900.0, 1200.0), scale: 1.0, time: self.t, events, modifiers: Modifiers::default() };
        let (s, view) = (&mut self.s, self.view);
        let out = self.ui.run(input, |ui| view(ui, s));
        self.tree = out.a11y;
    }
    fn idle(&mut self, n: usize) {
        for _ in 0..n {
            self.frame(vec![]);
        }
    }
    fn node(&self, role: Role, label: &str) -> A11yNode {
        let t = self.tree.as_ref().unwrap();
        t.find(role, label).cloned().unwrap_or_else(|| panic!("no {role:?} {label:?}\n{}", t.dump()))
    }
    fn all(&self, role: Role, label: &str) -> Vec<A11yNode> {
        self.tree.as_ref().unwrap().nodes.iter().filter(|n| n.info.role == role && n.info.label.as_deref() == Some(label)).cloned().collect()
    }
    fn click(&mut self, p: Vec2) {
        self.frame(vec![Event::PointerMoved(p), press(p)]);
        self.frame(vec![release(p)]);
        self.idle(1);
    }
    fn key(&mut self, k: Key) {
        self.frame(vec![Event::Key { key: k, pressed: true, repeat: false, modifiers: Modifiers::default() }, Event::Key { key: k, pressed: false, repeat: false, modifiers: Modifiers::default() }]);
    }
}

fn press(p: Vec2) -> Event {
    Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true }
}
fn release(p: Vec2) -> Event {
    Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false }
}

// ------------------------------------------------------------ query builder

fn fields() -> Vec<QueryField> {
    vec![QueryField::choice("Status", &["open", "closed"]), QueryField::number("Priority"), QueryField::text("Assignee")]
}
struct Q {
    q: QueryGroup,
}
fn q_view(ui: &mut Ui, s: &mut Q) {
    let f = fields();
    ui.column().pad(24).width(700.0).show(|ui| {
        ui.query_builder("issues", &f, &mut s.q);
    });
}

#[test]
fn query_builds_the_string_and_edits_rules() {
    let q = QueryGroup {
        any: false,
        rules: vec![QueryRule { field: 0, op: 0, value: "open".into() }, QueryRule { field: 1, op: 2, value: "2".into() }],
        groups: vec![QueryGroup { any: true, rules: vec![QueryRule { field: 2, op: 0, value: "ada".into() }, QueryRule { field: 2, op: 2, value: "bo".into() }], groups: vec![] }],
    };
    assert_eq!(q.to_query(&fields()), r#"status = "open" AND priority > 2 AND (assignee = "ada" OR assignee CONTAINS "bo")"#);
    let mut h = H::new(Q { q }, q_view);
    assert_eq!(h.node(Role::Status, "Query").info.value.as_deref(), Some(r#"status = "open" AND priority > 2 AND (assignee = "ada" OR assignee CONTAINS "bo")"#));
    // Two "Rule" buttons (root and nested); the first adds to the root.
    let add = h.all(Role::Button, "Rule");
    assert_eq!(add.len(), 2);
    h.click(add[0].rect.center());
    assert_eq!(h.s.q.rules.len(), 3);
    // Remove the nested group.
    h.click(h.node(Role::Button, "Remove group").rect.center());
    assert!(h.s.q.groups.is_empty());
    // Any joins the root with OR.
    h.click(h.all(Role::RadioButton, "Any")[0].rect.center());
    assert!(h.s.q.any);
    assert!(h.s.q.to_query(&fields()).contains(" OR "));
    // Removing a rule.
    let rm = h.all(Role::Button, "Remove rule");
    h.click(rm[0].rect.center());
    assert_eq!(h.s.q.rules.len(), 2);
    assert_eq!(h.s.q.rules[0].field, 1);
}

// ------------------------------------------------------------ regex tester

struct R {
    pattern: String,
    flags: RegexFlags,
    text: String,
    runs: Cell<u32>,
}
fn r_view(ui: &mut Ui, s: &mut R) {
    ui.column().pad(24).width(640.0).show(|ui| {
        let runs = &s.runs;
        // A tiny literal "engine": every occurrence of the pattern; "(" is
        // an error, and the ignore-case flag lowercases both sides.
        ui.regex_tester("t", &mut s.pattern, &mut s.flags, &mut s.text, |p, f, t| {
            runs.set(runs.get() + 1);
            if p.contains('(') {
                return Err("unclosed group".into());
            }
            let (p, t) = if f.ignore_case { (p.to_lowercase(), t.to_lowercase()) } else { (p.to_owned(), t.to_owned()) };
            Ok(t.match_indices(&p).map(|(i, m)| RegexMatch { range: i..i + m.len(), groups: vec![] }).collect())
        });
    });
}

#[test]
fn regex_runs_only_on_change_and_reports() {
    let mut h = H::new(R { pattern: "ab".into(), flags: RegexFlags::default(), text: "ab AB ab".into(), runs: Cell::new(0) }, r_view);
    assert_eq!(h.s.runs.get(), 1, "once, not every frame");
    assert!(h.tree.as_ref().unwrap().find(Role::Status, "2 matches").is_some());
    assert_eq!(h.all(Role::ListItem, "Match 1: ab").len(), 1);
    h.click(h.node(Role::Button, "Ignore case").rect.center());
    assert!(h.s.flags.ignore_case);
    assert_eq!(h.s.runs.get(), 2);
    assert!(h.tree.as_ref().unwrap().find(Role::Status, "3 matches").is_some());
    h.s.pattern = "(ab".into();
    h.idle(2);
    assert!(h.tree.as_ref().unwrap().find(Role::Status, "unclosed group").is_some());
}

// ------------------------------------------------------------ feature flags

struct F {
    flags: Vec<FeatureFlag>,
    changed: Vec<usize>,
}
fn f_view(ui: &mut Ui, s: &mut F) {
    ui.column().pad(24).width(800.0).show(|ui| {
        if let Some(i) = ui.feature_flags("f", &["Dev", "Prod"], &mut s.flags) {
            s.changed.push(i);
        }
    });
}

#[test]
fn flags_toggle_per_env_and_filter() {
    let flag = |key: &str| FeatureFlag { key: key.into(), description: "d".into(), on: vec![true, false], rollout: 0.0, stale: false };
    let mut h = H::new(F { flags: vec![flag("new-checkout"), flag("dark-mode")], changed: vec![] }, f_view);
    let sw = h.node(Role::Switch, "dark-mode in Prod");
    assert_eq!(sw.info.toggled, Some(Toggled::False));
    h.click(sw.rect.center());
    assert!(h.s.flags[1].on[1]);
    assert_eq!(h.s.changed, vec![1]);
    // The rollout slider works once the flag is live in Prod.
    h.idle(2);
    let roll = h.node(Role::Slider, "dark-mode rollout");
    assert!(!roll.info.disabled);
    h.frame(vec![Event::A11y { target: Id(roll.id), action: A11yAction::Increment }]);
    h.idle(1);
    assert_eq!(h.s.flags[1].rollout, 5.0);
    // Filtering.
    let filter = h.node(Role::TextInput, "Filter flags");
    h.click(filter.rect.center());
    h.frame(vec![Event::Text("check".into())]);
    h.idle(2);
    assert!(h.tree.as_ref().unwrap().find(Role::Switch, "dark-mode in Prod").is_none());
    assert!(h.tree.as_ref().unwrap().find(Role::Switch, "new-checkout in Prod").is_some());
}

// ------------------------------------------------------------ dependency graph

struct G {
    sel: Option<usize>,
}
fn g_view(ui: &mut Ui, s: &mut G) {
    let n = |name: &str| DepNode { name: name.into(), version: "1.0".into() };
    let nodes = vec![n("app"), n("web"), n("db"), n("io")];
    let edges = [(0, 1), (0, 2), (1, 3), (2, 3)];
    ui.column().pad(24).width(800.0).show(|ui| {
        ui.dependency_graph("g", &nodes, &edges, &mut s.sel);
    });
}

#[test]
fn graph_selects_and_traces() {
    let mut h = H::new(G { sel: None }, g_view);
    h.idle(60);
    let g = h.node(Role::Canvas, "Dependency graph");
    assert_eq!(g.info.value.as_deref(), Some("4 packages · 4 edges · select one to trace it"));
    // Keyboard: the first press picks the root, then walk along the layers.
    h.click(vec2(g.rect.max.x - 2.0, g.rect.max.y - 2.0));
    assert_eq!(h.s.sel, None, "a click on empty space selects nothing");
    h.key(Key::ArrowRight);
    assert_eq!(h.s.sel, Some(0));
    h.key(Key::ArrowRight);
    h.idle(1);
    let s1 = h.s.sel.unwrap();
    assert!(s1 == 1 || s1 == 2, "next layer: {s1}");
    h.key(Key::ArrowRight);
    assert_eq!(h.s.sel, Some(3));
    h.idle(1);
    let v = h.node(Role::Canvas, "Dependency graph").info.value.unwrap();
    assert_eq!(v, "io 1.0 · 0 direct dependencies · used by 2 · 0 in total below it");
    h.key(Key::Escape);
    assert_eq!(h.s.sel, None);
}
