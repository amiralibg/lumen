//! Margin: a Markdown notes app. Folders, a searchable note list, an editor
//! with write, preview and split modes, tags and a word count.

use lumen::{Align, ListRow, Toast, Tone, Ui};

struct Note {
    title: String,
    body: String,
    folder: usize,
    pinned: bool,
    tags: Vec<String>,
    edited: &'static str,
}

pub struct State {
    folder: usize,
    notes: Vec<Note>,
    selected: Option<usize>,
    search: String,
    mode: u8,
}

const FOLDERS: [(&str, &str); 3] = [("notebook", "Journal"), ("lightbulb", "Ideas"), ("briefcase", "Work")];

impl Default for State {
    fn default() -> Self {
        let n = |title: &str, body: &str, folder, pinned, tags: &[&str], edited| Note { title: title.into(), body: body.into(), folder, pinned, tags: tags.iter().map(|t| t.to_string()).collect(), edited };
        State {
            folder: 2,
            notes: vec![
                n("Launch checklist", "# Launch checklist\n\nThings to finish before **2.4** goes out.\n\n- Screenshots for every look\n- Changelog entry\n- Re-run the contrast audit\n- Tag the release\n\n> Ship when the checklist is green, not when the calendar says so.", 2, true, &["release", "todo"], "2 min ago"),
                n("Interview notes: power users", "# Interview notes\n\nSeven interviews. The strongest theme: **keyboard first**.\n\n- Shortcuts should be discoverable in place\n- Nobody reads the docs before trying\n- Undo matters more than confirm dialogs", 2, false, &["research"], "Yesterday"),
                n("Type scale thoughts", "# Type scale\n\nTry a 1.2 ratio for body sizes and 1.25 for headings. Optical sizes for Inter at display weights.", 1, false, &["design"], "Mon"),
                n("Saturday", "# Saturday\n\nLong walk by the canal. Finished the book about lighthouses. Bread came out flat again; try a longer proof.", 0, false, &[], "Sat"),
                n("App ideas", "# App ideas\n\n- A caffeine curve: how much is left at bedtime\n- Plant watering with a gentle reminder\n- A focus timer that dims the whole desktop", 1, true, &["ideas"], "Oct 30"),
            ],
            selected: Some(0),
            search: String::new(),
            mode: 2,
        }
    }
}

pub fn view(ui: &mut Ui, s: &mut State) {
    ui.row().fill().show(|ui| {
        ui.sidebar().width(200.0).show(|ui| {
            ui.nav_header("M", "Margin");
            ui.nav_section("Folders");
            for (i, (icon, name)) in FOLDERS.iter().enumerate() {
                let n = s.notes.iter().filter(|x| x.folder == i).count() as u32;
                if ui.nav_item(icon, *name).badge(n).selected(s.folder == i).clicked() {
                    s.folder = i;
                    s.selected = None;
                }
            }
            ui.nav_section("Tags");
            for t in ["release", "research", "design", "ideas"] {
                ui.nav_item("hash", t);
            }
        });

        let q = s.search.to_lowercase();
        let mut visible: Vec<usize> = (0..s.notes.len()).filter(|&i| s.notes[i].folder == s.folder).filter(|&i| q.is_empty() || s.notes[i].title.to_lowercase().contains(&q) || s.notes[i].body.to_lowercase().contains(&q)).collect();
        visible.sort_by_key(|&i| !s.notes[i].pinned);
        if s.selected.is_some_and(|i| !visible.contains(&i)) || s.selected.is_none() {
            s.selected = visible.first().copied();
        }

        ui.column().width(300.0).fill_height().pad(16).gap(12).bg(lumen::Token::Surface).show(|ui| {
            ui.row().fill_width().align(Align::Center).show(|ui| {
                ui.heading(FOLDERS[s.folder].1).level(3);
                ui.spacer();
                if ui.icon_button("square-pen").tooltip("New note").clicked() {
                    s.notes.push(Note { title: "Untitled".into(), body: "# Untitled\n\n".into(), folder: s.folder, pinned: false, tags: vec![], edited: "Just now" });
                    s.selected = Some(s.notes.len() - 1);
                }
            });
            ui.search_field(&mut s.search).placeholder("Search notes").fill_width();
            let rows: Vec<ListRow> = visible
                .iter()
                .map(|&i| {
                    let x = &s.notes[i];
                    let preview: String = x.body.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()).map(|l| l.trim_start_matches("- ").trim_start_matches("> ").replace("**", "")).collect::<Vec<_>>().join(" ");
                    let mut r = ListRow::new(x.title.clone()).meta(x.edited).preview(preview).starred(x.pinned);
                    if let Some(t) = x.tags.first() {
                        r = r.subtitle(format!("#{t}"));
                    }
                    r
                })
                .collect();
            let mut sel = s.selected.and_then(|i| visible.iter().position(|&v| v == i));
            if rows.is_empty() {
                ui.empty_state("notebook-pen", "No notes").description("Create one with the pen button.").compact().show();
            }
            ui.scroll("notes").show(|ui| {
                let out = ui.list_view("notes", &rows, &mut sel);
                if let Some(i) = out.starred {
                    let n = &mut s.notes[visible[i]];
                    n.pinned = !n.pinned;
                }
            });
            s.selected = sel.map(|i| visible[i]);
        });

        ui.column().fill().pad(20).gap(12).show(|ui| editor(ui, s));
    });
}

fn editor(ui: &mut Ui, s: &mut State) {
    let Some(i) = s.selected else {
        ui.empty_state("file-text", "No note open").description("Pick a note or start a new one.").show();
        return;
    };
    let mut delete = false;
    ui.row().fill_width().gap(8).align(Align::Center).show(|ui| {
        ui.editable_text(format!("note_title_{i}"), &mut s.notes[i].title).heading();
        if s.notes[i].pinned {
            ui.badge("Pinned", Tone::Warning);
        }
        ui.spacer();
        ui.segmented(&mut s.mode, &[(0, "Write"), (1, "Preview"), (2, "Split")]);
        if ui.icon_button("trash-2").ghost().tooltip("Delete note").clicked() {
            delete = true;
        }
    });
    ui.tag_input(&mut s.notes[i].tags).placeholder("Add a tag").fill_width();
    let note = &mut s.notes[i];
    match s.mode {
        0 => {
            ui.text_area(&mut note.body).rows(16, 40).mono().fill_width();
        }
        1 => {
            ui.scroll(("preview", i)).show(|ui| {
                ui.markdown(("md", i), &note.body);
            });
        }
        _ => {
            ui.row().fill().gap(16).align(Align::Start).show(|ui| {
                ui.column().fill_width().show(|ui| {
                    ui.text_area(&mut note.body).rows(16, 40).mono().fill_width();
                });
                ui.column().fill_width().show(|ui| {
                    ui.markdown(("md", i), &note.body);
                });
            });
        }
    }
    let words = note.body.split_whitespace().filter(|w| !w.starts_with('#') && *w != "-").count();
    let chars = note.body.chars().count();
    ui.status_bar(|bar| {
        bar.status("Saved", Tone::Success);
        bar.item("clock", note.edited);
        bar.spacer();
        bar.item("type", &format!("{words} words · {chars} characters"));
    });
    if delete {
        let t = s.notes.remove(i).title;
        s.selected = None;
        ui.toast(Toast::new("Note deleted").description(t).danger());
    }
}
