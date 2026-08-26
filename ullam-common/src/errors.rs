use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
#[error("Failed IO for: {location} during {context}. Reason: {error}")]
pub struct BetterIoError {
    pub location: PathBuf,
    pub context: &'static str,
    pub error: std::io::Error,
}

impl BetterIoError {
    #[inline]
    pub fn new(location: impl Into<PathBuf>, context: &'static str, error: std::io::Error) -> Self {
        Self {
            location: location.into(),
            context,
            error,
        }
    }
}
