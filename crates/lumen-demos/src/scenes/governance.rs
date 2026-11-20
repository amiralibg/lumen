//! Cloud Governance & FinOps showcase composition.

use lumen::{Align, Currency, Toast, Tone, TreemapItem, Ui, WaffleSlice};

pub struct State {
    pub fy_year: i32,
    pub fy_quarter: u8,
    pub budget_usd: f64,
    pub currency: Currency,
    pub rbac_grid: Vec<Vec<bool>>,
    pub avail_scopes: Vec<String>,
    pub assigned_scopes: Vec<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            fy_year: 2026,
            fy_quarter: 2,
            budget_usd: 500_000.0,
            currency: Currency::Usd,
            rbac_grid: vec![
                vec![true, true, true, true],
                vec![true, true, true, false],
                vec![true, false, false, false],
                vec![false, false, false, true],
            ],
            avail_scopes: vec![
                "billing:override".into(),
                "kms:destroy-key".into(),
                "org:saml-admin".into(),
                "runners:drain".into(),
            ],
            assigned_scopes: vec![
                "deploys:promote".into(),
                "secrets:rotate".into(),
                "metrics:export".into(),
            ],
        }
    }
}

pub fn view(ui: &mut Ui, s: &mut State) {
    let roles = ["Admin", "Maintainer", "Dev", "Auditor"];
    let caps = ["deploys:promote", "secrets:rotate", "runners:scale", "audit:export"];
    let spend = [
        TreemapItem::new("GPU Inference", 184.0).detail("eu-central · A100"),
        TreemapItem::new("Postgres HA", 112.0).detail("Multi-AZ NVMe"),
        TreemapItem::new("Edge Egress", 86.0).detail("14 global PoPs"),
        TreemapItem::new("Build Runners", 62.0).detail("ARM64 fleet"),
        TreemapItem::new("Log Archive", 38.0).detail("Cold storage"),
    ];
    let slices = [
        WaffleSlice::new("mTLS Enforced", 68, Tone::Success),
        WaffleSlice::new("Hardware KMS", 19, Tone::Accent),
        WaffleSlice::new("Short-lived Certs", 9, Tone::Info),
        WaffleSlice::new("Pending Audit", 4, Tone::Warning),
    ];

    ui.column().fill().pad(16).gap(12).show(|ui| {
        ui.row().fill_width().gap(12).align(Align::Center).show(|ui| {
            ui.quarter_picker("gov_fy", &mut s.fy_year, &mut s.fy_quarter);
            ui.spacer();
            ui.column().width(250.0).show(|ui| {
                ui.money_field(&mut s.budget_usd)
                    .label("Quarterly cloud cap")
                    .switchable_currency(&mut s.currency)
                    .step(500.0)
                    .fill_width();
            });
            if ui.button("Export SOC2").primary().icon("shield-check").clicked() {
                ui.toast(Toast::new("SOC2 Type II evidence bundle exported").success());
            }
        });

        ui.row().fill_width().gap(12).align(Align::Start).show(|ui| {
            ui.column().fill_width().show(|ui| {
                ui.permission_matrix("gov_rbac", &roles, &caps, &mut s.rbac_grid);
            });
            ui.column().width(380.0).show(|ui| {
                ui.card()
                    .title("Q2 Infrastructure Spend ($k)")
                    .pad(12.0)
                    .show(|ui| {
                        ui.treemap("gov_spend", &spend).unit("$k").height(174.0).show();
                    });
            });
        });

        ui.row().fill_width().gap(12).align(Align::Start).show(|ui| {
            ui.column().fill_width().show(|ui| {
                ui.transfer_list("gov_scopes", &mut s.avail_scopes, &mut s.assigned_scopes)
                    .titles("Available IAM scopes", "Granted to Maintainer")
                    .height(144.0);
            });
            ui.column().width(380.0).show(|ui| {
                ui.waffle_chart("gov_compliance", "Fleet Zero-Trust Compliance", &slices);
            });
        });
    });
}
