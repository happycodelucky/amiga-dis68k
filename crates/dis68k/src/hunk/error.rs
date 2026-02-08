use std::fmt;

/// Errors that can occur while parsing an Amiga hunk file.
///
/// All variants are self-contained (no std::io references) so the
/// library remains portable to environments without filesystem access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HunkError {
    /// File is too short to contain the expected data.
    TooShort {
        offset: usize,
        needed: usize,
        available: usize,
    },
    /// File does not start with the HUNK_HEADER magic (0x000003F3).
    BadMagic { found: u32 },
    /// Encountered an unrecognized hunk type ID.
    UnknownHunkType { raw: u32, offset: usize },
    /// Unexpected end of data while parsing a specific structure.
    UnexpectedEof { context: &'static str },
    /// A string length field exceeds reasonable bounds.
    InvalidStringLength { length: u32, offset: usize },
    /// The number of hunks found doesn't match the header.
    HunkCountMismatch { expected: usize, found: usize },
    /// An invalid value was encountered in a specific field.
    InvalidValue { context: &'static str, value: u32 },
}

impl fmt::Display for HunkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HunkError::TooShort {
                offset,
                needed,
                available,
            } => {
                write!(
                    f,
                    "at offset 0x{offset:X}: need {needed} bytes, only {available} available"
                )
            }
            HunkError::BadMagic { found } => {
                write!(
                    f,
                    "not an Amiga executable: expected magic 0x000003F3, found 0x{found:08X}"
                )
            }
            HunkError::UnknownHunkType { raw, offset } => {
                write!(
                    f,
                    "unknown hunk type 0x{raw:08X} at offset 0x{offset:X}"
                )
            }
            HunkError::UnexpectedEof { context } => {
                write!(f, "unexpected end of file while reading {context}")
            }
            HunkError::InvalidStringLength { length, offset } => {
                write!(
                    f,
                    "invalid string length {length} longwords at offset 0x{offset:X}"
                )
            }
            HunkError::HunkCountMismatch { expected, found } => {
                write!(
                    f,
                    "header declares {expected} hunks but found {found}"
                )
            }
            HunkError::InvalidValue { context, value } => {
                write!(f, "invalid {context}: 0x{value:08X}")
            }
        }
    }
}

impl std::error::Error for HunkError {}
