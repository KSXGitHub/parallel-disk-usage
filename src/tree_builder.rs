pub mod info;

pub use info::Info;

use super::{data_tree::DataTree, size};
use rayon::prelude::*;

/// Collection of functions and starting points in order to build a [`DataTree`] with [`From`] or [`Into`].
#[derive(Debug)]
pub struct TreeBuilder<Path, Name, Size, GetInfo, JoinPath>
where
    Path: Send + Sync,
    Name: Send + Sync,
    GetInfo: Fn(&Path) -> Info<Name, Size> + Copy + Send + Sync,
    JoinPath: Fn(&Path, &Name) -> Path + Copy + Send + Sync,
    Size: size::Size + Send,
{
    /// Path to the root.
    pub path: Path,
    /// Name of the root.
    pub name: Name,
    /// Function to extract necessary information from `path` (`size` and `children`).
    pub get_info: GetInfo,
    /// Function to join parent's `path` with a child's name to make the child's `name`.
    pub join_path: JoinPath,
    /// Deepest level of descendent to store as arrays. The sizes beyond the max depth still count toward total.
    pub max_depth: u64,
}

impl<Path, Name, Size, GetInfo, JoinPath> From<TreeBuilder<Path, Name, Size, GetInfo, JoinPath>>
    for DataTree<Name, Size>
where
    Path: Send + Sync,
    Name: Send + Sync,
    GetInfo: Fn(&Path) -> Info<Name, Size> + Copy + Send + Sync,
    JoinPath: Fn(&Path, &Name) -> Path + Copy + Send + Sync,
    Size: size::Size + Send,
{
    /// Create a [`DataTree`] from a [`TreeBuilder`].
    fn from(builder: TreeBuilder<Path, Name, Size, GetInfo, JoinPath>) -> Self {
        let TreeBuilder {
            path,
            name,
            get_info,
            join_path,
            max_depth,
        } = builder;

        let Info { size, children } = get_info(&path);
        let max_depth = max_depth.saturating_sub(1);

        let children: Vec<_> = children
            .into_par_iter()
            .map(|name| TreeBuilder {
                path: join_path(&path, &name),
                name,
                get_info,
                join_path,
                max_depth,
            })
            .map(Self::from)
            .collect();

        DataTree::dir(name, size, children, max_depth)
    }
}
