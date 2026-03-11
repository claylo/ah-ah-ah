//! Error types for ah-ah-ah

use thiserror::Error;

/// Errors that can occur in ah-ah-ah.
#[derive(Error, Debug)]
pub enum Error {
    /// Placeholder variant (remove when adding real errors).
    #[error("{0}")]
    Other(String),
}

/// Result type alias using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
