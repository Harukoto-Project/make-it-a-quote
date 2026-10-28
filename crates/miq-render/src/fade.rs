use libm::{erfc, sqrt};

pub fn mu_y(y: f32) -> f32 {
    let dy = y - 621.9;
    407.6 - 3.375e-4 * dy * dy
}

pub fn transmittance(x: f32, y: f32, sigma: f32) -> f32 {
    let mu = mu_y(y);
    let z = ((x - mu) as f64) / (sigma as f64 * sqrt(2.0));
    0.5 * erfc(z) as f32
}

pub fn apply_fade_gray(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    avatar_w: u32,
    light: bool,
    flip: bool,
) {
    let sigma = 66.0f32;
    for y in 0..height {
        for x in 0..avatar_w.min(width) {
            let fx = if flip {
                (width - 1 - x) as f32
            } else {
                x as f32
            };
            let t = transmittance(fx, y as f32, sigma).clamp(0.0, 1.0);
            let i = ((y * width + x) * 4) as usize;
            if light {
                let inv = 1.0 - t;
                for c in 0..3 {
                    let v = pixels[i + c] as f32;
                    pixels[i + c] = (v * t + 255.0 * inv).round().clamp(0.0, 255.0) as u8;
                }
            } else {
                for c in 0..3 {
                    pixels[i + c] = (pixels[i + c] as f32 * t).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
}

pub fn bt601_gray(r: u8, g: u8, b: u8) -> u8 {
    (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32).round() as u8
}

pub fn bt709_gray(r: u8, g: u8, b: u8) -> u8 {
    (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32).round() as u8
}

fn srgb_to_linear(c: f32) -> f32 {
    let c = c / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(y: f32) -> f32 {
    let v = if y <= 0.0031308 {
        12.92 * y
    } else {
        1.055 * y.powf(1.0 / 2.4) - 0.055
    };
    (v * 255.0).round().clamp(0.0, 255.0)
}

pub fn linear_bt709_gray(r: u8, g: u8, b: u8) -> u8 {
    let y = 0.2126 * srgb_to_linear(r as f32)
        + 0.7152 * srgb_to_linear(g as f32)
        + 0.0722 * srgb_to_linear(b as f32);
    linear_to_srgb(y) as u8
}
