use std::fmt;

use crate::hunk::error::HunkError;
use crate::m68k::decode::DecodeError;

/// Unified error type for the dis68k library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Hunk(HunkError),
    Decode(DecodeError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Hunk(e) => write!(f, "hunk error: {e}"),
            Error::Decode(e) => write!(f, "decode error: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<HunkError> for Error {
    fn from(e: HunkError) -> Self {
        Error::Hunk(e)
    }
}

impl From<DecodeError> for Error {
    fn from(e: DecodeError) -> Self {
        Error::Decode(e)
    }
}
