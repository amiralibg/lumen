//! A GPUI window with a native GPUI sidebar and a lumen dashboard panel.
//! State flows both ways: the GPUI sidebar picks the region lumen shows,
//! and lumen's controls update the GPUI header.
//!
//! cargo run --example dashboard   (from crates/lumen-gpui)


use gpui::{
    div, prelude::*, px, rgb, size, App, Bounds, Context, Entity, SharedString, Window, WindowBounds,
    WindowOptions,
};
use lumen_gpui::lumen_core::widgets::Series;
use lumen_gpui::lumen_core::{Align, Look, Toast, Tone, Ui};
use lumen_gpui::LumenView;

const REGIONS: [&str; 4] = ["Frankfurt", "Virginia", "Tokyo", "Sydney"];
const MONTHS: [&str; 8] = ["Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov"];

struct Dash {
    region: usize,
    range: u8,
    autoscale: bool,
    budget: f32,
    look: u8,
    deploys: u32,
    service: String,
}

fn dashboard(ui: &mut Ui, s: &mut Dash) {
    let looks = [Look::lumen(), Look::mac(), Look::windows(), Look::gnome()];
    ui.set_look(looks[s.look as usize]);
    ui.column().fill().pad(20).gap(16).show(|ui| {
        ui.row().fill_width().gap(12).align(Align::Center).show(|ui| {
            ui.heading(REGIONS[s.region]);
            ui.badge("Healthy", Tone::Success);
            ui.spacer();
            ui.segmented(&mut s.range, &[(0, "24h"), (1, "7d"), (2, "30d")]);
        });
        ui.row().fill_width().gap(12).show(|ui| {
            let k = 1.0 + s.region as f32 * 0.17 + s.range as f32 * 0.4;
            let stats = [
                ("Requests", format!("{:.1}M", 12.4 * k), 8.2, [3.0, 4.0, 3.6, 5.0, 5.4, 6.1, 6.0, 7.2]),
                ("p95 latency", format!("{:.0} ms", 84.0 / k), -3.1, [7.0, 6.4, 6.6, 5.8, 5.1, 5.3, 4.6, 4.2]),
                ("Deploys", s.deploys.to_string(), 12.0, [2.0, 2.0, 3.0, 2.5, 3.5, 4.0, 4.4, 5.0]),
            ];
            for (label, value, delta, trend) in stats {
                ui.card().pad(16.0).show(|ui| ui.stat(label, &value, Some(delta), Some(&trend)));
            }
        });
        ui.card().title("Traffic").show(|ui| {
            let base = 40.0 + s.region as f32 * 12.0;
            let a: Vec<f32> = (0..8).map(|i| base + (i as f32 * 0.9).sin() * 14.0 + i as f32 * 6.0).collect();
            let b: Vec<f32> = a.iter().map(|v| v * 0.55).collect();
            ui.line_chart("gpui-traffic", &MONTHS, vec![Series::new("Edge", a), Series::new("Origin", b)]).smooth(true).area().height(180.0);
        });
        ui.card().title("Scaling").show(|ui| {
            ui.text_input(&mut s.service).label("Service").fill_width();
            ui.switch("Autoscale replicas", &mut s.autoscale);
            let readout = format!("${:.0}k", s.budget * 40.0);
            ui.slider("Monthly budget", &mut s.budget, 0.0..=1.0).readout(readout).width(f32::INFINITY);
            ui.row().fill_width().gap(8).show(|ui| {
                ui.segmented(&mut s.look, &[(0, "Lumen"), (1, "macOS"), (2, "Windows"), (3, "GNOME")]);
                ui.spacer();
                if ui.button("Deploy").primary().icon("rocket").clicked() {
                    s.deploys += 1;
                    ui.toast(Toast::new("Deploy started").description(format!("Rolling {} out to {}.", s.service, REGIONS[s.region])).success());
                }
            });
        });
    });
}

struct Shell {
    panel: Entity<LumenView<Dash>>,
    _watch: gpui::Subscription,
}

impl Render for Shell {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let s = self.panel.read(cx).state();
        let (selected, deploys) = (s.region, s.deploys);
        let sidebar = div().w(px(200.)).h_full().flex().flex_col().gap_1().p_3().bg(rgb(0x0f1115)).children(
            REGIONS.iter().enumerate().map(|(i, name)| {
                let panel = self.panel.clone();
                div()
                    .id(i)
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .text_color(if i == selected { rgb(0xffffff) } else { rgb(0x9aa3b2) })
                    .when(i == selected, |d| d.bg(rgb(0x1f2a44)))
                    .hover(|d| d.bg(rgb(0x1a1f2b)))
                    .cursor_pointer()
                    .child(SharedString::from(*name))
                    .on_click(move |_, _, cx| panel.update(cx, |p, cx| p.update(cx, |s| s.region = i)))
            }),
        );
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x0b0d12))
            .text_color(rgb(0xe6e9ef))
            .child(
                div()
                    .h(px(44.))
                    .px_4()
                    .flex()
                    .items_center()
                    .justify_between()
                    .border_b_1()
                    .border_color(rgb(0x1c2130))
                    .child("GPUI shell")
                    .child(SharedString::from(format!("{deploys} deploys this session"))),
            )
            .child(div().flex_1().flex().child(sidebar).child(div().flex_1().h_full().child(self.panel.clone())))
    }
}

fn main() {
    lumen_gpui::application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1080.), px(720.)), cx);
        cx.open_window(WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() }, |window, cx| {
            let state = Dash { region: 0, range: 1, autoscale: true, budget: 0.6, look: 0, deploys: 1284, service: "edge-gateway".into() };
            let panel = LumenView::build(state, dashboard, window, cx);
            cx.new(|cx| {
                let watch = cx.observe(&panel, |_, _, cx| cx.notify());
                Shell { panel, _watch: watch }
            })
        })
        .unwrap();
        cx.activate(true);
    });
}
