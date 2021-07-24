pub mod reflection;

pub use reflection::Reflection;

pub use Reflection as DataTreeReflection;

use super::size::Size;

/// Disk usage data of a filesystem tree.
///
/// **Construction:** There are 3 main ways to create a `DataTree`:
/// * Use [`FsTreeBuilder`](crate::fs_tree_builder::FsTreeBuilder) to create it from the real
/// filesystem.
/// * Use [`TreeBuilder`](crate::tree_builder::TreeBuilder) to create it from a representation
/// of a filesystem.
/// * Use [`Reflection`].
///
/// **Visualization:** Use the [`Visualizer`](crate::visualizer::Visualizer) struct to create an
/// ASCII chart that visualizes `DataTree`.
///
/// **Serialization and deserialization:** _(feature: `json`)_ `DataTree` does not implement
/// `Serialize` and `Deserialize` traits directly, instead, it can be converted into/from a
/// [`Reflection`] which implements these traits.
#[derive(Debug, PartialEq, Eq)]
pub struct DataTree<Name, Data: Size> {
    name: Name,
    data: Data,
    children: Vec<Self>,
}

mod constructors;
mod getters;
mod retain;
mod sort;
