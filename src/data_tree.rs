pub mod reflection;

pub use reflection::Reflection;

pub use Reflection as DataTreeReflection;

use super::size::Size;

/// Disk usage data of a filesystem tree.
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
