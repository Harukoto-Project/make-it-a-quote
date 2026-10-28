use crate::emoji::{is_emoji_cluster, twemoji_path};
use crate::fonts::{FontCatalog, FontData};
use crate::layout::{baseline0, name_ink_top, pitch, CENTER_X};
use crate::measure::measure_line;
use crate::options::LayoutResult;
use image::{Rgba, RgbaImage};
use std::path::Path;
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::zeno::{Fill, Vector};
use swash::FontRef;
use unicode_segmentation::UnicodeSegmentation;

pub struct Colors {
    pub bg: [u8; 4],
    pub text: [u8; 4],
    pub muted: [u8; 4],
}

impl Colors {
    pub fn default_bw() -> Self {
        Self {
            bg: [0, 0, 0, 255],
            text: [255, 255, 255, 255],
            muted: [0xA2, 0xA2, 0xA2, 255],
        }
    }
}

thread_local! {
    static SCALE_CTX: std::cell::RefCell<ScaleContext> = std::cell::RefCell::new(ScaleContext::new());
    static SHAPE_CTX: std::cell::RefCell<swash::shape::ShapeContext> =
        std::cell::RefCell::new(swash::shape::ShapeContext::new());
}

pub fn blit(dst: &mut RgbaImage, src: &RgbaImage, dx: i32, dy: i32) {
    let (sw, sh) = (src.width() as i32, src.height() as i32);
    let (dw, dh) = (dst.width() as i32, dst.height() as i32);
    for y in 0..sh {
        let ty = dy + y;
        if ty < 0 || ty >= dh {
            continue;
        }
        for x in 0..sw {
            let tx = dx + x;
            if tx < 0 || tx >= dw {
                continue;
            }
            let p = src.get_pixel(x as u32, y as u32);
            if p[3] == 0 {
                continue;
            }
            if p[3] == 255 {
                dst.put_pixel(tx as u32, ty as u32, *p);
            } else {
                let d = dst.get_pixel(tx as u32, ty as u32).0;
                let a = p[3] as f32 / 255.0;
                let out = [
                    (p[0] as f32 * a + d[0] as f32 * (1.0 - a)).round() as u8,
                    (p[1] as f32 * a + d[1] as f32 * (1.0 - a)).round() as u8,
                    (p[2] as f32 * a + d[2] as f32 * (1.0 - a)).round() as u8,
                    255,
                ];
                dst.put_pixel(tx as u32, ty as u32, Rgba(out));
            }
        }
    }
}

const GLYPH_SS: f32 = 1.0;

fn draw_glyph_rgba(
    font: FontRef<'_>,
    glyph_id: u16,
    size: f32,
    color: [u8; 4],
) -> Option<(RgbaImage, i32, i32)> {
    SCALE_CTX.with(|ctx| {
        let mut ctx = ctx.borrow_mut();
        let ss = GLYPH_SS;
        let mut scaler = ctx.builder(font).size(size * ss).hint(false).build();
        let image = Render::new(&[
            Source::ColorOutline(0),
            Source::ColorBitmap(StrikeWith::BestFit),
            Source::Outline,
        ])
        .format(swash::zeno::Format::Alpha)
        .style(Fill::NonZero)
        .render(&mut scaler, glyph_id)?;
        let w = image.placement.width;
        let h = image.placement.height;
        let left = (image.placement.left as f32 / ss).round() as i32;
        let top = (image.placement.top as f32 / ss).round() as i32;
        if w == 0 || h == 0 {
            return Some((RgbaImage::new(1, 1), left, top));
        }

        let mut hi = RgbaImage::new(w, h);
        match image.content {
            swash::scale::image::Content::Mask => {
                for (i, a) in image.data.iter().enumerate() {
                    let x = (i as u32) % w;
                    let y = (i as u32) / w;
                    hi.put_pixel(x, y, Rgba([color[0], color[1], color[2], *a]));
                }
            }
            swash::scale::image::Content::Color => {
                for (i, px) in image.data.chunks(4).enumerate() {
                    let x = (i as u32) % w;
                    let y = (i as u32) / w;
                    hi.put_pixel(x, y, Rgba([px[0], px[1], px[2], px[3]]));
                }
            }
            swash::scale::image::Content::SubpixelMask => {
                for (i, a) in image.data.iter().enumerate() {
                    let x = (i as u32) % w;
                    let y = (i as u32) / w;
                    hi.put_pixel(x, y, Rgba([color[0], color[1], color[2], *a]));
                }
            }
        }
        Some((downsample_glyph(&hi, ss as u32), left, top))
    })
}

fn downsample_glyph(hi: &RgbaImage, factor: u32) -> RgbaImage {
    if factor <= 1 {
        return hi.clone();
    }
    let nw = (hi.width() + factor - 1) / factor;
    let nh = (hi.height() + factor - 1) / factor;
    let mut out = RgbaImage::new(nw.max(1), nh.max(1));
    for oy in 0..nh {
        for ox in 0..nw {
            let mut sum_a = 0u32;
            let mut sum_r = 0u32;
            let mut sum_g = 0u32;
            let mut sum_b = 0u32;
            let mut n = 0u32;
            for dy in 0..factor {
                for dx in 0..factor {
                    let x = ox * factor + dx;
                    let y = oy * factor + dy;
                    if x >= hi.width() || y >= hi.height() {
                        continue;
                    }
                    let p = hi.get_pixel(x, y).0;
                    sum_r += p[0] as u32;
                    sum_g += p[1] as u32;
                    sum_b += p[2] as u32;
                    sum_a += p[3] as u32;
                    n += 1;
                }
            }
            let n = n.max(1);
            out.put_pixel(
                ox,
                oy,
                Rgba([
                    ((sum_r + n / 2) / n) as u8,
                    ((sum_g + n / 2) / n) as u8,
                    ((sum_b + n / 2) / n) as u8,
                    ((sum_a + n / 2) / n) as u8,
                ]),
            );
        }
    }
    out
}

fn shape_glyphs(font: FontRef<'_>, text: &str, size: f32) -> Vec<(u16, f32)> {
    SHAPE_CTX.with(|ctx| {
        let mut ctx = ctx.borrow_mut();
        let mut shaper = ctx.builder(font).size(size).build();
        shaper.add_str(text);
        let mut out = Vec::new();
        shaper.shape_with(|cluster| {
            for g in cluster.glyphs {
                out.push((g.id, g.advance));
            }
        });
        out
    })
}

pub fn draw_text_line(
    canvas: &mut RgbaImage,
    catalog: &FontCatalog,
    text: &str,
    x: f32,
    baseline: f32,
    size: f32,
    color: [u8; 4],
    bold: bool,
) {
    let mut pen_x = x;
    for g in text.graphemes(true) {
        if is_emoji_cluster(g) {
            if let Some(path) = twemoji_path(&catalog.twemoji_dir(), g) {
                if let Some(img) = render_twemoji(&path, size) {
                    let top = (baseline - 0.88 * size).round() as i32;
                    blit(canvas, &img, pen_x.round() as i32, top);
                }
            }
            pen_x += size;
            continue;
        }
        let font_data = catalog.resolve_for_cluster(g, bold);
        let font = font_data.as_ref();
        let glyphs = shape_glyphs(font, g, size);
        for (gid, adv) in glyphs {
            if let Some((img, left, top)) = draw_glyph_rgba(font, gid, size, color) {
                blit(
                    canvas,
                    &img,
                    (pen_x + left as f32).round() as i32,
                    (baseline - top as f32).round() as i32,
                );
            }
            pen_x += adv;
        }
    }
}

fn render_twemoji(path: &Path, size: f32) -> Option<RgbaImage> {
    let data = std::fs::read(path).ok()?;
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(&data, &opt).ok()?;
    let s = size.ceil() as u32;
    let mut pixmap = tiny_skia::Pixmap::new(s, s)?;
    let scale = size / tree.size().width().max(1.0);
    let transform = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    let mut img = RgbaImage::new(s, s);
    for (i, px) in pixmap.data().chunks(4).enumerate() {
        let x = (i as u32) % s;
        let y = (i as u32) / s;
        img.put_pixel(x, y, Rgba([px[0], px[1], px[2], px[3]]));
    }
    Some(img)
}

fn ink_top_offset(catalog: &FontCatalog, text: &str, size: f32, bold: bool) -> f32 {
    let font = catalog.primary(bold);
    let font_ref = font.as_ref();
    let mut max_top = 0.0f32;
    let mut any = false;
    for g in text.graphemes(true) {
        if is_emoji_cluster(g) {
            continue;
        }
        for (gid, _) in shape_glyphs(font_ref, g, size) {
            if let Some((img, _l, top)) = draw_glyph_rgba(font_ref, gid, size, [255, 255, 255, 255])
            {
                let _ = img;
                max_top = if any {
                    max_top.max(top as f32)
                } else {
                    top as f32
                };
                any = true;
            }
        }
    }
    if any {
        max_top
    } else {
        0.8 * size
    }
}

pub fn draw_name_skewed(
    canvas: &mut RgbaImage,
    catalog: &FontCatalog,
    text: &str,
    center_x: f32,
    ink_top: f32,
    size: f32,
    color: [u8; 4],
    skew: f32,
) {
    let w = measure_line(catalog, text, size, false);
    let baseline_guess = ink_top + ink_top_offset(catalog, text, size, false);
    let start_x = center_x - w / 2.0;

    let mut pen_x = start_x;
    let font = catalog.primary(false);
    let font_ref = font.as_ref();
    for g in text.graphemes(true) {
        for (gid, adv) in shape_glyphs(font_ref, g, size) {
            if let Some((img, left, top)) = draw_glyph_rgba(font_ref, gid, size, color) {
                let base_x = pen_x + left as f32;
                let base_y = baseline_guess - top as f32;
                blit_skewed(canvas, &img, base_x, base_y, skew, baseline_guess);
            }
            pen_x += adv;
        }
    }
}

fn blit_skewed(dst: &mut RgbaImage, src: &RgbaImage, dx: f32, dy: f32, skew: f32, baseline: f32) {
    let (sw, sh) = (src.width(), src.height());
    for y in 0..sh {
        for x in 0..sw {
            let p = src.get_pixel(x, y);
            if p[3] == 0 {
                continue;
            }
            let sy = dy + y as f32;
            let sx = dx + x as f32 + skew * (baseline - sy);
            let x0 = sx.floor() as i32;
            let y0 = sy.floor() as i32;
            let fx = sx - x0 as f32;
            let fy = sy - y0 as f32;
            let a = p[3] as f32 / 255.0;
            let weights = [
                ((1.0 - fx) * (1.0 - fy) * a, x0, y0),
                (fx * (1.0 - fy) * a, x0 + 1, y0),
                ((1.0 - fx) * fy * a, x0, y0 + 1),
                (fx * fy * a, x0 + 1, y0 + 1),
            ];
            for (w, tx, ty) in weights {
                if w <= 0.001
                    || tx < 0
                    || ty < 0
                    || tx >= dst.width() as i32
                    || ty >= dst.height() as i32
                {
                    continue;
                }
                let d = dst.get_pixel(tx as u32, ty as u32).0;
                let out = [
                    (p[0] as f32 * w + d[0] as f32 * (1.0 - w)).round() as u8,
                    (p[1] as f32 * w + d[1] as f32 * (1.0 - w)).round() as u8,
                    (p[2] as f32 * w + d[2] as f32 * (1.0 - w)).round() as u8,
                    255,
                ];
                dst.put_pixel(tx as u32, ty as u32, Rgba(out));
            }
        }
    }
}

pub fn draw_body_and_meta(
    canvas: &mut RgbaImage,
    catalog: &FontCatalog,
    layout: &LayoutResult,
    display_name: &str,
    username: &str,
    watermark: &str,
    colors: &Colors,
) {
    let n = layout.lines.len();
    let size = layout.size;
    let p = pitch(size);
    let b0 = baseline0(size, n);
    for (i, line) in layout.lines.iter().enumerate() {
        let w = measure_line(catalog, line, size, false);
        let x = CENTER_X - w / 2.0;
        let baseline = b0 + i as f32 * p;
        draw_text_line(canvas, catalog, line, x, baseline, size, colors.text, false);
    }
    if n > 0 {
        let last_baseline = b0 + (n - 1) as f32 * p;
        let name_top = name_ink_top(last_baseline, size);
        let name = format!("- {display_name}");
        draw_name_skewed(
            canvas,
            catalog,
            &name,
            CENTER_X,
            name_top,
            30.0,
            colors.text,
            0.18,
        );
        if !username.is_empty() {
            let handle = format!("@{username}");
            let handle_top = name_top + 30.0;
            let hw = measure_line(catalog, &handle, 20.0, false);
            let hx = CENTER_X - hw / 2.0;
            let h_baseline = handle_top + ink_top_offset(catalog, &handle, 20.0, false);
            draw_text_line(
                canvas,
                catalog,
                &handle,
                hx,
                h_baseline,
                20.0,
                colors.muted,
                false,
            );
        }
    }
    if !watermark.is_empty() {
        let ww = measure_line(catalog, watermark, 20.0, false);
        let wx = 1189.0 - ww;
        let w_ink_top = 599.0;
        let w_baseline = w_ink_top + ink_top_offset(catalog, watermark, 20.0, false);
        draw_text_line(
            canvas,
            catalog,
            watermark,
            wx,
            w_baseline,
            20.0,
            colors.muted,
            false,
        );
    }
}

#[allow(dead_code)]
fn _keep_fontdata(_: &FontData) {}

#[allow(dead_code)]
fn _keep_vector(_: Vector) {}
