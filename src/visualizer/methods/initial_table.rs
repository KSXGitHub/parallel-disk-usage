use super::{NodeInfo, Table, BORDER_COLUMNS, PERCENTAGE_COLUMN_MAX_WIDTH};
use crate::{data_tree::DataTree, size, visualizer::Visualizer};
use assert_cmp::debug_assert_op;
use derive_more::{Deref, DerefMut};
use std::{cmp::max, fmt::Display, num::NonZeroUsize};

#[derive(Deref, DerefMut)]
pub(super) struct InitialRow<Name, NodeData> {
    #[deref]
    #[deref_mut]
    pub(super) node_info: NodeInfo<Name, NodeData>,
    pub(super) ancestors: Vec<NodeInfo<Name, NodeData>>,
    pub(super) preceding_sibling: Option<NodeInfo<Name, NodeData>>,
    pub(super) size: String,
    pub(super) percentage: String,
}

impl<Name, NodeData> InitialRow<Name, NodeData> {
    #[inline]
    pub(super) fn parent(&self) -> Option<&'_ NodeInfo<Name, NodeData>> {
        self.ancestors.last()
    }
}

#[derive(Default, Clone, Copy)]
pub(super) struct InitialColumnWidth {
    pub(super) size_column_width: usize,
}

impl InitialColumnWidth {
    #[inline]
    pub(super) const fn total_max_width(self) -> usize {
        self.size_column_width + PERCENTAGE_COLUMN_MAX_WIDTH + BORDER_COLUMNS
    }
}

pub(super) type InitialTable<Name, NodeData> =
    Table<InitialRow<Name, NodeData>, InitialColumnWidth>;

pub(super) fn render_initial<Name, Size>(
    visualizer: Visualizer<Name, Size>,
) -> InitialTable<&'_ Name, Size>
where
    Name: Display,
    Size: size::Size + Into<u64>,
{
    #[derive(Clone)]
    struct Param<Name, NodeData> {
        index_as_child: usize,
        ancestors: Vec<NodeInfo<Name, NodeData>>,
        preceding_sibling: Option<NodeInfo<Name, NodeData>>,
    }

    struct ActResult<Name, NodeData> {
        node_info: NodeInfo<Name, NodeData>,
    }

    struct TraverseResult<Name, NodeData> {
        node_info: NodeInfo<Name, NodeData>,
    }

    fn traverse<'a, Name, Size, Act>(
        tree: &'a DataTree<Name, Size>,
        act: &mut Act,
        param: Param<&'a Name, Size>,
    ) -> Option<TraverseResult<&'a Name, Size>>
    where
        Size: size::Size,
        Act: FnMut(&'a DataTree<Name, Size>, Param<&'a Name, Size>) -> ActResult<&'a Name, Size>,
    {
        let ActResult { node_info } = act(tree, param.clone());
        let mut preceding_sibling = None;
        for (index_as_child, child) in tree.children().iter().enumerate() {
            let mut ancestors = Vec::with_capacity(param.ancestors.len() + 1);
            ancestors.clone_from(&param.ancestors);
            ancestors.push(node_info);
            let traverse_result = traverse(
                child,
                act,
                Param {
                    index_as_child,
                    ancestors,
                    preceding_sibling,
                },
            );
            preceding_sibling = traverse_result.map(|x| x.node_info);
        }
        Some(TraverseResult { node_info })
    }

    let mut initial_table = InitialTable::default();
    let total_fs_size = visualizer.data_tree.size().into();

    traverse(
        visualizer.data_tree,
        &mut |node, param| {
            let Param {
                index_as_child,
                ancestors,
                preceding_sibling,
            } = param;
            let name = node.name();
            let node_data = node.size();
            let row_index = initial_table.len();
            let children_count = node.children().len();
            let fs_size = node.size().into();
            let percentage = if total_fs_size == 0 {
                "0%".to_string()
            } else {
                let percentage = rounded_div::u64(fs_size * 100, total_fs_size);
                format!("{percentage}%")
            };
            let size = node.size().display(visualizer.bytes_format).to_string();
            let sibling_count = ancestors.last().map_or(1, |parent| parent.children_count);
            debug_assert_op!(sibling_count != 0);
            debug_assert_op!(index_as_child < sibling_count);
            let sibling_count = unsafe { NonZeroUsize::new_unchecked(sibling_count) };
            let node_info = NodeInfo {
                name,
                node_data,
                row_index,
                sibling_count,
                index_as_child,
                children_count,
            };

            initial_table.column_width.size_column_width =
                max(initial_table.column_width.size_column_width, size.len());

            initial_table.push_back(InitialRow {
                node_info,
                ancestors,
                preceding_sibling,
                size,
                percentage,
            });

            ActResult { node_info }
        },
        Param {
            index_as_child: 0,
            ancestors: Vec::new(),
            preceding_sibling: None,
        },
    );

    initial_table
}
