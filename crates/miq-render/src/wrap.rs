use crate::budoux;
use crate::fonts::FontCatalog;
use crate::measure::{measure_line, paragraph_has_cjk};
use unicode_linebreak::{linebreaks, BreakOpportunity};
use unicode_segmentation::UnicodeSegmentation;

const LINE_HEAD_NG: &str =
    "、。，．）」』】〕〉》ー々ぁぃぅぇぉっゃゅょゎァィゥェォッャュョヮヵヶ!?！？…‥・：；）";
const LINE_TAIL_NG: &str = "「『（【〔〈《";

pub fn segments(paragraph: &str) -> Vec<String> {
    if paragraph.is_empty() {
        return vec![String::new()];
    }
    let mut units = if paragraph_has_cjk(paragraph) {
        budoux_units(paragraph)
    } else {
        uax14_units(paragraph)
    };
    apply_kinsoku(&mut units);
    units
}

fn budoux_units(paragraph: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut cjk_buf = String::new();

    let flush_cjk = |cjk: &mut String, out: &mut Vec<String>| {
        if cjk.is_empty() {
            return;
        }
        for p in budoux::parse(cjk) {
            out.push(p);
        }
        cjk.clear();
    };
    let flush_lat = |lat: &mut String, out: &mut Vec<String>| {
        if lat.is_empty() {
            return;
        }
        out.extend(uax14_units(lat));
        lat.clear();
    };

    for ch in paragraph.chars() {
        if crate::measure::is_cjk_char(ch) || "、。，．・：；！？ー々".contains(ch) {
            flush_lat(&mut buf, &mut out);
            cjk_buf.push(ch);
        } else {
            flush_cjk(&mut cjk_buf, &mut out);
            buf.push(ch);
        }
    }
    flush_cjk(&mut cjk_buf, &mut out);
    flush_lat(&mut buf, &mut out);
    if out.is_empty() {
        out.push(paragraph.to_string());
    }
    out
}

fn uax14_units(paragraph: &str) -> Vec<String> {
    let mut units = Vec::new();
    let mut last = 0usize;
    let bytes = paragraph.as_bytes();
    for (i, opportunity) in linebreaks(paragraph) {
        if opportunity == BreakOpportunity::Allowed || i == paragraph.len() {
            if i > last {
                let s = std::str::from_utf8(&bytes[last..i])
                    .unwrap_or("")
                    .to_string();
                if !s.is_empty() {
                    units.push(s);
                }
                last = i;
            }
        }
    }
    if units.is_empty() {
        units.push(paragraph.to_string());
    }
    units
}

fn apply_kinsoku(units: &mut Vec<String>) {
    if units.len() <= 1 {
        return;
    }
    let mut i = 1;
    while i < units.len() {
        let head_bad = units[i]
            .chars()
            .next()
            .map(|c| LINE_HEAD_NG.contains(c))
            .unwrap_or(false);
        let prev_tail_bad = units[i - 1]
            .chars()
            .last()
            .map(|c| LINE_TAIL_NG.contains(c))
            .unwrap_or(false);
        if head_bad || prev_tail_bad {
            let cur = units.remove(i);
            units[i - 1].push_str(&cur);
        } else {
            i += 1;
        }
    }
}

pub fn wrap(
    catalog: &FontCatalog,
    text: &str,
    size: f32,
    max_w: f32,
    force_bold: bool,
) -> Vec<String> {
    let mut lines = Vec::new();
    let parts: Vec<&str> = if text.is_empty() {
        vec![""]
    } else {
        text.split('\n').collect()
    };

    for para in parts {
        let mut cur = String::new();
        for u in segments(para) {
            let candidate = format!("{cur}{u}");
            if measure_line(catalog, candidate.trim_end(), size, force_bold) <= max_w {
                cur = candidate;
                continue;
            }
            if !cur.is_empty() {
                lines.push(cur.trim_end().to_string());
                cur.clear();
            }
            if measure_line(catalog, u.trim_end(), size, force_bold) <= max_w {
                cur = u;
                continue;
            }
            for g in u.graphemes(true) {
                let next = format!("{cur}{g}");
                if !cur.is_empty()
                    && measure_line(catalog, next.trim_end(), size, force_bold) > max_w
                {
                    lines.push(cur.trim_end().to_string());
                    cur.clear();
                }
                cur.push_str(g);
            }
        }
        lines.push(cur.trim_end().to_string());
    }
    lines
}

pub fn drop_last_grapheme(s: &str) -> String {
    let mut g: Vec<&str> = s.graphemes(true).collect();
    g.pop();
    g.concat()
}
