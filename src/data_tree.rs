pub mod reflection;

pub use reflection::Reflection;

pub use Reflection as DataTreeReflection;

use super::size::Size;

/// Disk usage data of a filesystem tree.
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
