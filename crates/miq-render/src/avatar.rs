use crate::error::{RenderError, Result};
use crate::fade::bt601_gray;
use image::imageops::FilterType;
use image::{DynamicImage, Rgba, RgbaImage};

pub const AVATAR_FILTER: FilterType = FilterType::Triangle;
pub const AVATAR_OFFSET_X: i32 = -25;
pub const AVATAR_OFFSET_Y: i32 = 0;

pub fn decode_avatar(bytes: &[u8]) -> Result<DynamicImage> {
    image::load_from_memory(bytes).map_err(|e| RenderError::AvatarDecode(e.to_string()))
}

fn composite_on_black(img: &RgbaImage) -> RgbaImage {
    let mut out = RgbaImage::from_pixel(img.width(), img.height(), Rgba([0, 0, 0, 255]));
    for (x, y, p) in img.enumerate_pixels() {
        let a = p[3] as f32 / 255.0;
        if a <= 0.0 {
            continue;
        }
        if a >= 1.0 {
            out.put_pixel(x, y, Rgba([p[0], p[1], p[2], 255]));
        } else {
            out.put_pixel(
                x,
                y,
                Rgba([
                    (p[0] as f32 * a).round() as u8,
                    (p[1] as f32 * a).round() as u8,
                    (p[2] as f32 * a).round() as u8,
                    255,
                ]),
            );
        }
    }
    out
}

pub fn prepare_avatar(img: &DynamicImage, size: u32, keep_color: bool) -> RgbaImage {
    let rgba = img.to_rgba8();
    let composited = composite_on_black(&rgba);
    let resized = image::imageops::resize(&composited, size, size, AVATAR_FILTER);
    if keep_color {
        return resized;
    }
    let mut out = resized;
    for px in out.pixels_mut() {
        let g = bt601_gray(px[0], px[1], px[2]);
        px[0] = g;
        px[1] = g;
        px[2] = g;
        px[3] = 255;
    }
    out
}

pub fn placeholder_avatar(size: u32, light: bool) -> RgbaImage {
    let v = if light { 220 } else { 40 };
    RgbaImage::from_pixel(size, size, Rgba([v, v, v, 255]))
}
