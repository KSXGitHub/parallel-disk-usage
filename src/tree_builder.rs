pub mod info;

pub use info::Info;

use super::{data_tree::DataTree, size};
use bon::builder;
use rayon::prelude::*;

/// Create a [`DataTree`] from a representation of a filesystem.
#[builder]
pub fn build_data_tree<Path, Name, Size, GetInfo, JoinPath>(
    /// Path to the root.
    path: Path,
    /// Name of the root.
    name: Name,
    /// Function to extract necessary information from `path` (`size` and `children`).
    get_info: GetInfo,
    /// Function to join parent's `path` with a child's name to make the child's `name`.
    join_path: JoinPath,
    /// Deepest level of descendant to store as arrays. The sizes beyond the max depth still count toward total.
    max_depth: u64,
) -> DataTree<Name, Size>
where
    Path: Send + Sync,
    Name: Send + Sync,
    GetInfo: Fn(&Path) -> Info<Name, Size> + Copy + Send + Sync,
    JoinPath: Fn(&Path, &Name) -> Path + Copy + Send + Sync,
    Size: size::Size + Send,
{
    let Info { size, children } = get_info(&path);
    let max_depth = max_depth.saturating_sub(1);

    let children = children
        .into_par_iter()
        .map(|name| {
            build_data_tree()
                .path(join_path(&path, &name))
                .name(name)
                .get_info(get_info)
                .join_path(join_path)
                .max_depth(max_depth)
                .call()
        });

    if max_depth > 0 {
        DataTree::dir(name, size, children.collect())
    } else {
        let size = size + children.map(|child| child.size()).sum();
        DataTree::dir(name, size, Vec::new())
    }
}
