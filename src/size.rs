use super::bytes_format::{self, BytesFormat};
use derive_more::{Add, AddAssign, From, Into, Sum};
use std::{
    fmt::{Debug, Display},
    iter::Sum,
    ops::{Add, AddAssign, Mul, MulAssign},
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

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

macro_rules! impl_mul {
    ($name:ident: $inner:ident *= $($num_type:ident)+) => {
        $(
            impl Mul<$num_type> for $name {
                type Output = Self;
                fn mul(self, rhs: $num_type) -> Self::Output {
                    self.0.mul(rhs as $inner).into()
                }
            }

            impl Mul<$name> for $num_type {
                type Output = $name;
                fn mul(self, rhs: $name) -> Self::Output {
                    rhs * self
                }
            }

            impl MulAssign<$num_type> for $name {
                fn mul_assign(&mut self, rhs: $num_type) {
                    self.0 *= rhs as $inner;
                }
            }
        )+
    };

    ($name:ident: u64) => {
        impl_mul!($name: u64 *= u8 u16 u32 u64);
    };
}

macro_rules! newtype {
    (
        $(#[$attribute:meta])*
        $name:ident = $inner:ty;
        display: ($display_format:ty) -> $display_output:ty = $display_impl:expr;
    ) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        #[derive(From, Into, Add, AddAssign, Sum)]
        #[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
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

        impl_mul!($name: u64);
    };
}

newtype!(
    /// Number of bytes.
    Bytes = u64;
    display: (BytesFormat) -> bytes_format::Output = |bytes, format| {
        format.format(bytes.into())
    };
);

newtype!(
    /// Number of blocks.
    Blocks = u64;
    display: (()) -> u64 = |blocks, ()| blocks.inner();
);
