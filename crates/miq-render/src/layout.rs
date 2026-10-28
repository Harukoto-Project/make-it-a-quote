use crate::fonts::FontCatalog;
use crate::markdown::preprocess;
use crate::measure::measure_line;
use crate::options::LayoutResult;
use crate::wrap::{drop_last_grapheme, wrap};

pub const SIZES: [f32; 8] = [60.0, 55.0, 50.0, 45.0, 40.0, 35.0, 30.0, 25.0];
pub const MAX_W: f32 = 630.0;
pub const MAX_H: f32 = 520.0;
pub const MAX_LINES: usize = 13;
pub const ELLIPSIS: &str = "...";
pub const CENTER_X: f32 = 855.0;
pub const CANVAS_W: u32 = 1200;
pub const CANVAS_H: u32 = 630;

pub fn pitch(size: f32) -> f32 {
    match size as i32 {
        60 => 75.0,
        55 => 69.0,
        50 => 63.0,
        45 => 57.0,
        40 => 50.0,
        35 => 44.0,
        30 => 39.0,
        25 => 32.0,
        _ => (size * 1.25).round(),
    }
}

pub fn layout_text(catalog: &FontCatalog, raw: &str) -> LayoutResult {
    let text = preprocess(raw);

    for &size in &SIZES {
        let lines = wrap(catalog, &text, size, MAX_W, false);
        if lines.len() <= MAX_LINES && lines.len() as f32 * pitch(size) <= MAX_H {
            return LayoutResult {
                size,
                lines,
                truncated: false,
                canvas_w: CANVAS_W,
                canvas_h: CANVAS_H,
            };
        }
    }

    let size = 25.0;
    let mut lines = wrap(catalog, &text, size, MAX_W, false);
    let mut truncated = false;
    if lines.len() > MAX_LINES {
        truncated = true;
        lines.truncate(MAX_LINES);
        let mut last = lines[MAX_LINES - 1].clone();
        while !last.is_empty()
            && measure_line(catalog, &format!("{last}{ELLIPSIS}"), size, false) > MAX_W
        {
            last = drop_last_grapheme(&last);
        }
        lines[MAX_LINES - 1] = format!("{last}{ELLIPSIS}");
    }

    LayoutResult {
        size,
        lines,
        truncated,
        canvas_w: CANVAS_W,
        canvas_h: CANVAS_H,
    }
}

pub fn baseline0(size: f32, n_lines: usize) -> f32 {
    let p = pitch(size);
    287.0 + 0.11 * size - (n_lines as f32 - 1.5) * p / 2.0
}

pub fn name_ink_top(last_baseline: f32, size: f32) -> f32 {
    last_baseline + 0.14 * size + 31.0
}
