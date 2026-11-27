//! Veil, onboarding checklist, swipe actions and docking panels, driven
//! through a headless `Ui` and found through the accessibility tree.

use lumen_core::*;

struct H<S> {
    ui: Ui,
    t: f64,
    s: S,
    view: fn(&mut Ui, &mut S),
    tree: Option<A11yTree>,
    m: Modifiers,
}

impl<S> H<S> {
    fn new(s: S, view: fn(&mut Ui, &mut S)) -> Self {
        let mut ui = Ui::new();
        ui.set_accessibility(true);
        let mut h = H { ui, t: 1.0, s, view, tree: None, m: Modifiers::default() };
        h.idle(3);
        h
    }
    fn frame(&mut self, events: Vec<Event>) {
        self.t += 1.0 / 60.0;
        let input = RawInput { screen_size: vec2(900.0, 900.0), scale: 1.0, time: self.t, events, modifiers: self.m };
        let (s, view) = (&mut self.s, self.view);
        let out = self.ui.run(input, |ui| view(ui, s));
        self.tree = out.a11y;
    }
    fn idle(&mut self, n: usize) {
        for _ in 0..n {
            self.frame(vec![]);
        }
    }
    fn find(&self, role: Role, label: &str) -> Option<A11yNode> {
        self.tree.as_ref().unwrap().find(role, label).cloned()
    }
    fn node(&self, role: Role, label: &str) -> A11yNode {
        self.find(role, label).unwrap_or_else(|| panic!("no {role:?} {label:?}\n{}", self.tree.as_ref().unwrap().dump()))
    }
    fn click(&mut self, p: Vec2) {
        self.frame(vec![Event::PointerMoved(p), press(p)]);
        self.frame(vec![release(p)]);
        self.idle(1);
    }
    fn drag(&mut self, from: Vec2, to: Vec2, steps: usize) {
        self.frame(vec![Event::PointerMoved(from), press(from)]);
        for k in 1..=steps {
            let p = from + (to - from) * (k as f32 / steps as f32);
            self.frame(vec![Event::PointerMoved(p)]);
        }
        self.idle(2);
        self.frame(vec![release(to)]);
        self.idle(2);
    }
    fn key(&mut self, k: Key, m: Modifiers) {
        self.m = m;
        self.frame(vec![Event::Key { key: k, pressed: true, repeat: false, modifiers: m }, Event::Key { key: k, pressed: false, repeat: false, modifiers: m }]);
        self.m = Modifiers::default();
        self.idle(1);
    }
}

fn press(p: Vec2) -> Event {
    Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true }
}
fn release(p: Vec2) -> Event {
    Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false }
}
const NONE: Modifiers = Modifiers { shift: false, ctrl: false, alt: false, logo: false, mac: true };
const CMD_SHIFT: Modifiers = Modifiers { shift: true, ctrl: false, alt: false, logo: true, mac: true };

// ------------------------------------------------------------ veil

#[derive(Default)]
struct V {
    revealed: bool,
    clicks: u32,
}
fn v_view(ui: &mut Ui, s: &mut V) {
    ui.column().pad(24).width(500.0).show(|ui| {
        ui.veil("k", &mut s.revealed, "Secret", |ui| {
            ui.column().pad(16).show(|ui| {
                if ui.button("Use key").show().clicked {
                    s.clicks += 1;
                }
            });
        });
    });
}

#[test]
fn veil_blocks_until_revealed() {
    let mut h = H::new(V::default(), v_view);
    assert!(h.find(Role::Button, "Use key").is_none(), "hidden from screen readers while veiled");
    let cover = h.node(Role::Button, "Reveal Secret");
    // A click on the veil reveals; it doesn't reach the button under it.
    h.click(cover.rect.center());
    assert!(h.s.revealed);
    assert_eq!(h.s.clicks, 0);
    h.idle(60);
    assert!(h.find(Role::Button, "Reveal Secret").is_none());
    let b = h.node(Role::Button, "Use key");
    h.click(b.rect.center());
    assert_eq!(h.s.clicks, 1, "revealed content works");
    h.s.revealed = false;
    h.idle(60);
    assert!(h.find(Role::Button, "Use key").is_none(), "veiled again");
}

// ------------------------------------------------------------ onboarding

struct O {
    steps: Vec<OnboardingStep>,
    open: bool,
    dismissed: bool,
}
fn o_view(ui: &mut Ui, s: &mut O) {
    ui.column().pad(24).show(|ui| {
        let out = ui.onboarding_checklist("setup", "Get started", &s.steps, &mut s.open);
        if let Some(i) = out.action {
            s.steps[i].done = true;
        }
        s.dismissed |= out.dismissed;
    });
}

#[test]
fn onboarding_opens_advances_and_finishes() {
    let steps = vec![OnboardingStep::new("One", "first", "Do one", true), OnboardingStep::new("Two", "second", "Do two", false)];
    let mut h = H::new(O { steps, open: false, dismissed: false }, o_view);
    let l = h.node(Role::Button, "Get started");
    assert_eq!(l.info.value.as_deref(), Some("1 of 2 done"));
    assert_eq!(l.info.expanded, Some(false));
    h.click(l.rect.center());
    assert!(h.s.open);
    h.idle(30);
    assert_eq!(h.node(Role::ListItem, "Two").info.value.as_deref(), Some("Next"));
    h.click(h.node(Role::Button, "Do two").rect.center());
    assert!(h.s.steps[1].done);
    h.idle(60);
    h.click(h.node(Role::Button, "Dismiss").rect.center());
    assert!(h.s.dismissed);
}

// ------------------------------------------------------------ swipe

#[derive(Default)]
struct S {
    hits: Vec<SwipeHit>,
}
fn s_view(ui: &mut Ui, s: &mut S) {
    let lead = [SwipeAction::new("Unread", "mail", Tone::Accent)];
    let trail = [SwipeAction::new("Archive", "archive", Tone::Success), SwipeAction::new("Delete", "trash-2", Tone::Danger)];
    ui.column().pad(24).width(400.0).show(|ui| {
        if let Some(hit) = ui.swipe_row("m", "Message from Ada", &lead, &trail, |ui| {
            ui.column().pad(16).show(|ui| {
                ui.label("Ada Lovelace");
                ui.label("Notes on the engine");
            });
        }) {
            s.hits.push(hit);
        }
    });
}

#[test]
fn swipe_opens_fires_and_works_from_keys() {
    let mut h = H::new(S::default(), s_view);
    let row = h.node(Role::Group, "Message from Ada").rect;
    // Drag left past half the actions: it snaps open on the trailing side.
    h.drag(row.center(), row.center() - vec2(90.0, 0.0), 6);
    assert!(h.s.hits.is_empty());
    h.idle(40);
    // Tap the last revealed action (Delete, at the right edge).
    h.click(vec2(row.max.x - 10.0, row.center().y));
    assert_eq!(h.s.hits, vec![SwipeHit { trailing: true, index: 1 }]);
    h.idle(40);
    // A long swipe right fires the leading action.
    h.drag(row.min + vec2(20.0, row.height() * 0.5), row.min + vec2(row.width() * 0.9, row.height() * 0.5), 10);
    assert_eq!(h.s.hits.last(), Some(&SwipeHit { trailing: false, index: 0 }));
    h.idle(60);
    // Keyboard: focus, ← opens the trailing side, Enter fires its outermost.
    h.click(row.center());
    h.key(Key::ArrowLeft, NONE);
    h.key(Key::Enter, NONE);
    assert_eq!(h.s.hits.last(), Some(&SwipeHit { trailing: true, index: 1 }));
    // Screen readers press the actions directly.
    let a = h.node(Role::Button, "Archive").id;
    h.frame(vec![Event::A11y { target: Id(a), action: A11yAction::Click }]);
    assert_eq!(h.s.hits.last(), Some(&SwipeHit { trailing: true, index: 0 }));
}

// ------------------------------------------------------------ docking

struct D {
    layout: DockLayout,
}
fn d_view(ui: &mut Ui, s: &mut D) {
    let panels = [DockPanel::new("files", "Files", "folder"), DockPanel::new("main", "main.rs", "file"), DockPanel::new("term", "Terminal", "terminal")];
    ui.column().pad(24).width(800.0).show(|ui| {
        ui.dock_area("ide", &panels, &mut s.layout, 400.0, |ui, pid| {
            ui.label(format!("{pid} body"));
        });
    });
}

#[test]
fn dock_moves_panels_by_drag_and_keys() {
    let v = |xs: &[&str]| xs.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let layout = DockLayout { zones: [v(&["files"]), v(&["main"]), vec![], v(&["term"])], active: [0; 4], left: 180.0, right: 180.0, bottom: 120.0 };
    let mut h = H::new(D { layout }, d_view);
    h.idle(30);
    assert!(h.find(Role::TabList, "Right dock").is_none(), "empty zones collapse");
    // Drag the Files tab to the right edge.
    let tab = h.node(Role::Tab, "Files").rect.center();
    let area = h.node(Role::TabList, "Editor area").rect;
    h.drag(tab, vec2(area.max.x - 20.0, area.min.y + 100.0), 12);
    assert_eq!(h.s.layout.zone_of("files"), Some(DockZone::Right));
    h.idle(30);
    assert!(h.find(Role::TabList, "Right dock").is_some());
    // Cmd+Shift+↓ moves the focused tab to the bottom dock.
    let tab = h.node(Role::Tab, "Files").rect.center();
    h.click(tab);
    h.key(Key::ArrowDown, CMD_SHIFT);
    assert_eq!(h.s.layout.zone_of("files"), Some(DockZone::Bottom));
    assert_eq!(h.s.layout.zones[DockZone::Bottom as usize], vec!["term".to_string(), "files".to_string()]);
    assert_eq!(h.s.layout.active[DockZone::Bottom as usize], 1, "the moved panel shows");
    // The bottom splitter resizes.
    h.idle(30);
    let before = h.s.layout.bottom;
    let sp = h.tree.as_ref().unwrap().nodes.iter().find(|n| n.info.label.as_deref() == Some("Resize bottom dock")).unwrap().rect.center();
    h.drag(sp, sp - vec2(0.0, 40.0), 5);
    assert!((h.s.layout.bottom - (before + 40.0)).abs() < 2.0, "{} vs {}", h.s.layout.bottom, before);
}
