use std::collections::BTreeSet;

use lumen::{Align, EditorTab, SuggestionDecision, Toast, Token, Tone, TreeNode, Ui};

const SHADER_ORIGINAL: &str = r#"@fragment fn fs_shadow(in: Varyings) -> @location(0) vec4<f32> {
    let dist = sd_round_rect(in.local, in.half_size, in.radii);
    let alpha = clamp(1.0 - dist / in.blur, 0.0, 1.0);
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}"#;

const SHADER_ACCEPTED: &str = r#"@fragment fn fs_shadow(in: Varyings) -> @location(0) vec4<f32> {
    let dist = sd_round_rect(in.local, in.half_size, in.radii);
    let sigma = max(in.blur * 0.5, 0.5);
    let alpha = 0.5 * erfc(dist / (sigma * 1.4142135));
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}"#;

const PAINT_SNIPPET: &str = r#"pub fn shadow_rrect(&mut self, rect: Rect, radii: Corners, blur: f32, color: Color) {
    let pad = (blur * 1.5).ceil();
    let bounds = self.snap_rect(rect.expand(pad));
    self.push_sdf_quad(bounds, radii, blur, color);
}"#;

const THEME_SNIPPET: &str = r#"pub fn resolve_token(&self, mode: Mode, token: Token, t: f32) -> Color {
    let from = self.source_look.color(mode, token);
    let into = self.target_look.color(mode, token);
    from.mix_oklab(into, t.clamp(0.0, 1.0))
}"#;

const ANIM_SNIPPET: &str = r#"pub fn is_settled(&self, target: f32) -> bool {
    let displacement = (self.pos - target).abs();
    displacement < 0.0005 && self.vel.abs() < 0.0005
}"#;

const SCRATCH_SNIPPET: &str = r#"#[test]
fn sdf_shadow_settles_without_wakeups() {
    let mut ui = Ui::headless(1040.0, 640.0);
    assert!(ui.end_frame().repaint_after.is_none());
}"#;

pub struct State {
    pub split_ratio: f32,
    pub tree_open: BTreeSet<String>,
    pub tree_sel: String,
    pub ed_tabs: Vec<EditorTab>,
    pub ed_active: usize,
    pub patch_decision: SuggestionDecision,
    pub prompt: String,
    pub ai_model: usize,
    pub ai_files: Vec<String>,
    pub ai_streaming: bool,
    pub scratch_seq: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            split_ratio: 0.24,
            tree_open: ["crates", "lumen-wgpu", "lumen-core"]
                .into_iter()
                .map(String::from)
                .collect(),
            tree_sel: "shader.wgsl".into(),
            ed_tabs: vec![
                EditorTab::new("shader.wgsl", "shader.wgsl")
                    .icon("sparkles")
                    .dirty(true),
                EditorTab::new("paint.rs", "paint.rs").icon("file-code"),
                EditorTab::new("theme.rs", "theme.rs").icon("palette"),
                EditorTab::new("anim.rs", "anim.rs").icon("activity"),
            ],
            ed_active: 0,
            patch_decision: SuggestionDecision::Pending,
            prompt: "Verify analytical erfc shadow falloff stays branch-free under 2x Retina scissor clips.".into(),
            ai_model: 0,
            ai_files: vec!["shader.wgsl".into(), "paint.rs".into()],
            ai_streaming: false,
            scratch_seq: 0,
        }
    }
}

fn file_icon(name: &str) -> &'static str {
    match name {
        "shader.wgsl" => "sparkles",
        "paint.rs" => "file-code",
        "theme.rs" => "palette",
        "anim.rs" => "activity",
        _ => "file-code",
    }
}

fn is_workspace_leaf(name: &str) -> bool {
    matches!(name, "shader.wgsl" | "paint.rs" | "theme.rs" | "anim.rs")
}

pub fn view(ui: &mut Ui, s: &mut State) {
    let tree = [TreeNode::branch(
        "crates",
        "crates",
        [
            TreeNode::branch(
                "lumen-wgpu",
                "lumen-wgpu",
                [TreeNode::leaf("shader.wgsl", "shader.wgsl")
                    .icon("sparkles")
                    .meta("6.1 KB")],
            ),
            TreeNode::branch(
                "lumen-core",
                "lumen-core",
                [
                    TreeNode::leaf("paint.rs", "paint.rs")
                        .icon("file-code")
                        .meta("18.2 KB"),
                    TreeNode::leaf("theme.rs", "theme.rs")
                        .icon("palette")
                        .meta("24.8 KB"),
                    TreeNode::leaf("anim.rs", "anim.rs")
                        .icon("activity")
                        .meta("9.4 KB"),
                ],
            ),
        ],
    )];

    ui.column().fill().pad(14).gap(12).show(|ui| {
        // Top workspace menu + build check controls
        ui.row().fill_width().align(Align::Center).gap(10).show(|ui| {
            let mut menu_run_checks = false;
            let mut menu_new_file = false;
            ui.menu_bar("ide_menubar", |bar| {
                bar.menu("File", |ui| {
                    if ui.menu_item("plus", "New file").shortcut("⌘N").clicked() {
                        menu_new_file = true;
                    }
                    ui.menu_item("folder-open", "Open workspace…").shortcut("⌘O");
                });
                bar.menu("Edit", |ui| {
                    ui.menu_item("", "Undo").shortcut("⌘Z");
                    ui.menu_item("", "Redo").shortcut("⌘⇧Z");
                    ui.menu_separator();
                    ui.menu_item("copy", "Copy selection").shortcut("⌘C");
                });
                bar.menu("Build", |ui| {
                    if ui.menu_item("play", "Run checks").shortcut("⌘R").clicked() {
                        menu_run_checks = true;
                    }
                    ui.menu_item("flame", "Profile WGSL pass");
                });
            });
            if menu_new_file {
                s.scratch_seq += 1;
                let name = format!("scratch-{}.rs", s.scratch_seq);
                s.ed_tabs
                    .push(EditorTab::new(name.clone(), name.clone()).icon("file-code").dirty(true));
                s.ed_active = s.ed_tabs.len() - 1;
                s.tree_sel = name;
            }

            ui.spacer();
            ui.badge("0 warnings", Tone::Success);
            if ui.button("Run checks").primary().small().icon("play").clicked() || menu_run_checks {
                ui.toast(Toast::new("cargo check --workspace passed in 0.14s").success());
            }
        });

        // Center split workspace: Explorer tree on the left, editor + AI composer on the right
        let pane_h = (ui.screen_size().y - 122.0).max(440.0);
        let mut split_ratio = s.split_ratio;
        let cell = std::cell::RefCell::new(&mut *s);
        ui.split_pane("ide_split", &mut split_ratio)
            .height(pane_h)
            .show(
                |ui| {
                    let mut g = cell.borrow_mut();
                    let s: &mut State = &mut g;
                    ui.column().fill_width().gap(8).show(|ui| {
                        ui.row().fill_width().align(Align::Center).gap(6).show(|ui| {
                            let ic = ui.color(Token::AccentStrong);
                            ui.icon_leaf("layers", 13.0, ic);
                            ui.label("Explorer").strong();
                            ui.spacer();
                            ui.label("lumen").small().muted();
                        });
                        ui.divider();

                        let prev_sel = s.tree_sel.clone();
                        ui.tree("ide_tree", &tree, &mut s.tree_open, &mut s.tree_sel)
                            .show();

                        if s.tree_sel != prev_sel && is_workspace_leaf(&s.tree_sel) {
                            if let Some(idx) = s.ed_tabs.iter().position(|t| t.label == s.tree_sel)
                            {
                                s.ed_active = idx;
                            } else {
                                let name = s.tree_sel.clone();
                                s.ed_tabs.push(
                                    EditorTab::new(name.clone(), name.clone())
                                        .icon(file_icon(&name)),
                                );
                                s.ed_active = s.ed_tabs.len() - 1;
                            }
                        }
                    });
                },
                |ui| {
                    let mut g = cell.borrow_mut();
                    let s: &mut State = &mut g;
                    ui.column().fill_width().gap(10).show(|ui| {
                        let prev_active = s.ed_active;
                        let out = ui.editor_tabs("ide_tabs", &s.ed_tabs, &mut s.ed_active);

                        if let Some(ci) = out.closed {
                            if s.ed_tabs.len() > 1 {
                                s.ed_tabs.remove(ci);
                                if s.ed_active >= s.ed_tabs.len() {
                                    s.ed_active = s.ed_tabs.len() - 1;
                                }
                            }
                        }
                        if out.add_clicked {
                            s.scratch_seq += 1;
                            let name = format!("scratch-{}.rs", s.scratch_seq);
                            s.ed_tabs.push(
                                EditorTab::new(name.clone(), name)
                                    .icon("file-code")
                                    .dirty(true),
                            );
                            s.ed_active = s.ed_tabs.len() - 1;
                        }
                        if s.ed_active != prev_active || out.closed.is_some() || out.add_clicked {
                            if let Some(tab) = s.ed_tabs.get(s.ed_active) {
                                s.tree_sel = tab.label.clone();
                            }
                        }

                        let active_name = s
                            .ed_tabs
                            .get(s.ed_active)
                            .map(|t| t.label.clone())
                            .unwrap_or_else(|| "shader.wgsl".into());

                        if active_name == "shader.wgsl" {
                            match s.patch_decision {
                                SuggestionDecision::Pending => {
                                    let removed =
                                        ["    let alpha = clamp(1.0 - dist / in.blur, 0.0, 1.0);"];
                                    let added = [
                                        "    let sigma = max(in.blur * 0.5, 0.5);",
                                        "    let alpha = 0.5 * erfc(dist / (sigma * 1.4142135));",
                                    ];
                                    ui.code_suggestion(
                                        "sdf_patch",
                                        "crates/lumen-wgpu/src/shader.wgsl",
                                        "Replace linear shadow falloff with analytical Gaussian erfc integration.",
                                        &removed,
                                        &added,
                                        &mut s.patch_decision,
                                    );
                                    match s.patch_decision {
                                        SuggestionDecision::Pending => {}
                                        SuggestionDecision::Accepted => {
                                            if let Some(tab) = s
                                                .ed_tabs
                                                .iter_mut()
                                                .find(|t| t.label == "shader.wgsl")
                                            {
                                                *tab = EditorTab::new("shader.wgsl", "shader.wgsl")
                                                    .icon("sparkles");
                                            }
                                            ui.toast(
                                                Toast::new("Applied patch to shader.wgsl")
                                                    .success(),
                                            );
                                        }
                                        SuggestionDecision::Dismissed => {
                                            if let Some(tab) = s
                                                .ed_tabs
                                                .iter_mut()
                                                .find(|t| t.label == "shader.wgsl")
                                            {
                                                *tab = EditorTab::new("shader.wgsl", "shader.wgsl")
                                                    .icon("sparkles");
                                            }
                                        }
                                    }
                                }
                                SuggestionDecision::Accepted => {
                                    ui.row().fill_width().align(Align::Center).gap(8).show(|ui| {
                                        ui.badge("Patch applied · Gaussian erfc", Tone::Success);
                                        ui.spacer();
                                        if ui
                                            .button("Reset patch")
                                            .ghost()
                                            .small()
                                            .icon("refresh-cw")
                                            .clicked()
                                        {
                                            s.patch_decision = SuggestionDecision::Pending;
                                            if let Some(tab) = s
                                                .ed_tabs
                                                .iter_mut()
                                                .find(|t| t.label == "shader.wgsl")
                                            {
                                                *tab = EditorTab::new("shader.wgsl", "shader.wgsl")
                                                    .icon("sparkles")
                                                    .dirty(true);
                                            }
                                        }
                                    });
                                    ui.code_block("ed_shader_accepted", SHADER_ACCEPTED)
                                        .title("crates/lumen-wgpu/src/shader.wgsl")
                                        .language("wgsl")
                                        .highlight_lines(&[3, 4])
                                        .show();
                                }
                                SuggestionDecision::Dismissed => {
                                    ui.row().fill_width().align(Align::Center).gap(8).show(|ui| {
                                        ui.badge(
                                            "Patch dismissed · kept linear falloff",
                                            Tone::Neutral,
                                        );
                                        ui.spacer();
                                        if ui
                                            .button("Reset patch")
                                            .ghost()
                                            .small()
                                            .icon("refresh-cw")
                                            .clicked()
                                        {
                                            s.patch_decision = SuggestionDecision::Pending;
                                            if let Some(tab) = s
                                                .ed_tabs
                                                .iter_mut()
                                                .find(|t| t.label == "shader.wgsl")
                                            {
                                                *tab = EditorTab::new("shader.wgsl", "shader.wgsl")
                                                    .icon("sparkles")
                                                    .dirty(true);
                                            }
                                        }
                                    });
                                    ui.code_block("ed_shader_rejected", SHADER_ORIGINAL)
                                        .title("crates/lumen-wgpu/src/shader.wgsl")
                                        .language("wgsl")
                                        .highlight_lines(&[3])
                                        .show();
                                }
                            }
                        } else {
                            let (path, lang, snippet, hl): (&str, &str, &str, &[usize]) =
                                match active_name.as_str() {
                                    "paint.rs" => (
                                        "crates/lumen-core/src/paint.rs",
                                        "rust",
                                        PAINT_SNIPPET,
                                        &[2, 3],
                                    ),
                                    "theme.rs" => (
                                        "crates/lumen-core/src/theme.rs",
                                        "rust",
                                        THEME_SNIPPET,
                                        &[2, 3],
                                    ),
                                    "anim.rs" => (
                                        "crates/lumen-core/src/anim.rs",
                                        "rust",
                                        ANIM_SNIPPET,
                                        &[2, 3],
                                    ),
                                    _ => (
                                        "crates/lumen-core/tests/scratch.rs",
                                        "rust",
                                        SCRATCH_SNIPPET,
                                        &[3, 4],
                                    ),
                                };
                            ui.code_block("ed_code", snippet)
                                .title(path)
                                .language(lang)
                                .highlight_lines(hl)
                                .show();
                        }

                        let comp = ui
                            .prompt_composer("ide_composer", &mut s.prompt, &mut s.ai_model)
                            .models(&["Claude Sonnet", "Gemini Pro", "Local Qwen"])
                            .attachments(&mut s.ai_files)
                            .context_used(0.36)
                            .streaming(s.ai_streaming)
                            .show();

                        if comp.submitted {
                            s.ai_streaming = true;
                            ui.toast(
                                Toast::new("Running assistant with workspace context")
                                    .tone(Tone::Accent),
                            );
                        } else if comp.stopped {
                            s.ai_streaming = false;
                        } else if comp.attach_clicked && s.ai_files.len() < 4 {
                            if !s.ai_files.contains(&active_name) {
                                s.ai_files.push(active_name.clone());
                            } else {
                                s.ai_files
                                    .push(format!("context-{}.rs", s.ai_files.len() + 1));
                            }
                        }
                    });
                },
            );
        drop(cell);
        s.split_ratio = split_ratio;

        // Bottom workspace status bar
        let active_is_wgsl = s
            .ed_tabs
            .get(s.ed_active)
            .map(|t| t.label.ends_with(".wgsl"))
            .unwrap_or(true);
        ui.status_bar(|bar| {
            bar.status("LSP Ready", Tone::Success);
            if bar.item("git-branch", "ethereal*") {
                bar.ui.toast(Toast::new("Branch ethereal · clean working tree"));
            }
            bar.spacer();
            bar.text(if active_is_wgsl {
                "Ln 84, Col 16"
            } else {
                "Ln 42, Col 12"
            });
            bar.text(if active_is_wgsl {
                "WGSL · Rust"
            } else {
                "Rust · UTF-8"
            });
        });
    });
}
