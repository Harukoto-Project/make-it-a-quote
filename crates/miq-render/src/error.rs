use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("invalid field: {0}")]
    InvalidField(String),
    #[error("unsupported option: {0}")]
    UnsupportedOption(String),
    #[error("font not found: {0}")]
    FontNotFound(String),
    #[error("avatar decode failed: {0}")]
    AvatarDecode(String),
    #[error("render failed: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, RenderError>;
