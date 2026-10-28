use crate::avatar::{
    decode_avatar, placeholder_avatar, prepare_avatar, AVATAR_OFFSET_X, AVATAR_OFFSET_Y,
};
use crate::draw::{blit, draw_body_and_meta, Colors};
use crate::error::Result;
use crate::fade::apply_fade_gray;
use crate::fonts::FontCatalog;
use crate::layout::{layout_text, CANVAS_H, CANVAS_W};
use crate::options::{LayoutResult, RenderOptions};
use image::RgbaImage;

pub struct RenderOutput {
    pub image: RgbaImage,
    pub layout: LayoutResult,
}

pub fn render_quote(
    catalog: &FontCatalog,
    opts: &RenderOptions,
    avatar_bytes: Option<&[u8]>,
) -> Result<RenderOutput> {
    let layout = layout_text(catalog, &opts.text);
    let mut canvas = RgbaImage::from_pixel(CANVAS_W, CANVAS_H, image::Rgba([0, 0, 0, 255]));

    let avatar_img = match avatar_bytes {
        Some(b) => {
            let dynimg = decode_avatar(b)?;
            prepare_avatar(&dynimg, CANVAS_H, false)
        }
        None => placeholder_avatar(CANVAS_H, false),
    };
    blit(&mut canvas, &avatar_img, AVATAR_OFFSET_X, AVATAR_OFFSET_Y);
    apply_fade_gray(canvas.as_mut(), CANVAS_W, CANVAS_H, CANVAS_H, false, false);

    let colors = Colors::default_bw();
    draw_body_and_meta(
        &mut canvas,
        catalog,
        &layout,
        &opts.display_name,
        &opts.username,
        &opts.watermark,
        &colors,
    );

    Ok(RenderOutput {
        image: canvas,
        layout,
    })
}

pub fn encode_png(img: &RgbaImage) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    image::ImageEncoder::write_image(
        encoder,
        img.as_raw(),
        img.width(),
        img.height(),
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|e| crate::error::RenderError::Internal(e.to_string()))?;
    Ok(buf)
}
