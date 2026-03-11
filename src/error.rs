//! Error types for ah-ah-ah.

use thiserror::Error;

/// Errors that can occur during token counting.
#[derive(Error, Debug)]
pub enum Error {
    /// The embedded Claude vocabulary failed to parse.
    #[error("failed to parse Claude vocabulary: {0}")]
    VocabParse(#[from] serde_json::Error),
}

/// Result type alias using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
