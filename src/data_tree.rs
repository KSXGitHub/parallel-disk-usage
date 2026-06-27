#![cfg_attr(
    dylint_lib = "perfectionist",
    expect(
        perfectionist::import_grouping_mismatch,
        reason = "pub use re-exports are kept in their own group; see #442"
    )
)]

pub mod reflection;

pub use Reflection as DataTreeReflection;
pub use reflection::Reflection;

use super::size;

/// Disk usage data of a filesystem tree.
///
/// **Construction:** There are 3 main ways to create a [`DataTree`]:
/// * Use [`FsTreeBuilder`](crate::fs_tree_builder::FsTreeBuilder) to create it from the real
///   filesystem.
/// * Use [`TreeBuilder`](crate::tree_builder::TreeBuilder) to create it from a representation
///   of a filesystem.
/// * Use [`Reflection`].
///
/// **Visualization:** Use the [`Visualizer`](crate::visualizer::Visualizer) struct to create an
/// ASCII chart that visualizes [`DataTree`].
///
/// **Serialization and deserialization:** _(feature: `json`)_ [`DataTree`] does not implement
/// `Serialize` and `Deserialize` traits directly, instead, it can be converted into/from a
/// [`Reflection`] which implements these traits.
#[derive(Debug, PartialEq, Eq)]
pub struct DataTree<Name, Size: size::Size> {
    name: Name,
    size: Size,
    children: Vec<Self>,
}

mod constructors;
mod getters;
mod retain;
mod sort;

#[cfg(unix)] // for now, it is only available on unix
mod hardlink;
