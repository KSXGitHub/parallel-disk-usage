pub mod info;

pub use info::Info;

use super::{data_tree::DataTree, size};
use pipe_trait::Pipe;
use rayon::prelude::*;
use std::{marker::PhantomData, ops::Not};

/// Collection of functions and starting points in order to build a [`DataTree`] with [`From`] or [`Into`].
#[derive(Debug)]
pub struct TreeBuilder<Path, NameIter, Size, GetInfo, JoinPath>
where
    Path: Send + Sync,
    NameIter: IntoIterator,
    NameIter::IntoIter: Send,
    NameIter::Item: Send,
    GetInfo: Fn(&Path) -> Info<NameIter, Size> + Copy + Send + Sync,
    JoinPath: Fn(&Path, &NameIter::Item) -> Path + Copy + Send + Sync,
    Size: size::Size + Send,
{
    /// Path to the root.
    pub path: Path,
    /// Name of the root.
    pub name: NameIter::Item,
    /// Function to extract necessary information from `path` (`size` and `children`).
    pub get_info: GetInfo,
    /// Function to join parent's `path` with a child's name to make the child's `name`.
    pub join_path: JoinPath,
    /// Deepest level of descendent to store as arrays. The sizes beyond the max depth still count toward total.
    pub max_depth: u64,
}

impl<Path, NameIter, Size, GetInfo, JoinPath>
    From<TreeBuilder<Path, NameIter, Size, GetInfo, JoinPath>> for DataTree<NameIter::Item, Size>
where
    Path: Send + Sync,
    NameIter: IntoIterator,
    NameIter::IntoIter: Send,
    NameIter::Item: Send,
    GetInfo: Fn(&Path) -> Info<NameIter, Size> + Copy + Send + Sync,
    JoinPath: Fn(&Path, &NameIter::Item) -> Path + Copy + Send + Sync,
    Size: size::Size + Send,
{
    /// Create a [`DataTree`] from a [`TreeBuilder`].
    fn from(builder: TreeBuilder<Path, NameIter, Size, GetInfo, JoinPath>) -> Self {
        let TreeBuilder {
            path,
            name,
            get_info,
            join_path,
            max_depth,
        } = builder;

        let Info { size, children } = get_info(&path);
        let max_depth = max_depth.saturating_sub(1);

        let children = children
            .into_iter()
            .pipe(Chunks::<64, _>::new)
            .map(|names: Vec<_>| -> Vec<_> {
                names
                    .into_par_iter()
                    .map(|name| TreeBuilder {
                        path: join_path(&path, &name),
                        name,
                        get_info,
                        join_path,
                        max_depth,
                    })
                    .map(Self::from)
                    .collect()
            })
            .fold(Vec::new(), |mut a, b| {
                a.extend(b);
                a
            });

        if max_depth > 0 {
            DataTree::dir(name, size, children)
        } else {
            let size = size + children.into_iter().map(|child| child.size()).sum();
            DataTree::dir(name, size, Vec::new())
        }
    }
}

/// Utility type to iterate over each `Vec` of at most `LEN` items.
#[derive(Debug, Clone, Copy)]
struct Chunks<const LEN: usize, Iter> {
    iter: Iter,
    _phantom: PhantomData<[(); LEN]>,
}

impl<const LEN: usize, Iter> Chunks<LEN, Iter> {
    /// Start iterating chunks.
    fn new(iter: Iter) -> Self {
        Chunks {
            iter,
            _phantom: PhantomData,
        }
    }
}

impl<const LEN: usize, Iter> Iterator for Chunks<LEN, Iter>
where
    Iter: Iterator,
{
    type Item = Vec<Iter::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        let chunk: Vec<_> = self.iter.by_ref().take(LEN).collect();
        chunk.is_empty().not().then_some(chunk)
    }
}
