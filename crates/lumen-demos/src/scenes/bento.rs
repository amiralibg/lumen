//! Tightened three-column Overview Bento composition.

use lumen::{Align, Date, Toast, Tone, Ui, UptimeStatus};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Region {
    Frankfurt,
    Virginia,
    Tokyo,
    Sydney,
}

pub struct State {
    pub os: u8,
    pub runner: String,
    pub region: Region,
    pub labels: [bool; 4],
    pub jobs: i64,
    pub on: [bool; 3],
    pub vol: f32,
    pub span: Option<(Date, Date)>,
    pub slot: u8,
}

impl Default for State {
    fn default() -> Self {
        let today = Date::today();
        Self {
            os: 0,
            runner: "linux-arm64-02".into(),
            region: Region::Frankfurt,
            labels: [true, true, false, false],
            jobs: 4,
            on: [true, true, false],
            vol: 0.68,
            span: Some((today.add_days(-3), today.add_days(4))),
            slot: 1,
        }
    }
}

pub fn view(ui: &mut Ui, s: &mut State) {
    let mut uptime = [UptimeStatus::Operational; 28];
    uptime[19] = UptimeStatus::Degraded;

    ui.row().fill().pad(20).gap(14).align(Align::Start).show(|ui| {
        // Column 1: Add a runner + Traffic
        ui.column().fill_width().gap(14).show(|ui| {
            ui.card().title("Add a runner").pad(16.0).show(|ui| {
                ui.segmented(&mut s.os, &[(0, "Linux"), (1, "macOS"), (2, "Windows")]);
                ui.text_input(&mut s.runner).label("Name").fill_width();
                ui.select(
                    &mut s.region,
                    &[
                        (Region::Frankfurt, "Frankfurt (eu-central)"),
                        (Region::Virginia, "Virginia (us-east)"),
                        (Region::Tokyo, "Tokyo (ap-northeast)"),
                        (Region::Sydney, "Sydney (ap-southeast)"),
                    ],
                )
                .label("Region")
                .fill_width();
                ui.column().fill_width().gap(6).show(|ui| {
                    ui.label("Labels").small().secondary();
                    ui.row().gap(6).show(|ui| {
                        for (i, tag) in ["arm64", "gpu", "docker", "nightly"].iter().enumerate() {
                            ui.chip(tag, &mut s.labels[i]);
                        }
                    });
                });
                ui.row().fill_width().align(Align::Center).show(|ui| {
                    ui.label("Concurrent jobs");
                    ui.spacer();
                    ui.stepper("jobs", &mut s.jobs, 1, 16);
                });
                ui.row().fill_width().gap(8).align(Align::Center).show(|ui| {
                    ui.spacer();
                    if ui.button("Cancel").ghost().clicked() {
                        s.runner = "linux-arm64-02".into();
                        s.labels = [true, true, false, false];
                        s.jobs = 4;
                    }
                    if ui.button("Add runner").primary().icon("plus").clicked() {
                        ui.toast(
                            Toast::new("Runner registered")
                                .description(format!("{} is online.", s.runner))
                                .success(),
                        );
                    }
                });
            });

            ui.card().title("Traffic").pad(16.0).show(|ui| {
                ui.donut_sized(
                    "bento-traffic",
                    &[
                        ("Search", 42.0),
                        ("Direct", 26.0),
                        ("Social", 17.0),
                        ("Other", 15.0),
                    ],
                    116.0,
                );
            });
        });

        // Column 2: Deploys + Quick settings + Reviewers
        ui.column().fill_width().gap(14).show(|ui| {
            ui.card().pad(16.0).show(|ui| {
                ui.stat(
                    "Deploys",
                    "1,284",
                    Some(12.4),
                    Some(&[9.0, 8.6, 9.4, 10.8, 10.0, 11.1, 11.3, 12.4, 12.6, 13.8, 15.1]),
                );
            });

            ui.card().title("Quick settings").pad(16.0).show(|ui| {
                ui.settings(|rows| {
                    rows.row("Wi-Fi", "Northwind-5G", |ui| {
                        ui.switch("", &mut s.on[0]);
                    });
                    rows.row("Bluetooth", "2 devices connected", |ui| {
                        ui.switch("", &mut s.on[1]);
                    });
                    rows.row("Focus", "Pause notifications", |ui| {
                        ui.switch("", &mut s.on[2]);
                    });
                });
                let v = format!("{:.0}%", s.vol * 100.0);
                ui.slider("Volume", &mut s.vol, 0.0..=1.0)
                    .readout(v)
                    .width(f32::INFINITY);
            });

            ui.card().title("Reviewers").pad(16.0).show(|ui| {
                for (ini, idx, name, role, online, status, tone) in [
                    ("JB", 0usize, "Jonas Berg", "Systems", Some(true), "Approved", Tone::Success),
                    ("LF", 3usize, "Lena Fischer", "Graphics", Some(true), "Approved", Tone::Success),
                    ("PN", 1usize, "Priya Nair", "Security", Some(false), "Pending", Tone::Warning),
                ] {
                    ui.row().fill_width().gap(10).align(Align::Center).show(|ui| {
                        ui.avatar(ini, idx, 32.0, online);
                        ui.column().fill_width().gap(1).show(|ui| {
                            ui.label(name).strong();
                            ui.label(role).small().muted();
                        });
                        ui.badge(status, tone);
                    });
                }
                ui.row().fill_width().align(Align::Center).show(|ui| {
                    ui.label("2 of 3 approvals").small().secondary();
                    ui.spacer();
                    ui.label("66%").small().muted();
                });
                ui.progress(Some(0.66));
            });
        });

        // Column 3: Active users + Release 2.4 + Edge health
        ui.column().fill_width().gap(14).show(|ui| {
            ui.card().pad(16.0).show(|ui| {
                ui.stat(
                    "Active users",
                    "8,214",
                    Some(4.1),
                    Some(&[3.0, 3.4, 3.2, 4.1, 4.0, 4.8, 5.2, 5.0, 6.1, 6.4, 7.2]),
                );
            });

            ui.card().pad(16.0).show(|ui| {
                ui.calendar_range(&mut s.span);
                ui.divider();
                ui.column().fill_width().gap(2).show(|ui| {
                    ui.label("Release 2.4").strong();
                    ui.label("Canary freeze & global rollout window").small().muted();
                });
                ui.toggle_group("slots", &mut s.slot, &[(0, "09:00"), (1, "13:30"), (2, "18:00")]);
            });

            ui.card().title("Edge health").pad(16.0).show(|ui| {
                ui.uptime_bars("bento_uptime", "Global PoPs · 99.98%", &uptime);
            });
        });
    });
}
