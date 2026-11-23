//! Tally: personal finance. Accounts, a cash-flow chart, budgets that fill
//! as you spend, a transaction list and an add-transaction dialog.

use super::Fit;
use lumen::{Align, Date, ListRow, Series, Toast, Tone, Ui};

const CATEGORIES: [(&str, &str, f64); 5] = [("Groceries", "shopping-basket", 600.0), ("Dining", "utensils", 250.0), ("Transport", "train-front", 180.0), ("Home", "house", 1400.0), ("Fun", "ticket", 200.0)];

struct Txn {
    payee: String,
    category: usize,
    amount: f64,
    date: Date,
}

pub struct State {
    account: usize,
    txns: Vec<Txn>,
    selected: Option<usize>,
    range: u8,
    add: bool,
    payee: String,
    amount: f64,
    category: usize,
    date: Date,
}

impl Default for State {
    fn default() -> Self {
        let today = Date::today();
        let t = |payee: &str, category: usize, amount: f64, days: i64| Txn { payee: payee.into(), category, amount, date: today.add_days(-days) };
        State {
            account: 0,
            txns: vec![
                t("Corner Market", 0, -42.18, 0),
                t("Rail pass", 2, -64.00, 1),
                t("Salary", 3, 4200.00, 2),
                t("Noodle Bar", 1, -23.50, 2),
                t("Rent", 3, -1350.00, 3),
                t("Cinema", 4, -28.00, 4),
                t("Farmers market", 0, -31.40, 5),
                t("Bistro Vert", 1, -71.20, 6),
                t("Hardware store", 3, -19.99, 8),
                t("Corner Market", 0, -56.75, 9),
            ],
            selected: None,
            range: 1,
            add: false,
            payee: String::new(),
            amount: 25.0,
            category: 0,
            date: today,
        }
    }
}

fn money(v: f64) -> String {
    let sign = if v < 0.0 { "−" } else { "" };
    let a = v.abs();
    let whole = a.trunc() as u64;
    let s = whole.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    format!("{sign}${out}.{:02}", ((a - a.trunc()) * 100.0).round() as u64 % 100)
}

pub fn view(ui: &mut Ui, s: &mut State) {
    let balance = 12840.55 + s.txns.iter().map(|t| t.amount).sum::<f64>();
    let spent: f64 = s.txns.iter().filter(|t| t.amount < 0.0).map(|t| -t.amount).sum();
    let income: f64 = s.txns.iter().filter(|t| t.amount > 0.0).map(|t| t.amount).sum();

    let fit = Fit::of(ui);
    ui.row().fill().show(|ui| {
        ui.sidebar().width(212.0).collapsed(fit.rail()).show(|ui| {
            ui.nav_header("T", "Tally");
            ui.nav_section("Accounts");
            for (i, (icon, name)) in [("wallet", "Everyday"), ("piggy-bank", "Savings"), ("credit-card", "Credit card")].iter().enumerate() {
                if ui.nav_item(icon, *name).selected(s.account == i).clicked() {
                    s.account = i;
                }
            }
            ui.nav_section("Plan");
            ui.nav_item("target", "Goals");
            ui.nav_item("repeat", "Subscriptions").badge(6);
        });

        ui.column().fill().show(|ui| {
            let pad = if fit == Fit::Narrow { 16.0 } else { 20.0 };
            ui.scroll("finance").pad(pad).gap(16.0).show(|ui| {
                let ranges = [(0, "Week"), (1, "Month"), (2, "Year")];
                ui.row().fill_width().gap(10).align(Align::Center).show(|ui| {
                    ui.heading("Everyday").level(2);
                    ui.spacer();
                    if fit == Fit::Narrow {
                        if ui.icon_button("plus").primary().tooltip("Add transaction").clicked() {
                            s.add = true;
                        }
                    } else {
                        ui.segmented(&mut s.range, &ranges);
                        if ui.button("Add transaction").primary().icon("plus").clicked() {
                            s.add = true;
                        }
                    }
                });
                if fit == Fit::Narrow {
                    ui.segmented(&mut s.range, &ranges);
                }
                // Stat cards side by side, stacked on a phone.
                let stats = |ui: &mut Ui| {
                    ui.card().pad(16.0).show(|ui| ui.stat("Balance", &money(balance), Some(3.2), Some(&[11.2, 11.6, 11.4, 12.1, 12.0, 12.6, 12.4, 12.9])));
                    ui.card().pad(16.0).show(|ui| ui.stat("Spent this month", &money(spent), Some(-8.4), Some(&[2.4, 2.1, 2.6, 2.2, 1.9, 2.0, 1.8, 1.7])));
                    ui.card().pad(16.0).show(|ui| ui.stat("Income", &money(income), Some(0.0), Some(&[4.2, 4.2, 4.2, 4.2, 4.2, 4.2, 4.2, 4.2])));
                };
                if fit == Fit::Narrow {
                    ui.column().fill_width().gap(12).show(stats);
                } else {
                    ui.row().fill_width().gap(14).show(stats);
                }
                // Chart and budgets side by side when wide, stacked otherwise.
                let stacked = fit != Fit::Wide;
                let pair = if stacked { ui.column().fill_width().gap(14) } else { ui.row().fill_width().gap(14).align(Align::Start) };
                pair.show(|ui| {
                    ui.column().fill_width().show(|ui| {
                        ui.card().title("Cash flow").show(|ui| {
                            let labels = ["Jun", "Jul", "Aug", "Sep", "Oct", "Nov"];
                            ui.bar_chart("cash", &labels, vec![Series::new("In", vec![4.2, 4.2, 4.6, 4.2, 4.2, 4.2]), Series::new("Out", vec![3.1, 3.6, 2.9, 3.3, 3.8, (spent / 1000.0) as f32])]).height(200.0);
                        });
                    });
                    let side = if stacked { ui.column().fill_width() } else { ui.column().width(320.0) };
                    side.show(|ui| {
                        ui.card().title("Budgets").description("This month").show(|ui| {
                            for (ci, (name, icon, limit)) in CATEGORIES.iter().enumerate() {
                                let used: f64 = s.txns.iter().filter(|t| t.category == ci && t.amount < 0.0).map(|t| -t.amount).sum();
                                let frac = (used / limit) as f32;
                                ui.column().fill_width().gap(6).show(|ui| {
                                    ui.row().fill_width().gap(8).align(Align::Center).show(|ui| {
                                        ui.icon_leaf(icon, 14.0, ui.color(lumen::Token::TextSecondary));
                                        ui.label(*name);
                                        ui.spacer();
                                        ui.label(format!("{} of {}", money(used), money(*limit))).muted().small().tabular();
                                    });
                                    ui.progress(Some(frac.min(1.0)));
                                });
                            }
                        });
                    });
                });
                ui.card().title("Transactions").show(|ui| {
                    let rows: Vec<ListRow> = s
                        .txns
                        .iter()
                        .map(|t| {
                            let (cat, icon, _) = CATEGORIES[t.category];
                            let mut r = ListRow::new(t.payee.clone()).subtitle(format!("{cat} · {}", t.date.short())).icon(icon).meta(money(t.amount));
                            if t.amount > 0.0 {
                                r = r.badge("Income", Tone::Success);
                            }
                            r
                        })
                        .collect();
                    ui.list_view("txns", &rows, &mut s.selected);
                });
            });
        });
    });

    let mut open = s.add;
    let mut save = false;
    ui.dialog("add_txn", &mut open)
        .title("Add transaction")
        .width(460.0)
        .footer(|ui| {
            ui.spacer();
            ui.button("Cancel").ghost();
            save = ui.button("Add").primary().clicked();
        })
        .show(|ui| {
            ui.text_input(&mut s.payee).label("Payee").placeholder("Where did the money go?").fill_width();
            ui.money_field(&mut s.amount).label("Amount").fill_width();
            let cats: Vec<(usize, &str)> = CATEGORIES.iter().enumerate().map(|(i, c)| (i, c.0)).collect();
            ui.select(&mut s.category, &cats).label("Category").fill_width();
            ui.date_picker("Date", &mut s.date);
        });
    if save {
        let payee = if s.payee.trim().is_empty() { CATEGORIES[s.category].0.to_string() } else { std::mem::take(&mut s.payee) };
        s.txns.insert(0, Txn { payee: payee.clone(), category: s.category, amount: -s.amount.abs(), date: s.date });
        ui.toast(Toast::new("Transaction added").description(format!("{payee} · {}", money(-s.amount.abs()))).success());
        open = false;
    }
    s.add = open;
}
