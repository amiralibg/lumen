//! Postbox: a three-pane mail client. Folders, a searchable message list,
//! a reading pane with a quick reply, and a compose dialog.

use super::{back, Fit};
use lumen::{Align, ListRow, Toast, Tone, Ui};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Folder {
    Inbox,
    Starred,
    Sent,
    Archive,
    Trash,
}

#[derive(Clone)]
struct Mail {
    from: &'static str,
    initials: &'static str,
    tone: usize,
    address: &'static str,
    subject: String,
    body: String,
    time: &'static str,
    unread: bool,
    starred: bool,
    folder: Folder,
    label: Option<(&'static str, Tone)>,
}

pub struct State {
    folder: Folder,
    mails: Vec<Mail>,
    selected: Option<usize>,
    search: String,
    unread_only: u8,
    reply: String,
    compose: bool,
    to: String,
    subject: String,
    body: String,
    /// Narrow layout: the message is open instead of the list.
    reading: bool,
}

fn seed() -> Vec<Mail> {
    let m = |from, initials, tone, address, subject: &str, body: &str, time, unread, starred, label| Mail {
        from,
        initials,
        tone,
        address,
        subject: subject.into(),
        body: body.into(),
        time,
        unread,
        starred,
        folder: Folder::Inbox,
        label,
    };
    vec![
        m("Priya Nair", "PN", 2, "priya@harbor.dev", "Design review moved to Thursday", "Hi team,\n\nThe design review for the onboarding flow moves to Thursday at 14:00. I've attached the latest prototype and the research notes from last week's sessions.\n\nTwo things I'd like us to settle in the room: whether the empty states get illustrations, and how far we take the reduced-motion variants.\n\nPriya", "09:42", true, false, Some(("Design", Tone::Accent))),
        m("Jonas Berg", "JB", 1, "jonas@northlight.io", "Re: GPU frame budget for the editor", "Measured on the M3 and the 2019 Intel MacBook: the editor stays under 4 ms a frame with 40k lines open. The glyph atlas is the only thing that grows, and it plateaus after the first scroll through the file.\n\nI'll write it up properly tomorrow.", "08:15", true, true, Some(("Perf", Tone::Success))),
        m("Lena Fischer", "LF", 0, "lena@kiln.app", "Invoice #2041 for November", "Hello,\n\nPlease find the invoice for November attached. Payment is due within 30 days. Let me know if anything needs to change on the billing address.\n\nBest,\nLena", "Yesterday", false, false, Some(("Billing", Tone::Warning))),
        m("Tomas Rivera", "TR", 3, "tomas@relay.so", "Release 2.4 is out", "2.4 shipped to all regions at 17:20 UTC. Error rates stayed flat through the rollout, and the canary bake caught the one regression we expected.\n\nThanks for the quick reviews this week.", "Yesterday", false, true, None),
        m("Aiko Kato", "AK", 4, "aiko@lattice.dev", "Coffee next week?", "I'm in Berlin from Tuesday to Friday. Want to grab a coffee and talk about the type scale work? I have opinions about optical sizes.", "Mon", false, false, None),
        m("Northwind Billing", "NB", 5, "billing@northwind.io", "Your receipt from Northwind", "Thanks for your payment of $48.00 for the Team plan. Your next charge is on December 18.", "Mon", false, false, Some(("Billing", Tone::Warning))),
        m("Marta Silva", "MS", 6, "marta@fieldnotes.co", "Interview notes: power users", "Seven interviews done. The strongest theme: people want keyboard shortcuts for everything, and they want to discover them without reading docs. Full notes inside.", "Sun", false, false, Some(("Research", Tone::Neutral))),
    ]
}

impl Default for State {
    fn default() -> Self {
        State {
            folder: Folder::Inbox,
            mails: seed(),
            selected: Some(0),
            search: String::new(),
            unread_only: 0,
            reply: String::new(),
            compose: false,
            reading: false,
            to: String::new(),
            subject: String::new(),
            body: String::new(),
        }
    }
}

fn in_folder(m: &Mail, f: Folder) -> bool {
    match f {
        Folder::Starred => m.starred && m.folder != Folder::Trash,
        f => m.folder == f,
    }
}

pub fn view(ui: &mut Ui, s: &mut State) {
    let visible: Vec<usize> = s
        .mails
        .iter()
        .enumerate()
        .filter(|(_, m)| in_folder(m, s.folder))
        .filter(|(_, m)| s.unread_only == 0 || m.unread)
        .filter(|(_, m)| {
            let q = s.search.to_lowercase();
            q.is_empty() || m.subject.to_lowercase().contains(&q) || m.from.to_lowercase().contains(&q) || m.body.to_lowercase().contains(&q)
        })
        .map(|(i, _)| i)
        .collect();
    if s.selected.is_some_and(|i| !visible.contains(&i)) {
        s.selected = visible.first().copied();
    }

    let fit = Fit::of(ui);
    ui.row().fill().show(|ui| {
        sidebar(ui, s, fit);
        // Narrow: the list or the open message, one at a time.
        if fit == Fit::Narrow && s.reading && s.selected.is_some() {
            reading_pane(ui, s, fit);
            return;
        }
        let list = ui.column().fill_height().pad(16).gap(12).bg(lumen::Token::Surface);
        let list = match fit {
            Fit::Wide => list.width(340.0),
            Fit::Medium => list.width(290.0),
            Fit::Narrow => list.fill_width(),
        };
        list.show(|ui| {
            ui.row().fill_width().align(Align::Center).gap(8).show(|ui| {
                ui.heading(folder_name(s.folder)).level(3);
                ui.spacer();
                ui.segmented(&mut s.unread_only, &[(0, "All"), (1, "Unread")]);
            });
            ui.search_field(&mut s.search).placeholder("Search mail").shortcut("⌘F").cmd_key('f').fill_width();
            if visible.is_empty() {
                ui.empty_state("inbox", "Nothing here").description("No messages match. Try another folder or clear the search.").compact().show();
            } else {
                let rows: Vec<ListRow> = visible
                    .iter()
                    .map(|&i| {
                        let m = &s.mails[i];
                        let mut r = ListRow::new(m.from).subtitle(m.subject.clone()).preview(m.body.lines().find(|l| !l.trim().is_empty() && !l.starts_with("Hi") && !l.starts_with("Hello")).unwrap_or("")).meta(m.time).avatar(m.initials, m.tone).unread(m.unread).starred(m.starred);
                        if let Some((l, t)) = m.label {
                            r = r.badge(l, t);
                        }
                        r
                    })
                    .collect();
                let mut sel = s.selected.and_then(|i| visible.iter().position(|&v| v == i));
                ui.scroll("mail_list").show(|ui| {
                    let out = ui.list_view("mail", &rows, &mut sel);
                    if out.clicked.is_some() || out.activated.is_some() {
                        s.reading = true;
                    }
                    if let Some(i) = out.starred {
                        let m = &mut s.mails[visible[i]];
                        m.starred = !m.starred;
                    }
                });
                s.selected = sel.map(|i| visible[i]);
            }
        });
        if fit != Fit::Narrow {
            reading_pane(ui, s, fit);
        }
    });

    compose(ui, s);
}

fn folder_name(f: Folder) -> &'static str {
    match f {
        Folder::Inbox => "Inbox",
        Folder::Starred => "Starred",
        Folder::Sent => "Sent",
        Folder::Archive => "Archive",
        Folder::Trash => "Trash",
    }
}

fn sidebar(ui: &mut Ui, s: &mut State, fit: Fit) {
    let count = |f: Folder, unread: bool| s.mails.iter().filter(|m| in_folder(m, f) && (!unread || m.unread)).count() as u32;
    let counts = [count(Folder::Inbox, true), count(Folder::Starred, false), count(Folder::Sent, false), count(Folder::Archive, false), count(Folder::Trash, false)];
    ui.sidebar().width(212.0).collapsed(fit.rail()).show(|ui| {
        ui.nav_header("P", "Postbox");
        ui.column().pad((8.0, 6.0)).fill_width().show(|ui| {
            let compose = if fit.rail() {
                ui.icon_button("pen-line").primary().tooltip("Compose").clicked()
            } else {
                ui.button("Compose").primary().icon("pen-line").full_width().clicked()
            };
            if compose {
                s.compose = true;
            }
        });
        ui.nav_section("Mailboxes");
        for (i, (f, icon)) in [(Folder::Inbox, "inbox"), (Folder::Starred, "star"), (Folder::Sent, "send"), (Folder::Archive, "archive"), (Folder::Trash, "trash-2")].into_iter().enumerate() {
            let mut item = ui.nav_item(icon, folder_name(f)).selected(s.folder == f);
            if counts[i] > 0 && (i == 0 || i == 1) {
                item = item.badge(counts[i]);
            }
            if item.clicked() {
                s.folder = f;
                s.selected = None;
                s.reading = false;
            }
        }
        ui.nav_section("Labels");
        for (icon, l) in [("palette", "Design"), ("gauge", "Perf"), ("receipt", "Billing"), ("notebook-pen", "Research")] {
            ui.nav_item(icon, l);
        }
    });
}

fn reading_pane(ui: &mut Ui, s: &mut State, fit: Fit) {
    let pad = if fit == Fit::Narrow { 16.0 } else { 20.0 };
    ui.column().fill().pad(pad).gap(16).show(|ui| {
        let Some(i) = s.selected else {
            ui.empty_state("mail-open", "No message selected").description("Pick a message from the list to read it here.").show();
            return;
        };
        if fit == Fit::Narrow && back(ui, folder_name(s.folder)) {
            s.reading = false;
        }
        if s.mails[i].unread {
            s.mails[i].unread = false;
        }
        let mut action = None;
        ui.toolbar("mail_actions", |tb| {
            if tb.action("archive", "Archive") {
                action = Some("archive");
            }
            if tb.action("trash-2", "Delete") {
                action = Some("trash");
            }
            tb.separator();
            if tb.action("mail", "Mark unread") {
                action = Some("unread");
            }
            if tb.action("star", if s.mails[i].starred { "Unstar" } else { "Star" }) {
                action = Some("star");
            }
        });
        let m = s.mails[i].clone();
        ui.scroll("mail_body").gap(16.0).show(|ui| {
            ui.row().fill_width().gap(8).align(Align::Center).show(|ui| {
                ui.heading(m.subject.clone()).level(2);
                if let Some((l, t)) = m.label {
                    ui.badge(l, t);
                }
            });
            ui.row().fill_width().gap(12).align(Align::Center).show(|ui| {
                ui.avatar(m.initials, m.tone, 40.0, Some(true));
                ui.column().gap(2).fill_width().show(|ui| {
                    ui.label(m.from).strong();
                    ui.label(m.address).muted().small();
                });
                ui.label(m.time).muted().small();
            });
            ui.divider();
            for para in m.body.split("\n\n") {
                ui.label(para.to_string());
            }
            ui.card().pad(14.0).show(|ui| {
                ui.text_area(&mut s.reply).placeholder(format!("Reply to {}…", m.from.split(' ').next().unwrap_or(m.from))).rows(3, 8).fill_width();
                ui.row().fill_width().gap(8).show(|ui| {
                    ui.spacer();
                    let empty = s.reply.trim().is_empty();
                    if ui.button("Send reply").primary().icon("send").disabled(empty).clicked() {
                        let mut sent = m.clone();
                        sent.folder = Folder::Sent;
                        sent.subject = format!("Re: {}", m.subject.trim_start_matches("Re: "));
                        sent.body = std::mem::take(&mut s.reply);
                        sent.time = "Now";
                        sent.unread = false;
                        s.mails.insert(0, sent);
                        s.selected = Some(i + 1);
                        ui.toast(Toast::new("Reply sent").description(format!("To {}", m.address)).success());
                    }
                });
            });
        });
        match action {
            Some("archive") => {
                s.reading = false;
                s.mails[i].folder = Folder::Archive;
                ui.toast(Toast::new("Conversation archived").description(m.subject.clone()));
            }
            Some("trash") => {
                s.reading = false;
                s.mails[i].folder = Folder::Trash;
                ui.toast(Toast::new("Moved to Trash").description(m.subject.clone()).danger());
            }
            Some("unread") => {
                s.mails[i].unread = true;
                s.selected = None;
            }
            Some("star") => s.mails[i].starred = !s.mails[i].starred,
            _ => {}
        }
    });
}

fn compose(ui: &mut Ui, s: &mut State) {
    let mut open = s.compose;
    let mut send = false;
    ui.dialog("compose", &mut open)
        .title("New message")
        .width(560.0)
        .footer(|ui| {
            ui.spacer();
            ui.button("Discard").ghost();
            send = ui.button("Send").primary().icon("send").clicked();
        })
        .show(|ui| {
            ui.text_input(&mut s.to).label("To").placeholder("name@company.com").fill_width();
            ui.text_input(&mut s.subject).label("Subject").fill_width();
            ui.text_area(&mut s.body).label("Message").rows(6, 12).fill_width();
        });
    if send {
        let subject = if s.subject.trim().is_empty() { "(no subject)".to_string() } else { s.subject.clone() };
        s.mails.insert(0, Mail { from: "You", initials: "ME", tone: 7, address: "you@postbox.app", subject: subject.clone(), body: std::mem::take(&mut s.body), time: "Now", unread: false, starred: false, folder: Folder::Sent, label: None });
        ui.toast(Toast::new("Message sent").description(subject).success());
        s.to.clear();
        s.subject.clear();
        open = false;
    }
    s.compose = open;
}
