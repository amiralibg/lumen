//! Showcase scene: Financial & Analytics Terminal.

use lumen::{Align, BulletMetric, Candle, MetricRow, SankeyLink, TextRole, Toast, Tone, Ui};

pub struct State {
    pub portfolio_usd: i64,
    pub range: usize,
    pub rebalance_delta: i64,
    pub rebalance_step: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            portfolio_usd: 1_482_600,
            range: 0,
            rebalance_delta: 0,
            rebalance_step: 0,
        }
    }
}

fn base_nav_for_range(range: usize) -> i64 {
    match range {
        0 => 1_482_600,
        1 => 1_536_400,
        _ => 1_618_950,
    }
}

fn candles_for_range(range: usize) -> [Candle; 12] {
    match range {
        0 => [
            Candle::new("Nov 1", 142.0, 148.5, 139.0, 146.2),
            Candle::new("Nov 2", 146.2, 151.0, 144.5, 149.8),
            Candle::new("Nov 3", 149.8, 150.4, 143.0, 144.6),
            Candle::new("Nov 4", 144.6, 147.0, 140.2, 141.5),
            Candle::new("Nov 5", 141.5, 149.2, 141.0, 148.4),
            Candle::new("Nov 6", 148.4, 155.0, 147.2, 153.6),
            Candle::new("Nov 7", 153.6, 158.4, 151.8, 156.9),
            Candle::new("Nov 8", 156.9, 157.5, 150.2, 151.8),
            Candle::new("Nov 9", 151.8, 156.0, 149.5, 155.1),
            Candle::new("Nov 10", 155.1, 162.4, 154.0, 160.8),
            Candle::new("Nov 11", 160.8, 163.5, 157.0, 158.4),
            Candle::new("Nov 12", 158.4, 166.2, 158.0, 165.0),
        ],
        1 => [
            Candle::new("Oct 7", 129.0, 134.5, 127.2, 133.4),
            Candle::new("Oct 10", 133.4, 137.8, 131.0, 136.2),
            Candle::new("Oct 14", 136.2, 138.0, 130.5, 132.0),
            Candle::new("Oct 17", 132.0, 139.4, 131.5, 138.6),
            Candle::new("Oct 21", 138.6, 144.0, 137.0, 142.8),
            Candle::new("Oct 24", 142.8, 145.2, 138.4, 140.1),
            Candle::new("Oct 28", 140.1, 148.0, 139.5, 146.9),
            Candle::new("Oct 31", 146.9, 152.4, 145.0, 151.2),
            Candle::new("Nov 4", 151.2, 154.0, 147.8, 149.4),
            Candle::new("Nov 6", 149.4, 158.2, 148.9, 156.8),
            Candle::new("Nov 9", 156.8, 162.0, 155.0, 160.4),
            Candle::new("Nov 12", 160.4, 166.8, 159.2, 165.0),
        ],
        _ => [
            Candle::new("W1", 116.0, 122.4, 114.5, 120.8),
            Candle::new("W2", 120.8, 126.0, 119.0, 124.5),
            Candle::new("W3", 124.5, 125.8, 118.2, 119.6),
            Candle::new("W4", 119.6, 128.4, 119.0, 127.2),
            Candle::new("W5", 127.2, 134.0, 126.0, 132.8),
            Candle::new("W6", 132.8, 139.5, 131.4, 138.0),
            Candle::new("W7", 138.0, 141.0, 133.5, 135.4),
            Candle::new("W8", 135.4, 145.2, 134.8, 143.9),
            Candle::new("W9", 143.9, 151.6, 142.5, 149.8),
            Candle::new("W10", 149.8, 156.4, 148.0, 154.2),
            Candle::new("W11", 154.2, 161.0, 152.6, 159.6),
            Candle::new("W12", 159.6, 166.8, 158.0, 165.0),
        ],
    }
}

pub fn view(ui: &mut Ui, s: &mut State) {
    ui.column().fill().pad(16).gap(12).show(|ui| {
        // Top executive bar
        ui.row().fill_width().align(Align::Center).gap(10).show(|ui| {
            ui.badge("LUMN · NASDAQ", Tone::Accent);
            ui.rolling_number("nav_val", s.portfolio_usd)
                .prefix("$")
                .role(TextRole::Heading2)
                .show();
            ui.badge("+6.8% YTD", Tone::Success);
            ui.spacer();

            let prev_range = s.range;
            ui.segmented(&mut s.range, &[(0usize, "1W"), (1, "1M"), (2, "1Q")]);
            if s.range != prev_range {
                s.portfolio_usd = base_nav_for_range(s.range) + s.rebalance_delta;
            }

            if ui.button("Rebalance").primary().icon("refresh-cw").clicked() {
                s.rebalance_step = s.rebalance_step.wrapping_add(1);
                let step_delta = if s.rebalance_step % 2 == 1 { 14_850 } else { -6_200 };
                s.rebalance_delta += step_delta;
                s.portfolio_usd = base_nav_for_range(s.range) + s.rebalance_delta;
                ui.toast(Toast::new("Rebalanced portfolio to target risk weights").success());
            }
        });

        // Upper 2-column row: OHLC candlestick + equity watchlist
        let candles = candles_for_range(s.range);
        let rows = [
            MetricRow::new(
                "LUMN",
                "Cloud GPU Index",
                vec![142.0, 146.0, 144.0, 149.0, 155.0, 160.0, 165.0],
                "$165.00",
                8.4,
            ),
            MetricRow::new(
                "NVDA",
                "Tensor Silicon",
                vec![126.0, 129.0, 131.5, 130.0, 134.2, 136.8, 138.4],
                "$138.40",
                6.2,
            ),
            MetricRow::new(
                "ANET",
                "Optical Fabric",
                vec![89.0, 91.2, 90.4, 93.0, 94.5, 95.1, 96.2],
                "$96.20",
                4.9,
            ),
            MetricRow::new(
                "SNOW",
                "Data Warehouse",
                vec![118.0, 116.5, 117.2, 115.0, 114.4, 113.8, 112.9],
                "$112.90",
                -2.1,
            ),
        ];

        ui.row().fill_width().gap(12).align(Align::Start).show(|ui| {
            ui.column().fill_width().show(|ui| {
                ui.card().title("Price action · OHLC").pad(12.0).show(|ui| {
                    ui.candlestick_chart("fin_ohlc", &candles).height(188.0).show();
                });
            });
            ui.column().width(380.0).show(|ui| {
                ui.metric_table("fin_watch", &rows);
            });
        });

        // Lower 2-column row: ARR-to-FCF Sankey + operating bullet targets
        let nodes = [
            "Total ARR",
            "Gross Profit",
            "Cloud COGS",
            "Free Cash Flow",
            "R&D + GTM",
            "GPU Hosting",
        ];
        let links = [
            SankeyLink::new(0, 1, 116.0),
            SankeyLink::new(0, 2, 32.0),
            SankeyLink::new(1, 3, 54.0),
            SankeyLink::new(1, 4, 62.0),
            SankeyLink::new(2, 5, 32.0),
        ];
        let bullets = [
            BulletMetric::new("Gross margin", 78.4, 75.0, 100.0, "%"),
            BulletMetric::new("Net dollar retention", 128.0, 120.0, 150.0, "%"),
            BulletMetric::new("FCF conversion", 36.5, 32.0, 50.0, "%"),
        ];

        ui.row().fill_width().gap(12).align(Align::Start).show(|ui| {
            ui.column().fill_width().show(|ui| {
                ui.card().title("ARR to free cash flow ($M)").pad(12.0).show(|ui| {
                    ui.sankey_chart("fin_sankey", &nodes, &links, 172.0);
                });
            });
            ui.column().width(380.0).show(|ui| {
                ui.bullet_chart("fin_bullets", &bullets);
            });
        });
    });
}
