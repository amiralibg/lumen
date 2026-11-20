//! Global On-Call & Launch Room showcase composition.

use lumen::{
    Align, DateStripDay, GanttTask, NotificationItem, ScheduleEvent, Tone, Ui, WorldClock,
};

pub struct State {
    pub strip_day: usize,
    pub notifs: Vec<NotificationItem>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            strip_day: 2,
            notifs: vec![
                NotificationItem::new(
                    "n1",
                    "LF",
                    "Canary promoted in eu-central",
                    "Zero 5xx burn across 14 PoPs.",
                    "4m ago",
                    true,
                ),
                NotificationItem::new(
                    "n2",
                    "JB",
                    "On-call handoff acknowledged",
                    "Berlin primary rotation active until 01:00 UTC.",
                    "18m ago",
                    false,
                ),
            ],
        }
    }
}

pub fn view(ui: &mut Ui, s: &mut State) {
    let clocks = [
        WorldClock::new("San Francisco", "SFO", "08:30", "UTC-8", true),
        WorldClock::new("London", "LHR", "16:30", "UTC+0", true),
        WorldClock::new("Berlin", "BER", "17:30", "UTC+1", true),
        WorldClock::new("Tokyo", "HND", "01:30", "UTC+9", false),
    ];
    let days = [
        DateStripDay::new("Mon", 16, 1),
        DateStripDay::new("Tue", 17, 2),
        DateStripDay::new("Wed", 18, 3),
        DateStripDay::new("Thu", 19, 1),
        DateStripDay::new("Fri", 20, 2),
        DateStripDay::new("Sat", 21, 0),
        DateStripDay::new("Sun", 22, 0),
    ];
    let cal_days = ["Mon 16", "Tue 17", "Wed 18", "Thu 19"];
    let events = [
        ScheduleEvent::new(0, 9.5, 2.0, "WGSL shader audit", Tone::Accent).room("Lab A"),
        ScheduleEvent::new(1, 11.0, 2.0, "SLO burn review", Tone::Success).room("Ops"),
        ScheduleEvent::new(2, 9.5, 1.8, "WebGPU bundle cut", Tone::Info).room("Build"),
        ScheduleEvent::new(2, 13.8, 2.2, "Canary rollout", Tone::Warning).room("Release"),
        ScheduleEvent::new(3, 12.5, 2.0, "v0.5 RC sign-off", Tone::Accent).room("Bridge"),
    ];
    let weeks = ["W1", "W2", "W3", "W4", "W5", "W6"];
    let tasks = [
        GanttTask::new("Token & OKLab engine", 0.0, 1.8, 1.0)
            .owner("JB")
            .tone(Tone::Success),
        GanttTask::new("Single-shader wgpu pass", 1.0, 2.1, 1.0)
            .owner("LF")
            .tone(Tone::Success),
        GanttTask::new("Global PoP soak test", 2.5, 2.0, 0.84)
            .owner("PN")
            .tone(Tone::Accent),
        GanttTask::new("Launch runbook drill", 4.0, 1.4, 0.60)
            .owner("AK")
            .tone(Tone::Info),
        GanttTask::milestone("v0.5 RC freeze", 5.5),
    ];

    ui.column().fill().pad(16).gap(12).show(|ui| {
        ui.world_clocks("ops_clocks", &clocks);

        ui.row().fill_width().gap(12).align(Align::Start).show(|ui| {
            ui.column().fill_width().gap(12).show(|ui| {
                ui.date_strip("ops_strip", &days, &mut s.strip_day);
                ui.day_schedule("ops_cal", &cal_days, 9, 17, Some((2, 13.3)), &events, 232.0);
            });

            ui.column().width(410.0).gap(12).show(|ui| {
                ui.gantt_chart("ops_gantt", &weeks, &tasks);
                ui.notification_list("ops_inbox", &mut s.notifs);
            });
        });
    });
}
