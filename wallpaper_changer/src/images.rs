use ab_glyph::FontRef;
use ab_glyph::PxScale;
use image::imageops::blur;
use image::DynamicImage;
use image::GenericImageView;
use image::Rgba;
use image::RgbaImage;
use imageproc::drawing::{draw_text_mut, text_size as get_text_size};
use log::info;
use std::env;
use std::error::Error;

/// Writes text on an image.
///
/// # Errors
/// Fails if the font can't be loaded.
pub(crate) fn write_text_on_image(
    img: &mut DynamicImage,
    text: &str,
    font_size: u32,
    label_position: &str,
) -> Result<(), Box<dyn Error>> {
    if label_position == "none" {
        return Ok(());
    }
    info!("Writing text on image...");

    let font_data = include_bytes!(concat!(env!("OUT_DIR"), "/Montserrat-Bold.ttf"));
    let font = FontRef::try_from_slice(font_data)?;

    let scale = PxScale {
        x: font_size as f32,
        y: font_size as f32,
    };

    let (width, height) = img.dimensions();

    let mut image_buffer = img.to_rgba8();

    // Calculate text size
    let text_size = get_text_size(scale, &font, text);
    let (x, y) = match label_position {
        "center" => (
            (width as i32 - text_size.0 as i32) / 2,
            (height as i32 - text_size.1 as i32) / 2,
        ),
        "top_right" => (width as i32 - text_size.0 as i32 - 10, 10),
        "bottom_left" => (10, height as i32 - text_size.1 as i32 - 10),
        "bottom_right" => (
            width as i32 - text_size.0 as i32 - 10,
            height as i32 - text_size.1 as i32 - 10,
        ),
        // top_left
        _ => (10, 10),
    };

    // Create a shadow image with the text
    let mut shadow_image = RgbaImage::new(width, height);
    for (i, line) in text.lines().enumerate() {
        let line_width = get_text_size(scale, &font, line).0;
        let line_x = match label_position {
            "center" => ((width - line_width as u32) / 2) as i32,
            "top_right" | "bottom_right" => width as i32 - line_width as i32 - 10,
            _ => x,
        };
        let line_y = y + i as i32 * (scale.y as i32 + 5);
        draw_text_mut(
            &mut shadow_image,
            Rgba([0, 0, 0, 255]),
            line_x,
            line_y,
            scale,
            &font,
            line,
        );
    }

    // Apply blur to the shadow image
    let shadow_image = blur(&shadow_image, 5.0);

    // Overlay the shadow image onto the original image
    for y in 0..height {
        for x in 0..width {
            let shadow_pixel = shadow_image.get_pixel(x, y);
            if shadow_pixel[3] > 0 {
                let original_pixel = image_buffer.get_pixel_mut(x, y);
                *original_pixel = blend(original_pixel, shadow_pixel);
            }
        }
    }

    // Draw the original text on top of the shadow with an outline
    for (i, line) in text.lines().enumerate() {
        let line_width = get_text_size(scale, &font, line).0;
        let line_x = match label_position {
            "center" => ((width - line_width as u32) / 2) as i32,
            "top_right" | "bottom_right" => width as i32 - line_width as i32 - 10,
            _ => x,
        };
        let line_y = y + i as i32 * (scale.y as i32 + 5);
        draw_text_with_outline(
            &mut image_buffer,
            Rgba([255, 255, 255, 255]),
            Rgba([0, 0, 0, 255]),
            line_x,
            line_y,
            scale,
            &font,
            line,
            1,
        );
    }

    *img = DynamicImage::ImageRgba8(image_buffer);
    Ok(())
}

pub(crate) fn blend(base: &Rgba<u8>, overlay: &Rgba<u8>) -> Rgba<u8> {
    let alpha = overlay[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    Rgba([
        (base[0] as f32 * inv_alpha + overlay[0] as f32 * alpha) as u8,
        (base[1] as f32 * inv_alpha + overlay[1] as f32 * alpha) as u8,
        (base[2] as f32 * inv_alpha + overlay[2] as f32 * alpha) as u8,
        255,
    ])
}

pub(crate) fn draw_text_with_outline(
    image: &mut RgbaImage,
    color: Rgba<u8>,
    outline_color: Rgba<u8>,
    x: i32,
    y: i32,
    scale: PxScale,
    font: &FontRef,
    text: &str,
    outline_width: i32,
) {
    // Draw outline
    for dy in -outline_width..=outline_width {
        for dx in -outline_width..=outline_width {
            if dx != 0 || dy != 0 {
                draw_text_mut(image, outline_color, x + dx, y + dy, scale, font, text);
            }
        }
    }

    // Draw text
    draw_text_mut(image, color, x, y, scale, font, text);
}
