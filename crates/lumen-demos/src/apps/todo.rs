//! Today: the to-do app from the tutorial. Add, tick off, filter, clear.

use lumen::{Align, Toast, Tone, Ui};

pub struct State {
    items: Vec<(String, bool)>,
    draft: String,
    filter: u8,
}

impl Default for State {
    fn default() -> Self {
        State {
            items: vec![("Water the plants".into(), true), ("Book the dentist".into(), false), ("Reply to Priya about Thursday".into(), false), ("Proof the bread for longer".into(), false)],
            draft: String::new(),
            filter: 0,
        }
    }
}

pub fn view(ui: &mut Ui, s: &mut State) {
    ui.column().fill().pad(32).align(Align::Center).show(|ui| {
        ui.column().width(520.0).gap(16).show(|ui| {
            let left = s.items.iter().filter(|i| !i.1).count();
            ui.row().fill_width().align(Align::Center).gap(10).show(|ui| {
                ui.heading("Today").level(1);
                ui.badge(format!("{left} left"), if left == 0 { Tone::Success } else { Tone::Accent });
            });
            ui.row().fill_width().gap(8).show(|ui| {
                ui.text_input(&mut s.draft).placeholder("Add a task and press Add").fill_width();
                if ui.button("Add").primary().icon("plus").clicked() && !s.draft.trim().is_empty() {
                    s.items.push((std::mem::take(&mut s.draft), false));
                }
            });
            ui.segmented(&mut s.filter, &[(0, "All"), (1, "Active"), (2, "Done")]);
            ui.card().pad(8.0).show(|ui| {
                let mut shown = 0;
                for (i, (text, done)) in s.items.iter_mut().enumerate() {
                    if (s.filter == 1 && *done) || (s.filter == 2 && !*done) {
                        continue;
                    }
                    shown += 1;
                    ui.scope(i, |ui| {
                        ui.row().fill_width().pad((8.0, 6.0)).show(|ui| {
                            ui.checkbox(text.clone(), done);
                        });
                    });
                }
                if shown == 0 {
                    ui.empty_state("party-popper", "Nothing here").description("Add a task above, or switch the filter.").compact().show();
                }
            });
            ui.row().fill_width().align(Align::Center).show(|ui| {
                ui.label(format!("{} of {} done", s.items.len() - left, s.items.len())).muted();
                ui.spacer();
                let done = s.items.len() - left;
                if ui.button("Clear done").ghost().disabled(done == 0).clicked() {
                    s.items.retain(|i| !i.1);
                    ui.toast(Toast::new(format!("Cleared {done} done task{}", if done == 1 { "" } else { "s" })));
                }
            });
        });
    });
}
