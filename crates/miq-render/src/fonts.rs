use crate::error::{RenderError, Result};
use once_cell::sync::OnceCell;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use swash::{CacheKey, FontRef};

#[derive(Clone)]
pub struct FontData {
    pub bytes: Arc<Vec<u8>>,
    pub key: CacheKey,
    pub index: usize,
    pub label: String,
}

impl FontData {
    pub fn as_ref(&self) -> FontRef<'_> {
        FontRef::from_index(self.bytes.as_slice(), self.index).expect("valid font")
    }

    pub fn load_named(path: &Path, label: &str) -> Result<Self> {
        let bytes = std::fs::read(path)
            .map_err(|e| RenderError::FontNotFound(format!("{} ({label}): {e}", path.display())))?;
        let index = 0usize;
        let font = FontRef::from_index(&bytes, index)
            .ok_or_else(|| RenderError::FontNotFound(label.to_string()))?;
        Ok(Self {
            key: font.key,
            bytes: Arc::new(bytes),
            index,
            label: label.to_string(),
        })
    }

    pub fn covers(&self, ch: char) -> bool {
        self.as_ref().charmap().map(ch) != 0
    }

    pub fn covers_cluster(&self, cluster: &str) -> bool {
        for c in cluster.chars() {
            if c.is_whitespace() || is_combiner(c) {
                continue;
            }
            if !self.covers(c) {
                return false;
            }
        }
        true
    }
}

fn is_combiner(c: char) -> bool {
    matches!(
        c as u32,
        0x0300..=0x036F
            | 0x1AB0..=0x1AFF
            | 0x1DC0..=0x1DFF
            | 0x20D0..=0x20FF
            | 0xFE00..=0xFE0F
            | 0xFE20..=0xFE2F
            | 0x200D
            | 0x200C
    )
}

pub struct FontCatalog {
    pub assets_dir: PathBuf,
    primary_regular: FontData,
    primary_bold: Option<FontData>,
    fallbacks: Vec<FontData>,
}

static GLOBAL_ASSETS: OnceCell<PathBuf> = OnceCell::new();

pub fn set_assets_dir(path: impl Into<PathBuf>) {
    let _ = GLOBAL_ASSETS.set(path.into());
}

pub fn default_assets_dir() -> PathBuf {
    if let Some(p) = GLOBAL_ASSETS.get() {
        return p.clone();
    }
    if let Ok(p) = std::env::var("MIQ_ASSETS") {
        return PathBuf::from(p);
    }
    for c in [
        PathBuf::from("assets"),
        PathBuf::from("../assets"),
        PathBuf::from("../../assets"),
        PathBuf::from("/workspace/assets"),
    ] {
        if c.join("fonts").exists() {
            return c;
        }
    }
    PathBuf::from("assets")
}

fn fallback_names() -> [&'static str; 13] {
    [
        "NotoSansJP-wght.ttf",
        "NotoSansSC-wght.ttf",
        "NotoSansTC-wght.ttf",
        "NotoSansKR-wght.ttf",
        "NotoSans-wdth-wght.ttf",
        "NotoSansArabic-wdth-wght.ttf",
        "NotoSansHebrew-wdth-wght.ttf",
        "NotoSansThai-wdth-wght.ttf",
        "NotoSansDevanagari-wdth-wght.ttf",
        "NotoSansMath-Regular.ttf",
        "NotoSansSymbols-wght.ttf",
        "NotoSansSymbols2-Regular.ttf",
        "MPLUSRounded1c-Regular.ttf",
    ]
}

impl FontCatalog {
    pub fn load_default() -> Result<Self> {
        let assets = default_assets_dir();
        let fonts = assets.join("fonts");
        let primary_regular =
            FontData::load_named(&fonts.join("MPLUSRounded1c-Regular.ttf"), "mplus")?;
        let primary_bold =
            FontData::load_named(&fonts.join("MPLUSRounded1c-Bold.ttf"), "mplus-bold").ok();

        let mut fallbacks = Vec::new();
        for name in fallback_names() {
            if name.starts_with("MPLUS") {
                continue;
            }
            let path = fonts.join(name);
            if path.exists() {
                if let Ok(f) = FontData::load_named(&path, name) {
                    fallbacks.push(f);
                }
            }
        }

        Ok(Self {
            assets_dir: assets,
            primary_regular,
            primary_bold,
            fallbacks,
        })
    }

    pub fn primary(&self, bold: bool) -> &FontData {
        if bold {
            self.primary_bold.as_ref().unwrap_or(&self.primary_regular)
        } else {
            &self.primary_regular
        }
    }

    pub fn resolve_for_cluster(&self, cluster: &str, bold: bool) -> &FontData {
        let primary = self.primary(bold);
        if primary.covers_cluster(cluster) {
            return primary;
        }
        for fb in &self.fallbacks {
            if fb.covers_cluster(cluster) {
                return fb;
            }
        }
        primary
    }

    pub fn twemoji_dir(&self) -> PathBuf {
        self.assets_dir.join("twemoji/svg")
    }
}
