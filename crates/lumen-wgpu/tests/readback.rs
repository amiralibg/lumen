//! Offscreen readback: the blocking path and the pipelined ring agree.

use lumen_core::paint::Painter;
use lumen_core::{vec2, Color, Rect};
use lumen_wgpu::offscreen::{self, AsyncReadback, Readback};
use lumen_wgpu::Renderer;

fn frame(fill: Color) -> lumen_core::paint::DrawData {
    let mut p = Painter::new(vec2(40.0, 30.0), 1.0, Color::WHITE);
    p.rect(Rect::from_xywh(0.0, 0.0, 20.0, 30.0), fill);
    p.finish()
}

#[test]
fn async_readback_returns_the_newest_frame() {
    let Some((device, queue)) = offscreen::headless_device() else {
        eprintln!("no wgpu adapter; skipping");
        return;
    };
    let format = offscreen::FORMAT;
    let mut renderer = Renderer::new(&device, format);
    let blocking = Readback::default().render(&device, &queue, &mut renderer, &frame(Color::BLACK)).to_vec();

    let mut rb = AsyncReadback::new(format, 2);
    assert_eq!(rb.submit(&device, &queue, &mut renderer, &frame(Color::BLACK)), Some(1));
    assert_eq!(rb.submit(&device, &queue, &mut renderer, &frame(Color::BLACK)), Some(2));
    assert!(rb.submit(&device, &queue, &mut renderer, &frame(Color::BLACK)).is_none(), "both slots are busy");
    let f = rb.wait(&device).expect("a finished frame");
    assert_eq!((f.size, f.seq), ([40, 30], 2));
    assert_eq!(f.pixels, &blocking[..]);
    assert!(rb.try_take(&device).is_none(), "nothing newer");
    // Odd widths are repacked from padded rows.
    let mut p = Painter::new(vec2(37.0, 5.0), 1.0, Color::BLACK);
    p.rect(Rect::from_xywh(0.0, 0.0, 1.0, 1.0), Color::WHITE);
    assert!(rb.submit(&device, &queue, &mut renderer, &p.finish()).is_some());
    let f = rb.wait(&device).unwrap();
    assert_eq!((f.size, f.pixels.len()), ([37, 5], 37 * 5 * 4));
    assert_eq!(&f.pixels[..4], &[255, 255, 255, 255]);
    assert_eq!(&f.pixels[4..8], &[0, 0, 0, 255]);
}
