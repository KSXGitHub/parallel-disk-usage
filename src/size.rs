use super::measurement_system::{Binary, MeasurementSystem, Metric, ParsedValue};
use derive_more::{Add, AddAssign, Display, From, Into, Sum};
use pipe_trait::Pipe;
use std::{
    fmt::{Debug, Display},
    iter::Sum,
    ops::{Add, AddAssign, Mul, MulAssign},
};

/// Types whose values can be used as disk usage statistic.
pub trait Size:
    Debug
    + Default
    + Clone
    + Copy
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + Add<Output = Self>
    + AddAssign
    + Sum
{
    /// Underlying type
    type Inner: From<Self> + Into<Self> + Mul<Self, Output = Self>;
    /// Format to be used to [`display`](Size::display) the value.
    type DisplayFormat: Copy;
    /// Return type of [`display`](Size::display).
    type DisplayOutput: Display;
    /// Display the disk usage in a measurement system.
    fn display(self, input: Self::DisplayFormat) -> Self::DisplayOutput;
}

macro_rules! newtype {
    (
        $(#[$attribute:meta])*
        $name:ident = $inner:ty;
        display: ($display_format:ty) -> $display_output:ty = $display_impl:expr;
    ) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        #[derive(From, Into, Add, AddAssign, Sum)]
        $(#[$attribute])*
        pub struct $name($inner);

        impl $name {
            pub const fn new(inner: $inner) -> Self {
                $name(inner)
            }

            pub const fn inner(self) -> $inner {
                self.0
            }
        }

        impl Size for $name {
            type Inner = $inner;
            type DisplayFormat = $display_format;
            type DisplayOutput = $display_output;
            fn display(self, format: Self::DisplayFormat) -> Self::DisplayOutput {
                let display: fn(Self, Self::DisplayFormat) -> Self::DisplayOutput = $display_impl;
                display(self, format)
            }
        }

        impl Mul<$inner> for $name {
            type Output = Self;
            fn mul(self, rhs: $inner) -> Self::Output {
                self.0.mul(rhs).into()
            }
        }

        impl Mul<$name> for $inner {
            type Output = $name;
            fn mul(self, rhs: $name) -> Self::Output {
                rhs * self
            }
        }

        impl MulAssign<$inner> for $name {
            fn mul_assign(&mut self, rhs: $inner) {
                self.0 *= rhs;
            }
        }
    };
}

/// The [`DisplayFormat`](Size::DisplayFormat) type of [`Bytes`].
#[derive(Debug, Clone, Copy)]
pub enum BytesDisplayFormat {
    /// Display the value as-is.
    PlainNumber,
    /// Display the value with a unit suffix in [metric scale](Metric).
    MetricUnits,
    /// Display the value with a unit suffix in [binary scale](Binary).
    BinaryUnits,
}

/// The [`DisplayOutput`](Size::DisplayOutput) type of [`Bytes`].
#[derive(Debug, Display, Clone, Copy)]
pub enum BytesDisplayOutput {
    /// Display the value as-is.
    PlainNumber(u64),
    /// Display the value with unit a suffix.
    Units(ParsedValue),
}

newtype!(
    #[doc = "Number of bytes."]
    Bytes = u64;
    display: (BytesDisplayFormat) -> BytesDisplayOutput = |bytes, format| {
        let value = bytes.inner();
        match format {
            BytesDisplayFormat::PlainNumber => BytesDisplayOutput::PlainNumber(value),
            BytesDisplayFormat::MetricUnits => {
                value.pipe(Metric::parse_value).pipe(BytesDisplayOutput::Units)
            }
            BytesDisplayFormat::BinaryUnits => {
                value.pipe(Binary::parse_value).pipe(BytesDisplayOutput::Units)
            }
        }
    };
);

newtype!(
    #[doc = "Number of blocks."]
    Blocks = u64;
    display: (()) -> u64 = |blocks, ()| blocks.inner();
);
