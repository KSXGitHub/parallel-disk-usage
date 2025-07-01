use crate::size;
use derive_more::From;

/// Information to return from `get_info` of [`super::TreeBuilder`].
#[derive(Debug, Default, From)]
pub struct Info<NameIter, Size: size::Size> {
    /// Size associated with given `path`.
    pub size: Size,
    /// Direct descendants of given `path`.
    pub children: NameIter,
}
