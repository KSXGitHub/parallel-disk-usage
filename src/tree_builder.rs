pub mod info;

pub use info::Info;

use super::{data_tree::DataTree, size::Size};
use rayon::prelude::*;

/// Collection of functions and starting points in order to build a [`DataTree`] with [`From`] or [`Into`].
#[derive(Debug)]
pub struct TreeBuilder<Path, Name, Data, GetInfo, JoinPath, PostProcessChildren>
where
    Path: Send + Sync,
    Name: Send + Sync,
    Data: Size + Send,
    GetInfo: Fn(&Path) -> Info<Name, Data> + Copy + Send + Sync,
    JoinPath: Fn(&Path, &Name) -> Path + Copy + Send + Sync,
    PostProcessChildren: Fn(&mut Vec<DataTree<Name, Data>>) + Copy + Send + Sync,
{
    /// Path to the root.
    pub path: Path,
    /// Name of the root.
    pub name: Name,
    /// Function to extract necessary information from `path` (`data` and `children`).
    pub get_info: GetInfo,
    /// Function to join parent's `path` with a child's name to make the child's `name`.
    pub join_path: JoinPath,
    /// Function to process each list of children after forming.
    pub post_process_children: PostProcessChildren,
}

impl<Path, Name, Data, GetInfo, JoinPath, PostProcessChildren>
    From<TreeBuilder<Path, Name, Data, GetInfo, JoinPath, PostProcessChildren>>
    for DataTree<Name, Data>
where
    Path: Send + Sync,
    Name: Send + Sync,
    Data: Size + Send,
    GetInfo: Fn(&Path) -> Info<Name, Data> + Copy + Send + Sync,
    JoinPath: Fn(&Path, &Name) -> Path + Copy + Send + Sync,
    PostProcessChildren: Fn(&mut Vec<DataTree<Name, Data>>) + Copy + Send + Sync,
{
    fn from(
        builder: TreeBuilder<Path, Name, Data, GetInfo, JoinPath, PostProcessChildren>,
    ) -> Self {
        let TreeBuilder {
            path,
            name,
            get_info,
            join_path,
            post_process_children,
        } = builder;

        let Info { data, children } = get_info(&path);

        let mut children: Vec<_> = children
            .into_par_iter()
            .map(|name| TreeBuilder {
                path: join_path(&path, &name),
                name,
                get_info,
                join_path,
                post_process_children,
            })
            .map(Self::from)
            .collect();

        post_process_children(&mut children);

        DataTree::dir(name, data, children)
    }
}
