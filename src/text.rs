use ab_glyph::{point, Font, FontRef, FontVec, Glyph, Point, PxScale, PxScaleFont, ScaleFont};
use image::flat::NormalForm::ImagePacked;
use image::{DynamicImage, GenericImage, GrayImage, ImageBuffer, Luma, Pixel, Rgb, RgbImage, Rgba, Primitive};

use crate::sat::to_summed_area_table;
#[derive(Clone, Debug)]
pub struct GlyphData {
    pub glyphs: Vec<Glyph>,
    pub width: u32,
    pub height: u32,
}

pub fn text_to_glyphs(text: &str, font: &FontRef, scale: PxScale) -> GlyphData {
    let scaled_font = font.as_scaled(scale);

    let mut glyphs = Vec::new();
    layout_paragraph(scaled_font, point(0.0, 0.0), 9999.0, text, &mut glyphs);

    // work out the layout size
    let glyphs_height = scaled_font.height().ceil() as u32;
    let glyphs_width = {
        let min_x = glyphs.first().unwrap().position.x;
        let last_glyph = glyphs.last().unwrap();
        let max_x = last_glyph.position.x + scaled_font.h_advance(last_glyph.id);
        (max_x - min_x).ceil() as u32
    };

    GlyphData {
        glyphs,
        width: glyphs_width,
        height: glyphs_height,
    }
}

pub fn draw_glyphs_to_rgb_buffer(
    buffer: &mut RgbImage,
    glyph_data: GlyphData,
    font: &FontRef,
    point: Point,
    rotate: bool,
    pixel: Rgb<u8>,
) {
    let width = glyph_data.width;

    for glyph in glyph_data.glyphs {
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();

            outlined.draw(|x, y, v| {
                let (final_x, final_y) = if !rotate {
                    (point.x as u32 + bounds.min.x as u32 + x, point.y as u32 + bounds.min.y as u32 + y)
                }
                else {
                    (y + point.x as u32 + bounds.min.y as u32, width + point.y as u32 - bounds.min.x as u32 - x)
                };

                let mut px = buffer.get_pixel_mut(final_x, final_y);
                px.apply2(&pixel, |first, second| {
                    ((first as f32 + second as f32 * v) as u32).min(255) as u8
                });
            });
        }
    }
}

pub fn draw_glyphs_to_gray_buffer(
    buffer: &mut GrayImage,
    glyph_data: GlyphData,
    font: &FontRef,
    point: Point,
    rotate: bool,
    pixel: Luma<u8>,
) {
    let width = glyph_data.width;

    for glyph in glyph_data.glyphs {
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();

            outlined.draw(|x, y, v| {
                if v < 0.05 {
                    return;
                }

                let (final_x, final_y) = if !rotate {
                    (point.x as u32 + bounds.min.x as u32 + x, point.y as u32 + bounds.min.y as u32 + y)
                }
                else {
                    (y + point.x as u32 + bounds.min.y as u32, width + point.y as u32 - bounds.min.x as u32 - x)
                };

                let mut px = buffer.get_pixel_mut(final_x, final_y);
                *px = Luma([255]);
            });
        }
    }
}

pub fn layout_paragraph<F, SF>(
    font: SF,
    position: Point,
    max_width: f32,
    text: &str,
    target: &mut Vec<Glyph>,
) where
    F: Font,
    SF: ScaleFont<F>,
{
    let v_advance = font.height() + font.line_gap();
    let mut caret = position + point(0.0, font.ascent());
    let mut last_glyph: Option<Glyph> = None;
    for c in text.chars() {
        if c.is_control() {
            if c == '\n' {
                caret = point(position.x, caret.y + v_advance);
                last_glyph = None;
            }
            continue;
        }
        let mut glyph = font.scaled_glyph(c);
        if let Some(previous) = last_glyph.take() {
            caret.x += font.kern(previous.id, glyph.id);
        }
        glyph.position = caret;

        last_glyph = Some(glyph.clone());
        caret.x += font.h_advance(glyph.id);

        // if !c.is_whitespace() && caret.x > position.x + max_width {
        //     caret = point(position.x, caret.y + v_advance);
        //     glyph.position = caret;
        //     last_glyph = None;
        // }

        target.push(glyph);
    }
}