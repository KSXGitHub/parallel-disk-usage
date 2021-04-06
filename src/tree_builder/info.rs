use super::Size;
use derive_more::From;

/// Information to return from `get_info` of [`super::TreeBuilder`]
#[derive(Debug, From)]
pub struct Info<Id, Data: Size> {
    /// Data associated with given `id`
    pub data: Data,
    /// Direct descendants of given `id`
    pub children: Vec<Id>,
}
