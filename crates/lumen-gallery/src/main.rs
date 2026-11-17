//! lumen gallery: a settings screen for a fictional deploy tool, "Relay",
//! showing every batch-1 widget doing a real job (docs/widgets-batch1.md §11).
//!
//! Run: `cargo run -p lumen-gallery`
//! Screenshot: `cargo run -p lumen-gallery -- --screenshot out.png --dark --accent blue`

use lumen::{Align, ControlSize, Id, Justify, Key, Mode, Rect, Token, Ui};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Page {
    General,
    Members,
    Billing,
    ApiKeys,
    Profile,
    Notifications,
    Appearance,
    Security,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Section {
    General,
    Profile,
    Notifications,
    Appearance,
    ApiKeys,
    Danger,
    States,
}

impl Section {
    const ALL: [Section; 7] =
        [Section::General, Section::Profile, Section::Notifications, Section::Appearance, Section::ApiKeys, Section::Danger, Section::States];
    fn title(self) -> &'static str {
        match self {
            Section::General => "General",
            Section::Profile => "Profile",
            Section::Notifications => "Notifications",
            Section::Appearance => "Appearance",
            Section::ApiKeys => "API keys",
            Section::Danger => "Danger zone",
            Section::States => "States",
        }
    }
    fn description(self) -> &'static str {
        match self {
            Section::General => "Basics for the Northwind Labs workspace.",
            Section::Profile => "How you appear to teammates.",
            Section::Notifications => "Choose what reaches you and where.",
            Section::Appearance => "Applies to this device only.",
            Section::ApiKeys => "Keys let scripts and CI act on behalf of this workspace.",
            Section::Danger => "These actions cannot be undone.",
            Section::States => "Every button variant at every size, for visual review.",
        }
    }
    fn page(self) -> Option<Page> {
        Some(match self {
            Section::General => Page::General,
            Section::Profile => Page::Profile,
            Section::Notifications => Page::Notifications,
            Section::Appearance => Page::Appearance,
            Section::ApiKeys => Page::ApiKeys,
            _ => return None,
        })
    }
}

struct Gallery {
    page: Page,
    collapsed: bool,
    scrolling_to: Option<Section>,
    current: Section,
    ws_name: String,
    ws_url: String,
    billing_email: String,
    ws_id: String,
    full_name: String,
    role: String,
    search: String,
    deploy_failures: bool,
    mentions: bool,
    digest: bool,
    audit: bool,
    email: bool,
    desktop: bool,
    mobile: bool,
    sms: bool,
    text_size: f32,
    density: f32,
    reduce_motion: bool,
    anim_speed: f32,
    confirm: String,
    saved: u32,
}

impl Default for Gallery {
    fn default() -> Self {
        Gallery {
            page: Page::General,
            collapsed: false,
            scrolling_to: None,
            current: Section::General,
            ws_name: "Northwind Labs".into(),
            ws_url: "northwind labs".into(),
            billing_email: String::new(),
            ws_id: "ws_8f3k2m9q".into(),
            full_name: "Mara Okafor".into(),
            role: String::new(),
            search: String::new(),
            deploy_failures: true,
            mentions: true,
            digest: false,
            audit: false,
            email: true,
            desktop: true,
            mobile: false,
            sms: false,
            text_size: 13.0,
            density: 64.0,
            reduce_motion: false,
            anim_speed: 100.0,
            confirm: String::new(),
            saved: 0,
        }
    }
}

fn section_id(ui: &Ui, scroll: Id, s: Section) -> Id {
    // Containers made with `.id(key)` inside the scroll scope.
    let _ = ui;
    scroll.with(("container", s.title()))
}

fn view(ui: &mut Ui, s: &mut Gallery) {
    if ui.shortcut(Key::Char('t')) {
        let m = ui.mode().toggled();
        ui.set_mode(m);
    }
    ui.set_reduced_motion(s.reduce_motion);

    ui.row().fill().show(|ui| {
        sidebar(ui, s);
        ui.column().fill().show(|ui| {
            let scroll_id = ui.scroll_id("page");
            let info_offset = *ui.mem::<f32>(scroll_id.with("offset"));
            top_bar(ui, s, info_offset);
            page(ui, s, scroll_id);
        });
    });
}

fn sidebar(ui: &mut Ui, s: &mut Gallery) {
    let mut go: Option<Page> = None;
    ui.sidebar().collapsed(s.collapsed).show(|ui| {
        ui.nav_header("N", "Northwind Labs");
        ui.nav_section("Workspace");
        let items = [
            (Page::General, "settings-2", "General", None),
            (Page::Members, "users", "Members", Some(12)),
            (Page::Billing, "credit-card", "Billing", None),
            (Page::ApiKeys, "key-round", "API keys", None),
        ];
        for (p, icon, label, badge) in items {
            let mut it = ui.nav_item(icon, label).selected(s.page == p);
            if let Some(b) = badge {
                it = it.badge(b);
            }
            if it.clicked() {
                go = Some(p);
            }
        }
        ui.nav_section("Account");
        let items = [
            (Page::Profile, "user", "Profile", None, false),
            (Page::Notifications, "bell", "Notifications", Some(3), false),
            (Page::Appearance, "palette", "Appearance", None, false),
            (Page::Security, "shield-check", "Security", None, true),
        ];
        for (p, icon, label, badge, disabled) in items {
            let mut it = ui.nav_item(icon, label).selected(s.page == p).disabled(disabled);
            if let Some(b) = badge {
                it = it.badge(b);
            }
            if it.clicked() {
                go = Some(p);
            }
        }
        ui.nav_footer(|ui| {
            let (icon, label) = if s.collapsed { ("panel-left-open", "Expand") } else { ("panel-left-close", "Collapse") };
            if ui.nav_item(icon, label).clicked() {
                s.collapsed = !s.collapsed;
            }
        });
    });
    if let Some(p) = go {
        s.page = p;
        s.scrolling_to = Section::ALL.iter().copied().find(|sec| sec.page() == Some(p));
    }
}

fn top_bar(ui: &mut Ui, s: &mut Gallery, scroll: f32) {
    let edge = ui.color(Token::BorderSubtle).multiply_alpha((scroll / 16.0).clamp(0.0, 1.0));
    ui.row()
        .fill_width()
        .height(56.0)
        .pad((48.0, 0.0))
        .align(Align::Center)
        .bg(Token::Background)
        .show(|ui| {
            let bar = ui.last_node();
            ui.set_paint_after(
                bar,
                Box::new(move |cx, r| {
                    cx.painter.rect(Rect::from_xywh(r.min.x, r.max.y - 1.0, r.width(), 1.0), edge);
                }),
            );
            ui.row().align(Align::Center).show(|ui| {
                ui.label("Settings").muted();
                ui.space(6.0);
                ui.label("/").color(Token::TextDisabled);
                ui.space(6.0);
                ui.label(s.current.title()).strong();
            });
            ui.spacer();
            ui.accent_picker();
            ui.space(12.0);
            ui.row().width(1.0).height(16.0).bg(Token::Border).show(|_| {});
            ui.space(12.0);
            let dark = ui.mode() == Mode::Dark;
            let (icon, tip) = if dark { ("sun", "Switch to light theme") } else { ("moon", "Switch to dark theme") };
            if ui.icon_button(icon).ghost().tooltip_shortcut(tip, "T").clicked() {
                let m = ui.mode().toggled();
                ui.set_mode(m);
            }
        });
}

fn page(ui: &mut Ui, s: &mut Gallery, scroll_id: Id) {
    let width = ui.screen_size().x - if s.collapsed { 56.0 } else { 240.0 };
    let side = if width < 816.0 { 24.0 } else { 48.0 };
    let (_, info) = ui.scroll("page").pad(lumen::Sides { left: side, right: side, top: 40.0, bottom: 96.0 }).align(Align::Center).show(|ui| {
        ui.column().fill_width().max_width(720.0).show(|ui| {
            ui.heading("Settings").display();
            ui.space(8.0);
            ui.label("Manage your workspace, your account, and how Relay looks on this device.").secondary();
            ui.space(32.0);
            for (i, sec) in Section::ALL.into_iter().enumerate() {
                let c = ui.column().fill_width().id(sec.title()).margin_top(if i == 0 { 0.0 } else { 48.0 });
                c.show(|ui| {
                    ui.heading(sec.title()).level(3);
                    ui.space(4.0);
                    ui.label(sec.description()).secondary();
                    ui.space(16.0);
                    ui.column().fill_width().gap(16).show(|ui| section(ui, s, sec));
                });
            }
        });
    });
    *ui.mem::<f32>(scroll_id.with("offset")) = info.offset;

    // Scroll position <-> selected section.
    let view_top = ui.rect_of(scroll_id).map(|r| r.min.y).unwrap_or(56.0);
    let rects: Vec<(Section, Rect)> =
        Section::ALL.iter().filter_map(|&sec| ui.rect_of(section_id(ui, scroll_id, sec)).map(|r| (sec, r))).collect();
    if let Some(target) = s.scrolling_to {
        if let Some((_, r)) = rects.iter().find(|(sec, _)| *sec == target) {
            let goal = (info.offset + r.min.y - view_top - 40.0).max(0.0);
            let max = (info.content_height - info.viewport_height).max(0.0);
            let goal = goal.min(max);
            if (info.offset - goal).abs() < 1.0 {
                s.scrolling_to = None;
                s.current = target;
            } else {
                ui.scroll_to(scroll_id, goal);
            }
        }
    } else {
        let probe = view_top + 120.0;
        let mut cur = Section::General;
        for (sec, r) in &rects {
            if r.min.y <= probe {
                cur = *sec;
            }
        }
        // At the very bottom, the last section is current.
        if info.content_height - info.viewport_height - info.offset < 2.0 && info.offset > 0.0 {
            cur = rects.last().map(|r| r.0).unwrap_or(cur);
        }
        if cur != s.current {
            s.current = cur;
            if let Some(p) = cur.page() {
                if !matches!(s.page, Page::Members | Page::Billing) || cur != Section::General {
                    s.page = p;
                }
            }
        }
    }
}

fn section(ui: &mut Ui, s: &mut Gallery, sec: Section) {
    match sec {
        Section::General => general(ui, s),
        Section::Profile => profile(ui, s),
        Section::Notifications => notifications(ui, s),
        Section::Appearance => appearance(ui, s),
        Section::ApiKeys => api_keys(ui, s),
        Section::Danger => danger(ui, s),
        Section::States => states(ui),
    }
}

fn url_error(url: &str) -> Option<&'static str> {
    let ok = !url.is_empty() && url.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    (!ok).then_some("URLs can use lowercase letters, numbers and hyphens only.")
}

fn general(ui: &mut Ui, s: &mut Gallery) {
    let mut save = false;
    ui.card()
        .title("Workspace")
        .description("This name and URL appear in invites and deploy notifications.")
        .footer("Last saved 2 minutes ago.", |ui| {
            ui.button("Discard").ghost();
            if ui.button("Save changes").primary().clicked() {
                save = true;
            }
        })
        .show(|ui| {
            ui.row().fill_width().gap(16).align(Align::Start).show(|ui| {
                ui.text_input(&mut s.ws_name).label("Workspace name").fill_width();
                let err = url_error(&s.ws_url);
                ui.text_input(&mut s.ws_url).label("Workspace URL").icon("at-sign").error(err).fill_width();
            });
            ui.text_input(&mut s.billing_email)
                .label("Billing email")
                .icon("mail")
                .placeholder("billing@northwind.dev")
                .helper("Invoices and payment receipts are sent here.")
                .fill_width();
            ui.text_input(&mut s.ws_id).label("Workspace ID").mono().disabled(true).helper("Used in API requests. This cannot be changed.").fill_width();
        });
    if save {
        s.saved += 1;
    }
}

fn profile(ui: &mut Ui, s: &mut Gallery) {
    ui.card()
        .title("Your details")
        .action(|ui| {
            ui.button("Upload photo").small().icon("upload");
        })
        .show(|ui| {
            ui.row().fill_width().gap(16).align(Align::Start).show(|ui| {
                ui.text_input(&mut s.full_name).label("Full name").fill_width();
                ui.text_input(&mut s.role).label("Role").placeholder("e.g. Platform engineer").fill_width();
            });
            ui.text_input(&mut s.search).label("Search teammates").icon("search").placeholder("Find someone by name or email").fill_width();
        });
}

fn notifications(ui: &mut Ui, s: &mut Gallery) {
    ui.card().title("Activity").show(|ui| {
        ui.settings(|rows| {
            rows.row("Deploy failures", "When a production deploy fails or is rolled back.", |ui| {
                ui.switch("", &mut s.deploy_failures);
            });
            rows.row("Mentions and replies", "When someone mentions you or replies to your comment.", |ui| {
                ui.switch("", &mut s.mentions);
            });
            rows.row("Weekly digest", "A Monday summary of releases, incidents and review time.", |ui| {
                ui.switch("", &mut s.digest);
            });
            rows.row("Audit log exports", "Available on the Enterprise plan.", |ui| {
                ui.switch("", &mut s.audit).disabled(true);
            });
        });
    });
    ui.card().title("Channels").description("Where notifications are delivered.").show(|ui| {
        let enabled = [s.email, s.desktop, s.mobile];
        let all_on = enabled.iter().all(|&b| b);
        let none_on = enabled.iter().all(|&b| !b);
        let mut all = all_on;
        let parent = ui.checkbox("All channels", &mut all).indeterminate(!all_on && !none_on).show();
        if parent.changed() {
            let v = if !all_on && !none_on { true } else { all };
            s.email = v;
            s.desktop = v;
            s.mobile = v;
        }
        ui.column().gap(12).pad(lumen::Sides { left: 24.0, ..Default::default() }).show(|ui| {
            ui.checkbox("Email", &mut s.email).description("mara@northwind.dev");
            ui.checkbox("Desktop", &mut s.desktop);
            ui.checkbox("Mobile push", &mut s.mobile).description("Install the Relay app to enable.");
            ui.checkbox("SMS", &mut s.sms).description("Add a phone number in Security first.").disabled(true);
        });
    });
}

fn appearance(ui: &mut Ui, s: &mut Gallery) {
    ui.card().title("Interface").show(|ui| {
        ui.settings(|rows| {
            let ts = format!("{} px", s.text_size as i32);
            rows.row("Text size", "Scales interface text. The default is 13 px.", |ui| {
                ui.slider("", &mut s.text_size, 11.0..=16.0).step(1.0).readout(ts);
            });
            let d = format!("{}%", s.density.round() as i32);
            rows.row("Sidebar density", "How tightly navigation items are packed.", |ui| {
                ui.slider("", &mut s.density, 0.0..=100.0).readout(d);
            });
            rows.row("Reduce motion", "Replace movement with fades. Follows your system setting by default.", |ui| {
                ui.switch("", &mut s.reduce_motion);
            });
            let sp = format!("{}%", s.anim_speed.round() as i32);
            let locked = s.reduce_motion;
            rows.row("Animation speed", "Locked while Reduce motion is on.", |ui| {
                ui.slider("", &mut s.anim_speed, 25.0..=200.0).readout(sp).disabled(locked);
            });
        });
    });
    ui.theme.motion_scale = 100.0 / s.anim_speed.max(25.0);
}

fn api_keys(ui: &mut Ui, _s: &mut Gallery) {
    ui.card()
        .title("Active keys")
        .action(|ui| {
            ui.button("New key").icon("plus");
        })
        .footer("Keys are shown once at creation. Store them in your secret manager.", |ui| {
            ui.button("View audit log").trailing_icon("arrow-right");
        })
        .show(|ui| {
            ui.settings(|rows| {
                let keys = [
                    ("Production deploys", "rk_live_9d2f...c41a", "Last used 4 minutes ago"),
                    ("Staging CI", "rk_test_41ab...77e0", "Last used yesterday"),
                    ("Local development", "rk_test_c09e...1b3d", "Never used"),
                ];
                for (name, key, used) in keys {
                    rows.row_with(
                        |ui| {
                            ui.label(name).strong();
                            ui.label(key).mono().secondary();
                            ui.label(used).caption();
                        },
                        |ui| {
                            ui.scope(name, |ui| {
                                ui.row().gap(4).show(|ui| {
                                    ui.icon_button("copy").ghost().tooltip_shortcut("Copy key", "C");
                                    ui.icon_button("trash-2").ghost().tooltip("Revoke key");
                                });
                            });
                        },
                    );
                }
            });
        });
}

fn danger(ui: &mut Ui, s: &mut Gallery) {
    let ok = s.confirm == "Northwind Labs";
    ui.card()
        .title("Delete workspace")
        .description("Permanently removes Northwind Labs, its 12 members, and all deploy history. Billing stops at the end of the current period.")
        .footer("You will be asked to confirm once more.", |ui| {
            ui.button("Export data").icon("download");
            ui.button("Delete workspace").danger().disabled(!ok);
        })
        .show(|ui| {
            ui.text_input(&mut s.confirm).label("Type the workspace name to confirm").placeholder("Northwind Labs").width(320.0);
        });
}

fn states(ui: &mut Ui) {
    ui.card().show(|ui| {
        let cols: [(&str, f32); 6] = [("sm", 72.0), ("md", 84.0), ("lg", 100.0), ("md + icon", 112.0), ("icon only", 48.0), ("disabled", 84.0)];
        ui.column().gap(12).show(|ui| {
            ui.row().gap(16).align(Align::Center).show(|ui| {
                ui.row().width(96.0).show(|_| {});
                for (name, w) in cols {
                    ui.row().width(w).show(|ui| {
                        ui.label(name).caption();
                    });
                }
            });
            let rows = [
                ("Primary", lumen::Variant::Primary, "Deploy", "rocket"),
                ("Secondary", lumen::Variant::Secondary, "Preview", "eye"),
                ("Ghost", lumen::Variant::Ghost, "Cancel", "x"),
                ("Danger", lumen::Variant::Danger, "Delete", "trash-2"),
            ];
            for (name, v, text, icon) in rows {
                ui.scope(name, |ui| {
                    ui.row().gap(16).align(Align::Center).show(|ui| {
                        ui.row().width(96.0).show(|ui| {
                            ui.label(name).small().muted();
                        });
                        let cell = |ui: &mut Ui, w: f32, f: &mut dyn FnMut(&mut Ui)| {
                            ui.row().width(w).align(Align::Center).show(|ui| f(ui));
                        };
                        cell(ui, cols[0].1, &mut |ui| {
                            ui.button(text).variant(v).size(ControlSize::Sm);
                        });
                        cell(ui, cols[1].1, &mut |ui| {
                            ui.scope("md", |ui| {
                                ui.button(text).variant(v);
                            });
                        });
                        cell(ui, cols[2].1, &mut |ui| {
                            ui.scope("lg", |ui| {
                                ui.button(text).variant(v).size(ControlSize::Lg);
                            });
                        });
                        cell(ui, cols[3].1, &mut |ui| {
                            ui.scope("icon", |ui| {
                                ui.button(text).variant(v).icon(icon);
                            });
                        });
                        cell(ui, cols[4].1, &mut |ui| {
                            ui.icon_button(icon).variant(v).tooltip(text);
                        });
                        cell(ui, cols[5].1, &mut |ui| {
                            ui.scope("disabled", |ui| {
                                ui.button(text).variant(v).disabled(true);
                            });
                        });
                    });
                });
            }
        });
        ui.divider();
        ui.label("Tab through this page to see focus rings. Hover any icon button for its tooltip.").secondary();
        let _ = Justify::Start;
    });
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(code) = cli::run() {
        std::process::exit(code);
    }
    lumen::App::new("lumen gallery").size(1200, 800).min_size(960, 600).canvas("lumen").run(Gallery::default(), view);
}

#[cfg(not(target_arch = "wasm32"))]
mod cli {
    use super::*;
    use lumen::{vec2, Accent, Modifiers, Shot, ShotStep};

    fn accent(s: &str) -> Option<Accent> {
        Accent::ALL.iter().copied().find(|a| a.name().eq_ignore_ascii_case(s))
    }

    fn point(s: &str) -> lumen::Vec2 {
        let (x, y) = s.split_once(',').expect("x,y");
        vec2(x.trim().parse().unwrap(), y.trim().parse().unwrap())
    }

    /// `--screenshot out.png [--dark] [--accent NAME] [--size WxH] [--scale S]
    /// [--hover x,y] [--press x,y] [--click x,y] [--scroll dy] [--tab N]
    /// [--type TEXT] [--wait SECS] [--collapsed]`
    pub fn run() -> Option<i32> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let out = args.iter().position(|a| a == "--screenshot").map(|i| args[i + 1].clone())?;
        let mut shot = Shot::default();
        let mut state = Gallery::default();
        let mut i = 0;
        while i < args.len() {
            let next = || args.get(i + 1).cloned().unwrap_or_default();
            match args[i].as_str() {
                "--dark" => shot.mode = Mode::Dark,
                "--accent" => {
                    shot.accent = accent(&next()).expect("accent name");
                    i += 1;
                }
                "--size" => {
                    let v = next();
                    let (w, h) = v.split_once('x').unwrap();
                    shot.size = (w.parse().unwrap(), h.parse().unwrap());
                    i += 1;
                }
                "--scale" => {
                    shot.scale = next().parse().unwrap();
                    i += 1;
                }
                "--hover" => {
                    shot.steps.push(ShotStep::Hover(point(&next())));
                    i += 1;
                }
                "--press" => {
                    shot.steps.push(ShotStep::Press(point(&next())));
                    i += 1;
                }
                "--click" => {
                    shot.steps.push(ShotStep::Click(point(&next())));
                    i += 1;
                }
                "--scroll" => {
                    shot.steps.push(ShotStep::Scroll(vec2(700.0, 400.0), next().parse().unwrap()));
                    shot.steps.push(ShotStep::Wait(0.1));
                    i += 1;
                }
                "--tab" => {
                    for _ in 0..next().parse::<u32>().unwrap() {
                        shot.steps.push(ShotStep::Key(Key::Tab, Modifiers::default()));
                    }
                    i += 1;
                }
                "--type" => {
                    shot.steps.push(ShotStep::Type(next()));
                    i += 1;
                }
                "--wait" => {
                    shot.steps.push(ShotStep::Wait(next().parse().unwrap()));
                    i += 1;
                }
                "--collapsed" => state.collapsed = true,
                _ => {}
            }
            i += 1;
        }
        match lumen::screenshot(&out, &shot, &mut state, view) {
            Ok(()) => {
                println!("wrote {out}");
                Some(0)
            }
            Err(e) => {
                eprintln!("screenshot failed: {e}");
                Some(1)
            }
        }
    }
}
