//! Utilities used in this crate which can also be generally useful downstream.

use super::{utils::string::String, Word};
use core::fmt::{self, Display, Write};

#[cfg(not(feature = "std"))]
pub use alloc::{format, vec};

#[cfg(feature = "std")]
pub use std::{format, vec};

mod diff;
mod kv_map;

// RE-EXPORTS
// ================================================================================================
pub use winter_utils::{
    string, uninit_vector, Box, ByteReader, ByteWriter, Deserializable, DeserializationError,
    Serializable, SliceReader,
};

pub mod collections {
    pub use super::diff::*;
    pub use super::kv_map::*;
    pub use winter_utils::collections::*;
}

// UTILITY FUNCTIONS
// ================================================================================================

/// Converts a [Word] into hex.
pub fn word_to_hex(w: &Word) -> Result<String, fmt::Error> {
    let mut s = String::new();

    for byte in w.iter().flat_map(|e| e.to_bytes()) {
        write!(s, "{byte:02x}")?;
    }

    Ok(s)
}

/// Renders an array of bytes as hex into a String.
pub fn bytes_to_hex_string<const N: usize>(data: [u8; N]) -> String {
    let mut s = String::with_capacity(N + 2);

    s.push_str("0x");
    for byte in data.iter() {
        write!(s, "{byte:02x}").expect("formatting hex failed");
    }

    s
}

/// Defines errors which can occur during parsing of hexadecimal strings.
#[derive(Debug)]
pub enum HexParseError {
    InvalidLength { expected: usize, actual: usize },
    MissingPrefix,
    InvalidChar,
    OutOfRange,
}

impl Display for HexParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HexParseError::InvalidLength { expected, actual } => {
                write!(f, "Hex encoded RpoDigest must have length 66, including the 0x prefix. expected {expected} got {actual}")
            }
            HexParseError::MissingPrefix => {
                write!(f, "Hex encoded RpoDigest must start with 0x prefix")
            }
            HexParseError::InvalidChar => {
                write!(f, "Hex encoded RpoDigest must contain characters [a-zA-Z0-9]")
            }
            HexParseError::OutOfRange => {
                write!(f, "Hex encoded values of an RpoDigest must be inside the field modulus")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for HexParseError {}

/// Parses a hex string into an array of bytes of known size.
pub fn hex_to_bytes<const N: usize>(value: &str) -> Result<[u8; N], HexParseError> {
    let expected: usize = (N * 2) + 2;
    if value.len() != expected {
        return Err(HexParseError::InvalidLength { expected, actual: value.len() });
    }

    if !value.starts_with("0x") {
        return Err(HexParseError::MissingPrefix);
    }

    let mut data = value.bytes().skip(2).map(|v| match v {
        b'0'..=b'9' => Ok(v - b'0'),
        b'a'..=b'f' => Ok(v - b'a' + 10),
        b'A'..=b'F' => Ok(v - b'A' + 10),
        _ => Err(HexParseError::InvalidChar),
    });

    let mut decoded = [0u8; N];
    #[allow(clippy::needless_range_loop)]
    for pos in 0..N {
        // These `unwrap` calls are okay because the length was checked above
        let high: u8 = data.next().unwrap()?;
        let low: u8 = data.next().unwrap()?;
        decoded[pos] = (high << 4) + low;
    }

    Ok(decoded)
}
