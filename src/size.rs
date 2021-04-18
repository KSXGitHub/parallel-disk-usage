use super::measurement_system::{MeasurementSystem, ParsedValue};
use derive_more::{Add, AddAssign, From, Into, Sum};
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
    /// Return type of [`display`](Size::display).
    type Display: Display;
    /// Display the disk usage in a measurement system.
    fn display(self, measurement_system: MeasurementSystem) -> Self::Display;
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
            fn display(self, measurement_system: MeasurementSystem) -> Self::Display {
                let display: fn(Self, MeasurementSystem) -> Self::Display = $display_impl;
                display(self, measurement_system)
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
    #[doc = "Number of bytes."]
    Bytes = u64;
    display -> ParsedValue = |bytes, measurement_system| {
        measurement_system.parse_value(bytes.inner())
    };
);
newtype!(
    #[doc = "Number of blocks."]
    Blocks = u64;
    display -> u64 = |blocks, _| blocks.inner();
);
