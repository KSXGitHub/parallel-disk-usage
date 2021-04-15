use derive_more::{Add, AddAssign, From, Into, Sum};
use std::{
    fmt::Debug,
    iter::Sum,
    ops::{Add, AddAssign},
};

/// Types whose values can be used as disk usage statistic.
pub trait Size:
    Debug + Default + Clone + Copy + PartialEq + Eq + Add<Output = Self> + AddAssign + Sum
{
    /// Underlying type
    type Inner: From<Self> + Into<Self>;
}

macro_rules! newtype {
    ($(#[$attribute:meta])* $name:ident = $inner:ty) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, From, Into, Add, AddAssign, Sum)]
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
