use derive_more::{Add, AddAssign, From, Into, Sum};
use std::{
    fmt::Debug,
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
}

macro_rules! newtype {
    ($(#[$attribute:meta])* $name:ident = $inner:ty) => {
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
    Bytes = u64
);
newtype!(
    #[doc = "Number of blocks."]
    Blocks = u64
);
