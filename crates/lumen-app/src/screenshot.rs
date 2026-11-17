//! Headless rendering to PNG, for visual review and tests.

use std::path::Path;

use lumen_core::input::{Event, Key, Modifiers, PointerButton, RawInput};
use lumen_core::{vec2, Accent, Color, Look, Mode, Ui, Vec2};
use lumen_wgpu::{offscreen, Renderer};

/// A scripted input step applied before settling and capturing.
#[derive(Clone, Debug)]
pub enum ShotStep {
    /// Move the pointer (hover).
    Hover(Vec2),
    /// Press and hold the primary button.
    Press(Vec2),
    /// Full click.
    Click(Vec2),
    /// Press a key (with modifiers).
    Key(Key, Modifiers),
    /// Type text.
    Type(String),
    /// Scroll by `dy` logical px at the pointer.
    Scroll(Vec2, f32),
    /// Let `secs` of time pass (animations, tooltip delays).
    Wait(f32),
}

/// Screenshot options.
#[derive(Clone, Debug)]
pub struct Shot {
    pub size: (u32, u32),
    pub scale: f32,
    pub mode: Mode,
    pub accent: Accent,
    pub look: Look,
    /// Stand-in for the OS accent (for `Accent::Native`).
    pub native_accent: Option<Color>,
    pub steps: Vec<ShotStep>,
}

impl Default for Shot {
    fn default() -> Self {
        Shot { size: (1280, 860), scale: 2.0, mode: Mode::Light, accent: Accent::Blue, look: Look::lumen(), native_accent: None, steps: vec![] }
    }
}

/// Render `view` headlessly after running `shot.steps`, letting animations
/// settle, and write a PNG to `path`.
pub fn screenshot<S>(path: impl AsRef<Path>, shot: &Shot, state: &mut S, mut view: impl FnMut(&mut Ui, &mut S)) -> Result<(), String> {
    let (device, queue) = offscreen::headless_device().ok_or("no GPU adapter")?;
    let mut renderer = Renderer::new(&device, offscreen::FORMAT);
    let mut ui = Ui::new();
    ui.theme = lumen_core::theme::Theme::with_look(shot.mode, shot.accent, shot.look);
    ui.set_native_accent(shot.native_accent);
    ui.theme.settle();
    ui.caret_blink = false;
    let mut t = 1.0f64;
    let dt = 1.0 / 60.0;
    let screen = vec2(shot.size.0 as f32, shot.size.1 as f32);
    let mut frame = |ui: &mut Ui, events: Vec<Event>, t: &mut f64, state: &mut S| {
        *t += dt;
        let input = RawInput { screen_size: screen, scale: shot.scale, time: *t, events, modifiers: Modifiers { mac: true, ..Default::default() } };
        ui.begin_frame(input);
        view(ui, state);
        let out = ui.end_frame();
        // Keep the GPU atlas in sync every frame.
        renderer.prepare(&device, &queue, &out.draw);
        out
    };
    let settle = |ui: &mut Ui, frame: &mut dyn FnMut(&mut Ui, Vec<Event>, &mut f64, &mut S) -> lumen_core::FrameOutput, t: &mut f64, state: &mut S, secs: f32| {
        let n = (secs / dt as f32).ceil() as usize;
        for _ in 0..n.max(1) {
            frame(ui, vec![], t, state);
        }
    };
    settle(&mut ui, &mut frame, &mut t, state, 0.1);
    let mut pointer = Vec2::ZERO;
    for step in &shot.steps {
        let evs = match step {
            ShotStep::Hover(p) => {
                pointer = *p;
                vec![Event::PointerMoved(*p)]
            }
            ShotStep::Press(p) => {
                pointer = *p;
                vec![Event::PointerMoved(*p), Event::PointerButton { pos: *p, button: PointerButton::Primary, pressed: true }]
            }
            ShotStep::Click(p) => {
                pointer = *p;
                frame(&mut ui, vec![Event::PointerMoved(*p), Event::PointerButton { pos: *p, button: PointerButton::Primary, pressed: true }], &mut t, state);
                vec![Event::PointerButton { pos: *p, button: PointerButton::Primary, pressed: false }]
            }
            ShotStep::Key(k, m) => vec![Event::Key { key: *k, pressed: true, repeat: false, modifiers: *m }, Event::Key { key: *k, pressed: false, repeat: false, modifiers: *m }],
            ShotStep::Type(s) => vec![Event::Text(s.clone())],
            ShotStep::Scroll(p, dy) => {
                pointer = *p;
                vec![Event::PointerMoved(*p), Event::Scroll(vec2(0.0, *dy))]
            }
            ShotStep::Wait(s) => {
                settle(&mut ui, &mut frame, &mut t, state, *s);
                continue;
            }
        };
        frame(&mut ui, evs, &mut t, state);
        settle(&mut ui, &mut frame, &mut t, state, 0.05);
    }
    let _ = pointer;
    // Let springs and fades finish; timers (caret blink, scrollbar hide)
    // don't count, so the capture shows the state right after the steps.
    for _ in 0..120 {
        let out = frame(&mut ui, vec![], &mut t, state);
        if !out.animating {
            break;
        }
    }
    let out = frame(&mut ui, vec![], &mut t, state);
    let px = offscreen::capture(&device, &queue, &mut renderer, &out.draw);
    let [w, h] = out.draw.size_px;
    write_png(path.as_ref(), w, h, &px)
}

fn write_png(path: &Path, w: u32, h: u32, rgba: &[u8]) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut wr = enc.write_header().map_err(|e| e.to_string())?;
    wr.write_image_data(rgba).map_err(|e| e.to_string())?;
    Ok(())
}
