pub mod info;

pub use info::Info;

use super::{size::Size, tree::Tree};
use rayon::prelude::*;

/// Collection of functions and starting points in order to build a [`Tree`] with [`From`] or [`Into`].
#[derive(Debug)]
pub struct TreeBuilder<Id, Data, GetInfo, JoinPath>
where
    Id: Send + Sync,
    Data: Size + Send,
    GetInfo: Fn(&Id) -> Info<Id, Data> + Copy + Send + Sync,
    JoinPath: Fn(&Id, &Id) -> Id + Copy + Send + Sync,
{
    /// Root identification.
    pub id: Id,
    /// Function to extract necessary information from `id` (`data` and `children`).
    pub get_info: GetInfo,
    /// Function to join parent's `id` with a child's name to make the child's `id`.
    pub join_path: JoinPath,
}

impl<Id, Data, GetInfo, JoinPath> From<TreeBuilder<Id, Data, GetInfo, JoinPath>> for Tree<Id, Data>
where
    Id: Send + Sync,
    Data: Size + Send,
    GetInfo: Fn(&Id) -> Info<Id, Data> + Copy + Send + Sync,
    JoinPath: Fn(&Id, &Id) -> Id + Copy + Send + Sync,
{
    fn from(builder: TreeBuilder<Id, Data, GetInfo, JoinPath>) -> Self {
        let TreeBuilder {
            id,
            get_info,
            join_path,
        } = builder;

        let Info { data, children } = get_info(&id);

        let children: Vec<_> = children
            .into_par_iter()
            .map(|child_name| TreeBuilder {
                id: join_path(&id, &child_name),
                get_info,
                join_path,
            })
            .map(Into::<Self>::into)
            .collect();

        Tree::from_children(id, children).add_dir_size(data)
    }
}
