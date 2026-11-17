//! One live demo per lumen component.
//!
//! - Native: `cargo run -p lumen-demos -- chart-line [--dark] [--look mac]`
//! - Web: `demo.html?demo=chart-line&dark=1&look=mac&accent=blue`
//! - Screenshot: `cargo run -p lumen-demos -- chart-line --screenshot out.png`

use lumen::{Accent, Align, Color, Command, Date, Justify, Look, Mode, Placement, Platform, Series, Sides, Time, Toast, Tone, Ui};

/// Every demo: url name, title.
pub const DEMOS: &[(&str, &str)] = &[
    ("bento", "Overview bento"),
    ("hero", "Dashboard"),
    ("button", "Button"),
    ("switch", "Switch"),
    ("checkbox", "Checkbox"),
    ("radio", "Radio"),
    ("slider", "Slider"),
    ("text-input", "Text input"),
    ("text-area", "Text area"),
    ("number-field", "Number field"),
    ("tag-input", "Tag input"),
    ("otp-input", "One-time code"),
    ("password-input", "Password input"),
    ("search-field", "Search field"),
    ("select", "Select"),
    ("segmented", "Segmented control"),
    ("tabs", "Tabs"),
    ("menu", "Menu"),
    ("popover", "Popover"),
    ("dialog", "Dialog"),
    ("sheet", "Sheet"),
    ("command-palette", "Command palette"),
    ("hover-card", "Hover card"),
    ("context-menu", "Context menu"),
    ("confirm-popover", "Confirm popover"),
    ("toast", "Toast"),
    ("tooltip", "Tooltip"),
    ("chips", "Filter chips"),
    ("stepper", "Stepper"),
    ("toggle-group", "Toggle group"),
    ("avatar", "Avatar"),
    ("card", "Card and settings"),
    ("badge", "Badge and status"),
    ("progress", "Progress and spinner"),
    ("calendar", "Calendar"),
    ("date-picker", "Date picker"),
    ("combobox", "Combobox"),
    ("multi-select", "Multi-select"),
    ("date-range-picker", "Date range picker"),
    ("time-picker", "Time picker"),
    ("color-picker", "Color picker"),
    ("chart-line", "Line chart"),
    ("chart-area", "Area chart"),
    ("chart-bar", "Bar chart"),
    ("donut", "Donut chart"),
    ("gauge", "Gauge"),
    ("sparkline", "Sparkline and stats"),
    ("looks", "Looks"),
    ("vault", "Vault (password manager preview)"),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Region {
    Frankfurt,
    Virginia,
    Tokyo,
    Sydney,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Range {
    Day,
    Week,
    Month,
    Year,
}

struct State {
    demo: String,
    embed: bool,
    /// Screenshots: show overlays (menus, dialogs) open.
    force_open: bool,
    on: [bool; 4],
    checks: [bool; 4],
    plan: u8,
    vol: f32,
    price: f32,
    name: String,
    email: String,
    region: Region,
    range: Range,
    tab: u8,
    open: bool,
    date: Date,
    span: Option<(Date, Date)>,
    look: Platform,
    progress: f32,
    t: u32,
    search: String,
    os: u8,
    runner: String,
    labels: [bool; 5],
    jobs: i64,
    slot: u8,
    reduced: bool,
    tz: Option<usize>,
    reviewers: Vec<usize>,
    deploy_time: Time,
    quiet_end: Time,
    brand_color: Color,
    glow_color: Color,
    sheet_side: Placement,
    pinned: bool,
    revoked: bool,
    hover_shot: bool,
    release_notes: String,
    timeout_ms: f64,
    mem_gb: f64,
    budget_usd: f64,
    tags: Vec<String>,
    otp_code: String,
    new_pw: String,
    confirm_pw: String,
    deploy_query: String,
}

impl State {
    fn new(demo: String) -> State {
        let today = Date::today();
        State {
            demo,
            embed: false,
            force_open: false,
            on: [true, false, true, false],
            checks: [true, false, true, false],
            plan: 1,
            vol: 0.64,
            price: 40.0,
            name: "Ada Lovelace".into(),
            email: String::new(),
            region: Region::Frankfurt,
            range: Range::Week,
            tab: 0,
            open: false,
            date: today,
            span: Some((today.add_days(-3), today.add_days(4))),
            look: Platform::Lumen,
            progress: 0.42,
            t: 0,
            search: String::new(),
            os: 0,
            runner: "linux-arm64-02".into(),
            labels: [true, true, false, false, false],
            jobs: 4,
            slot: 1,
            reduced: false,
            tz: Some(2),
            reviewers: vec![0, 3],
            deploy_time: Time::new(14, 30),
            quiet_end: Time::new(8, 0),
            brand_color: Color::hex(0x0E6EE6),
            glow_color: Color::hex(0x7D55EC).with_alpha(0.72),
            sheet_side: Placement::Right,
            pinned: true,
            revoked: false,
            hover_shot: false,
            release_notes: "Added zero-copy glyph uploads and spring-driven sheet drawers.\nReduced idle frame wakeups to zero across all four looks.".into(),
            timeout_ms: 2500.0,
            mem_gb: 8.0,
            budget_usd: 240.0,
            tags: vec!["rust".into(), "wgpu".into(), "wasm".into(), "arm64".into(), "metal".into(), "linux".into()],
            otp_code: "482".into(),
            new_pw: "Northwind42".into(),
            confirm_pw: String::new(),
            deploy_query: String::new(),
        }
    }
}

const ZONES: [&str; 14] = [
    "Pacific/Honolulu", "America/Los_Angeles", "America/Denver", "America/Chicago", "America/New_York", "America/Sao_Paulo", "Europe/London",
    "Europe/Berlin", "Europe/Istanbul", "Asia/Dubai", "Asia/Kolkata", "Asia/Singapore", "Asia/Tokyo", "Australia/Sydney",
];
const PEOPLE: [&str; 8] = ["Jonas Berg", "Priya Nair", "Tomas Rivera", "Lena Fischer", "Sam Wu", "Dmitri Markov", "Aiko Kato", "Noor Haddad"];

const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

fn revenue() -> Vec<f32> {
    vec![42.0, 48.0, 45.0, 61.0, 58.0, 72.0, 69.0, 84.0, 80.0, 96.0, 104.0, 118.0]
}
fn costs() -> Vec<f32> {
    vec![30.0, 33.0, 35.0, 38.0, 41.0, 40.0, 46.0, 49.0, 52.0, 55.0, 58.0, 61.0]
}

fn frame(ui: &mut Ui, f: impl FnOnce(&mut Ui)) {
    frame_at(ui, Justify::Center, f)
}

/// Demos whose overlays open downward start near the top.
fn frame_at(ui: &mut Ui, justify: Justify, f: impl FnOnce(&mut Ui)) {
    ui.column().fill().pad(40).align(Align::Center).justify(justify).show(|ui| {
        ui.column().fill_width().max_width(760.0).gap(16).align(Align::Center).show(f);
    });
}

fn view(ui: &mut Ui, s: &mut State) {
    if s.reduced {
        ui.set_reduced_motion(true);
    }
    let demo = s.demo.clone();
    // Embedded component demos sit on the host page's white card.
    if s.embed && !matches!(demo.as_str(), "hero" | "vault" | "bento") {
        for m in [Mode::Light, Mode::Dark] {
            let surf = ui.theme.target_look().neutrals(m).surface;
            ui.override_color(m, lumen::Token::Background, Some(surf));
        }
    }
    match demo.as_str() {
        "hero" => hero(ui, s),
        "bento" => bento(ui, s),
        "looks" => looks(ui, s),
        "vault" => vault(ui, s),
        "menu"
        | "popover"
        | "select"
        | "date-picker"
        | "stepper"
        | "combobox"
        | "multi-select"
        | "date-range-picker"
        | "time-picker"
        | "color-picker"
        | "hover-card"
        | "context-menu"
        | "confirm-popover" => frame_at(ui, Justify::Start, |ui| component(ui, s, &demo)),
        _ => frame(ui, |ui| component(ui, s, &demo)),
    }
}

fn component(ui: &mut Ui, s: &mut State, demo: &str) {
    match demo {
        "button" => {
            ui.row().gap(10).align(Align::Center).show(|ui| {
                ui.button("Deploy").primary().icon("rocket");
                ui.button("Preview").icon("eye");
                ui.button("Cancel").ghost();
                ui.button("Delete").danger().icon("trash-2");
                ui.icon_button("copy").tooltip_shortcut("Copy", "⌘C");
            });
            ui.row().gap(10).align(Align::Center).show(|ui| {
                ui.button("Small").small();
                ui.button("Medium");
                ui.button("Large").large().primary();
                ui.button("Disabled").disabled(true);
            });
        }
        "switch" => {
            ui.column().gap(14).show(|ui| {
                ui.switch("Wi-Fi", &mut s.on[0]);
                ui.switch("Bluetooth", &mut s.on[1]);
                ui.switch("AirDrop", &mut s.on[2]);
                ui.switch("Hotspot (managed)", &mut s.on[3]).disabled(true);
            });
        }
        "checkbox" => {
            ui.column().gap(12).show(|ui| {
                let all = s.checks.iter().all(|c| *c);
                let none = s.checks.iter().all(|c| !*c);
                let mut a = all;
                if ui.checkbox("Select all", &mut a).indeterminate(!all && !none).changed() {
                    let v = !all;
                    s.checks = [v; 4];
                }
                ui.column().gap(10).pad(Sides { left: 24.0, ..Default::default() }).show(|ui| {
                    ui.checkbox("Email", &mut s.checks[0]).description("Digests and receipts");
                    ui.checkbox("Push", &mut s.checks[1]);
                    ui.checkbox("SMS", &mut s.checks[2]);
                    ui.checkbox("Pager", &mut s.checks[3]);
                });
            });
        }
        "radio" => {
            ui.column().gap(12).show(|ui| {
                ui.radio(&mut s.plan, 0, "Hobby — free");
                ui.radio(&mut s.plan, 1, "Pro — $20 / month");
                ui.radio(&mut s.plan, 2, "Team — $80 / month");
            });
        }
        "slider" => {
            ui.column().gap(20).show(|ui| {
                let v = format!("{:.0}%", s.vol * 100.0);
                ui.slider("Volume", &mut s.vol, 0.0..=1.0).readout(v).width(320.0);
                let p = format!("${:.0}", s.price);
                ui.slider("Max price", &mut s.price, 0.0..=100.0).step(5.0).readout(p).width(320.0);
            });
        }
        "text-input" => {
            ui.column().gap(16).fill_width().max_width(360.0).show(|ui| {
                ui.text_input(&mut s.name).label("Full name").fill_width();
                let err = (!s.email.is_empty() && !s.email.contains('@')).then_some("Enter a valid email address.");
                ui.text_input(&mut s.email).label("Email").icon("mail").placeholder("you@example.com").error(err).helper("We never share it.").fill_width();
                ui.text_input(&mut s.search).icon("search").placeholder("Search…").fill_width();
            });
        }
        "text-area" => {
            ui.column().gap(12).fill_width().max_width(420.0).show(|ui| {
                ui.text_area(&mut s.release_notes)
                    .label("Release notes")
                    .placeholder("Summarize what shipped in this tag…")
                    .rows(3, 6)
                    .max_chars(240)
                    .helper("Shown on the public changelog.")
                    .fill_width();
                ui.row().fill_width().gap(8).align(Align::Center).show(|ui| {
                    ui.spacer();
                    if ui.button("Clear").ghost().small().disabled(s.release_notes.is_empty()).clicked() {
                        s.release_notes.clear();
                    }
                    if ui.button("Publish notes").primary().small().disabled(s.release_notes.trim().is_empty()).clicked() {
                        ui.toast(Toast::new("Release notes published").success());
                    }
                });
            });
        }
        "number-field" => {
            ui.column().gap(14).fill_width().max_width(420.0).show(|ui| {
                ui.row().fill_width().gap(12).show(|ui| {
                    ui.number_field(&mut s.timeout_ms).label("Timeout").range(100.0..=30000.0).step(50.0).suffix("ms").fill_width();
                    ui.number_field(&mut s.mem_gb).label("Memory").range(0.5..=64.0).step(0.5).decimals(1).suffix("GB").fill_width();
                });
                ui.number_field(&mut s.budget_usd)
                    .label("Monthly spend cap")
                    .range(0.0..=5000.0)
                    .step(10.0)
                    .prefix("$")
                    .helper("Drag any label or handle left/right to scrub · Shift 10× · Alt 0.1×")
                    .fill_width();
            });
        }
        "tag-input" => {
            ui.column().gap(12).fill_width().max_width(400.0).show(|ui| {
                ui.tag_input(&mut s.tags)
                    .label("Runner labels")
                    .placeholder("Add label…")
                    .lowercase(true)
                    .max_tags(8)
                    .helper("Press Enter or comma to add · Backspace twice removes the last tag.")
                    .fill_width();
            });
        }
        "otp-input" => {
            ui.column().gap(12).align(Align::Center).show(|ui| {
                ui.column().gap(2).align(Align::Center).show(|ui| {
                    ui.label("Verify hardware key").strong();
                    ui.label("Enter the 6-digit code from your authenticator.").small().secondary();
                });
                let bad = (s.otp_code == "000000").then_some("Invalid verification code. Try any other 6 digits.");
                if ui.otp_input("2fa", &mut s.otp_code).error(bad).completed() && s.otp_code != "000000" {
                    ui.toast(Toast::new("Session verified").success());
                }
                if s.otp_code.len() == 6 && s.otp_code != "000000" {
                    ui.badge("Verified", Tone::Success);
                } else {
                    ui.label("Tip: paste “482-913” to fill every cell at once.").small().muted();
                }
            });
        }
        "password-input" => {
            ui.column().gap(14).fill_width().max_width(380.0).show(|ui| {
                ui.password_input(&mut s.new_pw)
                    .label("Master password")
                    .icon("lock")
                    .placeholder("Create a strong passphrase")
                    .strength(true)
                    .copyable(true)
                    .fill_width();
                let mismatch = (!s.confirm_pw.is_empty() && s.confirm_pw != s.new_pw).then_some("Passwords do not match.");
                ui.password_input(&mut s.confirm_pw)
                    .label("Confirm password")
                    .placeholder("Re-enter master password")
                    .error(mismatch)
                    .fill_width();
            });
        }
        "search-field" => {
            const DEPLOYS: [(&str, &str, Tone); 5] = [
                ("v2.4.1 · Frankfurt", "2m ago", Tone::Success),
                ("v2.4.0 · Virginia", "1h ago", Tone::Success),
                ("v2.4.0-rc2 · Tokyo", "4h ago", Tone::Info),
                ("canary-882 · Sydney", "6h ago", Tone::Warning),
                ("v2.3.9 · London", "Yesterday", Tone::Neutral),
            ];
            let q = s.deploy_query.trim().to_lowercase();
            let hits: Vec<_> = DEPLOYS.iter().copied().filter(|(name, _, _)| q.is_empty() || name.to_lowercase().contains(&q)).collect();
            ui.column().gap(10).fill_width().max_width(400.0).show(|ui| {
                ui.search_field(&mut s.deploy_query)
                    .placeholder("Filter deployments…")
                    .shortcut("/")
                    .focus_key('/')
                    .results(Some(hits.len()))
                    .fill_width();
                ui.card().pad(6.0).show(|ui| {
                    if hits.is_empty() {
                        ui.row().fill_width().pad(10.0).justify(Justify::Center).show(|ui| {
                            ui.label(format!("No deployments matching “{}”.", s.deploy_query.trim())).small().muted();
                        });
                    } else {
                        ui.column().fill_width().gap(2).show(|ui| {
                            for (name, when, tone) in hits.into_iter().take(3) {
                                ui.row().fill_width().pad(Sides::xy(10.0, 6.0)).gap(10).align(Align::Center).show(|ui| {
                                    ui.status(name, tone);
                                    ui.spacer();
                                    ui.label(when).small().muted();
                                });
                            }
                        });
                    }
                });
            });
        }
        "select" => {
            ui.column().gap(16).fill_width().max_width(280.0).show(|ui| {
                ui.select(&mut s.region, &[(Region::Frankfurt, "Frankfurt (eu-central)"), (Region::Virginia, "Virginia (us-east)"), (Region::Tokyo, "Tokyo (ap-northeast)"), (Region::Sydney, "Sydney (ap-southeast)")])
                    .label("Region")
                    .fill_width();
            });
        }
        "segmented" => {
            ui.segmented(&mut s.range, &[(Range::Day, "Day"), (Range::Week, "Week"), (Range::Month, "Month"), (Range::Year, "Year")]);
        }
        "tabs" => {
            ui.column().fill_width().max_width(480.0).gap(16).show(|ui| {
                ui.tabs(&mut s.tab, &[(0, "Overview"), (1, "Activity"), (2, "Settings"), (3, "Billing")]);
                let text = ["Everything at a glance.", "What changed, and who did it.", "Preferences for this project.", "Invoices and payment methods."][s.tab as usize];
                ui.label(text).secondary();
            });
        }
        "menu" => {
            let r = ui.button("Actions").trailing_icon("chevron-down").show();
            if std::mem::take(&mut s.force_open) {
                ui.set_popover_open(r.id, true);
            }
            ui.menu(&r, |ui| {
                ui.menu_label("Project");
                ui.menu_item("pencil", "Rename").shortcut("⌘R");
                ui.menu_item("copy", "Duplicate").shortcut("⌘D");
                ui.menu_item("share-2", "Share…");
                ui.menu_separator();
                ui.menu_item("archive", "Archive");
                ui.menu_item("trash-2", "Delete").danger().shortcut("⌫");
            });
            ui.label("Click the button, or Tab to it and press Enter.").muted();
        }
        "popover" => {
            let r = ui.button("Share").icon("share-2").show();
            ui.popover(&r).pad(16.0).width(300.0).show(|ui| {
                ui.column().gap(10).fill_width().show(|ui| {
                    ui.label("Share this board").strong();
                    ui.label("Anyone with the link can view.").secondary();
                    ui.row().gap(8).fill_width().show(|ui| {
                        let mut link = "lumen.app/b/8f3k".to_string();
                        ui.text_input(&mut link).fill_width().id("link");
                        if ui.button("Copy").primary().clicked() {
                            ui.close_layer();
                        }
                    });
                });
            });
        }
        "dialog" => {
            if ui.button("Delete project").danger().icon("trash-2").clicked() || std::mem::take(&mut s.force_open) {
                s.open = true;
            }
            ui.dialog("confirm", &mut s.open)
                .title("Delete “Northwind”?")
                .description("This removes the project, its 14 deploys and all logs. You can't undo this.")
                .footer(|ui| {
                    if ui.button("Cancel").ghost().clicked() {
                        ui.close_layer();
                    }
                    if ui.button("Delete").danger().clicked() {
                        ui.close_layer();
                    }
                })
                .show(|_| {});
        }
        "sheet" => {
            if std::mem::take(&mut s.force_open) {
                s.open = true;
            }
            ui.row().gap(12).align(Align::Center).show(|ui| {
                ui.segmented(&mut s.sheet_side, &[(Placement::Right, "Right"), (Placement::Left, "Left"), (Placement::Bottom, "Bottom")]);
                if ui.button("Inspect runner").primary().icon("panel-right").clicked() {
                    s.open = true;
                }
            });
            ui.sheet("runner", &mut s.open)
                .side(s.sheet_side)
                .title("linux-arm64-02")
                .description("Frankfurt · online · 4 concurrent jobs")
                .footer(|ui| {
                    if ui.button("Cancel").ghost().clicked() {
                        ui.close_layer();
                    }
                    if ui.button("Save changes").primary().clicked() {
                        ui.toast(Toast::new("Runner updated").success());
                        ui.close_layer();
                    }
                })
                .show(|ui| {
                    ui.text_input(&mut s.runner).label("Hostname").fill_width();
                    ui.select(&mut s.region, &[(Region::Frankfurt, "Frankfurt (eu-central)"), (Region::Virginia, "Virginia (us-east)"), (Region::Tokyo, "Tokyo (ap-northeast)"), (Region::Sydney, "Sydney (ap-southeast)")])
                        .label("Region")
                        .fill_width();
                    ui.settings(|rows| {
                        rows.row("Auto-drain before reboot", "Finish running jobs before restarting.", |ui| {
                            ui.switch("", &mut s.on[0]);
                        });
                        rows.row("Spot fallback", "Allow burst capacity on spot instances.", |ui| {
                            ui.switch("", &mut s.on[2]);
                        });
                    });
                });
        }
        "command-palette" => {
            const CMDS: &[Command<'static>] = &[
                Command::new("deploy", "Deploy production").group("Actions").icon("rocket").shortcut("⌘D").keywords("ship release"),
                Command::new("logs", "Tail live logs").group("Actions").icon("terminal").shortcut("⌘L").keywords("console output"),
                Command::new("branch", "Switch branch…").group("Actions").icon("git-branch").shortcut("⌘B"),
                Command::new("runners", "Go to Runners").group("Navigation").icon("users").shortcut("G R"),
                Command::new("settings", "Workspace settings").group("Navigation").icon("settings").shortcut("⌘,"),
                Command::new("theme", "Toggle dark mode").group("Appearance").icon("moon").shortcut("⌘T"),
            ];
            if std::mem::take(&mut s.force_open) {
                s.open = true;
            }
            ui.column().gap(10).align(Align::Center).show(|ui| {
                if ui.button("Open command palette").icon("search").clicked() {
                    s.open = true;
                }
                ui.label("Or press ⌘K / Ctrl+K anywhere in this demo.").muted();
            });
            if let Some(cmd) = ui.command_palette("main", &mut s.open, CMDS).show() {
                ui.toast(Toast::new(format!("Triggered “{cmd}”")).tone(Tone::Accent));
            }
        }
        "hover-card" => {
            if std::mem::take(&mut s.force_open) {
                s.hover_shot = true;
            }
            let people = [
                ("@jonas", "JB", "Jonas Berg", "Staff Systems · Frankfurt", "42 deploys this month", Some(true)),
                ("@priya", "PN", "Priya Nair", "Security Lead · London", "19 reviews this week", Some(false)),
                ("@lena", "LF", "Lena Fischer", "Runtime · Berlin", "Maintains lumen-wgpu", Some(true)),
            ];
            ui.row().gap(8).align(Align::Center).show(|ui| {
                ui.label("Approved by").secondary();
                for (i, (handle, ini, name, role, note, pres)) in people.iter().enumerate() {
                    let r = ui.button(*handle).ghost().small().show();
                    ui.hover_card(&r).width(276.0).open(s.hover_shot && i == 0).show(|ui| {
                        ui.row().gap(12).align(Align::Center).show(|ui| {
                            ui.avatar(ini, i, 38.0, *pres);
                            ui.column().gap(2).fill_width().show(|ui| {
                                ui.label(*name).strong();
                                ui.label(*role).small().secondary();
                            });
                        });
                        ui.divider();
                        ui.label(*note).small().muted();
                    });
                }
            });
        }
        "context-menu" => {
            let zone = ui.column().fill_width().max_width(440.0).id("release-row").show(|ui| {
                ui.card().pad(16.0).show(|ui| {
                    ui.row().fill_width().gap(12).align(Align::Center).show(|ui| {
                        ui.column().gap(2).fill_width().show(|ui| {
                            ui.label("v2.4.1 — Production").strong();
                            ui.label("Right-click this card (or focus and press Shift+Enter).").small().muted();
                        });
                        if s.pinned {
                            ui.badge("Pinned", Tone::Accent);
                        }
                        ui.badge("Live", Tone::Success);
                    });
                });
            });
            let resp = zone.response();
            if std::mem::take(&mut s.force_open) {
                ui.open_context_menu(resp.id, lumen::vec2(260.0, 126.0));
            }
            ui.context_menu(&resp, |ui| {
                ui.menu_label("Release v2.4.1");
                ui.menu_item("file-text", "View build log").shortcut("⌘L");
                ui.menu_item("copy", "Copy commit SHA").shortcut("⌘C");
                if ui.menu_item("pin", "Pin to overview").checked(s.pinned).clicked() {
                    s.pinned = !s.pinned;
                }
                ui.menu_separator();
                if ui.menu_item("rotate-ccw", "Roll back release…").danger().clicked() {
                    ui.toast(Toast::new("Rollback queued").tone(Tone::Warning));
                }
            });
        }
        "confirm-popover" => {
            ui.column().fill_width().max_width(440.0).show(|ui| {
                ui.card().pad(16.0).show(|ui| {
                    ui.row().fill_width().gap(12).align(Align::Center).show(|ui| {
                        ui.column().gap(2).fill_width().show(|ui| {
                            ui.label("Production API key").strong();
                            ui.label(if s.revoked { "Revoked · create a new key to restore access" } else { "rk_live_9d2f…8a1c · last used 2m ago" }).small().muted();
                        });
                        let r = ui.button(if s.revoked { "Revoked" } else { "Revoke" }).danger().small().disabled(s.revoked).show();
                        if std::mem::take(&mut s.force_open) {
                            ui.set_popover_open(r.id, true);
                        }
                        if ui
                            .confirm_popover(&r, "Revoke this API key?")
                            .description("Clients authenticating with rk_live_9d2f will be rejected immediately.")
                            .confirm_label("Revoke key")
                            .confirmed()
                        {
                            s.revoked = true;
                            ui.toast(Toast::new("API key revoked").danger());
                        }
                    });
                });
            });
        }
        "toast" => {
            ui.row().gap(10).show(|ui| {
                if ui.button("Saved").icon("check").clicked() {
                    ui.toast(Toast::new("Changes saved").description("Your profile is up to date.").success());
                }
                if ui.button("Failed").icon("x").clicked() {
                    ui.toast(Toast::new("Deploy failed").description("Build step exited with code 1.").danger());
                }
                if ui.button("Info").icon("info").clicked() {
                    ui.toast(Toast::new("New version available").tone(Tone::Info));
                }
            });
        }
        "chips" => {
            ui.column().gap(12).align(Align::Center).show(|ui| {
                ui.row().gap(8).show(|ui| {
                    for (i, l) in ["arm64", "gpu", "docker", "nightly", "macos-14"].iter().enumerate() {
                        ui.chip(l, &mut s.labels[i]);
                    }
                });
                let n = s.labels.iter().filter(|x| **x).count();
                ui.label(format!("{n} of 5 labels: runners must match all of them.")).muted();
            });
        }
        "stepper" => {
            ui.column().gap(16).fill_width().max_width(320.0).show(|ui| {
                ui.row().fill_width().align(Align::Center).show(|ui| {
                    ui.label("Concurrent jobs");
                    ui.spacer();
                    ui.stepper("jobs", &mut s.jobs, 1, 16);
                });
                ui.label("Click − and +, or focus a button and use the arrow keys. The number rolls the way it moved.").muted();
            });
        }
        "toggle-group" => {
            ui.column().gap(12).fill_width().max_width(360.0).show(|ui| {
                ui.label("Release window").strong();
                ui.toggle_group("slots", &mut s.slot, &[(0, "09:00"), (1, "11:30"), (2, "14:00"), (3, "16:30")]);
            });
        }
        "avatar" => {
            ui.column().gap(18).align(Align::Center).show(|ui| {
                ui.row().gap(12).align(Align::Center).show(|ui| {
                    ui.avatar("JB", 0, 40.0, Some(true));
                    ui.avatar("PN", 1, 40.0, Some(false));
                    ui.avatar("TR", 2, 40.0, None);
                    ui.avatar("LF", 3, 32.0, None);
                    ui.avatar("SW", 4, 24.0, None);
                });
                ui.avatar_group(&["JB", "PN", "TR", "LF", "SW", "DM", "AK"], 4, 32.0);
            });
        }
        "tooltip" => {
            ui.row().gap(8).show(|ui| {
                ui.icon_button("bold").ghost().tooltip_shortcut("Bold", "⌘B");
                ui.icon_button("italic").ghost().tooltip_shortcut("Italic", "⌘I");
                ui.icon_button("underline").ghost().tooltip_shortcut("Underline", "⌘U");
                ui.icon_button("link").ghost().tooltip("Insert link");
            });
            ui.label("Hover an icon. Tooltips open after a delay, then instantly while you move along.").muted();
        }
        "card" => {
            ui.column().fill_width().max_width(560.0).show(|ui| {
                ui.card().title("Notifications").description("Choose what reaches you.").show(|ui| {
                    ui.settings(|rows| {
                        rows.row("Deploy failures", "When a production deploy fails.", |ui| {
                            ui.switch("", &mut s.on[0]);
                        });
                        rows.row("Weekly digest", "A Monday summary.", |ui| {
                            ui.switch("", &mut s.on[1]);
                        });
                    });
                });
            });
        }
        "badge" => {
            ui.row().gap(8).show(|ui| {
                ui.badge("Neutral", Tone::Neutral);
                ui.badge("Accent", Tone::Accent);
                ui.badge("Live", Tone::Success);
                ui.badge("Degraded", Tone::Warning);
                ui.badge("Down", Tone::Danger);
                ui.badge("Beta", Tone::Info);
            });
            ui.row().gap(20).show(|ui| {
                ui.status("Operational", Tone::Success);
                ui.status("Partial outage", Tone::Warning);
                ui.status("Major outage", Tone::Danger);
            });
        }
        "progress" => {
            ui.column().fill_width().max_width(360.0).gap(18).show(|ui| {
                s.t += 1;
                if s.t % 90 == 0 {
                    s.progress = if s.progress > 0.9 { 0.1 } else { s.progress + 0.2 };
                }
                ui.request_repaint_after(0.05);
                ui.label(format!("Uploading… {:.0}%", s.progress * 100.0)).small().secondary();
                ui.progress(Some(s.progress));
                ui.label("Indeterminate").small().secondary();
                ui.scope("indet", |ui| ui.progress(None));
                ui.row().gap(16).align(Align::Center).show(|ui| {
                    ui.spinner(20.0);
                    ui.label("Syncing").secondary();
                });
            });
        }
        "calendar" => {
            ui.row().gap(32).align(Align::Start).show(|ui| {
                ui.calendar(&mut s.date);
                ui.calendar_range(&mut s.span);
            });
        }
        "date-picker" => {
            ui.row().gap(16).align(Align::Start).show(|ui| {
                let r = ui.date_picker("Due date", &mut s.date);
                if std::mem::take(&mut s.force_open) {
                    ui.set_popover_open(r.id, true);
                }
                ui.date_range_picker("Stay", &mut s.span);
            });
        }
        "combobox" => {
            ui.column().gap(12).fill_width().max_width(320.0).show(|ui| {
                let opts: Vec<(usize, &str)> = ZONES.iter().copied().enumerate().collect();
                ui.combobox(&mut s.tz, &opts).label("Time zone").placeholder("Search time zones").fill_width();
                let now = s.tz.map(|i| ZONES[i]).unwrap_or("none");
                ui.label(format!("Reports run at 09:00 in {now}.")).muted();
            });
        }
        "multi-select" => {
            ui.column().gap(12).fill_width().max_width(380.0).show(|ui| {
                let opts: Vec<(usize, &str)> = PEOPLE.iter().copied().enumerate().collect();
                ui.multi_select(&mut s.reviewers, &opts).label("Reviewers").placeholder("Add reviewers").fill_width();
                ui.label(format!("{} of 8 required to approve.", s.reviewers.len().min(2))).muted();
            });
        }
        "date-range-picker" => {
            ui.column().gap(12).align(Align::Start).show(|ui| {
                let r = ui.date_range_picker("Maintenance window", &mut s.span);
                if std::mem::take(&mut s.force_open) {
                    ui.set_popover_open(r.id, true);
                }
                let msg = match s.span {
                    Some((a, b)) if a != b => format!("{} nights scheduled · status page updates automatically.", b.to_days() - a.to_days()),
                    Some((a, _)) => format!("Single day: {}.", a.short()),
                    None => "No window scheduled.".into(),
                };
                ui.label(msg).muted();
            });
        }
        "time-picker" => {
            ui.row().gap(16).align(Align::Start).show(|ui| {
                let r = ui.time_picker("Deploy at (UTC)", &mut s.deploy_time).show();
                if std::mem::take(&mut s.force_open) {
                    ui.set_popover_open(r.id, true);
                }
                ui.time_picker("Quiet hours end", &mut s.quiet_end).twelve_hour(true).step(15);
            });
        }
        "color-picker" => {
            ui.row().gap(16).align(Align::Start).show(|ui| {
                let r = ui.color_picker("Brand color", &mut s.brand_color).show();
                if std::mem::take(&mut s.force_open) {
                    ui.set_popover_open(r.id, true);
                }
                ui.color_picker("Spotlight glow", &mut s.glow_color).alpha(true);
            });
        }
        "chart-line" => {
            ui.line_chart("rev", &MONTHS, vec![Series::new("Revenue", revenue()), Series::new("Costs", costs())]).smooth(true).height(260.0).format(|v| if v == 0.0 { "$0".into() } else { format!("${}k", lumen::widgets::chart::compact(v)) });
        }
        "chart-area" => {
            let v: Vec<f32> = (0..30).map(|i| 40.0 + 18.0 * ((i as f32) * 0.45).sin() + i as f32 * 1.6).collect();
            let labels: Vec<String> = (1..=30).map(|d| format!("Nov {d}")).collect();
            let l: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
            ui.area_chart("visitors", &l, vec![Series::new("Visitors", v)]).smooth(true).height(260.0);
        }
        "chart-bar" => {
            ui.bar_chart(
                "week",
                &DAYS,
                vec![Series::new("Web", vec![12.0, 18.0, 15.0, 22.0, 26.0, 9.0, 7.0]), Series::new("iOS", vec![8.0, 10.0, 9.0, 14.0, 15.0, 11.0, 10.0]), Series::new("Android", vec![5.0, 7.0, 8.0, 9.0, 11.0, 6.0, 5.0])],
            )
            .stacked()
            .height(260.0);
        }
        "donut" => {
            ui.donut("traffic", &[("Search", 4200.0), ("Direct", 2600.0), ("Social", 1500.0), ("Referral", 900.0), ("Email", 600.0)]);
        }
        "gauge" => {
            ui.row().gap(24).show(|ui| {
                ui.gauge("cpu", 0.38, "CPU");
                ui.gauge("mem", 0.71, "Memory");
                ui.gauge("disk", 0.92, "Disk");
            });
        }
        "sparkline" => {
            ui.row().fill_width().gap(16).show(|ui| {
                for (label, value, delta, trend) in [
                    ("Revenue", "$118k", 12.4, revenue()),
                    ("Active users", "8,214", 4.1, vec![60.0, 62.0, 61.0, 66.0, 70.0, 69.0, 74.0, 78.0]),
                    ("Churn", "2.1%", -0.6, vec![3.0, 2.8, 2.9, 2.6, 2.5, 2.3, 2.2, 2.1]),
                ] {
                    ui.card().show(|ui| {
                        ui.stat(label, value, Some(delta), Some(&trend));
                    });
                }
            });
        }
        _ => {
            ui.label(format!("Unknown demo “{demo}”.")).secondary();
        }
    }
}

/// The overview bento on the site: one scene of everyday components, all of
/// them live. Mirrors the Ethereal concept in site/concepts.
fn bento(ui: &mut Ui, s: &mut State) {
    ui.row().fill().pad(24).gap(16).align(Align::Start).show(|ui| {
        ui.column().fill_width().gap(16).show(|ui| {
            ui.card().title("Add a runner").description("Runners pick up jobs for the platforms you choose.").show(|ui| {
                ui.segmented(&mut s.os, &[(0, "Linux"), (1, "macOS"), (2, "Windows")]);
                ui.text_input(&mut s.runner).label("Name").fill_width();
                ui.select(&mut s.region, &[(Region::Frankfurt, "Frankfurt"), (Region::Virginia, "Virginia"), (Region::Tokyo, "Tokyo"), (Region::Sydney, "Sydney")]).label("Region").fill_width();
                ui.column().gap(6).show(|ui| {
                    ui.label("Labels").strong();
                    ui.row().gap(6).show(|ui| {
                        for (i, l) in ["arm64", "gpu", "docker", "nightly"].iter().enumerate() {
                            ui.chip(l, &mut s.labels[i]);
                        }
                    });
                });
                ui.row().fill_width().align(Align::Center).show(|ui| {
                    ui.label("Concurrent jobs").strong();
                    ui.spacer();
                    ui.stepper("jobs", &mut s.jobs, 1, 16);
                });
                ui.row().fill_width().gap(8).show(|ui| {
                    ui.spacer();
                    ui.button("Cancel").ghost();
                    if ui.button("Add runner").primary().clicked() {
                        let d = format!("{} is online and taking jobs.", s.runner);
                        ui.toast(Toast::new("Runner added").description(d).success());
                    }
                });
            });
            ui.card().title("Traffic").show(|ui| {
                ui.donut_sized("bento-traffic", &[("Search", 42.0), ("Direct", 26.0), ("Social", 17.0), ("Other", 15.0)], 120.0);
            });
        });
        ui.column().fill_width().gap(16).show(|ui| {
            ui.card().show(|ui| ui.stat("Deploys", "1,284", Some(12.4), Some(&[9.0, 8.6, 9.4, 10.8, 10.0, 11.1, 11.3, 12.4, 12.6, 13.8, 15.1])));
            ui.card().title("Quick settings").show(|ui| {
                ui.settings(|rows| {
                    rows.row("Wi-Fi", "", |ui| {
                        ui.switch("", &mut s.on[0]);
                    });
                    rows.row("Bluetooth", "", |ui| {
                        ui.switch("", &mut s.on[1]);
                    });
                    rows.row("Focus", "", |ui| {
                        ui.switch("", &mut s.on[2]);
                    });
                });
                let v = format!("{:.0}%", s.vol * 100.0);
                ui.slider("Volume", &mut s.vol, 0.0..=1.0).readout(v).width(f32::INFINITY);
            });
            ui.card().title("Reviewers").show(|ui| {
                for (i, (ini, name, mail, req)) in [("JB", "Jonas Berg", "jonas@northlight.io", "Required"), ("PN", "Priya Nair", "priya@harbor.dev", "Required"), ("TR", "Tomas Rivera", "tomas@kiln.app", "Optional")].iter().enumerate() {
                    ui.row().fill_width().gap(12).align(Align::Center).show(|ui| {
                        ui.avatar(ini, i, 34.0, [Some(true), Some(false), None][i]);
                        ui.column().gap(1).fill_width().show(|ui| {
                            ui.label(*name).strong();
                            ui.label(*mail).muted();
                        });
                        ui.badge(*req, if i < 2 { Tone::Accent } else { Tone::Neutral });
                    });
                }
                ui.progress(Some(0.66));
            });
        });
        ui.column().fill_width().gap(16).show(|ui| {
            ui.card().show(|ui| ui.stat("Active users", "8,214", Some(4.1), Some(&[3.0, 3.4, 3.2, 4.1, 4.0, 4.8, 5.2, 5.0, 6.1, 6.4, 7.2])));
            ui.card().show(|ui| {
                ui.calendar_range(&mut s.span);
                ui.divider();
                ui.column().gap(2).show(|ui| {
                    ui.label("Release 2.4").strong();
                    ui.label("Pick a window while the freeze holds.").muted();
                });
                ui.toggle_group("slots", &mut s.slot, &[(0, "09:00"), (1, "11:30"), (2, "14:00"), (3, "16:30")]);
            });
        });
    });
}

/// The landing-page hero: a compact analytics dashboard.
fn hero(ui: &mut Ui, s: &mut State) {
    ui.row().fill().show(|ui| {
        ui.sidebar().width(208.0).show(|ui| {
            ui.nav_header("N", "Northwind");
            ui.nav_section("Workspace");
            ui.nav_item("layout-dashboard", "Overview").selected(true);
            ui.nav_item("chart-line", "Reports");
            ui.nav_item("users", "Customers").badge(4);
            ui.nav_item("settings", "Settings");
        });
        ui.column().fill().pad(24).gap(16).show(|ui| {
            ui.row().fill_width().align(Align::Center).gap(12).show(|ui| {
                ui.heading("Overview").level(2);
                ui.spacer();
                ui.segmented(&mut s.range, &[(Range::Day, "Day"), (Range::Week, "Week"), (Range::Month, "Month")]);
                ui.date_picker("", &mut s.date);
            });
            ui.row().fill_width().gap(16).show(|ui| {
                for (label, value, delta, trend) in [
                    ("Revenue", "$118.4k", 12.4, revenue()),
                    ("Active users", "8,214", 4.1, vec![60.0, 62.0, 61.0, 66.0, 70.0, 69.0, 74.0, 78.0]),
                    ("Churn", "2.1%", -0.6, vec![3.0, 2.8, 2.9, 2.6, 2.5, 2.3, 2.2, 2.1]),
                ] {
                    ui.card().pad(16.0).show(|ui| ui.stat(label, value, Some(delta), Some(&trend)));
                }
            });
            ui.row().fill_width().gap(16).align(Align::Start).show(|ui| {
                ui.column().fill_width().show(|ui| {
                    ui.card().title("Revenue").pad(16.0).show(|ui| {
                        ui.line_chart("hero-rev", &MONTHS, vec![Series::new("Revenue", revenue()), Series::new("Costs", costs())]).smooth(true).area().height(200.0);
                    });
                });
                ui.column().width(330.0).show(|ui| {
                    ui.card().title("Traffic").pad(16.0).show(|ui| {
                        ui.donut_sized("hero-traffic", &[("Search", 42.0), ("Direct", 26.0), ("Social", 15.0), ("Other", 17.0)], 120.0);
                    });
                });
            });
        });
    });
}

/// The same controls in every look.
fn looks(ui: &mut Ui, s: &mut State) {
    ui.set_look(s.look.look());
    frame(ui, |ui| {
        let opts: Vec<(Platform, &str)> = Platform::ALL.iter().map(|p| (*p, p.name())).collect();
        ui.segmented(&mut s.look, &opts);
        ui.column().fill_width().max_width(520.0).show(|ui| {
            ui.card().title("Sync").description("Everything blends when you switch looks.").show(|ui| {
                ui.settings(|rows| {
                    rows.row("iCloud Drive", "Keep files up to date.", |ui| {
                        ui.switch("", &mut s.on[0]);
                    });
                    let v = format!("{:.0}%", s.vol * 100.0);
                    rows.row("Quality", "", |ui| {
                        ui.slider("", &mut s.vol, 0.0..=1.0).readout(v).width(180.0);
                    });
                });
                ui.row().gap(12).show(|ui| {
                    ui.checkbox("Remember me", &mut s.checks[0]);
                    ui.radio(&mut s.plan, 1, "Fast");
                    ui.radio(&mut s.plan, 2, "Small");
                });
                ui.row().gap(8).show(|ui| {
                    ui.text_input(&mut s.name).fill_width();
                    ui.button("Cancel");
                    ui.button("Save").primary();
                });
            });
        });
    });
}

/// A preview of the password manager we plan to build with lumen.
fn vault(ui: &mut Ui, s: &mut State) {
    ui.row().fill().show(|ui| {
        ui.sidebar().width(200.0).show(|ui| {
            ui.nav_header("V", "Vault");
            ui.nav_section("Library");
            ui.nav_item("key-round", "All items").badge(128).selected(true);
            ui.nav_item("star", "Favorites");
            ui.nav_item("credit-card", "Cards");
            ui.nav_item("file-lock", "Secure notes");
            ui.nav_section("Health");
            ui.nav_item("shield-alert", "Watchtower").badge(3);
        });
        ui.column().fill().pad(24).gap(16).show(|ui| {
            ui.row().fill_width().gap(10).align(Align::Center).show(|ui| {
                ui.text_input(&mut s.search).icon("search").placeholder("Search 128 items").fill_width();
                ui.button("New item").primary().icon("plus");
            });
            ui.card().pad(0.0).show(|ui| {
                ui.column().fill_width().pad(16).gap(14).show(|ui| {
                    ui.row().fill_width().gap(12).align(Align::Center).show(|ui| {
                        ui.column().gap(2).fill_width().show(|ui| {
                            ui.label("GitHub").strong();
                            ui.label("ada@analytical.engine").secondary();
                        });
                        ui.badge("Strong", Tone::Success);
                    });
                    ui.divider();
                    ui.row().fill_width().gap(10).align(Align::Center).show(|ui| {
                        let mut pw = "••••••••••••••••".to_string();
                        ui.text_input(&mut pw).label("Password").mono().fill_width().id("pw");
                        ui.column().pad(Sides { top: 22.0, ..Default::default() }).show(|ui| {
                            ui.row().gap(4).show(|ui| {
                                ui.icon_button("eye").tooltip("Reveal");
                                ui.icon_button("copy").tooltip_shortcut("Copy password", "⌘C");
                            });
                        });
                    });
                    ui.label("One-time code").small().secondary();
                    ui.row().gap(12).align(Align::Center).show(|ui| {
                        ui.label("482 913").role(lumen::TextRole::Heading2);
                        ui.scope("otp", |ui| ui.progress(Some(0.6)));
                    });
                });
            });
            ui.card().title("Password health").pad(16.0).show(|ui| {
                ui.row().gap(24).align(Align::Center).show(|ui| {
                    let good = ui.color(lumen::Token::Success);
                    ui.gauge_colored("health", 0.82, "Score", good);
                    ui.column().gap(8).show(|ui| {
                        ui.status("3 reused passwords", Tone::Warning);
                        ui.status("1 in a known breach", Tone::Danger);
                        ui.status("124 strong", Tone::Success);
                    });
                });
            });
        });
    });
}

/// Input that opens a demo's overlay for its screenshot (`--open`).
#[cfg(not(target_arch = "wasm32"))]
fn open_steps(demo: &str) -> Vec<lumen::ShotStep> {
    use lumen::{Modifiers, ShotStep::*};
    let k = |key| Key(key, Modifiers::default());
    match demo {
        "combobox" => vec![k(lumen::Key::Tab), Type("an".into()), Wait(0.6)],
        "multi-select" => vec![k(lumen::Key::Tab), k(lumen::Key::ArrowDown), k(lumen::Key::ArrowDown), Wait(0.6)],
        "otp-input" => vec![k(lumen::Key::Tab), Wait(0.4)],
        _ => vec![],
    }
}

fn parse_look(s: &str) -> Platform {
    match s {
        "mac" | "macos" => Platform::Mac,
        "windows" | "win" => Platform::Windows,
        "gnome" | "linux" => Platform::Gnome,
        _ => Platform::Lumen,
    }
}

fn parse_accent(s: &str) -> Accent {
    if let Some(hex) = s.strip_prefix('#') {
        if let Ok(v) = u32::from_str_radix(hex, 16) {
            return Accent::Custom(v);
        }
    }
    Accent::ALL.iter().copied().find(|a| a.name().eq_ignore_ascii_case(s)).unwrap_or(Accent::Blue)
}

struct Params {
    demo: String,
    dark: bool,
    look: Platform,
    accent: Accent,
    screenshot: Option<String>,
    size: (u32, u32),
    scale: f32,
    embed: bool,
    open: bool,
    speed: f32,
    reduced: bool,
}

fn params() -> Params {
    let mut p = Params { demo: "hero".into(), dark: false, look: Platform::Lumen, accent: Accent::Blue, screenshot: None, size: (960, 600), scale: 2.0, embed: false, open: false, speed: 1.0, reduced: false };
    #[cfg(target_arch = "wasm32")]
    {
        let q = web_sys::window().and_then(|w| w.location().search().ok()).unwrap_or_default();
        for kv in q.trim_start_matches('?').split('&') {
            let (k, v) = kv.split_once('=').unwrap_or((kv, ""));
            let v = v.replace("%23", "#");
            match k {
                "demo" => p.demo = v.to_string(),
                "dark" => p.dark = v == "1" || v == "true",
                "look" => p.look = parse_look(&v),
                "accent" => p.accent = parse_accent(&v),
                "embed" => p.embed = v == "1",
                "speed" => p.speed = v.parse().unwrap_or(1.0),
                "reduced" => p.reduced = v == "1",
                _ => {}
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let mut i = 0;
        while i < args.len() {
            let next = args.get(i + 1).cloned().unwrap_or_default();
            match args[i].as_str() {
                "--dark" => p.dark = true,
                "--look" => {
                    p.look = parse_look(&next);
                    i += 1;
                }
                "--accent" => {
                    p.accent = parse_accent(&next);
                    i += 1;
                }
                "--screenshot" => {
                    p.screenshot = Some(next);
                    i += 1;
                }
                "--size" => {
                    let (w, h) = next.split_once('x').unwrap_or(("960", "600"));
                    p.size = (w.parse().unwrap_or(960), h.parse().unwrap_or(600));
                    i += 1;
                }
                "--embed" => p.embed = true,
                "--open" => p.open = true,
                "--scale" => {
                    p.scale = next.parse().unwrap_or(2.0);
                    i += 1;
                }
                "--list" => {
                    for (k, t) in DEMOS {
                        println!("{k}\t{t}");
                    }
                    std::process::exit(0);
                }
                d if !d.starts_with('-') => p.demo = d.to_string(),
                _ => {}
            }
            i += 1;
        }
    }
    p
}

fn main() {
    let mut p = params();
    if p.look != Platform::Lumen && p.accent == Accent::Blue {
        p.accent = Accent::Native;
    }
    let mut state = State::new(p.demo.clone());
    state.embed = p.embed;
    state.force_open = p.open;
    state.look = p.look;
    let mode = if p.dark { Mode::Dark } else { Mode::Light };
    let mut look: Look = p.look.look();
    if p.speed > 0.0 && p.speed != 1.0 {
        look.motion = look.motion.slower(1.0 / p.speed);
    }
    if p.reduced {
        state.reduced = true;
    }
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(out) = &p.screenshot {
        let steps = if p.open { open_steps(&p.demo) } else { vec![] };
        let shot = lumen::Shot { size: p.size, scale: p.scale, mode, accent: p.accent, look, steps, ..Default::default() };
        match lumen::screenshot(out, &shot, &mut state, view) {
            Ok(()) => println!("wrote {out}"),
            Err(e) => {
                eprintln!("screenshot failed: {e}");
                std::process::exit(1);
            }
        }
        return;
    }
    let title = DEMOS.iter().find(|d| d.0 == p.demo).map(|d| d.1).unwrap_or("lumen");
    lumen::App::new(format!("lumen — {title}")).size(p.size.0, p.size.1).mode(mode).accent(p.accent).look(look).canvas("lumen").run(state, view);
}
