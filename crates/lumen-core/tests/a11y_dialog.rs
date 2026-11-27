//! A menu item opens a dialog; a screen reader presses its button. Frames
//! run only while something animates or input arrives, like an app.

use lumen_core::*;

#[derive(Default)]
struct S {
    dialog: bool,
    log: String,
}

fn view(ui: &mut Ui, s: &mut S) {
    ui.column().pad(16.0).show(|ui| {
        let more = ui.button("More").show();
        ui.menu(&more, |ui| {
            if ui.menu_item("trash", "Delete…").show().clicked() {
                s.dialog = true;
            }
        });
    });
    let mut deleted = false;
    ui.dialog("c", &mut s.dialog).title("Delete profile?").show(|ui| {
        ui.row().gap(8).show(|ui| {
            if ui.button("Cancel").show().clicked() {
                ui.close_layer();
            }
            if ui.button("Delete").show().clicked() {
                deleted = true;
                ui.close_layer();
            }
        });
    });
    if deleted {
        s.log = "Deleted".into();
    }
}

#[test]
fn dialog_from_menu_then_click() {
    let mut ui = Ui::new();
    ui.set_accessibility(true);
    let mut s = S::default();
    let mut t = 0.0;
    let mut tree = None;
    let mut frame = |ui: &mut Ui, s: &mut S, ev: Vec<Event>| -> FrameOutput {
        t += 1.0 / 60.0;
        ui.run(RawInput { screen_size: vec2(800.0, 600.0), time: t, events: ev, ..Default::default() }, |ui| view(ui, s))
    };
    // Run like an app: frames only while animating.
    let mut settle = |ui: &mut Ui, s: &mut S, ev: Vec<Event>, tree: &mut Option<A11yTree>| {
        let mut out = frame(ui, s, ev);
        let mut n = 0;
        while out.animating && n < 200 {
            out = frame(ui, s, vec![]);
            n += 1;
        }
        *tree = out.a11y;
    };
    settle(&mut ui, &mut s, vec![], &mut tree);
    let id = |tree: &Option<A11yTree>, r: Role, l: &str| Id(tree.as_ref().unwrap().find(r, l).unwrap_or_else(|| panic!("{l}\n{}", tree.as_ref().unwrap().dump())).id);
    let more = id(&tree, Role::Button, "More");
    settle(&mut ui, &mut s, vec![Event::A11y { target: more, action: A11yAction::Click }], &mut tree);
    let del = id(&tree, Role::MenuItem, "Delete…");
    settle(&mut ui, &mut s, vec![Event::A11y { target: del, action: A11yAction::Click }], &mut tree);
    println!("{}", tree.as_ref().unwrap().dump());
    let d = id(&tree, Role::Button, "Delete");
    settle(&mut ui, &mut s, vec![Event::A11y { target: d, action: A11yAction::Click }], &mut tree);
    assert_eq!(s.log, "Deleted", "{}", tree.as_ref().unwrap().dump());
}

/// A click that changes state the view already read this frame asks for
/// another frame, so the change shows without waiting for more input.
#[test]
fn input_frames_ask_for_one_more() {
    let mut ui = Ui::new();
    let mut s = S::default();
    let mut t = 0.0;
    let mut run = |ui: &mut Ui, s: &mut S, ev: Vec<Event>| {
        t += 1.0;
        ui.run(RawInput { screen_size: vec2(800.0, 600.0), time: t, events: ev, ..Default::default() }, |ui| view(ui, s))
    };
    run(&mut ui, &mut s, vec![]);
    let idle = run(&mut ui, &mut s, vec![]);
    assert_eq!(idle.repaint_after, None, "idle frames don't spin");
    let more = ui.rect_of(Id::new(("button", "More"))).map(|r| r.center());
    let p = more.unwrap_or(vec2(40.0, 30.0));
    run(&mut ui, &mut s, vec![Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true }]);
    let out = run(&mut ui, &mut s, vec![Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false }]);
    assert_eq!(out.repaint_after, Some(0.0));
}
