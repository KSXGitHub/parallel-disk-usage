pub mod info;

pub use info::Info;

use super::{size::Size, tree::Tree};
use rayon::prelude::*;

/// Collection of functions and starting points in order to build a [`Tree`] with [`From`] or [`Into`].
#[derive(Debug)]
pub struct TreeBuilder<Id, Name, Data, GetInfo, JoinPath>
where
    Id: Send + Sync,
    Name: Send + Sync,
    Data: Size + Send,
    GetInfo: Fn(&Id) -> Info<Name, Data> + Copy + Send + Sync,
    JoinPath: Fn(&Id, &Name) -> Id + Copy + Send + Sync,
{
    /// Root identification.
    pub id: Id,
    /// Root name.
    pub name: Name,
    /// Function to extract necessary information from `id` (`data` and `children`).
    pub get_info: GetInfo,
    /// Function to join parent's `id` with a child's name to make the child's `id`.
    pub join_path: JoinPath,
}

impl<Id, Name, Data, GetInfo, JoinPath> From<TreeBuilder<Id, Name, Data, GetInfo, JoinPath>>
    for Tree<Name, Data>
where
    Id: Send + Sync,
    Name: Send + Sync,
    Data: Size + Send,
    GetInfo: Fn(&Id) -> Info<Name, Data> + Copy + Send + Sync,
    JoinPath: Fn(&Id, &Name) -> Id + Copy + Send + Sync,
{
    fn from(builder: TreeBuilder<Id, Name, Data, GetInfo, JoinPath>) -> Self {
        let TreeBuilder {
            id,
            name,
            get_info,
            join_path,
        } = builder;

        let Info { data, children } = get_info(&id);

        let children: Vec<_> = children
            .into_par_iter()
            .map(|name| TreeBuilder {
                id: join_path(&id, &name),
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
