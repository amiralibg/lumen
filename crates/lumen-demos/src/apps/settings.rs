//! Preferences: a settings app. A searchable sidebar of sections; each pane
//! slides in. Appearance changes the whole window live (theme, look,
//! accent, motion), and Do Not Disturb runs on two time wheels.

use lumen::{vec2, Align, Look, Mode, Time, Toast, Tone, Ui};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pane {
    General,
    Appearance,
    Notifications,
    Keyboard,
    Privacy,
    About,
}

const PANES: [(Pane, &str, &str, &str); 6] = [
    (Pane::General, "settings", "General", "language startup updates"),
    (Pane::Appearance, "palette", "Appearance", "theme dark light look accent motion text size"),
    (Pane::Notifications, "bell", "Notifications", "alerts sounds do not disturb schedule"),
    (Pane::Keyboard, "keyboard", "Keyboard", "shortcuts keys"),
    (Pane::Privacy, "shield", "Privacy", "analytics history crash reports"),
    (Pane::About, "monitor", "About", "version updates licenses"),
];

pub struct State {
    pane: Pane,
    search: String,
    pane_born: f64,
    language: u8,
    login: bool,
    updates: bool,
    beta: bool,
    /// 0 System, 1 Light, 2 Dark; None until read from the window.
    theme: Option<u8>,
    look: Option<u8>,
    text_size: f32,
    reduced: bool,
    notify: [bool; 4],
    sounds: bool,
    dnd: bool,
    dnd_from: Time,
    dnd_to: Time,
    analytics: bool,
    crash: bool,
    history: bool,
    confirm_clear: bool,
}

impl Default for State {
    fn default() -> Self {
        State {
            pane: Pane::Appearance,
            search: String::new(),
            pane_born: -10.0,
            language: 0,
            login: true,
            updates: true,
            beta: false,
            theme: None,
            look: None,
            text_size: 1.0,
            reduced: false,
            notify: [true, true, false, true],
            sounds: true,
            dnd: true,
            dnd_from: Time::new(22, 0),
            dnd_to: Time::new(7, 30),
            analytics: false,
            crash: true,
            history: true,
            confirm_clear: false,
        }
    }
}

pub fn view(ui: &mut Ui, s: &mut State) {
    // Start from the window's real theme and look.
    let look_now = match ui.look().platform {
        lumen::Platform::Lumen => 0u8,
        lumen::Platform::Mac => 1,
        lumen::Platform::Windows => 2,
        lumen::Platform::Gnome => 3,
    };
    s.theme.get_or_insert(0);
    s.look.get_or_insert(look_now);
    ui.row().fill().show(|ui| {
        sidebar(ui, s);
        ui.scroll("settings_pane").pad(28.0).gap(18.0).show(|ui| {
            // The pane fades and rises in when it changes.
            let t = (((ui.time() - s.pane_born) / 0.28).clamp(0.0, 1.0)) as f32;
            let e = if ui.reduced_motion() { 1.0 } else { lumen::Ease::Out.apply(t) };
            if e < 1.0 {
                ui.anims.keep_alive();
            }
            ui.column().fill_width().max_width(620.0).gap(18).layer(e, vec2(0.0, (1.0 - e) * 10.0)).show(|ui| pane(ui, s));
        });
    });
}

fn sidebar(ui: &mut Ui, s: &mut State) {
    ui.sidebar().width(220.0).show(|ui| {
        ui.nav_header("P", "Preferences");
        ui.column().pad((8.0, 4.0)).fill_width().show(|ui| {
            ui.search_field(&mut s.search).placeholder("Search settings").fill_width().show();
        });
        let q = s.search.trim().to_lowercase();
        let mut any = false;
        for (p, icon, title, words) in PANES {
            if !q.is_empty() && !title.to_lowercase().contains(&q) && !words.contains(&q) {
                continue;
            }
            any = true;
            if ui.nav_item(icon, title).selected(s.pane == p).clicked() && s.pane != p {
                s.pane = p;
                s.pane_born = ui.time();
            }
        }
        if !any {
            ui.column().pad((12.0, 8.0)).show(|ui| {
                ui.label("No settings match").muted().small();
            });
        }
    });
}

fn pane(ui: &mut Ui, s: &mut State) {
    let title = PANES.iter().find(|p| p.0 == s.pane).map(|p| p.2).unwrap_or("");
    ui.heading(title).level(2);
    match s.pane {
        Pane::General => {
            ui.card().show(|ui| {
                ui.settings(|rows| {
                    rows.row("Language", "Used for menus, dates and numbers.", |ui| {
                        ui.select(&mut s.language, &[(0u8, "English"), (1, "Deutsch"), (2, "Français"), (3, "日本語"), (4, "Português")]).width(180.0);
                    });
                    rows.row("Open at login", "Start in the menu bar when you sign in.", |ui| {
                        ui.switch("", &mut s.login);
                    });
                });
            });
            ui.card().title("Updates").show(|ui| {
                ui.settings(|rows| {
                    rows.row("Install updates automatically", "Restarts only when nothing is open.", |ui| {
                        ui.switch("", &mut s.updates);
                    });
                    rows.row("Beta releases", "Try new features a few weeks early.", |ui| {
                        ui.switch("", &mut s.beta);
                    });
                });
            });
        }
        Pane::Appearance => {
            ui.card().show(|ui| {
                ui.settings(|rows| {
                    rows.row("Theme", "System follows your OS.", |ui| {
                        let mut t = s.theme.unwrap_or(0);
                        if ui.segmented(&mut t, &[(0u8, "System"), (1, "Light"), (2, "Dark")]).changed {
                            s.theme = Some(t);
                            match t {
                                1 => ui.set_mode(Mode::Light),
                                2 => ui.set_mode(Mode::Dark),
                                _ => {}
                            }
                        }
                    });
                    rows.row("Look", "Controls, corners and motion of each platform.", |ui| {
                        let mut l = s.look.unwrap_or(0);
                        if ui.segmented(&mut l, &[(0u8, "Lumen"), (1, "macOS"), (2, "Windows"), (3, "GNOME")]).changed {
                            s.look = Some(l);
                            ui.set_look([Look::lumen(), Look::mac(), Look::windows(), Look::gnome()][l as usize].text_scale(s.text_size));
                        }
                    });
                    rows.row("Accent", "", |ui| {
                        ui.accent_picker();
                    });
                });
            });
            ui.card().title("Motion and text").show(|ui| {
                ui.settings(|rows| {
                    rows.row("Reduce motion", "Springs jump; fades stay short.", |ui| {
                        if ui.switch("", &mut s.reduced).changed() {
                            ui.set_reduced_motion(s.reduced);
                        }
                    });
                    rows.row("Text size", "", |ui| {
                        let r = format!("{:.0}%", s.text_size * 100.0);
                        if ui.slider("", &mut s.text_size, 0.85..=1.3).readout(r).width(200.0).changed() {
                            let look = *ui.theme.target_look();
                            ui.set_look(look.text_scale(s.text_size));
                        }
                    });
                });
            });
        }
        Pane::Notifications => {
            ui.card().show(|ui| {
                ui.settings(|rows| {
                    for (k, (t, d)) in [("Mentions", "When someone @-mentions you."), ("Deploys", "Started, finished and rolled back."), ("Weekly digest", "Mondays at 09:00."), ("Security alerts", "New sign-ins and expiring keys.")].iter().enumerate() {
                        rows.row(t, d, |ui| {
                            ui.scope(k, |ui| {
                                ui.switch("", &mut s.notify[k]);
                            });
                        });
                    }
                    rows.row("Sounds", "", |ui| {
                        ui.switch("", &mut s.sounds);
                    });
                });
            });
            ui.card().title("Do Not Disturb").description("Silence everything but security alerts on a schedule.").show(|ui| {
                ui.switch("Scheduled", &mut s.dnd);
                ui.row().gap(28).align(Align::Center).show(|ui| {
                    ui.column().gap(6).align(Align::Center).show(|ui| {
                        ui.label("From").muted().small();
                        ui.time_wheel("dnd_from", &mut s.dnd_from, 15);
                    });
                    ui.column().gap(6).align(Align::Center).show(|ui| {
                        ui.label("To").muted().small();
                        ui.time_wheel("dnd_to", &mut s.dnd_to, 15);
                    });
                    let mins = |t: Time| t.hour as i32 * 60 + t.minute as i32;
                    let span = (mins(s.dnd_to) - mins(s.dnd_from)).rem_euclid(24 * 60);
                    ui.column().gap(4).show(|ui| {
                        ui.label(format!("{}h {:02}m of quiet", span / 60, span % 60)).strong();
                        ui.label(format!("{} – {}", s.dnd_from.format_24(), s.dnd_to.format_24())).muted();
                        ui.badge(if s.dnd { "On" } else { "Off" }, if s.dnd { Tone::Accent } else { Tone::Neutral });
                    });
                });
            });
        }
        Pane::Keyboard => {
            ui.card().show(|ui| {
                ui.settings(|rows| {
                    for (t, keys) in [("Command palette", ["⌘", "K"]), ("Search settings", ["⌘", "F"]), ("New project", ["⌘", "N"]), ("Toggle sidebar", ["⌘", "\\"]), ("Next pane", ["⌃", "⇥"])] {
                        rows.row(t, "", |ui| {
                            ui.row().gap(4).show(|ui| {
                                for k in keys {
                                    ui.kbd(k);
                                }
                            });
                        });
                    }
                });
            });
        }
        Pane::Privacy => {
            ui.card().show(|ui| {
                ui.settings(|rows| {
                    rows.row("Usage analytics", "Anonymous counts that help us prioritize.", |ui| {
                        ui.switch("", &mut s.analytics);
                    });
                    rows.row("Crash reports", "Sent only when something breaks.", |ui| {
                        ui.switch("", &mut s.crash);
                    });
                    rows.row("Keep history", "Recent files and searches on this device.", |ui| {
                        ui.switch("", &mut s.history);
                    });
                });
            });
            ui.row().fill_width().gap(10).align(Align::Center).show(|ui| {
                ui.label("Clearing removes local history, caches and recent files.").muted();
                ui.spacer();
                if s.confirm_clear {
                    if ui.button("Cancel").ghost().clicked() {
                        s.confirm_clear = false;
                    }
                    if ui.button("Clear now").danger().clicked() {
                        s.confirm_clear = false;
                        ui.toast(Toast::new("History cleared").description("312 MB freed on this device.").success());
                    }
                } else if ui.button("Clear data…").clicked() {
                    s.confirm_clear = true;
                }
            });
        }
        Pane::About => {
            ui.card().show(|ui| {
                ui.row().gap(14).align(Align::Center).show(|ui| {
                    ui.avatar("L", 2, 48.0, None);
                    ui.column().gap(2).show(|ui| {
                        ui.label("Preferences").strong();
                        ui.label("Version 2.4.0 (build 8812) · lumen 0.1").muted();
                    });
                    ui.spacer();
                    if ui.button("Check for updates").clicked() {
                        ui.toast(Toast::new("You're up to date").description("2.4.0 is the newest version."));
                    }
                });
                ui.divider();
                ui.label("Built with lumen: Geist and Geist Mono (SIL OFL 1.1), Lucide icons (ISC).").muted().small();
            });
        }
    }
}
