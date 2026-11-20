//! Cadence: a music player. Library and playlists, a now-playing panel
//! with a scrubbable waveform, transport, volume, an equalizer and a queue.

use lumen::{Align, ListRow, Toast, Tone, Ui};

struct Track {
    title: &'static str,
    artist: &'static str,
    album: &'static str,
    secs: u32,
    liked: bool,
    seed: u32,
}

pub struct State {
    view: u8,
    tracks: Vec<Track>,
    selected: Option<usize>,
    current: usize,
    playing: bool,
    progress: f32,
    volume: f32,
    shuffle: bool,
    repeat: bool,
    eq: [f32; 8],
    eq_preset: usize,
    search: String,
}

impl Default for State {
    fn default() -> Self {
        let t = |title, artist, album, secs, liked, seed| Track { title, artist, album, secs, liked, seed };
        State {
            view: 0,
            tracks: vec![
                t("Tidewater", "Halcyon Drift", "Low Light", 214, true, 3),
                t("Glass Orchard", "Mirela Voss", "Paper Suns", 251, false, 7),
                t("Northbound", "The Quiet Engines", "Signal Fires", 198, true, 11),
                t("Cinder & Salt", "Halcyon Drift", "Low Light", 236, false, 5),
                t("Slow Weather", "Okapi Choir", "Rooms", 302, false, 9),
                t("Afterimage", "Mirela Voss", "Paper Suns", 189, true, 2),
                t("Lantern Season", "June Atlas", "Small Hours", 227, false, 13),
                t("Fieldwork", "The Quiet Engines", "Signal Fires", 264, false, 17),
                t("Halfway Blue", "Okapi Choir", "Rooms", 243, false, 19),
            ],
            selected: Some(0),
            current: 0,
            playing: false,
            progress: 0.34,
            volume: 0.7,
            shuffle: false,
            repeat: true,
            eq: [1.5, 2.5, 1.0, -0.5, -1.0, 0.5, 2.0, 3.0],
            eq_preset: 2,
            search: String::new(),
        }
    }
}

fn mmss(secs: f32) -> String {
    let s = secs.max(0.0).round() as u32;
    format!("{}:{:02}", s / 60, s % 60)
}

/// A deterministic waveform for a track, shaped like a song: quiet intro,
/// a build, a loud chorus and a fade.
fn samples(seed: u32) -> Vec<f32> {
    (0..96)
        .map(|i| {
            let x = i as f32 / 95.0;
            let env = (x * 3.2).min(1.0) * (1.0 - ((x - 0.85).max(0.0) * 5.0)).max(0.15);
            let chorus = if (0.42..0.7).contains(&x) { 1.0 } else { 0.72 };
            let n = ((i as u32).wrapping_mul(2654435761).wrapping_add(seed.wrapping_mul(40503)) >> 16) as f32 / 65535.0;
            (0.18 + 0.82 * env * chorus * (0.55 + 0.45 * n)).clamp(0.05, 1.0)
        })
        .collect()
}

pub fn view(ui: &mut Ui, s: &mut State) {
    ui.row().fill().show(|ui| {
        ui.sidebar().width(212.0).show(|ui| {
            ui.nav_header("C", "Cadence");
            ui.nav_section("Library");
            for (i, (icon, l)) in [("music", "Songs"), ("disc-3", "Albums"), ("mic-vocal", "Artists"), ("heart", "Liked")].iter().enumerate() {
                if ui.nav_item(icon, *l).selected(s.view == i as u8).clicked() {
                    s.view = i as u8;
                    s.selected = None;
                }
            }
            ui.nav_section("Playlists");
            for (icon, l) in [("sunrise", "Morning focus"), ("moon", "Late drive"), ("coffee", "Café radio")] {
                ui.nav_item(icon, l);
            }
        });

        ui.column().fill().pad(20).gap(16).show(|ui| {
            library(ui, s);
        });

        ui.column().width(360.0).fill_height().bg(lumen::Token::Surface).show(|ui| {
            ui.scroll("now_playing").pad(20).gap(16.0).show(|ui| now_playing(ui, s));
        });
    });
}

fn library(ui: &mut Ui, s: &mut State) {
    let title = ["Songs", "Albums", "Artists", "Liked"][s.view as usize];
    ui.row().fill_width().align(Align::Center).gap(10).show(|ui| {
        ui.heading(title).level(2);
        ui.badge(format!("{} tracks", s.tracks.len()), Tone::Neutral);
        ui.spacer();
        ui.search_field(&mut s.search).placeholder("Search songs").width(240.0);
        if ui.button("Shuffle all").icon("shuffle").clicked() {
            s.shuffle = true;
            s.current = (s.current + 4) % s.tracks.len();
            s.progress = 0.0;
            s.playing = true;
        }
    });
    let q = s.search.to_lowercase();
    let visible: Vec<usize> = (0..s.tracks.len())
        .filter(|&i| s.view != 3 || s.tracks[i].liked)
        .filter(|&i| q.is_empty() || s.tracks[i].title.to_lowercase().contains(&q) || s.tracks[i].artist.to_lowercase().contains(&q))
        .collect();
    let rows: Vec<ListRow> = visible
        .iter()
        .map(|&i| {
            let t = &s.tracks[i];
            let playing_here = i == s.current;
            let mut r = ListRow::new(t.title).subtitle(format!("{} · {}", t.artist, t.album)).meta(mmss(t.secs as f32)).starred(t.liked);
            r = r.icon(if playing_here && s.playing { "audio-lines" } else if playing_here { "pause" } else { "music-2" });
            if playing_here {
                r = r.badge(if s.playing { "Playing" } else { "Paused" }, Tone::Accent);
            }
            r
        })
        .collect();
    let mut sel = s.selected.and_then(|i| visible.iter().position(|&v| v == i));
    if visible.is_empty() {
        ui.empty_state("search-x", "No songs found").description("Nothing in your library matches that search.").show();
        return;
    }
    ui.scroll("tracks").show(|ui| {
        let out = ui.list_view("tracks", &rows, &mut sel);
        if let Some(i) = out.activated {
            s.current = visible[i];
            s.progress = 0.0;
            s.playing = true;
        }
        if let Some(i) = out.starred {
            let t = &mut s.tracks[visible[i]];
            t.liked = !t.liked;
            let msg = if t.liked { "Added to Liked" } else { "Removed from Liked" };
            ui.toast(Toast::new(msg).description(t.title));
        }
    });
    s.selected = sel.map(|i| visible[i]);
    ui.label("Double-click or press Enter to play.").muted().small();
}

fn now_playing(ui: &mut Ui, s: &mut State) {
    let n = s.tracks.len();
    let t = &s.tracks[s.current];
    let (title, artist, album, secs, seed) = (t.title, t.artist, t.album, t.secs as f32, t.seed);
    ui.label("Now playing").muted().small();
    ui.image_frame("cover", album).caption(artist).height(190.0).show();
    ui.column().gap(2).fill_width().show(|ui| {
        ui.heading(title).level(3);
        ui.label(format!("{artist} · {album}")).muted();
    });
    let wave = samples(seed);
    let was_playing = s.playing;
    ui.waveform("wave", &wave, &mut s.progress, &mut s.playing).duration(secs).show();
    if was_playing && !s.playing && s.progress == 0.0 && s.repeat {
        // Finished: move on to the next song.
        s.current = (s.current + 1) % n;
        s.playing = true;
    }
    ui.row().fill_width().align(Align::Center).gap(6).show(|ui| {
        ui.toolbar("transport_l", |tb| {
            tb.toggle("shuffle", "Shuffle", &mut s.shuffle);
        });
        ui.spacer();
        if ui.icon_button("skip-back").tooltip("Previous").clicked() {
            if s.progress * secs > 3.0 {
                s.progress = 0.0;
            } else {
                s.current = (s.current + n - 1) % n;
                s.progress = 0.0;
            }
        }
        let play = if s.playing { "pause" } else { "play" };
        if ui.icon_button(play).primary().large().tooltip(if s.playing { "Pause" } else { "Play" }).clicked() {
            s.playing = !s.playing;
        }
        if ui.icon_button("skip-forward").tooltip("Next").clicked() {
            s.current = (s.current + 1) % n;
            s.progress = 0.0;
        }
        ui.spacer();
        ui.toolbar("transport_r", |tb| {
            tb.toggle("repeat", "Repeat", &mut s.repeat);
        });
    });
    let readout = format!("{:.0}%", s.volume * 100.0);
    ui.slider("Volume", &mut s.volume, 0.0..=1.0).readout(readout).width(f32::INFINITY);
    ui.graphic_eq("eq", &mut s.eq, &mut s.eq_preset);
    ui.label(format!("Up next: {}", s.tracks[(s.current + 1) % n].title)).muted().small();
}
