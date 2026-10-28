use std::path::Path;

pub fn is_emoji_cluster(cluster: &str) -> bool {
    let chars: Vec<char> = cluster.chars().collect();
    if chars.is_empty() {
        return false;
    }
    if is_keycap(&chars) {
        return true;
    }
    if is_flag_pair(&chars) {
        return true;
    }
    chars.iter().any(|&c| {
        is_extended_pictographic(c)
            || matches!(c as u32, 0x1F3FB..=0x1F3FF)
            || c == '\u{200D}'
            || c == '\u{FE0F}'
    }) && chars.iter().any(|&c| {
        is_extended_pictographic(c) || matches!(c as u32, 0x1F1E6..=0x1F1FF) || is_keycap_base(c)
    })
}

fn is_keycap_base(c: char) -> bool {
    matches!(c, '0'..='9' | '#' | '*')
}

fn is_keycap(chars: &[char]) -> bool {
    if chars.is_empty() {
        return false;
    }
    if !is_keycap_base(chars[0]) {
        return false;
    }
    chars.iter().any(|c| *c == '\u{20E3}')
}

fn is_flag_pair(chars: &[char]) -> bool {
    let r: Vec<char> = chars
        .iter()
        .copied()
        .filter(|c| matches!(*c as u32, 0x1F1E6..=0x1F1FF))
        .collect();
    r.len() >= 2
}

fn is_extended_pictographic(c: char) -> bool {
    let u = c as u32;
    matches!(
        u,
        0x00A9
            | 0x00AE
            | 0x203C
            | 0x2049
            | 0x2122
            | 0x2139
            | 0x2194..=0x2199
            | 0x21A9..=0x21AA
            | 0x231A..=0x231B
            | 0x2328
            | 0x23CF
            | 0x23E9..=0x23F3
            | 0x23F8..=0x23FA
            | 0x24C2
            | 0x25AA..=0x25AB
            | 0x25B6
            | 0x25C0
            | 0x25FB..=0x25FE
            | 0x2600..=0x27BF
            | 0x2934..=0x2935
            | 0x2B05..=0x2B07
            | 0x2B1B..=0x2B1C
            | 0x2B50
            | 0x2B55
            | 0x3030
            | 0x303D
            | 0x3297
            | 0x3299
            | 0x1F000..=0x1FAFF
    )
}

pub fn twemoji_filename(cluster: &str) -> String {
    let chars: Vec<u32> = cluster.chars().map(|c| c as u32).collect();
    let has_zwj = chars.contains(&0x200D);
    let codes: Vec<u32> = chars
        .into_iter()
        .filter(|&c| if has_zwj { true } else { c != 0xFE0F })
        .collect();
    codes
        .iter()
        .map(|c| format!("{c:x}"))
        .collect::<Vec<_>>()
        .join("-")
}

pub fn twemoji_path(dir: &Path, cluster: &str) -> Option<std::path::PathBuf> {
    let name = twemoji_filename(cluster);
    let path = dir.join(format!("{name}.svg"));
    if path.exists() {
        return Some(path);
    }
    None
}
