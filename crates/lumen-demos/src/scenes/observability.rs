use lumen::{Align, FlameSpan, KpiMetric, LogEntry, LogLevel, Toast, Token, Tone, TraceSpan, Ui};

pub struct State {
    pub range: usize,
    pub captures: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            range: 0,
            captures: 0,
        }
    }
}

pub fn view(ui: &mut Ui, s: &mut State) {
    let (window_lbl, metrics, f_scale, base_t_scale): (&str, [KpiMetric; 3], f32, f32) =
        match s.range {
            1 => (
                "1h",
                [
                    KpiMetric::new(
                        "Throughput",
                        "136.4k/s",
                        9.8,
                        vec![88.0, 94.0, 101.0, 109.0, 118.0, 128.0, 136.4],
                    ),
                    KpiMetric::new(
                        "Median frame",
                        "0.21 ms",
                        5.2,
                        vec![0.28, 0.27, 0.25, 0.24, 0.23, 0.22, 0.21],
                    ),
                    KpiMetric::new(
                        "5xx error burn",
                        "0.03%",
                        -0.2,
                        vec![0.07, 0.06, 0.05, 0.05, 0.04, 0.03, 0.03],
                    ),
                ],
                1.12,
                1.18,
            ),
            2 => (
                "24h",
                [
                    KpiMetric::new(
                        "Throughput",
                        "119.6k/s",
                        18.4,
                        vec![72.0, 84.0, 79.0, 95.0, 108.0, 114.0, 119.6],
                    ),
                    KpiMetric::new(
                        "Median frame",
                        "0.24 ms",
                        11.0,
                        vec![0.31, 0.29, 0.28, 0.27, 0.26, 0.25, 0.24],
                    ),
                    KpiMetric::new(
                        "5xx error burn",
                        "0.04%",
                        -0.7,
                        vec![0.11, 0.09, 0.08, 0.06, 0.05, 0.05, 0.04],
                    ),
                ],
                1.25,
                1.35,
            ),
            _ => (
                "15m",
                [
                    KpiMetric::new(
                        "Throughput",
                        "148.2k/s",
                        14.2,
                        vec![94.0, 102.0, 108.0, 118.0, 126.0, 139.0, 148.2],
                    ),
                    KpiMetric::new(
                        "Median frame",
                        "0.18 ms",
                        8.4,
                        vec![0.26, 0.24, 0.23, 0.21, 0.20, 0.19, 0.18],
                    ),
                    KpiMetric::new(
                        "5xx error burn",
                        "0.02%",
                        -0.4,
                        vec![0.06, 0.05, 0.05, 0.04, 0.03, 0.02, 0.02],
                    ),
                ],
                1.0,
                1.0,
            ),
        };

    let capture_factor = if s.captures == 0 {
        1.0
    } else {
        1.0 - 0.03 * ((s.captures % 3) as f32)
    };
    let t_scale = base_t_scale * capture_factor;

    let frames = [
        FlameSpan::new("Ui::run_frame", 0, 0.0, 4.80 * f_scale),
        FlameSpan::new("layout::solve", 1, 0.12 * f_scale, 1.35 * f_scale),
        FlameSpan::new("measure_text", 2, 0.24 * f_scale, 0.78 * f_scale),
        FlameSpan::new("place_flex", 2, 1.06 * f_scale, 0.38 * f_scale),
        FlameSpan::new("paint::tessellate", 1, 1.55 * f_scale, 1.92 * f_scale),
        FlameSpan::new("Painter::rrect", 2, 1.60 * f_scale, 0.84 * f_scale),
        FlameSpan::new("Painter::text", 2, 2.48 * f_scale, 0.94 * f_scale),
        FlameSpan::new("wgpu::submit", 1, 3.55 * f_scale, 1.18 * f_scale),
    ];

    let spans = [
        TraceSpan::new("POST /v1/inference", "edge-proxy", 0.0, 142.0 * t_scale, 0),
        TraceSpan::new("verify_mtls_jwt", "auth-svc", 3.0 * t_scale, 13.8 * t_scale, 1),
        TraceSpan::new("pg.fetch_context", "postgres-eu", 17.5 * t_scale, 26.5 * t_scale, 1),
        TraceSpan::new("kv_cache.lookup", "redis-l1", 20.5 * t_scale, 6.8 * t_scale, 2),
        TraceSpan::new("model.forward_stream", "gpu-pool", 45.5 * t_scale, 93.0 * t_scale, 1),
    ];

    let logs = [
        LogEntry::new(
            "09:14:01.10",
            LogLevel::Info,
            "wgpu::hal",
            "Adapter Apple M3 Max (Metal) acquired · single-pass SDF pipeline warm",
        ),
        LogEntry::new(
            "09:14:01.34",
            LogLevel::Info,
            "lumen::atlas",
            "Glyph coverage atlas synced: 312 subpixel glyphs in 1.2 ms",
        ),
        LogEntry::new(
            "09:14:02.18",
            LogLevel::Warn,
            "edge::tls",
            "OCSP staple in ap-northeast-1 expires in 18m; refreshing ticket",
        ),
        LogEntry::new(
            "09:14:03.05",
            LogLevel::Info,
            "edge::router",
            "Matched POST /v1/inference -> eu-central-1 (4.1 ms, cache HIT)",
        ),
        LogEntry::new(
            "09:14:04.62",
            LogLevel::Error,
            "worker::shard",
            "Replica gpu-03 heartbeat timed out (1500 ms); draining active streams",
        ),
        LogEntry::new(
            "09:14:05.09",
            LogLevel::Info,
            "worker::shard",
            "Replica gpu-03 rejoined cluster cleanly · zero dropped frames",
        ),
    ];

    ui.column().fill().pad(16).gap(12).show(|ui| {
        // Top header bar
        ui.row().fill_width().align(Align::Center).gap(12).show(|ui| {
            ui.heading("Edge Telemetry & Frame Profiler");
            ui.status("Healthy · 14 PoPs", Tone::Success);
            ui.spacer();
            ui.segmented(
                &mut s.range,
                &[(0usize, "15m"), (1usize, "1h"), (2usize, "24h")],
            );
            if ui
                .button("Capture trace")
                .primary()
                .small()
                .icon("activity")
                .clicked()
            {
                s.captures += 1;
                ui.toast(
                    Toast::new(format!(
                        "Captured trace #{} ({} window)",
                        1040 + s.captures,
                        window_lbl
                    ))
                    .success(),
                );
            }
        });

        // Executive KPI strip
        ui.kpi_strip("obs_kpis", &metrics);

        // Middle 2-column profiling & distributed trace cards
        ui.row()
            .fill_width()
            .gap(12)
            .align(Align::Start)
            .show(|ui| {
                ui.column().fill_width().show(|ui| {
                    ui.card().pad(12.0).show(|ui| {
                        ui.column().fill_width().gap(8).show(|ui| {
                            ui.row().fill_width().align(Align::Center).gap(6).show(|ui| {
                                let flame_c = ui.color(Token::AccentStrong);
                                ui.icon_leaf("flame", 14.0, flame_c);
                                ui.label("CPU / GPU Frame Profile").strong();
                                ui.spacer();
                                ui.badge(format!("{:.2} ms", 4.80 * f_scale), Tone::Accent);
                            });
                            ui.flame_graph("obs_flame", &frames);
                        });
                    });
                });

                ui.column().fill_width().show(|ui| {
                    ui.card().pad(12.0).show(|ui| {
                        ui.column().fill_width().gap(8).show(|ui| {
                            ui.row().fill_width().align(Align::Center).gap(6).show(|ui| {
                                let trace_c = ui.color(Token::Info);
                                ui.icon_leaf("zap", 14.0, trace_c);
                                ui.label("Distributed Request Waterfall").strong();
                                ui.spacer();
                                ui.badge(format!("{:.1} ms", 142.0 * t_scale), Tone::Neutral);
                            });
                            ui.waterfall("obs_trace", &spans);
                        });
                    });
                });
            });

        // Bottom live structured log stream
        ui.log_viewer("obs_logs", &logs).height(138.0).show();
    });
}
