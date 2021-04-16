pub mod info;

pub use info::Info;

use super::{size::Size, tree::Tree};
use rayon::prelude::*;

/// Collection of functions and starting points in order to build a [`Tree`] with [`From`] or [`Into`].
#[derive(Debug)]
pub struct TreeBuilder<Path, Name, Data, GetInfo, JoinPath>
where
    Path: Send + Sync,
    Name: Send + Sync,
    Data: Size + Send,
    GetInfo: Fn(&Path) -> Info<Name, Data> + Copy + Send + Sync,
    JoinPath: Fn(&Path, &Name) -> Path + Copy + Send + Sync,
{
    /// Path to the root.
    pub path: Path,
    /// Name of the root.
    pub name: Name,
    /// Function to extract necessary information from `path` (`data` and `children`).
    pub get_info: GetInfo,
    /// Function to join parent's `path` with a child's name to make the child's `name`.
    pub join_path: JoinPath,
}

impl<Path, Name, Data, GetInfo, JoinPath> From<TreeBuilder<Path, Name, Data, GetInfo, JoinPath>>
    for Tree<Name, Data>
where
    Path: Send + Sync,
    Name: Send + Sync,
    Data: Size + Send,
    GetInfo: Fn(&Path) -> Info<Name, Data> + Copy + Send + Sync,
    JoinPath: Fn(&Path, &Name) -> Path + Copy + Send + Sync,
{
    fn from(builder: TreeBuilder<Path, Name, Data, GetInfo, JoinPath>) -> Self {
        let TreeBuilder {
            path,
            name,
            get_info,
            join_path,
        } = builder;

        let Info { data, children } = get_info(&path);

        let children: Vec<_> = children
            .into_par_iter()
            .map(|name| TreeBuilder {
                path: join_path(&path, &name),
                name,
                get_info,
                join_path,
            })
            .map(Into::<Self>::into)
            .collect();

        Tree::from_children(name, children).add_dir_size(data)
    }
}

#[cfg(test)]
mod test;
