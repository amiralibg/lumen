//! Relay: an issue tracker. Projects, a filterable list or a board, an
//! issue detail sheet and a new-issue dialog.

use super::Fit;
use lumen::{Align, KanbanCard, KanbanCol, ListRow, Placement, TimelineItem, Toast, Tone, Ui, ViewMode};

const STATUS: [&str; 4] = ["Backlog", "Todo", "In progress", "Done"];
const PRIORITY: [(&str, Tone); 4] = [("Low", Tone::Neutral), ("Medium", Tone::Accent), ("High", Tone::Warning), ("Urgent", Tone::Danger)];
const PEOPLE: [(&str, &str); 5] = [("PN", "Priya Nair"), ("JB", "Jonas Berg"), ("LF", "Lena Fischer"), ("TR", "Tomas Rivera"), ("AK", "Aiko Kato")];

struct Issue {
    key: String,
    title: String,
    status: usize,
    priority: usize,
    assignee: usize,
    label: &'static str,
    body: &'static str,
}

pub struct State {
    project: usize,
    issues: Vec<Issue>,
    view: usize,
    show: [bool; 4],
    selected: Option<usize>,
    detail: bool,
    new_open: bool,
    new_title: String,
    new_priority: usize,
    new_assignee: usize,
    next: u32,
}

impl Default for State {
    fn default() -> Self {
        let i = |n: u32, title: &str, status, priority, assignee, label, body| Issue { key: format!("REL-{n}"), title: title.into(), status, priority, assignee, label, body };
        State {
            project: 0,
            issues: vec![
                i(142, "Command palette ignores reduced motion", 2, 2, 1, "Bug", "The palette's list stagger still runs when the OS asks for reduced motion. It should cut straight to the final state, like the select menu does."),
                i(141, "Data grid: keep selection when sorting", 1, 1, 2, "Feature", "Sorting a column clears the selected rows. Keep the selection by row id so it survives sorts and filters."),
                i(139, "Toast stack overlaps the dock on small windows", 1, 3, 0, "Bug", "Below 640 px the toast stack sits on top of the dock. Offset it by the dock's height when both are visible."),
                i(137, "Ship GNOME look accent detection", 2, 1, 3, "Platform", "Read the accent from gsettings and fall back to blue. Needs testing on Fedora and Ubuntu."),
                i(133, "Calendar: week numbers option", 0, 0, 4, "Feature", "Show ISO week numbers in a narrow leading column. Off by default."),
                i(131, "Line chart tooltip clips at the right edge", 3, 1, 1, "Bug", "Fixed: the tooltip now flips to the left of the crosshair near the edge."),
                i(128, "Docs: GPUI hosting guide", 3, 0, 0, "Docs", "Covers LumenView, sharing state and the cost of the offscreen bridge."),
                i(126, "Virtualized list for 100k rows", 0, 2, 2, "Feature", "Only lay out the rows in view, with a spacer above and below."),
            ],
            view: 0,
            show: [true; 4],
            selected: None,
            detail: false,
            new_open: false,
            new_title: String::new(),
            new_priority: 1,
            new_assignee: 0,
            next: 143,
        }
    }
}

pub fn view(ui: &mut Ui, s: &mut State) {
    let fit = Fit::of(ui);
    ui.row().fill().show(|ui| {
        ui.sidebar().width(212.0).collapsed(fit.rail()).show(|ui| {
            ui.nav_header("R", "Relay");
            ui.nav_section("Projects");
            for (i, (icon, name)) in [("layers", "lumen"), ("globe", "Website"), ("shield", "Vault")].iter().enumerate() {
                let open = s.issues.iter().filter(|x| x.status < 3).count() as u32;
                let mut item = ui.nav_item(icon, *name).selected(s.project == i);
                if i == 0 {
                    item = item.badge(open);
                }
                if item.clicked() {
                    s.project = i;
                }
            }
            ui.nav_section("Views");
            ui.nav_item("user", "Assigned to me");
            ui.nav_item("flame", "High priority");
            ui.nav_item("calendar-clock", "This cycle");
        });
        let pad = if fit == Fit::Narrow { 16.0 } else { 20.0 };
        ui.column().fill().pad(pad).gap(14).show(|ui| main(ui, s, fit));
    });
    detail(ui, s, fit);
    new_issue(ui, s);
}

fn main(ui: &mut Ui, s: &mut State, fit: Fit) {
    let views = [ViewMode::new("list", "List"), ViewMode::new("kanban", "Board")];
    ui.row().fill_width().gap(10).align(Align::Center).show(|ui| {
        ui.heading("lumen").level(2);
        let done = s.issues.iter().filter(|x| x.status == 3).count();
        ui.badge(format!("{done} of {} done", s.issues.len()), Tone::Success);
        ui.spacer();
        if fit == Fit::Narrow {
            if ui.icon_button("plus").primary().tooltip("New issue").clicked() {
                s.new_open = true;
            }
        } else {
            ui.view_switcher("rel_view", &views, &mut s.view);
            if ui.button("New issue").primary().icon("plus").clicked() {
                s.new_open = true;
            }
        }
    });
    if fit == Fit::Narrow {
        ui.view_switcher("rel_view", &views, &mut s.view);
    }
    let total = s.issues.len().max(1) as f32;
    let done = s.issues.iter().filter(|x| x.status == 3).count() as f32;
    ui.progress(Some(done / total));

    if s.view == 1 {
        let cols: Vec<KanbanCol> = STATUS.iter().zip([Tone::Neutral, Tone::Accent, Tone::Warning, Tone::Success]).map(|(n, t)| KanbanCol::new(*n, t)).collect();
        let mut cards: Vec<KanbanCard> = s
            .issues
            .iter()
            .map(|x| KanbanCard::new(x.key.clone(), x.status, x.title.clone()).tag(PRIORITY[x.priority].0, PRIORITY[x.priority].1).assignee(PEOPLE[x.assignee].0).meta(x.key.clone()))
            .collect();
        ui.scroll("board").show(|ui| {
            let moved = if fit == Fit::Narrow {
                // Four columns don't fit a phone: they keep a usable width
                // and the board scrolls sideways.
                let row = lumen::layout::Style { width: lumen::Size::FILL, ..Default::default() };
                ui.scroll_row("board_x", row, |ui| ui.column().width(4.0 * 210.0).show(|ui| ui.kanban("rel_board", &cols, &mut cards)).inner).0
            } else {
                ui.kanban("rel_board", &cols, &mut cards)
            };
            if let Some((card, col)) = moved {
                s.issues[card].status = col;
                ui.toast(Toast::new(format!("{} moved to {}", s.issues[card].key, STATUS[col])));
            }
        });
        return;
    }

    ui.row().fill_width().gap(6).wrap().align(Align::Center).show(|ui| {
        for (i, st) in STATUS.iter().enumerate() {
            let n = s.issues.iter().filter(|x| x.status == i).count();
            ui.chip(&format!("{st} {n}"), &mut s.show[i]);
        }
    });
    let visible: Vec<usize> = (0..s.issues.len()).filter(|&i| s.show[s.issues[i].status]).collect();
    if visible.is_empty() {
        ui.empty_state("list-filter", "No issues match").description("Turn a status filter back on to see its issues.").show();
        return;
    }
    let rows: Vec<ListRow> = visible
        .iter()
        .map(|&i| {
            let x = &s.issues[i];
            let icon = ["circle-dashed", "circle", "loader-circle", "circle-check"][x.status];
            ListRow::new(x.title.clone()).subtitle(format!("{} · {} · {}", x.key, x.label, PEOPLE[x.assignee].1)).icon(icon).badge(PRIORITY[x.priority].0, PRIORITY[x.priority].1).meta(STATUS[x.status])
        })
        .collect();
    let mut sel = s.selected.and_then(|i| visible.iter().position(|&v| v == i));
    ui.scroll("issues").show(|ui| {
        let out = ui.list_view("issues", &rows, &mut sel);
        // On a phone a tap opens the issue.
        if out.activated.is_some() || (fit == Fit::Narrow && out.clicked.is_some()) {
            s.detail = true;
        }
    });
    s.selected = sel.map(|i| visible[i]);
    if fit != Fit::Narrow {
        ui.label("Double-click an issue, or press Enter, to open it.").muted().small();
    }
}

fn detail(ui: &mut Ui, s: &mut State, fit: Fit) {
    let Some(i) = s.selected else { return };
    let mut open = s.detail;
    let key = s.issues[i].key.clone();
    let title = s.issues[i].title.clone();
    // A phone gets a bottom sheet over most of the screen.
    let (side, size) = if fit == Fit::Narrow { (Placement::Bottom, (ui.screen_size().y * 0.85).round()) } else { (Placement::Right, 420.0) };
    ui.sheet("issue", &mut open).side(side).size(size).title(key.clone()).description(title).show(|ui| {
        let x = &mut s.issues[i];
        ui.label(x.body);
        ui.select(&mut x.status, &[(0usize, "Backlog"), (1, "Todo"), (2, "In progress"), (3, "Done")]).label("Status");
        ui.column().gap(6).fill_width().show(|ui| {
            ui.label("Priority").strong();
            ui.segmented(&mut x.priority, &[(0usize, "Low"), (1, "Medium"), (2, "High"), (3, "Urgent")]);
        });
        let people: Vec<(usize, &str)> = PEOPLE.iter().enumerate().map(|(k, p)| (k, p.1)).collect();
        ui.select(&mut x.assignee, &people).label("Assignee");
        ui.divider();
        ui.label("Activity").strong();
        ui.timeline(
            ("activity", &key),
            &[
                TimelineItem::new("Status changed to In progress", "2h ago").description(PEOPLE[x.assignee].1),
                TimelineItem::new("Priority raised", "Yesterday").description("Triage"),
                TimelineItem::new("Issue created", "Mon").description("Priya Nair"),
            ],
        );
    });
    s.detail = open;
}

fn new_issue(ui: &mut Ui, s: &mut State) {
    let mut open = s.new_open;
    let mut create = false;
    ui.dialog("new_issue", &mut open)
        .title("New issue")
        .description("It lands in Todo for the current cycle.")
        .width(520.0)
        .footer(|ui| {
            ui.spacer();
            ui.button("Cancel").ghost();
            create = ui.button("Create issue").primary().clicked();
        })
        .show(|ui| {
            ui.text_input(&mut s.new_title).label("Title").placeholder("What needs doing?").fill_width();
            ui.column().gap(6).fill_width().show(|ui| {
                ui.label("Priority").strong();
                ui.segmented(&mut s.new_priority, &[(0usize, "Low"), (1, "Medium"), (2, "High"), (3, "Urgent")]);
            });
            let people: Vec<(usize, &str)> = PEOPLE.iter().enumerate().map(|(k, p)| (k, p.1)).collect();
            ui.select(&mut s.new_assignee, &people).label("Assignee");
        });
    if create {
        let title = if s.new_title.trim().is_empty() { "Untitled issue".to_string() } else { std::mem::take(&mut s.new_title) };
        let key = format!("REL-{}", s.next);
        s.next += 1;
        s.issues.insert(0, Issue { key: key.clone(), title: title.clone(), status: 1, priority: s.new_priority, assignee: s.new_assignee, label: "Feature", body: "No description yet." });
        s.selected = Some(0);
        ui.toast(Toast::new(format!("{key} created")).description(title).success());
        open = false;
    }
    s.new_open = open;
}
