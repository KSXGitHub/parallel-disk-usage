use super::measurement_system::{Binary, MeasurementSystem, Metric, ParsedValue};
use derive_more::{Add, AddAssign, From, Into, Sum};
use std::{
    fmt::{Debug, Display, Error, Formatter},
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
    + Display
{
    /// Underlying type
    type Inner: From<Self> + Into<Self> + Mul<Self, Output = Self>;
    /// Return type of [`display`](Size::display).
    type Display: Display;
    /// Display the disk usage in a measurement system.
    fn display(self) -> Self::Display;
}

macro_rules! newtype {
    (
        $(#[$attribute:meta])*
        $name:ident = $inner:ty;
        display -> $display_type:ty = $display_impl:expr;
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
            type Display = $display_type;
            fn display(self) -> Self::Display {
                let display: fn(Self) -> Self::Display = $display_impl;
                display(self)
            }
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
                write!(formatter, "{}", self.display())
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

newtype!(
    #[doc = "Number of bytes (display in metric units)."]
    MetricBytes = u64;
    display -> ParsedValue = |bytes| {
        Metric::parse_value(bytes.inner())
    };
);
newtype!(
    #[doc = "Number of bytes (display in binary units)."]
    BinaryBytes = u64;
    display -> ParsedValue = |bytes| {
        Binary::parse_value(bytes.inner())
    };
);
newtype!(
    #[doc = "Number of blocks."]
    Blocks = u64;
    display -> u64 = |blocks| blocks.inner();
);

/// Number of bytes
pub trait Bytes: Size<Inner = u64> + From<u64> + Into<u64> {
    /// Set displaying unit to metric system.
    fn into_metric_bytes(self) -> MetricBytes {
        self.into().into()
    }

    /// Set displaying unit to binary system.
    fn into_binary_bytes(self) -> BinaryBytes {
        self.into().into()
    }
}

impl Bytes for MetricBytes {}
impl Bytes for BinaryBytes {}
