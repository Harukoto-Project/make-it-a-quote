#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRun {
    pub text: String,
    pub bold: bool,
}

pub fn parse_discord_bold(input: &str) -> Vec<TextRun> {
    let mut runs = Vec::new();
    let mut buf = String::new();
    let mut bold = false;
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '*' && i + 1 < chars.len() && chars[i + 1] == '*' {
            if !buf.is_empty() {
                runs.push(TextRun {
                    text: std::mem::take(&mut buf),
                    bold,
                });
            }
            bold = !bold;
            i += 2;
            continue;
        }
        buf.push(chars[i]);
        i += 1;
    }
    if !buf.is_empty() || runs.is_empty() {
        runs.push(TextRun { text: buf, bold });
    }
    if runs.len() == 1 && runs[0].text.is_empty() && !runs[0].bold {
        return vec![TextRun {
            text: String::new(),
            bold: false,
        }];
    }
    runs
}

pub fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn trim_end_each_line(s: &str) -> String {
    s.lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn strip_bold_markers(s: &str) -> String {
    parse_discord_bold(s).into_iter().map(|r| r.text).collect()
}

pub fn preprocess(raw: &str) -> String {
    strip_bold_markers(&trim_end_each_line(&normalize_newlines(raw)))
}
