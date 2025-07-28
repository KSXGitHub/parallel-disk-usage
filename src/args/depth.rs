use derive_more::{Display, Error};
use std::{
    num::{NonZeroU64, ParseIntError, TryFromIntError},
    str::FromStr,
};

const INFINITE: &str = "inf";

/// Maximum depth of the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Depth {
    #[display("{INFINITE}")]
    Infinite,
    Finite(NonZeroU64),
}

impl Depth {
    /// Convert depth into something comparable.
    pub(crate) fn get(self) -> u64 {
        match self {
            Depth::Infinite => u64::MAX,
            Depth::Finite(value) => value.get(),
        }
    }
}

/// Error that occurs when parsing a string as [`Depth`].
#[derive(Debug, Display, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum FromStrError {
    #[display("Value is neither {INFINITE:?} nor a positive integer: {_0}")]
    InvalidSyntax(ParseIntError),
}

impl FromStr for Depth {
    type Err = FromStrError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text = text.trim();
        if text == INFINITE {
            return Ok(Depth::Infinite);
        }
        text.parse()
            .map_err(FromStrError::InvalidSyntax)
            .map(Depth::Finite)
    }
}

impl TryFrom<u64> for Depth {
    type Error = TryFromIntError;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        value.try_into().map(Depth::Finite)
    }
}
