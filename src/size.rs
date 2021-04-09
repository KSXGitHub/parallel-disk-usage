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
    ($(#[$attribute:meta])* $name:ident = $inner:ty) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, From, Into, Add, AddAssign, Sum)]
        $(#[$attribute])*
        pub struct $name($inner);

        impl Size for $name {
            type Inner = $inner;
        }
    };
}

newtype!(
    #[doc = "Number of bytes"]
    Bytes = u64
);
newtype!(
    #[doc = "Number of blocks"]
    Blocks = u64
);
