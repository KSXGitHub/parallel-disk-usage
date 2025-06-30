use crate::size;
use derive_more::From;
use smart_default::SmartDefault;

/// Information to return from `get_info` of [`super::TreeBuilder`].
#[derive(Debug, SmartDefault, From)]
pub struct Info<Name, Size: size::Size> {
    /// Size associated with given `path`.
    pub size: Size,
    /// Direct descendants of given `path`.
    pub children: Vec<Name>, // TODO: change this into an iterator
}
