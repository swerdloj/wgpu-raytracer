use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text};

use crate::system::WGPU;

pub struct TextRenderer {
    font: ab_glyph::FontArc,
    // glyph_brush: wgpu_glyph::GlyphBrush,
    // ...
    // TODO: Implement
}

// TODO: The font and glyph brush should be stored for future uses.
//       This will likely mean creating a new struct for text rendering, and storing a map of fonts for re-use
pub fn render_text(wgpu: &mut WGPU, view: &wgpu::TextureView, width: u32, height: u32, text: &str) {
    let font = ab_glyph::FontArc::try_from_slice(include_bytes!("../res/font.ttf"))
        .unwrap();

    let mut glyph_brush = GlyphBrushBuilder::using_font(font)
        .build(&wgpu.device, wgpu::TextureFormat::Bgra8Unorm);

    let section = Section {
        screen_position: (10.0, 10.0),
        text: vec![Text::new(text).with_scale(25.0)],
        ..Section::default()
    };

    glyph_brush.queue(section);

    let mut encoder = wgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("text_encoder"),
    });

    glyph_brush.draw_queued(
        &wgpu.device,
        &mut encoder,
        view,
        width,
        height,
    ).unwrap();

    wgpu.queue.submit(&[encoder.finish()]);
}

/*
let font = ab_glyph::FontArc::try_from_slice(include_bytes!("SomeFont.ttf"))
    .expect("Load font");

let mut glyph_brush = GlyphBrushBuilder::using_font(font)
    .build(&device, render_format);

let section = Section {
    screen_position: (10.0, 10.0),
    text: vec![Text::new("Hello wgpu_glyph")],
    ..Section::default()
};

glyph_brush.queue(section);

glyph_brush.draw_queued(
    &device,
    &mut encoder,
    &frame.view,
    frame.width,
    frame.height,
);

device.get_queue().submit(&[encoder.finish()]);
*/