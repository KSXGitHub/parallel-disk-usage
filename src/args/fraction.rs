use clap::builder::{TypedValueParser, ValueParserFactory};
use derive_more::{AsRef, Deref, Display, Into};
use std::{
    convert::{TryFrom, TryInto},
    num::ParseFloatError,
    str::FromStr,
};

/// Floating-point value that is greater than or equal to 0 and less than 1.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, AsRef, Deref, Display, Into)]
pub struct Fraction(f32);

/// Error that occurs when calling [`Fraction::new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum ConversionError {
    /// Provided value is greater than or equal to 1.
    #[display(fmt = "greater than or equal to 1")]
    UpperBound,
    /// Provided value is less than 0.
    #[display(fmt = "less than 0")]
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

#[derive(Debug, Clone, PartialEq, Eq, Display)]
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

impl ValueParserFactory for Fraction {
    type Parser = FractionValueParser;
    fn value_parser() -> Self::Parser {
        FractionValueParser
    }
}

/// The [parser](ValueParserFactory::Parser) of [`Fraction`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FractionValueParser;

impl TypedValueParser for FractionValueParser {
    type Value = Fraction;

    fn parse_ref(
        &self,
        _: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let value = value
            .to_str()
            .ok_or_else(|| clap::Error::raw(clap::ErrorKind::InvalidUtf8, "Invalid UTF-8"))?;
        value.parse().map_err(|error| {
            clap::Error::raw(
                clap::ErrorKind::ValueValidation,
                format_args!(
                    "Invalid value {value:?} for '{arg}': {error}\n\nFor more information try --help\n",
                    arg = arg.map_or_else(|| "...".to_string(), |arg| arg.to_string()),
                ),
            )
        })
    }
}
