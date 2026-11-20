//! Visual Shader Studio showcase scene (`1040x640`).

use lumen::{vec2, Align, GraphNode, GraphWire, Toast, Token, Tone, Ui};

pub struct State {
    pub preset: usize,
    pub snap_grid: bool,
    pub isolines: bool,
    pub nodes: Vec<GraphNode>,
    pub origin: (f64, f64),
    pub corner_radius: f64,
    pub blur_sigma: f64,
    pub opacity: f64,
    pub premultiply: bool,
    pub compare_ratio: f32,
    pub dispersion: f32,
    pub white_point: f32,
}

fn default_nodes() -> Vec<GraphNode> {
    vec![
        GraphNode::new("UV & DPI", vec2(14.0, 34.0), &[], &["uv", "dpi"], Tone::Info),
        GraphNode::new(
            "SDF RoundRect",
            vec2(196.0, 28.0),
            &["uv", "radii"],
            &["dist"],
            Tone::Accent,
        ),
        GraphNode::new(
            "OKLab Shade",
            vec2(382.0, 48.0),
            &["dist", "dpi"],
            &["rgba"],
            Tone::Success,
        ),
    ]
}

impl Default for State {
    fn default() -> Self {
        Self {
            preset: 0,
            snap_grid: true,
            isolines: false,
            nodes: default_nodes(),
            origin: (24.0, 16.0),
            corner_radius: 18.0,
            blur_sigma: 28.0,
            opacity: 88.0,
            premultiply: true,
            compare_ratio: 0.52,
            dispersion: 0.42,
            white_point: 5800.0,
        }
    }
}

impl State {
    fn apply_preset(&mut self, idx: usize) {
        match idx {
            0 => {
                // Frosted Glass
                self.corner_radius = 18.0;
                self.blur_sigma = 28.0;
                self.dispersion = 0.42;
                self.opacity = 88.0;
                self.white_point = 5800.0;
            }
            1 => {
                // Aurora
                self.corner_radius = 24.0;
                self.blur_sigma = 44.0;
                self.dispersion = 0.84;
                self.opacity = 78.0;
                self.white_point = 6500.0;
            }
            _ => {
                // Hairline
                self.corner_radius = 8.0;
                self.blur_sigma = 6.0;
                self.dispersion = 0.12;
                self.opacity = 96.0;
                self.white_point = 5200.0;
            }
        }
    }
}

pub fn view(ui: &mut Ui, s: &mut State) {
    ui.column().fill().pad(16).gap(12).show(|ui| {
        // Top studio header & controls
        ui.row().fill_width().gap(12).align(Align::Center).show(|ui| {
            ui.heading("WGSL Uber-Shader Studio").level(2);
            let prev_preset = s.preset;
            ui.segmented(
                &mut s.preset,
                &[(0, "Frosted Glass"), (1, "Aurora"), (2, "Hairline")],
            );
            if s.preset != prev_preset {
                s.apply_preset(s.preset);
            }
            ui.spacer();
            ui.toolbar("shader_tb", |tb| {
                tb.toggle("grid", "Snap grid", &mut s.snap_grid);
                tb.toggle("eye", "SDF isolines", &mut s.isolines);
                tb.separator();
                if tb.action("rotate-ccw", "Reset") {
                    s.nodes = default_nodes();
                    tb.ui.toast(Toast::new("Reset shader graph positions").tone(Tone::Info));
                }
                tb.separator();
                if tb.primary("Compile WGSL") {
                    tb.ui.toast(Toast::new("Compiled uber-shader in 1.8 ms").success());
                }
            });
        });

        // Upper row: DAG node editor + property inspector
        let wires = [
            GraphWire::new(0, 0, 1, 0),
            GraphWire::new(0, 1, 2, 1),
            GraphWire::new(1, 0, 2, 0),
        ];
        ui.row().fill_width().gap(12).align(Align::Start).show(|ui| {
            ui.column().fill_width().show(|ui| {
                ui.node_editor("shader_dag", &mut s.nodes, &wires, 224.0);
            });
            ui.column().width(316.0).show(|ui| {
                ui.property_inspector("shader_props", |insp| {
                    insp.section("Transform & SDF");
                    let (mut px, mut py) = s.origin;
                    insp.vec2_row("Origin", &mut px, &mut py);
                    s.origin = (px, py);
                    insp.number("Corner radius", &mut s.corner_radius, 0.0..=48.0, "px");
                    insp.section("Material");
                    insp.number("Shadow blur", &mut s.blur_sigma, 0.0..=64.0, "px");
                    insp.number("Glass opacity", &mut s.opacity, 0.0..=100.0, "%");
                    insp.toggle("Premultiply alpha", &mut s.premultiply);
                });
            });
        });

        // Lower row: A/B wireframe vs SDF split + chromatic dispersion & Kelvin tuning
        let aurora_stops = [
            ui.color(Token::Series0),
            ui.color(Token::Series3),
            ui.color(Token::Series4),
        ];
        let kelvin_stops = [
            ui.color(Token::Warning),
            ui.color(Token::Knob),
            ui.color(Token::Info),
        ];
        ui.row().fill_width().gap(12).align(Align::Start).show(|ui| {
            ui.column().fill_width().show(|ui| {
                ui.compare_split(
                    "sdf_ab",
                    &mut s.compare_ratio,
                    "Wireframe mesh",
                    "SDF shaded",
                )
                .height(190.0);
            });
            ui.column().width(316.0).show(|ui| {
                ui.card()
                    .title("Chromatic & Kelvin")
                    .pad(14.0)
                    .show(|ui| {
                        ui.column().fill_width().gap(14).show(|ui| {
                            ui.gradient_slider(
                                "aurora_mix",
                                "Aurora dispersion",
                                &mut s.dispersion,
                                0.0..=1.0,
                                &aurora_stops,
                                "",
                            );
                            ui.gradient_slider(
                                "white_point",
                                "White point",
                                &mut s.white_point,
                                2700.0..=7500.0,
                                &kelvin_stops,
                                " K",
                            );
                        });
                    });
            });
        });
    });
}
