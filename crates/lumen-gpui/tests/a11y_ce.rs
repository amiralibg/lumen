//! gpui-ce: lumen's accessibility tree shows up in the window's AccessKit
//! tree, and actions on its nodes reach lumen. Headless (gpui's test
//! platform plus an offscreen wgpu device):
//!
//!     cargo +1.95 test --no-default-features --features ce-test --test a11y_ce

use gpui::accesskit::{Action, ActionData, NodeId, Role, Toggled, TreeUpdate};
use gpui::{AppContext as _, TestAppContext};
use lumen_core::Ui;
use lumen_gpui::LumenView;

#[derive(Default)]
struct S {
    saves: u32,
    wifi: bool,
    name: String,
}

fn view(ui: &mut Ui, s: &mut S) {
    ui.column().pad(16.0).gap(8.0).show(|ui| {
        ui.heading("Profile").level(2).show();
        if ui.button("Save").show().clicked() {
            s.saves += 1;
        }
        ui.switch("Wi-Fi", &mut s.wifi).show();
        ui.text_input(&mut s.name).label("Name").show();
    });
}

fn find(t: &TreeUpdate, role: Role, label: &str) -> Option<NodeId> {
    t.nodes.iter().find(|(_, n)| n.role() == role && n.label() == Some(label)).map(|(id, _)| *id)
}

#[test]
fn lumen_nodes_in_the_gpui_tree() {
    let mut cx = TestAppContext::single();
    let (entity, vcx) = cx.add_window_view(|window, cx| {
        window.set_a11y_forced(true);
        LumenView::new(S::default(), view, window, cx)
    });
    let tree = |vcx: &mut gpui::VisualTestContext| {
        // lumen's tree lags GPUI's by a frame: draw until it settles.
        for _ in 0..4 {
            vcx.update(|w, _| w.refresh());
            vcx.run_until_parked();
        }
        vcx.update(|w, _| w.a11y_tree().cloned()).expect("a tree")
    };
    let t = tree(vcx);
    let save = find(&t, Role::Button, "Save").unwrap_or_else(|| panic!("no Save button: {:#?}", t.nodes.iter().map(|(_, n)| (n.role(), n.label().map(str::to_owned))).collect::<Vec<_>>()));
    assert!(find(&t, Role::Heading, "Profile").is_some());
    let wifi = find(&t, Role::Switch, "Wi-Fi").expect("switch");
    let name = find(&t, Role::TextInput, "Name").expect("text input");
    // Every lumen node hangs off one parent, in the window's tree.
    let parents: usize = t.nodes.iter().map(|(_, n)| n.children().iter().filter(|c| **c == save).count()).sum();
    assert_eq!(parents, 1);

    vcx.update(|w, cx| w.perform_a11y_action(save, Action::Click, None, cx));
    vcx.update(|w, cx| w.perform_a11y_action(wifi, Action::Click, None, cx));
    vcx.update(|w, cx| w.perform_a11y_action(name, Action::SetValue, Some(ActionData::Value("Ada".into())), cx));
    let t = tree(vcx);
    let s = vcx.update(|_, cx| entity.read(cx).state().saves);
    assert_eq!(s, 1);
    assert!(vcx.update(|_, cx| entity.read(cx).state().wifi));
    assert_eq!(vcx.update(|_, cx| entity.read(cx).state().name.clone()), "Ada");
    let (_, sw) = t.nodes.iter().find(|(id, _)| *id == wifi).unwrap();
    assert_eq!(sw.toggled(), Some(Toggled::True));
}

struct Two {
    a: gpui::Entity<LumenView<S>>,
    b: gpui::Entity<LumenView<S>>,
}

impl gpui::Render for Two {
    fn render(&mut self, _: &mut gpui::Window, _: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
        use gpui::{div, prelude::*};
        div().size_full().flex().flex_col().child(div().h(gpui::px(240.)).child(self.a.clone())).child(div().h(gpui::px(240.)).child(self.b.clone()))
    }
}

/// Two lumen views in one window build the same node ids; each must keep
/// its own nodes, and a press must reach the view it belongs to.
#[test]
fn two_views_in_one_window() {
    let mut cx = TestAppContext::single();
    let (two, vcx) = cx.add_window_view(|window, cx| {
        window.set_a11y_forced(true);
        let a = cx.new(|cx| LumenView::new(S::default(), view, window, cx));
        let b = cx.new(|cx| LumenView::new(S::default(), view, window, cx));
        Two { a, b }
    });
    let mut t = None;
    for _ in 0..4 {
        vcx.update(|w, _| w.refresh());
        vcx.run_until_parked();
        t = vcx.update(|w, _| w.a11y_tree().cloned());
    }
    let t = t.expect("a tree");
    let saves: Vec<NodeId> = t.nodes.iter().filter(|(_, n)| n.role() == Role::Button && n.label() == Some("Save")).map(|(id, _)| *id).collect();
    assert_eq!(saves.len(), 2, "both views' buttons are in the tree");
    assert_ne!(saves[0], saves[1]);
    for s in &saves {
        let parents: usize = t.nodes.iter().map(|(_, n)| n.children().iter().filter(|c| *c == s).count()).sum();
        assert_eq!(parents, 1, "each button has one parent");
    }
    vcx.update(|w, cx| w.perform_a11y_action(saves[1], Action::Click, None, cx));
    vcx.update(|w, _| w.refresh());
    vcx.run_until_parked();
    let (a, b) = vcx.update(|_, cx| {
        let two = two.read(cx);
        (two.a.read(cx).state().saves, two.b.read(cx).state().saves)
    });
    assert_eq!(a + b, 1, "one press, one save");
}
