use crate::emoji::{is_emoji_cluster, twemoji_path};
use crate::fonts::FontCatalog;
use crate::markdown::TextRun;
use std::cell::RefCell;
use swash::shape::ShapeContext;
use swash::text::Script;
use unicode_segmentation::UnicodeSegmentation;

thread_local! {
    static SHAPE_CTX: RefCell<ShapeContext> = RefCell::new(ShapeContext::new());
}

pub fn measure_plain(catalog: &FontCatalog, text: &str, size: f32, force_bold: bool) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let runs = [TextRun {
        text: text.to_string(),
        bold: force_bold,
    }];
    measure_runs(catalog, &runs, size, force_bold)
}

pub fn measure_runs(catalog: &FontCatalog, runs: &[TextRun], size: f32, force_bold: bool) -> f32 {
    let mut width = 0.0f32;
    for run in runs {
        let bold = force_bold || run.bold;
        for grapheme in run.text.graphemes(true) {
            if is_emoji_cluster(grapheme) {
                let _ = twemoji_path(&catalog.twemoji_dir(), grapheme);
                width += size;
                continue;
            }
            let font = catalog.resolve_for_cluster(grapheme, bold);
            width += shape_advance(font.as_ref(), grapheme, size);
        }
    }
    width
}

pub fn measure_line(catalog: &FontCatalog, line: &str, size: f32, force_bold: bool) -> f32 {
    measure_plain(catalog, line.trim_end(), size, force_bold)
}

fn shape_advance(font: swash::FontRef<'_>, text: &str, size: f32) -> f32 {
    SHAPE_CTX.with(|ctx| {
        let mut ctx = ctx.borrow_mut();
        let mut shaper = ctx.builder(font).script(Script::Latin).size(size).build();
        shaper.add_str(text);
        let mut adv = 0.0f32;
        shaper.shape_with(|cluster| {
            for g in cluster.glyphs {
                adv += g.advance;
            }
        });
        adv
    })
}

pub fn is_cjk_char(c: char) -> bool {
    matches!(
        c as u32,
        0x3040..=0x30FF
            | 0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0xFF66..=0xFF9D
            | 0x3000..=0x303F
            | 0xAC00..=0xD7AF
            | 0x1100..=0x11FF
            | 0x3130..=0x318F
    )
}

pub fn paragraph_has_cjk(s: &str) -> bool {
    s.chars().any(is_cjk_char)
}
