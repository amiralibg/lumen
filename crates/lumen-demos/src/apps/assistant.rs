//! Lumi: an AI assistant. Conversation history, a chat thread whose
//! replies stream in word by word, a composer with model choice and
//! attachments, and suggested prompts for an empty chat.

use lumen::{Align, ChatMessage, ListRow, Toast, Tone, Ui};

const MODELS: [&str; 3] = ["Quick", "Balanced", "Deep think"];

struct Chat {
    title: String,
    when: &'static str,
    messages: Vec<ChatMessage>,
}

pub struct State {
    chats: Vec<Chat>,
    selected: Option<usize>,
    prompt: String,
    model: usize,
    files: Vec<String>,
    /// Reply being streamed: (chat, full text, start time).
    stream: Option<(usize, String, f64)>,
}

impl Default for State {
    fn default() -> Self {
        State {
            chats: vec![
                Chat {
                    title: "Spring physics for UI".into(),
                    when: "Today",
                    messages: vec![
                        ChatMessage::user("Why do UI toolkits use springs instead of easing curves?").time("10:02"),
                        ChatMessage::assistant("Lumi", "A spring keeps its velocity when you interrupt it. If a toggle is halfway through animating and you click again, an easing curve has to restart from zero speed, which looks like a hitch. A spring just changes its target and carries its momentum into the new direction.\n\nSolved in closed form, a spring is also exact at any frame rate: the position at time t doesn't depend on how many frames you drew to get there.").citations(&["Hooke's law", "Damped oscillators"]).time("10:02"),
                    ],
                },
                Chat { title: "Trip to Lisbon".into(), when: "Yesterday", messages: vec![ChatMessage::user("Three days in Lisbon in March: what should I not miss?").time("18:40"), ChatMessage::assistant("Lumi", "Walk Alfama early before the crowds, take tram 28 once for the ride, then spend an afternoon in Belém. Keep one evening for a miradouro at sunset; Senhora do Monte has the widest view.").time("18:41")] },
                Chat { title: "Regex for ISO dates".into(), when: "Mon", messages: vec![ChatMessage::user("Regex for YYYY-MM-DD?").time("09:15"), ChatMessage::assistant("Lumi", "^\\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12]\\d|3[01])$ checks the shape and the ranges. It won't reject 2026-02-30; parse the date to be sure.").time("09:15")] },
            ],
            selected: Some(0),
            prompt: String::new(),
            model: 1,
            files: vec![],
            stream: None,
        }
    }
}

const REPLY: &str = "Good question. Here's how I'd approach it:\n\n1. Start from what the user is trying to finish, not from the feature list.\n2. Make the common path one step shorter than it is today.\n3. Measure it: time to first success is the number that matters.\n\nWant me to turn this into a checklist?";

pub fn view(ui: &mut Ui, s: &mut State) {
    let now = ui.time();
    // Stream the reply a few words at a time.
    if let Some((ci, full, t0)) = &s.stream {
        let words: Vec<&str> = full.split_inclusive(' ').collect();
        let n = (((now - t0) * 18.0) as usize).min(words.len());
        let text: String = words[..n].concat();
        if let Some(last) = s.chats[*ci].messages.last_mut() {
            last.body = text;
        }
        if n >= words.len() {
            s.stream = None;
        } else {
            ui.request_repaint_after(0.0);
        }
    }

    ui.row().fill().show(|ui| {
        ui.column().width(260.0).fill_height().pad(14).gap(10).bg(lumen::Token::Surface).show(|ui| {
            ui.row().fill_width().gap(8).align(Align::Center).show(|ui| {
                ui.avatar("L", 3, 28.0, None);
                ui.label("Lumi").strong();
                ui.spacer();
                if ui.icon_button("square-pen").tooltip("New chat").clicked() {
                    s.chats.insert(0, Chat { title: "New chat".into(), when: "Now", messages: vec![] });
                    s.selected = Some(0);
                    if let Some((ci, _, _)) = &mut s.stream {
                        *ci += 1;
                    }
                }
            });
            let rows: Vec<ListRow> = s.chats.iter().map(|c| ListRow::new(c.title.clone()).icon("message-square").meta(c.when)).collect();
            ui.scroll("chats").show(|ui| {
                ui.list_view("chats", &rows, &mut s.selected);
            });
            ui.card().pad(12.0).show(|ui| {
                ui.label("Free plan").strong();
                ui.label("38 of 50 messages left today").muted().small();
                ui.progress(Some(0.24));
            });
        });

        ui.column().fill().pad((24.0, 18.0)).gap(14).show(|ui| {
            let ci = s.selected.unwrap_or(0).min(s.chats.len().saturating_sub(1));
            ui.row().fill_width().gap(8).align(Align::Center).show(|ui| {
                ui.heading(s.chats[ci].title.clone()).level(3);
                ui.badge(MODELS[s.model], Tone::Accent);
                ui.spacer();
                if ui.icon_button("share").ghost().tooltip("Share").clicked() {
                    ui.toast(Toast::new("Link copied").description("Anyone with the link can read this chat."));
                }
            });
            ui.scroll(("thread", ci)).show(|ui| {
                if s.chats[ci].messages.is_empty() {
                    ui.column().fill_width().gap(12).pad((0.0, 40.0)).align(Align::Center).show(|ui| {
                        ui.heading("What are we working on?").level(2);
                        ui.label("Ask anything, or start from one of these.").muted();
                        ui.row().gap(8).show(|ui| {
                            for p in ["Plan a launch checklist", "Explain closed-form springs", "Draft a polite follow-up"] {
                                if ui.button(p).icon("sparkles").clicked() {
                                    s.prompt = p.to_string();
                                }
                            }
                        });
                    });
                } else {
                    ui.chat_thread(("chat", ci), &s.chats[ci].messages);
                }
            });
            let streaming = s.stream.is_some();
            let out = ui
                .prompt_composer("composer", &mut s.prompt, &mut s.model)
                .models(&MODELS)
                .attachments(&mut s.files)
                .placeholder("Message Lumi")
                .context_used(0.18 + 0.04 * s.chats[ci].messages.len() as f32)
                .streaming(streaming)
                .show();
            if out.attach_clicked {
                s.files.push(format!("notes-{}.md", s.files.len() + 1));
            }
            if out.stopped {
                s.stream = None;
            }
            if out.submitted && !streaming && !s.prompt.trim().is_empty() {
                let text = std::mem::take(&mut s.prompt);
                let chat = &mut s.chats[ci];
                if chat.messages.is_empty() {
                    chat.title = text.chars().take(32).collect();
                }
                chat.messages.push(ChatMessage::user(text).time("Now"));
                chat.messages.push(ChatMessage::assistant("Lumi", "").time("Now"));
                s.files.clear();
                s.stream = Some((ci, REPLY.to_string(), now));
            }
        });
    });
}
