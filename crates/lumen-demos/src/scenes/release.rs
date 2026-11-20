//! Release Control Plane showcase scene (`1040x640`).

use lumen::{Align, CommitRow, KanbanCard, KanbanCol, Toast, Tone, Ui, UptimeStatus};

struct GateSpec {
    id: &'static str,
    default_col: usize,
    title: &'static str,
    tag: &'static str,
    tone: Tone,
    meta: &'static str,
    assignee: &'static str,
}

const GATES: [GateSpec; 4] = [
    GateSpec {
        id: "r1",
        default_col: 0,
        title: "Verify ARM64 Metal pipeline",
        tag: "Gate",
        tone: Tone::Accent,
        meta: "REL-41",
        assignee: "LF",
    },
    GateSpec {
        id: "r2",
        default_col: 0,
        title: "Sign WebGPU WASM bundle",
        tag: "Supply",
        tone: Tone::Info,
        meta: "REL-42",
        assignee: "AK",
    },
    GateSpec {
        id: "r3",
        default_col: 1,
        title: "Canary bake · eu-central-1",
        tag: "SLO",
        tone: Tone::Warning,
        meta: "REL-40",
        assignee: "JB",
    },
    GateSpec {
        id: "r4",
        default_col: 2,
        title: "Zero idle wakeups v0.5",
        tag: "Live",
        tone: Tone::Success,
        meta: "REL-39",
        assignee: "PN",
    },
];

const COMMIT_REFS: [(&str, &str); 4] = [
    ("9e4c51c", "HEAD → ethereal · 4m ago"),
    ("825366a", "ring-clip-sdf · 22m ago"),
    ("6ca5a39", "v0.5-rc2 · 1h ago"),
    ("f58d8e2", "springs-idle · 3h ago"),
];

fn build_card(spec: &GateSpec, col: usize) -> KanbanCard {
    KanbanCard::new(spec.id, col, spec.title)
        .tag(spec.tag, spec.tone)
        .meta(spec.meta)
        .assignee(spec.assignee)
}

pub struct State {
    pub env_idx: usize,
    pub env_locked: bool,
    pub commit_sel: usize,
    pub cards: Vec<KanbanCard>,
    card_cols: [usize; 4],
}

impl Default for State {
    fn default() -> Self {
        let card_cols = [
            GATES[0].default_col,
            GATES[1].default_col,
            GATES[2].default_col,
            GATES[3].default_col,
        ];
        let cards = GATES
            .iter()
            .enumerate()
            .map(|(i, g)| build_card(g, card_cols[i]))
            .collect();
        Self {
            env_idx: 2,
            env_locked: true,
            commit_sel: 0,
            cards,
            card_cols,
        }
    }
}

#[derive(Clone, Copy)]
enum RolloutAction {
    PromoteNext,
    ShiftCanary,
    Rollback,
}

fn run_rollout_action(ui: &mut Ui, s: &mut State, action: RolloutAction, sha: &str) {
    if s.env_locked {
        ui.toast(
            Toast::new("Environment locked — unlock banner before promoting")
                .tone(Tone::Warning),
        );
        return;
    }

    match action {
        RolloutAction::PromoteNext => {
            // Advance a Canary card to Production first, otherwise advance a Queued card to Canary 25%.
            let target = s
                .card_cols
                .iter()
                .position(|&c| c == 1)
                .or_else(|| s.card_cols.iter().position(|&c| c == 0));
            if let Some(idx) = target {
                s.card_cols[idx] = (s.card_cols[idx] + 1).min(2);
                let stage = if s.card_cols[idx] == 2 {
                    "Production"
                } else {
                    "Canary 25%"
                };
                if idx < s.cards.len() {
                    s.cards[idx] = build_card(&GATES[idx], s.card_cols[idx]);
                }
                ui.toast(
                    Toast::new(format!(
                        "Promoted {} ({sha}) → {stage}",
                        GATES[idx].meta
                    ))
                    .success(),
                );
            } else {
                ui.toast(Toast::new("All release gates are live in Production").tone(Tone::Info));
            }
        }
        RolloutAction::ShiftCanary => {
            if let Some(idx) = s.card_cols.iter().position(|&c| c == 0) {
                s.card_cols[idx] = 1;
                if idx < s.cards.len() {
                    s.cards[idx] = build_card(&GATES[idx], 1);
                }
                ui.toast(
                    Toast::new(format!("Shifted {} to Canary 25%", GATES[idx].meta))
                        .tone(Tone::Accent),
                );
            } else {
                ui.toast(Toast::new("No queued builds waiting for canary").tone(Tone::Info));
            }
        }
        RolloutAction::Rollback => {
            for (i, g) in GATES.iter().enumerate() {
                s.card_cols[i] = g.default_col;
            }
            s.cards = GATES
                .iter()
                .enumerate()
                .map(|(i, g)| build_card(g, s.card_cols[i]))
                .collect();
            ui.toast(Toast::new("Rolled back canary traffic to v0.4.9").tone(Tone::Warning));
        }
    }
}

pub fn view(ui: &mut Ui, s: &mut State) {
    ui.column().fill().pad(16).gap(12).show(|ui| {
        // Top environment safety banner
        ui.env_banner("rel_env", &mut s.env_idx, "eu-central-1", &mut s.env_locked);

        // Compact header row with selected commit badge & split promote button
        let sel = s.commit_sel.min(COMMIT_REFS.len() - 1);
        let (sel_sha, sel_ref) = COMMIT_REFS[sel];
        ui.row().fill_width().gap(10).align(Align::Center).show(|ui| {
            ui.heading("Release Control Plane").level(2);
            ui.badge(format!("Commit {sel_sha}"), Tone::Accent);
            ui.badge(
                if s.env_locked { "Locked" } else { "Unlocked" },
                if s.env_locked {
                    Tone::Warning
                } else {
                    Tone::Success
                },
            );
            ui.label(sel_ref).small().muted();
            ui.spacer();

            let mut pending: Option<RolloutAction> = None;
            if ui
                .split_button("Promote canary")
                .primary()
                .icon("rocket")
                .show(|ui| {
                    if ui.menu_item("zap", "Shift canary to 25%").clicked() {
                        pending = Some(RolloutAction::ShiftCanary);
                    }
                    if ui.menu_item("rotate-ccw", "Rollback to v0.4.9").clicked() {
                        pending = Some(RolloutAction::Rollback);
                    }
                })
                .clicked()
            {
                pending = Some(RolloutAction::PromoteNext);
            }
            if let Some(act) = pending {
                run_rollout_action(ui, s, act, sel_sha);
            }
        });

        // Middle 2-column row: commit graph (4 commits) + regional SLO & uptime (36 bars each)
        let commits = [
            CommitRow::new(
                "9e4c51c",
                "widgets: studio EQ, AI patches & 170 components",
                "AL",
                "4m ago",
                0,
                vec![0, 1],
            )
            .badge("HEAD → ethereal"),
            CommitRow::new(
                "825366a",
                "paint: analytical Gaussian shadow ring clipping",
                "LF",
                "22m ago",
                1,
                vec![0],
            ),
            CommitRow::new(
                "6ca5a39",
                "wgpu: persistent pipeline cache & glyph atlas",
                "JB",
                "1h ago",
                0,
                vec![0],
            )
            .badge("v0.5-rc2"),
            CommitRow::new(
                "f58d8e2",
                "core: closed-form springs & zero idle wakeups",
                "PN",
                "3h ago",
                0,
                vec![],
            ),
        ];

        let mut edge_bars = vec![UptimeStatus::Operational; 36];
        edge_bars[11] = UptimeStatus::Degraded;
        let mut cdn_bars = vec![UptimeStatus::Operational; 36];
        cdn_bars[22] = UptimeStatus::Degraded;
        cdn_bars[23] = UptimeStatus::Outage;

        ui.row().fill_width().gap(12).align(Align::Start).show(|ui| {
            ui.column().fill_width().show(|ui| {
                ui.commit_graph("rel_commits", &commits, &mut s.commit_sel);
            });
            ui.column().width(360.0).show(|ui| {
                ui.card()
                    .title("Regional SLO & Uptime (45d)")
                    .pad(14.0)
                    .show(|ui| {
                        ui.column().fill_width().gap(10).show(|ui| {
                            ui.uptime_bars("u_edge", "Edge Ingress · 99.98%", &edge_bars);
                            ui.uptime_bars("u_cdn", "Artifact CDN · 99.94%", &cdn_bars);
                        });
                    });
            });
        });

        // Bottom progressive rollout Kanban board
        let cols = [
            KanbanCol::new("Queued", Tone::Neutral),
            KanbanCol::new("Canary 25%", Tone::Accent),
            KanbanCol::new("Production", Tone::Success),
        ];
        ui.kanban("rel_kanban", &cols, &mut s.cards);
    });
}
