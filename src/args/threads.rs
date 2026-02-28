use derive_more::{Display, Error};
use std::{
    num::{NonZeroUsize, ParseIntError},
    str::FromStr,
};

const AUTO: &str = "auto";
const MAX: &str = "max";

/// Number of rayon threads.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Display)]
pub enum Threads {
    #[default]
    #[display("{AUTO}")]
    Auto,
    #[display("{MAX}")]
    Max,
    Fixed(NonZeroUsize),
}

/// Error that occurs when parsing a string as [`Threads`].
#[derive(Debug, Display, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum FromStrError {
    #[display("Value is neither {AUTO:?}, {MAX:?}, nor a number: {_0}")]
    InvalidSyntax(ParseIntError),
}

impl FromStr for Threads {
    type Err = FromStrError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text = text.trim();
        match text {
            AUTO => return Ok(Threads::Auto),
            MAX => return Ok(Threads::Max),
            _ => {}
        };
        text.parse()
            .map_err(FromStrError::InvalidSyntax)
            .map(Threads::Fixed)
    }
}
