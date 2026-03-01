use crate::size;
use derive_more::From;
use smart_default::SmartDefault;

/// Information to return from `get_info` of [`super::build_data_tree`].
#[derive(Debug, SmartDefault, From)]
pub struct Info<Name, Size: size::Size> {
    /// Size associated with given `path`.
    pub size: Size,
    /// Direct descendants of given `path`.
    pub children: Vec<Name>,
}
