use derive_more::{AsRef, Deref, Display, Error, Into};
use std::{
    convert::{TryFrom, TryInto},
    num::ParseFloatError,
    str::FromStr,
};

/// Floating-point value that is greater than or equal to 0 and less than 1.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, AsRef, Deref, Display, Into)]
pub struct Fraction(f32);

/// Error that occurs when calling [`Fraction::new`].
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Error)]
pub enum ConversionError {
    /// Provided value is greater than or equal to 1.
    #[display("greater than or equal to 1")]
    UpperBound,
    /// Provided value is less than 0.
    #[display("less than 0")]
    LowerBound,
}

impl Fraction {
    /// Create a [`Fraction`].
    pub fn new(value: f32) -> Result<Self, ConversionError> {
        use ConversionError::*;
        if value >= 1.0 {
            return Err(UpperBound);
        }
        if value < 0.0 {
            return Err(LowerBound);
        }
        Ok(Fraction(value))
    }
}

impl TryFrom<f32> for Fraction {
    type Error = ConversionError;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Fraction::new(value)
    }
}

/// Error that occurs when converting a string into an instance of [`Fraction`].
#[derive(Debug, Display, Clone, PartialEq, Eq, Error)]
pub enum FromStrError {
    ParseFloatError(ParseFloatError),
    Conversion(ConversionError),
}

impl FromStr for Fraction {
    type Err = FromStrError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.parse::<f32>()
            .map_err(FromStrError::ParseFloatError)?
            .try_into()
            .map_err(FromStrError::Conversion)
    }
}
