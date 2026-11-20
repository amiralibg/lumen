//! Showcase scene: Collaborative Spec & RFC Studio.

use lumen::{
    Align, AttachmentItem, ChecklistStep, CommentEntry, PresencePeer, ReactionChip, Toast, Tone, Ui,
};

const SPEC_MD: &str = r#"# Unified SDF & Atlas Pipeline
All rounded rectangles, hairline borders, Gaussian shadows and glyph quads batch into **one draw pass**.
> Semantic tokens interpolate in perceptual OKLab so theme transitions stay crisp across every OS look.
- 64-byte compact vertex layout eliminates pipeline state switches
- Physical pixel snapping keeps 1px strokes sharp at 1.5x and 2.0x DPI
```rust
painter.rrect(bounds, radii, ui.color(Token::Surface));
```"#;

pub struct State {
    pub doc_title: String,
    pub follow_peer: bool,
    pub reactions: Vec<ReactionChip>,
    pub attachments: Vec<AttachmentItem>,
    pub comments: Vec<CommentEntry>,
    pub resolved: bool,
    pub steps: Vec<ChecklistStep>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            doc_title: "Single-Pass OKLab Uber-Shader".into(),
            follow_peer: true,
            reactions: vec![
                ReactionChip::new("rocket", "Ship it", 12, true),
                ReactionChip::new("sparkles", "Crisp", 9, true),
                ReactionChip::new("flame", "Fast", 6, false),
                ReactionChip::new("heart", "Clean", 4, false),
            ],
            attachments: vec![
                AttachmentItem::uploaded("sdf_uber.wgsl", "WGSL", "14.6 KB"),
                AttachmentItem::uploaded("frame_bench_m4.json", "JSON", "4.8 KB"),
            ],
            comments: vec![
                CommentEntry::new(
                    "Lena Fischer",
                    "LF",
                    "4m ago",
                    "Clamp Gaussian sigma to a 0.5px floor so 2x Retina scissor corners stay sharp.",
                )
                .with_likes(4, true),
                CommentEntry::new(
                    "Jonas Berg",
                    "JB",
                    "1m ago",
                    "Done in sdf_uber.wgsl — verified zero Mach banding across Light and Dark modes.",
                )
                .reply()
                .with_likes(3, false),
            ],
            resolved: false,
            steps: vec![
                ChecklistStep::new(
                    "Zero idle wakeups verified",
                    "repaint_after == None across all looks.",
                    "Perf",
                    true,
                ),
                ChecklistStep::new(
                    "Semantic OKLab contrast audit",
                    "APCA contrast verified across Light and Dark.",
                    "Theme",
                    true,
                ),
                ChecklistStep::new(
                    "Final WGSL binding sign-off",
                    "Freeze uniform layout for v0.5 release.",
                    "Gate",
                    false,
                ),
            ],
        }
    }
}

pub fn view(ui: &mut Ui, s: &mut State) {
    let peers = [
        PresencePeer::new("LF", "Lena Fischer", "Reviewing shader spec", 0).editing(true),
        PresencePeer::new("JB", "Jonas Berg", "Verifying scissor clip", 1).editing(true),
        PresencePeer::new("PN", "Priya Nair", "Auditing OKLab tokens", 2),
        PresencePeer::new("TR", "Tomas Rivera", "Reading WGSL bindings", 3),
        PresencePeer::new("AK", "Aiko Kato", "Testing WebGPU target", 0),
    ];

    ui.column().fill().pad(16).gap(12).show(|ui| {
        // Top header bar
        ui.row().fill_width().align(Align::Center).gap(10).show(|ui| {
            ui.badge("RFC-042", Tone::Accent);
            ui.editable_text("rfc_title", &mut s.doc_title).heading();
            ui.spacer();
            // The pile's status column fills; give the pile a definite width
            // so the title can't squeeze it to nothing.
            ui.column().width(400.0).show(|ui| {
                if let Some(idx) = ui.presence_pile("rfc_peers", &peers, 4, &mut s.follow_peer) {
                    ui.toast(Toast::new(format!("Jumped to {}’s viewport", peers[idx].name)).tone(Tone::Info));
                }
            });
            if ui.button("Approve RFC").primary().icon("shield-check").clicked() {
                s.steps = vec![
                    ChecklistStep::new(
                        "Zero idle wakeups verified",
                        "repaint_after == None across all looks.",
                        "Perf",
                        true,
                    ),
                    ChecklistStep::new(
                        "Semantic OKLab contrast audit",
                        "APCA contrast verified across Light and Dark.",
                        "Theme",
                        true,
                    ),
                    ChecklistStep::new(
                        "Final WGSL binding sign-off",
                        "Freeze uniform layout for v0.5 release.",
                        "Gate",
                        true,
                    ),
                ];
                s.resolved = true;
                ui.toast(Toast::new("RFC-042 approved · all sign-off gates complete").success());
            }
        });

        // Main 2-column studio workspace
        ui.row().fill_width().gap(12).align(Align::Start).show(|ui| {
            // Left column: spec card + reactions, stacked above uploaded attachments
            ui.column().fill_width().gap(12).show(|ui| {
                ui.card().pad(14.0).show(|ui| {
                    ui.column().fill_width().gap(10).show(|ui| {
                        ui.markdown("rfc_md", SPEC_MD);
                        ui.divider();
                        ui.reaction_bar("rfc_rx", &mut s.reactions);
                    });
                });
                ui.attachment_list("rfc_files", &mut s.attachments);
            });

            // Right column: inline comment thread stacked above sign-off checklist
            ui.column().width(400.0).gap(12).show(|ui| {
                ui.comment_thread(
                    "rfc_thread",
                    "Inline review · paint.rs",
                    &mut s.comments,
                    &mut s.resolved,
                );
                ui.checklist_card("rfc_gates", "Sign-off checklist", &mut s.steps);
            });
        });
    });
}
