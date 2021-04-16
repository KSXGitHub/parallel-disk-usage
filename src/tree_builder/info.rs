use super::Size;
use derive_more::From;
use smart_default::SmartDefault;

/// Information to return from `get_info` of [`super::TreeBuilder`].
#[derive(Debug, SmartDefault, From)]
pub struct Info<Name, Data: Size> {
    /// Data associated with given `path`.
    pub data: Data,
    /// Direct descendants of given `path`.
    pub children: Vec<Name>,
}
