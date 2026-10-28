use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteRequest {
    pub text: String,
    pub display_name: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub watermark: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub text: String,
    pub display_name: String,
    pub username: String,
    pub watermark: String,
}

impl From<QuoteRequest> for RenderOptions {
    fn from(r: QuoteRequest) -> Self {
        Self {
            text: r.text,
            display_name: r.display_name,
            username: r.username,
            watermark: r.watermark,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub size: f32,
    pub lines: Vec<String>,
    pub truncated: bool,
    pub canvas_w: u32,
    pub canvas_h: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExtensionFlags {
    pub light: bool,
    pub color: bool,
    pub flip: bool,
    pub bold: bool,
    pub vertical: bool,
}
