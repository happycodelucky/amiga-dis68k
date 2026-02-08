use std::fmt;

use crate::hunk::error::HunkError;

/// Unified error type for the dis68k library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Hunk(HunkError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Hunk(e) => write!(f, "hunk error: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<HunkError> for Error {
    fn from(e: HunkError) -> Self {
        Error::Hunk(e)
    }
}
