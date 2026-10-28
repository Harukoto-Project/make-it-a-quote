pub mod avatar;
pub mod budoux;
pub mod draw;
pub mod emoji;
pub mod error;
pub mod fade;
pub mod fonts;
pub mod layout;
pub mod markdown;
pub mod measure;
pub mod options;
pub mod render;
pub mod wrap;

pub use error::{RenderError, Result};
pub use fonts::{set_assets_dir, FontCatalog};
pub use layout::layout_text;
pub use options::{LayoutResult, QuoteRequest, RenderOptions};
pub use render::{encode_png, render_quote, RenderOutput};
