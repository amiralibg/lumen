//! `Ui::check_layout`: text spilling out of a squeezed fixed-size leaf.

use lumen_core::layout::Style;
use lumen_core::ui::Sense;
use lumen_core::*;

fn issues(view: impl Fn(&mut Ui)) -> Vec<String> {
    let mut ui = Ui::new();
    ui.check_layout = true;
    let mut t = 1.0;
    for _ in 0..3 {
        t += 1.0 / 60.0;
        let input = RawInput { screen_size: vec2(480.0, 320.0), scale: 1.0, time: t, events: vec![], modifiers: Modifiers::default() };
        ui.run(input, |ui| view(ui));
    }
    ui.layout_issues().to_vec()
}

/// A leaf sized from its label but without a `min`: a narrow row squeezes
/// it and the label draws past the rect.
fn label_pill(ui: &mut Ui, label: &str) {
    let st = ui.text_style(TextRole::Body);
    let g = ui.galley(label, &st);
    let fg = ui.color(Token::Foreground);
    let w = g.width + 24.0;
    ui.leaf(
        Style::default(),
        vec2(w, 28.0),
        None,
        Sense::NONE,
        CursorIcon::Default,
        Some(Box::new(move |cx, r| cx.painter.text(cx.text, &g, vec2(r.min.x + 12.0, r.min.y + 4.0), fg))),
    );
}

#[test]
fn squeezed_leaf_with_text_is_reported() {
    let found = issues(|ui| {
        ui.row().max_width(160.0).gap(8).show(|ui| {
            for l in ["release-candidate", "nightly-builds", "macos-14"] {
                label_pill(ui, l);
            }
        });
    });
    assert!(found.iter().any(|s| s.starts_with("squeezed:")), "{found:#?}");
}

#[test]
fn chips_with_room_are_clean() {
    let found = issues(|ui| {
        ui.row().max_width(160.0).gap(8).wrap().show(|ui| {
            let mut on = [true, false, false];
            for (i, l) in ["arm64", "gpu", "docker"].iter().enumerate() {
                ui.chip(l, &mut on[i]);
            }
        });
    });
    assert!(!found.iter().any(|s| s.starts_with("squeezed:")), "{found:#?}");
}
