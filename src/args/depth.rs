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
    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let source = source.trim();
        if source == INFINITE {
            return Ok(Depth::Infinite);
        }
        source
            .parse()
            .map_err(FromStrError::InvalidSyntax)
            .map(Depth::Finite)
    }
}

impl TryFrom<u64> for Depth {
    type Error = TryFromIntError;
    fn try_from(source: u64) -> Result<Self, Self::Error> {
        source.try_into().map(Depth::Finite)
    }
}
