use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct ModelFile {
    #[serde(flatten)]
    features: HashMap<String, HashMap<String, i32>>,
}

static JA_MODEL: Lazy<ModelFile> =
    Lazy::new(|| serde_json::from_str(include_str!("budoux_ja.json")).expect("budoux ja model"));

fn get_score(model: &ModelFile, key: &str, seq: &str) -> i32 {
    model
        .features
        .get(key)
        .and_then(|m| m.get(seq))
        .copied()
        .unwrap_or(0)
}

fn char_at(chars: &[char], i: isize) -> char {
    if i < 0 || i as usize >= chars.len() {
        '▁'
    } else {
        chars[i as usize]
    }
}

pub fn parse(text: &str) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }
    let model = &*JA_MODEL;
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= 1 {
        return vec![text.to_string()];
    }

    let base_score = -100i32;
    let mut boundaries = Vec::new();
    for i in 1..chars.len() {
        let mut score = base_score;
        let i = i as isize;
        score += get_score(model, "UW1", &char_at(&chars, i - 3).to_string());
        score += get_score(model, "UW2", &char_at(&chars, i - 2).to_string());
        score += get_score(model, "UW3", &char_at(&chars, i - 1).to_string());
        score += get_score(model, "UW4", &char_at(&chars, i).to_string());
        score += get_score(model, "UW5", &char_at(&chars, i + 1).to_string());
        score += get_score(model, "UW6", &char_at(&chars, i + 2).to_string());
        score += get_score(
            model,
            "BW1",
            &format!("{}{}", char_at(&chars, i - 2), char_at(&chars, i - 1)),
        );
        score += get_score(
            model,
            "BW2",
            &format!("{}{}", char_at(&chars, i - 1), char_at(&chars, i)),
        );
        score += get_score(
            model,
            "BW3",
            &format!("{}{}", char_at(&chars, i), char_at(&chars, i + 1)),
        );
        score += get_score(
            model,
            "TW1",
            &format!(
                "{}{}{}",
                char_at(&chars, i - 3),
                char_at(&chars, i - 2),
                char_at(&chars, i - 1)
            ),
        );
        score += get_score(
            model,
            "TW2",
            &format!(
                "{}{}{}",
                char_at(&chars, i - 2),
                char_at(&chars, i - 1),
                char_at(&chars, i)
            ),
        );
        score += get_score(
            model,
            "TW3",
            &format!(
                "{}{}{}",
                char_at(&chars, i - 1),
                char_at(&chars, i),
                char_at(&chars, i + 1)
            ),
        );
        score += get_score(
            model,
            "TW4",
            &format!(
                "{}{}{}",
                char_at(&chars, i),
                char_at(&chars, i + 1),
                char_at(&chars, i + 2)
            ),
        );
        if score > 0 {
            boundaries.push(i as usize);
        }
    }

    let mut out = Vec::new();
    let mut start = 0usize;
    for &b in &boundaries {
        out.push(chars[start..b].iter().collect());
        start = b;
    }
    out.push(chars[start..].iter().collect());
    out
}
