//! Frame cost of 100k-row lists while scrolling (build, layout and
//! tessellation, no GPU).
//!
//! `cargo bench -p lumen-core --bench virtual_rows`

use std::time::{Duration, Instant};

use lumen_core::*;

fn rows(n: usize) -> Vec<ListRow> {
    (0..n)
        .map(|i| {
            let mut r = ListRow::new(format!("Deployed service #{i}")).key(i as u64).subtitle("eu-central").meta("09:12").icon("rocket");
            if i % 3 == 0 {
                r = r.preview("4 replicas healthy · p95 82 ms");
            }
            r
        })
        .collect()
}

fn stats(name: &str, mut t: Vec<Duration>) {
    t.sort();
    let ms = |d: Duration| d.as_secs_f64() * 1e3;
    let mean = t.iter().map(|d| ms(*d)).sum::<f64>() / t.len() as f64;
    println!(
        "{name:<44} mean {:>6.3} ms   p50 {:>6.3}   p95 {:>6.3}   max {:>6.3}   ({} frames)",
        mean,
        ms(t[t.len() / 2]),
        ms(t[t.len() * 95 / 100]),
        ms(*t.last().unwrap()),
        t.len()
    );
}

fn run(name: &str, frames: usize, view: impl FnMut(&mut Ui, usize)) {
    run_excl(name, frames, view, || Duration::ZERO);
}

/// `excluded()` returns time spent this frame on the caller's own data (not
/// lumen's work), subtracted from the frame time.
fn run_excl(name: &str, frames: usize, view: impl FnMut(&mut Ui, usize), excluded: impl Fn() -> Duration) {
    run_with(name, frames, false, view, excluded);
}

/// `a11y`: also build the accessibility tree each frame, as when a screen
/// reader is running.
fn run_with(name: &str, frames: usize, a11y: bool, mut view: impl FnMut(&mut Ui, usize), excluded: impl Fn() -> Duration) {
    let mut ui = Ui::new();
    ui.theme.reduced_motion = true;
    ui.set_accessibility(a11y);
    let mut t = 1.0;
    let mut times = Vec::with_capacity(frames);
    for f in 0..frames + 20 {
        t += 1.0 / 60.0;
        let events = if f > 2 { vec![Event::PointerMoved(vec2(400.0, 300.0)), Event::Scroll(vec2(0.0, 40.0))] } else { vec![] };
        let input = RawInput { screen_size: vec2(1200.0, 800.0), scale: 2.0, time: t, events, modifiers: Modifiers::default() };
        let t0 = Instant::now();
        let out = ui.run(input, |ui| view(ui, f));
        std::hint::black_box(&out);
        if f >= 20 {
            times.push(t0.elapsed().saturating_sub(excluded()));
        }
    }
    stats(name, times);
}

fn main() {
    let n = 100_000;
    let list = rows(n);
    let mut sel = None;
    let mut seen = (0usize, 0f32);
    run("list_view, 100k rows, scrolling 40 px/frame", 600, |ui, _| {
        let (o, info) = ui.scroll("s").show(|ui| ui.list_view("l", &list, &mut sel));
        seen = (o.built, info.offset);
    });
    println!("    (last frame: {} rows built, scrolled {:.0} px)", seen.0, seen.1);
    run_with("list_view, 100k rows, scrolling, accessibility tree on", 600, true, |ui, _| {
        ui.scroll("s").show(|ui| ui.list_view("l", &list, &mut sel));
    }, || Duration::ZERO);

    let names: Vec<String> = (0..n).map(|i| format!("service-{i}")).collect();
    run("table (lazy_rows), 100k rows, scrolling", 600, |ui, _| {
        ui.table("t")
            .columns([TableCol::new("Id").width(90.0), TableCol::new("Name"), TableCol::new("Latency").right()])
            .lazy_rows(n, |i| i, |i| vec![TableCell::mono(format!("#{i}")), TableCell::text(names[i].clone()), TableCell::muted(format!("{} ms", i % 97))])
            .height(700.0)
            .show();
    });

    let keys: Vec<u64> = (0..1_000_000).collect();
    run("virtual_rows, 1M keyed rows, scrolling", 600, |ui, _| {
        let k = |i: usize| keys[i];
        ui.scroll("s").show(|ui| {
            ui.virtual_rows("v", keys.len()).keys(&k).estimate(28.0).show(|ui, i| {
                ui.label(format!("Line {i}"));
            });
        });
    });

    // For contrast: the same kind of rows without virtualization.
    run("baseline: 10k labels in a plain column", 20, |ui, _| {
        ui.scroll("s").show(|ui| {
            for i in 0..10_000 {
                ui.label(format!("Line {i}"));
            }
        });
    });

    // Prepending 25 rows every 10th frame (re-indexing 100k keys).
    let mut grow = rows(n);
    let mut next = n as u64;
    let mut sel2 = None;
    let prep = std::cell::Cell::new(Duration::ZERO);
    run_excl("list_view, 100k rows, +25 above every 10 frames", 300, |ui, f| {
        prep.set(Duration::ZERO);
        if f % 10 == 0 {
            let tp = Instant::now();
            let mut older: Vec<ListRow> = (0..25).map(|k| ListRow::new(format!("Older #{}", next + k)).key(next + k)).collect();
            next += 25;
            older.append(&mut grow);
            grow = older;
            prep.set(tp.elapsed());
        }
        ui.scroll("s").show(|ui| {
            ui.list_view("l", &grow, &mut sel2);
        });
    }, || prep.get());
}
