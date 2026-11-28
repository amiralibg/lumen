//! Chart annotations, bubble chart and dot-matrix display.

use lumen_core::*;

struct H<S> {
    ui: Ui,
    t: f64,
    s: S,
    view: fn(&mut Ui, &mut S),
    out: Option<FrameOutput>,
}

impl<S> H<S> {
    fn new(s: S, view: fn(&mut Ui, &mut S)) -> Self {
        let mut ui = Ui::new();
        ui.set_accessibility(true);
        let mut h = H { ui, t: 1.0, s, view, out: None };
        h.idle(3);
        h
    }
    fn frame(&mut self, events: Vec<Event>) {
        self.t += 1.0 / 60.0;
        let input = RawInput { screen_size: vec2(900.0, 900.0), scale: 1.0, time: self.t, events, modifiers: Modifiers::default() };
        let (s, view) = (&mut self.s, self.view);
        self.out = Some(self.ui.run(input, |ui| view(ui, s)));
    }
    fn idle(&mut self, n: usize) {
        for _ in 0..n {
            self.frame(vec![]);
        }
    }
    fn tree(&self) -> &A11yTree {
        self.out.as_ref().unwrap().a11y.as_ref().unwrap()
    }
    fn role(&self, role: Role) -> Vec<A11yNode> {
        self.tree().nodes.iter().filter(|n| n.info.role == role).cloned().collect()
    }
    fn click(&mut self, p: Vec2) {
        self.frame(vec![Event::PointerMoved(p), Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true }]);
        self.frame(vec![Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false }]);
    }
    fn key(&mut self, k: Key) {
        self.frame(vec![Event::Key { key: k, pressed: true, repeat: false, modifiers: Modifiers::default() }, Event::Key { key: k, pressed: false, repeat: false, modifiers: Modifiers::default() }]);
    }
    fn settled(&mut self, max: usize) -> bool {
        for _ in 0..max {
            self.idle(1);
            if !self.out.as_ref().unwrap().animating {
                return true;
            }
        }
        false
    }
}

// ------------------------------------------------------------ annotations

struct A;
fn a_view(ui: &mut Ui, _: &mut A) {
    ui.column().pad(24).width(700.0).show(|ui| {
        ui.line_chart("s", &["W1", "W2", "W3", "W4"], vec![Series::new("Signups", vec![10.0, 20.0, 15.0, 30.0])])
            .annotate(Annotation::target(50.0, "Goal"))
            .annotate(Annotation::band(1, 2, "Outage", Tone::Danger))
            .annotate(Annotation::point(3, 0, "Launch"))
            .show();
    });
}

#[test]
fn annotations_are_described_and_settle() {
    let mut h = H::new(A, a_view);
    let img = &h.role(Role::Image)[0];
    assert_eq!(img.info.label.as_deref(), Some("Line chart: Signups"));
    assert_eq!(img.info.description.as_deref(), Some("Goal at 50; Outage from W2 to W3; Launch: Signups at W4"));
    assert!(h.settled(240), "annotations finish drawing in");
}

// ------------------------------------------------------------ bubbles

struct B {
    year: usize,
}
fn b_view(ui: &mut Ui, s: &mut B) {
    let k = s.year as f32;
    let data = vec![Bubble::new("Small", 10.0 + k, 50.0, 100.0, 0), Bubble::new("Big", 80.0, 80.0 - k * 10.0, 900.0, 1)];
    ui.column().pad(24).width(700.0).show(|ui| {
        ui.bubble_chart("b", &data).axes("Revenue", "Score").size_label("Users").x_range(0.0, 100.0).y_range(0.0, 100.0).max_size(1000.0).frame(&format!("{}", 2020 + s.year)).height(300.0).show();
    });
}

#[test]
fn bubbles_select_by_keys_and_glide_between_frames() {
    let mut h = H::new(B { year: 0 }, b_view);
    assert!(h.settled(240));
    let img = h.role(Role::Image)[0].clone();
    assert_eq!(img.info.label.as_deref(), Some("Bubble chart: Score by Revenue"));
    assert_eq!(img.info.value.as_deref(), Some("2 bubbles"));
    // Focus it with a click on empty space, then walk with the arrows.
    h.click(img.rect.min + vec2(80.0, 20.0));
    h.key(Key::ArrowRight);
    h.idle(1);
    assert_eq!(h.role(Role::Image)[0].info.value.as_deref(), Some("Small: Revenue 10, Score 50, Users 100"));
    h.key(Key::ArrowRight);
    h.idle(1);
    assert!(h.role(Role::Image)[0].info.value.as_deref().unwrap().starts_with("Big:"));
    // A new frame animates, then rests.
    h.s.year = 3;
    h.idle(1);
    assert!(h.out.as_ref().unwrap().animating, "bubbles glide to the new year");
    assert!(h.settled(240));
}

// ------------------------------------------------------------ dot matrix

struct D {
    text: String,
    scroll: bool,
}
fn d_view(ui: &mut Ui, s: &mut D) {
    ui.column().pad(24).show(|ui| {
        ui.dot_matrix("m", &s.text).cols(40).scroll(s.scroll).show();
    });
}

#[test]
fn dot_matrix_reads_flips_and_scrolls() {
    let mut h = H::new(D { text: "ON TIME".into(), scroll: false }, d_view);
    assert!(h.settled(120));
    assert_eq!(h.role(Role::Label).iter().filter(|n| n.info.label.as_deref() == Some("ON TIME")).count(), 1);
    h.s.text = "DELAYED".into();
    h.idle(1);
    assert!(h.out.as_ref().unwrap().animating, "dots flip column by column");
    assert!(h.settled(120), "and stop");
    // Text longer than the panel scrolls for as long as it's shown.
    h.s.text = "PLATFORM 4 NOW PLATFORM 6".into();
    h.s.scroll = true;
    h.idle(200);
    assert!(h.out.as_ref().unwrap().animating);
    h.ui.set_reduced_motion(true);
    assert!(h.settled(10), "reduced motion: no marquee");
}
