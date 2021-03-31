use derive_more::{Add, AddAssign, From, Into, Sum};
use std::{
    fmt::Debug,
    iter::Sum,
    ops::{Add, AddAssign},
};

/// Types whose values can be used as disk usage statistic
pub trait Size: Debug + Default + Clone + Copy + PartialEq + Eq + Add + AddAssign + Sum {
    /// Underlying type
    type Inner: From<Self> + Into<Self>;
}

macro_rules! newtype {
    ($name:ident = $inner:ty | $doc:literal) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, From, Into, Add, AddAssign, Sum)]
        #[doc = $doc]
        pub struct $name($inner);

        impl Size for $name {
            type Inner = $inner;
        }
    };
}

newtype!(Bytes = u64 | "Number of bytes");
newtype!(Blocks = u64 | "Number of blocks");
